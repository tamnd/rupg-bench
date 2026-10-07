//! The query runs that ClickBench and TPC-H share (spec/20 sections 20.4, 20.5 and 20.7).
//!
//! Each query runs `runs` times in a row, and each run is a new session: a new `duckdb` process, or a new connection to the server. Before the first run of each query, a cold run drops the page cache, and for a server that is a systemd unit it also restarts the server. The harness reads the counters of the cgroup around each run. The first run is the cold run and the other runs are the hot runs. The hot time of a query is the minimum of its hot runs.
//!
//! DuckDB runs in a new cgroup for each query, so `memory.peak` is the peak of that query on every kernel. For a server, `memory.peak` cannot be reset on a kernel older than 6.12. After a restart it counts from the restart, because systemd makes a new cgroup for the unit. Without a restart it counts from the start of the server, and the report says so.
//!
//! The answer of a query is the result of its first run. For DuckDB it comes from a separate run that writes the result as CSV, which is not timed and not measured.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::answers::{self, Answer};
use crate::cgroup::{self, Cgroup, Interval, Usage};
use crate::json::Json;
use crate::load;
use crate::pg::{Config, Conn};

/// One query of a suite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Query {
    pub(crate) name: String,
    pub(crate) sql: String,
}

/// The system that runs the queries.
#[derive(Debug)]
pub(crate) enum Target {
    /// The `duckdb` program on a database file.
    DuckDb { bin: String, db: PathBuf, threads: Option<u32>, cpus: Option<String> },
    /// A PostgreSQL protocol server in a cgroup. `unit` is set when the server is a systemd unit that the harness may restart.
    Server { conn: Config, cg: Cgroup, unit: Option<String> },
}

impl Target {
    pub(crate) fn to_json(&self) -> Json {
        match self {
            Target::DuckDb { bin, db, threads, cpus } => Json::obj()
                .with("mode", "in process, a new duckdb process for each run")
                .with("bin", bin.as_str())
                .with("db", db.display().to_string())
                .with("threads", threads.map(u64::from))
                .with("cpus", cpus.clone()),
            Target::Server { conn, cg, unit } => Json::obj()
                .with("mode", "server, a new connection for each run")
                .with("host", conn.host.as_str())
                .with("port", u64::from(conn.port))
                .with("user", conn.user.as_str())
                .with("dbname", conn.dbname.as_str())
                .with("cgroup", cg.path.display().to_string())
                .with("unit", unit.clone()),
        }
    }
}

/// The settings of a suite run.
#[derive(Clone, Debug)]
pub(crate) struct Settings {
    /// Runs of each query, 3 in spec/20.
    pub(crate) runs: usize,
    /// Drop the page cache, and restart a server unit, before the first run of each query.
    pub(crate) cold: bool,
    /// The name prefix of the cgroups for DuckDB.
    pub(crate) cgroup_prefix: String,
}

/// One run of one query.
#[derive(Clone, Debug)]
pub(crate) struct Run {
    /// The time of the query as the client measures it.
    pub(crate) secs: f64,
    pub(crate) usage: Usage,
}

/// The result of one query.
#[derive(Clone, Debug)]
pub(crate) struct QueryResult {
    pub(crate) name: String,
    pub(crate) runs: Vec<Run>,
    /// The answer of the first run, when the query did not fail.
    pub(crate) answer: Option<Answer>,
    pub(crate) error: Option<String>,
    /// The result of the answer check, when there is an expected answer.
    pub(crate) check: Option<Result<(), String>>,
    /// The rule of the check when it is not the comparison of all rows, or why the answer is not compared.
    pub(crate) check_rule: Option<String>,
}

impl QueryResult {
    /// The minimum of the hot runs, or the only run.
    pub(crate) fn hot(&self) -> Option<f64> {
        let hot = if self.runs.len() > 1 { &self.runs[1..] } else { &self.runs[..] };
        hot.iter().map(|r| r.secs).min_by(f64::total_cmp)
    }

    pub(crate) fn cold(&self) -> Option<f64> {
        self.runs.first().map(|r| r.secs)
    }

    /// The maximum `memory.peak` over the runs.
    fn peak(&self) -> u64 {
        self.runs.iter().map(|r| r.usage.memory_peak).max().unwrap_or(0)
    }

    pub(crate) fn to_json(&self) -> Json {
        let hot_cpu: u64 = self.runs.iter().skip(1).map(|r| r.usage.cpu_usec).sum();
        let first = self.runs.first().map(|r| &r.usage);
        Json::obj()
            .with("query", self.name.as_str())
            .with("runs_s", self.runs.iter().map(|r| r.secs).collect::<Vec<f64>>())
            .with("cold_s", self.cold())
            .with("hot_s", self.hot())
            .with("cold_rbytes", first.and_then(|u| u.rbytes))
            .with("cold_cpu_usec", first.map(|u| u.cpu_usec))
            .with("hot_cpu_usec", hot_cpu)
            .with("memory_peak_bytes", self.peak())
            .with("pss_max_bytes", self.runs.iter().map(|r| r.usage.pss_max).max().unwrap_or(0))
            .with("rows", self.answer.as_ref().map(|a| a.rows.len()))
            .with(
                "answer",
                match &self.check {
                    None => "not checked".to_owned(),
                    Some(Ok(())) => "ok".to_owned(),
                    Some(Err(e)) => format!("WRONG: {e}"),
                },
            )
            .with("answer_rule", self.check_rule.clone())
            .with("error", self.error.clone())
    }
}

