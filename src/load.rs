//! The load cases of spec/20 section 20.3.1.
//!
//! Case A loads from a source file that is not in the page cache. The harness drops the caches, checks with `fincore` that no page of the source is resident, and times `cat <source> > /dev/null`. That time is the reference `R`. Then it drops the caches again, checks again, and times the load `L`. The result records `L`, `R` and `L / R`.
//!
//! Case B loads from a source file that is in the page cache. The harness reads the file twice with `cat`, and `fincore` must report at least 99 percent of the pages resident. If fewer pages are resident, the run is case A and the harness does the steps of case A.

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::json::Json;

/// The share of resident pages that case B needs.
pub(crate) const CASE_B_RESIDENT: f64 = 0.99;

/// The case of a load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Case {
    A,
    B,
}

impl Case {
    pub(crate) fn parse(s: &str) -> Result<Case, String> {
        match s {
            "a" | "A" => Ok(Case::A),
            "b" | "B" => Ok(Case::B),
            _ => Err(format!("usage: the case {s:?} is not a or b")),
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Case::A => "A",
            Case::B => "B",
        }
    }
}

/// What `fincore` reports for one file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Fincore {
    /// Resident pages.
    pub(crate) pages: u64,
    /// Resident bytes.
    pub(crate) res: u64,
    /// The size of the file in bytes.
    pub(crate) size: u64,
    /// The line that `fincore` printed, for the result.
    pub(crate) raw: String,
}

impl Fincore {
    /// The share of the pages of the file that are resident. An empty file counts as fully resident.
    pub(crate) fn resident(&self, page_size: u64) -> f64 {
        let total = self.size.div_ceil(page_size);
        if total == 0 { 1.0 } else { self.pages as f64 / total as f64 }
    }

    fn to_json(&self, page_size: u64) -> Json {
        Json::obj()
            .with("resident_pages", self.pages)
            .with("resident_bytes", self.res)
            .with("size_bytes", self.size)
            .with("resident_share", self.resident(page_size))
            .with("fincore", self.raw.as_str())
    }
}

/// Parses `fincore --bytes --noheadings --raw --output PAGES,RES,SIZE,FILE`.
fn parse_fincore(text: &str) -> Result<Fincore, String> {
    let line = text.lines().next().unwrap_or("").trim();
    let mut words = line.split_whitespace();
    let mut num = |what: &str| -> Result<u64, String> {
        words
            .next()
            .and_then(|w| w.parse().ok())
            .ok_or(format!("fincore printed {line:?}, with no number for {what}"))
    };
    let pages = num("PAGES")?;
    let res = num("RES")?;
    let size = num("SIZE")?;
    Ok(Fincore { pages, res, size, raw: line.to_owned() })
}

