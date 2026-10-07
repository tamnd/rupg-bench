//! The YCSB driver of spec/20 section 20.9.
//!
//! The driver follows the core workload of YCSB: the table `usertable` with the key `ycsb_key` and ten text fields of 100 bytes, the keys `user<FNV-1a hash of the record number>`, and the scrambled zipfian request distribution with the constant 0.99. The workloads are A (50 percent reads, 50 percent updates), B (95 and 5), C (reads only) and F (50 percent reads, 50 percent read-modify-write). A read reads all fields, and an update writes one field that the driver picks at random, as in the defaults of YCSB.
//!
//! The driver talks to the server with prepared statements of the extended protocol. Each client has one connection and one thread. With a pipeline depth of 1, a client waits for each result before it sends the next statement. With a depth of D, up to D statements wait for their results at the same time, as in the pipeline mode of `libpq`. Each statement has its own Sync, so each one commits on its own. A read-modify-write is a read and then an update of the same key. Its update does not use the value of the read, as in YCSB, so in a pipeline the update can go before the read result comes back.
//!
//! The latencies go into a log-linear histogram with 128 buckets for each power of two, so a percentile has an error below 1 percent. The mean and the maximum are exact.

use std::io::{BufReader, Read};
use std::sync::Barrier;
use std::time::{Duration, Instant};

use crate::cgroup::{Cgroup, Interval, Usage};
use crate::json::Json;
use crate::pg::{Config, Conn, PgError};

pub(crate) const FIELDS: usize = 10;
pub(crate) const FIELD_LENGTH: usize = 100;
pub(crate) const ZIPFIAN_CONSTANT: f64 = 0.99;
/// The item count and its zeta of the scrambled zipfian generator of YCSB.
const SCRAMBLED_ITEMS: u64 = 10_000_000_000;
const SCRAMBLED_ZETAN: f64 = 26.469_028_201_783_02;

/// `Utils.fnvhash64` of YCSB: FNV-1a over the 8 bytes of the value, low byte first, then the absolute value as a signed number.
pub(crate) fn fnv_hash64(mut v: u64) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for _ in 0..8 {
        h ^= v & 0xff;
        v >>= 8;
        h = h.wrapping_mul(1_099_511_628_211);
    }
    (h as i64).unsigned_abs()
}

/// The key of a record, as `CoreWorkload.buildKeyName` with the hashed insert order.
pub(crate) fn key_name(keynum: u64) -> String {
    format!("user{}", fnv_hash64(keynum))
}

/// A small fast generator (SplitMix64). Each client has its own seed, so a run can be repeated.
#[derive(Clone, Debug)]
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in [0, 1).
    pub(crate) fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub(crate) fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// Fills `out` with letters and digits.
    pub(crate) fn fill_text(&mut self, out: &mut [u8]) {
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        for chunk in out.chunks_mut(8) {
            let mut r = self.next_u64();
            for b in chunk {
                *b = CHARS[(r % CHARS.len() as u64) as usize];
                r /= CHARS.len() as u64;
            }
        }
    }
}

/// The `ZipfianGenerator` of YCSB, from Gray et al., "Quickly Generating Billion-Record Synthetic Databases", SIGMOD 1994. It gives 0 most often.
#[derive(Clone, Debug)]
pub(crate) struct Zipfian {
    items: u64,
    theta: f64,
    alpha: f64,
    zetan: f64,
    eta: f64,
}

fn zeta(n: u64, theta: f64) -> f64 {
    (1..=n).map(|i| 1.0 / (i as f64).powf(theta)).sum()
}

impl Zipfian {
    /// `zetan` is the zeta of `items`, which takes a long time to compute for a large count. None computes it.
    pub(crate) fn new(items: u64, theta: f64, zetan: Option<f64>) -> Zipfian {
        let zetan = zetan.unwrap_or_else(|| zeta(items, theta));
        let zeta2 = zeta(2, theta);
        let alpha = 1.0 / (1.0 - theta);
        let eta = (1.0 - (2.0 / items as f64).powf(1.0 - theta)) / (1.0 - zeta2 / zetan);
        Zipfian { items, theta, alpha, zetan, eta }
    }

