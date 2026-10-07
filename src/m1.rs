//! The M1 numbers of spec/23 section 23.4 of tamnd/rupg: the size of an empty file, the commit latency at one writer, the commits per second at one writer per core, and the recovery time after 1 GiB of log.
//!
//! The runs use the `rupg` facade at the commit in `Cargo.toml`, with the default options of the facade unless a step says otherwise. Each step makes a new file in the work directory and removes it at the end. Each commit is durable when it returns, so each commit waits for an `fdatasync` of the log, alone or in a group with the commits of other writers.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use rupg::{ColumnDef, Database, Datum, Options, RowId, TypeId};

use crate::json::Json;

/// The size of the text value of a row in the latency and throughput steps.
const SMALL_VALUE: usize = 100;

/// The size of an extent of the log ring in the facade.
const EXTENT: u64 = 16 << 20;

/// The settings of one run.
#[derive(Clone, Debug)]
pub(crate) struct Config {
    /// The directory for the database files.
    pub(crate) dir: PathBuf,
    /// The commits before the latency step starts to count.
    pub(crate) warmup: u32,
    /// The commits that the latency step counts.
    pub(crate) commits: u32,
    /// The writers of the throughput step.
    pub(crate) writers: usize,
    /// The length of the throughput step.
    pub(crate) seconds: f64,
    /// The bytes of row values that the recovery step writes to the log.
    pub(crate) log_bytes: u64,
    /// The rows that the recovery step updates in each transaction.
    pub(crate) rows: u32,
    /// The size of a row value in the recovery step.
    pub(crate) value: usize,
}

/// The steps of the `m1` command.
pub(crate) const STEPS: [&str; 4] = ["empty", "latency", "throughput", "recovery"];

/// The table of each step: a key and a text value.
fn columns() -> Vec<ColumnDef> {
    vec![
        ColumnDef { name: "k".into(), ty: TypeId::INT8, nullable: false },
        ColumnDef { name: "v".into(), ty: TypeId::TEXT, nullable: false },
    ]
}

fn fail(what: &str) -> impl Fn(rupg::Error) -> String + '_ {
    move |e| format!("{what}: {e}")
}

/// Removes a database file of an earlier run.
fn remove(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(format!("{}: {e}", path.display()))
        }
        _ => Ok(()),
    }
}

/// The size of the file and the bytes that the file system gives it.
fn sizes(path: &Path) -> Result<(u64, u64), String> {
    let m = fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    #[cfg(unix)]
    let allocated = std::os::unix::fs::MetadataExt::blocks(&m) * 512;
    #[cfg(not(unix))]
    let allocated = m.len();
    Ok((m.len(), allocated))
}

/// The empty file: a new database with the default options, closed with no table.
pub(crate) fn empty(c: &Config) -> Result<(Json, String), String> {
    let path = c.dir.join("empty.rupg");
    remove(&path)?;
    let db = Database::open(&path, Options::default()).map_err(fail("open"))?;
    db.close().map_err(fail("close"))?;
    let (len, allocated) = sizes(&path)?;
    remove(&path)?;
    let json = Json::obj().with("file_bytes", len).with("allocated_bytes", allocated);
    let text = format!(
        "empty file      {} on the file system, {} allocated\n",
        crate::cgroup::mib(len),
        crate::cgroup::mib(allocated)
    );
    Ok((json, text))
}

/// The value of row `k` with `len` bytes.
fn small(k: i64) -> Vec<Datum> {
    vec![Datum::Int8(k), Datum::Text(format!("{k:0width$}", width = SMALL_VALUE))]
}

/// Percentiles of the sorted samples in microseconds.
fn percentiles(sorted: &[Duration]) -> Json {
    let at = |p: f64| {
        let i = ((sorted.len() as f64 * p).ceil() as usize).clamp(1, sorted.len()) - 1;
        sorted[i].as_secs_f64() * 1e6
    };
    let mean = sorted.iter().map(Duration::as_secs_f64).sum::<f64>() / sorted.len() as f64 * 1e6;
    Json::obj()
        .with("p50_us", at(0.50))
        .with("p90_us", at(0.90))
        .with("p99_us", at(0.99))
        .with("p999_us", at(0.999))
        .with("max_us", at(1.0))
        .with("mean_us", mean)
}

