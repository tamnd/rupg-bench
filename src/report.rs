//! The report generator of spec/20 section 20.12.
//!
//! Each run writes `reports/<date>/<commit>-<machine>-<suite>.json` with every number of the run, and `.md` next to it for a reader. The Markdown file is made from the JSON file only, so `rupg-bench report` can make it again from a JSON file. A smoke run has `-smoke` at the end of its name and a line at the top that says it is not a baseline.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::json::Json;

/// The fields that name a result. Each driver sets them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Meta {
    pub(crate) suite: String,
    pub(crate) machine: String,
    /// `YYYY-MM-DD` in UTC.
    pub(crate) date: String,
    /// The commit of the system that the run measures, or of the harness for a baseline run.
    pub(crate) commit: String,
    pub(crate) smoke: bool,
}

impl Meta {
    /// The meta of a run that starts now on this machine. The machine is the host name unless `machine` is given.
    pub(crate) fn now(
        suite: &str,
        machine: Option<String>,
        commit: Option<String>,
        smoke: bool,
    ) -> Meta {
        let machine = machine.unwrap_or_else(|| {
            fs::read_to_string("/proc/sys/kernel/hostname")
                .map(|s| s.trim().to_owned())
                .unwrap_or_else(|_| "unknown".to_owned())
        });
        let commit = commit.unwrap_or_else(harness_commit);
        Meta { suite: suite.to_owned(), machine, date: today(), commit, smoke }
    }

    pub(crate) fn to_json(&self) -> Json {
        Json::obj()
            .with("suite", self.suite.as_str())
            .with("machine", self.machine.as_str())
            .with("date", self.date.as_str())
            .with("commit", self.commit.as_str())
            .with("smoke", self.smoke)
    }

    pub(crate) fn from_json(j: &Json) -> Result<Meta, String> {
        let s = |k: &str| {
            j.get(k)
                .and_then(Json::as_str)
                .map(str::to_owned)
                .ok_or(format!("the result has no {k}"))
        };
        Ok(Meta {
            suite: s("suite")?,
            machine: s("machine")?,
            date: s("date")?,
            commit: s("commit")?,
            smoke: j.get("smoke").and_then(Json::as_bool).ok_or("the result has no smoke field")?,
        })
    }

    /// `<date>/<commit>-<machine>-<suite>`, with `-smoke` for a smoke run.
    pub(crate) fn stem(&self) -> PathBuf {
        let mut name = format!("{}-{}-{}", self.commit, self.machine, self.suite);
        if self.smoke {
            name.push_str("-smoke");
        }
        Path::new(&self.date).join(name)
    }
}

/// The short commit of the harness: `RUPG_BENCH_COMMIT` when it is set, for a copy of the tree with no `.git`, else `git rev-parse` in the tree, else `unknown`.
fn harness_commit() -> String {
    if let Ok(c) = std::env::var("RUPG_BENCH_COMMIT") {
        if !c.is_empty() {
            return c;
        }
    }
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Today in UTC as `YYYY-MM-DD`.
pub(crate) fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    civil_date(secs / 86400)
}

