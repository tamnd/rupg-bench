//! The TPC-C driver of spec/20 section 20.8.
//!
//! It has two forms on the same schema. The procedure form runs HammerDB TPROC-C, which calls the stored procedures that HammerDB makes in the database. The statement form is a driver in this module that sends plain statements from the client, so a New-Order makes 5 + 2n round trips (document 02 section 2.7.2). HammerDB has no statement form for PostgreSQL, so the harness has its own. Both forms run on the schema that HammerDB builds, and both are measured in the same way: NOPM from the change of `sum(d_next_o_id)`, as HammerDB counts it, the server cgroup over the measured interval after the ramp, and the consistency conditions 1 to 4 of the TPC-C specification, clause 3.3.2, after each run.

use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Write as _};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Barrier, OnceLock};
use std::time::{Duration, Instant};

use crate::cgroup::{Cgroup, Interval, Usage};
use crate::json::Json;
use crate::pg::{Config, Conn, PgError, Rows};
use crate::ycsb::{Histogram, Rng};

/// The five transactions, in the order of the reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tx {
    NewOrder,
    Payment,
    OrderStatus,
    Delivery,
    StockLevel,
}

/// The transactions with the names that HammerDB gives its procedures.
pub(crate) const TXS: [(Tx, &str); 5] = [
    (Tx::NewOrder, "neword"),
    (Tx::Payment, "payment"),
    (Tx::OrderStatus, "ostat"),
    (Tx::Delivery, "delivery"),
    (Tx::StockLevel, "slev"),
];

/// The mix of the HammerDB driver: a number from 1 to 23, New-Order for 1 to 10, Payment for 11 to 20, then one each for Delivery, Stock-Level and Order-Status. Each of the last three is above the 4 percent minimum of clause 5.2.3.
fn pick(rng: &mut Rng) -> Tx {
    match uniform(rng, 1, 23) {
        1..=10 => Tx::NewOrder,
        11..=20 => Tx::Payment,
        21 => Tx::Delivery,
        22 => Tx::StockLevel,
        _ => Tx::OrderStatus,
    }
}

/// A number in [lo, hi].
fn uniform(rng: &mut Rng, lo: u32, hi: u32) -> u32 {
    lo + rng.below(u64::from(hi - lo + 1)) as u32
}

/// NURand(A, x, y) of clause 2.1.6.
fn nurand(rng: &mut Rng, a: u32, x: u32, y: u32, c: u32) -> u32 {
    (((uniform(rng, 0, a) | uniform(rng, x, y)) + c) % (y - x + 1)) + x
}

const SYLLABLES: [&str; 10] =
    ["BAR", "OUGHT", "ABLE", "PRI", "PRES", "ESE", "ANTI", "CALLY", "ATION", "EING"];

/// The customer last name of clause 4.3.2.3 for a number from 0 to 999.
pub(crate) fn last_name(n: u32) -> String {
    let mut s = String::new();
    for d in [n / 100, n / 10 % 10, n % 10] {
        s.push_str(SYLLABLES[d as usize]);
    }
    s
}

/// The constants C of NURand for one run (clause 2.1.6).
#[derive(Clone, Copy, Debug)]
struct Consts {
    c_last: u32,
    c_id: u32,
    ol_i_id: u32,
}

/// Cents from a numeric value as text, for example `12.5` or `-3.07`.
pub(crate) fn cents(s: &str) -> Option<i64> {
    let (neg, s) = s.strip_prefix('-').map_or((false, s), |r| (true, r));
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    let mut frac = frac.to_owned();
    if frac.len() > 2 || !frac.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    while frac.len() < 2 {
        frac.push('0');
    }
    let v = int.parse::<i64>().ok()? * 100 + frac.parse::<i64>().ok()?;
    Some(if neg { -v } else { v })
}

/// Cents as a numeric value, for example `1250` as `12.50`.
pub(crate) fn money(c: i64) -> String {
    let sign = if c < 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", c.abs() / 100, c.abs() % 100)
}

/// A PostgreSQL array of text values.
pub(crate) fn text_array<'a>(items: impl IntoIterator<Item = &'a str>) -> String {
    let mut s = String::from("{");
    for (i, v) in items.into_iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        for c in v.chars() {
            if c == '"' || c == '\\' {
                s.push('\\');
            }
            s.push(c);
        }
        s.push('"');
    }
    s.push('}');
    s
}

/// The sync bound of document 03 section 3.12.2: `0.45 x 60 x T / s` New-Orders a minute, with `T` terminals and `s` the `fdatasync` p50 in seconds.
pub(crate) fn sync_bound(terminals: u32, fdatasync: Duration) -> Option<f64> {
    let s = fdatasync.as_secs_f64();
    (s > 0.0).then(|| 0.45 * 60.0 * f64::from(terminals) / s)
}

/// The p50 of `rounds` writes of 8 KiB at the start of a file in `dir`, each followed by `fdatasync`. The file is removed at the end.
pub(crate) fn fdatasync_p50(dir: &Path, rounds: u32) -> Result<Duration, String> {
    let path = dir.join(format!(".rupg-bench-fdatasync-{}", std::process::id()));
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let block = [0x5au8; 8192];
    let mut times = Vec::with_capacity(rounds as usize);
    let mut failed = None;
    for _ in 0..rounds {
        let t = Instant::now();
        if let Err(e) = file.write_all_at(&block, 0).and_then(|()| file.sync_data()) {
            failed = Some(e);
            break;
        }
        times.push(t.elapsed());
    }
    drop(file);
    let removed = std::fs::remove_file(&path);
    if let Some(e) = failed {
        return Err(format!("{}: {e}", path.display()));
    }
    removed.map_err(|e| format!("{}: {e}", path.display()))?;
    times.sort_unstable();
    times.get(times.len() / 2).copied().ok_or_else(|| "no rounds".to_owned())
}

/// The number of CPUs in a list such as `0-3,6`.
pub(crate) fn cpu_count(list: &str) -> Option<u32> {
    let mut n = 0;
    for part in list.trim().split(',').filter(|p| !p.is_empty()) {
        let (a, b) = part.split_once('-').unwrap_or((part, part));
        let (a, b): (u32, u32) = (a.parse().ok()?, b.parse().ok()?);
        n += b.checked_sub(a)? + 1;
    }
    (n > 0).then_some(n)
}

/// The CPUs that the server can use: `cpuset.cpus.effective` of its cgroup, or all CPUs of the machine.
pub(crate) fn server_cores(cg: &Cgroup) -> u32 {
    std::fs::read_to_string(cg.path.join("cpuset.cpus.effective"))
        .ok()
        .and_then(|s| cpu_count(&s))
        .or_else(|| std::thread::available_parallelism().ok().map(|n| n.get() as u32))
        .unwrap_or(1)
}

// The procedure form.

