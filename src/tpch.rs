//! The TPC-H driver of spec/20 section 20.7: the data from `dbgen` 3.0.1, the 22 queries from `qgen -d`, the schema with the keys of the specification, and the load into DuckDB or a PostgreSQL protocol server.

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::pg::Conn;
use crate::suite::Query;

/// One table of clause 1.4. The column types follow clause 1.4.1: an identifier is `INTEGER`, which holds every key up to SF 300, a decimal is `DECIMAL(15,2)`, fixed text is `CHAR(n)` and variable text is `VARCHAR(n)`.
pub(crate) struct Table {
    pub(crate) name: &'static str,
    pub(crate) columns: &'static [(&'static str, &'static str)],
    /// The primary key of clause 1.4.2.2.
    pub(crate) key: &'static [&'static str],
    /// The foreign keys of clause 1.4.2.3: the columns, the table and its columns.
    pub(crate) references:
        &'static [(&'static [&'static str], &'static str, &'static [&'static str])],
}

const I: &str = "INTEGER NOT NULL";
const D: &str = "DECIMAL(15,2) NOT NULL";
const DATE: &str = "DATE NOT NULL";

/// The eight tables, a table after the tables it references.
pub(crate) const TABLES: [Table; 8] = [
    Table {
        name: "region",
        columns: &[
            ("r_regionkey", I),
            ("r_name", "CHAR(25) NOT NULL"),
            ("r_comment", "VARCHAR(152)"),
        ],
        key: &["r_regionkey"],
        references: &[],
    },
    Table {
        name: "nation",
        columns: &[
            ("n_nationkey", I),
            ("n_name", "CHAR(25) NOT NULL"),
            ("n_regionkey", I),
            ("n_comment", "VARCHAR(152)"),
        ],
        key: &["n_nationkey"],
        references: &[(&["n_regionkey"], "region", &["r_regionkey"])],
    },
    Table {
        name: "part",
        columns: &[
            ("p_partkey", I),
            ("p_name", "VARCHAR(55) NOT NULL"),
            ("p_mfgr", "CHAR(25) NOT NULL"),
            ("p_brand", "CHAR(10) NOT NULL"),
            ("p_type", "VARCHAR(25) NOT NULL"),
            ("p_size", I),
            ("p_container", "CHAR(10) NOT NULL"),
            ("p_retailprice", D),
            ("p_comment", "VARCHAR(23) NOT NULL"),
        ],
        key: &["p_partkey"],
        references: &[],
    },
    Table {
        name: "supplier",
        columns: &[
            ("s_suppkey", I),
            ("s_name", "CHAR(25) NOT NULL"),
            ("s_address", "VARCHAR(40) NOT NULL"),
            ("s_nationkey", I),
            ("s_phone", "CHAR(15) NOT NULL"),
            ("s_acctbal", D),
            ("s_comment", "VARCHAR(101) NOT NULL"),
        ],
        key: &["s_suppkey"],
        references: &[(&["s_nationkey"], "nation", &["n_nationkey"])],
    },
    Table {
        name: "partsupp",
        columns: &[
            ("ps_partkey", I),
            ("ps_suppkey", I),
            ("ps_availqty", I),
            ("ps_supplycost", D),
            ("ps_comment", "VARCHAR(199) NOT NULL"),
        ],
        key: &["ps_partkey", "ps_suppkey"],
        references: &[
            (&["ps_partkey"], "part", &["p_partkey"]),
            (&["ps_suppkey"], "supplier", &["s_suppkey"]),
        ],
    },
    Table {
        name: "customer",
        columns: &[
            ("c_custkey", I),
            ("c_name", "VARCHAR(25) NOT NULL"),
            ("c_address", "VARCHAR(40) NOT NULL"),
            ("c_nationkey", I),
            ("c_phone", "CHAR(15) NOT NULL"),
            ("c_acctbal", D),
            ("c_mktsegment", "CHAR(10) NOT NULL"),
            ("c_comment", "VARCHAR(117) NOT NULL"),
        ],
        key: &["c_custkey"],
        references: &[(&["c_nationkey"], "nation", &["n_nationkey"])],
    },
    Table {
        name: "orders",
        columns: &[
            ("o_orderkey", I),
            ("o_custkey", I),
            ("o_orderstatus", "CHAR(1) NOT NULL"),
            ("o_totalprice", D),
            ("o_orderdate", DATE),
            ("o_orderpriority", "CHAR(15) NOT NULL"),
            ("o_clerk", "CHAR(15) NOT NULL"),
            ("o_shippriority", I),
            ("o_comment", "VARCHAR(79) NOT NULL"),
        ],
        key: &["o_orderkey"],
        references: &[(&["o_custkey"], "customer", &["c_custkey"])],
    },
    Table {
        name: "lineitem",
        columns: &[
            ("l_orderkey", I),
            ("l_partkey", I),
            ("l_suppkey", I),
            ("l_linenumber", I),
            ("l_quantity", D),
            ("l_extendedprice", D),
            ("l_discount", D),
            ("l_tax", D),
            ("l_returnflag", "CHAR(1) NOT NULL"),
            ("l_linestatus", "CHAR(1) NOT NULL"),
            ("l_shipdate", DATE),
            ("l_commitdate", DATE),
            ("l_receiptdate", DATE),
            ("l_shipinstruct", "CHAR(25) NOT NULL"),
            ("l_shipmode", "CHAR(10) NOT NULL"),
            ("l_comment", "VARCHAR(44) NOT NULL"),
        ],
        key: &["l_orderkey", "l_linenumber"],
        references: &[
            (&["l_orderkey"], "orders", &["o_orderkey"]),
            (&["l_partkey"], "part", &["p_partkey"]),
            (&["l_suppkey"], "supplier", &["s_suppkey"]),
            (&["l_partkey", "l_suppkey"], "partsupp", &["ps_partkey", "ps_suppkey"]),
        ],
    },
];