/// The commit latency at one writer. Each transaction inserts one row and commits. The time is from the start of the transaction to the return of the commit.
pub(crate) fn latency(c: &Config) -> Result<(Json, String), String> {
    let path = c.dir.join("latency.rupg");
    remove(&path)?;
    let db = Database::open(&path, Options::default()).map_err(fail("open"))?;
    let t = db.create_table("t", columns()).map_err(fail("create table"))?;
    let mut samples = Vec::with_capacity(c.commits as usize);
    for i in 0..c.warmup + c.commits {
        let start = Instant::now();
        let mut tx = db.transaction().map_err(fail("begin"))?;
        tx.insert(&t, &small(i64::from(i))).map_err(fail("insert"))?;
        tx.commit().map_err(fail("commit"))?;
        if i >= c.warmup {
            samples.push(start.elapsed());
        }
    }
    db.close().map_err(fail("close"))?;
    remove(&path)?;
    samples.sort_unstable();
    let p = percentiles(&samples);
    let num = |k: &str| match p.get(k) {
        Some(Json::Num(x)) => *x,
        _ => 0.0,
    };
    let text = format!(
        "commit latency  {} commits after {} warmup: p50 {:.1} us, p90 {:.1} us, p99 {:.1} us, max {:.1} us\n",
        c.commits,
        c.warmup,
        num("p50_us"),
        num("p90_us"),
        num("p99_us"),
        num("max_us")
    );
    let json = Json::obj()
        .with("warmup", c.warmup)
        .with("commits", c.commits)
        .with("value_bytes", SMALL_VALUE)
        .with("latency", p);
    Ok((json, text))
}

/// The commits per second with one writer on each core. Each writer has its own table, inserts one row in each transaction and commits, so the writers meet only in the log.
pub(crate) fn throughput(c: &Config) -> Result<(Json, String), String> {
    let path = c.dir.join("throughput.rupg");
    remove(&path)?;
    let db = Database::open(&path, Options::default()).map_err(fail("open"))?;
    let tables = (0..c.writers)
        .map(|w| db.create_table(&format!("t{w}"), columns()).map_err(fail("create table")))
        .collect::<Result<Vec<_>, _>>()?;
    let stop = Arc::new(AtomicBool::new(false));
    let start = Arc::new(Barrier::new(c.writers + 1));
    let handles: Vec<_> = tables
        .into_iter()
        .map(|t| {
            let (db, stop, start) = (db.clone(), stop.clone(), start.clone());
            std::thread::spawn(move || -> Result<u64, String> {
                start.wait();
                let mut n = 0u64;
                while !stop.load(Ordering::Relaxed) {
                    let mut tx = db.transaction().map_err(fail("begin"))?;
                    tx.insert(&t, &small(n as i64)).map_err(fail("insert"))?;
                    tx.commit().map_err(fail("commit"))?;
                    n += 1;
                }
                Ok(n)
            })
        })
        .collect();
    start.wait();
    let began = Instant::now();
    std::thread::sleep(Duration::from_secs_f64(c.seconds));
    stop.store(true, Ordering::Relaxed);
    let mut counts = Vec::with_capacity(handles.len());
    for h in handles {
        counts.push(h.join().map_err(|_| "a writer panicked".to_owned())??);
    }
    let wall = began.elapsed().as_secs_f64();
    db.close().map_err(fail("close"))?;
    remove(&path)?;
    let total: u64 = counts.iter().sum();
    let rate = total as f64 / wall;
    let (min, max) = (counts.iter().min().copied(), counts.iter().max().copied());
    let json = Json::obj()
        .with("writers", c.writers)
        .with("seconds", wall)
        .with("commits", total)
        .with("commits_per_second", rate)
        .with("writer_min", min.unwrap_or(0))
        .with("writer_max", max.unwrap_or(0));
    let text = format!(
        "throughput      {} writers for {wall:.1} s: {total} commits, {rate:.0} commits/s (each writer {} to {})\n",
        c.writers,
        min.unwrap_or(0),
        max.unwrap_or(0)
    );
    Ok((json, text))
}

