//! A JSON writer and parser for the result files. The harness has no dependencies, so this replaces `serde_json`.
//!
//! The writer escapes each character that is not ASCII, so a result file is ASCII.

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
            c if c.is_ascii() && (c as u32) >= 0x20 => out.push(c),
            c => {
                let mut units = [0u16; 2];
                for u in c.encode_utf16(&mut units) {
                    let _ = write!(out, "\\u{u:04x}");
                }
            }
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

impl Json {
    /// The value of `key` in an object.
    pub(crate) fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Parses a JSON text. An error names the byte offset.
    pub(crate) fn parse(text: &str) -> Result<Json, String> {
        let mut p = Parser { b: text.as_bytes(), at: 0 };
        let v = p.value(0)?;
        p.space();
        if p.at != p.b.len() {
            return Err(p.err("text after the value"));
        }
        Ok(v)
    }
}

/// The parser. Nesting is limited, so a bad file cannot overflow the stack.
struct Parser<'a> {
    b: &'a [u8],
    at: usize,
}

const MAX_DEPTH: usize = 128;

impl Parser<'_> {
    fn err(&self, what: &str) -> String {
        format!("json at byte {}: {what}", self.at)
    }

    fn space(&mut self) {
        while self.at < self.b.len() && matches!(self.b[self.at], b' ' | b'\t' | b'\n' | b'\r') {
            self.at += 1;
        }
    }

    fn eat(&mut self, word: &str) -> bool {
        if self.b[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            true
        } else {
            false
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > MAX_DEPTH {
            return Err(self.err("nested too deep"));
        }
        self.space();
        match self.b.get(self.at) {
            None => Err(self.err("no value")),
            Some(b'n') if self.eat("null") => Ok(Json::Null),
            Some(b't') if self.eat("true") => Ok(Json::Bool(true)),
            Some(b'f') if self.eat("false") => Ok(Json::Bool(false)),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b'[') => {
                self.at += 1;
                let mut items = Vec::new();
                self.space();
                if self.eat("]") {
                    return Ok(Json::Arr(items));
                }
                loop {
                    items.push(self.value(depth + 1)?);
                    self.space();
                    if self.eat(",") {
                        continue;
                    }
                    if self.eat("]") {
                        return Ok(Json::Arr(items));
                    }
                    return Err(self.err("expected , or ]"));
                }
            }
            Some(b'{') => {
                self.at += 1;
                let mut entries = Vec::new();
                self.space();
                if self.eat("}") {
                    return Ok(Json::Obj(entries));
                }
                loop {
                    self.space();
                    if self.b.get(self.at) != Some(&b'"') {
                        return Err(self.err("expected a key"));
                    }
                    let key = self.string()?;
                    self.space();
                    if !self.eat(":") {
                        return Err(self.err("expected :"));
                    }
                    let v = self.value(depth + 1)?;
                    entries.push((key, v));
                    self.space();
                    if self.eat(",") {
                        continue;
                    }
                    if self.eat("}") {
                        return Ok(Json::Obj(entries));
                    }
                    return Err(self.err("expected , or }"));
                }
            }
            Some(c) if *c == b'-' || c.is_ascii_digit() => self.number(),
            Some(_) => Err(self.err("not a value")),
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.at;
        while self.at < self.b.len()
            && matches!(self.b[self.at], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        {
            self.at += 1;
        }
        let s = std::str::from_utf8(&self.b[start..self.at]).map_err(|_| self.err("bad number"))?;
        if let Ok(n) = s.parse::<u64>() {
            return Ok(Json::UInt(n));
        }
        if let Ok(n) = s.parse::<i64>() {
            return Ok(Json::Int(n));
        }
        s.parse::<f64>().map(Json::Num).map_err(|_| self.err("bad number"))
    }

    fn string(&mut self) -> Result<String, String> {
        self.at += 1;
        let mut out = Vec::new();
        loop {
            let Some(&c) = self.b.get(self.at) else {
                return Err(self.err("a string with no end"));
            };
            self.at += 1;
            match c {
                b'"' => {
                    return String::from_utf8(out)
                        .map_err(|_| self.err("a string that is not UTF-8"));
                }
                b'\\' => {
                    let Some(&e) = self.b.get(self.at) else {
                        return Err(self.err("a string with no end"));
                    };
                    self.at += 1;
                    match e {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let mut code = self.hex4()?;
                            if (0xd800..0xdc00).contains(&code) {
                                if !self.eat("\\u") {
                                    return Err(self.err("a lone surrogate"));
                                }
                                let low = self.hex4()?;
                                if !(0xdc00..0xe000).contains(&low) {
                                    return Err(self.err("a lone surrogate"));
                                }
                                code = 0x10000 + ((code - 0xd800) << 10) + (low - 0xdc00);
                            }
                            let ch = char::from_u32(code).ok_or(self.err("a bad escape"))?;
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => return Err(self.err("a bad escape")),
                    }
                }
                c => out.push(c),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let s = self.b.get(self.at..self.at + 4).ok_or(self.err("a short escape"))?;
        let s = std::str::from_utf8(s).map_err(|_| self.err("a bad escape"))?;
        let n = u32::from_str_radix(s, 16).map_err(|_| self.err("a bad escape"))?;
        self.at += 4;
        Ok(n)
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

    #[test]
    fn parse_what_the_writer_writes() {
        let j = Json::obj()
            .with("name", "a \"b\"\n\u{e9}\u{2014}\u{1f600}")
            .with("times", vec![0.5, 1.25])
            .with("big", u64::MAX)
            .with("neg", -3i64)
            .with("none", Option::<u64>::None)
            .with("yes", true)
            .with("rows", vec![Json::obj().with("n", 1u64)])
            .with("empty", Json::obj());
        let text = j.pretty();
        assert!(text.is_ascii());
        assert!(text.contains("\\u00e9\\u2014\\ud83d\\ude00"));
        assert_eq!(Json::parse(&text).unwrap(), j);
        assert_eq!(j.get("neg"), Some(&Json::Int(-3)));
        assert_eq!(j.get("yes").and_then(Json::as_bool), Some(true));
    }

    #[test]
    fn parse_errors() {
        assert_eq!(
            Json::parse(" [1, 2.5e1, \"x\"] ").unwrap(),
            Json::Arr(vec![Json::UInt(1), Json::Num(25.0), Json::Str("x".into())])
        );
        for bad in
            ["", "[1,", "{\"a\" 1}", "[1] x", "\"abc", "\"\\q\"", "\"\\ud800\"", "nul", "[-]"]
        {
            assert!(Json::parse(bad).is_err(), "{bad:?}");
        }
        let deep = "[".repeat(200) + &"]".repeat(200);
        assert!(Json::parse(&deep).unwrap_err().contains("nested too deep"));
    }
}