/// The sums of a suite run. The time sums leave out the queries that failed.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Totals {
    pub(crate) hot: f64,
    pub(crate) cold: f64,
    pub(crate) failed: usize,
    pub(crate) wrong: usize,
    pub(crate) checked: usize,
}

pub(crate) fn totals(results: &[QueryResult]) -> Totals {
    Totals {
        hot: results.iter().filter(|r| r.error.is_none()).filter_map(QueryResult::hot).sum(),
        cold: results.iter().filter(|r| r.error.is_none()).filter_map(QueryResult::cold).sum(),
        failed: results.iter().filter(|r| r.error.is_some()).count(),
        wrong: results.iter().filter(|r| matches!(r.check, Some(Err(_)))).count(),
        checked: results.iter().filter(|r| r.check.is_some()).count(),
    }
}

/// `Run Time (s): real 0.123 user 0.1 sys 0.02`, the line of `.timer on` in the DuckDB shell.
pub(crate) fn parse_duckdb_timer(text: &str) -> Option<f64> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix("Run Time (s): real "))
        .filter_map(|rest| rest.split_whitespace().next()?.parse::<f64>().ok())
        .next_back()
}

/// The query without the `;` at its end, so it can go inside `COPY (...)`.
pub(crate) fn strip_semicolon(sql: &str) -> &str {
    sql.trim_end().trim_end_matches(';').trim_end()
}

/// Runs every query and calls `progress` with a line for each query.
pub(crate) fn run(
    target: &Target,
    queries: &[Query],
    settings: &Settings,
    mut progress: impl FnMut(&str),
) -> Vec<QueryResult> {
    let mut out = Vec::new();
    for q in queries {
        let r = match target {
            Target::DuckDb { bin, db, threads, cpus } => {
                duckdb_query(bin, db, *threads, cpus.as_deref(), q, settings)
            }
            Target::Server { conn, cg, unit } => {
                server_query(conn, cg, unit.as_deref(), q, settings)
            }
        };
        let mut line = format!("{:<6}", q.name);
        for run in &r.runs {
            let _ = write!(line, " {:>9.3}", run.secs);
        }
        if let Some(e) = &r.error {
            let _ = write!(line, "  FAILED: {e}");
        }
        progress(&line);
        out.push(r);
    }
    out
}

pub(crate) fn duckdb_argv(bin: &str, db: &Path, threads: Option<u32>) -> Vec<String> {
    let mut argv = vec![bin.to_owned(), "-readonly".to_owned(), db.display().to_string()];
    if let Some(t) = threads {
        argv.extend(["-cmd".to_owned(), format!("SET threads = {t}")]);
    }
    argv
}