pub(crate) fn fincore(path: &Path) -> Result<Fincore, String> {
    let out = Command::new("fincore")
        .args(["--bytes", "--noheadings", "--raw", "--output", "PAGES,RES,SIZE,FILE"])
        .arg(path)
        .output()
        .map_err(|e| format!("fincore: {e}. It is in util-linux-extra on Ubuntu"))?;
    if !out.status.success() {
        return Err(format!(
            "fincore {}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    parse_fincore(&String::from_utf8_lossy(&out.stdout))
}

pub(crate) fn page_size() -> Result<u64, String> {
    let out =
        Command::new("getconf").arg("PAGESIZE").output().map_err(|e| format!("getconf: {e}"))?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .map_err(|_| "getconf PAGESIZE printed no number".to_owned())
}

/// `sync`, then `echo 3 > /proc/sys/vm/drop_caches`. It needs root.
pub(crate) fn drop_caches() -> Result<(), String> {
    let status = Command::new("sync").status().map_err(|e| format!("sync: {e}"))?;
    if !status.success() {
        return Err(format!("sync failed: {status}"));
    }
    fs::write("/proc/sys/vm/drop_caches", "3")
        .map_err(|e| format!("drop_caches: {e}. It needs root"))
}

/// `cat <source> > /dev/null`, timed.
pub(crate) fn timed_cat(path: &Path) -> Result<Duration, String> {
    let start = Instant::now();
    let status = Command::new("cat")
        .arg(path)
        .stdout(Stdio::null())
        .status()
        .map_err(|e| format!("cat: {e}"))?;
    let took = start.elapsed();
    if !status.success() {
        return Err(format!("cat {} failed: {status}", path.display()));
    }
    Ok(took)
}

/// The steps before the load, and what they found.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Prepared {
    /// The case that was asked for.
    pub(crate) wanted: Case,
    /// The case that the load runs in.
    pub(crate) case: Case,
    /// Why the case is not the one that was asked for.
    pub(crate) note: Option<String>,
    /// The reference time `R`. Only case A has it.
    pub(crate) read: Option<Duration>,
    /// Each `fincore` check, with the step that ran it.
    pub(crate) checks: Vec<(&'static str, Fincore)>,
    /// How many drops of the caches each case A check needed.
    pub(crate) drops: Vec<u32>,
    pub(crate) page_size: u64,
}

impl Prepared {
    /// The case A ratio `L / R`.
    pub(crate) fn ratio(&self, load: Duration) -> Option<f64> {
        self.read.map(|r| load.as_secs_f64() / r.as_secs_f64())
    }

    /// Lines for a terminal.
    pub(crate) fn text(&self, load: Duration) -> String {
        let mut t =
            format!("case            {} (asked for {})\n", self.case.name(), self.wanted.name());
        if let Some(note) = &self.note {
            t.push_str(&format!("note            {note}\n"));
        }
        for (step, f) in &self.checks {
            t.push_str(&format!(
                "fincore         {:.2} percent resident, {step}: {}\n",
                f.resident(self.page_size) * 100.0,
                f.raw
            ));
        }
        if !self.drops.is_empty() {
            let drops: Vec<String> = self.drops.iter().map(u32::to_string).collect();
            t.push_str(&format!(
                "drops           {} (drops of the caches before each check)\n",
                drops.join(", ")
            ));
        }
        if let Some(r) = self.read {
            t.push_str(&format!("R (cat)         {:.3} s\n", r.as_secs_f64()));
        }
        t.push_str(&format!("L (load)        {:.3} s\n", load.as_secs_f64()));
        if let Some(ratio) = self.ratio(load) {
            t.push_str(&format!("L / R           {ratio:.3}\n"));
        }
        t
    }

    pub(crate) fn to_json(&self, load: Option<Duration>) -> Json {
        let checks: Vec<Json> = self
            .checks
            .iter()
            .map(|(step, f)| f.to_json(self.page_size).with("step", *step))
            .collect();
        Json::obj()
            .with("case_wanted", self.wanted.name())
            .with("case", self.case.name())
            .with("note", self.note.clone())
            .with("read_s", self.read.map(|r| r.as_secs_f64()))
            .with("load_s", load.map(|l| l.as_secs_f64()))
            .with("load_over_read", load.and_then(|l| self.ratio(l)))
            .with("page_size", self.page_size)
            .with("drops", self.drops.clone())
            .with("fincore", Json::Arr(checks))
    }
}

/// Runs the steps of spec/20 section 20.3.1 before the load. The caller times the load right after this returns.
pub(crate) fn prepare(source: &Path, wanted: Case) -> Result<Prepared, String> {
    let page_size = page_size()?;
    let mut p = Prepared {
        wanted,
        case: wanted,
        note: None,
        read: None,
        checks: Vec::new(),
        drops: Vec::new(),
        page_size,
    };
    if wanted == Case::B {
        timed_cat(source)?;
        timed_cat(source)?;
        let f = fincore(source)?;
        let share = f.resident(page_size);
        p.checks.push(("case B, after two reads", f));
        if share >= CASE_B_RESIDENT {
            return Ok(p);
        }
        p.case = Case::A;
        p.note = Some(format!(
            "only {:.2} percent of the pages were resident after two reads, below the 99 percent of case B, so the run is case A",
            share * 100.0
        ));
    }
    for (i, step) in ["case A, before the read", "case A, before the load"].into_iter().enumerate()
    {
        let (f, drops) = cold(source, page_size, step)?;
        p.checks.push((step, f));
        p.drops.push(drops);
        if i == 0 {
            p.read = Some(timed_cat(source)?);
        }
    }
    Ok(p)
}

/// The number of times the harness drops the caches before it gives up on a source that stays resident.
const DROP_TRIES: u32 = 5;

/// Drops the caches until `fincore` reports 0 resident pages, and returns the check and the number of drops. A few pages can stay after one drop while they sit in a per CPU list of the kernel, so a second drop removes them. Pages that stay after five drops are held by another process, or the file is on tmpfs, and the run fails.
fn cold(source: &Path, page_size: u64, step: &str) -> Result<(Fincore, u32), String> {
    let mut last = None;
    for drops in 1..=DROP_TRIES {
        drop_caches()?;
        let f = fincore(source)?;
        if f.pages == 0 {
            return Ok((f, drops));
        }
        last = Some(f);
        std::thread::sleep(Duration::from_millis(200));
    }
    let f = last.expect("the loop ran at least once");
    Err(format!(
        "{step}: {} has {} resident pages ({:.2} percent) after {DROP_TRIES} drops of the caches. Another process holds the file open with mapped pages, or the file is on tmpfs",
        source.display(),
        f.pages,
        f.resident(page_size) * 100.0
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fincore_line() {
        let f = parse_fincore("2560 10485760 10485761 /data/hits.tsv\n").unwrap();
        assert_eq!((f.pages, f.res, f.size), (2560, 10485760, 10485761));
        // 10485761 bytes are 2561 pages of 4096 bytes.
        assert!((f.resident(4096) - 2560.0 / 2561.0).abs() < 1e-12);
        assert_eq!(f.raw, "2560 10485760 10485761 /data/hits.tsv");
        assert!(parse_fincore("").is_err());
        assert!(parse_fincore("12 x 3 f").unwrap_err().contains("RES"));
    }

    #[test]
    fn an_empty_file_is_resident() {
        let f = Fincore { pages: 0, res: 0, size: 0, raw: String::new() };
        assert_eq!(f.resident(4096), 1.0);
    }

    #[test]
    fn the_ratio_needs_case_a() {
        let mut p = Prepared {
            wanted: Case::B,
            case: Case::B,
            note: None,
            read: None,
            checks: Vec::new(),
            drops: Vec::new(),
            page_size: 4096,
        };
        assert_eq!(p.ratio(Duration::from_secs(5)), None);
        p.case = Case::A;
        p.read = Some(Duration::from_secs(4));
        assert_eq!(p.ratio(Duration::from_secs(5)), Some(1.25));
        let text = p.to_json(Some(Duration::from_secs(5))).pretty();
        assert!(text.contains("\"case_wanted\": \"B\""));
        assert!(text.contains("\"load_over_read\": 1.25"));
    }

    #[test]
    fn case_names() {
        assert_eq!(Case::parse("a").unwrap(), Case::A);
        assert_eq!(Case::parse("B").unwrap(), Case::B);
        assert!(Case::parse("c").is_err());
    }
}