/// Where the server is and how HammerDB logs in. HammerDB makes the database `tpcc` and the role `tpcc` with the superuser `hammerdb`. Both roles have the password in `password_file` (see machines/install/tpcc-roles.sh).
#[derive(Clone, Debug)]
pub(crate) struct Server {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) password_file: PathBuf,
    pub(crate) password: String,
}

impl Server {
    pub(crate) fn new(host: String, port: u16, password_file: PathBuf) -> Result<Server, String> {
        let password = std::fs::read_to_string(&password_file)
            .map_err(|e| {
                format!("{}: {e}. Run machines/install/tpcc-roles.sh", password_file.display())
            })?
            .trim()
            .to_owned();
        if password.is_empty() {
            return Err(format!("{} is empty", password_file.display()));
        }
        Ok(Server { host, port, password_file, password })
    }

    /// The connection of the role `tpcc` to the database `tpcc`.
    pub(crate) fn tpcc(&self) -> Config {
        self.config("tpcc", "tpcc")
    }

    /// The connection of the superuser `hammerdb` to the database `postgres`.
    pub(crate) fn admin(&self) -> Config {
        self.config("hammerdb", "postgres")
    }

    fn config(&self, user: &str, dbname: &str) -> Config {
        Config {
            host: self.host.clone(),
            port: self.port,
            user: user.to_owned(),
            dbname: dbname.to_owned(),
            password: Some(self.password.clone()),
        }
    }

    /// The `dbset` and `diset` lines that every script has. The script reads the password from the file, so the password is not in the script.
    fn tcl_head(&self) -> Result<String, String> {
        Ok(format!(
            "set pass [string trim [read [open {}]]]\ndbset db pg\ndbset bm TPC-C\ndiset connection pg_host {}\ndiset connection pg_port {}\ndiset connection pg_sslmode disable\ndiset tpcc pg_superuser hammerdb\ndiset tpcc pg_superuserpass $pass\ndiset tpcc pg_defaultdbase postgres\ndiset tpcc pg_user tpcc\ndiset tpcc pg_pass $pass\ndiset tpcc pg_dbase tpcc\ndiset tpcc pg_storedprocs true\n",
            tcl_word(&self.password_file.display().to_string())?,
            tcl_word(&self.host)?,
            self.port
        ))
    }
}

/// A word for a Tcl script, in braces. Words with braces, backslashes or line breaks are refused.
pub(crate) fn tcl_word(s: &str) -> Result<String, String> {
    if s.is_empty() || s.contains(['{', '}', '\\', '\n', '\r']) {
        return Err(format!("{s:?} cannot be a word of a HammerDB script"));
    }
    Ok(format!("{{{s}}}"))
}

/// The script that builds the schema with `vu` virtual users. HammerDB partitions `order_line` from 200 warehouses, as in its sample script.
pub(crate) fn build_script(s: &Server, warehouses: u32, vu: u32) -> Result<String, String> {
    let mut t = s.tcl_head()?;
    let _ = write!(
        t,
        "diset tpcc pg_count_ware {warehouses}\ndiset tpcc pg_num_vu {vu}\ndiset tpcc pg_tspace pg_default\ndiset tpcc pg_partition {}\nbuildschema\nputs \"RUPG-BENCH BUILD DONE\"\n",
        warehouses >= 200
    );
    Ok(t)
}

/// The script of a timed run: a ramp of `rampup` minutes, then `duration` minutes that count, with the time profile on. It does not vacuum at the end.
pub(crate) fn run_script(
    s: &Server,
    vu: u32,
    rampup: u32,
    duration: u32,
) -> Result<String, String> {
    let mut t = s.tcl_head()?;
    let _ = write!(
        t,
        "diset tpcc pg_driver timed\ndiset tpcc pg_total_iterations 10000000\ndiset tpcc pg_rampup {rampup}\ndiset tpcc pg_duration {duration}\ndiset tpcc pg_allwarehouse false\ndiset tpcc pg_timeprofile true\ndiset tpcc pg_vacuum false\nloadscript\nvuset vu {vu}\nvuset logtotemp 0\nvucreate\nvurun\nvudestroy\nputs \"RUPG-BENCH RUN DONE\"\n"
    );
    Ok(t)
}

/// The output of one HammerDB script.
#[derive(Debug)]
pub(crate) struct HammerOut {
    pub(crate) lines: Vec<String>,
    /// The server cgroup from "Rampup complete" to "Test complete", for a run.
    pub(crate) measured: Option<Usage>,
    pub(crate) log: PathBuf,
    pub(crate) work: PathBuf,
}

