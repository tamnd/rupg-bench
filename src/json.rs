//! A JSON writer for the result files. The harness has no dependencies, so this replaces `serde_json`.

use std::fmt::Write as _;

/// One JSON value. Objects keep the order of their keys, so a result file reads in the order that the code writes it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Json {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub(crate) fn obj() -> Json {
        Json::Obj(Vec::new())
    }

    /// Adds a key to an object and returns the object. It panics on a value that is not an object, which is a bug in the caller.
    #[must_use]
    pub(crate) fn with(mut self, key: &str, value: impl Into<Json>) -> Json {
        match &mut self {
            Json::Obj(entries) => entries.push((key.to_owned(), value.into())),
            _ => panic!("Json::with on a value that is not an object"),
        }
        self
    }

    /// The value with two spaces of indent and a final newline.
    pub(crate) fn pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    fn write(&self, out: &mut String, depth: usize) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Int(n) => {
                let _ = write!(out, "{n}");
            }
            Json::UInt(n) => {
                let _ = write!(out, "{n}");
            }
            Json::Num(x) if x.is_finite() => {
                let _ = write!(out, "{x}");
            }
            Json::Num(_) => out.push_str("null"),
            Json::Str(s) => quote(out, s),
            Json::Arr(items) if items.is_empty() => out.push_str("[]"),
            Json::Arr(items) => {
                // Arrays of numbers stay on one line, for example the three times of a query.
                let flat = items
                    .iter()
                    .all(|i| matches!(i, Json::Int(_) | Json::UInt(_) | Json::Num(_) | Json::Null));
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    if flat {
                        if i > 0 {
                            out.push(' ');
                        }
                    } else {
                        newline(out, depth + 1);
                    }
                    item.write(out, depth + 1);
                }
                if !flat {
                    newline(out, depth);
                }
                out.push(']');
            }
            Json::Obj(entries) if entries.is_empty() => out.push_str("{}"),
            Json::Obj(entries) => {
                out.push('{');
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    newline(out, depth + 1);
                    quote(out, key);
                    out.push_str(": ");
                    value.write(out, depth + 1);
                }
                newline(out, depth);
                out.push('}');
            }
        }
    }
}

fn newline(out: &mut String, depth: usize) {
    out.push('\n');
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn quote(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

impl From<bool> for Json {
    fn from(b: bool) -> Json {
        Json::Bool(b)
    }
}

impl From<u64> for Json {
    fn from(n: u64) -> Json {
        Json::UInt(n)
    }
}

impl From<u32> for Json {
    fn from(n: u32) -> Json {
        Json::UInt(n.into())
    }
}

impl From<usize> for Json {
    fn from(n: usize) -> Json {
        Json::UInt(n as u64)
    }
}

impl From<i64> for Json {
    fn from(n: i64) -> Json {
        Json::Int(n)
    }
}

impl From<f64> for Json {
    fn from(x: f64) -> Json {
        Json::Num(x)
    }
}

impl From<&str> for Json {
    fn from(s: &str) -> Json {
        Json::Str(s.to_owned())
    }
}

impl From<String> for Json {
    fn from(s: String) -> Json {
        Json::Str(s)
    }
}

impl<T: Into<Json>> From<Option<T>> for Json {
    fn from(v: Option<T>) -> Json {
        v.map_or(Json::Null, Into::into)
    }
}

impl<T: Into<Json>> From<Vec<T>> for Json {
    fn from(v: Vec<T>) -> Json {
        Json::Arr(v.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objects_arrays_and_escapes() {
        let j = Json::obj()
            .with("name", "a \"b\"\n")
            .with("times", vec![0.5, 1.25])
            .with("none", Option::<u64>::None)
            .with("nan", f64::NAN)
            .with("rows", vec![Json::obj().with("n", 1u64)])
            .with("empty", Json::obj());
        let want = "{\n  \"name\": \"a \\\"b\\\"\\n\",\n  \"times\": [0.5, 1.25],\n  \"none\": null,\n  \"nan\": null,\n  \"rows\": [\n    {\n      \"n\": 1\n    }\n  ],\n  \"empty\": {}\n}\n";
        assert_eq!(j.pretty(), want);
    }
}