impl Table {
    /// `CREATE TABLE` with the columns only, or with the keys too.
    pub(crate) fn create(&self, with_keys: bool) -> String {
        let mut parts: Vec<String> = self.columns.iter().map(|(c, t)| format!("{c} {t}")).collect();
        if with_keys {
            parts.push(format!("PRIMARY KEY ({})", self.key.join(", ")));
            for (cols, table, to) in self.references {
                parts.push(format!(
                    "FOREIGN KEY ({}) REFERENCES {table} ({})",
                    cols.join(", "),
                    to.join(", ")
                ));
            }
        }
        format!("CREATE TABLE {} ({})", self.name, parts.join(", "))
    }

    /// The `ALTER TABLE` statements that add the keys after the load, the primary key first.
    pub(crate) fn add_keys(&self) -> Vec<String> {
        let mut out =
            vec![format!("ALTER TABLE {} ADD PRIMARY KEY ({})", self.name, self.key.join(", "))];
        for (cols, table, to) in self.references {
            out.push(format!(
                "ALTER TABLE {} ADD FOREIGN KEY ({}) REFERENCES {table} ({})",
                self.name,
                cols.join(", "),
                to.join(", ")
            ));
        }
        out
    }
}

/// `DROP TABLE` for the eight tables, the referencing tables first.
pub(crate) fn drop_all() -> String {
    let names: Vec<&str> = TABLES.iter().rev().map(|t| t.name).collect();
    format!("DROP TABLE IF EXISTS {}", names.join(", "))
}

/// The `dbgen` directory of the TPC-H tools, with `dbgen`, `qgen`, `dists.dss` and the query templates.
#[derive(Clone, Debug)]
pub(crate) struct Tools {
    pub(crate) dbgen: PathBuf,
}

impl Tools {
    pub(crate) fn new(dir: &Path) -> Result<Tools, String> {
        for f in ["dbgen", "qgen", "dists.dss"] {
            if !dir.join(f).is_file() {
                return Err(format!(
                    "{} has no {f}. Run machines/install/tpch-tools.sh",
                    dir.display()
                ));
            }
        }
        Ok(Tools { dbgen: dir.to_owned() })
    }