    pub(crate) fn next(&self, rng: &mut Rng) -> u64 {
        let u = rng.next_f64();
        let uz = u * self.zetan;
        if uz < 1.0 {
            return 0;
        }
        if uz < 1.0 + 0.5f64.powf(self.theta) {
            return 1;
        }
        let v = (self.items as f64 * (self.eta * u - self.eta + 1.0).powf(self.alpha)) as u64;
        v.min(self.items - 1)
    }
}

/// The `ScrambledZipfianGenerator` of YCSB: the zipfian ranks over 10 billion items, hashed onto the records, so the hot records are spread over the key space.
#[derive(Clone, Debug)]
pub(crate) struct ScrambledZipfian {
    ranks: Zipfian,
    records: u64,
}

impl ScrambledZipfian {
    pub(crate) fn new(records: u64) -> ScrambledZipfian {
        ScrambledZipfian {
            ranks: Zipfian::new(SCRAMBLED_ITEMS + 1, ZIPFIAN_CONSTANT, Some(SCRAMBLED_ZETAN)),
            records,
        }
    }

    pub(crate) fn next(&self, rng: &mut Rng) -> u64 {
        fnv_hash64(self.ranks.next(rng)) % self.records
    }

    /// The record of rank 0, which the generator gives most often.
    pub(crate) fn hottest(&self) -> u64 {
        fnv_hash64(0) % self.records
    }
}

/// The share of each operation in a workload.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Workload {
    pub(crate) name: char,
    pub(crate) read: f64,
    pub(crate) update: f64,
    pub(crate) rmw: f64,
}

impl Workload {
    pub(crate) fn get(name: &str) -> Result<Workload, String> {
        let (n, read, update, rmw) = match name {
            "a" | "A" => ('a', 0.5, 0.5, 0.0),
            "b" | "B" => ('b', 0.95, 0.05, 0.0),
            "c" | "C" => ('c', 1.0, 0.0, 0.0),
            "f" | "F" => ('f', 0.5, 0.0, 0.5),
            _ => return Err(format!("usage: workload {name:?}: the workloads are a, b, c and f")),
        };
        Ok(Workload { name: n, read, update, rmw })
    }

    fn pick(&self, rng: &mut Rng) -> Op {
        let u = rng.next_f64();
        if u < self.read {
            Op::Read
        } else if u < self.read + self.update {
            Op::Update
        } else {
            Op::Rmw
        }
    }
}

/// The operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Op {
    Read = 0,
    Update = 1,
    Rmw = 2,
}

pub(crate) const OPS: [(Op, &str); 3] =
    [(Op::Read, "read"), (Op::Update, "update"), (Op::Rmw, "read_modify_write")];

/// One row of the report: the clients and the pipeline depth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Shape {
    pub(crate) clients: u32,
    pub(crate) depth: u32,
}

impl Shape {
    /// `16` is 16 clients without a pipeline, `16x64` is 16 clients with a pipeline depth of 64.
    pub(crate) fn parse(s: &str) -> Result<Shape, String> {
        let (c, d) = s.split_once('x').unwrap_or((s, "1"));
        let n = |v: &str| v.parse::<u32>().ok().filter(|n| *n > 0);
        match (n(c), n(d)) {
            (Some(clients), Some(depth)) => Ok(Shape { clients, depth }),
            _ => Err(format!("usage: row {s:?} is not CLIENTS or CLIENTSxDEPTH")),
        }
    }

    pub(crate) fn name(&self) -> String {
        if self.depth == 1 {
            format!("{} clients", self.clients)
        } else {
            format!("{} clients, pipeline {}", self.clients, self.depth)
        }
    }
}

/// A log-linear histogram of nanoseconds.
#[derive(Clone, Debug)]
pub(crate) struct Histogram {
    counts: Vec<u64>,
    count: u64,
    sum: u128,
    max: u64,
}

const SUB_BITS: u32 = 7;
const SUB: u64 = 1 << SUB_BITS;
const BUCKETS: usize = ((64 - SUB_BITS as usize) + 1) * SUB as usize;

impl Default for Histogram {
    fn default() -> Histogram {
        Histogram { counts: vec![0; BUCKETS], count: 0, sum: 0, max: 0 }
    }
}

