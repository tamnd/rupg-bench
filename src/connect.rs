//! The connection rate of spec/06 section 6.18 of tamnd/rupg.
//!
//! Each of `--clients` threads opens a connection, reads the messages of the server until `ReadyForQuery`, sends `Terminate` and closes the socket. It does this again and again for `--time` seconds. The rate is the count of connections that got to `ReadyForQuery`, divided by the time. The result also has the p50, p99 and p99.9 of the time from the connect call to `ReadyForQuery`. The client sends one startup message and reads with one small buffer, so most of the time is the work of the server. The target of M2 is at least 10x the rate of PostgreSQL 19 with `trust` over a Unix socket. With `sslmode=require` or a password in `--conn`, each connection goes through the full client of `pg.rs`, with TLS and SCRAM-SHA-256, as the second target of section 6.18 asks.
//!
//! The rate also depends on the client when the client runs on the same machine, because the client uses the same cores. With `--pid` the result also has the server CPU for each connection, which does not depend on the client. It is the change of the CPU time of the process and the processes below it, divided by the connections. A backend of PostgreSQL that ends goes into the CPU time of the postmaster when the postmaster reaps it, so the test waits for one second after the last connection.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::json::Json;
use crate::ycsb::Histogram;
use crate::{idle, pg};

/// The startup message of protocol 3.0 for the user and the database of the configuration.
fn startup(c: &pg::Config) -> Vec<u8> {
    let mut body = 196608u32.to_be_bytes().to_vec();
    for (k, v) in [
        ("user", c.user.as_str()),
        ("database", c.dbname.as_str()),
        ("application_name", "rupg-bench"),
    ] {
        body.extend_from_slice(k.as_bytes());
        body.push(0);
        body.extend_from_slice(v.as_bytes());
        body.push(0);
    }
    body.push(0);
    let mut m = u32::try_from(body.len() + 4).unwrap_or(u32::MAX).to_be_bytes().to_vec();
    m.extend_from_slice(&body);
    m
}

/// The `M` field of an `ErrorResponse`.
fn error_text(body: &[u8]) -> String {
    for field in body.split(|&b| b == 0) {
        if let Some(text) = field.strip_prefix(b"M") {
            return String::from_utf8_lossy(text).into_owned();
        }
    }
    "an error with no message".to_owned()
}

/// What the messages in `buf` say: `Some(Ok(()))` at `ReadyForQuery`, `Some(Err)` at an error or a request for a password, and `None` when the server has more to send.
fn scan(buf: &[u8]) -> Option<Result<(), String>> {
    let mut at = 0;
    while let Some(head) = buf.get(at..at + 5) {
        let len = u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize;
        let body = buf.get(at + 5..at + 1 + len)?;
        match head[0] {
            b'Z' => return Some(Ok(())),
            b'E' => return Some(Err(error_text(body))),
            b'R' if body.get(..4) != Some(&[0, 0, 0, 0]) => {
                return Some(Err(
                    "the server asks for a password. The connection rate uses trust, so set trust for the user in pg_hba.conf".to_owned(),
                ));
            }
            _ => {}
        }
        at += 1 + len;
    }
    None
}