    /// The template of query `n`. `queries/<n>.sql` is the template of the kit. The copy of the pin has no `queries/6.sql`, so Q6 comes from `6.sql` beside it, which has the text of the kit. Q15 is the approved variant A of `variants/15a.sql`, because the base form makes a view and drops it, which is three statements and not one query.
    pub(crate) fn template(&self, n: u32) -> PathBuf {
        let kit = self.dbgen.join("queries").join(format!("{n}.sql"));
        match n {
            15 => self.dbgen.join("variants").join("15a.sql"),
            _ if kit.is_file() => kit,
            _ => self.dbgen.join(format!("{n}.sql")),
        }
    }

    /// Runs `dbgen -f -s <scale>` into `out`. A file `SCALE` in `out` with the same scale means the data is there, and nothing runs.
    pub(crate) fn generate(&self, scale: &str, out: &Path) -> Result<bool, String> {
        let mark = out.join("SCALE");
        if fs::read_to_string(&mark).is_ok_and(|s| s.trim() == scale) {
            return Ok(false);
        }
        fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
        let _ = fs::remove_file(&mark);
        let status = Command::new(self.dbgen.join("dbgen"))
            .args(["-f", "-s", scale])
            .env("DSS_CONFIG", &self.dbgen)
            .env("DSS_PATH", out)
            .current_dir(out)
            .status()
            .map_err(|e| format!("dbgen: {e}"))?;
        if !status.success() {
            return Err(format!("dbgen failed: {status}"));
        }
        for t in &TABLES {
            if !out.join(format!("{}.tbl", t.name)).is_file() {
                return Err(format!("dbgen made no {}.tbl", t.name));
            }
        }
        fs::write(&mark, format!("{scale}\n")).map_err(|e| format!("{}: {e}", mark.display()))?;
        Ok(true)
    }

