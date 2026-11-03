//! The YCSB driver for DuckDB in the process of the driver, through the C API of `libduckdb`.
//!
//! DuckDB has no network protocol, so its clients are threads of this process, each with its own connection to one database. Each thread runs one operation at a time with prepared statements, so a row is CLIENTS with no pipeline. The keys, the values, the distribution and the check after the run are the same as in [`crate::ycsb`]. Each statement commits on its own. A write that conflicts with a write of another transaction fails in DuckDB, and the driver runs it again: the latency of the operation includes the retries, and the row counts them.
//!
//! The CPU time is the time of the whole process from `getrusage`, the engine and the clients together, because both run in the same threads. The report says so. A database at `:memory:` has no durability, and a database in a file writes its WAL at each commit.

use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::Barrier;
use std::time::Instant;

use crate::ycsb::{
    FIELD_LENGTH, FIELDS, Op, Rng, RunSettings, ScrambledZipfian, Tally, Write, allowed,
    fnv_hash64, key_name, print,
};

type Idx = u64;

/// `duckdb_result`: the caller owns the memory and the library fills it.
#[repr(C)]
struct DuckResult {
    column_count: Idx,
    row_count: Idx,
    rows_changed: Idx,
    columns: *mut c_void,
    error_message: *mut c_char,
    internal_data: *mut c_void,
}

impl DuckResult {
    fn new() -> DuckResult {
        DuckResult {
            column_count: 0,
            row_count: 0,
            rows_changed: 0,
            columns: std::ptr::null_mut(),
            error_message: std::ptr::null_mut(),
            internal_data: std::ptr::null_mut(),
        }
    }
}

type Handle = *mut c_void;

#[link(name = "duckdb")]
unsafe extern "C" {
    fn duckdb_open(path: *const c_char, out: *mut Handle) -> u32;
    fn duckdb_close(db: *mut Handle);
    fn duckdb_connect(db: Handle, out: *mut Handle) -> u32;
    fn duckdb_disconnect(conn: *mut Handle);
    fn duckdb_query(conn: Handle, sql: *const c_char, out: *mut DuckResult) -> u32;
    fn duckdb_prepare(conn: Handle, sql: *const c_char, out: *mut Handle) -> u32;
    fn duckdb_prepare_error(stmt: Handle) -> *const c_char;
    fn duckdb_destroy_prepare(stmt: *mut Handle);
    fn duckdb_bind_varchar_length(stmt: Handle, index: Idx, value: *const c_char, len: Idx) -> u32;
    fn duckdb_execute_prepared(stmt: Handle, out: *mut DuckResult) -> u32;
    fn duckdb_result_error(result: *mut DuckResult) -> *const c_char;
    fn duckdb_destroy_result(result: *mut DuckResult);
    fn duckdb_row_count(result: *mut DuckResult) -> Idx;
    fn duckdb_column_count(result: *mut DuckResult) -> Idx;
    fn duckdb_rows_changed(result: *mut DuckResult) -> Idx;
    fn duckdb_value_varchar(result: *mut DuckResult, col: Idx, row: Idx) -> *mut c_char;
    fn duckdb_value_int64(result: *mut DuckResult, col: Idx, row: Idx) -> i64;
    fn duckdb_free(ptr: *mut c_void);
    fn duckdb_appender_create(
        conn: Handle,
        schema: *const c_char,
        table: *const c_char,
        out: *mut Handle,
    ) -> u32;
    fn duckdb_append_varchar_length(app: Handle, value: *const c_char, len: Idx) -> u32;
    fn duckdb_appender_end_row(app: Handle) -> u32;
    fn duckdb_appender_error(app: Handle) -> *const c_char;
    fn duckdb_appender_destroy(app: *mut Handle) -> u32;
}

