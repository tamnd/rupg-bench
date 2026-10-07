//! Answer sets and the answer check of spec/20 sections 20.5 and 20.6.
//!
//! An answer set is a directory with one file for each query, `<query>.tsv`. The first line has the column names. Each other line is one row in the text format of PostgreSQL `COPY`: a tab between fields, `\N` for NULL, and `\\`, `\t`, `\n` and `\r` for those bytes in a value. So `COPY (<query>) TO STDOUT WITH (HEADER)` writes an answer file with no other step, and PostgreSQL 15 and later print each float with the shortest exact digits.
//!
//! The check compares two answers as multisets of rows. It sorts both by all columns and then compares row by row. Two numbers match when they differ by at most the tolerance. Two texts match when they are equal after the spaces at the end are removed, because `char(n)` pads with spaces and the TPC-H answer files pad each column.

use std::cmp::Ordering;
use std::fmt::Write as _;

/// The answer of one query.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Answer {
    pub(crate) columns: Vec<String>,
    pub(crate) rows: Vec<Vec<Option<String>>>,
}

/// How two numbers are compared.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Tolerance {
    /// `|a - b| <= r * max(|a|, |b|)`. spec/20 section 20.5 sets 1e-9 for ClickBench.
    Relative(f64),
    /// The rules of TPC-H clause 2.1.3.5 for the answer set at SF1. The column name of the answer file gives the kind of value (see `tpch_kind`). A column value or a count must be equal after the actual value is rounded to the decimals of the expected value. A sum may differ by 100. An average, rounded to 2 decimals, must be within 1 percent. A ratio of sums must pass both rules.
    Tpch,
}

/// The tolerance of spec/20 section 20.5.
pub(crate) const RELATIVE: Tolerance = Tolerance::Relative(1e-9);

/// Reads an answer in the `COPY` text format with a header line.
pub(crate) fn parse_tsv(text: &str) -> Result<Answer, String> {
    let mut lines = text.split('\n');
    let header = lines.next().ok_or("an answer file with no header")?;
    let columns: Vec<String> = header
        .split('\t')
        .map(unescape)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect();
    let mut rows = Vec::new();
    for (i, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let row: Vec<Option<String>> = line.split('\t').map(unescape).collect::<Result<_, _>>()?;
        if row.len() != columns.len() {
            return Err(format!(
                "line {} has {} fields and the header has {}",
                i + 2,
                row.len(),
                columns.len()
            ));
        }
        rows.push(row);
    }
    Ok(Answer { columns, rows })
}

fn unescape(field: &str) -> Result<Option<String>, String> {
    if field == "\\N" {
        return Ok(None);
    }
    let mut out = String::with_capacity(field.len());
    let mut chars = field.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            other => {
                return Err(format!(
                    "a bad escape \\{} in {field:?}",
                    other.map(String::from).unwrap_or_default()
                ));
            }
        }
    }
    Ok(Some(out))
}

/// Writes an answer in the `COPY` text format with a header line, so `parse_tsv` reads it back.
pub(crate) fn to_tsv(answer: &Answer) -> String {
    let mut out = String::new();
    let line = |out: &mut String, fields: &mut dyn Iterator<Item = Option<&str>>| {
        for (i, f) in fields.enumerate() {
            if i > 0 {
                out.push('\t');
            }
            match f {
                None => out.push_str("\\N"),
                Some(v) => {
                    for c in v.chars() {
                        match c {
                            '\\' => out.push_str("\\\\"),
                            '\t' => out.push_str("\\t"),
                            '\n' => out.push_str("\\n"),
                            '\r' => out.push_str("\\r"),
                            c => out.push(c),
                        }
                    }
                }
            }
        }
        out.push('\n');
    };
    line(&mut out, &mut answer.columns.iter().map(|c| Some(c.as_str())));
    for row in &answer.rows {
        line(&mut out, &mut row.iter().map(Option::as_deref));
    }
    out
}