/// The answer of a query from one more `duckdb` run that is not measured. `base` is the program, its options and the database file.
pub(crate) fn duckdb_answer(mut argv: Vec<String>, sql: &str) -> Result<Answer, String> {
    let copy = format!("COPY ({}) TO '/dev/stdout' (FORMAT csv, HEADER)", strip_semicolon(sql));
    argv.extend(["-c".to_owned(), copy]);
    let out = Command::new(&argv[0])
        .args(&argv[1..])
        .output()
        .map_err(|e| format!("{}: {e}", argv[0]))?;
    if !out.status.success() {
        return Err(format!(
            "the answer run failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    answers::parse_csv(&String::from_utf8_lossy(&out.stdout))
}

fn duckdb_query(
    bin: &str,
    db: &Path,
    threads: Option<u32>,
    cpus: Option<&str>,
    q: &Query,
    settings: &Settings,
) -> QueryResult {
    let mut result = QueryResult {
        name: q.name.clone(),
        runs: Vec::new(),
        answer: None,
        error: None,
        check: None,
        check_rule: None,
    };
    let attempt = (|| -> Result<(), String> {
        let cg = Cgroup::create(&format!("{}-{}", settings.cgroup_prefix, q.name), cpus)?;
        let base = duckdb_argv(bin, db, threads);
        for i in 0..settings.runs {
            if i == 0 && settings.cold {
                load::drop_caches()?;
            }
            let mut argv = base.clone();
            argv.extend(
                ["-c", ".timer on", "-c", ".mode trash", "-c", q.sql.as_str()].map(str::to_owned),
            );
            let mut cmd = cg.command(&argv)?;
            let interval = Interval::start(&cg)?;
            let out = cmd.output().map_err(|e| format!("{bin}: {e}"));
            let usage = interval.finish()?;
            let out = out?;
            let text = String::from_utf8_lossy(&out.stdout);
            if !out.status.success() {
                return Err(format!(
                    "duckdb failed: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                ));
            }
            let secs = parse_duckdb_timer(&text).ok_or("duckdb printed no Run Time line")?;
            result.runs.push(Run { secs, usage });
        }
        cg.kill_all()?;
        cg.remove()?;
        // The answer, from one more run that is not measured.
        result.answer = Some(duckdb_answer(base, &q.sql)?);
        Ok(())
    })();
    if let Err(e) = attempt {
        result.error = Some(e);
    }
    result
}

/// Stops the unit, drops the page cache and starts the unit again, then waits until the server accepts a connection.
fn restart_cold(unit: &str, conn: &Config) -> Result<(), String> {
    systemctl("stop", unit)?;
    load::drop_caches()?;
    systemctl("start", unit)?;
    let start = Instant::now();
    loop {
        match Conn::connect(conn) {
            Ok(_) => return Ok(()),
            Err(e) if start.elapsed() > Duration::from_secs(120) => {
                return Err(format!("the server did not come back after a restart: {e}"));
            }
            Err(_) => std::thread::sleep(Duration::from_millis(200)),
        }
    }
}

pub(crate) fn systemctl(verb: &str, unit: &str) -> Result<(), String> {
    let status = Command::new("systemctl")
        .args([verb, unit])
        .status()
        .map_err(|e| format!("systemctl: {e}"))?;
    if status.success() { Ok(()) } else { Err(format!("systemctl {verb} {unit} failed: {status}")) }
}

fn server_query(
    conn: &Config,
    cg: &Cgroup,
    unit: Option<&str>,
    q: &Query,
    settings: &Settings,
) -> QueryResult {
    let mut result = QueryResult {
        name: q.name.clone(),
        runs: Vec::new(),
        answer: None,
        error: None,
        check: None,
        check_rule: None,
    };
    let attempt = (|| -> Result<(), String> {
        let mut cg = Cgroup::attach(&cg.path)?;
        for i in 0..settings.runs {
            if i == 0 && settings.cold {
                match unit {
                    Some(u) => {
                        restart_cold(u, conn)?;
                        cg = Cgroup::of_unit(u)?;
                    }
                    None => load::drop_caches()?,
                }
            }
            let mut c = Conn::connect(conn)?;
            let interval = Interval::start(&cg)?;
            let start = Instant::now();
            let rows = c.query(&q.sql);
            let secs = start.elapsed().as_secs_f64();
            let usage = interval.finish()?;
            let rows = rows?;
            if i == 0 {
                result.answer = Some(Answer { columns: rows.columns, rows: rows.rows });
            }
            result.runs.push(Run { secs, usage });
        }
        Ok(())
    })();
    if let Err(e) = attempt {
        result.error = Some(e);
    }
    result
}

/// Measures the idle base of a target over `duration`: the server cgroup, or for DuckDB nothing, because no DuckDB process runs between the queries.
pub(crate) fn idle_base(target: &Target, duration: Duration) -> Result<Option<Usage>, String> {
    match target {
        Target::DuckDb { .. } => Ok(None),
        Target::Server { cg, .. } => cgroup::idle_base(cg, duration).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duckdb_timer() {
        let text = "Run Time (s): real 0.012 user 0.003563 sys 0.002441\n";
        assert_eq!(parse_duckdb_timer(text), Some(0.012));
        let two = "Run Time (s): real 0.5 user 0 sys 0\nRun Time (s): real 1.25 user 1 sys 0\n";
        assert_eq!(parse_duckdb_timer(two), Some(1.25));
        assert_eq!(parse_duckdb_timer("Error: x\n"), None);
    }

    #[test]
    fn semicolons() {
        assert_eq!(strip_semicolon("select 1;\n"), "select 1");
        assert_eq!(strip_semicolon("select 1"), "select 1");
    }

    fn run(secs: f64) -> Run {
        Run {
            secs,
            usage: Usage {
                wall: Duration::ZERO,
                cpu_usec: 1,
                user_usec: 1,
                system_usec: 0,
                rbytes: None,
                wbytes: None,
                memory_peak: (secs * 10.0) as u64,
                peak_scope: cgroup::PeakScope::Cgroup,
                file_max: 0,
                pss_max: 0,
                current_max: 0,
                samples: 0,
            },
        }
    }

    #[test]
    fn hot_and_cold() {
        let r = QueryResult {
            name: "q1".to_owned(),
            runs: vec![run(3.0), run(2.0), run(1.5)],
            answer: None,
            error: None,
            check: Some(Err("x".to_owned())),
            check_rule: None,
        };
        assert_eq!((r.cold(), r.hot(), r.peak()), (Some(3.0), Some(1.5), 30));
        let one = QueryResult { runs: vec![run(4.0)], check: None, ..r.clone() };
        assert_eq!(one.hot(), Some(4.0));
        let failed = QueryResult { error: Some("x".to_owned()), check: None, ..r.clone() };
        let t = totals(&[r, one, failed]);
        assert_eq!(t, Totals { hot: 5.5, cold: 7.0, failed: 1, wrong: 1, checked: 1 });
    }
}
