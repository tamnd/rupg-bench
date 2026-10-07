//! The instruction count runner of spec/21 section 21.14.
//!
//! The runner counts the instructions retired for each query of a fixed set with `perf stat -e instructions`. For an engine in the process of its client, such as DuckDB or rupg in embedded mode, perf counts the client process. For a server, perf counts the server cgroup on all CPUs (`-a -G`) while the client sends the query, so the count is the work of the server and not of the client. Each query runs `repeat` times and the minimum counts. The count of `SELECT 1` in the same way is the base, and the net count of a query is its count minus the base.
//!
//! `ratchet.toml` holds the net count of each query from the dedicated runner. A change fails when one query rises by more than 3 percent or the total by more than 1 percent.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::json::Json;
use crate::toml::{Doc, Value};

/// A query rises by at most 3 percent.
pub(crate) const QUERY_BUDGET: f64 = 0.03;
/// The total rises by at most 1 percent.
pub(crate) const TOTAL_BUDGET: f64 = 0.01;

/// One query of the set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Query {
    pub(crate) name: String,
    pub(crate) sql: String,
}

/// Reads a set. A file has one query on each line, named `q0`, `q1` and so on, as in `queries.sql` of ClickBench. A directory has one query in each `<name>.sql` file, as `qgen` writes them for TPC-H.
pub(crate) fn read_set(path: &Path) -> Result<Vec<Query>, String> {
    let err = |e: std::io::Error| format!("{}: {e}", path.display());
    if path.is_dir() {
        let mut files: Vec<PathBuf> = std::fs::read_dir(path)
            .map_err(err)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "sql"))
            .collect();
        files.sort_by_key(|p| natural_key(&p.file_stem().unwrap_or_default().to_string_lossy()));
        files
            .iter()
            .map(|p| {
                let sql =
                    std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
                Ok(Query {
                    name: p.file_stem().unwrap_or_default().to_string_lossy().into_owned(),
                    sql,
                })
            })
            .collect()
    } else {
        let text = std::fs::read_to_string(path).map_err(err)?;
        Ok(text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, l)| Query { name: format!("q{i}"), sql: l.trim().to_owned() })
            .collect())
    }
}

/// `q2` sorts before `q10`.
fn natural_key(name: &str) -> (String, u64) {
    let digits = name.trim_start_matches(|c: char| !c.is_ascii_digit());
    let prefix = &name[..name.len() - digits.len()];
    (prefix.to_owned(), digits.parse().unwrap_or(u64::MAX))
}

/// How a query reaches the engine.
#[derive(Clone, Debug)]
pub(crate) enum Engine {
    /// The client program with the engine in its process: argv before the query.
    InProcess(Vec<String>),
    /// A server in a cgroup (the path under `/sys/fs/cgroup`) and its client: argv before the query.
    Server { cgroup: String, client: Vec<String> },
}

impl Engine {
    /// The `perf stat` command for one query. perf writes its counts to `out`.
    fn command(&self, sql: &str, out: &Path) -> Command {
        let mut c = Command::new("perf");
        c.args(["stat", "-e", "instructions", "-x", ","]).arg("-o").arg(out);
        let argv = match self {
            Engine::InProcess(argv) => argv,
            Engine::Server { cgroup, client } => {
                c.args(["-a", "-G", cgroup]);
                client
            }
        };
        c.arg("--").args(argv).arg(sql);
        c.stdout(std::process::Stdio::null());
        c
    }

    pub(crate) fn to_json(&self) -> Json {
        match self {
            Engine::InProcess(argv) => {
                Json::obj().with("mode", "in process").with("client", argv.clone())
            }
            Engine::Server { cgroup, client } => Json::obj()
                .with("mode", "server cgroup")
                .with("cgroup", cgroup.as_str())
                .with("client", client.clone()),
        }
    }
}