/// Runs `hammerdbcli auto` on `script` in a new directory `work/name`, which is also its TMP, so the settings of an earlier run do not carry over. The log is `work/name/hammerdb.log`, with the password replaced. For a run, the server cgroup is measured between the lines of the monitor that start and end the timed interval.
pub(crate) fn run_hammerdb(
    hammerdb: &Path,
    work: &Path,
    name: &str,
    script: &str,
    server: &Server,
    cg: &Cgroup,
    say: &dyn Fn(&str),
) -> Result<HammerOut, String> {
    let dir = work.join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let tcl = dir.join("script.tcl");
    std::fs::write(&tcl, script).map_err(|e| format!("{}: {e}", tcl.display()))?;
    let log_path = dir.join("hammerdb.log");
    let mut log =
        std::fs::File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
    let stderr = log.try_clone().map_err(|e| e.to_string())?;
    let mut child = Command::new(hammerdb.join("hammerdbcli"))
        .arg("auto")
        .arg(&tcl)
        .current_dir(hammerdb)
        .env("TMP", &dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| format!("{}/hammerdbcli: {e}", hammerdb.display()))?;
    let out = child.stdout.take().ok_or("no stdout")?;
    let mut lines = Vec::new();
    let mut interval = None;
    let mut measured = None;
    let mut failed = None;
    for line in BufReader::new(out).lines() {
        let line = match line {
            Ok(l) => l.replace(&server.password, "********"),
            Err(e) => {
                failed = Some(e.to_string());
                break;
            }
        };
        let _ = writeln!(log, "{line}");
        if line.contains("Rampup complete, Taking start Transaction Count") {
            say("         rampup complete, the measured interval starts");
            interval = Some(Interval::start(cg)?);
        } else if line.contains("Test complete, Taking end Transaction Count")
            && let Some(i) = interval.take()
        {
            measured = Some(i.finish()?);
        }
        if line.contains("Error") || line.contains("FINISHED FAILED") {
            say(&format!("         hammerdb: {line}"));
        }
        lines.push(line);
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    if let Some(e) = failed {
        return Err(format!("reading the output of hammerdbcli: {e}"));
    }
    if !status.success() {
        return Err(format!("hammerdbcli failed: {status}, see {}", log_path.display()));
    }
    Ok(HammerOut { lines, measured, log: log_path, work: dir })
}

/// NOPM and TPM from "TEST RESULT : System achieved N NOPM from M PostgreSQL TPM".
pub(crate) fn parse_result(lines: &[String]) -> Option<(u64, u64)> {
    let line = lines.iter().find(|l| l.contains("TEST RESULT"))?;
    let words: Vec<&str> = line.split_whitespace().collect();
    let at = |w: &str| words.iter().position(|x| *x == w);
    let nopm = words.get(at("NOPM")?.checked_sub(1)?)?.parse().ok()?;
    let tpm = words.get(at("TPM")?.checked_sub(2)?)?.parse().ok()?;
    Some((nopm, tpm))
}

/// The summary of the HammerDB time profile (`hdbxtprofile.log`): for each procedure, its calls and its times in milliseconds. The profile covers the whole run of each virtual user, the ramp too.
pub(crate) fn parse_profile(text: &str) -> Vec<(String, Json)> {
    let mut out = Vec::new();
    let Some(start) = text.rfind(">>>>> SUMMARY OF") else {
        return out;
    };
    let mut cur: Option<(String, Json)> = None;
    for line in text[start..].lines().skip(1) {
        if line.starts_with("+-") {
            break;
        }
        if let Some(name) = line.strip_prefix(">>>>> PROC: ") {
            out.extend(cur.take());
            cur = Some((name.trim().to_ascii_lowercase(), Json::obj()));
            continue;
        }
        let Some((_, j)) = cur.as_mut() else { continue };
        for part in line.split('\t') {
            let Some((k, v)) = part.split_once(':') else { continue };
            let (key, v) = (k.trim().to_ascii_lowercase(), v.trim());
            let value = if key == "calls" {
                v.parse::<u64>().ok().map(|n| ("calls".to_owned(), Json::from(n)))
            } else if let Some(ms) = v.strip_suffix("ms").and_then(|n| n.parse::<f64>().ok()) {
                Some((format!("{key}_ms"), Json::from(ms)))
            } else {
                v.strip_suffix('%')
                    .and_then(|n| n.parse::<f64>().ok())
                    .map(|pc| (format!("{key}_percent"), Json::from(pc)))
            };
            if let Some((name, value)) = value {
                *j = std::mem::replace(j, Json::Null).with(&name, value);
            }
        }
    }
    out.extend(cur);
    out
}

// The consistency conditions.

/// One consistency condition of clause 3.3.2: how many warehouses or districts it checked and how many fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Condition {
    pub(crate) number: u32,
    pub(crate) text: &'static str,
    pub(crate) checked: u64,
    pub(crate) failed: u64,
}

/// The SQL of conditions 1 to 4. Each returns the units it checked and the units that fail. Condition 2 compares `max(NO_O_ID)` only in a district that still has rows in NEW-ORDER, because Delivery can empty a district.
const CONDITIONS: [(u32, &str, &str); 4] = [
    (
        1,
        "W_YTD = sum(D_YTD) for each warehouse",
        "SELECT count(*), count(*) FILTER (WHERE w.w_ytd <> d.s) FROM warehouse w JOIN (SELECT d_w_id, sum(d_ytd) AS s FROM district GROUP BY d_w_id) d ON d.d_w_id = w.w_id",
    ),
    (
        2,
        "D_NEXT_O_ID - 1 = max(O_ID) = max(NO_O_ID) for each district",
        "SELECT count(*), count(*) FILTER (WHERE o.m IS DISTINCT FROM d.d_next_o_id - 1 OR (n.m IS NOT NULL AND n.m <> d.d_next_o_id - 1)) FROM district d LEFT JOIN (SELECT o_w_id, o_d_id, max(o_id) AS m FROM orders GROUP BY 1, 2) o ON o.o_w_id = d.d_w_id AND o.o_d_id = d.d_id LEFT JOIN (SELECT no_w_id, no_d_id, max(no_o_id) AS m FROM new_order GROUP BY 1, 2) n ON n.no_w_id = d.d_w_id AND n.no_d_id = d.d_id",
    ),
    (
        3,
        "max(NO_O_ID) - min(NO_O_ID) + 1 = rows in NEW-ORDER for each district",
        "SELECT count(*), count(*) FILTER (WHERE hi - lo + 1 <> n) FROM (SELECT max(no_o_id) AS hi, min(no_o_id) AS lo, count(*) AS n FROM new_order GROUP BY no_w_id, no_d_id) x",
    ),
    (
        4,
        "sum(O_OL_CNT) = rows in ORDER-LINE for each district",
        "SELECT count(*), count(*) FILTER (WHERE o.s IS DISTINCT FROM l.c) FROM (SELECT o_w_id AS w, o_d_id AS d, sum(o_ol_cnt) AS s FROM orders GROUP BY 1, 2) o FULL JOIN (SELECT ol_w_id AS w, ol_d_id AS d, count(*) AS c FROM order_line GROUP BY 1, 2) l ON l.w = o.w AND l.d = o.d",
    ),
];

/// Runs conditions 1 to 4 of clause 3.3.2.
pub(crate) fn consistency(db: &mut Conn) -> Result<Vec<Condition>, String> {
    let mut out = Vec::new();
    for (number, text, sql) in CONDITIONS {
        let r = db.query(sql)?;
        let get = |i: usize| -> Result<u64, String> {
            r.rows
                .first()
                .and_then(|row| row.get(i).cloned().flatten())
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| format!("condition {number}: no count"))
        };
        out.push(Condition { number, text, checked: get(0)?, failed: get(1)? });
    }
    Ok(out)
}

pub(crate) fn conditions_json(c: &[Condition]) -> Json {
    c.iter()
        .map(|c| {
            Json::obj()
                .with("condition", c.number)
                .with("text", c.text)
                .with("checked", c.checked)
                .with("failed", c.failed)
        })
        .collect::<Vec<_>>()
        .into()
}

/// The sum of `d_next_o_id`. Its change over an interval is the count of New-Orders that committed in it.
pub(crate) fn next_order_sum(db: &mut Conn) -> Result<u64, String> {
    let r = db.query("SELECT sum(d_next_o_id) FROM district")?;
    r.rows
        .first()
        .and_then(|row| row.first().cloned().flatten())
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| "district is empty".to_owned())
}

// The statement form.