/// Reads CSV with a header line, as the `COPY ... (FORMAT csv, HEADER)` of DuckDB writes it. An empty field with no quotes is NULL, and `""` is the empty string, which is how DuckDB writes the two.
pub(crate) fn parse_csv(text: &str) -> Result<Answer, String> {
    let mut records = Vec::new();
    let mut record: Vec<Option<String>> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    let mut in_quotes = false;
    let mut at_start = true;
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' if field.is_empty() && !quoted => {
                in_quotes = true;
                quoted = true;
            }
            ',' | '\n' => {
                let value = std::mem::take(&mut field);
                record.push(if value.is_empty() && !quoted { None } else { Some(value) });
                quoted = false;
                if c == '\n' {
                    records.push(std::mem::take(&mut record));
                }
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            c => field.push(c),
        }
        at_start = false;
    }
    if in_quotes {
        return Err("a CSV field has no closing quote".to_owned());
    }
    if !at_start && (!field.is_empty() || quoted || !record.is_empty()) {
        record.push(if field.is_empty() && !quoted { None } else { Some(field) });
        records.push(record);
    }
    let mut records = records.into_iter();
    let columns: Vec<String> = records
        .next()
        .ok_or("a CSV answer with no header")?
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect();
    let mut rows = Vec::new();
    for (i, row) in records.enumerate() {
        if row.len() != columns.len() {
            return Err(format!(
                "CSV record {} has {} fields and the header has {}",
                i + 2,
                row.len(),
                columns.len()
            ));
        }
        rows.push(row);
    }
    Ok(Answer { columns, rows })
}

/// Reads a file of the TPC-H answer set, for example `dbgen/answers/q1.out`: a header line, then one line for each row, with `|` between the fields and spaces that pad them. Spaces pad a number on the left and a text on the right. A text can start with a space in the data, for example a `c_address`, so only the spaces at its end are removed.
pub(crate) fn parse_tpch(text: &str) -> Result<Answer, String> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or("an answer file with no header")?;
    let columns: Vec<String> = header.split('|').map(|c| c.trim().to_owned()).collect();
    let field = |f: &str| {
        let t = f.trim();
        Some(if number(t).is_some() { t.to_owned() } else { f.trim_end().to_owned() })
    };
    let mut rows = Vec::new();
    for line in lines {
        let row: Vec<Option<String>> = line.split('|').map(field).collect();
        if row.len() != columns.len() {
            return Err(format!(
                "{line:?} has {} fields and the header has {}",
                row.len(),
                columns.len()
            ));
        }
        rows.push(row);
    }
    Ok(Answer { columns, rows })
}

/// The value as a number, when it has the form of one. `NaN` and `Infinity` stay text and must be equal.
fn number(v: &str) -> Option<f64> {
    let v = v.trim();
    let ok = v.bytes().any(|b| b.is_ascii_digit())
        && v.bytes().all(|b| matches!(b, b'0'..=b'9' | b'+' | b'-' | b'.' | b'e' | b'E'));
    if ok { v.parse().ok() } else { None }
}

fn decimals(v: &str) -> usize {
    let v = v.trim();
    let mantissa = v.split(['e', 'E']).next().unwrap_or(v);
    mantissa.split_once('.').map_or(0, |(_, d)| d.len())
}

/// The kinds of value of TPC-H clause 2.1.3.5.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TpchKind {
    /// A column value, a count, or a `sum(l_quantity)` (comment 4 of the clause).
    Exact,
    /// The result of a `sum`: within 100.
    Sum,
    /// The result of an `avg`: within 1 percent after rounding to 2 decimals.
    Avg,
    /// A ratio of sums, as in Q8, Q14 and Q17: both rules (comment 1 of the clause).
    SumRatio,
}

