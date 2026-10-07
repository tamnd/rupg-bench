//! The pins of `pins.toml`: the version of every system that a run uses.
//!
//! spec/20 section 20.1 asks for exact versions. A run copies the pins into its result, next to the version string that each system reports.

use crate::toml::{Doc, Table};

/// The tables that `pins.toml` must have, with the keys that each table must have.
const REQUIRED: [(&str, &[&str]); 11] = [
    ("clickbench", &["repo", "commit"]),
    ("postgresql", &["repo", "branch", "commit", "version"]),
    ("rudb", &["repo", "commit"]),
    ("rupg", &["repo", "commit"]),
    ("clickhouse", &["version", "url", "sha256"]),
    ("duckdb", &["version", "url", "sha256"]),
    ("umbra", &["version", "image", "license"]),
    ("cedardb", &["version", "license"]),
    ("sqlite", &["version", "url", "sha3_256"]),
    ("hammerdb", &["version", "url", "sha256"]),
    ("tpch_tools", &["version", "repo", "commit", "path"]),
];

/// The pins of one file.
#[derive(Clone, Debug)]
pub(crate) struct Pins {
    doc: Doc,
}

impl Pins {
    /// Reads and checks `pins.toml`.
    pub(crate) fn parse(text: &str) -> Result<Pins, String> {
        let doc = Doc::parse(text)?;
        for (name, keys) in REQUIRED {
            let table = doc.table(name).ok_or(format!("pins.toml has no [{name}]"))?;
            for key in keys {
                if table.str(key).is_none() {
                    return Err(format!("[{name}] has no string {key}"));
                }
            }
        }
        for table in &doc.tables {
            if let Some(commit) = table.str("commit")
                && !commit.is_empty()
                && !is_full_sha(commit)
            {
                return Err(format!("[{}] commit {commit:?} is not a full SHA-1", table.name));
            }
            for key in ["sha256", "sha3_256"] {
                if let Some(h) = table.str(key)
                    && !(h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
                {
                    return Err(format!("[{}] {key} is not 64 hex digits", table.name));
                }
            }
        }
        Ok(Pins { doc })
    }

    pub(crate) fn load(path: &str) -> Result<Pins, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        Pins::parse(&text).map_err(|e| format!("{path}: {e}"))
    }

    pub(crate) fn get(&self, system: &str, key: &str) -> Option<&str> {
        self.doc.table(system)?.str(key)
    }

    /// The tables in the order of the file, without the root table.
    pub(crate) fn systems(&self) -> impl Iterator<Item = &Table> {
        self.doc.tables.iter().filter(|t| !t.name.is_empty())
    }

    /// One line for each system: the name, the version or the short commit, and the date.
    pub(crate) fn summary(&self) -> String {
        let mut out = String::new();
        for t in self.systems() {
            let version = t.str("version").unwrap_or("");
            let commit = t.str("commit").map(|c| &c[..c.len().min(9)]).unwrap_or("");
            let id = match (version.is_empty(), commit.is_empty()) {
                (false, false) => format!("{version} at {commit}"),
                (false, true) => version.to_owned(),
                (true, false) => commit.to_owned(),
                (true, true) => "not pinned".to_owned(),
            };
            let date = t.str("date").unwrap_or("");
            out.push_str(format!("{:<12} {:<28} {}", t.name, id, date).trim_end());
            out.push('\n');
        }
        out
    }

    /// The pins as shell assignments, `PIN_<SYSTEM>_<KEY>='value'`, for the machine scripts.
    pub(crate) fn shell(&self) -> String {
        let mut out = String::new();
        for t in self.systems() {
            for (key, value) in &t.entries {
                let var = format!("PIN_{}_{}", t.name, key).to_uppercase().replace('-', "_");
                let value = value.to_string().replace('\'', r"'\''");
                out.push_str(&format!("{var}='{value}'\n"));
            }
        }
        out
    }
}

fn is_full_sha(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = include_str!("../pins.toml");

    #[test]
    fn the_file_has_the_pins_of_the_spec() {
        let pins = Pins::parse(FILE).unwrap();
        // spec/20, first paragraph, and issue #1.
        assert!(pins.get("clickbench", "commit").unwrap().starts_with("e6bda4e"));
        assert!(pins.get("postgresql", "commit").unwrap().starts_with("7d3d2db7"));
        assert_eq!(pins.get("postgresql", "branch"), Some("REL_19_STABLE"));
        assert!(pins.get("rudb", "commit").unwrap().starts_with("cd9f9676"));
        assert_eq!(pins.get("tpch_tools", "version"), Some("3.0.1"));
    }

    #[test]
    fn the_rust_pin_matches_the_toolchain_file() {
        let pins = Pins::parse(FILE).unwrap();
        let toolchain = include_str!("../rust-toolchain.toml");
        let want = format!("channel = \"{}\"", pins.get("rust", "version").unwrap());
        assert!(toolchain.contains(&want));
    }

    #[test]
    fn a_short_commit_or_a_missing_table_fails() {
        let short = FILE.replace("e6bda4e229d7ba3543662eec26fbf514cb3fe935", "e6bda4e");
        assert!(Pins::parse(&short).unwrap_err().contains("full SHA-1"));
        let no_duckdb = FILE.replace("[duckdb]", "[duck]");
        assert!(Pins::parse(&no_duckdb).unwrap_err().contains("[duckdb]"));
    }

    #[test]
    fn shell_quotes_the_values() {
        let pins =
            Pins::parse("[clickbench]\nrepo = \"r\"\ncommit = \"\"\n[x-y]\nnote = \"it's\"\n");
        // The file above misses most tables, so build the output from the document directly.
        assert!(pins.is_err());
        let pins = Pins { doc: Doc::parse("[x-y]\nnote = \"it's\"\nn = 3\n").unwrap() };
        assert_eq!(pins.shell(), "PIN_X_Y_NOTE='it'\\''s'\nPIN_X_Y_N='3'\n");
    }

    #[test]
    fn the_summary_has_one_line_for_each_system() {
        let pins = Pins::parse(FILE).unwrap();
        let summary = pins.summary();
        assert_eq!(summary.lines().count(), pins.systems().count());
        assert!(summary.contains("postgresql   19beta4 at 7d3d2db7d"));
        assert!(summary.contains("rupg         not pinned"));
    }
}