/// The options of the recovery file. The ring holds the log with a quarter more for the block headers and the fill, and no automatic checkpoint starts before the log is full.
fn recovery_options(log_bytes: u64) -> Options {
    let ring = (log_bytes + log_bytes / 4).div_ceil(EXTENT) + 1;
    Options {
        log_extents: ring as usize,
        checkpoint_log: Some(ring * EXTENT),
        ..Options::default()
    }
}

/// The text value of round `round` with `len` bytes. The first 20 bytes are the round.
fn round_value(round: u64, len: usize) -> String {
    let mut s = format!("{round:020}");
    s.extend(std::iter::repeat_n('x', len.saturating_sub(20)));
    s
}

/// The child of the recovery step, `rupg-bench m1-fill`. It inserts the rows, takes a checkpoint, and then updates every row in each transaction until the row values in the log reach `log_bytes`. It prints `done ROUND BYTES` and exits with no close, so the next open must replay the log.
pub(crate) fn fill(path: &Path, c: &Config) -> Result<(), String> {
    let db = Database::open(path, recovery_options(c.log_bytes)).map_err(fail("open"))?;
    let t = db.create_table("t", columns()).map_err(fail("create table"))?;
    let mut tx = db.transaction().map_err(fail("begin"))?;
    let mut ids = Vec::with_capacity(c.rows as usize);
    for k in 0..c.rows {
        let row = [Datum::Int8(i64::from(k)), Datum::Text(round_value(0, c.value))];
        ids.push(tx.insert(&t, &row).map_err(fail("insert"))?);
    }
    tx.commit().map_err(fail("commit"))?;
    db.checkpoint().map_err(fail("checkpoint"))?;
    let (mut round, mut written) = (0u64, 0u64);
    while written < c.log_bytes {
        round += 1;
        let value = round_value(round, c.value);
        let mut tx = db.transaction().map_err(fail("begin"))?;
        for (k, id) in ids.iter().enumerate() {
            let row = [Datum::Int8(k as i64), Datum::Text(value.clone())];
            tx.update(&t, *id, &row).map_err(fail("update"))?;
        }
        tx.commit().map_err(fail("commit"))?;
        written += u64::from(c.rows) * c.value as u64;
    }
    let mut out = std::io::stdout().lock();
    writeln!(out, "done {round} {written}")
        .and_then(|()| out.flush())
        .map_err(|e| e.to_string())?;
    // No close and no drop: the file is as after a crash of the process. The data is in the page cache.
    std::process::exit(0)
}