/// The kind of a column of the TPC-H answer set at SF1, from its name in the answer file.
pub(crate) fn tpch_kind(column: &str) -> TpchKind {
    match column {
        // Q1 sum_qty and Q18 col6 are sum(l_quantity).
        "sum_qty" | "col6" => TpchKind::Exact,
        "mkt_share" | "promo_revenue" | "avg_yearly" => TpchKind::SumRatio,
        "revenue" | "total_revenue" | "value" | "totacctbal" => TpchKind::Sum,
        c if c.starts_with("sum_") => TpchKind::Sum,
        c if c.starts_with("avg_") => TpchKind::Avg,
        _ => TpchKind::Exact,
    }
}

fn round(x: f64, decimals: usize) -> String {
    let s = format!("{x:.decimals$}");
    if s.trim_start_matches(['-', '0', '.']).is_empty() {
        s.trim_start_matches('-').to_owned()
    } else {
        s
    }
}

/// `0.99 * v <= round(r, 2) <= 1.01 * v`, for a negative `v` too.
fn within_one_percent(v: f64, r: f64) -> bool {
    let r: f64 = round(r, 2).parse().unwrap_or(f64::NAN);
    let (lo, hi) = if v < 0.0 { (1.01 * v, 0.99 * v) } else { (0.99 * v, 1.01 * v) };
    lo <= r && r <= hi
}

fn value_matches(
    expected: Option<&str>,
    actual: Option<&str>,
    tol: Tolerance,
    column: &str,
) -> bool {
    let (Some(e), Some(a)) = (expected, actual) else {
        return expected.is_none() && actual.is_none();
    };
    if let (Some(x), Some(y)) = (number(e), number(a)) {
        #[allow(clippy::float_cmp)]
        if x == y {
            return true;
        }
        return match tol {
            Tolerance::Relative(r) => (x - y).abs() <= r * x.abs().max(y.abs()),
            Tolerance::Tpch => match tpch_kind(column) {
                TpchKind::Exact => round(y, decimals(e)) == round(x, decimals(e)),
                TpchKind::Sum => (x - y).abs() <= 100.0,
                TpchKind::Avg => within_one_percent(x, y),
                TpchKind::SumRatio => (x - y).abs() <= 100.0 && within_one_percent(x, y),
            },
        };
    }
    e.trim_end_matches(' ') == a.trim_end_matches(' ')
}

/// The order of the sort: NULL first, then numbers by value, then text.
fn cmp_value(a: Option<&str>, b: Option<&str>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => match (number(x), number(y)) {
            (Some(p), Some(q)) => p.total_cmp(&q),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => x.trim_end_matches(' ').cmp(y.trim_end_matches(' ')),
        },
    }
}

fn sorted(rows: &[Vec<Option<String>>]) -> Vec<&Vec<Option<String>>> {
    let mut out: Vec<&Vec<Option<String>>> = rows.iter().collect();
    out.sort_by(|a, b| {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| cmp_value(x.as_deref(), y.as_deref()))
            .find(|o| o.is_ne())
            .unwrap_or(Ordering::Equal)
    });
    out
}

/// Compares two answers as multisets of rows. The names of the columns are not compared, because each system names an expression in its own way. The error names the first difference.
pub(crate) fn compare(expected: &Answer, actual: &Answer, tol: Tolerance) -> Result<(), String> {
    if expected.columns.len() != actual.columns.len() {
        return Err(format!(
            "{} columns, expected {}",
            actual.columns.len(),
            expected.columns.len()
        ));
    }
    if expected.rows.len() != actual.rows.len() {
        return Err(format!("{} rows, expected {}", actual.rows.len(), expected.rows.len()));
    }
    for (i, (e, a)) in sorted(&expected.rows).into_iter().zip(sorted(&actual.rows)).enumerate() {
        for (c, (x, y)) in e.iter().zip(a.iter()).enumerate() {
            if !value_matches(x.as_deref(), y.as_deref(), tol, &expected.columns[c]) {
                let mut msg =
                    format!("sorted row {} column {} ({}): ", i + 1, c + 1, expected.columns[c]);
                let _ = write!(msg, "got {}, expected {}", show(y.as_deref()), show(x.as_deref()));
                return Err(msg);
            }
        }
    }
    Ok(())
}

