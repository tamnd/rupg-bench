//! The cgroup v2 runner of spec/20 section 20.4.
//!
//! Each system runs in its own cgroup. The runner creates the cgroup, starts the system in it or attaches to the cgroup of a systemd unit, and reads these numbers over an interval:
//!
//! - peak memory: `memory.peak`. Since Linux 6.12 a write to the file resets the peak for later reads through the same descriptor. On an older kernel the write fails, and the peak counts from the creation of the cgroup, so a driver uses a new cgroup for each suite.
//! - page cache part: `file` in `memory.stat`, sampled every 100 ms, the maximum.
//! - process memory: `Pss` in `/proc/<pid>/smaps_rollup`, summed over the processes of the cgroup, sampled every 100 ms, the maximum.
//! - CPU time: `usage_usec` in `cpu.stat`, end minus start.
//! - device bytes: `rbytes` and `wbytes` in `io.stat`, summed over the devices, end minus start.
//! - disk at rest: `du --apparent-size --bytes` over a path.
//!
//! The idle base is the same interval over 10 s with the system idle. It is published next to the suite numbers and never subtracted.

use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::json::Json;

pub(crate) const CGROUP_ROOT: &str = "/sys/fs/cgroup";
/// The parent of the cgroups that the harness creates. It holds no process, as cgroup v2 requires.
pub(crate) const PARENT: &str = "rupg-bench";
pub(crate) const SAMPLE_EVERY: Duration = Duration::from_millis(100);
pub(crate) const IDLE_BASE: Duration = Duration::from_secs(10);
const CONTROLLERS: [&str; 4] = ["cpu", "memory", "io", "pids"];

/// One cgroup.
#[derive(Debug)]
pub(crate) struct Cgroup {
    pub(crate) path: PathBuf,
    /// True when the harness created the cgroup and removes it.
    owned: bool,
}