    /// The 22 queries from `qgen -d -s <scale>`, after `fix_query`. The templates are copied into `work` first, because qgen reads all of them from one directory.
    pub(crate) fn queries(&self, scale: &str, work: &Path) -> Result<Vec<Query>, String> {
        let templates = work.join("templates");
        fs::create_dir_all(&templates).map_err(|e| format!("{}: {e}", templates.display()))?;
        let mut out = Vec::new();
        for n in 1..=22 {
            let from = self.template(n);
            fs::copy(&from, templates.join(format!("{n}.sql")))
                .map_err(|e| format!("{}: {e}", from.display()))?;
        }
        for n in 1..=22 {
            let o = Command::new(self.dbgen.join("qgen"))
                .args(["-d", "-s", scale, &n.to_string()])
                .env("DSS_CONFIG", &self.dbgen)
                .env("DSS_QUERY", &templates)
                .current_dir(&self.dbgen)
                .output()
                .map_err(|e| format!("qgen: {e}"))?;
            if !o.status.success() {
                return Err(format!(
                    "qgen {n} failed: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                ));
            }
            let sql = fix_query(&String::from_utf8_lossy(&o.stdout))?;
            out.push(Query { name: format!("q{n}"), sql });
        }
        Ok(out)
    }
}

/// The changes to the text of qgen, and nothing else:
///
/// 1. qgen prints the row count of a query as the line `--LIMIT n` after the query, because the kit has no SQL form for it. The query gets `LIMIT n` before its `;`.
/// 2. Q1 has `interval '90' day (3)`. PostgreSQL and DuckDB do not accept a precision there, so it becomes `interval '90' day`. The value is the same.
/// 3. The comment lines are removed.
pub(crate) fn fix_query(text: &str) -> Result<String, String> {
    let mut limit = None;
    let mut lines = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(n) = t.strip_prefix("--LIMIT ") {
            limit =
                Some(n.trim().parse::<u64>().map_err(|_| format!("a bad row count line {t:?}"))?);
        } else if !t.starts_with("--") {
            lines.push(line.trim_end());
        }
    }
    let mut sql = lines.join("\n").trim().replace(" day (3)", " day");
    if !sql.ends_with(';') {
        return Err(format!("the query from qgen does not end with ';': {sql:?}"));
    }
    if let Some(n) = limit {
        sql.pop();
        sql = format!("{}\nlimit {n};", sql.trim_end());
    }
    Ok(sql)
}

/// The `.tbl` file of a table.
pub(crate) fn tbl(data: &Path, table: &str) -> PathBuf {
    data.join(format!("{table}.tbl"))
}

/// The SQL script that loads DuckDB: the tables with their keys, then `COPY` of each file, then `CHECKPOINT`. DuckDB adds no key to a table after its load, so the keys come first and the tables load in the order of `TABLES`.
pub(crate) fn duckdb_script(data: &Path) -> String {
    let mut s = String::new();
    for t in &TABLES {
        s.push_str(&t.create(true));
        s.push_str(";\n");
    }
    for t in &TABLES {
        s.push_str(&format!(
            "COPY {} FROM '{}' (DELIMITER '|', HEADER false);\n",
            t.name,
            tbl(data, t.name).display()
        ));
    }
    s.push_str("CHECKPOINT;\n");
    s
}

/// Loads a server over one connection: the tables, `COPY FROM STDIN` of each file, the keys, `VACUUM ANALYZE` and `CHECKPOINT`. It returns the rows of each table.
pub(crate) fn load_server(
    conn: &mut Conn,
    data: &Path,
    mut progress: impl FnMut(&str),
) -> Result<Vec<(String, u64)>, String> {
    conn.simple(&drop_all())?;
    for t in &TABLES {
        conn.simple(&t.create(false))?;
    }
    let mut counts = Vec::new();
    for t in &TABLES {
        let path = tbl(data, t.name);
        let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut input = BufReader::with_capacity(1 << 20, file);
        let tag = conn.copy_in(
            &format!("COPY {} FROM STDIN (DELIMITER '|')", t.name),
            &mut input,
            Some(b'|'),
        )?;
        let rows = tag.strip_prefix("COPY ").and_then(|n| n.parse().ok()).unwrap_or(0);
        progress(&format!("{:<9} {rows} rows", t.name));
        counts.push((t.name.to_owned(), rows));
    }
    for t in &TABLES {
        for sql in t.add_keys() {
            conn.simple(&sql)?;
        }
    }
    conn.simple("VACUUM ANALYZE")?;
    conn.simple("CHECKPOINT")?;
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema() {
        let region = &TABLES[0];
        assert_eq!(
            region.create(false),
            "CREATE TABLE region (r_regionkey INTEGER NOT NULL, r_name CHAR(25) NOT NULL, r_comment VARCHAR(152))"
        );
        let lineitem = &TABLES[7];
        let keys = lineitem.add_keys();
        assert_eq!(keys[0], "ALTER TABLE lineitem ADD PRIMARY KEY (l_orderkey, l_linenumber)");
        assert_eq!(
            keys[4],
            "ALTER TABLE lineitem ADD FOREIGN KEY (l_partkey, l_suppkey) REFERENCES partsupp (ps_partkey, ps_suppkey)"
        );
        assert!(lineitem.create(true).ends_with("REFERENCES partsupp (ps_partkey, ps_suppkey))"));
        // A table comes after the tables it references.
        for (i, t) in TABLES.iter().enumerate() {
            for (_, to, _) in t.references {
                assert!(TABLES[..i].iter().any(|p| p.name == *to), "{} references {to}", t.name);
            }
        }
        assert!(drop_all().starts_with("DROP TABLE IF EXISTS lineitem, orders,"));
        let script = duckdb_script(Path::new("/d"));
        assert!(
            script.contains("COPY region FROM '/d/region.tbl' (DELIMITER '|', HEADER false);\n")
        );
    }

    #[test]
    fn qgen_text() {
        let q3 = "-- using default substitutions\n\n\nselect\n\tl_orderkey\nfrom\n\tlineitem\norder by\n\trevenue desc,\n\to_orderdate;\n--LIMIT 10\n";
        assert_eq!(
            fix_query(q3).unwrap(),
            "select\n\tl_orderkey\nfrom\n\tlineitem\norder by\n\trevenue desc,\n\to_orderdate\nlimit 10;"
        );
        let q1 = "select 1 where d <= date '1998-12-01' - interval '90' day (3)\ngroup by x;\n";
        assert_eq!(
            fix_query(q1).unwrap(),
            "select 1 where d <= date '1998-12-01' - interval '90' day\ngroup by x;"
        );
        assert!(fix_query("select 1\n").is_err());
        assert!(fix_query("select 1;\n--LIMIT x\n").is_err());
    }
}