/// What a query fixes of its result. A query with `LIMIT` fixes its rows only up to the ties of its sort key, so two correct systems can return other rows (spec/20 section 20.5 compares with PostgreSQL 19).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Determined {
    /// No `LIMIT`: the whole multiset of rows.
    All,
    /// `ORDER BY` on output columns (by position) with `LIMIT`. The keys of all rows are fixed. A row whose key ties with the last row, or with the first row when there is an `OFFSET`, can be any row with that key. Every other row is fixed.
    UpToTies { keys: Vec<usize>, offset: bool },
    /// `LIMIT` with no `ORDER BY`, or with a sort key that is not an output column: only the shape is fixed.
    RowCount,
}

impl Determined {
    /// Reads the `SELECT` list, the `ORDER BY` list, `LIMIT` and `OFFSET` at the top level of one query.
    pub(crate) fn of_query(sql: &str) -> Determined {
        let text: String =
            sql.trim().trim_end_matches(';').split_whitespace().collect::<Vec<_>>().join(" ");
        let q = Masked::new(&text);
        let Some(limit) = q.last("LIMIT") else { return Determined::All };
        let offset = q.last("OFFSET").is_some();
        let Some(order) = q.last("ORDER BY") else { return Determined::RowCount };
        let (Some(select), Some(from)) = (q.first("SELECT"), q.first("FROM")) else {
            return Determined::RowCount;
        };
        let items: Vec<(String, Option<String>)> = q
            .split(select + "SELECT".len(), from)
            .iter()
            .map(|item| {
                let m = Masked::new(item);
                match m.last("AS") {
                    Some(at) => (
                        norm(&item[..at]),
                        Some(norm(&item[at + 2..]).trim_matches('"').to_owned()),
                    ),
                    None => (norm(item), None),
                }
            })
            .collect();
        let mut keys = Vec::new();
        for key in q.split(order + "ORDER BY".len(), limit) {
            let mut k = norm(&key);
            for suffix in ["nullsfirst", "nullslast", "desc", "asc"] {
                if let Some(rest) = k.strip_suffix(suffix) {
                    k = rest.to_owned();
                }
            }
            let at = match k.parse::<usize>() {
                Ok(n) if n >= 1 && n <= items.len() => Some(n - 1),
                _ => items
                    .iter()
                    .position(|(expr, alias)| *expr == k || alias.as_deref() == Some(&k)),
            };
            match at {
                Some(i) => keys.push(i),
                None => return Determined::RowCount,
            }
        }
        if keys.is_empty() { Determined::RowCount } else { Determined::UpToTies { keys, offset } }
    }

    /// The rule in words, for the result.
    pub(crate) fn rule(&self) -> String {
        match self {
            Determined::All => "all rows".to_owned(),
            Determined::UpToTies { keys, offset } => format!(
                "all sort keys, and the rows that do not tie with the {} row (sort key columns {})",
                if *offset { "first or the last" } else { "last" },
                keys.iter().map(|k| (k + 1).to_string()).collect::<Vec<_>>().join(", ")
            ),
            Determined::RowCount => {
                "the number of rows only: LIMIT with no sort key in the output fixes no row"
                    .to_owned()
            }
        }
    }
}

/// Lower case with no spaces, so `COUNT(*) AS c` and `count(*) as c` match.
fn norm(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase()
}

/// A query with the text of its literals and quoted names hidden, and the depth of the parentheses at each byte, to find keywords at the top level.
struct Masked {
    text: String,
    upper: Vec<u8>,
    depth: Vec<i32>,
}

impl Masked {
    fn new(text: &str) -> Masked {
        let mut upper = Vec::with_capacity(text.len());
        let mut depth = Vec::with_capacity(text.len());
        let (mut quote, mut d) = (None, 0);
        for b in text.bytes() {
            match quote {
                Some(q) => {
                    if b == q {
                        quote = None;
                    }
                    upper.push(b'_');
                }
                None => {
                    match b {
                        b'\'' | b'"' => quote = Some(b),
                        b'(' => d += 1,
                        b')' => d -= 1,
                        _ => {}
                    }
                    upper.push(b.to_ascii_uppercase());
                }
            }
            depth.push(d);
        }
        Masked { text: text.to_owned(), upper, depth }
    }

