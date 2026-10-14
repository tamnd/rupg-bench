//! The idle session test of spec/06 section 6.4 and spec/21 section 21.13 of tamnd/rupg.
//!
//! The test opens `--sessions` connections. Each connection completes the startup and then waits in the state `Idle`. The test reads the memory of the server before the first connection and after the last one, and divides the change by the number of sessions. The memory is the sum of `Pss` in `/proc/<pid>/smaps_rollup` over the processes of the server: the processes of a cgroup, or a process and the processes below it. For a cgroup the test also reads `memory.current`, which also counts the kernel memory of the sockets, and publishes that number next to the first one. The limit of spec/06 section 6.4 is 64 KiB for each session.
//!
//! The variant `plain` only connects. The variant `discard` sends `DISCARD ALL` on each connection. The variant `prepare` prepares a statement and drops it on each connection, as spec/21 section 21.13 asks. The client of the harness has no TLS and no SCRAM, so the variants with TLS of spec/06 section 6.4.3 are not here yet.

use std::collections::BTreeMap;
use std::fs;
use std::thread;
use std::time::Duration;

use crate::cgroup::{self, Cgroup};
use crate::json::Json;
use crate::pg;

/// The limit of spec/06 section 6.4.
pub(crate) const LIMIT: u64 = 64 << 10;

/// The samples of memory at each end of the test. The median counts.
const SAMPLES: usize = 5;
const SAMPLE_GAP: Duration = Duration::from_millis(200);

/// The processes of the server.
#[derive(Debug)]
pub(crate) enum Server {
    Cgroup(Cgroup),
    /// A process and the processes below it, for example the postmaster of PostgreSQL and its backends.
    Pid(u32),
}

impl Server {
    /// The name of the server in the result.
    pub(crate) fn describe(&self) -> String {
        match self {
            Server::Cgroup(cg) => format!("cgroup {}", cg.path.display()),
            Server::Pid(pid) => format!("pid {pid} and the processes below it"),
        }
    }

    fn pids(&self) -> Vec<u32> {
        match self {
            Server::Cgroup(cg) => cg.pids(),
            Server::Pid(pid) => tree(*pid),
        }
    }

    /// The sum of `Pss` and, for a cgroup, `memory.current`.
    fn sample(&self) -> (u64, Option<u64>) {
        let pss = self.pids().into_iter().filter_map(cgroup::pss_of).sum();
        let current = match self {
            Server::Cgroup(cg) => cg.counters().ok().map(|c| c.memory_current),
            Server::Pid(_) => None,
        };
        (pss, current)
    }

    /// The median of [`SAMPLES`] samples.
    fn settled(&self) -> (u64, Option<u64>) {
        let mut pss = Vec::with_capacity(SAMPLES);
        let mut current = Vec::with_capacity(SAMPLES);
        for i in 0..SAMPLES {
            if i > 0 {
                thread::sleep(SAMPLE_GAP);
            }
            let (p, c) = self.sample();
            pss.push(p);
            current.extend(c);
        }
        (median(pss), (current.len() == SAMPLES).then(|| median(current)))
    }
}

fn median(mut v: Vec<u64>) -> u64 {
    v.sort_unstable();
    v[v.len() / 2]
}

/// The parent pid in the text of `/proc/<pid>/stat`. The name of the program is in parentheses and can hold spaces, so the fields count from the last `)`.
fn parent_of(stat: &str) -> Option<u32> {
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace().nth(1)?.parse().ok()
}

/// The process and every process below it.
fn tree(root: u32) -> Vec<u32> {
    let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for e in entries.flatten() {
            let Some(pid) = e.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else {
                continue;
            };
            // A process that exits before this read is not in the tree.
            if let Some(parent) =
                fs::read_to_string(e.path().join("stat")).ok().as_deref().and_then(parent_of)
            {
                children.entry(parent).or_default().push(pid);
            }
        }
    }
    let mut out = vec![root];
    let mut i = 0;
    while i < out.len() {
        if let Some(c) = children.get(&out[i]) {
            out.extend(c);
        }
        i += 1;
    }
    out
}

/// What each session does after the startup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Variant {
    Plain,
    Discard,
    Prepare,
}

impl Variant {
    pub(crate) fn parse(s: &str) -> Result<Variant, String> {
        match s {
            "plain" => Ok(Variant::Plain),
            "discard" => Ok(Variant::Discard),
            "prepare" => Ok(Variant::Prepare),
            _ => Err(format!("usage: the variant {s:?} is not plain, discard or prepare")),
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Variant::Plain => "plain",
            Variant::Discard => "discard",
            Variant::Prepare => "prepare",
        }
    }

    fn apply(self, conn: &mut pg::Conn) -> Result<(), pg::PgError> {
        match self {
            Variant::Plain => Ok(()),
            Variant::Discard => conn.simple("DISCARD ALL").map(drop),
            Variant::Prepare => {
                conn.prepare("rupg_bench_idle", "SELECT 1")?;
                conn.simple("DEALLOCATE rupg_bench_idle").map(drop)
            }
        }
    }
}