/// The text of a C string of the library, which the library owns.
fn text(p: *const c_char) -> String {
    if p.is_null() {
        return "no message".to_owned();
    }
    // SAFETY: the library returns a valid string with a terminating zero that lives until the next call on its object, and the text is copied before that.
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

fn c(s: &str) -> Result<CString, String> {
    CString::new(s).map_err(|_| format!("the text {s:?} has a zero byte"))
}

/// One database. The connections of the threads borrow it.
pub(crate) struct Database(Handle);

// SAFETY: a `duckdb_database` may be used by many threads at the same time, each with its own connection. That is the model of the C API.
unsafe impl Sync for Database {}
// SAFETY: as for `Sync`, the handle does not belong to one thread.
unsafe impl Send for Database {}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Database")
    }
}

impl Database {
    pub(crate) fn open(path: &str) -> Result<Database, String> {
        let p = c(path)?;
        let mut db: Handle = std::ptr::null_mut();
        // SAFETY: `p` is a valid string and `db` is a place for the handle.
        if unsafe { duckdb_open(p.as_ptr(), &mut db) } != 0 {
            return Err(format!("duckdb_open {path:?} failed"));
        }
        Ok(Database(db))
    }

    pub(crate) fn connect(&self) -> Result<Connection, String> {
        let mut conn: Handle = std::ptr::null_mut();
        // SAFETY: the database is open until `drop`, which comes after the connections of the scope end.
        if unsafe { duckdb_connect(self.0, &mut conn) } != 0 {
            return Err("duckdb_connect failed".to_owned());
        }
        Ok(Connection(conn))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // SAFETY: the handle came from `duckdb_open` and no connection uses it now.
        unsafe { duckdb_close(&mut self.0) }
    }
}

/// A connection of one thread.
pub(crate) struct Connection(Handle);

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Connection")
    }
}

/// A result that frees itself.
pub(crate) struct Answer(DuckResult);

impl Drop for Answer {
    fn drop(&mut self) {
        // SAFETY: the result was filled by a call of the library, also on an error.
        unsafe { duckdb_destroy_result(&mut self.0) }
    }
}

impl Answer {
    fn error(&mut self) -> String {
        // SAFETY: the result is valid until `drop`.
        text(unsafe { duckdb_result_error(&mut self.0) })
    }

    fn rows(&mut self) -> u64 {
        // SAFETY: as in `error`.
        unsafe { duckdb_row_count(&mut self.0) }
    }

    fn columns(&mut self) -> u64 {
        // SAFETY: as in `error`.
        unsafe { duckdb_column_count(&mut self.0) }
    }

    fn changed(&mut self) -> u64 {
        // SAFETY: as in `error`.
        unsafe { duckdb_rows_changed(&mut self.0) }
    }

    fn int(&mut self, col: u64, row: u64) -> i64 {
        // SAFETY: as in `error`. The library checks the place and gives 0 out of range.
        unsafe { duckdb_value_int64(&mut self.0, col, row) }
    }

    /// The value as text, or None for a null.
    fn varchar(&mut self, col: u64, row: u64) -> Option<String> {
        // SAFETY: as in `int`. The library gives a new string, or null for a null value.
        let p = unsafe { duckdb_value_varchar(&mut self.0, col, row) };
        if p.is_null() {
            return None;
        }
        let s = text(p);
        // SAFETY: `p` came from `duckdb_value_varchar`, which the caller frees with `duckdb_free`.
        unsafe { duckdb_free(p.cast()) };
        Some(s)
    }
}

impl Connection {
    pub(crate) fn query(&self, sql: &str) -> Result<Answer, String> {
        let s = c(sql)?;
        let mut a = Answer(DuckResult::new());
        // SAFETY: the connection is open and `a` is a place for the result.
        if unsafe { duckdb_query(self.0, s.as_ptr(), &mut a.0) } != 0 {
            return Err(format!("{sql}: {}", a.error()));
        }
        Ok(a)
    }

