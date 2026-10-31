//! The `compare` command: two ClickBench result files side by side.
//!
//! A query time is the ClickBench hot time, the best of the tries after the first, or the first try when it is the only one.
//! The speedup of a query is `(base + 0.01) / (new + 0.01)`, with the 10 ms that ClickBench adds so that a fast query does not
//! rule the score. The summary is the geometric mean of the speedups over the queries that have a time in both files.

use crate::json::Json;

/// The 10 ms of the ClickBench score.
const SHIFT: f64 = 0.01;

/// One line of the comparison.
#[derive(Debug, PartialEq)]
pub(crate) struct Line {
    pub(crate) query: usize,
    pub(crate) base: Option<f64>,
    pub(crate) new: Option<f64>,
}

impl Line {
    pub(crate) fn speedup(&self) -> Option<f64> {
        Some((self.base? + SHIFT) / (self.new? + SHIFT))
    }
}

fn number(v: &Json) -> Option<f64> {
    match v {
        #[allow(clippy::cast_precision_loss)]
        Json::Int(i) => Some(*i as f64),
        #[allow(clippy::cast_precision_loss)]
        Json::UInt(u) => Some(*u as f64),
        Json::Num(n) => Some(*n),
        _ => None,
    }
}

/// The hot time of each query of a result file.
pub(crate) fn hot_times(file: &Json) -> Result<Vec<Option<f64>>, String> {
    let Some(Json::Arr(result)) = file.get("result") else {
        return Err("the file has no result array".to_owned());
    };
    result
        .iter()
        .map(|tries| {
            let Json::Arr(tries) = tries else {
                return Err("a query of result is not an array".to_owned());
            };
            let times: Vec<f64> = tries.iter().filter_map(number).collect();
            let hot = if tries.len() > 1 {
                tries[1..].iter().filter_map(number).reduce(f64::min)
            } else {
                None
            };
            Ok(hot.or_else(|| times.first().copied()))
        })
        .collect()
}

/// The lines of two result files. The files must have the same number of queries.
pub(crate) fn lines(base: &Json, new: &Json) -> Result<Vec<Line>, String> {
    let (b, n) = (hot_times(base)?, hot_times(new)?);
    if b.len() != n.len() {
        return Err(format!("the base has {} queries and the new file has {}", b.len(), n.len()));
    }
    Ok(b.into_iter()
        .zip(n)
        .enumerate()
        .map(|(i, (base, new))| Line { query: i + 1, base, new })
        .collect())
}

/// The geometric mean of the speedups, and the number of queries in it.
pub(crate) fn geomean(lines: &[Line]) -> Option<(f64, usize)> {
    let speedups: Vec<f64> = lines.iter().filter_map(Line::speedup).collect();
    if speedups.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let mean = speedups.iter().map(|s| s.ln()).sum::<f64>() / speedups.len() as f64;
    Some((mean.exp(), speedups.len()))
}

fn cell(v: Option<f64>) -> String {
    v.map_or_else(|| "null".to_owned(), |v| format!("{v:.3}"))
}

/// The text table and the summary.
pub(crate) fn table(base: &Json, new: &Json, lines: &[Line]) -> String {
    let name = |f: &Json| f.get("system").and_then(Json::as_str).unwrap_or("?").to_owned();
    let mut out = format!("query\t{}\t{}\tspeedup\n", name(base), name(new));
    for l in lines {
        let speedup = l.speedup().map_or_else(|| "-".to_owned(), |s| format!("{s:.2}x"));
        out.push_str(&format!("Q{}\t{}\t{}\t{speedup}\n", l.query - 1, cell(l.base), cell(l.new)));
    }
    for key in ["load_time", "data_size", "concurrent_qps"] {
        let (b, n) = (base.get(key).and_then(number), new.get(key).and_then(number));
        if b.is_some() || n.is_some() {
            let ratio = match (b, n) {
                (Some(b), Some(n)) if n > 0.0 => format!("{:.2}", b / n),
                _ => "-".to_owned(),
            };
            out.push_str(&format!("{key}\t{}\t{}\t{ratio}\n", cell(b), cell(n)));
        }
    }
    match geomean(lines) {
        Some((g, n)) => out.push_str(&format!("geomean speedup over {n} queries: {g:.2}x\n")),
        None => out.push_str("no query has a time in both files\n"),
    }
    out
}

/// The `--json` form.
pub(crate) fn json(lines: &[Line]) -> Json {
    let (g, n) = geomean(lines).map_or((None, 0u64), |(g, n)| (Some(g), n as u64));
    Json::obj()
        .with("geomean_speedup", g)
        .with("queries", n)
        .with("speedup", Json::Arr(lines.iter().map(|l| Json::from(l.speedup())).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(system: &str, result: &str) -> Json {
        Json::parse(&format!(r#"{{"system": "{system}", "load_time": 10, "result": {result}}}"#))
            .unwrap()
    }

    #[test]
    fn hot_time_is_the_best_try_after_the_first() {
        let f = file(
            "a",
            "[[9.0, 2.0, 1.0], [null, null, 3], [0.5], [null, 4.0, null], [7.0, null, null]]",
        );
        assert_eq!(
            hot_times(&f).unwrap(),
            vec![Some(1.0), Some(3.0), Some(0.5), Some(4.0), Some(7.0)]
        );
    }

    #[test]
    fn speedups_and_geomean() {
        let base = file("pg", "[[1, 0.99, 0.99], [1, 0.09, 0.09], [1, null, null]]");
        let new = file("rupg", "[[1, 0.09, 0.09], [1, 0.99, 0.99], [null, null, null]]");
        let l = lines(&base, &new).unwrap();
        assert_eq!(l[0].speedup(), Some(10.0));
        assert!((l[1].speedup().unwrap() - 0.1).abs() < 1e-12);
        assert_eq!(l[2].speedup(), None);
        let (g, n) = geomean(&l).unwrap();
        assert!((g - 1.0).abs() < 1e-12);
        assert_eq!(n, 2);
        let t = table(&base, &new, &l);
        assert!(t.starts_with("query\tpg\trupg\tspeedup\nQ0\t0.990\t0.090\t10.00x\n"), "{t}");
        assert!(t.contains("load_time\t10.000\t10.000\t1.00\n"), "{t}");
        assert!(t.ends_with("geomean speedup over 2 queries: 1.00x\n"), "{t}");
    }

    #[test]
    fn query_counts_must_match() {
        assert!(lines(&file("a", "[[1]]"), &file("b", "[[1], [2]]")).is_err());
    }
}
