//! The pgbench driver and its consistency check (spec/21 section 21.4.6).
//!
//! The driver runs the `pgbench` program of the PostgreSQL build: `pgbench -i` at the scale, then the built-in TPC-B like script for a fixed time. The server is measured in its cgroup over the run. After the run the driver checks that the sum of `abalance` equals the sum of `tbalance` and the sum of `bbalance`, and that the row count of `pgbench_history` equals the transaction count that pgbench reported. A run that fails the check is a wrong answer and not a number.

use std::path::Path;
use std::process::Command;

use crate::json::Json;
use crate::pg::{Config, Conn};

/// What pgbench printed at the end of a run.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Summary {
    pub(crate) transactions: u64,
    pub(crate) failed: u64,
    pub(crate) latency_ms: Option<f64>,
    pub(crate) tps: Option<f64>,
}

/// Parses the summary lines of pgbench 15 and later.
pub(crate) fn parse_summary(text: &str) -> Result<Summary, String> {
    let mut s = Summary::default();
    let mut found = false;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("number of transactions actually processed:") {
            // With -t the line is "N/M". With -T it is "N".
            let n = rest.trim().split('/').next().unwrap_or("");
            s.transactions = n.trim().parse().map_err(|_| format!("pgbench printed {line:?}"))?;
            found = true;
        } else if let Some(rest) = line.strip_prefix("number of failed transactions:") {
            s.failed = rest.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("latency average =") {
            s.latency_ms = rest.trim().trim_end_matches("ms").trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("tps =") {
            s.tps = rest.split_whitespace().next().and_then(|n| n.parse().ok());
        }
    }
    if found { Ok(s) } else { Err("pgbench printed no transaction count".to_owned()) }
}

/// The numbers of the consistency check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Check {
    pub(crate) abalance: i64,
    pub(crate) tbalance: i64,
    pub(crate) bbalance: i64,
    pub(crate) history: u64,
    pub(crate) transactions: u64,
}

impl Check {
    pub(crate) fn read(conn: &mut Conn, transactions: u64) -> Result<Check, String> {
        let r = conn.query(
            "SELECT (SELECT coalesce(sum(abalance), 0) FROM pgbench_accounts), \
                    (SELECT coalesce(sum(tbalance), 0) FROM pgbench_tellers), \
                    (SELECT coalesce(sum(bbalance), 0) FROM pgbench_branches), \
                    (SELECT count(*) FROM pgbench_history)",
        )?;
        let row = r.rows.first().ok_or("the check returned no row")?;
        let num = |i: usize| -> Result<i64, String> {
            row.get(i)
                .and_then(|v| v.as_deref())
                .and_then(|v| v.parse().ok())
                .ok_or(format!("the check returned {row:?}"))
        };
        Ok(Check {
            abalance: num(0)?,
            tbalance: num(1)?,
            bbalance: num(2)?,
            history: u64::try_from(num(3)?).map_err(|_| "a negative count")?,
            transactions,
        })
    }

    /// The failures of the check, empty when it passes.
    pub(crate) fn failures(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.abalance != self.tbalance || self.abalance != self.bbalance {
            out.push(format!(
                "sum(abalance) = {}, sum(tbalance) = {}, sum(bbalance) = {}: the sums differ",
                self.abalance, self.tbalance, self.bbalance
            ));
        }
        if self.history != self.transactions {
            out.push(format!(
                "pgbench_history has {} rows, pgbench reported {} transactions",
                self.history, self.transactions
            ));
        }
        out
    }

    pub(crate) fn to_json(&self) -> Json {
        let failures = self.failures();
        Json::obj()
            .with("sum_abalance", self.abalance)
            .with("sum_tbalance", self.tbalance)
            .with("sum_bbalance", self.bbalance)
            .with("history_rows", self.history)
            .with("transactions", self.transactions)
            .with("passed", failures.is_empty())
            .with("failures", failures)
    }
}

/// A `pgbench` command with the connection in the environment, as libpq reads it.
pub(crate) fn command(bin: &Path, c: &Config) -> Command {
    let mut cmd = Command::new(bin.join("pgbench"));
    cmd.env("PGHOST", &c.host)
        .env("PGPORT", c.port.to_string())
        .env("PGUSER", &c.user)
        .env("PGDATABASE", &c.dbname);
    if let Some(p) = &c.password {
        cmd.env("PGPASSWORD", p);
    }
    cmd
}

/// Runs a command and returns its standard output, or the error with its standard error.
pub(crate) fn output(mut cmd: Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("{:?}: {e}", cmd.get_program()))?;
    if !out.status.success() {
        return Err(format!(
            "{:?} failed: {}\n{}",
            cmd.get_program(),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUTPUT: &str = "pgbench (19beta4)
transaction type: <builtin: TPC-B (sort of)>
scaling factor: 1
query mode: simple
number of clients: 4
number of threads: 2
maximum number of tries: 1
duration: 30 s
number of transactions actually processed: 12345
number of failed transactions: 0 (0.000%)
latency average = 9.721 ms
initial connection time = 6.130 ms
tps = 411.497617 (without initial connection time)
";

    #[test]
    fn summary() {
        let s = parse_summary(OUTPUT).unwrap();
        assert_eq!(s.transactions, 12345);
        assert_eq!(s.failed, 0);
        assert_eq!(s.latency_ms, Some(9.721));
        assert_eq!(s.tps, Some(411.497617));
        let t = parse_summary("number of transactions actually processed: 100/100\n").unwrap();
        assert_eq!(t.transactions, 100);
        assert!(parse_summary("tps = 1\n").is_err());
    }

    #[test]
    fn the_check() {
        let mut c =
            Check { abalance: -5, tbalance: -5, bbalance: -5, history: 10, transactions: 10 };
        assert!(c.failures().is_empty());
        c.bbalance = 7;
        c.history = 9;
        let f = c.failures();
        assert_eq!(f.len(), 2);
        assert!(f[0].contains("the sums differ"));
        assert!(f[1].contains("9 rows"));
        assert!(c.to_json().pretty().contains("\"passed\": false"));
    }
}