    /// The byte positions of a keyword at the top level, with no letter, digit or `_` next to it.
    fn find(&self, word: &str) -> Vec<usize> {
        let w = word.as_bytes();
        let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        (0..self.upper.len().saturating_sub(w.len() - 1))
            .filter(|&i| {
                self.upper[i..i + w.len()] == *w
                    && self.depth[i] == 0
                    && (i == 0 || !ident(self.upper[i - 1]))
                    && self.upper.get(i + w.len()).is_none_or(|&b| !ident(b))
            })
            .collect()
    }

    fn first(&self, word: &str) -> Option<usize> {
        self.find(word).first().copied()
    }

    fn last(&self, word: &str) -> Option<usize> {
        self.find(word).last().copied()
    }

    /// The text between two positions, split at the commas of the top level.
    fn split(&self, from: usize, to: usize) -> Vec<String> {
        let mut out = Vec::new();
        let mut start = from;
        for i in from..to {
            if self.upper[i] == b',' && self.depth[i] == 0 {
                out.push(self.text[start..i].trim().to_owned());
                start = i + 1;
            }
        }
        out.push(self.text[start..to].trim().to_owned());
        out.retain(|s| !s.is_empty());
        out
    }
}

/// Compares two answers with the rule of `Determined`. For `All` it is `compare`.
pub(crate) fn compare_query(
    expected: &Answer,
    actual: &Answer,
    tol: Tolerance,
    rule: &Determined,
) -> Result<(), String> {
    let keys = match rule {
        Determined::All => return compare(expected, actual, tol),
        Determined::RowCount | Determined::UpToTies { .. } => {
            if expected.columns.len() != actual.columns.len() {
                return Err(format!(
                    "{} columns, expected {}",
                    actual.columns.len(),
                    expected.columns.len()
                ));
            }
            if expected.rows.len() != actual.rows.len() {
                return Err(format!(
                    "{} rows, expected {}",
                    actual.rows.len(),
                    expected.rows.len()
                ));
            }
            match rule {
                Determined::UpToTies { keys, .. } => keys,
                _ => return Ok(()),
            }
        }
    };
    let Determined::UpToTies { offset, .. } = rule else { unreachable!() };
    if expected.rows.is_empty() {
        return Ok(());
    }
    if let Some(&k) = keys.iter().find(|&&k| k >= expected.columns.len()) {
        return Err(format!("sort key column {} is not in the result", k + 1));
    }
    let key_answer = |a: &Answer| Answer {
        columns: keys.iter().map(|&k| expected.columns[k].clone()).collect(),
        rows: a.rows.iter().map(|r| keys.iter().map(|&k| r[k].clone()).collect()).collect(),
    };
    compare(&key_answer(expected), &key_answer(actual), tol)
        .map_err(|e| format!("the sort keys differ: {e}"))?;
    // The keys that can tie at the edge of the window, from the rows in the order that each system returned.
    let mut edges: Vec<Vec<Option<String>>> = Vec::new();
    for a in [expected, actual] {
        let key = |r: &Vec<Option<String>>| keys.iter().map(|&k| r[k].clone()).collect::<Vec<_>>();
        edges.extend(a.rows.last().map(key));
        if *offset {
            edges.extend(a.rows.first().map(key));
        }
    }
    let ties = |r: &Vec<Option<String>>| {
        edges.iter().any(|e| {
            keys.iter().zip(e).all(|(&k, v)| {
                value_matches(v.as_deref(), r[k].as_deref(), tol, &expected.columns[k])
            })
        })
    };
    let fixed = |a: &Answer| Answer {
        columns: a.columns.clone(),
        rows: a.rows.iter().filter(|r| !ties(r)).cloned().collect(),
    };
    compare(&fixed(expected), &fixed(actual), tol)
        .map_err(|e| format!("the rows that do not tie differ: {e}"))
}