/// One connection: the startup message, the messages of the server until `ReadyForQuery`, and `Terminate`.
fn talk<S: Read + Write>(mut s: S, startup: &[u8], buf: &mut Vec<u8>) -> Result<(), String> {
    s.write_all(startup).map_err(|e| format!("send: {e}"))?;
    buf.clear();
    let mut chunk = [0u8; 4096];
    loop {
        let n = s.read(&mut chunk).map_err(|e| format!("receive: {e}"))?;
        if n == 0 {
            return Err("the server closed the connection at the start".to_owned());
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(done) = scan(buf) {
            done?;
            break;
        }
    }
    // A failure of Terminate does not matter, the socket closes anyway.
    let _ = s.write_all(&[b'X', 0, 0, 0, 4]);
    Ok(())
}

fn once(c: &pg::Config, startup: &[u8], buf: &mut Vec<u8>) -> Result<(), String> {
    if c.tls || c.password.is_some() {
        // The drop of the connection sends Terminate.
        pg::Conn::connect(c)?;
        Ok(())
    } else if c.host.starts_with('/') {
        let path = format!("{}/.s.PGSQL.{}", c.host, c.port);
        let s = UnixStream::connect(&path).map_err(|e| format!("{path}: {e}"))?;
        talk(s, startup, buf)
    } else {
        let s = TcpStream::connect((c.host.as_str(), c.port))
            .map_err(|e| format!("{}:{}: {e}", c.host, c.port))?;
        s.set_nodelay(true).map_err(|e| format!("set_nodelay: {e}"))?;
        talk(s, startup, buf)
    }
}

/// The numbers of one run.
#[derive(Clone, Debug)]
pub(crate) struct Rate {
    pub(crate) clients: u32,
    pub(crate) seconds: f64,
    pub(crate) connections: u64,
    /// The time from the connect call to `ReadyForQuery`, in nanoseconds.
    pub(crate) latency: Histogram,
    /// The CPU time of the server over the run, with `--pid`.
    pub(crate) server_cpu: Option<Duration>,
}

impl Rate {
    pub(crate) fn per_second(&self) -> f64 {
        if self.seconds > 0.0 { self.connections as f64 / self.seconds } else { 0.0 }
    }

    /// The server CPU for each connection, in microseconds.
    pub(crate) fn server_us(&self) -> Option<f64> {
        let cpu = self.server_cpu?;
        (self.connections > 0).then(|| cpu.as_secs_f64() * 1e6 / self.connections as f64)
    }

    pub(crate) fn to_json(&self) -> Json {
        let mut j = Json::obj()
            .with("clients", self.clients)
            .with("seconds", self.seconds)
            .with("connections", self.connections)
            .with("connections_per_second", (self.per_second() * 10.0).round() / 10.0)
            .with("latency", self.latency.to_json());
        if let Some(cpu) = self.server_cpu {
            j = j.with("server_cpu_seconds", cpu.as_secs_f64());
        }
        if let Some(us) = self.server_us() {
            j = j.with("server_cpu_us_per_connection", (us * 10.0).round() / 10.0);
        }
        j
    }

    pub(crate) fn text(&self) -> String {
        let us = |q: f64| self.latency.quantile(q).map_or(0.0, |n| n as f64 / 1000.0);
        format!(
            "connections     {} in {:.1} s with {} clients\nrate            {:.1} connections a second\nlatency         p50 {:.1} us, p99 {:.1} us, p99.9 {:.1} us\n",
            self.connections,
            self.seconds,
            self.clients,
            self.per_second(),
            us(0.50),
            us(0.99),
            us(0.999),
        ) + &self
            .server_us()
            .map(|us| format!("server cpu      {us:.1} us for each connection\n"))
            .unwrap_or_default()
    }
}

/// The clock ticks of a second in `/proc/<pid>/stat`. Linux gives these times in `USER_HZ`, which is 100 on each architecture that the harness runs on.
const TICKS_PER_SECOND: u64 = 100;

/// The fields `utime`, `stime`, `cutime` and `cstime` of the text of `/proc/<pid>/stat`, in ticks. The fields count from the last `)`, because the name of the program can hold spaces.
fn stat_ticks(stat: &str) -> Option<u64> {
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace().skip(11).take(4).map(|v| v.parse::<u64>().ok()).sum()
}

/// The CPU time of `root` and the processes below it, with the processes that ended and that a process of the tree reaped.
pub(crate) fn tree_cpu(root: u32) -> Result<Duration, String> {
    let mut ticks = 0;
    for pid in idle::tree(root) {
        // A process that ends before this read is not counted until its parent reaps it.
        let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) else {
            if pid == root {
                return Err(format!("the process {root} does not exist"));
            }
            continue;
        };
        ticks += stat_ticks(&stat).ok_or_else(|| format!("/proc/{pid}/stat is not valid"))?;
    }
    Ok(Duration::from_millis(ticks * 1000 / TICKS_PER_SECOND))
}