/// The date of a day number since 1970-01-01 (the algorithm of Howard Hinnant's `civil_from_days`).
fn civil_date(days: u64) -> String {
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Writes the JSON file and the Markdown file under `root` and returns their paths.
pub(crate) fn write(root: &Path, result: &Json) -> Result<(PathBuf, PathBuf), String> {
    let meta = Meta::from_json(result)?;
    let stem = root.join(meta.stem());
    let dir = stem.parent().ok_or("a report path with no directory")?;
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let json = stem.with_extension("json");
    let md = stem.with_extension("md");
    fs::write(&json, result.pretty()).map_err(|e| format!("{}: {e}", json.display()))?;
    fs::write(&md, markdown(result)?).map_err(|e| format!("{}: {e}", md.display()))?;
    Ok((md, json))
}

/// The Markdown form of a result. Scalars go in a table of fields. An object gets a section with a table of its fields, and nested objects get dotted names. An array of objects gets a table with one row for each object. Text with more than one line goes in a code block.
pub(crate) fn markdown(result: &Json) -> Result<String, String> {
    let meta = Meta::from_json(result)?;
    let mut out = format!("# {} on {}, {}\n\n", meta.suite, meta.machine, meta.date);
    if meta.smoke {
        out.push_str("**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.\n\n");
    } else {
        out.push_str("This is a measured run with the rules of spec/20.\n\n");
    }
    let Json::Obj(entries) = result else { return Err("a result must be an object".to_owned()) };
    let mut fields = Vec::new();
    let mut blocks = Vec::new();
    let mut sections = Vec::new();
    for (k, v) in entries {
        match v {
            Json::Obj(_) => sections.push((k.as_str(), v)),
            Json::Arr(items) if items.iter().any(|i| matches!(i, Json::Obj(_))) => {
                sections.push((k.as_str(), v))
            }
            Json::Str(s) if needs_block(s) => blocks.push((k.clone(), s.clone())),
            _ => fields.push((k.clone(), scalar(v))),
        }
    }
    table(&mut out, &fields);
    code_blocks(&mut out, &blocks);
    for (k, v) in sections {
        out.push_str(&format!("## {k}\n\n"));
        match v {
            Json::Arr(items) => rows(&mut out, items),
            _ => {
                let mut fields = Vec::new();
                let mut blocks = Vec::new();
                flatten("", v, &mut fields, &mut blocks);
                table(&mut out, &fields);
                code_blocks(&mut out, &blocks);
            }
        }
    }
    Ok(out)
}

fn flatten(
    prefix: &str,
    v: &Json,
    fields: &mut Vec<(String, String)>,
    blocks: &mut Vec<(String, String)>,
) {
    match v {
        Json::Obj(entries) => {
            for (k, v) in entries {
                let name = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                flatten(&name, v, fields, blocks);
            }
        }
        Json::Arr(items) if items.iter().any(|i| matches!(i, Json::Obj(_))) => {
            for (i, item) in items.iter().enumerate() {
                flatten(&format!("{prefix}.{i}"), item, fields, blocks);
            }
        }
        Json::Str(s) if needs_block(s) => blocks.push((prefix.to_owned(), s.clone())),
        _ => fields.push((prefix.to_owned(), scalar(v))),
    }
}

/// Text with more than one line goes in a code block. So does text with a hyphen between two spaces, which the style check of the reports does not allow in running text.
fn needs_block(s: &str) -> bool {
    s.contains('\n') || s.contains(" - ")
}

/// The style check of the reports does not allow the en dash and the em dash, so the Markdown file shows them as JSON escapes. The JSON file escapes them too.
fn ascii_dashes(s: &str) -> String {
    s.replace('\u{2013}', "\\u2013").replace('\u{2014}', "\\u2014")
}

/// One cell. A `|` in text would end the cell, so it is escaped.
fn scalar(v: &Json) -> String {
    let s = match v {
        Json::Null => "null".to_owned(),
        Json::Bool(b) => b.to_string(),
        Json::Int(n) => n.to_string(),
        Json::UInt(n) => n.to_string(),
        Json::Num(x) => format_num(*x),
        Json::Str(s) => s.clone(),
        Json::Arr(items) => items.iter().map(scalar).collect::<Vec<_>>().join(", "),
        Json::Obj(_) => "(object)".to_owned(),
    };
    ascii_dashes(&s).replace('|', "\\|").replace('\n', " ")
}

/// A number with at most 6 decimals and no trailing zeros.
fn format_num(x: f64) -> String {
    if !x.is_finite() {
        return "null".to_owned();
    }
    let s = format!("{x:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".to_owned() } else { s.to_owned() }
}

fn table(out: &mut String, fields: &[(String, String)]) {
    if fields.is_empty() {
        return;
    }
    out.push_str("| Field | Value |\n|---|---|\n");
    for (k, v) in fields {
        out.push_str(&format!("| {k} | {v} |\n"));
    }
    out.push('\n');
}

fn code_blocks(out: &mut String, blocks: &[(String, String)]) {
    for (k, text) in blocks {
        out.push_str(&format!("{k}:\n\n```\n{}\n```\n\n", ascii_dashes(text.trim_end())));
    }
}

/// A table with one row for each object. The columns are the scalar fields of the objects, in the order of first use.
fn rows(out: &mut String, items: &[Json]) {
    let mut columns: Vec<String> = Vec::new();
    let mut flat: Vec<Vec<(String, String)>> = Vec::new();
    for item in items {
        let mut fields = Vec::new();
        let mut blocks = Vec::new();
        flatten("", item, &mut fields, &mut blocks);
        for (k, _) in &fields {
            if !columns.contains(k) {
                columns.push(k.clone());
            }
        }
        flat.push(fields);
    }
    if columns.is_empty() {
        return;
    }
    out.push_str(&format!("| {} |\n|{}\n", columns.join(" | "), "---|".repeat(columns.len())));
    for fields in flat {
        let cells: Vec<&str> = columns
            .iter()
            .map(|c| fields.iter().find(|(k, _)| k == c).map_or("", |(_, v)| v.as_str()))
            .collect();
        out.push_str(&format!("| {} |\n", cells.join(" | ")));
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Json {
        Meta {
            suite: "pgbench".to_owned(),
            machine: "server3".to_owned(),
            date: "2026-10-07".to_owned(),
            commit: "abc12345".to_owned(),
            smoke: true,
        }
        .to_json()
        .with("tps", 411.4976171)
        .with("output", "line one\nline | two")
        .with("command", "a - b")
        .with("dash", "x \u{2014} y")
        .with(
            "run",
            Json::obj().with("cpu_usec", 10u64).with("io", Json::obj().with("rbytes", 2u64)),
        )
        .with(
            "queries",
            vec![
                Json::obj().with("q", "Q1").with("times", vec![0.5, 0.25]),
                Json::obj().with("q", "Q2").with("cold", 1.0),
            ],
        )
    }

    #[test]
    fn the_name_of_a_report() {
        let m = Meta::from_json(&sample()).unwrap();
        assert_eq!(m.stem(), Path::new("2026-10-07/abc12345-server3-pgbench-smoke"));
    }

    #[test]
    fn markdown_of_a_result() {
        let md = markdown(&sample()).unwrap();
        assert!(md.starts_with("# pgbench on server3, 2026-10-07\n\n**This is a smoke run.**"));
        assert!(md.contains("| tps | 411.497617 |"));
        assert!(md.contains("output:\n\n```\nline one\nline | two\n```"));
        assert!(md.contains("command:\n\n```\na - b\n```"));
        assert!(md.contains("| dash | x \\u2014 y |"));
        assert!(md.contains(
            "## run\n\n| Field | Value |\n|---|---|\n| cpu_usec | 10 |\n| io.rbytes | 2 |"
        ));
        assert!(
            md.contains(
                "| q | times | cold |\n|---|---|---|\n| Q1 | 0.5, 0.25 |  |\n| Q2 |  | 1 |"
            )
        );
    }

    #[test]
    fn dates() {
        assert_eq!(civil_date(0), "1970-01-01");
        assert_eq!(civil_date(20_733), "2026-10-07");
        assert_eq!(civil_date(11_016), "2000-02-29");
    }

    #[test]
    fn numbers() {
        assert_eq!(format_num(1.0), "1");
        assert_eq!(format_num(0.1234567), "0.123457");
        assert_eq!(format_num(-0.0000001), "0");
    }
}
