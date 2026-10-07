//! A reader for the small part of TOML that `pins.toml` and `ratchet.toml` use.
//!
//! It reads tables (`[name]` and `[a.b]`), bare keys, basic and literal strings, integers, floats and booleans. It does not read arrays, inline tables or multi-line strings. A file that uses them fails with the line number. The harness has no dependencies, so this reader replaces the `toml` crate.

use std::fmt;

/// One value.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => f.write_str(s),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::Bool(b) => write!(f, "{b}"),
        }
    }
}

/// One table with its keys in the order of the file.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Table {
    pub(crate) name: String,
    pub(crate) entries: Vec<(String, Value)>,
}

impl Table {
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub(crate) fn str(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Value::Str(s)) => Some(s),
            _ => None,
        }
    }
}

/// A parsed file. Keys before the first table header go into a table with an empty name.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Doc {
    pub(crate) tables: Vec<Table>,
}

impl Doc {
    pub(crate) fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    /// Parses `text`. An error names the line.
    pub(crate) fn parse(text: &str) -> Result<Doc, String> {
        let mut doc = Doc { tables: vec![Table::default()] };
        for (i, raw) in text.lines().enumerate() {
            let n = i + 1;
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix('[') {
                let (name, tail) =
                    rest.split_once(']').ok_or(format!("line {n}: no ] after the table name"))?;
                let name = name.trim();
                if name.starts_with('[') || !valid_table_name(name) {
                    return Err(format!("line {n}: the table name {name:?} is not supported"));
                }
                if !is_comment_or_empty(tail) {
                    return Err(format!("line {n}: text after the table header"));
                }
                if doc.table(name).is_some() {
                    return Err(format!("line {n}: the table [{name}] is defined twice"));
                }
                doc.tables.push(Table { name: name.to_owned(), entries: Vec::new() });
                continue;
            }
            let (key, rest) =
                line.split_once('=').ok_or(format!("line {n}: a line must be a key = value"))?;
            let key = key.trim();
            if !valid_key(key) {
                return Err(format!("line {n}: the key {key:?} is not a bare key"));
            }
            let (value, tail) = parse_value(rest.trim()).map_err(|e| format!("line {n}: {e}"))?;
            if !is_comment_or_empty(tail) {
                return Err(format!("line {n}: text after the value"));
            }
            let table = doc.tables.last_mut().expect("the root table is always there");
            if table.get(key).is_some() {
                return Err(format!("line {n}: the key {key} is defined twice"));
            }
            table.entries.push((key.to_owned(), value));
        }
        Ok(doc)
    }
}

fn is_comment_or_empty(s: &str) -> bool {
    let s = s.trim();
    s.is_empty() || s.starts_with('#')
}

fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn valid_table_name(name: &str) -> bool {
    !name.is_empty() && name.split('.').all(valid_key)
}

/// Parses one value at the start of `s` and returns it with the rest of the line.
fn parse_value(s: &str) -> Result<(Value, &str), String> {
    if let Some(rest) = s.strip_prefix('"') {
        let mut out = String::new();
        let mut chars = rest.char_indices();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' => return Ok((Value::Str(out), &rest[i + 1..])),
                '\\' => match chars.next() {
                    Some((_, '"')) => out.push('"'),
                    Some((_, '\\')) => out.push('\\'),
                    Some((_, 'n')) => out.push('\n'),
                    Some((_, 't')) => out.push('\t'),
                    _ => return Err("an escape that is not \\\" \\\\ \\n or \\t".to_owned()),
                },
                c => out.push(c),
            }
        }
        return Err("a string with no closing quote".to_owned());
    }
    if let Some(rest) = s.strip_prefix('\'') {
        let end = rest.find('\'').ok_or("a string with no closing quote")?;
        return Ok((Value::Str(rest[..end].to_owned()), &rest[end + 1..]));
    }
    let end = s.find(|c: char| c.is_whitespace() || c == '#').unwrap_or(s.len());
    let (word, tail) = s.split_at(end);
    let value = match word {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "" => return Err("no value".to_owned()),
        w => {
            let digits: String = w.chars().filter(|&c| c != '_').collect();
            if let Ok(n) = digits.parse::<i64>() {
                Value::Int(n)
            } else if let Ok(x) = digits.parse::<f64>() {
                Value::Float(x)
            } else {
                return Err(format!("the value {w:?} is not supported"));
            }
        }
    };
    Ok((value, tail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_keys_and_values() {
        let doc = Doc::parse(
            "top = 1\n# a comment\n[a]\ns = \"x \\\"y\\\"\" # trailing\nl = 'c:\\path'\n\n[a.b]\nn = 1_000\nf = 0.5\nt = true\n",
        )
        .unwrap();
        assert_eq!(doc.table("").unwrap().get("top"), Some(&Value::Int(1)));
        let a = doc.table("a").unwrap();
        assert_eq!(a.str("s"), Some("x \"y\""));
        assert_eq!(a.str("l"), Some("c:\\path"));
        let b = doc.table("a.b").unwrap();
        assert_eq!(b.get("n"), Some(&Value::Int(1000)));
        assert_eq!(b.get("f"), Some(&Value::Float(0.5)));
        assert_eq!(b.get("t"), Some(&Value::Bool(true)));
    }

    #[test]
    fn errors_name_the_line() {
        for (text, want) in [
            ("[a]\nk = 1\nk = 2\n", "line 3"),
            ("[a]\n[a]\n", "line 2"),
            ("k = [1, 2]\n", "line 1"),
            ("k = \"open\n", "line 1"),
            ("[[array]]\n", "line 1"),
            ("k = 1 2\n", "line 1"),
            ("no value here\n", "line 1"),
        ] {
            let err = Doc::parse(text).unwrap_err();
            assert!(err.starts_with(want), "{text:?} gave {err}");
        }
    }
}