impl Cgroup {
    /// Creates `/sys/fs/cgroup/rupg-bench/<name>`. An empty cgroup with the same name is removed first, so the counters start at zero. `cpus` is a `cpuset.cpus` list such as `0-15`.
    pub(crate) fn create(name: &str, cpus: Option<&str>) -> Result<Cgroup, String> {
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        {
            return Err(format!("the cgroup name {name:?} must be letters, digits, - _ or ."));
        }
        let parent = Path::new(CGROUP_ROOT).join(PARENT);
        fs::create_dir_all(&parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        let mut controllers = CONTROLLERS.to_vec();
        if cpus.is_some() {
            controllers.push("cpuset");
        }
        let enabled = read(&parent.join("cgroup.subtree_control"))?;
        for c in controllers {
            if !enabled.split_whitespace().any(|e| e == c) {
                write(&parent.join("cgroup.subtree_control"), &format!("+{c}"))?;
            }
        }
        let path = parent.join(name);
        if path.exists() {
            let old = Cgroup { path: path.clone(), owned: true };
            if !old.pids().is_empty() {
                return Err(format!("{} exists and has processes", path.display()));
            }
            fs::remove_dir(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        fs::create_dir(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let cg = Cgroup { path, owned: true };
        if let Some(cpus) = cpus {
            write(&cg.path.join("cpuset.cpus"), cpus)?;
        }
        Ok(cg)
    }

    /// Uses a cgroup that exists, for example `/sys/fs/cgroup/system.slice/postgresql@19-main.service`. The harness reads it and does not remove it. systemd makes a new cgroup each time it starts the unit.
    pub(crate) fn attach(path: &Path) -> Result<Cgroup, String> {
        if !path.join("memory.peak").exists() {
            return Err(format!(
                "{} is not a cgroup v2 directory with memory.peak",
                path.display()
            ));
        }
        Ok(Cgroup { path: path.to_owned(), owned: false })
    }

    /// True when the harness created the cgroup.
    pub(crate) fn owned(&self) -> bool {
        self.owned
    }

    /// The cgroup of a systemd unit.
    pub(crate) fn of_unit(unit: &str) -> Result<Cgroup, String> {
        let out = Command::new("systemctl")
            .args(["show", "--property=ControlGroup", "--value", unit])
            .output()
            .map_err(|e| format!("systemctl: {e}"))?;
        let rel = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        if rel.is_empty() {
            return Err(format!("the unit {unit} has no cgroup. Is it running?"));
        }
        Cgroup::attach(&Path::new(CGROUP_ROOT).join(rel.trim_start_matches('/')))
    }

    /// A command that moves itself into this cgroup and then runs `argv`. The shell writes its own pid into `cgroup.procs` and then replaces itself with the program, so every child of the program is in the cgroup from its start.
    pub(crate) fn command(&self, argv: &[String]) -> Result<Command, String> {
        if argv.is_empty() {
            return Err("no command to run".to_owned());
        }
        let mut cmd = Command::new("/bin/sh");
        cmd.arg("-c")
            .arg("echo $$ > \"$0\" && exec \"$@\"")
            .arg(self.path.join("cgroup.procs"))
            .args(argv);
        Ok(cmd)
    }

    /// The pids of the cgroup and of the cgroups below it.
    pub(crate) fn pids(&self) -> Vec<u32> {
        let mut out = Vec::new();
        let mut stack = vec![self.path.clone()];
        while let Some(dir) = stack.pop() {
            if let Ok(text) = fs::read_to_string(dir.join("cgroup.procs")) {
                out.extend(text.lines().filter_map(|l| l.trim().parse::<u32>().ok()));
            }
            if let Ok(entries) = fs::read_dir(&dir) {
                stack.extend(entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()));
            }
        }
        out
    }

    /// The counters that are read at the start and the end of an interval.
    pub(crate) fn counters(&self) -> Result<Counters, String> {
        let (usage_usec, user_usec, system_usec) =
            parse_cpu_stat(&read(&self.path.join("cpu.stat"))?)?;
        // io.stat is empty until the cgroup does I/O. It is missing if the io controller is off for the cgroup, for example in a systemd unit without IOAccounting=yes. Then the bytes are not known, and they are not 0.
        let (rbytes, wbytes) = match fs::read_to_string(self.path.join("io.stat")) {
            Ok(text) => {
                let (r, w) = parse_io_stat(&text);
                (Some(r), Some(w))
            }
            Err(_) => (None, None),
        };
        let memory_current = read_u64(&self.path.join("memory.current"))?;
        Ok(Counters { usage_usec, user_usec, system_usec, rbytes, wbytes, memory_current })
    }

    /// One sample: the `file` bytes of `memory.stat`, the sum of `Pss` and `memory.current`.
    pub(crate) fn sample(&self) -> Sample {
        let file = fs::read_to_string(self.path.join("memory.stat"))
            .ok()
            .and_then(|t| stat_field(&t, "file"))
            .unwrap_or(0);
        let pss = self.pids().into_iter().filter_map(pss_of).sum();
        let current = read_u64(&self.path.join("memory.current")).unwrap_or(0);
        Sample { file, pss, current }
    }

    /// Kills every process of the cgroup and waits up to 10 s for them to exit.
    pub(crate) fn kill_all(&self) -> Result<(), String> {
        if self.pids().is_empty() {
            return Ok(());
        }
        write(&self.path.join("cgroup.kill"), "1")?;
        let deadline = Instant::now() + Duration::from_secs(10);
        while !self.pids().is_empty() {
            if Instant::now() > deadline {
                return Err(format!(
                    "{} still has processes after cgroup.kill",
                    self.path.display()
                ));
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }

    /// Removes the cgroup if the harness created it. The processes must have exited.
    pub(crate) fn remove(self) -> Result<(), String> {
        if !self.owned {
            return Ok(());
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match fs::remove_dir(&self.path) {
                Ok(()) => return Ok(()),
                // A process that has just exited can keep the cgroup busy for a moment.
                Err(_) if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
                Err(e) => return Err(format!("{}: {e}", self.path.display())),
            }
        }
    }
}

/// The counters at one moment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counters {
    pub(crate) usage_usec: u64,
    pub(crate) user_usec: u64,
    pub(crate) system_usec: u64,
    pub(crate) rbytes: Option<u64>,
    pub(crate) wbytes: Option<u64>,
    pub(crate) memory_current: u64,
}

/// One sample of the 100 ms sampler.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Sample {
    pub(crate) file: u64,
    pub(crate) pss: u64,
    pub(crate) current: u64,
}

/// What `memory.peak` counts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PeakScope {
    /// The write to `memory.peak` at the start of the interval worked (Linux 6.12 and later).
    Interval,
    /// The kernel cannot reset the peak, so it counts from the creation of the cgroup.
    Cgroup,
}

impl PeakScope {
    pub(crate) fn name(self) -> &'static str {
        match self {
            PeakScope::Interval => "reset at the start of the interval",
            PeakScope::Cgroup => "since the cgroup was created",
        }
    }
}

/// `memory.peak`, opened once so that a reset and the later read use the same descriptor.
#[derive(Debug)]
struct Peak {
    file: File,
    scope: PeakScope,
}

impl Peak {
    fn open(cg: &Cgroup) -> Result<Peak, String> {
        let path = cg.path.join("memory.peak");
        if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(&path)
            && file.write_all(b"reset\n").is_ok()
        {
            return Ok(Peak { file, scope: PeakScope::Interval });
        }
        let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Peak { file, scope: PeakScope::Cgroup })
    }

