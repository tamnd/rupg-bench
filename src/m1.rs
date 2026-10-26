//! The M1 numbers of spec/23 section 23.4 of tamnd/rupg: the size of an empty file, the commit latency at one writer next to the `fdatasync` time of the device, the commits per second at one writer per core, the recovery time after 1 GiB of log, a load with a log ring of 8 MiB for each writer (spec/19 section 19.10.1), the commit curve of PostgreSQL on the same machine, and the operations of the instruction count set of M1 (spec/21 section 21.14).
//!
//! The runs use the `rupg` facade at the commit in `Cargo.toml`, with the default options of the facade unless a step says otherwise. Each step makes a new file in the work directory and removes it at the end. Each commit is durable when it returns, so each commit waits for an `fdatasync` of the log, alone or in a group with the commits of other writers.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use rupg::{ColumnDef, Database, Datum, Options, Platform, RowId, TypeId};
use rupg_platform::sim::{SimClock, SimEntropy, SimIo};

use crate::json::Json;

/// The size of the text value of a row in the latency and throughput steps.
const SMALL_VALUE: usize = 100;

/// The size of an extent of the log ring in the facade.
const EXTENT: u64 = 16 << 20;

/// The ring for each writer that spec/19 section 19.10.1 asks about.
const WORKER_RING: u64 = 8 << 20;

/// The PostgreSQL server of the `postgresql` step.
#[derive(Clone, Debug)]
pub(crate) struct Postgres {
    /// The directory with `pgbench` and `psql`.
    pub(crate) bin: PathBuf,
    /// The connection: host, port, user and database.
    pub(crate) conn: crate::pg::Config,
    /// The client counts of the curve.
    pub(crate) clients: Vec<usize>,
}

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
    /// The server of the `postgresql` step.
    pub(crate) postgres: Option<Postgres>,
}

/// The steps of the `m1` command.
pub(crate) const STEPS: [&str; 6] =
    ["empty", "latency", "throughput", "recovery", "ring", "postgresql"];

/// The `fdatasync` samples of the latency step, after 10 that do not count.
const SYNCS: usize = 1_000;

/// The rows of one transaction of the ring step.
const LOAD_ROWS: u32 = 100;

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

/// The `fdatasync` time of the device under `dir`, as `machines/fdatasync.sh` measures it with fio: writes of 4 KiB to the end of a file, each with one `fdatasync`, at a queue depth of 1.
fn fdatasync(dir: &Path) -> Result<Vec<Duration>, String> {
    let path = dir.join("fdatasync.bin");
    let err = |e: std::io::Error| format!("{}: {e}", path.display());
    let mut f =
        fs::OpenOptions::new().create(true).truncate(true).write(true).open(&path).map_err(err)?;
    let block = [0x5a_u8; 4096];
    let mut samples = Vec::with_capacity(SYNCS);
    for i in 0..10 + SYNCS {
        f.write_all(&block).map_err(err)?;
        let start = Instant::now();
        f.sync_data().map_err(err)?;
        if i >= 10 {
            samples.push(start.elapsed());
        }
    }
    drop(f);
    remove(&path)?;
    samples.sort_unstable();
    Ok(samples)
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
    let sync = percentiles(&fdatasync(&c.dir)?);
    let num = |j: &Json, k: &str| match j.get(k) {
        Some(Json::Num(x)) => *x,
        _ => 0.0,
    };
    let text = format!(
        "commit latency  {} commits after {} warmup: p50 {:.1} us, p90 {:.1} us, p99 {:.1} us, max {:.1} us\n\
         fdatasync       {SYNCS} writes of 4 KiB: p50 {:.1} us, p99 {:.1} us\n",
        c.commits,
        c.warmup,
        num(&p, "p50_us"),
        num(&p, "p90_us"),
        num(&p, "p99_us"),
        num(&p, "max_us"),
        num(&sync, "p50_us"),
        num(&sync, "p99_us")
    );
    let json = Json::obj()
        .with("warmup", c.warmup)
        .with("commits", c.commits)
        .with("value_bytes", SMALL_VALUE)
        .with("latency", p)
        .with("fdatasync_syncs", SYNCS)
        .with("fdatasync", sync);
    Ok((json, text))
}