fn show(v: Option<&str>) -> String {
    v.map_or_else(|| "NULL".to_owned(), |s| format!("{:?}", s.trim_end_matches(' ')))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(columns: &[&str], rows: &[&[Option<&str>]]) -> Answer {
        Answer {
            columns: columns.iter().map(|c| (*c).to_owned()).collect(),
            rows: rows.iter().map(|r| r.iter().map(|v| v.map(str::to_owned)).collect()).collect(),
        }
    }

    #[test]
    fn what_a_query_determines() {
        assert_eq!(Determined::of_query("SELECT COUNT(*) FROM hits;"), Determined::All);
        assert_eq!(
            Determined::of_query(
                "SELECT UserID, SearchPhrase, COUNT(*) FROM hits GROUP BY UserID, SearchPhrase LIMIT 10;"
            ),
            Determined::RowCount
        );
        assert_eq!(
            Determined::of_query(
                "SELECT SearchPhrase FROM hits WHERE SearchPhrase <> '' ORDER BY EventTime LIMIT 10;"
            ),
            Determined::RowCount
        );
        assert_eq!(
            Determined::of_query(
                "SELECT UserID, extract(minute FROM EventTime) AS m, SearchPhrase, COUNT(*) FROM hits GROUP BY UserID, m, SearchPhrase ORDER BY COUNT(*) DESC LIMIT 10;"
            ),
            Determined::UpToTies { keys: vec![3], offset: false }
        );
        assert_eq!(
            Determined::of_query(
                "SELECT REGEXP_REPLACE(Referer, '^https?://(?:www\\.)?([^/]+)/.*$', '\\1') AS k, AVG(length(Referer)) AS l, COUNT(*) AS c, MIN(Referer) FROM hits WHERE Referer <> '' GROUP BY k HAVING COUNT(*) > 100000 ORDER BY l DESC LIMIT 25;"
            ),
            Determined::UpToTies { keys: vec![1], offset: false }
        );
        assert_eq!(
            Determined::of_query(
                "SELECT URL, COUNT(*) AS PageViews FROM hits WHERE CounterID = 62 GROUP BY URL ORDER BY PageViews DESC LIMIT 10 OFFSET 1000;"
            ),
            Determined::UpToTies { keys: vec![1], offset: true }
        );
        assert_eq!(
            Determined::of_query(
                "SELECT SearchPhrase, EventTime FROM hits ORDER BY 2, SearchPhrase LIMIT 10"
            ),
            Determined::UpToTies { keys: vec![1, 0], offset: false }
        );
        // A keyword in a literal or in parentheses is not a keyword of the query.
        assert_eq!(Determined::of_query("SELECT 'LIMIT 1' AS x FROM t"), Determined::All);
        assert_eq!(
            Determined::of_query("SELECT (SELECT a FROM t LIMIT 1) FROM u"),
            Determined::All
        );
    }

    #[test]
    fn ties_at_the_limit() {
        let rule = Determined::UpToTies { keys: vec![1], offset: false };
        let e = answer(
            &["u", "c"],
            &[&[Some("a"), Some("9")], &[Some("b"), Some("5")], &[Some("c"), Some("5")]],
        );
        // d ties with the last row, so it can take the place of c.
        let tie = answer(
            &["u", "c"],
            &[&[Some("a"), Some("9")], &[Some("d"), Some("5")], &[Some("b"), Some("5")]],
        );
        assert_eq!(compare_query(&e, &tie, RELATIVE, &rule), Ok(()));
        assert!(compare(&e, &tie, RELATIVE).is_err());
        // A row above the tie is fixed.
        let top = answer(
            &["u", "c"],
            &[&[Some("x"), Some("9")], &[Some("b"), Some("5")], &[Some("c"), Some("5")]],
        );
        assert!(compare_query(&e, &top, RELATIVE, &rule).unwrap_err().contains("do not tie"));
        // The keys are fixed.
        let key = answer(
            &["u", "c"],
            &[&[Some("a"), Some("9")], &[Some("b"), Some("5")], &[Some("c"), Some("4")]],
        );
        assert!(compare_query(&e, &key, RELATIVE, &rule).unwrap_err().contains("sort keys"));
        // With an OFFSET the first row can tie too.
        let off = Determined::UpToTies { keys: vec![1], offset: true };
        let first = answer(
            &["u", "c"],
            &[&[Some("z"), Some("9")], &[Some("b"), Some("5")], &[Some("c"), Some("5")]],
        );
        assert!(compare_query(&e, &first, RELATIVE, &rule).is_err());
        assert_eq!(compare_query(&e, &first, RELATIVE, &off), Ok(()));
        // Only the shape for RowCount.
        let other = answer(
            &["u", "c"],
            &[&[Some("p"), Some("1")], &[Some("q"), Some("1")], &[Some("r"), Some("1")]],
        );
        assert_eq!(compare_query(&e, &other, RELATIVE, &Determined::RowCount), Ok(()));
        assert!(
            compare_query(&e, &answer(&["u", "c"], &[]), RELATIVE, &Determined::RowCount).is_err()
        );
    }

    #[test]
    fn tsv_files() {
        let a = answer(&["k", "v"], &[&[Some("a\tb\\c\nd"), None], &[Some(""), Some("\\N")]]);
        assert_eq!(parse_tsv("k\tv\na\\tb\\\\c\\nd\t\\N\n\t\\\\N\n").unwrap(), a);
        assert!(parse_tsv("a\tb\n1\n").unwrap_err().contains("line 2 has 1 fields"));
        assert!(parse_tsv("a\n\\q\n").is_err());
    }

    #[test]
    fn written_tsv_reads_back() {
        let a = answer(&["k", "v"], &[&[Some("a\tb\\c\nd\r"), None], &[Some(""), Some("\\N")]]);
        assert_eq!(to_tsv(&a), "k\tv\na\\tb\\\\c\\nd\\r\t\\N\n\t\\\\N\n");
        assert_eq!(parse_tsv(&to_tsv(&a)).unwrap(), a);
    }

    #[test]
    fn csv_files() {
        let a = answer(&["k", "v w"], &[&[Some("a,\"b\"\nc"), None], &[Some(""), Some("1.5")]]);
        assert_eq!(parse_csv("k,\"v w\"\n\"a,\"\"b\"\"\nc\",\n\"\",1.5\n").unwrap(), a);
        assert_eq!(parse_csv("k,\"v w\"\r\n\"a,\"\"b\"\"\nc\",\r\n\"\",1.5").unwrap(), a);
        assert!(parse_csv("a,b\n1\n").unwrap_err().contains("record 2 has 1 fields"));
        assert!(parse_csv("a\n\"x\n").is_err());
        assert_eq!(parse_csv("a\n").unwrap().rows.len(), 0);
    }

    #[test]
    fn tpch_answer_files() {
        let spaced = parse_tpch("c_name    |c_address\nCustomer#1|  W556MX    \n").unwrap();
        assert_eq!(spaced.rows, [[Some("Customer#1".to_owned()), Some("  W556MX".to_owned())]]);
        let text = "c_name                   |c_custkey           |col6                                     \nCustomer#000128120       |              128120|323.00\n";
        let a = parse_tpch(text).unwrap();
        assert_eq!(a.columns, ["c_name", "c_custkey", "col6"]);
        assert_eq!(
            a.rows,
            [[
                Some("Customer#000128120".to_owned()),
                Some("128120".to_owned()),
                Some("323.00".to_owned())
            ]]
        );
        let ours = answer(
            &["c_name", "c_custkey", "sum"],
            &[&[Some("Customer#000128120       "), Some("128120"), Some("323")]],
        );
        assert_eq!(compare(&a, &ours, Tolerance::Tpch), Ok(()));
    }

    #[test]
    fn numbers_and_tolerances() {
        let rel = RELATIVE;
        let m = |e, a, tol, c| value_matches(e, a, tol, c);
        assert!(m(Some("1e10"), Some("10000000000.000001"), rel, "x"));
        assert!(!m(Some("1"), Some("1.00001"), rel, "x"));
        assert!(m(Some("NaN"), Some("NaN"), rel, "x"));
        assert!(!m(Some("NaN"), Some("nan"), rel, "x"));
        assert!(!m(Some("1994-04-07"), Some("1994-04-08"), rel, "x"));
        assert!(m(None, None, rel, "x"));
        assert!(!m(None, Some(""), rel, "x"));
        let t = Tolerance::Tpch;
        // Column values and counts.
        assert!(m(Some("9938.53"), Some("9938.53"), t, "s_acctbal"));
        assert!(m(Some("323.00"), Some("323"), t, "o_totalprice"));
        assert!(!m(Some("9938.53"), Some("9938.54"), t, "s_acctbal"));
        assert!(!m(Some("50005"), Some("50004"), t, "custdist"));
        assert!(m(Some("0"), Some("-0.0001"), t, "x"));
        // Sums within 100.
        assert!(m(Some("123141078.23"), Some("123141078.2283"), t, "revenue"));
        assert!(m(Some("53758257134.87"), Some("53758257100.00"), t, "sum_disc_price"));
        assert!(!m(Some("53758257134.87"), Some("53758257034.00"), t, "sum_disc_price"));
        // Averages and ratios within 1 percent after rounding to 2 decimals.
        assert!(m(Some("348406.02"), Some("348406.0542857143"), t, "avg_yearly"));
        assert!(m(Some("0.05"), Some("0.049985295838397614"), t, "avg_disc"));
        assert!(m(Some("16.38"), Some("16.3807"), t, "promo_revenue"));
        assert!(!m(Some("16.38"), Some("16.60"), t, "promo_revenue"));
        assert!(!m(Some("0.05"), Some("0.0449"), t, "avg_disc"));
        assert!(!m(Some("37734107.00"), Some("37734107.5"), t, "sum_qty"));
        assert!(!m(Some("348406.02"), Some("348606.02"), t, "avg_yearly"));
    }

    #[test]
    fn tpch_kinds() {
        assert_eq!(tpch_kind("sum_charge"), TpchKind::Sum);
        assert_eq!(tpch_kind("col6"), TpchKind::Exact);
        assert_eq!(tpch_kind("sum_qty"), TpchKind::Exact);
        assert_eq!(tpch_kind("avg_price"), TpchKind::Avg);
        assert_eq!(tpch_kind("mkt_share"), TpchKind::SumRatio);
        assert_eq!(tpch_kind("count_order"), TpchKind::Exact);
        assert_eq!(tpch_kind("l_returnflag"), TpchKind::Exact);
    }

    #[test]
    fn multisets() {
        let e = answer(
            &["a", "b"],
            &[&[Some("x"), Some("2")], &[Some("x"), Some("10")], &[None, Some("1")]],
        );
        let a = answer(
            &["a", "count"],
            &[&[Some("x"), Some("10")], &[None, Some("1")], &[Some("x"), Some("2")]],
        );
        assert_eq!(compare(&e, &a, RELATIVE), Ok(()));
        let b = answer(
            &["a", "b"],
            &[&[Some("x"), Some("10")], &[None, Some("1")], &[Some("x"), Some("3")]],
        );
        assert_eq!(
            compare(&e, &b, RELATIVE).unwrap_err(),
            "sorted row 2 column 2 (b): got \"3\", expected \"2\""
        );
        let c = answer(&["a", "b"], &[&[None, Some("1")]]);
        assert_eq!(compare(&e, &c, RELATIVE).unwrap_err(), "1 rows, expected 3");
    }
}