/// The statements of the statement form. Each connection prepares them once.
const STATEMENTS: [(&str, &str); 24] = [
    ("begin", "BEGIN"),
    ("commit", "COMMIT"),
    (
        "no_wc",
        "SELECT c_discount, c_last, c_credit, w_tax FROM customer, warehouse WHERE w_id = $1::int AND c_w_id = $1::int AND c_d_id = $2::int AND c_id = $3::int",
    ),
    (
        "no_d",
        "UPDATE district SET d_next_o_id = d_next_o_id + 1 WHERE d_w_id = $1::int AND d_id = $2::int RETURNING d_next_o_id - 1, d_tax",
    ),
    (
        "no_o",
        "INSERT INTO orders (o_id, o_d_id, o_w_id, o_c_id, o_entry_d, o_ol_cnt, o_all_local) VALUES ($1::int, $2::int, $3::int, $4::int, current_timestamp, $5::int, $6::int)",
    ),
    (
        "no_no",
        "INSERT INTO new_order (no_o_id, no_d_id, no_w_id) VALUES ($1::int, $2::int, $3::int)",
    ),
    ("no_i", "SELECT i_price, i_name, i_data FROM item WHERE i_id = $1::int"),
    (
        "no_s",
        "UPDATE stock SET s_quantity = CASE WHEN s_quantity >= $3::int + 10 THEN s_quantity - $3::int ELSE s_quantity - $3::int + 91 END, s_ytd = s_ytd + $3::int, s_order_cnt = s_order_cnt + 1, s_remote_cnt = s_remote_cnt + $4::int WHERE s_i_id = $1::int AND s_w_id = $2::int RETURNING s_quantity, (ARRAY[s_dist_01, s_dist_02, s_dist_03, s_dist_04, s_dist_05, s_dist_06, s_dist_07, s_dist_08, s_dist_09, s_dist_10])[$5::int], s_data",
    ),
    (
        "no_ol",
        "INSERT INTO order_line (ol_o_id, ol_d_id, ol_w_id, ol_number, ol_i_id, ol_supply_w_id, ol_quantity, ol_amount, ol_dist_info) SELECT $1::int, $2::int, $3::int, n, i, s, q, a, d FROM unnest($4::int[], $5::int[], $6::int[], $7::int[], $8::numeric[], $9::text[]) AS t(n, i, s, q, a, d)",
    ),
    (
        "pay_w",
        "UPDATE warehouse SET w_ytd = w_ytd + $2::numeric WHERE w_id = $1::int RETURNING w_name, w_street_1, w_street_2, w_city, w_state, w_zip",
    ),
    (
        "pay_d",
        "UPDATE district SET d_ytd = d_ytd + $3::numeric WHERE d_w_id = $1::int AND d_id = $2::int RETURNING d_name, d_street_1, d_street_2, d_city, d_state, d_zip",
    ),
    (
        "pay_cn",
        "SELECT c_id FROM customer WHERE c_w_id = $1::int AND c_d_id = $2::int AND c_last = $3::text ORDER BY c_first",
    ),
    (
        "pay_c",
        "UPDATE customer SET c_balance = c_balance - $4::numeric, c_ytd_payment = c_ytd_payment + $4::numeric, c_payment_cnt = c_payment_cnt + 1, c_data = CASE WHEN c_credit = 'BC' THEN left($5::text || ' ' || c_data, 500) ELSE c_data END WHERE c_w_id = $1::int AND c_d_id = $2::int AND c_id = $3::int RETURNING c_first, c_middle, c_last, c_street_1, c_street_2, c_city, c_state, c_zip, c_phone, c_since, c_credit, c_credit_lim, c_discount, c_balance, left(c_data, 200)",
    ),
    (
        "pay_h",
        "INSERT INTO history (h_c_id, h_c_d_id, h_c_w_id, h_d_id, h_w_id, h_date, h_amount, h_data) VALUES ($1::int, $2::int, $3::int, $4::int, $5::int, current_timestamp, $6::numeric, $7::text)",
    ),
    (
        "os_cn",
        "SELECT c_id, c_balance, c_first, c_middle, c_last FROM customer WHERE c_w_id = $1::int AND c_d_id = $2::int AND c_last = $3::text ORDER BY c_first",
    ),
    (
        "os_c",
        "SELECT c_id, c_balance, c_first, c_middle, c_last FROM customer WHERE c_w_id = $1::int AND c_d_id = $2::int AND c_id = $3::int",
    ),
    (
        "os_o",
        "SELECT o_id, o_carrier_id, o_entry_d, o_ol_cnt FROM orders WHERE o_w_id = $1::int AND o_d_id = $2::int AND o_c_id = $3::int ORDER BY o_id DESC LIMIT 1",
    ),
    (
        "os_ol",
        "SELECT ol_i_id, ol_supply_w_id, ol_quantity, ol_amount, ol_delivery_d FROM order_line WHERE ol_w_id = $1::int AND ol_d_id = $2::int AND ol_o_id = $3::int",
    ),
    (
        "dl_no",
        "DELETE FROM new_order WHERE no_w_id = $1::int AND no_d_id = $2::int AND no_o_id = (SELECT min(no_o_id) FROM new_order WHERE no_w_id = $1::int AND no_d_id = $2::int) RETURNING no_o_id",
    ),
    (
        "dl_o",
        "UPDATE orders SET o_carrier_id = $4::int WHERE o_w_id = $1::int AND o_d_id = $2::int AND o_id = $3::int RETURNING o_c_id",
    ),
    (
        "dl_ol",
        "WITH u AS (UPDATE order_line SET ol_delivery_d = current_timestamp WHERE ol_w_id = $1::int AND ol_d_id = $2::int AND ol_o_id = $3::int RETURNING ol_amount) SELECT coalesce(sum(ol_amount), 0), count(*) FROM u",
    ),
    (
        "dl_c",
        "UPDATE customer SET c_balance = c_balance + $4::numeric, c_delivery_cnt = c_delivery_cnt + 1 WHERE c_w_id = $1::int AND c_d_id = $2::int AND c_id = $3::int",
    ),
    ("sl_d", "SELECT d_next_o_id FROM district WHERE d_w_id = $1::int AND d_id = $2::int"),
    (
        "sl_c",
        "SELECT count(DISTINCT s_i_id) FROM order_line, stock WHERE ol_w_id = $1::int AND ol_d_id = $2::int AND ol_o_id < $3::int AND ol_o_id >= $3::int - 20 AND s_w_id = $1::int AND s_i_id = ol_i_id AND s_quantity < $4::int",
    ),
];

/// An error for a result that breaks a rule of the transaction, such as a customer that is not there. It is not a server error, but it counts as an error of the run.
fn wrong(message: String) -> PgError {
    PgError { code: "wrong".to_owned(), message }
}

/// One round trip: the statements go to the server together, and the result of each comes back. The first error of the server is returned after all results are read, so the connection stays in step. An error with an empty code is a failure of the connection.
fn trip(c: &mut Conn, list: &[(&str, Vec<String>)]) -> Result<Vec<Rows>, PgError> {
    for (name, params) in list {
        let p: Vec<&[u8]> = params.iter().map(|s| s.as_bytes()).collect();
        c.queue(name, &p)?;
    }
    c.flush()?;
    let mut out = Vec::with_capacity(list.len());
    let mut first = None;
    for _ in list {
        match c.next_rows()? {
            Ok(r) => out.push(r),
            Err(e) => {
                first = first.or(Some(e));
                out.push(Rows::default());
            }
        }
    }
    first.map_or(Ok(out), Err)
}

