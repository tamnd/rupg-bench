//! The ClickBench driver (spec/20 section 20.5).
//!
//! The driver runs the scripts of one system directory of the ClickBench pin in the order of `bench_main` and `bench_run_query` of `lib/benchmark-common.sh`: `./load` and a `sync`, then for each line of `queries.sql` the cold cycle and the tries of `./query`, then `./data-size`, then the concurrent test of the pin. It does in Rust what the shell loop does, because the harness reads the cgroup counters around each try. The concurrent test is the function `bench_concurrent_qps` of the pin, called from `bash`.
//!
//! The settings of a system come from the `export` lines of its `benchmark.sh`, with the defaults of the pin. The other `export` lines, for example `PGHOST`, go to the environment of the scripts.
//!
//! The `query` script prints the result in the format of its client, so the answers come from a separate pass after the timed runs. That pass is not timed and not measured. It reads the same database through the PostgreSQL protocol for a server, or with `duckdb` for DuckDB.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::cgroup::{Cgroup, Interval, Usage};
use crate::json::Json;
use crate::load;
use crate::suite::{QueryResult, Run};

/// The settings of `lib/benchmark-common.sh` and their defaults in the pin.
const DEFAULTS: [(&str, &str); 8] = [
    ("BENCH_DOWNLOAD_SCRIPT", ""),
    ("BENCH_RESTARTABLE", "yes"),
    ("BENCH_DURABLE", "yes"),
    ("BENCH_TRIES", "3"),
    ("BENCH_QUERIES_FILE", "queries.sql"),
    ("BENCH_CHECK_TIMEOUT", "300"),
    ("BENCH_CONCURRENT_CONNECTIONS", "10"),
    ("BENCH_CONCURRENT_DURATION", "600"),
];

/// The environment of the scripts of one system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Env {
    pub(crate) vars: Vec<(String, String)>,
}

fn unquote(v: &str) -> &str {
    for q in ['"', '\''] {
        if let Some(inner) = v.strip_prefix(q).and_then(|v| v.strip_suffix(q)) {
            return inner;
        }
    }
    v
}

impl Env {
    /// Reads the `export NAME=VALUE` lines of `benchmark.sh`. `env` gives the environment of the harness: it sets a value of the form `${NAME:-default}`, and a setting of the pin that the script does not set, as in the shell.
    pub(crate) fn parse(script: &str, env: &dyn Fn(&str) -> Option<String>) -> Result<Env, String> {
        let set = |name: &str| env(name).filter(|v| !v.is_empty());
        let mut out = Env {
            vars: DEFAULTS
                .iter()
                .map(|(k, v)| ((*k).to_owned(), set(k).unwrap_or_else(|| (*v).to_owned())))
                .collect(),
        };
        for line in script.lines() {
            let Some(rest) = line.trim().strip_prefix("export ") else { continue };
            let Some((name, value)) = rest.split_once('=') else { continue };
            let value = unquote(value.trim());
            let value = match value.strip_prefix("${").and_then(|v| v.strip_suffix('}')) {
                Some(inner) => match inner.split_once(":-") {
                    Some((n, default)) if n == name => {
                        set(name).unwrap_or_else(|| default.to_owned())
                    }
                    _ => return Err(format!("benchmark.sh: cannot read {line:?}")),
                },
                None if value.contains('$') || value.contains('`') => {
                    return Err(format!("benchmark.sh: cannot read {line:?}"));
                }
                None => value.to_owned(),
            };
            out.set(name, &value);
        }
        Ok(out)
    }

    fn set(&mut self, name: &str, value: &str) {
        match self.vars.iter_mut().find(|(k, _)| k == name) {
            Some(slot) => slot.1 = value.to_owned(),
            None => self.vars.push((name.to_owned(), value.to_owned())),
        }
    }

    pub(crate) fn get(&self, name: &str) -> &str {
        self.vars.iter().find(|(k, _)| k == name).map_or("", |(_, v)| v.as_str())
    }