/// The commits per second with one writer on each core. Each writer has its own table, inserts one row in each transaction and commits, so the writers meet only in the log.
pub(crate) fn throughput(c: &Config) -> Result<(Json, String), String> {
    let path = c.dir.join("throughput.rupg");
    let (counts, wall) = commits(&path, Options::default(), c, c.seconds)?;
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

/// Runs `c.writers` writers for `seconds` on a new file with `options`. It gives the commits of each writer and the seconds.
fn commits(
    path: &Path,
    options: Options,
    c: &Config,
    seconds: f64,
) -> Result<(Vec<u64>, f64), String> {
    remove(path)?;
    let db = Database::open(path, options).map_err(fail("open"))?;
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
    std::thread::sleep(Duration::from_secs_f64(seconds));
    stop.store(true, Ordering::Relaxed);
    let mut counts = Vec::with_capacity(handles.len());
    for h in handles {
        counts.push(h.join().map_err(|_| "a writer panicked".to_owned())??);
    }
    let wall = began.elapsed().as_secs_f64();
    db.close().map_err(fail("close"))?;
    remove(path)?;
    Ok((counts, wall))
}

/// One load of the ring step: `writers` threads insert rows of `value` bytes, `LOAD_ROWS` in each commit, until the values are `bytes`. It gives the seconds.
fn load(path: &Path, options: Options, c: &Config, bytes: u64) -> Result<f64, String> {
    remove(path)?;
    let db = Database::open(path, options).map_err(fail("open"))?;
    let tables = (0..c.writers)
        .map(|w| db.create_table(&format!("t{w}"), columns()).map_err(fail("create table")))
        .collect::<Result<Vec<_>, _>>()?;
    let commits = bytes.div_ceil(c.value as u64 * u64::from(LOAD_ROWS) * c.writers as u64);
    let start = Arc::new(Barrier::new(c.writers + 1));
    let handles: Vec<_> = tables
        .into_iter()
        .map(|t| {
            let (db, start, value) = (db.clone(), start.clone(), c.value);
            std::thread::spawn(move || -> Result<(), String> {
                start.wait();
                for round in 0..commits {
                    let v = round_value(round, value);
                    let mut tx = db.transaction().map_err(fail("begin"))?;
                    for r in 0..LOAD_ROWS {
                        let k = (round * u64::from(LOAD_ROWS) + u64::from(r)) as i64;
                        tx.insert(&t, &[Datum::Int8(k), Datum::Text(v.clone())])
                            .map_err(fail("insert"))?;
                    }
                    tx.commit().map_err(fail("commit"))?;
                }
                Ok(())
            })
        })
        .collect();
    start.wait();
    let began = Instant::now();
    for h in handles {
        h.join().map_err(|_| "a writer panicked".to_owned())??;
    }
    let wall = began.elapsed().as_secs_f64();
    db.close().map_err(fail("close"))?;
    remove(path)?;
    Ok(wall)
}

/// The extents of a ring of `WORKER_RING` bytes for each of `writers`. The file has one ring, so the ring of the file is the sum, at least one extent.
fn worker_extents(writers: usize) -> usize {
    (writers as u64 * WORKER_RING).div_ceil(EXTENT).max(1) as usize
}

/// The ring step: the same load and the same commit run with the default log ring, with a ring of 8 MiB for each writer, and with a ring of one extent, the smallest ring of the file format. The file has one ring for all writers, so the ring of 8 MiB for each writer is 8 MiB times the writers, in extents of 16 MiB. A ring is enough when its load is at most 10 percent slower than the load with the default ring and its commit rate is at most 10 percent lower.
pub(crate) fn ring(c: &Config) -> Result<(Json, String), String> {
    let path = c.dir.join("ring.rupg");
    let bytes = c.log_bytes;
    let gib = bytes as f64 / f64::from(1 << 30);
    let seconds = c.seconds.min(10.0);
    let rings = [
        ("default", Options::default().log_extents),
        ("worker", worker_extents(c.writers)),
        ("smallest", 1),
    ];
    let mut json = Json::obj()
        .with("writers", c.writers)
        .with("value_bytes", c.value)
        .with("rows_per_commit", LOAD_ROWS)
        .with("load_bytes", bytes)
        .with("commit_seconds", seconds);
    let mut text = String::new();
    let mut base = (0.0, 0.0);
    for (name, extents) in rings {
        let options = Options { log_extents: extents, ..Options::default() };
        let load_seconds = load(&path, options.clone(), c, bytes)?;
        let (counts, wall) = commits(&path, options, c, seconds)?;
        let rate = counts.iter().sum::<u64>() as f64 / wall;
        if name == "default" {
            base = (load_seconds, rate);
        }
        let slower = load_seconds / base.0 - 1.0;
        let fewer = 1.0 - rate / base.1;
        let enough = slower <= 0.10 && fewer <= 0.10;
        let ring_bytes = extents as u64 * EXTENT;
        json = json.with(
            name,
            Json::obj()
                .with("ring_bytes", ring_bytes)
                .with("load_seconds", load_seconds)
                .with("load_slower", slower)
                .with("commits_per_second", rate)
                .with("commits_fewer", fewer)
                .with("enough", enough),
        );
        text.push_str(&format!(
            "ring {name:<10} {} MiB: load of {gib:.3} GiB in {load_seconds:.1} s ({:+.1} percent), {rate:.0} commits/s ({:+.1} percent){}\n",
            ring_bytes >> 20,
            slower * 100.0,
            -fewer * 100.0,
            match (name, enough) {
                ("default", _) => "",
                (_, true) => ", enough",
                (_, false) => ", not enough",
            }
        ));
    }
    Ok((json, text))
}

/// The commit curve of PostgreSQL on the same machine (spec/03 section 3.12.2): for each client count, pgbench runs for `c.seconds` with one durable insert of one row of `SMALL_VALUE` bytes in each transaction, the same work as a commit of the throughput step.
pub(crate) fn postgresql(c: &Config) -> Result<(Json, String), String> {
    let pg = c.postgres.as_ref().ok_or("the postgresql step needs --pg-bin DIR")?;
    let conn = |cmd: &mut Command| {
        cmd.args(["-h", &pg.conn.host, "-p", &pg.conn.port.to_string()]).args([
            "-U",
            &pg.conn.user,
            "-d",
            &pg.conn.dbname,
        ]);
        if let Some(p) = &pg.conn.password {
            cmd.env("PGPASSWORD", p);
        }
    };
    let psql = |sql: &str| -> Result<String, String> {
        let mut cmd = Command::new(pg.bin.join("psql"));
        cmd.args(["-X", "-q", "-At", "-v", "ON_ERROR_STOP=1", "-c", sql]);
        conn(&mut cmd);
        let out = cmd.output().map_err(|e| format!("psql: {e}"))?;
        if !out.status.success() {
            return Err(format!("psql: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    };
    let version = psql("SHOW server_version")?;
    let settings = psql(
        "SELECT string_agg(name || '=' || setting, ' ' ORDER BY name) FROM pg_settings WHERE name IN ('fsync', 'synchronous_commit', 'wal_sync_method', 'full_page_writes', 'shared_buffers')",
    )?;
    let data = psql("SHOW data_directory")?;
    psql(
        "DROP TABLE IF EXISTS rupg_bench_commit; CREATE TABLE rupg_bench_commit (k bigint NOT NULL, v text NOT NULL)",
    )?;
    let script = c.dir.join("commit.sql");
    fs::write(
        &script,
        format!(
            "\\set k random(1, 1000000000)\nINSERT INTO rupg_bench_commit VALUES (:k, lpad(:k::text, {SMALL_VALUE}, '0'));\n"
        ),
    )
    .map_err(|e| format!("{}: {e}", script.display()))?;
    let cores = std::thread::available_parallelism().map_or(1, usize::from);
    let mut rows = Vec::new();
    let mut text = format!("postgresql      {version}, {settings}\n");
    for &clients in &pg.clients {
        let mut cmd = Command::new(pg.bin.join("pgbench"));
        cmd.args(["-n", "-f"]).arg(&script).args([
            "-c",
            &clients.to_string(),
            "-j",
            &clients.min(cores).to_string(),
            "-T",
            &format!("{}", c.seconds.ceil() as u64),
        ]);
        conn(&mut cmd);
        let out = cmd.output().map_err(|e| format!("pgbench: {e}"))?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !out.status.success() {
            return Err(format!("pgbench: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        let (tps, latency) = parse_pgbench(&stdout)?;
        rows.push(
            Json::obj()
                .with("clients", clients)
                .with("commits_per_second", tps)
                .with("latency_ms", latency),
        );
        text.push_str(&format!(
            "                {clients:>3} clients: {tps:.0} commits/s, {latency:.3} ms each\n"
        ));
    }
    psql("DROP TABLE rupg_bench_commit")?;
    let _ = fs::remove_file(&script);
    let json = Json::obj()
        .with("server_version", version.as_str())
        .with("settings", settings.as_str())
        .with("data_directory", data.as_str())
        .with("value_bytes", SMALL_VALUE)
        .with("seconds", c.seconds.ceil())
        .with("curve", Json::from(rows));
    Ok((json, text))
}

/// The transactions per second and the mean latency in milliseconds of the output of pgbench.
fn parse_pgbench(out: &str) -> Result<(f64, f64), String> {
    let number = |prefix: &str| {
        out.lines().find_map(|l| {
            l.trim().strip_prefix(prefix)?.split_whitespace().next()?.parse::<f64>().ok()
        })
    };
    let tps = number("tps = ").ok_or_else(|| format!("pgbench printed no tps: {out:?}"))?;
    let latency = number("latency average = ").unwrap_or(0.0);
    Ok((tps, latency))
}

/// The operations of the instruction count set of M1. Each runs in a new process on the in-memory file system of rupg, so the count has no work of the kernel for the file. `base` opens a new file, makes a table with `PRELOAD` rows, commits and closes the file. Each operation does the same work and its own work before the close, so its count minus the count of `base` is the work of the operation. `insert` inserts `PRELOAD` more rows in one transaction, `commit` commits `COMMITS` transactions of one row, `get` reads each row by its id, `scan` reads the table `REPEATS` times, `update` and `delete` change each row in one transaction, and `reopen` opens and closes the file `REPEATS` times after the close.
pub(crate) const OPS: [&str; 7] = ["insert", "commit", "get", "scan", "update", "delete", "reopen"];

/// The rows of the table before an operation.
const PRELOAD: i64 = 10_000;

/// The transactions of the `commit` operation.
const COMMITS: i64 = 1_000;

/// The scans of the `scan` operation and the opens of the `reopen` operation, so that each count is large next to the noise.
const REPEATS: usize = 20;

/// Runs the operation `name` of [`OPS`], or `base`.
pub(crate) fn op(name: &str) -> Result<(), String> {
    if name != "base" && !OPS.contains(&name) {
        return Err(format!("{name:?} is not an operation of the set"));
    }
    let io = SimIo::new(1);
    let platform = Platform {
        io: Arc::new(io.clone()),
        clock: Arc::new(SimClock::new(1_800_000_000_000)),
        entropy: Arc::new(SimEntropy::new(7)),
    };
    let options = Options {
        memory_limit: 64 << 20,
        shared_buffers: 16 << 20,
        log_extents: 1,
        window_pages: Some(1 << 16),
        ..Options::default()
    };
    let path = Path::new("m1.rupg");
    let db = Database::open_on(&platform, path, &options).map_err(fail("open"))?;
    let t = db.create_table("t", columns()).map_err(fail("create table"))?;
    let mut tx = db.transaction().map_err(fail("begin"))?;
    let mut ids = Vec::with_capacity(PRELOAD as usize);
    for k in 0..PRELOAD {
        ids.push(tx.insert(&t, &small(k)).map_err(fail("insert"))?);
    }
    tx.commit().map_err(fail("commit"))?;
    let mut tx = db.transaction().map_err(fail("begin"))?;
    match name {
        "insert" => {
            for k in PRELOAD..2 * PRELOAD {
                tx.insert(&t, &small(k)).map_err(fail("insert"))?;
            }
        }
        "commit" => {
            for k in PRELOAD..PRELOAD + COMMITS {
                tx.insert(&t, &small(k)).map_err(fail("insert"))?;
                tx.commit().map_err(fail("commit"))?;
                tx = db.transaction().map_err(fail("begin"))?;
            }
        }
        "get" => {
            for id in &ids {
                tx.get(&t, *id).map_err(fail("get"))?.ok_or("a row is missing")?;
            }
        }
        "scan" => {
            for _ in 0..REPEATS {
                let rows = tx.scan(&t, ..).map_err(fail("scan"))?;
                if rows.len() != ids.len() {
                    return Err(format!("the scan gave {} rows", rows.len()));
                }
            }
        }
        "update" => {
            for (k, id) in ids.iter().enumerate() {
                tx.update(&t, *id, &small(k as i64 + PRELOAD)).map_err(fail("update"))?;
            }
        }
        "delete" => {
            for id in &ids {
                tx.delete(&t, *id).map_err(fail("delete"))?;
            }
        }
        _ => {}
    }
    tx.commit().map_err(fail("commit"))?;
    db.close().map_err(fail("close"))?;
    if name == "reopen" {
        for _ in 0..REPEATS {
            let db = Database::open_on(&platform, path, &options).map_err(fail("open"))?;
            db.close().map_err(fail("close"))?;
        }
    }
    Ok(())
}

/// The options of the recovery file. The ring holds the log with a quarter more for the block headers and the fill, and no automatic checkpoint starts before the log is full. The limit for all rings together is the same as the limit for one ring, because the file has one ring.
fn recovery_options(log_bytes: u64) -> Options {
    let ring = (log_bytes + log_bytes / 4).div_ceil(EXTENT) + 1;
    Options {
        log_extents: ring as usize,
        checkpoint_log: Some(ring * EXTENT),
        max_wal_size: ring * EXTENT,
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
        assert_eq!(o.max_wal_size, o.log_extents as u64 * EXTENT);
    }

    #[test]
    fn the_worker_ring_is_8_mib_for_each_writer_in_extents() {
        assert_eq!(worker_extents(1), 1);
        assert_eq!(worker_extents(2), 1);
        assert_eq!(worker_extents(6), 3);
        assert_eq!(worker_extents(16), 8);
    }

    #[test]
    fn pgbench_output_gives_the_rate_and_the_latency() {
        let out = "number of transactions actually processed: 11130\nlatency average = 2.695 ms\ninitial connection time = 3.120 ms\ntps = 371.034722 (without initial connection time)\n";
        assert_eq!(parse_pgbench(out), Ok((371.034722, 2.695)));
        assert!(parse_pgbench("no numbers").is_err());
    }

    #[test]
    fn each_operation_runs() {
        op("base").unwrap();
        for name in OPS {
            op(name).unwrap();
        }
        assert!(op("nothing").is_err());
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
            postgres: None,
        };
        let (j, _) = empty(&c).unwrap();
        assert!(matches!(j.get("file_bytes"), Some(Json::UInt(n)) if *n > 0));
        latency(&c).unwrap();
        let (j, _) = throughput(&c).unwrap();
        assert!(matches!(j.get("commits"), Some(Json::UInt(n)) if *n > 0));
        fs::remove_dir_all(&dir).unwrap();
    }
}