    fn read(&mut self) -> Result<u64, String> {
        let mut text = String::new();
        self.file.seek(SeekFrom::Start(0)).map_err(|e| format!("memory.peak: {e}"))?;
        self.file.read_to_string(&mut text).map_err(|e| format!("memory.peak: {e}"))?;
        text.trim().parse().map_err(|_| format!("memory.peak: {text:?} is not a number"))
    }
}

/// The 100 ms sampler. It keeps the maximum of each sampled number.
#[derive(Debug)]
struct Sampler {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<(Sample, u64)>,
}

impl Sampler {
    fn start(path: PathBuf) -> Sampler {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            let cg = Cgroup { path, owned: false };
            let mut max = Sample::default();
            let mut count = 0;
            // The ticks are at fixed times, so a slow read of smaps_rollup does not stretch the period.
            let mut next = Instant::now();
            loop {
                let s = cg.sample();
                max.file = max.file.max(s.file);
                max.pss = max.pss.max(s.pss);
                max.current = max.current.max(s.current);
                count += 1;
                next += SAMPLE_EVERY;
                let now = Instant::now();
                if next < now {
                    next = now;
                }
                // Sleep in short steps, so that the end of an interval does not wait 100 ms.
                while Instant::now() < next {
                    if flag.load(Ordering::Relaxed) {
                        return (max, count);
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                if flag.load(Ordering::Relaxed) {
                    return (max, count);
                }
            }
        });
        Sampler { stop, handle }
    }

    fn finish(self) -> (Sample, u64) {
        self.stop.store(true, Ordering::Relaxed);
        self.handle.join().unwrap_or_default()
    }
}

/// An interval that is being measured.
#[derive(Debug)]
pub(crate) struct Interval {
    path: PathBuf,
    start: Counters,
    started: Instant,
    peak: Peak,
    sampler: Sampler,
}

impl Interval {
    /// Resets the peak if the kernel can, reads the counters and starts the sampler.
    pub(crate) fn start(cg: &Cgroup) -> Result<Interval, String> {
        let peak = Peak::open(cg)?;
        let start = cg.counters()?;
        Ok(Interval {
            path: cg.path.clone(),
            start,
            started: Instant::now(),
            peak,
            sampler: Sampler::start(cg.path.clone()),
        })
    }

    /// Stops the sampler and reads the counters. The processes of the cgroup may have exited.
    pub(crate) fn finish(mut self) -> Result<Usage, String> {
        let wall = self.started.elapsed();
        let (max, samples) = self.sampler.finish();
        let cg = Cgroup { path: self.path.clone(), owned: false };
        let end = cg.counters()?;
        let memory_peak = self.peak.read()?;
        Ok(Usage {
            wall,
            cpu_usec: end.usage_usec.saturating_sub(self.start.usage_usec),
            user_usec: end.user_usec.saturating_sub(self.start.user_usec),
            system_usec: end.system_usec.saturating_sub(self.start.system_usec),
            rbytes: end.rbytes.zip(self.start.rbytes).map(|(e, s)| e.saturating_sub(s)),
            wbytes: end.wbytes.zip(self.start.wbytes).map(|(e, s)| e.saturating_sub(s)),
            memory_peak,
            peak_scope: self.peak.scope,
            file_max: max.file,
            pss_max: max.pss,
            current_max: max.current.max(end.memory_current),
            samples,
        })
    }
}