    fn yes(&self, name: &str) -> bool {
        self.get(name) == "yes"
    }

    fn number(&self, name: &str) -> Result<u64, String> {
        self.get(name).parse().map_err(|_| format!("{name}={:?} is not a number", self.get(name)))
    }

    pub(crate) fn to_json(&self) -> Json {
        self.vars.iter().fold(Json::obj(), |j, (k, v)| j.with(k, v.as_str()))
    }
}

/// The name that `./load` reads, from the download script of the system.
pub(crate) fn source_name(download: &str) -> Result<&'static str, String> {
    match download {
        "download-hits-tsv" => Ok("hits.tsv"),
        "download-hits-csv" => Ok("hits.csv"),
        "download-hits-parquet-single" => Ok("hits.parquet"),
        other => Err(format!("the harness cannot place the source file for {other:?}")),
    }
}

/// The time on the last line of the stderr of `./query` that is only a number, as `bench_run_query` reads it. A carriage return also ends a line.
pub(crate) fn parse_timing(stderr: &str) -> Option<f64> {
    stderr
        .split(['\n', '\r'])
        .map(str::trim_end)
        .rfind(|l| {
            let (int, frac) = l.split_once('.').unwrap_or((l, "1"));
            !int.is_empty()
                && !frac.is_empty()
                && int.bytes().all(|b| b.is_ascii_digit())
                && frac.bytes().all(|b| b.is_ascii_digit())
        })
        .and_then(|l| l.parse().ok())
}

/// The value of a line `Name: value` of the output of `bench_concurrent_qps`. `null` is `None`.
pub(crate) fn concurrent_value(out: &str, name: &str) -> Result<Option<f64>, String> {
    let line = out
        .lines()
        .find_map(|l| l.strip_prefix(name)?.strip_prefix(':'))
        .ok_or_else(|| format!("bench_concurrent_qps printed no {name:?} line"))?;
    match line.trim() {
        "null" => Ok(None),
        v => v.parse().map(Some).map_err(|_| format!("{name}: {v:?} is not a number")),
    }
}

/// Where the harness reads the counters.
#[derive(Debug)]
pub(crate) enum Measure {
    /// An in-process system: each query runs in a new cgroup `<prefix>-<query>`, and the load in `<prefix>-load`.
    Own { prefix: String, cpus: Option<String> },
    /// A server in the cgroup of a systemd unit, or in a cgroup that exists.
    Server { path: PathBuf, unit: Option<String> },
}

impl Measure {
    /// The cgroup of the server now. After a restart, systemd makes a new cgroup for the unit.
    fn server_cgroup(&self) -> Result<Option<Cgroup>, String> {
        match self {
            Measure::Own { .. } => Ok(None),
            Measure::Server { unit: Some(u), .. } => Cgroup::of_unit(u).map(Some),
            Measure::Server { path, unit: None } => Cgroup::attach(path).map(Some),
        }
    }

    pub(crate) fn to_json(&self) -> Json {
        match self {
            Measure::Own { prefix, cpus } => Json::obj()
                .with("mode", "in process, each query in a new cgroup")
                .with("cgroup_prefix", prefix.as_str())
                .with("cpus", cpus.clone()),
            Measure::Server { path, unit } => Json::obj()
                .with("mode", "server, the cgroup of the server")
                .with("cgroup", path.display().to_string())
                .with("unit", unit.clone()),
        }
    }
}

/// One system directory of the ClickBench pin.
#[derive(Debug)]
pub(crate) struct System {
    pub(crate) dir: PathBuf,
    pub(crate) env: Env,
}

/// The tries of one query: the ClickBench times, with `None` for a failed try, and the measured runs.
#[derive(Debug)]
pub(crate) struct Tries {
    pub(crate) times: Vec<Option<f64>>,
    pub(crate) result: QueryResult,
}