impl Histogram {
    fn index(v: u64) -> usize {
        if v < SUB {
            return v as usize;
        }
        let e = 63 - v.leading_zeros();
        let m = v >> (e - SUB_BITS);
        ((u64::from(e - SUB_BITS) + 1) * SUB + (m - SUB)) as usize
    }

    /// The largest value of bucket `i`.
    fn upper(i: usize) -> u64 {
        let i = i as u64;
        if i < SUB {
            return i;
        }
        let k = i / SUB;
        let m = i % SUB + SUB;
        let shift = k - 1;
        u64::try_from((u128::from(m + 1) << shift) - 1).unwrap_or(u64::MAX)
    }

    pub(crate) fn record(&mut self, nanos: u64) {
        self.counts[Histogram::index(nanos)] += 1;
        self.count += 1;
        self.sum += u128::from(nanos);
        self.max = self.max.max(nanos);
    }

    pub(crate) fn merge(&mut self, other: &Histogram) {
        for (a, b) in self.counts.iter_mut().zip(&other.counts) {
            *a += b;
        }
        self.count += other.count;
        self.sum += other.sum;
        self.max = self.max.max(other.max);
    }

    pub(crate) fn count(&self) -> u64 {
        self.count
    }

    /// The value at the quantile `q`: the largest value of the bucket that has it, and never above the maximum.
    pub(crate) fn quantile(&self, q: f64) -> Option<u64> {
        if self.count == 0 {
            return None;
        }
        let rank = ((q * self.count as f64).ceil() as u64).clamp(1, self.count);
        let mut seen = 0;
        for (i, c) in self.counts.iter().enumerate() {
            seen += c;
            if seen >= rank {
                return Some(Histogram::upper(i).min(self.max));
            }
        }
        Some(self.max)
    }

    pub(crate) fn to_json(&self) -> Json {
        let us = |v: Option<u64>| v.map(|n| (n as f64 / 1000.0 * 10.0).round() / 10.0);
        let mean = (self.count > 0).then(|| (self.sum / u128::from(self.count)) as u64);
        Json::obj()
            .with("count", self.count)
            .with("mean_us", us(mean))
            .with("p50_us", us(self.quantile(0.50)))
            .with("p95_us", us(self.quantile(0.95)))
            .with("p99_us", us(self.quantile(0.99)))
            .with("p999_us", us(self.quantile(0.999)))
            .with("max_us", us((self.count > 0).then_some(self.max)))
    }
}

/// The numbers of one client, or of all clients after `merge`.
#[derive(Clone, Debug, Default)]
pub(crate) struct Tally {
    pub(crate) latency: [Histogram; 3],
    pub(crate) statements: u64,
    pub(crate) errors: u64,
    pub(crate) first_error: Option<String>,
    pub(crate) hot_updates: u64,
    pub(crate) elapsed: Duration,
    /// The acknowledged updates, for `check_final`.
    pub(crate) writes: Vec<Write>,
}

impl Tally {
    pub(crate) fn ops(&self) -> u64 {
        self.latency.iter().map(Histogram::count).sum()
    }

    pub(crate) fn merge(&mut self, mut other: Tally) {
        for (a, b) in self.latency.iter_mut().zip(&other.latency) {
            a.merge(b);
        }
        self.writes.append(&mut other.writes);
        self.statements += other.statements;
        self.errors += other.errors;
        if self.first_error.is_none() {
            self.first_error = other.first_error;
        }
        self.hot_updates += other.hot_updates;
        self.elapsed = self.elapsed.max(other.elapsed);
    }

    fn error(&mut self, e: String) {
        self.errors += 1;
        if self.first_error.is_none() {
            self.first_error = Some(e);
        }
    }
}

/// The SQL of the table.
pub(crate) fn create_table() -> String {
    let fields: Vec<String> = (0..FIELDS).map(|i| format!("field{i} TEXT")).collect();
    format!(
        "DROP TABLE IF EXISTS usertable; CREATE TABLE usertable (ycsb_key VARCHAR(255) PRIMARY KEY, {})",
        fields.join(", ")
    )
}

/// The rows of the load in the text format of COPY, made as they are read.
struct LoadRows {
    next: u64,
    records: u64,
    rng: Rng,
    buf: Vec<u8>,
    at: usize,
}