/// The numbers of one interval.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Usage {
    pub(crate) wall: Duration,
    pub(crate) cpu_usec: u64,
    pub(crate) user_usec: u64,
    pub(crate) system_usec: u64,
    /// None when the io controller is off for the cgroup.
    pub(crate) rbytes: Option<u64>,
    pub(crate) wbytes: Option<u64>,
    pub(crate) memory_peak: u64,
    pub(crate) peak_scope: PeakScope,
    pub(crate) file_max: u64,
    pub(crate) pss_max: u64,
    pub(crate) current_max: u64,
    pub(crate) samples: u64,
}

impl Usage {
    pub(crate) fn to_json(&self) -> Json {
        Json::obj()
            .with("wall_s", self.wall.as_secs_f64())
            .with("cpu_usec", self.cpu_usec)
            .with("user_usec", self.user_usec)
            .with("system_usec", self.system_usec)
            .with("rbytes", self.rbytes)
            .with("wbytes", self.wbytes)
            .with("memory_peak_bytes", self.memory_peak)
            .with("memory_peak_scope", self.peak_scope.name())
            .with("file_max_bytes", self.file_max)
            .with("pss_max_bytes", self.pss_max)
            .with("memory_current_max_bytes", self.current_max)
            .with("samples", self.samples)
    }

    /// Lines for a terminal, one number on each line.
    pub(crate) fn text(&self) -> String {
        format!(
            "wall            {:.3} s\ncpu             {:.3} s (user {:.3} s, system {:.3} s)\nmemory.peak     {} ({})\nfile max        {}\npss max         {}\ncurrent max     {}\nio read         {}\nio write        {}\nsamples         {}\n",
            self.wall.as_secs_f64(),
            self.cpu_usec as f64 / 1e6,
            self.user_usec as f64 / 1e6,
            self.system_usec as f64 / 1e6,
            mib(self.memory_peak),
            self.peak_scope.name(),
            mib(self.file_max),
            mib(self.pss_max),
            mib(self.current_max),
            io_bytes(self.rbytes),
            io_bytes(self.wbytes),
            self.samples
        )
    }
}

fn io_bytes(bytes: Option<u64>) -> String {
    bytes.map_or_else(|| "not measured: the io controller is off for this cgroup".to_owned(), mib)
}

/// Bytes as MiB with two decimals, and the exact byte count.
pub(crate) fn mib(bytes: u64) -> String {
    format!("{:.2} MiB ({bytes} bytes)", bytes as f64 / 1048576.0)
}

/// Runs `argv` and measures `cg` while it runs. In a cgroup that the harness created, the command is the system and runs in the cgroup, and every process left in the cgroup is killed at the end. In a cgroup that exists, the command is the client and runs outside it.
pub(crate) fn run(cg: &Cgroup, argv: &[String]) -> Result<(ExitStatus, Usage), String> {
    let mut cmd = if cg.owned() {
        cg.command(argv)?
    } else {
        let (program, args) = argv.split_first().ok_or("no command to run")?;
        let mut c = Command::new(program);
        c.args(args);
        c
    };
    let interval = Interval::start(cg)?;
    let status = cmd.status().map_err(|e| format!("{}: {e}", argv[0]));
    let usage = interval.finish()?;
    let status = status?;
    if cg.owned() {
        cg.kill_all()?;
    }
    Ok((status, usage))
}

/// Measures the idle base: the cgroup with nothing to do for `duration`.
pub(crate) fn idle_base(cg: &Cgroup, duration: Duration) -> Result<Usage, String> {
    let interval = Interval::start(cg)?;
    thread::sleep(duration);
    interval.finish()
}

/// The bytes at rest: `du --apparent-size --bytes -s <path>`.
pub(crate) fn du_apparent(path: &Path) -> Result<u64, String> {
    let out = Command::new("du")
        .args(["--apparent-size", "--bytes", "-s"])
        .arg(path)
        .output()
        .map_err(|e| format!("du: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "du {}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace()
        .next()
        .and_then(|n| n.parse().ok())
        .ok_or(format!("du printed {text:?}"))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn read_u64(path: &Path) -> Result<u64, String> {
    let text = read(path)?;
    text.trim().parse().map_err(|_| format!("{}: {text:?} is not a number", path.display()))
}

fn write(path: &Path, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|e| format!("write {value:?} to {}: {e}", path.display()))
}

/// The value of `name` in a file of `name value` lines, such as `memory.stat` and `cpu.stat`.
fn stat_field(text: &str, name: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let (key, value) = line.split_once(' ')?;
        if key == name { value.trim().parse().ok() } else { None }
    })
}