/// The value at `row` and `col`, or an error that names the statement.
fn val<'a>(r: &'a Rows, row: usize, col: usize, what: &str) -> Result<&'a str, PgError> {
    r.rows
        .get(row)
        .and_then(|x| x.get(col))
        .and_then(|v| v.as_deref())
        .ok_or_else(|| wrong(format!("{what}: no value at row {row}, column {col}")))
}

fn num<T: std::str::FromStr>(s: &str, what: &str) -> Result<T, PgError> {
    s.parse().map_err(|_| wrong(format!("{what}: {s:?} is not a number")))
}

fn args(v: &[&dyn std::fmt::Display]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

/// What one terminal knows.
struct Terminal<'a> {
    c: &'a mut Conn,
    rng: Rng,
    k: Consts,
    w: u32,
    warehouses: u32,
}

/// What a transaction did, for the counts.
enum Done {
    Committed,
    /// A New-Order with an unused item, which rolls back (clause 2.4.2.3).
    RolledBack,
    /// A Delivery that found no new order in some districts.
    Skipped(u32),
}

impl Terminal<'_> {
    /// A warehouse other than the home warehouse, or the home warehouse when there is one.
    fn other_warehouse(&mut self) -> u32 {
        if self.warehouses == 1 {
            return self.w;
        }
        let o = uniform(&mut self.rng, 1, self.warehouses - 1);
        if o >= self.w { o + 1 } else { o }
    }

    fn new_order(&mut self) -> Result<Done, PgError> {
        let (w, rng) = (self.w, &mut self.rng);
        let d = uniform(rng, 1, 10);
        let c_id = nurand(rng, 1023, 1, 3000, self.k.c_id);
        let n = uniform(rng, 5, 15);
        let rollback = uniform(rng, 1, 100) == 1;
        let mut items = Vec::with_capacity(n as usize);
        for j in 0..n {
            let rng = &mut self.rng;
            let i = if rollback && j == n - 1 {
                100_001
            } else {
                nurand(rng, 8191, 1, 100_000, self.k.ol_i_id)
            };
            let remote = uniform(rng, 1, 100) == 1;
            let qty = uniform(rng, 1, 10);
            let supply = if remote { self.other_warehouse() } else { w };
            items.push((i, supply, qty));
        }
        // In the order of the keys, so two New-Orders do not wait on each other in a cycle. The unused item is the largest, so it is still the last.
        items.sort_unstable();
        let all_local = u32::from(items.iter().all(|x| x.1 == w));
        let r = trip(self.c, &[("begin", vec![]), ("no_wc", args(&[&w, &d, &c_id]))])?;
        val(&r[1], 0, 0, "New-Order customer")?;
        let r = trip(self.c, &[("no_d", args(&[&w, &d]))])?;
        let o_id: u32 = num(val(&r[0], 0, 0, "New-Order district")?, "d_next_o_id")?;
        trip(self.c, &[("no_o", args(&[&o_id, &d, &w, &c_id, &n, &all_local]))])?;
        trip(self.c, &[("no_no", args(&[&o_id, &d, &w]))])?;
        let mut amounts = Vec::with_capacity(items.len());
        let mut dists = Vec::with_capacity(items.len());
        for &(i, supply, qty) in &items {
            let r = trip(self.c, &[("no_i", args(&[&i]))])?;
            if r[0].rows.is_empty() {
                // The unused item: the transaction rolls back.
                self.c.simple("ROLLBACK")?;
                if i != 100_001 {
                    return Err(wrong(format!("New-Order: item {i} is not there")));
                }
                return Ok(Done::RolledBack);
            }
            let price = cents(val(&r[0], 0, 0, "item")?)
                .ok_or_else(|| wrong("New-Order: the item price is not a number".to_owned()))?;
            let remote = u32::from(supply != w);
            let r = trip(self.c, &[("no_s", args(&[&i, &supply, &qty, &remote, &d]))])?;
            dists.push(val(&r[0], 0, 1, "stock")?.to_owned());
            amounts.push(money(price * i64::from(qty)));
        }
        let list = |f: &dyn Fn(usize) -> String| {
            format!("{{{}}}", (0..items.len()).map(f).collect::<Vec<_>>().join(","))
        };
        let numbers = list(&|j| (j + 1).to_string());
        let ids = list(&|j| items[j].0.to_string());
        let supplies = list(&|j| items[j].1.to_string());
        let qtys = list(&|j| items[j].2.to_string());
        let amounts = format!("{{{}}}", amounts.join(","));
        let dists = text_array(dists.iter().map(String::as_str));
        let mut p = args(&[&o_id, &d, &w]);
        p.extend([numbers, ids, supplies, qtys, amounts, dists]);
        let r = trip(self.c, &[("no_ol", p), ("commit", vec![])])?;
        if r[0].tag != format!("INSERT 0 {}", items.len()) {
            return Err(wrong(format!("New-Order: order_line insert gave {:?}", r[0].tag)));
        }
        Ok(Done::Committed)
    }

    fn payment(&mut self) -> Result<Done, PgError> {
        let (w, rng) = (self.w, &mut self.rng);
        let d = uniform(rng, 1, 10);
        let home = uniform(rng, 1, 100) <= 85;
        let by_name = uniform(rng, 1, 100) <= 60;
        let amount = money(i64::from(uniform(rng, 100, 500_000)));
        let name = last_name(nurand(rng, 255, 0, 999, self.k.c_last));
        let id = nurand(rng, 1023, 1, 3000, self.k.c_id);
        let (cw, cd) = if home || self.warehouses == 1 {
            (w, d)
        } else {
            let cd = uniform(&mut self.rng, 1, 10);
            (self.other_warehouse(), cd)
        };
        let r = trip(self.c, &[("begin", vec![]), ("pay_w", args(&[&w, &amount]))])?;
        let w_name = val(&r[1], 0, 0, "Payment warehouse")?.to_owned();
        let r = trip(self.c, &[("pay_d", args(&[&w, &d, &amount]))])?;
        let d_name = val(&r[0], 0, 0, "Payment district")?.to_owned();
        let c_id: u32 = if by_name {
            let r = trip(self.c, &[("pay_cn", args(&[&cw, &cd, &name]))])?;
            let n = r[0].rows.len();
            if n == 0 {
                return Err(wrong(format!("Payment: no customer {name} in {cw}/{cd}")));
            }
            num(val(&r[0], (n - 1) / 2, 0, "Payment customer")?, "c_id")?
        } else {
            id
        };
        let note = format!("{c_id} {cd} {cw} {d} {w} {amount}");
        let r = trip(self.c, &[("pay_c", args(&[&cw, &cd, &c_id, &amount, &note]))])?;
        val(&r[0], 0, 0, "Payment customer update")?;
        let h_data = format!("{w_name}    {d_name}");
        let r = trip(
            self.c,
            &[("pay_h", args(&[&c_id, &cd, &cw, &d, &w, &amount, &h_data])), ("commit", vec![])],
        )?;
        if r[0].tag != "INSERT 0 1" {
            return Err(wrong(format!("Payment: history insert gave {:?}", r[0].tag)));
        }
        Ok(Done::Committed)
    }

    fn order_status(&mut self) -> Result<Done, PgError> {
        let (w, rng) = (self.w, &mut self.rng);
        let d = uniform(rng, 1, 10);
        let by_name = uniform(rng, 1, 100) <= 60;
        let first = if by_name {
            let name = last_name(nurand(rng, 255, 0, 999, self.k.c_last));
            ("os_cn", args(&[&w, &d, &name]))
        } else {
            let id = nurand(rng, 1023, 1, 3000, self.k.c_id);
            ("os_c", args(&[&w, &d, &id]))
        };
        let r = trip(self.c, &[("begin", vec![]), first])?;
        let n = r[1].rows.len();
        if n == 0 {
            return Err(wrong(format!("Order-Status: no customer in {w}/{d}")));
        }
        let c_id: u32 = num(val(&r[1], (n - 1) / 2, 0, "Order-Status customer")?, "c_id")?;
        let r = trip(self.c, &[("os_o", args(&[&w, &d, &c_id]))])?;
        if r[0].rows.is_empty() {
            trip(self.c, &[("commit", vec![])])?;
            return Ok(Done::Committed);
        }
        let o_id: u32 = num(val(&r[0], 0, 0, "Order-Status order")?, "o_id")?;
        let lines: usize = num(val(&r[0], 0, 3, "Order-Status order")?, "o_ol_cnt")?;
        let r = trip(self.c, &[("os_ol", args(&[&w, &d, &o_id])), ("commit", vec![])])?;
        if r[0].rows.len() != lines {
            return Err(wrong(format!(
                "Order-Status: order {w}/{d}/{o_id} has o_ol_cnt {lines} and {} order lines",
                r[0].rows.len()
            )));
        }
        Ok(Done::Committed)
    }

    fn delivery(&mut self) -> Result<Done, PgError> {
        let w = self.w;
        let carrier = uniform(&mut self.rng, 1, 10);
        let mut skipped = 0;
        trip(self.c, &[("begin", vec![])])?;
        for d in 1..=10u32 {
            let r = trip(self.c, &[("dl_no", args(&[&w, &d]))])?;
            let Some(o) = r[0].rows.first().and_then(|row| row.first().cloned().flatten()) else {
                skipped += 1;
                continue;
            };
            let o_id: u32 = num(&o, "no_o_id")?;
            let r = trip(
                self.c,
                &[("dl_o", args(&[&w, &d, &o_id, &carrier])), ("dl_ol", args(&[&w, &d, &o_id]))],
            )?;
            let c_id: u32 = num(val(&r[0], 0, 0, "Delivery order")?, "o_c_id")?;
            let total = val(&r[1], 0, 0, "Delivery order lines")?.to_owned();
            if val(&r[1], 0, 1, "Delivery order lines")? == "0" {
                return Err(wrong(format!("Delivery: order {w}/{d}/{o_id} has no order lines")));
            }
            let r = trip(self.c, &[("dl_c", args(&[&w, &d, &c_id, &total]))])?;
            if r[0].tag != "UPDATE 1" {
                return Err(wrong(format!("Delivery: customer {w}/{d}/{c_id}: {:?}", r[0].tag)));
            }
        }
        trip(self.c, &[("commit", vec![])])?;
        Ok(if skipped > 0 { Done::Skipped(skipped) } else { Done::Committed })
    }

    fn stock_level(&mut self) -> Result<Done, PgError> {
        let w = self.w;
        let d = uniform(&mut self.rng, 1, 10);
        let threshold = uniform(&mut self.rng, 10, 20);
        let r = trip(self.c, &[("begin", vec![]), ("sl_d", args(&[&w, &d]))])?;
        let next: u32 = num(val(&r[1], 0, 0, "Stock-Level district")?, "d_next_o_id")?;
        let r = trip(self.c, &[("sl_c", args(&[&w, &d, &next, &threshold])), ("commit", vec![])])?;
        val(&r[0], 0, 0, "Stock-Level count")?;
        Ok(Done::Committed)
    }
}