impl Read for LoadRows {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.at == self.buf.len() {
            self.buf.clear();
            self.at = 0;
            while self.buf.len() < 1 << 16 && self.next < self.records {
                self.buf.extend_from_slice(key_name(self.next).as_bytes());
                for _ in 0..FIELDS {
                    self.buf.push(b'\t');
                    let start = self.buf.len();
                    self.buf.resize(start + FIELD_LENGTH, 0);
                    self.rng.fill_text(&mut self.buf[start..]);
                }
                self.buf.push(b'\n');
                self.next += 1;
            }
        }
        let n = out.len().min(self.buf.len() - self.at);
        out[..n].copy_from_slice(&self.buf[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

/// Makes the table and loads `records` rows with COPY. It returns the row count that the server reported.
pub(crate) fn load(conn: &mut Conn, records: u64, seed: u64) -> Result<u64, String> {
    conn.simple(&create_table())?;
    let rows = LoadRows { next: 0, records, rng: Rng::new(seed), buf: Vec::new(), at: 0 };
    let mut input = BufReader::with_capacity(1 << 16, rows);
    let tag = conn.copy_in("COPY usertable FROM STDIN", &mut input, None)?;
    let n: u64 = tag
        .strip_prefix("COPY ")
        .and_then(|n| n.parse().ok())
        .ok_or(format!("COPY returned {tag:?}"))?;
    if n != records {
        return Err(format!("COPY loaded {n} rows, not {records}"));
    }
    conn.simple("VACUUM ANALYZE usertable")?;
    conn.simple("CHECKPOINT")?;
    Ok(n)
}

/// The record count of the table, to check that a run starts on a loaded table.
pub(crate) fn count(conn: &mut Conn) -> Result<u64, String> {
    let r = conn.query("SELECT count(*) FROM usertable")?;
    r.rows
        .first()
        .and_then(|row| row.first())
        .and_then(|v| v.as_deref())
        .and_then(|v| v.parse().ok())
        .ok_or("the count returned no number".to_owned())
}

fn prepare(conn: &mut Conn) -> Result<(), PgError> {
    let fields: Vec<String> = (0..FIELDS).map(|i| format!("field{i}")).collect();
    conn.prepare(
        "read",
        &format!("SELECT ycsb_key, {} FROM usertable WHERE ycsb_key = $1", fields.join(", ")),
    )?;
    for (i, f) in fields.iter().enumerate() {
        conn.prepare(
            &format!("update{i}"),
            &format!("UPDATE usertable SET {f} = $1 WHERE ycsb_key = $2"),
        )?;
    }
    Ok(())
}

/// A statement that waits for its result.
struct Waiting {
    op: Op,
    /// The field of an update, None for a read.
    field: Option<u8>,
    last: bool,
    key: u64,
    /// The start of the operation, and the time when this statement went into the send buffer.
    start: Instant,
    queued: Instant,
    print: u64,
}

/// One acknowledged update, for the check after the run. The times are nanoseconds since the start of the row. `sent` is when the update went into the send buffer, which is not later than the time when it went to the server. `acked` is when the client read the result, which is not earlier than the commit. So the check can only miss a fault, it cannot report a false one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Write {
    pub(crate) key: u64,
    pub(crate) field: u8,
    pub(crate) sent: u64,
    pub(crate) acked: u64,
    pub(crate) print: u64,
}

/// The FNV-1a hash of a value, to keep a small print of each write.
pub(crate) fn print(value: &[u8]) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in value {
        h ^= u64::from(*b);
        h = h.wrapping_mul(1_099_511_628_211);
    }
    h
}

/// The settings of one run.
#[derive(Clone, Debug)]
pub(crate) struct RunSettings {
    pub(crate) workload: Workload,
    pub(crate) shape: Shape,
    pub(crate) records: u64,
    pub(crate) time: Duration,
    pub(crate) seed: u64,
    /// A number for each row of one command, mixed into the seed, so two rows do not write the same values.
    pub(crate) row: u64,
    /// `on` or `off` sets `synchronous_commit` for each connection. None keeps the setting of the server.
    pub(crate) sync: Option<String>,
}

fn nanos_since(epoch: Instant, t: Instant) -> u64 {
    u64::try_from(t.saturating_duration_since(epoch).as_nanos()).unwrap_or(u64::MAX)
}

/// One client: it opens its connection, waits at `ready`, runs until the time is over at `go`, and returns its numbers.
fn client(
    config: &Config,
    s: &RunSettings,
    index: u64,
    (ready, go): (&Barrier, &Barrier),
    epoch: &std::sync::OnceLock<Instant>,
) -> Result<Tally, String> {
    let opened = (|| -> Result<Conn, PgError> {
        let mut conn = Conn::connect(config)?;
        if let Some(v) = &s.sync {
            conn.simple(&format!("SET synchronous_commit = {v}"))?;
        }
        prepare(&mut conn)?;
        Ok(conn)
    })();
    // Wait at both barriers also after a failure, so the other threads do not wait for ever.
    ready.wait();
    go.wait();
    let mut conn = opened.map_err(|e| format!("client {index}: {e}"))?;
    let epoch = *epoch.get().ok_or("the row has no start time")?;
    let keys = ScrambledZipfian::new(s.records);
    let hottest = keys.hottest();
    let mut rng =
        Rng::new(s.seed ^ fnv_hash64(index + 1) ^ fnv_hash64(s.row.wrapping_add(1 << 32)));
    let mut tally = Tally::default();
    let mut waiting = std::collections::VecDeque::with_capacity(s.shape.depth as usize + 1);
    // The update of a read-modify-write, after its read.
    let mut next_update: Option<(u64, Instant)> = None;
    let mut value = [0u8; FIELD_LENGTH];
    let start = Instant::now();
    let end = start + s.time;
    loop {
        let over = Instant::now() >= end;
        // Fill the pipeline: first the update of a read-modify-write, then new operations.
        while waiting.len() < s.shape.depth as usize {
            let (op, update, last, key, t) = if let Some((key, t)) = next_update.take() {
                (Op::Rmw, true, true, key, t)
            } else if over {
                break;
            } else {
                let op = s.workload.pick(&mut rng);
                let key = keys.next(&mut rng);
                let t = Instant::now();
                if op == Op::Rmw {
                    next_update = Some((key, t));
                }
                (op, op == Op::Update, op != Op::Rmw, key, t)
            };
            let k = key_name(key);
            let (field, p) = if update {
                rng.fill_text(&mut value);
                let field = rng.below(FIELDS as u64) as u8;
                conn.queue(&format!("update{field}"), &[&value, k.as_bytes()])?;
                if key == hottest {
                    tally.hot_updates += 1;
                }
                (Some(field), print(&value))
            } else {
                conn.queue("read", &[k.as_bytes()])?;
                (None, 0)
            };
            waiting.push_back(Waiting {
                op,
                field,
                last,
                key,
                start: t,
                queued: Instant::now(),
                print: p,
            });
        }
        let Some(w) = waiting.pop_front() else { break };
        conn.flush()?;
        let done = conn.next_done()?;
        tally.statements += 1;
        match (done, w.field) {
            (Ok(d), Some(field)) => {
                if d.tag == "UPDATE 1" {
                    tally.writes.push(Write {
                        key: w.key,
                        field,
                        sent: nanos_since(epoch, w.queued),
                        acked: nanos_since(epoch, Instant::now()),
                        print: w.print,
                    });
                } else {
                    tally.error(format!("an update of {} returned {:?}", key_name(w.key), d.tag));
                }
            }
            (Ok(d), None) => {
                if d.rows != 1 || usize::from(d.fields) != FIELDS + 1 {
                    tally.error(format!(
                        "a read of {} returned {} rows of {} fields",
                        key_name(w.key),
                        d.rows,
                        d.fields
                    ));
                } else if d.first.as_deref() != Some(key_name(w.key).as_bytes()) {
                    tally.error(format!(
                        "a read of {} returned the row of {}",
                        key_name(w.key),
                        String::from_utf8_lossy(d.first.as_deref().unwrap_or_default())
                    ));
                }
            }
            (Err(e), _) => tally.error(e.to_string()),
        }
        if w.last {
            let nanos = u64::try_from(w.start.elapsed().as_nanos()).unwrap_or(u64::MAX);
            tally.latency[w.op as usize].record(nanos);
        }
    }
    tally.elapsed = start.elapsed();
    Ok(tally)
}

/// Runs the clients of one row and returns their merged numbers and the use of the server cgroup. The measure starts after all clients have connected and stops after the last one ends.
pub(crate) fn run(config: &Config, s: &RunSettings, cg: &Cgroup) -> Result<(Tally, Usage), String> {
    let n = s.shape.clients as usize;
    let ready = Barrier::new(n + 1);
    let go = Barrier::new(n + 1);
    let epoch = std::sync::OnceLock::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..n)
            .map(|i| {
                let (ready, go, epoch) = (&ready, &go, &epoch);
                scope.spawn(move || client(config, s, i as u64, (ready, go), epoch))
            })
            .collect();
        ready.wait();
        let interval = Interval::start(cg);
        let _ = epoch.set(Instant::now());
        go.wait();
        let mut total = Tally::default();
        let mut failed = None;
        for h in handles {
            match h.join() {
                Ok(Ok(t)) => total.merge(t),
                Ok(Err(e)) => failed = failed.or(Some(e)),
                Err(_) => failed = failed.or(Some("a client thread panicked".to_owned())),
            }
        }
        let usage = interval?.finish()?;
        match failed {
            Some(e) => Err(e),
            None => Ok((total, usage)),
        }
    })
}