/// The instruction count in the CSV output of `perf stat -x ,`. With `-G` perf prints one line for each cgroup, and the counts are added.
pub(crate) fn parse_perf(text: &str) -> Result<u64, String> {
    let mut total = None;
    for line in text.lines() {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 3 || !fields[2].starts_with("instructions") {
            continue;
        }
        let n: u64 =
            fields[0].parse().map_err(|_| format!("perf did not count instructions: {line:?}"))?;
        *total.get_or_insert(0) += n;
    }
    total.ok_or_else(|| format!("perf printed no instruction count: {:?}", text.trim()))
}

/// Runs one query once and returns its count.
fn count_once(engine: &Engine, sql: &str) -> Result<u64, String> {
    let out = std::env::temp_dir().join(format!("rupg-bench-perf-{}.csv", std::process::id()));
    let mut cmd = engine.command(sql, &out);
    let result = cmd.output().map_err(|e| format!("perf: {e}"));
    let text = std::fs::read_to_string(&out).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    let result = result?;
    if !result.status.success() {
        return Err(format!(
            "the query failed: {}\n{}",
            result.status,
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    parse_perf(&text)
}

/// The counts of one query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Count {
    pub(crate) name: String,
    pub(crate) runs: Vec<u64>,
    /// The minimum of the runs minus the base, at least 0.
    pub(crate) net: u64,
}

/// The counts of a set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Counts {
    pub(crate) base: Vec<u64>,
    pub(crate) queries: Vec<Count>,
}

impl Counts {
    pub(crate) fn total(&self) -> u64 {
        self.queries.iter().map(|q| q.net).sum()
    }

    /// The base and the total. The rows of the queries are in [`Counts::rows`].
    pub(crate) fn to_json(&self) -> Json {
        Json::obj()
            .with("base_runs", self.base.clone())
            .with("base", self.base.iter().min().copied().unwrap_or(0))
            .with("total_net", self.total())
    }

    /// One row for each query.
    pub(crate) fn rows(&self) -> Json {
        let rows: Vec<Json> = self
            .queries
            .iter()
            .map(|q| {
                Json::obj()
                    .with("query", q.name.as_str())
                    .with("net", q.net)
                    .with("runs", q.runs.clone())
            })
            .collect();
        Json::from(rows)
    }

    /// A table for `ratchet.toml`.
    pub(crate) fn ratchet_table(&self, set: &str, machine: &str, date: &str) -> String {
        let mut out = format!(
            "[instructions.{set}]\nmachine = \"{machine}\"\ndate = \"{date}\"\ntotal = {}\n",
            self.total()
        );
        for q in &self.queries {
            out.push_str(&format!("{} = {}\n", q.name, q.net));
        }
        out
    }
}

/// Counts each query of the set. `progress` gets one line for each query.
pub(crate) fn count(
    engine: &Engine,
    set: &[Query],
    repeat: usize,
    mut progress: impl FnMut(&str),
) -> Result<Counts, String> {
    let runs = |sql: &str| -> Result<Vec<u64>, String> {
        (0..repeat).map(|_| count_once(engine, sql)).collect()
    };
    let base = runs("SELECT 1")?;
    let floor = base.iter().min().copied().unwrap_or(0);
    let mut queries = Vec::new();
    for q in set {
        let r = runs(&q.sql).map_err(|e| format!("{}: {e}", q.name))?;
        let net = r.iter().min().copied().unwrap_or(0).saturating_sub(floor);
        progress(&format!("{:<8} {net:>16} net instructions, runs {r:?}", q.name));
        queries.push(Count { name: q.name.clone(), runs: r, net });
    }
    Ok(Counts { base, queries })
}

/// Compares the counts with the table `[instructions.<set>]` of a ratchet file. It returns one line for each query and for the total that is over its budget. A query that the ratchet does not have is not compared.
pub(crate) fn compare(counts: &Counts, ratchet: &Doc, set: &str) -> Result<Vec<String>, String> {
    let name = format!("instructions.{set}");
    let table = ratchet.table(&name).ok_or(format!("the ratchet has no table [{name}]"))?;
    let int = |key: &str| match table.get(key) {
        Some(Value::Int(n)) => u64::try_from(*n).ok(),
        _ => None,
    };
    let rise =
        |new: u64, old: u64| if old == 0 { 0.0 } else { (new as f64 - old as f64) / old as f64 };
    let mut over = Vec::new();
    for q in &counts.queries {
        if let Some(old) = int(&q.name) {
            let r = rise(q.net, old);
            if r > QUERY_BUDGET {
                over.push(format!(
                    "{}: {} against {old}, up {:.2} percent, budget 3",
                    q.name,
                    q.net,
                    r * 100.0
                ));
            }
        }
    }
    if let Some(old) = int("total") {
        let r = rise(counts.total(), old);
        if r > TOTAL_BUDGET {
            over.push(format!(
                "total: {} against {old}, up {:.2} percent, budget 1",
                counts.total(),
                r * 100.0
            ));
        }
    }
    Ok(over)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perf_csv() {
        assert_eq!(parse_perf("52133065,,instructions:u,210013041,100.00,,\n"), Ok(52133065));
        let g = "# started on Wed Oct  7\n\n15155433,,instructions,system.slice/a.service,22704897,100.00,,\n";
        assert_eq!(parse_perf(g), Ok(15155433));
        assert!(
            parse_perf("<not supported>,,instructions,,0,100.00,,\n")
                .unwrap_err()
                .contains("did not count")
        );
        assert!(parse_perf("").is_err());
    }

    #[test]
    fn sets() {
        let dir = std::env::temp_dir().join(format!("rupg-bench-set-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (name, sql) in [("q10", "SELECT 10;"), ("q2", "SELECT 2;"), ("notes", "x")] {
            let ext = if name == "notes" { "txt" } else { "sql" };
            std::fs::write(dir.join(format!("{name}.{ext}")), sql).unwrap();
        }
        let set = read_set(&dir).unwrap();
        assert_eq!(set.iter().map(|q| q.name.as_str()).collect::<Vec<_>>(), ["q2", "q10"]);
        let file = dir.join("queries.sql");
        std::fs::write(&file, "SELECT 1;\n\nSELECT 2;\n").unwrap();
        let set = read_set(&file).unwrap();
        assert_eq!(set[1], Query { name: "q1".to_owned(), sql: "SELECT 2;".to_owned() });
        std::fs::remove_dir_all(&dir).unwrap();
    }

    fn counts(nets: &[u64]) -> Counts {
        Counts {
            base: vec![10],
            queries: nets
                .iter()
                .enumerate()
                .map(|(i, n)| Count { name: format!("q{i}"), runs: vec![n + 10], net: *n })
                .collect(),
        }
    }

    #[test]
    fn budgets() {
        let old = counts(&[1000, 2000, 3000]);
        let text = old.ratchet_table("cb", "runner", "2026-10-07");
        assert!(text.starts_with("[instructions.cb]\nmachine = \"runner\"\ndate = \"2026-10-07\"\ntotal = 6000\nq0 = 1000\n"));
        let doc = Doc::parse(&text).unwrap();
        // 2.9 percent on one query and 0.5 percent in total pass.
        assert!(compare(&counts(&[1029, 2000, 3000]), &doc, "cb").unwrap().is_empty());
        // 3.1 percent on one query fails.
        let over = compare(&counts(&[1031, 2000, 3000]), &doc, "cb").unwrap();
        assert_eq!(over.len(), 1);
        assert!(over[0].starts_with("q0: 1031 against 1000, up 3.10 percent"));
        // 2 percent on each query fails the total only.
        let over = compare(&counts(&[1020, 2040, 3060]), &doc, "cb").unwrap();
        assert_eq!(over.len(), 1);
        assert!(over[0].starts_with("total: 6120 against 6000"));
        assert!(compare(&old, &doc, "tpch").is_err());
    }
}