    fn prepare(&self, sql: &str) -> Result<Prepared, String> {
        let s = c(sql)?;
        let mut stmt: Handle = std::ptr::null_mut();
        // SAFETY: the connection is open and `stmt` is a place for the handle. The library makes a handle also on an error, for the message.
        let failed = unsafe { duckdb_prepare(self.0, s.as_ptr(), &mut stmt) } != 0;
        let p = Prepared(stmt);
        if failed {
            // SAFETY: the handle is valid until `drop`.
            return Err(format!("{sql}: {}", text(unsafe { duckdb_prepare_error(p.0) })));
        }
        Ok(p)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // SAFETY: the handle came from `duckdb_connect`.
        unsafe { duckdb_disconnect(&mut self.0) }
    }
}

struct Prepared(Handle);

impl Drop for Prepared {
    fn drop(&mut self) {
        // SAFETY: the handle came from `duckdb_prepare`.
        unsafe { duckdb_destroy_prepare(&mut self.0) }
    }
}

impl Prepared {
    /// Binds the values to `$1`, `$2` and so on, and runs the statement.
    fn run(&self, values: &[&[u8]]) -> Result<Answer, String> {
        for (i, v) in values.iter().enumerate() {
            // SAFETY: the statement is valid and the library copies the bytes of the value.
            let bound = unsafe {
                duckdb_bind_varchar_length(self.0, i as u64 + 1, v.as_ptr().cast(), v.len() as u64)
            };
            if bound != 0 {
                return Err(format!("binding parameter {} failed", i + 1));
            }
        }
        let mut a = Answer(DuckResult::new());
        // SAFETY: the statement is valid and has a value for each parameter.
        if unsafe { duckdb_execute_prepared(self.0, &mut a.0) } != 0 {
            return Err(a.error());
        }
        Ok(a)
    }
}

/// `getrusage(RUSAGE_SELF)`: the user and system CPU time of the process in microseconds, and the peak resident size in KiB.
pub(crate) fn self_usage() -> (u64, u64, u64) {
    unsafe extern "C" {
        fn getrusage(who: i32, usage: *mut [i64; 18]) -> i32;
    }
    let mut r = [0i64; 18];
    // SAFETY: on Linux on 64-bit machines `struct rusage` is two `struct timeval` of two longs each and fourteen longs, which is 18 longs.
    if unsafe { getrusage(0, &mut r) } != 0 {
        return (0, 0, 0);
    }
    let us = |s: i64, u: i64| u64::try_from(s * 1_000_000 + u).unwrap_or(0);
    (us(r[0], r[1]), us(r[2], r[3]), u64::try_from(r[4]).unwrap_or(0))
}

/// Makes usertable with `records` rows through an appender, then a checkpoint.
pub(crate) fn load(db: &Database, records: u64, seed: u64) -> Result<u64, String> {
    let conn = db.connect()?;
    let fields: Vec<String> = (0..FIELDS).map(|i| format!("field{i} VARCHAR")).collect();
    conn.query(&format!(
        "DROP TABLE IF EXISTS usertable; CREATE TABLE usertable (ycsb_key VARCHAR PRIMARY KEY, {})",
        fields.join(", ")
    ))?;
    let mut app: Handle = std::ptr::null_mut();
    let (schema, table) = (c("main")?, c("usertable")?);
    // SAFETY: the connection is open and the names are valid strings.
    if unsafe { duckdb_appender_create(conn.0, schema.as_ptr(), table.as_ptr(), &mut app) } != 0 {
        return Err("duckdb_appender_create failed".to_owned());
    }
    let mut rng = Rng::new(seed);
    let mut value = [0u8; FIELD_LENGTH];
    let mut failed = None;
    'rows: for n in 0..records {
        let key = key_name(n);
        // SAFETY: the appender is open and the library copies each value.
        let mut state =
            unsafe { duckdb_append_varchar_length(app, key.as_ptr().cast(), key.len() as u64) };
        for _ in 0..FIELDS {
            rng.fill_text(&mut value);
            // SAFETY: as above.
            state |= unsafe {
                duckdb_append_varchar_length(app, value.as_ptr().cast(), FIELD_LENGTH as u64)
            };
        }
        // SAFETY: as above, with one value for each column.
        state |= unsafe { duckdb_appender_end_row(app) };
        if state != 0 {
            // SAFETY: the appender is open.
            failed = Some(text(unsafe { duckdb_appender_error(app) }));
            break 'rows;
        }
    }
    // SAFETY: the appender came from `duckdb_appender_create`. Destroy flushes the rows that it holds.
    let flushed = unsafe { duckdb_appender_destroy(&mut app) };
    if let Some(e) = failed {
        return Err(format!("the load failed: {e}"));
    }
    if flushed != 0 {
        return Err("the appender failed at its flush".to_owned());
    }
    conn.query("CHECKPOINT")?;
    count(db)
}