fn parse_cpu_stat(text: &str) -> Result<(u64, u64, u64), String> {
    let get = |name| stat_field(text, name).ok_or(format!("cpu.stat has no {name}"));
    Ok((get("usage_usec")?, get("user_usec")?, get("system_usec")?))
}

/// The sums of `rbytes` and `wbytes` over the devices of `io.stat`.
fn parse_io_stat(text: &str) -> (u64, u64) {
    let (mut r, mut w) = (0, 0);
    for field in text.split_whitespace() {
        if let Some(v) = field.strip_prefix("rbytes=") {
            r += v.parse::<u64>().unwrap_or(0);
        } else if let Some(v) = field.strip_prefix("wbytes=") {
            w += v.parse::<u64>().unwrap_or(0);
        }
    }
    (r, w)
}

/// `Pss` of `smaps_rollup` in bytes.
fn parse_pss(text: &str) -> Option<u64> {
    let line = text.lines().find(|l| l.starts_with("Pss:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb * 1024)
}

fn pss_of(pid: u32) -> Option<u64> {
    // A process that exits between the read of cgroup.procs and this read counts as zero.
    parse_pss(&fs::read_to_string(format!("/proc/{pid}/smaps_rollup")).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_stat() {
        let text = "usage_usec 1500\nuser_usec 1000\nsystem_usec 500\nnr_periods 0\n";
        assert_eq!(parse_cpu_stat(text).unwrap(), (1500, 1000, 500));
        assert!(parse_cpu_stat("user_usec 1\n").unwrap_err().contains("usage_usec"));
    }

    #[test]
    fn io_stat_sums_the_devices() {
        let text = "8:0 rbytes=4096 wbytes=8192 rios=1 wios=2 dbytes=0 dios=0\n259:0 rbytes=10 wbytes=20 rios=1 wios=1 dbytes=0 dios=0\n";
        assert_eq!(parse_io_stat(text), (4106, 8212));
        assert_eq!(parse_io_stat(""), (0, 0));
    }

    #[test]
    fn memory_stat_file_is_not_file_mapped() {
        let text = "anon 100\nfile 4096\nkernel 5\nfile_mapped 77\nfile_dirty 3\n";
        assert_eq!(stat_field(text, "file"), Some(4096));
        assert_eq!(stat_field(text, "file_mapped"), Some(77));
        assert_eq!(stat_field(text, "shmem"), None);
    }

    #[test]
    fn pss_is_in_bytes() {
        let text = "55a4c000-7ffd1000 ---p 00000000 00:00 0  [rollup]\nRss:  2048 kB\nPss:  1500 kB\nPss_Anon:  1000 kB\n";
        assert_eq!(parse_pss(text), Some(1500 * 1024));
    }

    #[test]
    fn usage_json_has_every_number_of_the_spec() {
        let u = Usage {
            wall: Duration::from_millis(1500),
            cpu_usec: 1,
            user_usec: 1,
            system_usec: 0,
            rbytes: Some(2),
            wbytes: None,
            memory_peak: 4,
            peak_scope: PeakScope::Cgroup,
            file_max: 5,
            pss_max: 6,
            current_max: 7,
            samples: 15,
        };
        let text = u.to_json().pretty();
        for key in
            ["cpu_usec", "rbytes", "wbytes", "memory_peak_bytes", "file_max_bytes", "pss_max_bytes"]
        {
            assert!(text.contains(&format!("\"{key}\"")), "{key}");
        }
        assert!(text.contains("since the cgroup was created"));
        assert!(text.contains("\"wbytes\": null"));
        assert!(u.text().contains("io write        not measured"));
    }

    #[test]
    fn a_bad_name_is_refused_before_any_write() {
        assert!(Cgroup::create("../x", None).is_err());
        assert!(Cgroup::create("", None).is_err());
    }
}