/// The numbers of one run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Measure {
    /// The `server_version` that the first session reported.
    pub(crate) server_version: String,
    pub(crate) sessions: u64,
    pub(crate) pss_before: u64,
    pub(crate) pss_after: u64,
    pub(crate) current_before: Option<u64>,
    pub(crate) current_after: Option<u64>,
}

impl Measure {
    /// The change of the sum of `Pss` for each session. It is 0 when the sum went down.
    pub(crate) fn pss_per_session(&self) -> u64 {
        self.pss_after.saturating_sub(self.pss_before) / self.sessions.max(1)
    }

    /// The change of `memory.current` for each session.
    pub(crate) fn current_per_session(&self) -> Option<u64> {
        Some(self.current_after?.saturating_sub(self.current_before?) / self.sessions.max(1))
    }

    pub(crate) fn passes(&self) -> bool {
        self.pss_per_session() <= LIMIT
    }

    pub(crate) fn to_json(&self) -> Json {
        Json::obj()
            .with("server_version", self.server_version.as_str())
            .with("sessions", self.sessions)
            .with("pss_before_bytes", self.pss_before)
            .with("pss_after_bytes", self.pss_after)
            .with("pss_per_session_bytes", self.pss_per_session())
            .with("memory_current_before_bytes", self.current_before)
            .with("memory_current_after_bytes", self.current_after)
            .with("memory_current_per_session_bytes", self.current_per_session())
            .with("limit_bytes", LIMIT)
            .with("passes", self.passes())
    }

    pub(crate) fn text(&self) -> String {
        let mut t = format!(
            "server version  {}\nsessions        {}\npss             {} before, {} after\npss per session {} bytes ({:.1} KiB), limit {} KiB: {}\n",
            self.server_version,
            self.sessions,
            cgroup::mib(self.pss_before),
            cgroup::mib(self.pss_after),
            self.pss_per_session(),
            self.pss_per_session() as f64 / 1024.0,
            LIMIT >> 10,
            if self.passes() { "pass" } else { "fail" },
        );
        if let Some(per) = self.current_per_session() {
            t.push_str(&format!(
                "memory.current  {per} bytes for each session, with the kernel memory of the sockets\n"
            ));
        }
        t
    }
}

/// Opens `sessions` connections, applies the variant on each, waits `settle` and measures. The connections close at the end.
pub(crate) fn run(
    server: &Server,
    conn: &pg::Config,
    sessions: u64,
    variant: Variant,
    settle: Duration,
) -> Result<Measure, String> {
    thread::sleep(settle);
    let (pss_before, current_before) = server.settled();
    let mut open = Vec::new();
    for i in 0..sessions {
        let mut c = pg::Conn::connect(conn).map_err(|e| {
            format!(
                "session {} of {sessions}: {e}. Each session takes two file descriptors in the harness, so raise ulimit -n, and max_connections of the server must allow the sessions",
                i + 1
            )
        })?;
        variant.apply(&mut c).map_err(|e| format!("session {}: {e}", i + 1))?;
        open.push(c);
    }
    thread::sleep(settle);
    let (pss_after, current_after) = server.settled();
    let server_version = open.first().map(|c| c.server_version.clone()).unwrap_or_default();
    drop(open);
    Ok(Measure { server_version, sessions, pss_before, pss_after, current_before, current_after })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_of_skips_the_name() {
        assert_eq!(parent_of("42 (postgres: io worker 0) S 7 42 42 0 -1"), Some(7));
        assert_eq!(parent_of("5 (a) b) R 1 5 5"), Some(1));
        assert_eq!(parent_of("5 (a"), None);
    }

    #[test]
    fn the_tree_has_this_process() {
        let me = std::process::id();
        assert!(tree(me).contains(&me));
    }

    #[test]
    fn per_session() {
        let m = Measure {
            server_version: "19devel".to_owned(),
            sessions: 1000,
            pss_before: 10 << 20,
            pss_after: (10 << 20) + 1000 * 40_000,
            current_before: Some(100),
            current_after: Some(100 + 1000 * 70_000),
        };
        assert_eq!(m.pss_per_session(), 40_000);
        assert_eq!(m.current_per_session(), Some(70_000));
        assert!(m.passes());
        let over = Measure { pss_after: (10 << 20) + 1000 * 70_000, ..m.clone() };
        assert!(!over.passes());
        let down = Measure { pss_after: 0, current_before: None, ..m };
        assert_eq!(down.pss_per_session(), 0);
        assert_eq!(down.current_per_session(), None);
    }

    #[test]
    fn variants() {
        for v in [Variant::Plain, Variant::Discard, Variant::Prepare] {
            assert_eq!(Variant::parse(v.name()), Ok(v));
        }
        assert!(Variant::parse("tls").is_err());
    }
}