pub(crate) fn count(db: &Database) -> Result<u64, String> {
    let conn = db.connect()?;
    let mut a = conn.query("SELECT count(*) FROM usertable")?;
    Ok(u64::try_from(a.int(0, 0)).unwrap_or(0))
}

/// True for the error of a write that conflicts with a write of another transaction.
fn conflict(e: &str) -> bool {
    e.contains("onflict")
}

fn nanos_since(epoch: Instant, t: Instant) -> u64 {
    u64::try_from(t.saturating_duration_since(epoch).as_nanos()).unwrap_or(u64::MAX)
}

/// The numbers of one thread, with the retries after conflicts.
fn client(
    db: &Database,
    s: &RunSettings,
    index: u64,
    (ready, go): (&Barrier, &Barrier),
    epoch: &std::sync::OnceLock<Instant>,
) -> Result<(Tally, u64), String> {
    let opened = (|| -> Result<_, String> {
        let conn = db.connect()?;
        let fields: Vec<String> = (0..FIELDS).map(|i| format!("field{i}")).collect();
        let read = conn.prepare(&format!(
            "SELECT ycsb_key, {} FROM usertable WHERE ycsb_key = $1",
            fields.join(", ")
        ))?;
        let updates = fields
            .iter()
            .map(|f| conn.prepare(&format!("UPDATE usertable SET {f} = $1 WHERE ycsb_key = $2")))
            .collect::<Result<Vec<_>, String>>()?;
        Ok((conn, read, updates))
    })();
    ready.wait();
    go.wait();
    let (_conn, read, updates) = opened.map_err(|e| format!("client {index}: {e}"))?;
    let epoch = *epoch.get().ok_or("the row has no start time")?;
    let keys = ScrambledZipfian::new(s.records);
    let hottest = keys.hottest();
    let mut rng =
        Rng::new(s.seed ^ fnv_hash64(index + 1) ^ fnv_hash64(s.row.wrapping_add(1 << 32)));
    let mut tally = Tally::default();
    let mut retries = 0;
    let mut value = [0u8; FIELD_LENGTH];
    let start = Instant::now();
    let end = start + s.time;
    while Instant::now() < end {
        let op = s.workload.pick(&mut rng);
        let key = keys.next(&mut rng);
        let k = key_name(key);
        let t = Instant::now();
        if op != Op::Update {
            tally.statements += 1;
            match read.run(&[k.as_bytes()]) {
                Ok(mut a) => {
                    if a.rows() != 1 || a.columns() != FIELDS as u64 + 1 {
                        tally.error(format!("a read of {k} returned {} rows", a.rows()));
                    } else if a.varchar(0, 0).as_deref() != Some(k.as_str()) {
                        tally.error(format!("a read of {k} returned another row"));
                    }
                }
                Err(e) => tally.error(e),
            }
        }
        if op != Op::Read {
            rng.fill_text(&mut value);
            let field = rng.below(FIELDS as u64) as u8;
            if key == hottest {
                tally.hot_updates += 1;
            }
            loop {
                tally.statements += 1;
                let queued = Instant::now();
                match updates[usize::from(field)]
                    .run(&[&value, k.as_bytes()])
                    .map(|mut a| a.changed())
                {
                    Ok(1) => {
                        tally.writes.push(Write {
                            key,
                            field,
                            sent: nanos_since(epoch, queued),
                            acked: nanos_since(epoch, Instant::now()),
                            print: print(&value),
                        });
                    }
                    Ok(n) => tally.error(format!("an update of {k} changed {n} rows")),
                    Err(e) if conflict(&e) => {
                        retries += 1;
                        continue;
                    }
                    Err(e) => tally.error(e),
                }
                break;
            }
        }
        let nanos = u64::try_from(t.elapsed().as_nanos()).unwrap_or(u64::MAX);
        tally.latency[op as usize].record(nanos);
    }
    tally.elapsed = start.elapsed();
    Ok((tally, retries))
}