/// Runs `clients` threads for `time`. The first error stops every thread and is the result. With `server`, the result has the CPU time of that process and the processes below it.
pub(crate) fn run(
    c: &pg::Config,
    clients: u32,
    time: Duration,
    server: Option<u32>,
) -> Result<Rate, String> {
    let startup = startup(c);
    let cpu_before = server.map(tree_cpu).transpose()?;
    let stop = AtomicBool::new(false);
    let first_error: Mutex<Option<String>> = Mutex::new(None);
    let started = Instant::now();
    let deadline = started + time;
    let parts: Vec<(u64, Histogram)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..clients)
            .map(|_| {
                scope.spawn(|| {
                    let mut hist = Histogram::default();
                    let mut count = 0;
                    let mut buf = Vec::with_capacity(1024);
                    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
                        let t = Instant::now();
                        if let Err(e) = once(c, &startup, &mut buf) {
                            stop.store(true, Ordering::Relaxed);
                            first_error
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .get_or_insert(e);
                            break;
                        }
                        hist.record(u64::try_from(t.elapsed().as_nanos()).unwrap_or(u64::MAX));
                        count += 1;
                    }
                    (count, hist)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap_or_default()).collect()
    });
    let seconds = started.elapsed().as_secs_f64();
    if let Some(e) = first_error.into_inner().unwrap_or_else(std::sync::PoisonError::into_inner) {
        return Err(e);
    }
    let server_cpu = match (server, cpu_before) {
        (Some(pid), Some(before)) => {
            thread::sleep(Duration::from_secs(1));
            Some(tree_cpu(pid)?.saturating_sub(before))
        }
        _ => None,
    };
    let mut latency = Histogram::default();
    let mut connections = 0;
    for (n, h) in &parts {
        connections += n;
        latency.merge(h);
    }
    Ok(Rate { clients, seconds, connections, latency, server_cpu })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(tag: u8, body: &[u8]) -> Vec<u8> {
        let mut m = vec![tag];
        m.extend_from_slice(&u32::try_from(body.len() + 4).unwrap().to_be_bytes());
        m.extend_from_slice(body);
        m
    }

    #[test]
    fn scan_waits_for_ready() {
        let mut buf = msg(b'R', &[0, 0, 0, 0]);
        buf.extend(msg(b'S', b"server_version\x0019devel\x00"));
        assert_eq!(scan(&buf), None);
        let ready = msg(b'Z', b"I");
        buf.extend_from_slice(&ready[..3]);
        assert_eq!(scan(&buf), None);
        buf.extend_from_slice(&ready[3..]);
        assert_eq!(scan(&buf), Some(Ok(())));
    }

    #[test]
    fn scan_stops_at_an_error_or_a_password() {
        let e = msg(b'E', b"SFATAL\x00C3D000\x00Mdatabase \"x\" does not exist\x00\x00");
        assert_eq!(scan(&e), Some(Err("database \"x\" does not exist".to_owned())));
        let md5 = msg(b'R', &[0, 0, 0, 5, 1, 2, 3, 4]);
        assert!(scan(&md5).unwrap().unwrap_err().contains("trust"));
    }

    #[test]
    fn the_startup_message_has_its_length() {
        let c = pg::Config::parse("user=bench dbname=bench").unwrap();
        let m = startup(&c);
        assert_eq!(u32::from_be_bytes([m[0], m[1], m[2], m[3]]) as usize, m.len());
        assert_eq!(&m[4..8], &196608u32.to_be_bytes());
        assert!(m.ends_with(b"rupg-bench\x00\x00"));
    }

    #[test]
    fn rate() {
        let mut latency = Histogram::default();
        latency.record(500_000);
        let mut r = Rate { clients: 4, seconds: 2.0, connections: 3000, latency, server_cpu: None };
        assert_eq!(r.per_second(), 1500.0);
        assert!(r.text().contains("1500.0 connections a second"));
        assert!(!r.text().contains("server cpu"));
        r.server_cpu = Some(Duration::from_millis(1500));
        assert_eq!(r.server_us(), Some(500.0));
        assert!(r.text().contains("server cpu      500.0 us for each connection"));
        assert!(r.to_json().pretty().contains("\"server_cpu_us_per_connection\": 500"));
    }

    #[test]
    fn the_ticks_of_a_stat_line() {
        let stat = "42 (a (b) c) S 1 42 42 0 -1 4194560 100 0 0 0 7 3 20 10 20 0 1 0 5 0 0";
        assert_eq!(stat_ticks(stat), Some(40));
        assert_eq!(stat_ticks("42 (x) S 1"), Some(0));
        assert_eq!(stat_ticks("no parenthesis"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn the_cpu_of_this_process_grows() {
        let me = std::process::id();
        let before = tree_cpu(me).unwrap();
        let t = Instant::now();
        let mut x = 0u64;
        while t.elapsed() < Duration::from_millis(100) {
            x = std::hint::black_box(x.wrapping_add(1));
        }
        assert!(tree_cpu(me).unwrap() >= before);
        assert!(tree_cpu(u32::MAX).is_err());
    }
}