/// The settings of one run of the statement form.
#[derive(Clone, Debug)]
pub(crate) struct RunSettings {
    pub(crate) warehouses: u32,
    pub(crate) terminals: u32,
    pub(crate) rampup: Duration,
    pub(crate) duration: Duration,
    pub(crate) seed: u64,
    pub(crate) sync: Option<String>,
}

/// The numbers of the terminals over the measured interval.
#[derive(Clone, Debug, Default)]
pub(crate) struct Tally {
    pub(crate) latency: [Histogram; 5],
    pub(crate) rolled_back: u64,
    pub(crate) deliveries_skipped: u64,
    pub(crate) retries: u64,
    pub(crate) errors: u64,
    pub(crate) first_error: Option<String>,
}

impl Tally {
    fn merge(&mut self, o: Tally) {
        for (a, b) in self.latency.iter_mut().zip(&o.latency) {
            a.merge(b);
        }
        self.rolled_back += o.rolled_back;
        self.deliveries_skipped += o.deliveries_skipped;
        self.retries += o.retries;
        self.errors += o.errors;
        self.first_error = self.first_error.take().or(o.first_error);
    }

    pub(crate) fn transactions(&self) -> u64 {
        self.latency.iter().map(Histogram::count).sum()
    }
}

/// The result of one run of the statement form.
#[derive(Debug)]
pub(crate) struct StatementRun {
    pub(crate) tally: Tally,
    pub(crate) usage: Usage,
    /// The change of `sum(d_next_o_id)` over the measured interval.
    pub(crate) new_orders: u64,
    pub(crate) measured: Duration,
}