/// The result of the check after a run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FinalCheck {
    pub(crate) writes: u64,
    pub(crate) fields: u64,
    pub(crate) bad: u64,
    pub(crate) examples: Vec<String>,
}

impl FinalCheck {
    pub(crate) fn to_json(&self) -> Json {
        Json::obj()
            .with("acknowledged_updates", self.writes)
            .with("fields_checked", self.fields)
            .with("fields_wrong", self.bad)
            .with("examples", self.examples.clone())
    }
}

/// For each field that the run updated, the prints of the writes that may hold it at the end. A write is replaced when another update of the field went to the server after the first was acknowledged. The value at the end must be one of the writes that no such update replaced.
pub(crate) fn allowed(writes: &mut [Write]) -> Vec<((u64, u8), Vec<u64>)> {
    writes.sort_unstable_by_key(|w| (w.key, w.field));
    let mut out = Vec::new();
    for group in writes.chunk_by(|a, b| (a.key, a.field) == (b.key, b.field)) {
        let last_sent = group.iter().map(|w| w.sent).max().unwrap_or(0);
        let prints = group.iter().filter(|w| w.acked >= last_sent).map(|w| w.print).collect();
        out.push(((group[0].key, group[0].field), prints));
    }
    out
}

/// Checks the table after a run: each field that the run updated holds a write that no later acknowledged update replaced. The run must be over, so the table does not change.
pub(crate) fn check_final(conn: &mut Conn, mut writes: Vec<Write>) -> Result<FinalCheck, String> {
    let mut check = FinalCheck { writes: writes.len() as u64, ..FinalCheck::default() };
    let allowed = allowed(&mut writes);
    let fields: Vec<String> = (0..FIELDS).map(|i| format!("field{i}")).collect();
    for batch in allowed.chunk_by(|a, b| a.0.0 == b.0.0).collect::<Vec<_>>().chunks(500) {
        let names: Vec<String> =
            batch.iter().map(|g| format!("'{}'", key_name(g[0].0.0))).collect();
        let sql = format!(
            "SELECT ycsb_key, {} FROM usertable WHERE ycsb_key IN ({})",
            fields.join(", "),
            names.join(", ")
        );
        let rows = conn.query(&sql)?;
        let by_key: std::collections::HashMap<&str, &Vec<Option<String>>> = rows
            .rows
            .iter()
            .filter_map(|r| r.first().and_then(|k| k.as_deref()).map(|k| (k, r)))
            .collect();
        for group in batch {
            let name = key_name(group[0].0.0);
            for ((_, field), prints) in *group {
                check.fields += 1;
                let stored = by_key
                    .get(name.as_str())
                    .and_then(|r| r.get(usize::from(*field) + 1))
                    .and_then(|v| v.as_deref());
                let ok = stored.is_some_and(|v| prints.contains(&print(v.as_bytes())));
                if !ok {
                    check.bad += 1;
                    if check.examples.len() < 5 {
                        check.examples.push(format!(
                            "{name} field{field}: the value is not one of the {} last acknowledged updates",
                            prints.len()
                        ));
                    }
                }
            }
        }
    }
    Ok(check)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_and_keys() {
        // FNV-1a of eight zero bytes.
        let mut h: u64 = 0xCBF2_9CE4_8422_2325;
        for _ in 0..8 {
            h = h.wrapping_mul(1_099_511_628_211);
        }
        assert_eq!(fnv_hash64(0), (h as i64).unsigned_abs());
        assert!(key_name(1).starts_with("user"));
        assert_ne!(key_name(1), key_name(2));
    }

    #[test]
    fn zipfian_is_skewed() {
        let z = Zipfian::new(1000, ZIPFIAN_CONSTANT, None);
        let mut rng = Rng::new(7);
        let mut counts = vec![0u32; 1000];
        for _ in 0..200_000 {
            counts[z.next(&mut rng) as usize] += 1;
        }
        // The share of rank 0 is 1 / zeta(1000, 0.99), about 13 percent.
        let share = f64::from(counts[0]) / 200_000.0;
        let expected = 1.0 / zeta(1000, ZIPFIAN_CONSTANT);
        assert!((share - expected).abs() < 0.01, "{share} {expected}");
        assert!(counts[0] > counts[1] && counts[1] > counts[10] && counts[10] > counts[500]);
        let s = ScrambledZipfian::new(1000);
        for _ in 0..1000 {
            assert!(s.next(&mut rng) < 1000);
        }
    }

    #[test]
    fn workloads_and_rows() {
        assert_eq!(Workload::get("f").unwrap().rmw, 0.5);
        assert!(Workload::get("e").is_err());
        assert_eq!(Shape::parse("16x64").unwrap(), Shape { clients: 16, depth: 64 });
        assert_eq!(Shape::parse("1").unwrap(), Shape { clients: 1, depth: 1 });
        assert!(Shape::parse("0").is_err() && Shape::parse("4x").is_err());
    }

    #[test]
    fn histogram_quantiles() {
        let mut h = Histogram::default();
        for v in 1..=100_000u64 {
            h.record(v * 1000);
        }
        for (q, exact) in [(0.5, 50_000_000.0), (0.99, 99_000_000.0)] {
            let got = h.quantile(q).unwrap() as f64;
            assert!(got >= exact && got < exact * 1.01, "{q}: {got}");
        }
        assert_eq!(h.quantile(1.0), Some(100_000_000));
        for v in [0, 1, 127, 128, 129, 1000, 123_456_789, u64::MAX] {
            let i = Histogram::index(v);
            assert!(Histogram::upper(i) >= v);
            assert!(i == 0 || Histogram::upper(i - 1) < v, "{v}");
        }
        assert!(Histogram::default().quantile(0.5).is_none());
    }

    #[test]
    fn load_rows() {
        let mut r = LoadRows { next: 0, records: 3, rng: Rng::new(1), buf: Vec::new(), at: 0 };
        let mut text = String::new();
        r.read_to_string(&mut text).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        let fields: Vec<&str> = lines[1].split('\t').collect();
        assert_eq!(fields[0], key_name(1));
        assert_eq!(fields.len(), FIELDS + 1);
        assert!(fields[1..].iter().all(|f| f.len() == FIELD_LENGTH));
    }

    #[test]
    fn allowed_writes() {
        let w = |key, field, sent, acked, print| Write { key, field, sent, acked, print };
        let mut writes = vec![
            // Key 1, field 0: write 10 was acknowledged before write 11 was sent, so only 11 may stay.
            w(1, 0, 0, 5, 10),
            w(1, 0, 6, 9, 11),
            // Key 1, field 1: the two writes overlap, so either may stay.
            w(1, 1, 0, 8, 20),
            w(1, 1, 3, 7, 21),
            w(2, 3, 1, 2, 30),
        ];
        let a = allowed(&mut writes);
        assert_eq!(a, vec![((1, 0), vec![11]), ((1, 1), vec![20, 21]), ((2, 3), vec![30])]);
    }
}