/// The use of the process over one row.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProcessUsage {
    pub(crate) user_usec: u64,
    pub(crate) system_usec: u64,
    pub(crate) peak_kib: u64,
}

/// Runs the threads of one row and returns their merged numbers, the retries and the CPU time of the process over the row.
pub(crate) fn run(db: &Database, s: &RunSettings) -> Result<(Tally, u64, ProcessUsage), String> {
    let n = s.shape.clients as usize;
    let ready = Barrier::new(n + 1);
    let go = Barrier::new(n + 1);
    let epoch = std::sync::OnceLock::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..n)
            .map(|i| {
                let (ready, go, epoch) = (&ready, &go, &epoch);
                scope.spawn(move || client(db, s, i as u64, (ready, go), epoch))
            })
            .collect();
        ready.wait();
        let (u0, s0, _) = self_usage();
        let _ = epoch.set(Instant::now());
        go.wait();
        let mut total = Tally::default();
        let mut retries = 0;
        let mut failed = None;
        for h in handles {
            match h.join() {
                Ok(Ok((t, r))) => {
                    total.merge(t);
                    retries += r;
                }
                Ok(Err(e)) => failed = failed.or(Some(e)),
                Err(_) => failed = failed.or(Some("a client thread panicked".to_owned())),
            }
        }
        let (u1, s1, peak) = self_usage();
        let usage = ProcessUsage { user_usec: u1 - u0, system_usec: s1 - s0, peak_kib: peak };
        match failed {
            Some(e) => Err(e),
            None => Ok((total, retries, usage)),
        }
    })
}

/// The check of [`crate::ycsb::check_final`] on DuckDB: each field that the run updated holds a write that no later acknowledged update replaced.
pub(crate) fn check_final(
    db: &Database,
    mut writes: Vec<Write>,
) -> Result<crate::ycsb::FinalCheck, String> {
    let mut check = crate::ycsb::FinalCheck { writes: writes.len() as u64, ..Default::default() };
    let conn = db.connect()?;
    let fields: Vec<String> = (0..FIELDS).map(|i| format!("field{i}")).collect();
    let read = conn.prepare(&format!(
        "SELECT ycsb_key, {} FROM usertable WHERE ycsb_key = $1",
        fields.join(", ")
    ))?;
    for ((key, field), prints) in allowed(&mut writes) {
        check.fields += 1;
        let name = key_name(key);
        let mut a = read.run(&[name.as_bytes()])?;
        let stored = a.varchar(u64::from(field) + 1, 0);
        if !stored.is_some_and(|v| prints.contains(&print(v.as_bytes()))) {
            check.bad += 1;
            if check.examples.len() < 5 {
                check.examples.push(format!(
                    "{name} field{field}: the value is not one of the {} last acknowledged updates",
                    prints.len()
                ));
            }
        }
    }
    Ok(check)
}