/// Drops the pages of `path` from the page cache with GNU dd, so the open reads the file from the disk. It needs no root and touches no other file. It gives false where it does not work, for example on macOS.
fn drop_cache(path: &Path) -> bool {
    let mut arg = std::ffi::OsString::from("if=");
    arg.push(path);
    Command::new("dd")
        .arg(arg)
        .args(["iflag=nocache", "count=0", "status=none"])
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// The recovery time after `log_bytes` of log. A child process writes the log and exits with no close. Then this process drops the file from the page cache where it can, times `Database::open`, which replays the log, and checks that each row has the value of the last commit.
pub(crate) fn recovery(c: &Config) -> Result<(Json, String), String> {
    let path = c.dir.join("recovery.rupg");
    remove(&path)?;
    let exe = std::env::current_exe().map_err(|e| format!("the path of rupg-bench: {e}"))?;
    let began = Instant::now();
    let mut child = Command::new(exe)
        .arg("m1-fill")
        .arg("--file")
        .arg(&path)
        .args(["--log-bytes", &c.log_bytes.to_string()])
        .args(["--rows", &c.rows.to_string()])
        .args(["--value", &c.value.to_string()])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("m1-fill: {e}"))?;
    let stdout = child.stdout.take().ok_or("m1-fill has no output")?;
    let line = BufReader::new(stdout).lines().next().transpose().map_err(|e| e.to_string())?;
    let status = child.wait().map_err(|e| e.to_string())?;
    let fill_seconds = began.elapsed().as_secs_f64();
    let (round, written) = line
        .as_deref()
        .and_then(|l| l.strip_prefix("done "))
        .and_then(|l| l.split_once(' '))
        .and_then(|(r, w)| Some((r.parse::<u64>().ok()?, w.parse::<u64>().ok()?)))
        .filter(|_| status.success())
        .ok_or(format!("m1-fill failed: {status}"))?;
    // The check reads the log after the redo position as an open does, and gives its size.
    let report = rupg::check(&path).map_err(fail("check"))?;
    let (file_bytes, _) = sizes(&path)?;
    let cold = drop_cache(&path);
    let start = Instant::now();
    let db = Database::open(&path, Options { create: false, ..recovery_options(c.log_bytes) })
        .map_err(fail("open"))?;
    let open = start.elapsed().as_secs_f64();
    let t = db.table("t").map_err(fail("table"))?;
    let want = round_value(round, c.value);
    let tx = db.transaction().map_err(fail("begin"))?;
    let rows = tx.scan(&t, RowId::from_bits(0)..).map_err(fail("scan"))?;
    let stale = rows.iter().filter(|(_, v)| v.get(1) != Some(&Datum::Text(want.clone()))).count();
    drop(tx);
    db.close().map_err(fail("close"))?;
    remove(&path)?;
    if rows.len() != c.rows as usize || stale > 0 {
        return Err(format!(
            "after recovery the table has {} rows and {stale} of them do not have the value of round {round}",
            rows.len()
        ));
    }
    let log = report.log_bytes as f64 / f64::from(1 << 30);
    let json = Json::obj()
        .with("rows", c.rows)
        .with("value_bytes", c.value)
        .with("rounds", round)
        .with("row_bytes_written", written)
        .with("log_blocks", report.log_blocks)
        .with("log_bytes", report.log_bytes)
        .with("file_bytes", file_bytes)
        .with("fill_seconds", fill_seconds)
        .with("page_cache_dropped", cold)
        .with("open_seconds", open)
        .with("replay_gib_per_second", log / open);
    let cache = if cold { "after dd iflag=nocache" } else { "with the file in the page cache" };
    let text = format!(
        "recovery        {:.3} GiB of log in {} blocks ({round} commits of {} rows): open {open:.3} s {cache}, {:.2} GiB/s\n",
        log,
        report.log_blocks,
        c.rows,
        log / open
    );
    Ok((json, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_take_the_sample_at_or_above() {
        let s: Vec<Duration> = (1..=100).map(Duration::from_micros).collect();
        let p = percentiles(&s);
        assert_eq!(p.get("p50_us"), Some(&Json::Num(50.0)));
        assert_eq!(p.get("p99_us"), Some(&Json::Num(99.0)));
        assert_eq!(p.get("max_us"), Some(&Json::Num(100.0)));
    }

    #[test]
    fn the_ring_holds_the_log() {
        let o = recovery_options(1 << 30);
        assert!(o.log_extents as u64 * EXTENT >= (1 << 30) + (1 << 28));
        assert_eq!(o.checkpoint_log, Some(o.log_extents as u64 * EXTENT));
    }

    #[test]
    fn round_values_start_with_the_round() {
        let v = round_value(7, 64);
        assert_eq!(v.len(), 64);
        assert!(v.starts_with("00000000000000000007x"));
    }

    #[test]
    fn the_pin_names_the_commit_of_the_dependency() {
        let cargo = include_str!("../Cargo.toml");
        let pins =
            crate::pins::Pins::load(concat!(env!("CARGO_MANIFEST_DIR"), "/pins.toml")).unwrap();
        let commit = pins.get("rupg", "commit").unwrap();
        assert_eq!(commit.len(), 40);
        assert!(
            cargo.contains(&format!("rev = \"{commit}\"")),
            "Cargo.toml and pins.toml name different commits of rupg"
        );
    }

    #[test]
    fn a_small_run_works() {
        let dir = std::env::temp_dir().join(format!("rupg-bench-m1-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let c = Config {
            dir: dir.clone(),
            warmup: 2,
            commits: 20,
            writers: 2,
            seconds: 0.2,
            log_bytes: 0,
            rows: 0,
            value: 0,
        };
        let (j, _) = empty(&c).unwrap();
        assert!(matches!(j.get("file_bytes"), Some(Json::UInt(n)) if *n > 0));
        latency(&c).unwrap();
        let (j, _) = throughput(&c).unwrap();
        assert!(matches!(j.get("commits"), Some(Json::UInt(n)) if *n > 0));
        fs::remove_dir_all(&dir).unwrap();
    }
}
