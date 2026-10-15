//! The connection rate of spec/06 section 6.18 of tamnd/rupg.
//!
//! Each of `--clients` threads opens a connection, reads the messages of the server until `ReadyForQuery`, sends `Terminate` and closes the socket. It does this again and again for `--time` seconds. The rate is the count of connections that got to `ReadyForQuery`, divided by the time. The result also has the p50, p99 and p99.9 of the time from the connect call to `ReadyForQuery`. The client sends one startup message and reads with one small buffer, so most of the time is the work of the server. The target of M2 is at least 10x the rate of PostgreSQL 19 with `trust` over a Unix socket. The client has no TLS and no SCRAM, so the target with SCRAM over TLS is not here yet.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::json::Json;
use crate::pg;
use crate::ycsb::Histogram;

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
    if c.host.starts_with('/') {
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
}

impl Rate {
    pub(crate) fn per_second(&self) -> f64 {
        if self.seconds > 0.0 { self.connections as f64 / self.seconds } else { 0.0 }
    }

    pub(crate) fn to_json(&self) -> Json {
        Json::obj()
            .with("clients", self.clients)
            .with("seconds", self.seconds)
            .with("connections", self.connections)
            .with("connections_per_second", (self.per_second() * 10.0).round() / 10.0)
            .with("latency", self.latency.to_json())
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
        )
    }
}

/// Runs `clients` threads for `time`. The first error stops every thread and is the result.
pub(crate) fn run(c: &pg::Config, clients: u32, time: Duration) -> Result<Rate, String> {
    let startup = startup(c);
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
    let mut latency = Histogram::default();
    let mut connections = 0;
    for (n, h) in &parts {
        connections += n;
        latency.merge(h);
    }
    Ok(Rate { clients, seconds, connections, latency })
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
        let r = Rate { clients: 4, seconds: 2.0, connections: 3000, latency };
        assert_eq!(r.per_second(), 1500.0);
        assert!(r.text().contains("1500.0 connections a second"));
    }
}