/// Runs the statement form: `terminals` connections, each on its home warehouse, with no keying or think time, as HammerDB runs. The server cgroup and `sum(d_next_o_id)` are read at the end of the ramp and at the end of the interval. A transaction counts when it starts and ends in the interval.
pub(crate) fn run_statements(
    config: &Config,
    s: &RunSettings,
    cg: &Cgroup,
) -> Result<StatementRun, String> {
    let mut seed_rng = Rng::new(s.seed);
    let k = Consts {
        c_last: uniform(&mut seed_rng, 0, 255),
        c_id: uniform(&mut seed_rng, 0, 1023),
        ol_i_id: uniform(&mut seed_rng, 0, 8191),
    };
    let n = s.terminals as usize;
    let ready = Barrier::new(n + 1);
    let go = Barrier::new(n + 1);
    let window: OnceLock<(Instant, Instant)> = OnceLock::new();
    let mut db = Conn::connect(config)?;
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..n)
            .map(|i| {
                let (ready, go, window) = (&ready, &go, &window);
                scope.spawn(move || terminal(config, s, k, i as u32, (ready, go), window))
            })
            .collect();
        ready.wait();
        let now = Instant::now();
        let (t0, t1) = (now + s.rampup, now + s.rampup + s.duration);
        let _ = window.set((t0, t1));
        go.wait();
        std::thread::sleep(t0.saturating_duration_since(Instant::now()));
        let measured = (|| -> Result<(Interval, u64, Instant), String> {
            let before = next_order_sum(&mut db)?;
            let start = Instant::now();
            Ok((Interval::start(cg)?, before, start))
        })();
        std::thread::sleep(t1.saturating_duration_since(Instant::now()));
        let after = measured.and_then(|(interval, before, start)| {
            let usage = interval.finish()?;
            let elapsed = start.elapsed();
            Ok((usage, next_order_sum(&mut db)?.saturating_sub(before), elapsed))
        });
        let mut total = Tally::default();
        let mut failed = None;
        for h in handles {
            match h.join() {
                Ok(Ok(t)) => total.merge(t),
                Ok(Err(e)) => failed = failed.or(Some(e)),
                Err(_) => failed = failed.or(Some("a terminal thread panicked".to_owned())),
            }
        }
        if let Some(e) = failed {
            return Err(e);
        }
        let (usage, new_orders, measured) = after?;
        Ok(StatementRun { tally: total, usage, new_orders, measured })
    })
}

fn terminal(
    config: &Config,
    s: &RunSettings,
    k: Consts,
    i: u32,
    (ready, go): (&Barrier, &Barrier),
    window: &OnceLock<(Instant, Instant)>,
) -> Result<Tally, String> {
    let setup = (|| -> Result<Conn, PgError> {
        let mut c = Conn::connect(config)?;
        if let Some(v) = &s.sync {
            c.simple(&format!("SET synchronous_commit = {v}"))?;
        }
        for (name, sql) in STATEMENTS {
            c.prepare(name, sql)?;
        }
        Ok(c)
    })();
    ready.wait();
    go.wait();
    let mut c = setup.map_err(|e| format!("terminal {i}: {e}"))?;
    let &(t0, t1) = window.get().ok_or("no window")?;
    let mut t = Terminal {
        c: &mut c,
        rng: Rng::new(s.seed.wrapping_mul(0x9E37_79B9).wrapping_add(u64::from(i) + 1)),
        k,
        w: i % s.warehouses + 1,
        warehouses: s.warehouses,
    };
    let mut tally = Tally::default();
    loop {
        let start = Instant::now();
        if start >= t1 {
            break;
        }
        let tx = pick(&mut t.rng);
        let mut retries = 0;
        let outcome = loop {
            let r = match tx {
                Tx::NewOrder => t.new_order(),
                Tx::Payment => t.payment(),
                Tx::OrderStatus => t.order_status(),
                Tx::Delivery => t.delivery(),
                Tx::StockLevel => t.stock_level(),
            };
            match r {
                Ok(done) => break Ok(done),
                Err(e) if e.code.is_empty() => return Err(format!("terminal {i}: {e}")),
                Err(e) => {
                    t.c.simple("ROLLBACK").map_err(|e| format!("terminal {i}: {e}"))?;
                    if (e.code == "40001" || e.code == "40P01") && retries < 100 {
                        retries += 1;
                        continue;
                    }
                    break Err(e);
                }
            }
        };
        let end = Instant::now();
        if start < t0 || end > t1 {
            continue;
        }
        tally.retries += retries;
        match outcome {
            Ok(done) => {
                tally.latency[tx as usize].record(end.duration_since(start).as_nanos() as u64);
                match done {
                    Done::Committed => {}
                    Done::RolledBack => tally.rolled_back += 1,
                    Done::Skipped(n) => tally.deliveries_skipped += u64::from(n),
                }
            }
            Err(e) => {
                tally.errors += 1;
                if tally.first_error.is_none() {
                    tally.first_error = Some(e.to_string());
                }
            }
        }
    }
    Ok(tally)
}