impl System {
    /// Reads `benchmark.sh` of the directory.
    pub(crate) fn open(dir: &Path) -> Result<System, String> {
        let path = dir.join("benchmark.sh");
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let env = Env::parse(&text, &|k| std::env::var(k).ok())?;
        if !env.yes("BENCH_DURABLE") {
            return Err("BENCH_DURABLE=no is not supported: the harness does not reload the data before each cold try".to_owned());
        }
        for script in ["check", "load", "query", "start", "stop", "data-size"] {
            if !dir.join(script).is_file() {
                return Err(format!("{} has no script {script}", dir.display()));
            }
        }
        Ok(System { dir: dir.to_owned(), env })
    }

    pub(crate) fn tries(&self) -> Result<usize, String> {
        usize::try_from(self.env.number("BENCH_TRIES")?).map_err(|e| e.to_string())
    }

    fn script(&self, name: &str) -> Command {
        let mut c = Command::new(self.dir.join(name));
        c.current_dir(&self.dir).envs(self.env.vars.iter().map(|(k, v)| (k, v)));
        c
    }

    /// Runs a script with no output and says whether it succeeded.
    fn quiet(&self, name: &str) -> bool {
        self.script(name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    /// The queries of `BENCH_QUERIES_FILE`, one on each line, named `q0` to `q42` as on the ClickBench site.
    pub(crate) fn queries(&self) -> Result<Vec<String>, String> {
        let path = self.dir.join(self.env.get("BENCH_QUERIES_FILE"));
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(text.lines().filter(|l| !l.is_empty()).map(str::to_owned).collect())
    }

    /// Links (or copies) the source file to the name that `./load` reads. The load script removes it after the load.
    pub(crate) fn place_source(&self, source: &Path) -> Result<PathBuf, String> {
        let name = source_name(self.env.get("BENCH_DOWNLOAD_SCRIPT"))?;
        let dest = self.dir.join(name);
        let _ = std::fs::remove_file(&dest);
        if std::fs::hard_link(source, &dest).is_err() {
            std::fs::copy(source, &dest)
                .map_err(|e| format!("copy {} to {}: {e}", source.display(), dest.display()))?;
        }
        Ok(dest)
    }

    /// `bench_check_loop`: `./check` once a second until it succeeds, for up to `BENCH_CHECK_TIMEOUT` seconds.
    fn check_loop(&self) -> Result<(), String> {
        let tries = self.env.number("BENCH_CHECK_TIMEOUT")?;
        let mut last = String::new();
        for _ in 0..tries {
            match self.script("check").stdin(Stdio::null()).output() {
                Ok(out) if out.status.success() => return Ok(()),
                Ok(out) => last = String::from_utf8_lossy(&out.stderr).trim().to_owned(),
                Err(e) => last = e.to_string(),
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        Err(format!("./check did not succeed within {tries} s: {last}"))
    }

    /// `bench_wait_stopped`: up to 60 s until `./check` fails.
    fn wait_stopped(&self) {
        for _ in 0..60 {
            if !self.quiet("check") {
                return;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    /// `bench_flush_caches`: `sync`, `drop_caches` and `./flush-caches` if the system has one.
    fn flush_caches(&self) -> Result<(), String> {
        load::drop_caches()?;
        if self.dir.join("flush-caches").is_file() {
            let _ = self.quiet("flush-caches");
        }
        Ok(())
    }

    /// The cold cycle of `bench_run_query`: stop, wait, flush the caches, start and check. A system with `BENCH_RESTARTABLE=no` only flushes the caches.
    pub(crate) fn cold_cycle(&self) -> Result<(), String> {
        if self.env.yes("BENCH_RESTARTABLE") {
            let _ = self.quiet("stop");
            self.wait_stopped();
            self.flush_caches()?;
            let _ = self.quiet("start");
            self.check_loop()
        } else {
            self.flush_caches()
        }
    }

    /// `./start` and the check loop, as `bench_start` does.
    pub(crate) fn start(&self) -> Result<(), String> {
        let _ = self.quiet("start");
        self.check_loop()
    }

    /// `./load` and `sync`, timed as `bench_load` times them, and measured. The output of the script goes to stderr.
    pub(crate) fn load(&self, measure: &Measure) -> Result<(f64, Usage), String> {
        let own = match measure {
            Measure::Own { prefix, cpus } => {
                Some(Cgroup::create(&format!("{prefix}-load"), cpus.as_deref())?)
            }
            Measure::Server { .. } => None,
        };
        let cg = match &own {
            Some(cg) => Cgroup::attach(&cg.path)?,
            None => measure.server_cgroup()?.ok_or("no cgroup")?,
        };
        let mut cmd = match &own {
            Some(cg) => {
                let mut c = cg.command(&["./load".to_owned()])?;
                c.current_dir(&self.dir).envs(self.env.vars.iter().map(|(k, v)| (k, v)));
                c
            }
            None => self.script("load"),
        };
        cmd.stdin(Stdio::null()).stdout(std::io::stderr());
        let interval = Interval::start(&cg)?;
        let start = Instant::now();
        let status = cmd.status().map_err(|e| format!("./load: {e}"));
        let synced = Command::new("sync").status().map_err(|e| format!("sync: {e}"));
        let secs = start.elapsed().as_secs_f64();
        let usage = interval.finish()?;
        if let Some(cg) = own {
            cg.kill_all()?;
            cg.remove()?;
        }
        let status = status?;
        synced?;
        if !status.success() {
            return Err(format!("./load failed: {status}"));
        }
        if !self.quiet("check") {
            return Err("./check failed after ./load".to_owned());
        }
        Ok((secs, usage))
    }

    /// `./data-size`, the bytes that the system reports.
    pub(crate) fn data_size(&self) -> Result<u64, String> {
        let out = self
            .script("data-size")
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("./data-size: {e}"))?;
        let text = String::from_utf8_lossy(&out.stdout);
        text.trim().parse().map_err(|_| format!("./data-size printed {:?}", text.trim()))
    }

    /// The cold cycle and the tries of one query, as `bench_run_query` runs them, with the counters of each try.
    pub(crate) fn run_query(&self, measure: &Measure, name: &str, sql: &str) -> Tries {
        let mut out = Tries {
            times: Vec::new(),
            result: QueryResult {
                name: name.to_owned(),
                runs: Vec::new(),
                answer: None,
                error: None,
                check: None,
            },
        };
        let attempt = (|| -> Result<(), String> {
            self.cold_cycle()?;
            let own = match measure {
                Measure::Own { prefix, cpus } => {
                    Some(Cgroup::create(&format!("{prefix}-{name}"), cpus.as_deref())?)
                }
                Measure::Server { .. } => None,
            };
            let cg = match &own {
                Some(cg) => Cgroup::attach(&cg.path)?,
                None => measure.server_cgroup()?.ok_or("no cgroup")?,
            };
            let mut errors = Vec::new();
            for i in 0..self.tries()? {
                let mut cmd = match &own {
                    Some(cg) => {
                        let mut c = cg.command(&["./query".to_owned()])?;
                        c.current_dir(&self.dir).envs(self.env.vars.iter().map(|(k, v)| (k, v)));
                        c
                    }
                    None => self.script("query"),
                };
                cmd.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped());
                let interval = Interval::start(&cg)?;
                let child = cmd.spawn().map_err(|e| format!("./query: {e}"));
                let output = child.and_then(|mut child| {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(format!("{sql}\n").as_bytes());
                    }
                    child.wait_with_output().map_err(|e| format!("./query: {e}"))
                });
                let usage = interval.finish()?;
                let output = output?;
                let stderr = String::from_utf8_lossy(&output.stderr);
                let secs = if output.status.success() { parse_timing(&stderr) } else { None };
                out.times.push(secs);
                match secs {
                    Some(secs) => out.result.runs.push(Run { secs, usage }),
                    None => errors.push(format!("try {}: {}", i + 1, last_line(&stderr))),
                }
            }
            if let Some(cg) = own {
                cg.kill_all()?;
                cg.remove()?;
            }
            if errors.is_empty() { Ok(()) } else { Err(errors.join("; ")) }
        })();
        if let Err(e) = attempt {
            out.result.error = Some(e);
        }
        out
    }

    /// `bench_concurrent_qps` of the pin, called from `bash`. For a server the harness does the cold cycle first and tells the function that the system cannot restart, so the server stays in the cgroup that is measured. The function still drops the caches. It returns `None` when the test is off.
    pub(crate) fn concurrent(
        &self,
        lib: &Path,
        measure: &Measure,
    ) -> Result<Option<Concurrent>, String> {
        if self.env.number("BENCH_CONCURRENT_DURATION")? == 0 {
            return Ok(None);
        }
        let own = match measure {
            Measure::Own { prefix, cpus } => {
                Some(Cgroup::create(&format!("{prefix}-concurrent"), cpus.as_deref())?)
            }
            Measure::Server { .. } => {
                self.cold_cycle()?;
                None
            }
        };
        let cg = match &own {
            Some(cg) => Cgroup::attach(&cg.path)?,
            None => measure.server_cgroup()?.ok_or("no cgroup")?,
        };
        let argv: Vec<String> = [
            "bash",
            "-c",
            "source \"$1\" && bench_concurrent_qps",
            "rupg-bench",
            &lib.display().to_string(),
        ]
        .map(str::to_owned)
        .to_vec();
        let mut cmd = match &own {
            Some(cg) => cg.command(&argv)?,
            None => {
                let mut c = Command::new(&argv[0]);
                c.args(&argv[1..]);
                c
            }
        };
        cmd.current_dir(&self.dir).envs(self.env.vars.iter().map(|(k, v)| (k, v)));
        if own.is_none() {
            cmd.env("BENCH_RESTARTABLE", "no");
        }
        cmd.stdin(Stdio::null()).stderr(Stdio::inherit());
        let interval = Interval::start(&cg)?;
        let output = cmd.output().map_err(|e| format!("bash: {e}"));
        let usage = interval.finish()?;
        if let Some(cg) = own {
            cg.kill_all()?;
            cg.remove()?;
        }
        let output = output?;
        if !output.status.success() {
            return Err(format!("bench_concurrent_qps failed: {}", output.status));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(Some(Concurrent {
            qps: concurrent_value(&text, "Concurrent QPS")?,
            error_ratio: concurrent_value(&text, "Concurrent error ratio")?,
            usage,
        }))
    }
}

/// The result of the concurrent test. `None` is the `null` of the pin.
#[derive(Debug)]
pub(crate) struct Concurrent {
    pub(crate) qps: Option<f64>,
    pub(crate) error_ratio: Option<f64>,
    pub(crate) usage: Usage,
}

fn last_line(text: &str) -> &str {
    text.lines().map(str::trim).rfind(|l| !l.is_empty()).unwrap_or("failed, no output")
}

/// The result file in the format of ClickBench: `template.json` of the system with the date, the machine, the load time, the data size, the concurrent test and the times of each try. A smoke run says so in `comment`.
pub(crate) fn result_file(
    template: &Json,
    date: &str,
    machine: &str,
    smoke: bool,
    (load_time, data_size): (Option<f64>, Option<u64>),
    concurrent: Option<(Option<f64>, Option<f64>)>,
    times: &[Vec<Option<f64>>],
) -> Result<Json, String> {
    let Json::Obj(entries) = template else {
        return Err("template.json is not an object".to_owned());
    };
    let mut out = Json::obj();
    for (k, v) in entries {
        out = out.with(k, v.clone());
        if k == "system" {
            out = out.with("date", date).with("machine", machine).with("cluster_size", 1u64);
        }
    }
    if smoke {
        out = out.with("comment", "smoke run of rupg-bench, not a baseline: do not submit");
    }
    let (qps, ratio) = concurrent.unwrap_or((None, None));
    Ok(out
        .with("load_time", load_time)
        .with("data_size", data_size)
        .with("concurrent_qps", qps)
        .with("concurrent_error_ratio", ratio)
        .with(
            "result",
            Json::Arr(
                times
                    .iter()
                    .map(|t| Json::Arr(t.iter().map(|v| Json::from(*v)).collect()))
                    .collect(),
            ),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_sh() {
        let duckdb = "#!/bin/bash\nexport BENCH_DOWNLOAD_SCRIPT=\"download-hits-parquet-single\"\nexport BENCH_RESTARTABLE=no\n# export BENCH_TRIES=9\nexport BENCH_CONCURRENT_DURATION=\"${BENCH_CONCURRENT_DURATION:-0}\"\nexport PGHOST=\"/tmp\"\nexec ../lib/benchmark-common.sh\n";
        let none = |_: &str| None;
        let e = Env::parse(duckdb, &none).unwrap();
        assert_eq!(e.get("BENCH_DOWNLOAD_SCRIPT"), "download-hits-parquet-single");
        assert_eq!(e.get("BENCH_RESTARTABLE"), "no");
        assert_eq!(e.get("BENCH_TRIES"), "3");
        assert_eq!(e.get("BENCH_CONCURRENT_DURATION"), "0");
        assert_eq!(e.get("PGHOST"), "/tmp");
        let env = |k: &str| match k {
            "BENCH_CONCURRENT_DURATION" => Some("30".to_owned()),
            "BENCH_RESTARTABLE" | "BENCH_TRIES" => Some("5".to_owned()),
            _ => None,
        };
        let e = Env::parse(duckdb, &env).unwrap();
        // The script sets BENCH_RESTARTABLE with no default, so the environment does not change it.
        assert_eq!(
            (e.get("BENCH_CONCURRENT_DURATION"), e.get("BENCH_RESTARTABLE"), e.get("BENCH_TRIES")),
            ("30", "no", "5")
        );
        assert!(Env::parse("export BENCH_TRIES=$(nproc)\n", &none).is_err());
        assert_eq!(source_name("download-hits-tsv"), Ok("hits.tsv"));
        assert!(source_name("download-hits-parquet-partitioned").is_err());
    }

    #[test]
    fn timings() {
        assert_eq!(parse_timing("0.123\n"), Some(0.123));
        assert_eq!(parse_timing("1.5\nStopping SparkContext\n"), Some(1.5));
        assert_eq!(parse_timing("[Stage 1:>  (0 + 96) / 111]\r   \r2.25\n"), Some(2.25));
        assert_eq!(parse_timing("12\n"), Some(12.0));
        assert_eq!(parse_timing("no timing in psql output\n"), None);
        assert_eq!(parse_timing("1.\n.5\n-1\n"), None);
        let out = "Concurrent QPS: 1.250\nConcurrent error ratio: null\n";
        assert_eq!(concurrent_value(out, "Concurrent QPS"), Ok(Some(1.25)));
        assert_eq!(concurrent_value(out, "Concurrent error ratio"), Ok(None));
        assert!(concurrent_value("", "Concurrent QPS").is_err());
    }

    #[test]
    fn clickbench_file() {
        let template =
            Json::parse("{\"system\": \"DuckDB\", \"tuned\": \"no\", \"tags\": [\"C++\"]}")
                .unwrap();
        let j = result_file(
            &template,
            "2026-10-07",
            "server3",
            true,
            (Some(1.5), Some(10)),
            None,
            &[vec![Some(0.5), None, Some(0.25)]],
        )
        .unwrap();
        let keys: Vec<&str> = match &j {
            Json::Obj(e) => e.iter().map(|(k, _)| k.as_str()).collect(),
            _ => Vec::new(),
        };
        assert_eq!(
            keys,
            [
                "system",
                "date",
                "machine",
                "cluster_size",
                "tuned",
                "tags",
                "comment",
                "load_time",
                "data_size",
                "concurrent_qps",
                "concurrent_error_ratio",
                "result"
            ]
        );
        assert!(j.pretty().contains("null"));
    }
}