/// `count` New-Order transactions in plain statements, with 5 + 2n statements each, as the terminals of the statement form send them. The statements that need the new order number read it from `district`, so the text needs no client between the statements. No order rolls back. They are the TPC-C part of the fixed set of spec/21 section 21.14.
pub(crate) fn fixed_new_orders(warehouses: u32, count: u32, seed: u64) -> String {
    let mut rng = Rng::new(seed);
    let c_id_c = uniform(&mut rng, 0, 1023);
    let item_c = uniform(&mut rng, 0, 8191);
    let mut s = String::new();
    for k in 0..count {
        let w = k % warehouses + 1;
        let d = uniform(&mut rng, 1, 10);
        let c = nurand(&mut rng, 1023, 1, 3000, c_id_c);
        let n = uniform(&mut rng, 5, 15);
        let mut items: Vec<(u32, u32, u32)> = (0..n)
            .map(|_| {
                let i = nurand(&mut rng, 8191, 1, 100_000, item_c);
                let remote = warehouses > 1 && uniform(&mut rng, 1, 100) == 1;
                let supply = if remote { w % warehouses + 1 } else { w };
                (i, supply, uniform(&mut rng, 1, 10))
            })
            .collect();
        items.sort_unstable();
        let local = u32::from(items.iter().all(|x| x.1 == w));
        let district = format!("FROM district WHERE d_w_id = {w} AND d_id = {d}");
        let _ = writeln!(s, "BEGIN;");
        let _ = writeln!(
            s,
            "SELECT c_discount, c_last, c_credit, w_tax FROM customer, warehouse WHERE w_id = {w} AND c_w_id = {w} AND c_d_id = {d} AND c_id = {c};"
        );
        let _ = writeln!(
            s,
            "UPDATE district SET d_next_o_id = d_next_o_id + 1 WHERE d_w_id = {w} AND d_id = {d} RETURNING d_next_o_id - 1, d_tax;"
        );
        let _ = writeln!(
            s,
            "INSERT INTO orders (o_id, o_d_id, o_w_id, o_c_id, o_entry_d, o_ol_cnt, o_all_local) SELECT d_next_o_id - 1, {d}, {w}, {c}, current_timestamp, {n}, {local} {district};"
        );
        let _ = writeln!(
            s,
            "INSERT INTO new_order (no_o_id, no_d_id, no_w_id) SELECT d_next_o_id - 1, {d}, {w} {district};"
        );
        for &(i, supply, q) in &items {
            let _ = writeln!(s, "SELECT i_price, i_name, i_data FROM item WHERE i_id = {i};");
            let _ = writeln!(
                s,
                "UPDATE stock SET s_quantity = CASE WHEN s_quantity >= {q} + 10 THEN s_quantity - {q} ELSE s_quantity - {q} + 91 END, s_ytd = s_ytd + {q}, s_order_cnt = s_order_cnt + 1, s_remote_cnt = s_remote_cnt + {} WHERE s_i_id = {i} AND s_w_id = {supply} RETURNING s_quantity, s_dist_{d:02}, s_data;",
                u32::from(supply != w)
            );
        }
        let col = |f: &dyn Fn(usize) -> u32| {
            (0..items.len()).map(|j| f(j).to_string()).collect::<Vec<_>>().join(",")
        };
        let _ = writeln!(
            s,
            "INSERT INTO order_line (ol_o_id, ol_d_id, ol_w_id, ol_number, ol_i_id, ol_supply_w_id, ol_quantity, ol_amount, ol_dist_info) SELECT d.d_next_o_id - 1, {d}, {w}, t.n, t.i, t.s, t.q, t.q * it.i_price, st.s_dist_{d:02} FROM district d, unnest(ARRAY[{}], ARRAY[{}], ARRAY[{}], ARRAY[{}]) AS t(n, i, s, q) JOIN item it ON it.i_id = t.i JOIN stock st ON st.s_i_id = t.i AND st.s_w_id = t.s WHERE d.d_w_id = {w} AND d.d_id = {d};",
            col(&|j| j as u32 + 1),
            col(&|j| items[j].0),
            col(&|j| items[j].1),
            col(&|j| items[j].2)
        );
        let _ = writeln!(s, "COMMIT;");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_nurand() {
        assert_eq!(last_name(0), "BARBARBAR");
        assert_eq!(last_name(371), "PRICALLYOUGHT");
        assert_eq!(last_name(999), "EINGEINGEING");
        let mut rng = Rng::new(7);
        for _ in 0..10_000 {
            let v = nurand(&mut rng, 1023, 1, 3000, 259);
            assert!((1..=3000).contains(&v));
            let v = nurand(&mut rng, 255, 0, 999, 0);
            assert!(v <= 999);
        }
        let mut counts = [0u32; 5];
        for _ in 0..23_000 {
            counts[pick(&mut rng) as usize] += 1;
        }
        // About 10000, 10000, 1000, 1000 and 1000.
        assert!(counts[0] > 9_500 && counts[0] < 10_500, "{counts:?}");
        assert!(counts[2] > 800 && counts[2] < 1_200, "{counts:?}");
    }

    #[test]
    fn money_and_arrays() {
        assert_eq!(cents("12.34"), Some(1234));
        assert_eq!(cents("12.3"), Some(1230));
        assert_eq!(cents("-0.07"), Some(-7));
        assert_eq!(cents("7"), Some(700));
        assert_eq!(cents("1.234"), None);
        assert_eq!(money(1234), "12.34");
        assert_eq!(money(-7), "-0.07");
        assert_eq!(money(500_000), "5000.00");
        assert_eq!(text_array(["ab", "c\"d", "e\\f"]), r#"{"ab","c\"d","e\\f"}"#);
        assert_eq!(cpu_count("0-7\n"), Some(8));
        assert_eq!(cpu_count("0-3,6"), Some(5));
        assert_eq!(cpu_count(""), None);
        let b = sync_bound(16, Duration::from_millis(2)).unwrap_or_default();
        assert!((b - 216_000.0).abs() < 1e-6);
    }

    #[test]
    fn hammerdb_scripts_and_output() {
        let s = Server {
            host: "/var/run/postgresql".to_owned(),
            port: 5433,
            password_file: PathBuf::from("/etc/rupg-bench/tpcc.pass"),
            password: "secret".to_owned(),
        };
        let b = build_script(&s, 4, 2).unwrap_or_default();
        assert!(b.contains("diset tpcc pg_count_ware 4\n"));
        assert!(b.contains("diset connection pg_port 5433\n"));
        assert!(b.contains("[open {/etc/rupg-bench/tpcc.pass}]"));
        assert!(!b.contains("secret"));
        let r = run_script(&s, 8, 5, 20).unwrap_or_default();
        assert!(r.contains("diset tpcc pg_rampup 5\ndiset tpcc pg_duration 20\n"));
        assert!(r.contains("vuset vu 8\n"));
        assert!(tcl_word("a{b").is_err());
        let lines = vec![
            "Vuser 1:2 Active Virtual Users configured".to_owned(),
            "Vuser 1:TEST RESULT : System achieved 1785 NOPM from 4109 PostgreSQL TPM".to_owned(),
        ];
        assert_eq!(parse_result(&lines), Some((1785, 4109)));
        assert_eq!(parse_result(&lines[..1]), None);
        let profile = ">>>>> VIRTUAL USER 2 : ELAPSED TIME : 120522ms\n>>>>> PROC: NEWORD\nCALLS: 9\tMIN: 1.0ms\tAVG: 2.0ms\tMAX: 3.0ms\tTOTAL: 18.0ms\nP99: 3.0ms\tP95: 3.0ms\tP75: 2.5ms\tP50: 2.0ms\tP25: 1.5ms\tSD: 0.5ms\tRATIO: 50.0%\n+-+-+\n>>>>> SUMMARY OF 2 ACTIVE VIRTUAL USERS : MEDIAN ELAPSED TIME : 127146ms\n>>>>> PROC: NEWORD\nCALLS: 3427\tMIN: 1.730ms\tAVG: 36.332ms\tMAX: 1220.165ms\tTOTAL: 124508.229ms\nP99: 210.118ms\tP95: 104.921ms\tP75: 50.942ms\tP50: 19.126ms\tP25: 9.919ms\tSD: 49.663ms\tRATIO: 48.963%\n>>>>> PROC: OSTAT\nCALLS: 332\tMIN: 0.485ms\tAVG: 15.077ms\tMAX: 225.815ms\tTOTAL: 5005.542ms\nP99: 112.663ms\tP95: 70.415ms\tP75: 13.620ms\tP50: 4.711ms\tP25: 1.395ms\tSD: 26.570ms\tRATIO: 1.968%\n+-+-+-+\n";
        let p = parse_profile(profile);
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].0, "neword");
        assert_eq!(p[0].1.get("calls"), Some(&Json::from(3427u64)));
        assert_eq!(p[0].1.get("p50_ms"), Some(&Json::from(19.126)));
        assert_eq!(p[1].0, "ostat");
        assert_eq!(p[1].1.get("ratio_percent"), Some(&Json::from(1.968)));
    }

    #[test]
    fn fixed_new_orders_have_5_plus_2n_statements() {
        let text = fixed_new_orders(2, 100, 1);
        let orders: Vec<&str> = text.split("BEGIN;\n").skip(1).collect();
        assert_eq!(orders.len(), 100);
        for o in orders {
            let items = o.matches("FROM item WHERE").count();
            assert!((5..=15).contains(&items));
            // BEGIN and COMMIT are not round trips of their own: they go with the first and the last statement.
            let statements = o.matches(";\n").count() - 1;
            assert_eq!(statements, 5 + 2 * items, "{o}");
        }
    }
}
