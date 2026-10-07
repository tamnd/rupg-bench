//! The benchmark harness for rupg.
//!
//! The harness runs rupg and the baselines on the same machine, with the same transport, and reports the ratio. The rules are in `spec/02-the-goal.md` and `spec/20-benchmarks.md` of tamnd/rupg.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cgroup::Cgroup;
use crate::json::Json;

mod answers;
mod args;
mod cgroup;
mod clickbench;
mod gates;
mod instructions;
mod json;
mod load;
mod m1;
mod pg;
mod pgbench;
mod pins;
mod report;
mod suite;
mod toml;
mod tpcc;
mod tpch;
mod ycsb;

const USAGE: &str = "usage: rupg-bench <command> [options]

commands:
  gates                       print the twelve gates of spec/02 section 2.10
  pins [--file F] [--shell | --get SYSTEM.KEY]
                              print the pins of pins.toml, as shell assignments, or one value
  measure (--name N [--cpus LIST] | --attach PATH | --unit UNIT) [--idle [SECONDS]] [--du PATH] [--json] [-- COMMAND...]
                              measure a command or a server with the cgroup v2 runner of spec/20 section 20.4.
                              --name starts COMMAND in the new cgroup /sys/fs/cgroup/rupg-bench/N.
                              --attach and --unit measure a cgroup that exists, such as a systemd unit,
                              while COMMAND (the client) runs outside it. --idle measures the idle base first (10 s by default).
  load --case a|b --source FILE (--name N [--cpus LIST] | --attach PATH | --unit UNIT) [--du PATH] [--json] -- COMMAND...
                              run the steps of case A or case B of spec/20 section 20.3.1 on FILE
                              (drop_caches, fincore, the timed cat), then time COMMAND, the load,
                              in the cgroup runner. Case A records L, R and L / R.
  report --result FILE [--out DIR] [--suite S] [--machine M] [--commit C] [--smoke]
                              write DIR/<date>/<commit>-<machine>-<suite>.json and .md from a result file
                              (the --json output of a command). The options fill the fields that the file lacks.
                              --smoke marks the run as a smoke run, not a baseline. DIR is reports by default.
  answers --expected PATH --actual PATH
                              compare two answer sets as multisets of rows (spec/20 section 20.5).
                              PATH is a file or a directory of <query>.tsv files. An expected file
                              <query>.out is a TPC-H answer file and is compared with the rules of TPC-H clause 2.1.3.5.
  pgbench (--unit UNIT | --attach PATH) [--conn WORDS] [--bin DIR] [--scale N] [--clients N] [--jobs N] [--time SECONDS]
          [--smoke] [--json] [--report DIR [--machine M]]
                              run pgbench -i and the TPC-B like script against the server, measure the server cgroup
                              with the idle base, and run the consistency check of spec/21 section 21.4.6.
                              --conn takes libpq words: host, port, user, dbname, password (user=bench dbname=bench by default).
                              --report writes the report files under DIR (see report).
                              --smoke marks the result as a smoke run, which is not a baseline.
  instructions --queries PATH --set NAME (--duckdb DB [--duckdb-bin B] [--threads N] | --engine NAME (--unit UNIT | --attach PATH))
          [--bin DIR] [--conn WORDS] [--password-file F] [--repeat N] [--ratchet FILE] [--save FILE] [--smoke] [--json] [--report DIR [--machine M]]
                              count the instructions retired for each query with perf stat (spec/21 section 21.14).
                              PATH is a file with one query on each line or a directory of <name>.sql files.
                              --duckdb counts the duckdb process. --unit and --attach count the server cgroup on all
                              CPUs while psql from --bin reads the query on stdin and sends each statement. Each query runs --repeat times (3) and the
                              minimum counts, minus the count of SELECT 1. --ratchet compares with [instructions.NAME]
                              of a ratchet file and fails over the budgets (3 percent a query, 1 percent in total).
                              --save writes that table for this run. --password-file gives psql the password in PGPASSWORD.
  fixed-set --out DIR [--records N] [--warehouses N] [--seed N]
                              write the YCSB and TPC-C parts of the fixed set of spec/21 section 21.14 for instructions:
                              DIR/ycsb/read1000.sql has 1,000 YCSB point reads on a usertable of --records rows (100,000),
                              and DIR/tpcc/neword100.sql has 100 New-Order transactions in plain statements on --warehouses (2).
  tpch --tools DIR --scale S --data DIR (--duckdb DB [--duckdb-bin B] [--threads N] [--cpus LIST] | --engine NAME (--unit UNIT | --attach PATH) [--conn WORDS])
          [--steps gen,load,run] [--runs N] [--no-cold] [--answers DIR] [--save-answers DIR] [--du PATH] [--smoke] [--json] [--report DIR [--machine M]]
                              the TPC-H driver of spec/20 section 20.7. DIR of --tools is the dbgen directory that
                              machines/install/tpch-tools.sh builds. gen runs dbgen into --data and writes the 22 queries of
                              qgen -d to DATA/queries. load loads the .tbl files with the keys of the specification. run runs
                              each query --runs times (3), the first run cold, and checks the answers against --answers
                              (the .out files of the kit at SF1, or .tsv files).
  ycsb (--unit UNIT | --attach PATH) [--engine NAME] [--conn WORDS] [--records N] [--steps load,run] [--workloads a,b,c,f]
          [--rows 1,16,16x64] [--time SECONDS] [--sync on|off] [--seed N] [--smoke] [--json] [--report DIR [--machine M]]
                              the YCSB driver of spec/20 section 20.9: the core workloads A, B, C and F with the scrambled
                              zipfian distribution (0.99). load makes usertable with --records rows (100,000). run runs each
                              workload for each row of --rows for --time seconds (60). A row is CLIENTS, or CLIENTSxDEPTH
                              for a pipeline of DEPTH statements on each connection. --sync sets synchronous_commit on each
                              connection. The server cgroup is measured over each run, with the idle base first.
  tpcc (--unit UNIT | --attach PATH) [--hammerdb DIR] [--host H] [--port P] [--password-file F] [--steps build,procedures,statements]
          [--warehouses N] [--build-vu N] [--vu 8] [--rampup MINUTES] [--duration MINUTES] [--sync on,off] [--sync-dir DIR]
          [--work DIR] [--seed N] [--smoke] [--json] [--report DIR [--machine M]]
                              the TPC-C driver of spec/20 section 20.8. build drops the database tpcc and builds --warehouses
                              (10) with HammerDB from DIR (/opt/rupg-bench/hammerdb). procedures runs HammerDB TPROC-C with its
                              stored procedures. statements runs the driver of the harness, which sends plain statements, so a
                              New-Order makes 5 + 2n round trips. Each run has a ramp of --rampup minutes (5) and counts
                              --duration minutes (20) for each count of --vu and each value of --sync. NOPM is the change of
                              sum(d_next_o_id). After each run the consistency conditions 1 to 4 of clause 3.3.2 must hold.
                              The conditions are also checked before the runs, and --steps check only checks them.
                              The roles come from machines/install/tpcc-roles.sh, which writes the password file
                              (/etc/rupg-bench/tpcc.pass). The sync bound uses the fdatasync p50 in --sync-dir (next to the data directory).
  clickbench --dir DIR --engine NAME [--unit UNIT | --attach PATH | --cpus LIST] [--source FILE] [--steps load,run,concurrent]
          [--lib FILE] [--conn WORDS | --duckdb-bin B] [--answers DIR] [--save-answers DIR] [--result-file FILE] [--smoke] [--json] [--report DIR [--machine M]]
                              the ClickBench driver of spec/20 section 20.5. DIR is a system directory of the ClickBench pin.
                              It runs ./load with --source, then each query of queries.sql as bench_run_query of the pin
                              runs it, with true cold runs, then ./data-size, then bench_concurrent_qps of --lib
                              (DIR/../lib/benchmark-common.sh). With --unit or --attach the server cgroup is measured,
                              else each query runs in a new cgroup. The answers come from an unmeasured pass, through
                              --conn (user=bench dbname=test) for a server or with duckdb on DIR/hits.db.
  m1 --dir DIR [--steps empty,latency,throughput,recovery] [--warmup N] [--commits N] [--writers N] [--seconds S]
          [--log-mib N] [--rows N] [--value BYTES] [--smoke] [--json] [--report DIR [--machine M]]
                              the M1 numbers of spec/23 section 23.4 with the rupg facade at the commit of pins.toml.
                              empty makes a database with the default options and gives the size of the file. latency
                              times --commits (10,000) commits of one row at one writer after --warmup (1,000). throughput
                              runs --writers (one for each core) for --seconds (30), each with its own table. recovery
                              starts rupg-bench m1-fill, which updates --rows (256) rows of --value bytes (2,000) in each
                              commit until --log-mib (1,024) MiB of values are in the log and exits with no close. Then it
                              drops the file from the page cache with dd where it can and times the open that replays the log.
  --version                   print the version";

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match run(&argv) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("rupg-bench: {e}");
            ExitCode::from(if e.starts_with("usage") { 2 } else { 1 })
        }
    }
}

fn run(argv: &[String]) -> Result<(), String> {
    let Some((command, rest)) = argv.split_first() else {
        return Err(USAGE.to_owned());
    };
    let mut a = args::Args::parse(rest)?;
    match command.as_str() {
        "gates" => {
            a.finish()?;
            for gate in gates::GATES {
                println!("{:<4} {:<4} {:<28} {}", gate.id, gate.milestone, gate.suite, gate.target);
            }
        }
        "pins" => {
            let file = a.value("file").unwrap_or_else(|| "pins.toml".to_owned());
            let shell = a.flag("shell");
            let get = a.value("get");
            a.finish()?;
            let pins = pins::Pins::load(&file)?;
            if let Some(path) = get {
                let (system, key) = path.split_once('.').ok_or("usage: --get SYSTEM.KEY")?;
                let value = pins.get(system, key).ok_or(format!("{file} has no {path}"))?;
                println!("{value}");
            } else {
                print!("{}", if shell { pins.shell() } else { pins.summary() });
            }
        }
        "measure" => measure(a)?,
        "load" => load_command(a)?,
        "report" => report_command(a)?,
        "answers" => answers_command(a)?,
        "instructions" => instructions_command(a)?,
        "tpch" => tpch_command(a)?,
        "clickbench" => clickbench_command(a)?,
        "pgbench" => pgbench_command(a)?,
        "ycsb" => ycsb_command(a)?,
        "tpcc" => tpcc_command(a)?,
        "fixed-set" => fixed_set_command(a)?,
        "m1" => m1_command(a)?,
        "m1-fill" => m1_fill_command(a)?,
        "--version" | "-V" => println!("rupg-bench {}", env!("CARGO_PKG_VERSION")),
        "--help" | "-h" | "help" => println!("{USAGE}"),
        other => return Err(format!("usage: unknown command {other:?}\n{USAGE}")),
    }
    Ok(())
}

/// The cgroup of `--name N [--cpus LIST]`, `--attach PATH` or `--unit UNIT`.
fn target(a: &mut args::Args) -> Result<Cgroup, String> {
    let cpus = a.value("cpus");
    let cg = match (a.value("name"), a.value("attach"), a.value("unit")) {
        (Some(n), None, None) => Cgroup::create(&n, cpus.as_deref())?,
        (None, Some(p), None) => Cgroup::attach(Path::new(&p))?,
        (None, None, Some(u)) => Cgroup::of_unit(&u)?,
        _ => return Err("usage: give one of --name, --attach and --unit".to_owned()),
    };
    if cpus.is_some() && !cg.owned() {
        return Err("usage: --cpus works only with --name".to_owned());
    }
    Ok(cg)
}

/// A result as JSON and as lines for a terminal.
#[derive(Debug)]
struct Out {
    json: Json,
    text: String,
}

impl Out {
    fn new(cg: &Cgroup) -> Out {
        let json = Json::obj()
            .with("cgroup", cg.path.display().to_string())
            .with(
                "unix_time",
                SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()),
            )
            .with(
                "kernel",
                std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default().trim(),
            );
        Out { json, text: format!("cgroup          {}\n", cg.path.display()) }
    }

    fn add(&mut self, key: &str, value: impl Into<Json>, text: &str) {
        let json = std::mem::replace(&mut self.json, Json::Null);
        self.json = json.with(key, value);
        self.text.push_str(text);
    }

    fn print(&self, json: bool) {
        if json {
            print!("{}", self.json.pretty());
        } else {
            print!("{}", self.text);
        }
    }

    /// Runs the command, adds its numbers and fails if it failed.
    fn run(&mut self, cg: &Cgroup, argv: &[String]) -> Result<cgroup::Usage, String> {
        let (status, usage) = cgroup::run(cg, argv)?;
        self.add("command", argv.to_vec(), &format!("command         {}\n", argv.join(" ")));
        self.add("exit_code", status.code().map(i64::from), &format!("exit            {status}\n"));
        self.add("run", usage.to_json(), &usage.text());
        if status.success() { Ok(usage) } else { Err(format!("the command failed: {status}")) }
    }

    fn du(&mut self, path: Option<String>) -> Result<(), String> {
        if let Some(path) = path {
            let bytes = cgroup::du_apparent(Path::new(&path))?;
            let text = format!("du {path}  {}\n", cgroup::mib(bytes));
            self.add("du", Json::obj().with("path", path).with("apparent_bytes", bytes), &text);
        }
        Ok(())
    }
}

/// Runs `body`, prints the result, and removes the cgroup, also when `body` fails.
fn with_cgroup(
    cg: Cgroup,
    json: bool,
    body: impl FnOnce(&Cgroup, &mut Out) -> Result<(), String>,
) -> Result<(), String> {
    let mut out = Out::new(&cg);
    let result = body(&cg, &mut out);
    out.print(json);
    let removed = cg.remove();
    result.and(removed)
}

/// The `measure` command.
fn measure(mut a: args::Args) -> Result<(), String> {
    // --idle alone is the 10 s of spec/20 section 20.4.
    let idle = if a.flag("idle") {
        Some(cgroup::IDLE_BASE)
    } else {
        a.value("idle")
            .map(|s| {
                s.parse::<f64>()
                    .ok()
                    .filter(|x| x.is_finite() && *x > 0.0)
                    .map(Duration::from_secs_f64)
                    .ok_or(format!("usage: --idle {s:?} is not a number of seconds"))
            })
            .transpose()?
    };
    let du = a.value("du");
    let json = a.flag("json");
    let argv = std::mem::take(&mut a.rest);
    let new = a.value_is_set("name");
    if new && idle.is_some() {
        return Err("usage: --idle needs --attach or --unit, because a new cgroup has nothing in it to measure".to_owned());
    }
    if new && argv.is_empty() {
        return Err("usage: --name needs a command after --".to_owned());
    }
    let cg = target(&mut a)?;
    a.finish()?;
    with_cgroup(cg, json, |cg, out| {
        if let Some(d) = idle {
            let base = cgroup::idle_base(cg, d)?;
            out.add(
                "idle_base",
                base.to_json(),
                &format!("idle base over {} s:\n{}", d.as_secs_f64(), base.text()),
            );
        }
        if !argv.is_empty() {
            out.run(cg, &argv)?;
        }
        out.du(du)
    })
}

/// The `load` command: the steps of case A or case B of spec/20 section 20.3.1, then the load in the cgroup runner.
fn load_command(mut a: args::Args) -> Result<(), String> {
    let case =
        load::Case::parse(&a.value("case").ok_or("usage: load needs --case a or --case b")?)?;
    let source = a.value("source").ok_or("usage: load needs --source FILE")?;
    let du = a.value("du");
    let json = a.flag("json");
    let argv = std::mem::take(&mut a.rest);
    if argv.is_empty() {
        return Err("usage: load needs the load command after --".to_owned());
    }
    let cg = target(&mut a)?;
    a.finish()?;
    with_cgroup(cg, json, |cg, out| {
        let source = Path::new(&source);
        let prepared = load::prepare(source, case)?;
        // The load starts right after the last check. spec/20 section 20.3.4 measures the resources over the load.
        let usage = out.run(cg, &argv)?;
        out.add(
            "source",
            source.display().to_string(),
            &format!("source          {}\n", source.display()),
        );
        out.add("load_case", prepared.to_json(Some(usage.wall)), &prepared.text(usage.wall));
        out.du(du)
    })
}

/// The `report` command.
fn report_command(mut a: args::Args) -> Result<(), String> {
    let file = a.value("result").ok_or("usage: report needs --result FILE")?;
    let out = a.value("out").unwrap_or_else(|| "reports".to_owned());
    let suite = a.value("suite");
    let machine = a.value("machine");
    let commit = a.value("commit");
    let smoke = a.flag("smoke");
    a.finish()?;
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{file}: {e}"))?;
    let mut result = Json::parse(&text).map_err(|e| format!("{file}: {e}"))?;
    if report::Meta::from_json(&result).is_err() {
        let suite = suite.ok_or(format!("usage: {file} has no suite, so give --suite"))?;
        let meta = report::Meta::now(&suite, machine, commit, smoke);
        let Json::Obj(entries) = result else { return Err(format!("{file} is not a JSON object")) };
        let Json::Obj(mut head) = meta.to_json() else {
            unreachable!("Meta::to_json is an object")
        };
        head.extend(entries.into_iter().filter(|(k, _)| {
            !matches!(k.as_str(), "suite" | "machine" | "date" | "commit" | "smoke")
        }));
        result = Json::Obj(head);
    } else if suite.is_some() || machine.is_some() || commit.is_some() || smoke {
        return Err(format!(
            "usage: {file} names its run already, so --suite, --machine, --commit and --smoke do not apply"
        ));
    }
    let (md, json) = report::write(Path::new(&out), &result)?;
    println!("{}\n{}", md.display(), json.display());
    Ok(())
}

/// The `answers` command.
fn answers_command(mut a: args::Args) -> Result<(), String> {
    let expected = a.value("expected").ok_or("usage: answers needs --expected PATH")?;
    let actual = a.value("actual").ok_or("usage: answers needs --actual PATH")?;
    a.finish()?;
    let (expected, actual) = (Path::new(&expected), Path::new(&actual));
    let pairs: Vec<(PathBuf, PathBuf)> = if expected.is_dir() {
        let mut files: Vec<_> = std::fs::read_dir(expected)
            .map_err(|e| format!("{}: {e}", expected.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| matches!(p.extension().and_then(|x| x.to_str()), Some("tsv" | "out")))
            .collect();
        files.sort();
        files
            .into_iter()
            .map(|e| {
                let stem =
                    e.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                (e, actual.join(format!("{stem}.tsv")))
            })
            .collect()
    } else {
        vec![(expected.to_path_buf(), actual.to_path_buf())]
    };
    if pairs.is_empty() {
        return Err(format!("{} has no .tsv or .out file", expected.display()));
    }
    let mut wrong = 0;
    for (e, act) in &pairs {
        let read =
            |p: &Path| std::fs::read_to_string(p).map_err(|err| format!("{}: {err}", p.display()));
        let tpch = e.extension().is_some_and(|x| x == "out");
        let verdict = (|| {
            let want =
                if tpch { answers::parse_tpch(&read(e)?)? } else { answers::parse_tsv(&read(e)?)? };
            let got = answers::parse_tsv(&read(act)?)?;
            let tol = if tpch { answers::Tolerance::Tpch } else { answers::RELATIVE };
            answers::compare(&want, &got, tol).map(|()| want.rows.len())
        })();
        let name = e.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        match verdict {
            Ok(rows) => println!("{name:<8} ok, {rows} rows"),
            Err(err) => {
                wrong += 1;
                println!("{name:<8} WRONG: {err}");
            }
        }
    }
    if wrong > 0 { Err(format!("{wrong} of {} answers are wrong", pairs.len())) } else { Ok(()) }
}

/// A number option with a default.
fn number<T: std::str::FromStr>(a: &mut args::Args, name: &str, default: T) -> Result<T, String> {
    match a.value(name) {
        None => Ok(default),
        Some(v) => v.parse().map_err(|_| format!("usage: --{name} {v:?} is not a number")),
    }
}

/// The settings of `m1` and `m1-fill`.
fn m1_config(a: &mut args::Args, dir: PathBuf) -> Result<m1::Config, String> {
    let cores = std::thread::available_parallelism().map_or(1, usize::from);
    let seconds: f64 = number(a, "seconds", 30.0)?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err(format!("usage: --seconds {seconds} is not a number of seconds"));
    }
    let log_mib: u64 = number(a, "log-mib", 1024)?;
    let log_bytes = match a.value("log-bytes") {
        Some(v) => v.parse().map_err(|_| format!("usage: --log-bytes {v:?} is not a number"))?,
        None => log_mib << 20,
    };
    Ok(m1::Config {
        dir,
        warmup: number(a, "warmup", 1_000)?,
        commits: number(a, "commits", 10_000)?,
        writers: number(a, "writers", cores)?.max(1),
        seconds,
        log_bytes,
        rows: number(a, "rows", 256)?.max(1),
        value: number(a, "value", 2_000)?.max(20),
    })
}

/// The `m1` command: the M1 numbers of spec/23 section 23.4.
fn m1_command(mut a: args::Args) -> Result<(), String> {
    let dir = PathBuf::from(a.value("dir").ok_or("usage: m1 needs --dir DIR")?);
    let steps = a.value("steps").unwrap_or_else(|| m1::STEPS.join(","));
    let steps: Vec<&str> = steps.split(',').collect();
    if let Some(s) = steps.iter().find(|s| !m1::STEPS.contains(s)) {
        return Err(format!("usage: {s:?} is not a step of m1"));
    }
    let c = m1_config(&mut a, dir)?;
    let smoke = a.flag("smoke");
    let json = a.flag("json");
    let report_dir = a.value("report");
    let machine = a.value("machine");
    a.finish()?;
    std::fs::create_dir_all(&c.dir).map_err(|e| format!("{}: {e}", c.dir.display()))?;
    let pins = pins::Pins::load(concat!(env!("CARGO_MANIFEST_DIR"), "/pins.toml"))?;
    let rupg_commit = pins.get("rupg", "commit").unwrap_or_default().to_owned();
    let meta = report::Meta::now("m1", machine, Some(rupg_commit.chars().take(8).collect()), smoke);
    let mut out = Out { json: meta.to_json(), text: String::new() };
    out.text.push_str(&format!(
        "suite           m1
machine         {}
date            {}
",
        meta.machine, meta.date
    ));
    if smoke {
        out.text.push_str(
            "smoke run: it shows that the driver works, it is not a baseline
",
        );
    }
    out.add(
        "rupg_commit",
        rupg_commit.as_str(),
        &format!(
            "rupg            {} at {rupg_commit}
",
            rupg::VERSION
        ),
    );
    out.add("rupg_version", rupg::VERSION, "");
    out.add("cores", std::thread::available_parallelism().map_or(1, usize::from), "");
    out.add("os", std::env::consts::OS, "");
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    out.add("kernel", kernel.trim(), "");
    out.add("dir", c.dir.display().to_string(), &format!("directory       {}\n", c.dir.display()));
    let mut result = Ok(());
    for step in &steps {
        let r = match *step {
            "empty" => m1::empty(&c),
            "latency" => m1::latency(&c),
            "throughput" => m1::throughput(&c),
            _ => m1::recovery(&c),
        };
        match r {
            Ok((j, text)) => out.add(step, j, &text),
            Err(e) => {
                out.add("error", format!("{step}: {e}"), "");
                result = Err(format!("{step}: {e}"));
                break;
            }
        }
    }
    if let Some(dir) = &report_dir {
        let (md, json) = report::write(Path::new(dir), &out.json)?;
        let text =
            format!("report          {}\n                {}\n", md.display(), json.display());
        out.add("report", md.display().to_string(), &text);
    }
    out.print(json);
    result
}

/// `m1-fill`, the child of the recovery step of `m1`. It does not return.
fn m1_fill_command(mut a: args::Args) -> Result<(), String> {
    let file = PathBuf::from(a.value("file").ok_or("usage: m1-fill needs --file FILE")?);
    let c = m1_config(&mut a, PathBuf::new())?;
    a.finish()?;
    m1::fill(&file, &c)
}

/// The `pgbench` command: the pgbench run and its consistency check of spec/21 section 21.4.6.
fn pgbench_command(mut a: args::Args) -> Result<(), String> {
    // The role and the database that machines/install/postgresql.sh makes.
    let conn = pg::Config::parse(
        &a.value("conn").unwrap_or_else(|| "user=bench dbname=bench".to_owned()),
    )?;
    let bin = a.value("bin").unwrap_or_else(|| "/usr/lib/postgresql/19/bin".to_owned());
    let scale: u32 = number(&mut a, "scale", 1)?;
    let clients: u32 = number(&mut a, "clients", 4)?;
    let jobs: u32 = number(&mut a, "jobs", clients.min(4))?;
    let time: u32 = number(&mut a, "time", 60)?;
    let smoke = a.flag("smoke");
    let json = a.flag("json");
    let report_dir = a.value("report");
    let machine = a.value("machine");
    if a.value_is_set("name") {
        return Err(
            "usage: pgbench measures a running server, so give --unit or --attach".to_owned()
        );
    }
    let cg = target(&mut a)?;
    a.finish()?;
    let bin = Path::new(&bin);
    with_cgroup(cg, json, |cg, out| {
        let meta = report::Meta::now("pgbench", machine, None, smoke);
        out.add("suite", "pgbench", "suite           pgbench\n");
        out.add("machine", meta.machine.as_str(), &format!("machine         {}\n", meta.machine));
        out.add("date", meta.date.as_str(), &format!("date            {}\n", meta.date));
        out.add("commit", meta.commit.as_str(), &format!("harness commit  {}\n", meta.commit));
        let note = if smoke {
            "smoke run: it shows that the driver works, it is not a baseline\n"
        } else {
            ""
        };
        out.add("smoke", smoke, note);
        // machines/install/postgresql.sh writes the commit of the build next to bin.
        if let Ok(c) = std::fs::read_to_string(bin.join("../COMMIT")) {
            out.add("postgresql_commit", c.trim(), &format!("postgresql      {}\n", c.trim()));
        }
        let result = pgbench_run(cg, out, bin, &conn, [scale, clients, jobs, time]);
        if let Err(e) = &result {
            out.add("error", e.as_str(), "");
        }
        if let Some(dir) = &report_dir {
            // A failed run gets a report too, with its error.
            let (md, json) = report::write(Path::new(dir), &out.json)?;
            let text =
                format!("report          {}\n                {}\n", md.display(), json.display());
            out.add("report", md.display().to_string(), &text);
        }
        result
    })
}

/// The steps of the `pgbench` command after the header of the result.
fn pgbench_run(
    cg: &Cgroup,
    out: &mut Out,
    bin: &Path,
    conn: &pg::Config,
    [scale, clients, jobs, time]: [u32; 4],
) -> Result<(), String> {
    let version = pgbench::output({
        let mut c = std::process::Command::new(bin.join("pgbench"));
        c.arg("--version");
        c
    })?;
    out.add("pgbench_version", version.trim(), &format!("pgbench         {}\n", version.trim()));
    let settings = Json::obj()
        .with("scale", scale)
        .with("clients", clients)
        .with("jobs", jobs)
        .with("time_s", time);
    out.add(
        "settings",
        settings,
        &format!("settings        scale {scale}, {clients} clients, {jobs} threads, {time} s\n"),
    );

    let mut init = pgbench::command(bin, conn);
    init.args(["-i", "-q", "-s", &scale.to_string()]);
    pgbench::output(init)?;
    let mut db = pg::Conn::connect(conn)?;
    let server = db.server_version.clone();
    out.add("server_version", server.as_str(), &format!("server          {server}\n"));

    let base = cgroup::idle_base(cg, cgroup::IDLE_BASE)?;
    out.add("idle_base", base.to_json(), &format!("idle base over 10 s:\n{}", base.text()));

    let mut run = pgbench::command(bin, conn);
    run.args(["-c", &clients.to_string(), "-j", &jobs.to_string(), "-T", &time.to_string()]);
    let interval = cgroup::Interval::start(cg)?;
    let printed = pgbench::output(run);
    let usage = interval.finish()?;
    let printed = printed?;
    let summary = pgbench::parse_summary(&printed)?;
    let sj = Json::obj()
        .with("transactions", summary.transactions)
        .with("failed", summary.failed)
        .with("latency_average_ms", summary.latency_ms)
        .with("tps", summary.tps)
        .with("output", printed.as_str());
    out.add("pgbench", sj, &format!("pgbench output:\n{printed}"));
    out.add("run", usage.to_json(), &format!("server over the run:\n{}", usage.text()));

    let check = pgbench::Check::read(&mut db, summary.transactions)?;
    let failures = check.failures();
    let text = format!(
        "check           sum(abalance) {}, sum(tbalance) {}, sum(bbalance) {}, history rows {}, transactions {}: {}\n",
        check.abalance,
        check.tbalance,
        check.bbalance,
        check.history,
        check.transactions,
        if failures.is_empty() { "passed" } else { "FAILED" }
    );
    out.add("check", check.to_json(), &text);
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("wrong answer, the run is not a number: {}", failures.join("; ")))
    }
}

/// The `instructions` command.
fn instructions_command(mut a: args::Args) -> Result<(), String> {
    let queries = a.value("queries").ok_or("usage: instructions needs --queries PATH")?;
    let set_name = a.value("set").ok_or("usage: instructions needs --set NAME")?;
    if !set_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(format!(
            "usage: the set name {set_name:?} may have only letters, digits, _ and -"
        ));
    }
    let repeat: usize = number(&mut a, "repeat", 3)?;
    if repeat == 0 {
        return Err("usage: --repeat must be at least 1".to_owned());
    }
    let ratchet = a.value("ratchet");
    let save = a.value("save");
    let smoke = a.flag("smoke");
    let json = a.flag("json");
    let report_dir = a.value("report");
    let machine = a.value("machine");
    let bin = a.value("bin").unwrap_or_else(|| "/usr/lib/postgresql/19/bin".to_owned());
    let mut conn = pg::Config::parse(
        &a.value("conn").unwrap_or_else(|| "user=bench dbname=bench".to_owned()),
    )?;
    if let Some(f) = a.value("password-file") {
        let p = std::fs::read_to_string(&f).map_err(|e| format!("{f}: {e}"))?;
        conn.password = Some(p.trim().to_owned());
    }
    let engine_name = a.value("engine");
    let engine = if let Some(db) = a.value("duckdb") {
        let duckdb = a.value("duckdb-bin").unwrap_or_else(|| "duckdb".to_owned());
        let threads: u32 = number(&mut a, "threads", 1)?;
        let argv = [
            duckdb,
            "-readonly".to_owned(),
            db,
            "-cmd".to_owned(),
            format!("SET threads = {threads}"),
            "-c".to_owned(),
        ];
        instructions::Engine::InProcess(argv.to_vec())
    } else {
        if a.value_is_set("name") {
            return Err("usage: instructions counts a running server, so give --unit or --attach"
                .to_owned());
        }
        let cg = target(&mut a)?;
        let cgroup = cg
            .path
            .strip_prefix(cgroup::CGROUP_ROOT)
            .map_err(|_| format!("{} is not under {}", cg.path.display(), cgroup::CGROUP_ROOT))?
            .display()
            .to_string();
        let psql = Path::new(&bin).join("psql").display().to_string();
        let mut client = vec![
            psql,
            "-X".to_owned(),
            "-q".to_owned(),
            "-v".to_owned(),
            "ON_ERROR_STOP=1".to_owned(),
        ];
        client.extend(["-h".to_owned(), conn.host.clone(), "-p".to_owned(), conn.port.to_string()]);
        client.extend(["-U".to_owned(), conn.user.clone(), "-d".to_owned(), conn.dbname.clone()]);
        client.extend(["-o".to_owned(), "/dev/null".to_owned(), "-f".to_owned(), "-".to_owned()]);
        instructions::Engine::Server { cgroup, client, password: conn.password.clone() }
    };
    a.finish()?;
    let engine_name = match (&engine, engine_name) {
        (_, Some(n)) => n,
        (instructions::Engine::InProcess(_), None) => "duckdb".to_owned(),
        (instructions::Engine::Server { .. }, None) => {
            return Err("usage: give --engine NAME for a server, for example postgresql".to_owned());
        }
    };
    if !engine_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(format!(
            "usage: the engine name {engine_name:?} may have only letters, digits, _ and -"
        ));
    }
    let set = instructions::read_set(Path::new(&queries))?;
    if set.is_empty() {
        return Err(format!("{queries} has no query"));
    }
    let meta =
        report::Meta::now(&format!("instructions-{set_name}-{engine_name}"), machine, None, smoke);
    let mut result = meta.to_json();
    if smoke {
        result =
            result.with("note", "smoke run: it shows that the runner works, it is not the ratchet");
    }
    result = result
        .with("set", set_name.as_str())
        .with("engine_name", engine_name.as_str())
        .with("queries_from", queries.as_str())
        .with("repeat", repeat)
        .with(
            "kernel",
            std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default().trim(),
        )
        .with("engine", engine.to_json());
    let counts = instructions::count(&engine, &set, repeat, |line| {
        if !json {
            println!("{line}");
        }
    })?;
    result = result.with("counts", counts.to_json()).with("queries", counts.rows());
    let mut over = Vec::new();
    if let Some(file) = &ratchet {
        let text = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
        let doc = toml::Doc::parse(&text).map_err(|e| format!("{file}: {e}"))?;
        over = instructions::compare(&counts, &doc, &set_name)?;
        result = result.with(
            "ratchet",
            Json::obj().with("file", file.as_str()).with("over_budget", over.clone()),
        );
    }
    if let Some(file) = &save {
        let text = format!(
            "# Instruction counts of spec/21 section 21.14, written by rupg-bench instructions --save.\n{}",
            counts.ratchet_table(&set_name, &meta.machine, &meta.date)
        );
        std::fs::write(file, text).map_err(|e| format!("{file}: {e}"))?;
    }
    if let Some(dir) = &report_dir {
        let (md, js) = report::write(Path::new(dir), &result)?;
        if !json {
            println!("report   {}\n         {}", md.display(), js.display());
        }
    }
    if json {
        print!("{}", result.pretty());
    } else {
        println!("base     {:?} for SELECT 1", counts.base);
        println!(
            "total    {} net instructions in {} queries",
            counts.total(),
            counts.queries.len()
        );
        for line in &over {
            println!("OVER     {line}");
        }
    }
    if over.is_empty() {
        Ok(())
    } else {
        Err(format!("{} counts are over their budget", over.len()))
    }
}

/// The system of a suite: `--duckdb DB [--duckdb-bin B] [--threads N] [--cpus LIST]`, or `--engine NAME` with `--unit UNIT` or `--attach PATH` and `--conn WORDS`. It returns the name of the engine and the target.
fn suite_target(a: &mut args::Args) -> Result<(String, suite::Target), String> {
    let engine = a.value("engine");
    if let Some(db) = a.value("duckdb") {
        let bin = a.value("duckdb-bin").unwrap_or_else(|| "duckdb".to_owned());
        let threads = match a.value("threads") {
            Some(t) => Some(
                t.parse::<u32>().map_err(|_| format!("usage: --threads {t:?} is not a number"))?,
            ),
            None => None,
        };
        let target = suite::Target::DuckDb { bin, db: db.into(), threads, cpus: a.value("cpus") };
        return Ok((engine.unwrap_or_else(|| "duckdb".to_owned()), target));
    }
    let engine =
        engine.ok_or("usage: give --duckdb DB, or --engine NAME with --unit or --attach")?;
    if !engine.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(format!(
            "usage: the engine name {engine:?} may have only letters, digits, _ and -"
        ));
    }
    if a.value_is_set("name") {
        return Err("usage: a server runs in its own cgroup, so give --unit or --attach".to_owned());
    }
    let unit = a.value("unit");
    let cg = match (&unit, a.value("attach")) {
        (Some(u), None) => Cgroup::of_unit(u)?,
        (None, Some(p)) => Cgroup::attach(Path::new(&p))?,
        _ => return Err("usage: give one of --unit and --attach".to_owned()),
    };
    let conn = pg::Config::parse(
        &a.value("conn").unwrap_or_else(|| "user=bench dbname=bench".to_owned()),
    )?;
    Ok((engine, suite::Target::Server { conn, cg, unit }))
}

/// The `tpch` command.
fn tpch_command(mut a: args::Args) -> Result<(), String> {
    let tools = a.value("tools").ok_or("usage: tpch needs --tools DIR")?;
    let scale = a.value("scale").ok_or("usage: tpch needs --scale S")?;
    if scale.parse::<f64>().map_or(true, |s| s <= 0.0) {
        return Err(format!("usage: --scale {scale:?} is not a positive number"));
    }
    let data = a.value("data").ok_or("usage: tpch needs --data DIR")?;
    let steps = a.value("steps").unwrap_or_else(|| "gen,load,run".to_owned());
    let steps: Vec<&str> = steps.split(',').collect();
    if let Some(bad) = steps.iter().find(|s| !["gen", "load", "run"].contains(s)) {
        return Err(format!("usage: unknown step {bad:?}, the steps are gen, load and run"));
    }
    let runs: usize = number(&mut a, "runs", 3)?;
    if runs == 0 {
        return Err("usage: --runs must be at least 1".to_owned());
    }
    let cold = !a.flag("no-cold");
    let expected = a.value("answers");
    let save_answers = a.value("save-answers");
    let du = a.value("du");
    let smoke = a.flag("smoke");
    let json = a.flag("json");
    let report_dir = a.value("report");
    let machine = a.value("machine");
    let (engine, target) = suite_target(&mut a)?;
    a.finish()?;
    // dbgen and qgen run in other directories, so the paths must be absolute.
    let absolute = |p: &str| std::path::absolute(p).map_err(|e| format!("{p}: {e}"));
    let tools = tpch::Tools::new(&absolute(&tools)?)?;
    let data = absolute(&data)?;
    let data = data.as_path();
    let say = |line: &str| {
        if !json {
            println!("{line}");
        }
    };
    let meta = report::Meta::now(&format!("tpch-sf{scale}-{engine}"), machine, None, smoke);
    let mut result = meta.to_json();
    if smoke {
        result =
            result.with("note", "smoke run: it shows that the driver works, it is not a baseline");
    }
    result = result
        .with("scale", scale.as_str())
        .with("engine_name", engine.as_str())
        .with("engine", target.to_json())
        .with("steps", steps.iter().map(|s| (*s).to_owned()).collect::<Vec<String>>())
        .with("runs", runs)
        .with(
            "cold",
            if cold {
                "page cache dropped, and a unit restarted, before the first run of each query"
            } else {
                "no: the first run is not cold"
            },
        )
        .with(
            "kernel",
            std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default().trim(),
        );
    let outcome = tpch_steps(
        &tools,
        &scale,
        data,
        &steps,
        &target,
        &engine,
        (runs, cold),
        (expected.as_deref(), save_answers.as_deref(), du.as_deref()),
        &mut result,
        &say,
    );
    if let Err(e) = &outcome {
        result = result.with("error", e.as_str());
    }
    if let Some(dir) = &report_dir {
        let (md, js) = report::write(Path::new(dir), &result)?;
        say(&format!("report   {}\n         {}", md.display(), js.display()));
    }
    if json {
        print!("{}", result.pretty());
    }
    outcome
}

/// The steps of the `tpch` command. They add their numbers to `result`.
#[allow(clippy::too_many_arguments)]
fn tpch_steps(
    tools: &tpch::Tools,
    scale: &str,
    data: &Path,
    steps: &[&str],
    target: &suite::Target,
    engine: &str,
    (runs, cold): (usize, bool),
    (expected, save_answers, du): (Option<&str>, Option<&str>, Option<&str>),
    result: &mut Json,
    say: &dyn Fn(&str),
) -> Result<(), String> {
    let add = |result: &mut Json, key: &str, value: Json| {
        *result = std::mem::replace(result, Json::Null).with(key, value);
    };
    add(result, "tpch_tools", Json::from(tools.dbgen.display().to_string()));
    match target {
        suite::Target::DuckDb { bin, .. } => {
            let v = std::process::Command::new(bin)
                .arg("--version")
                .output()
                .map_err(|e| format!("{bin}: {e}"))?;
            add(result, "engine_version", Json::from(String::from_utf8_lossy(&v.stdout).trim()));
        }
        suite::Target::Server { conn, .. } => {
            let c = pg::Conn::connect(conn)?;
            add(result, "engine_version", Json::from(c.server_version.as_str()));
        }
    }
    if steps.contains(&"gen") {
        say(&format!("dbgen    scale {scale} into {}", data.display()));
        let start = std::time::Instant::now();
        let made = tools.generate(scale, data)?;
        let generated = if made {
            format!("{:.1} s", start.elapsed().as_secs_f64())
        } else {
            "the data was there".to_owned()
        };
        add(result, "dbgen", Json::from(generated));
    }
    let queries = tools.queries(scale, data)?;
    let qdir = data.join("queries");
    std::fs::create_dir_all(&qdir).map_err(|e| format!("{}: {e}", qdir.display()))?;
    for q in &queries {
        std::fs::write(qdir.join(format!("{}.sql", q.name)), format!("{}\n", q.sql))
            .map_err(|e| format!("{}: {e}", qdir.display()))?;
    }
    add(
        result,
        "queries",
        Json::from(format!("qgen -d -s {scale}, written to {}", qdir.display())),
    );
    if steps.contains(&"load") {
        say("load");
        let (usage, rows) = match target {
            suite::Target::DuckDb { bin, db, cpus, .. } => {
                for f in [
                    db.clone(),
                    db.with_extension("db.wal"),
                    PathBuf::from(format!("{}.wal", db.display())),
                ] {
                    let _ = std::fs::remove_file(f);
                }
                let script = data.join("load-duckdb.sql");
                std::fs::write(&script, tpch::duckdb_script(data))
                    .map_err(|e| format!("{}: {e}", script.display()))?;
                let cg = Cgroup::create(&format!("tpch-{engine}-load"), cpus.as_deref())?;
                let argv = [
                    bin.clone(),
                    db.display().to_string(),
                    "-f".to_owned(),
                    script.display().to_string(),
                ];
                let (status, usage) = cgroup::run(&cg, &argv)?;
                cg.remove()?;
                if !status.success() {
                    return Err(format!("the DuckDB load failed: {status}"));
                }
                (usage, None)
            }
            suite::Target::Server { conn, cg, .. } => {
                let mut c = pg::Conn::connect(conn)?;
                let interval = cgroup::Interval::start(cg)?;
                let rows = tpch::load_server(&mut c, data, say);
                let usage = interval.finish()?;
                (usage, Some(rows?))
            }
        };
        say(&format!("load     {:.1} s", usage.wall.as_secs_f64()));
        let mut load = Json::obj().with("usage", usage.to_json());
        if let Some(rows) = rows {
            load = load.with(
                "rows",
                Json::from(
                    rows.into_iter()
                        .map(|(t, n)| Json::obj().with("table", t).with("rows", n))
                        .collect::<Vec<Json>>(),
                ),
            );
        }
        let at_rest = match (du, target) {
            (Some(p), _) => Some(PathBuf::from(p)),
            (None, suite::Target::DuckDb { db, .. }) => Some(db.clone()),
            (None, suite::Target::Server { .. }) => None,
        };
        if let Some(p) = at_rest {
            load = load
                .with("du_path", p.display().to_string())
                .with("du_apparent_bytes", cgroup::du_apparent(&p)?);
        }
        add(result, "load", load);
    }
    if !steps.contains(&"run") {
        return Ok(());
    }
    if let Some(idle) = suite::idle_base(target, cgroup::IDLE_BASE)? {
        add(result, "idle_base", idle.to_json());
    }
    let settings = suite::Settings { runs, cold, cgroup_prefix: format!("tpch-{engine}") };
    let mut results = suite::run(target, &queries, &settings, say);
    answer_files(&mut results, save_answers, expected, None)?;
    let rows: Vec<Json> = results.iter().map(suite::QueryResult::to_json).collect();
    finish_suite(&results, rows, expected, result, say)
}

/// Saves the answers of the queries to `save`, as `.tsv` files, and checks them against `expected`: the `.out` files of the TPC-H kit, or `.tsv` files from an earlier `--save-answers`.
/// Saves the answers to `save` and checks them against `expected`. With `queries`, an answer in a `.tsv` file is checked with the rule of `answers::Determined` for its query, so the rows that tie at a `LIMIT` can differ.
fn answer_files(
    results: &mut [suite::QueryResult],
    save: Option<&str>,
    expected: Option<&str>,
    queries: Option<&[String]>,
) -> Result<(), String> {
    for (i, r) in results.iter_mut().enumerate() {
        let Some(actual) = &r.answer else { continue };
        if let Some(dir) = save {
            std::fs::create_dir_all(dir).map_err(|e| format!("{dir}: {e}"))?;
            let path = Path::new(dir).join(format!("{}.tsv", r.name));
            std::fs::write(&path, answers::to_tsv(actual))
                .map_err(|e| format!("{}: {e}", path.display()))?;
        }
        if let Some(dir) = expected {
            let out = Path::new(dir).join(format!("{}.out", r.name));
            let tsv = Path::new(dir).join(format!("{}.tsv", r.name));
            let (path, tol) = if out.is_file() {
                (out, answers::Tolerance::Tpch)
            } else {
                (tsv, answers::RELATIVE)
            };
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let want = if tol == answers::Tolerance::Tpch {
                answers::parse_tpch(&text)?
            } else {
                answers::parse_tsv(&text)?
            };
            let rule = match queries.and_then(|q| q.get(i)) {
                Some(sql) if tol == answers::RELATIVE => answers::Determined::of_query(sql),
                _ => answers::Determined::All,
            };
            if rule != answers::Determined::All {
                r.check_rule = Some(rule.rule());
            }
            r.check = Some(answers::compare_query(&want, actual, tol, &rule));
        }
    }
    Ok(())
}

/// Adds the totals and the rows of the queries to `result`. It fails when a query failed or gave a wrong answer.
fn finish_suite(
    results: &[suite::QueryResult],
    rows: Vec<Json>,
    expected: Option<&str>,
    result: &mut Json,
    say: &dyn Fn(&str),
) -> Result<(), String> {
    let t = suite::totals(results);
    for r in results {
        if let Some(Err(e)) = &r.check {
            say(&format!("{:<6} WRONG: {e}", r.name));
        }
    }
    say(&format!(
        "hot total {:.3} s, cold total {:.3} s, {} failed, {} of {} checked answers wrong",
        t.hot, t.cold, t.failed, t.wrong, t.checked
    ));
    *result = std::mem::replace(result, Json::Null)
        .with(
            "totals",
            Json::obj()
                .with("hot_s", t.hot)
                .with("cold_s", t.cold)
                .with("failed", t.failed)
                .with("checked", t.checked)
                .with("wrong", t.wrong)
                .with("answers_from", expected.map(str::to_owned)),
        )
        .with("results", Json::from(rows));
    if t.failed > 0 {
        return Err(format!("{} of {} queries failed", t.failed, results.len()));
    }
    if t.wrong > 0 {
        return Err(format!("wrong answer: {} of {} queries", t.wrong, t.checked));
    }
    Ok(())
}

/// The `ycsb` command.
fn ycsb_command(mut a: args::Args) -> Result<(), String> {
    let engine = a.value("engine").unwrap_or_else(|| "postgresql".to_owned());
    if !engine.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(format!(
            "usage: the engine name {engine:?} may have only letters, digits, _ and -"
        ));
    }
    let conn = pg::Config::parse(
        &a.value("conn").unwrap_or_else(|| "user=bench dbname=bench".to_owned()),
    )?;
    let records: u64 = number(&mut a, "records", 100_000)?;
    if records == 0 {
        return Err("usage: --records must be at least 1".to_owned());
    }
    let steps = a.value("steps").unwrap_or_else(|| "load,run".to_owned());
    let steps: Vec<&str> = steps.split(',').collect();
    if let Some(bad) = steps.iter().find(|s| !["load", "run"].contains(s)) {
        return Err(format!("usage: unknown step {bad:?}, the steps are load and run"));
    }
    let workloads = a.value("workloads").unwrap_or_else(|| "a,b,c,f".to_owned());
    let workloads =
        workloads.split(',').map(ycsb::Workload::get).collect::<Result<Vec<_>, String>>()?;
    let rows = a.value("rows").unwrap_or_else(|| "1,16,16x64".to_owned());
    let rows = rows.split(',').map(ycsb::Shape::parse).collect::<Result<Vec<_>, String>>()?;
    let time: u64 = number(&mut a, "time", 60)?;
    let sync = a.value("sync");
    if let Some(v) = &sync
        && v != "on"
        && v != "off"
    {
        return Err(format!("usage: --sync {v:?}: give on or off"));
    }
    let seed: u64 = number(&mut a, "seed", 1)?;
    let smoke = a.flag("smoke");
    let json = a.flag("json");
    let report_dir = a.value("report");
    let machine = a.value("machine");
    if a.value_is_set("name") {
        return Err("usage: a server runs in its own cgroup, so give --unit or --attach".to_owned());
    }
    let cg = target(&mut a)?;
    a.finish()?;
    let say = |line: &str| {
        if !json {
            println!("{line}");
        }
    };
    // A run with --sync names its report after the setting, so the runs with on and off do not write the same file.
    let name = match &sync {
        Some(v) => format!("ycsb-{engine}-sync-{v}"),
        None => format!("ycsb-{engine}"),
    };
    let meta = report::Meta::now(&name, machine, None, smoke);
    let mut result = meta.to_json();
    if smoke {
        result =
            result.with("note", "smoke run: it shows that the driver works, it is not a baseline");
    }
    result = result
        .with("engine_name", engine.as_str())
        .with("cgroup", cg.path.display().to_string())
        .with("records", records)
        .with("field_count", ycsb::FIELDS)
        .with("field_length", ycsb::FIELD_LENGTH)
        .with("distribution", "scrambled zipfian, constant 0.99")
        .with("time_s", time)
        .with("seed", seed)
        .with(
            "kernel",
            std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default().trim(),
        );
    let settings = (records, time, seed, sync);
    let outcome = ycsb_steps(&cg, &conn, &steps, &workloads, &rows, settings, &mut result, &say);
    if let Err(e) = &outcome {
        result = result.with("error", e.as_str());
    }
    if let Some(dir) = &report_dir {
        let (md, js) = report::write(Path::new(dir), &result)?;
        say(&format!("report   {}\n         {}", md.display(), js.display()));
    }
    if json {
        print!("{}", result.pretty());
    }
    let removed = cg.remove();
    outcome.and(removed)
}

/// The steps of the `ycsb` command. They add their numbers to `result`.
#[allow(clippy::too_many_arguments)]
fn ycsb_steps(
    cg: &Cgroup,
    conn: &pg::Config,
    steps: &[&str],
    workloads: &[ycsb::Workload],
    rows: &[ycsb::Shape],
    (records, time, seed, sync): (u64, u64, u64, Option<String>),
    result: &mut Json,
    say: &dyn Fn(&str),
) -> Result<(), String> {
    let add = |result: &mut Json, key: &str, value: Json| {
        *result = std::mem::replace(result, Json::Null).with(key, value);
    };
    let mut db = pg::Conn::connect(conn)?;
    add(result, "server_version", db.server_version.as_str().into());
    let setting = |db: &mut pg::Conn, name: &str| -> Result<String, String> {
        let r = db.query(&format!("SHOW {name}"))?;
        Ok(r.rows.first().and_then(|r| r.first()).cloned().flatten().unwrap_or_default())
    };
    let server_sync = setting(&mut db, "synchronous_commit")?;
    let used_sync = sync.clone().unwrap_or(server_sync.clone());
    add(
        result,
        "synchronous_commit",
        Json::obj()
            .with("server", server_sync.as_str())
            .with("run", used_sync.as_str())
            .with("set_by_driver", sync.is_some()),
    );
    say(&format!("server   {}, synchronous_commit {used_sync}", db.server_version));
    if steps.contains(&"load") {
        let interval = cgroup::Interval::start(cg)?;
        let started = std::time::Instant::now();
        let loaded = ycsb::load(&mut db, records, seed);
        let secs = started.elapsed().as_secs_f64();
        let usage = interval.finish()?;
        let n = loaded?;
        say(&format!("load     {n} records in {secs:.1} s"));
        add(
            result,
            "load",
            Json::obj()
                .with("records", n)
                .with("secs", (secs * 1000.0).round() / 1000.0)
                .with("statements", "CREATE TABLE, COPY FROM STDIN, VACUUM ANALYZE, CHECKPOINT")
                .with("server", usage.to_json()),
        );
    }
    if !steps.contains(&"run") {
        return Ok(());
    }
    let have = ycsb::count(&mut db)?;
    if have != records {
        return Err(format!(
            "usertable has {have} rows, not --records {records}. Run the load step"
        ));
    }
    let base = cgroup::idle_base(cg, cgroup::IDLE_BASE)?;
    add(result, "idle_base", base.to_json());
    let mut out = Vec::new();
    let mut failures = Vec::new();
    let mut row_number = 0;
    for w in workloads {
        for shape in rows {
            row_number += 1;
            let s = ycsb::RunSettings {
                workload: *w,
                shape: *shape,
                records,
                time: Duration::from_secs(time),
                seed,
                row: row_number,
                sync: sync.clone(),
            };
            let (mut tally, usage) = ycsb::run(conn, &s, cg)?;
            let check = ycsb::check_final(&mut db, std::mem::take(&mut tally.writes))?;
            let ops = tally.ops();
            let secs = tally.elapsed.as_secs_f64();
            let throughput = if secs > 0.0 { ops as f64 / secs } else { 0.0 };
            let cpu_per_op = (ops > 0).then(|| usage.cpu_usec as f64 / ops as f64);
            let mut row = Json::obj()
                .with("workload", w.name.to_string())
                .with("clients", shape.clients)
                .with("pipeline", shape.depth)
                .with("ops", ops)
                .with("secs", (secs * 1000.0).round() / 1000.0)
                .with("ops_per_s", throughput.round())
                .with("errors", tally.errors)
                .with("first_error", tally.first_error.clone())
                .with("server_cpu_usec_per_op", cpu_per_op.map(|v| (v * 10.0).round() / 10.0))
                .with("server_memory_peak", usage.memory_peak);
            for (op, name) in ycsb::OPS {
                let h = &tally.latency[op as usize];
                if h.count() > 0 {
                    row = row.with(name, h.to_json());
                }
            }
            if w.update > 0.0 || w.rmw > 0.0 {
                let hot = ycsb::ScrambledZipfian::new(records).hottest();
                row = row.with(
                    "hottest_key",
                    Json::obj()
                        .with("key", ycsb::key_name(hot))
                        .with("updates", tally.hot_updates)
                        .with(
                            "updates_per_s",
                            (tally.hot_updates as f64 / secs * 10.0).round() / 10.0,
                        ),
                );
            }
            row = row.with("server", usage.to_json()).with("check", check.to_json());
            let p99 = |op: ycsb::Op| {
                tally.latency[op as usize]
                    .quantile(0.99)
                    .map_or("-".to_owned(), |n| format!("{:.0}", n as f64 / 1000.0))
            };
            say(&format!(
                "{} {:<22} {:>10.0} ops/s  read p99 {} us  update p99 {} us  rmw p99 {} us  cpu {} us/op  errors {}",
                w.name,
                shape.name(),
                throughput,
                p99(ycsb::Op::Read),
                p99(ycsb::Op::Update),
                p99(ycsb::Op::Rmw),
                cpu_per_op.map_or("-".to_owned(), |v| format!("{v:.1}")),
                tally.errors
            ));
            say(&format!(
                "  check: {} acknowledged updates, {} fields checked, {} wrong",
                check.writes, check.fields, check.bad
            ));
            if check.bad > 0 {
                failures.push(format!(
                    "workload {} with {}: {} fields do not hold their last acknowledged update: {}",
                    w.name,
                    shape.name(),
                    check.bad,
                    check.examples.join("; ")
                ));
            }
            if tally.errors > 0 {
                failures.push(format!(
                    "workload {} with {}: {} errors, the first: {}",
                    w.name,
                    shape.name(),
                    tally.errors,
                    tally.first_error.unwrap_or_default()
                ));
            }
            out.push(row);
        }
    }
    add(result, "rows", out.into());
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("wrong answers, the rows are not numbers: {}", failures.join("; ")))
    }
}

/// The `tpcc` command.
fn tpcc_command(mut a: args::Args) -> Result<(), String> {
    let opt = std::env::var("RUPG_BENCH_OPT").unwrap_or_else(|_| "/opt/rupg-bench".to_owned());
    let hammerdb = PathBuf::from(a.value("hammerdb").unwrap_or_else(|| format!("{opt}/hammerdb")));
    let host = a.value("host").unwrap_or_else(|| "/var/run/postgresql".to_owned());
    let port: u16 = number(&mut a, "port", 5432)?;
    let password_file = PathBuf::from(
        a.value("password-file").unwrap_or_else(|| "/etc/rupg-bench/tpcc.pass".to_owned()),
    );
    let steps = a.value("steps").unwrap_or_else(|| "build,procedures,statements".to_owned());
    let steps: Vec<String> = steps.split(',').map(str::to_owned).collect();
    if let Some(bad) =
        steps.iter().find(|s| !["build", "check", "procedures", "statements"].contains(&s.as_str()))
    {
        return Err(format!(
            "usage: unknown step {bad:?}, the steps are build, check, procedures and statements"
        ));
    }
    let warehouses: u32 = number(&mut a, "warehouses", 10)?;
    let build_vu: u32 = number(&mut a, "build-vu", warehouses.clamp(1, 4))?;
    let vus = a.value("vu").unwrap_or_else(|| "8".to_owned());
    let vus = vus
        .split(',')
        .map(|v| {
            v.parse::<u32>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or(format!("usage: --vu {v:?} is not a count"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let rampup: u32 = number(&mut a, "rampup", 5)?;
    let duration: u32 = number(&mut a, "duration", 20)?;
    if warehouses == 0 || duration == 0 {
        return Err("usage: --warehouses and --duration must be at least 1".to_owned());
    }
    let syncs: Vec<Option<String>> = match a.value("sync") {
        None => vec![None],
        Some(v) => v
            .split(',')
            .map(|s| match s {
                "on" | "off" => Ok(Some(s.to_owned())),
                _ => Err(format!("usage: --sync {s:?}: give on, off or on,off")),
            })
            .collect::<Result<_, _>>()?,
    };
    let sync_dir = a.value("sync-dir").map(PathBuf::from);
    let work = PathBuf::from(
        a.value("work")
            .unwrap_or_else(|| std::env::temp_dir().join("rupg-bench-tpcc").display().to_string()),
    );
    let seed: u64 = number(&mut a, "seed", 1)?;
    let smoke = a.flag("smoke");
    let json = a.flag("json");
    let report_dir = a.value("report");
    let machine = a.value("machine");
    if a.value_is_set("name") {
        return Err("usage: a server runs in its own cgroup, so give --unit or --attach".to_owned());
    }
    let cg = target(&mut a)?;
    a.finish()?;
    let say = |line: &str| {
        if !json {
            println!("{line}");
        }
    };
    // The report names the warehouse count, because spec/20 section 20.8 runs two counts.
    let meta = report::Meta::now(&format!("tpcc-postgresql-w{warehouses}"), machine, None, smoke);
    let mut result = meta.to_json();
    if smoke {
        result =
            result.with("note", "smoke run: it shows that the driver works, it is not a baseline");
    }
    let hammerdb_version = std::fs::read_to_string(hammerdb.join("VERSION"))
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    result = result
        .with("engine_name", "postgresql")
        .with("cgroup", cg.path.display().to_string())
        .with("hammerdb_version", hammerdb_version)
        .with("rampup_min", rampup)
        .with("duration_min", duration)
        .with("seed", seed)
        .with("mix", "HammerDB: New-Order 10/23, Payment 10/23, Delivery, Stock-Level and Order-Status 1/23 each, no keying or think time")
        .with(
            "kernel",
            std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default().trim(),
        );
    let plan = TpccPlan {
        hammerdb,
        work,
        steps,
        warehouses,
        build_vu,
        vus,
        rampup,
        duration,
        syncs,
        sync_dir,
        seed,
    };
    let outcome = tpcc::Server::new(host, port, password_file)
        .and_then(|server| tpcc_steps(&cg, &server, &plan, &mut result, &say));
    if let Err(e) = &outcome {
        result = result.with("error", e.as_str());
    }
    if let Some(dir) = &report_dir {
        let (md, js) = report::write(Path::new(dir), &result)?;
        say(&format!("report   {}\n         {}", md.display(), js.display()));
    }
    if json {
        print!("{}", result.pretty());
    }
    let removed = cg.remove();
    outcome.and(removed)
}

/// The settings of the `tpcc` command.
#[derive(Debug)]
struct TpccPlan {
    hammerdb: PathBuf,
    work: PathBuf,
    steps: Vec<String>,
    warehouses: u32,
    build_vu: u32,
    vus: Vec<u32>,
    rampup: u32,
    duration: u32,
    syncs: Vec<Option<String>>,
    sync_dir: Option<PathBuf>,
    seed: u64,
}

/// The steps of the `tpcc` command. They add their numbers to `result`.
fn tpcc_steps(
    cg: &Cgroup,
    server: &tpcc::Server,
    plan: &TpccPlan,
    result: &mut Json,
    say: &dyn Fn(&str),
) -> Result<(), String> {
    let add = |result: &mut Json, key: &str, value: Json| {
        *result = std::mem::replace(result, Json::Null).with(key, value);
    };
    let has = |s: &str| plan.steps.iter().any(|x| x == s);
    let mut admin = pg::Conn::connect(&server.admin())?;
    add(result, "server_version", admin.server_version.as_str().into());
    let setting = |db: &mut pg::Conn, name: &str| -> Result<String, String> {
        let r = db.query(&format!("SHOW {name}"))?;
        Ok(r.rows.first().and_then(|r| r.first()).cloned().flatten().unwrap_or_default())
    };
    let server_sync = setting(&mut admin, "synchronous_commit")?;
    let data_dir = setting(&mut admin, "data_directory")?;
    say(&format!("server   {}, synchronous_commit {server_sync}", admin.server_version));
    if has("build") {
        say(&format!(
            "build    {} warehouses with {} virtual users",
            plan.warehouses, plan.build_vu
        ));
        admin.simple("DROP DATABASE IF EXISTS tpcc WITH (FORCE)")?;
        admin.simple("DROP ROLE IF EXISTS tpcc")?;
        let script = tpcc::build_script(server, plan.warehouses, plan.build_vu)?;
        let interval = cgroup::Interval::start(cg)?;
        let started = std::time::Instant::now();
        let out = tpcc::run_hammerdb(&plan.hammerdb, &plan.work, "build", &script, server, cg, say);
        let secs = started.elapsed().as_secs_f64();
        let usage = interval.finish()?;
        let out = out?;
        if !out.lines.iter().any(|l| l.contains("TPCC SCHEMA COMPLETE")) {
            return Err(format!("the HammerDB build did not complete, see {}", out.log.display()));
        }
        let size = admin.query("SELECT pg_database_size('tpcc')")?;
        let size: Option<u64> = size
            .rows
            .first()
            .and_then(|r| r.first().cloned().flatten())
            .and_then(|v| v.parse().ok());
        say(&format!("         done in {secs:.1} s"));
        add(
            result,
            "build",
            Json::obj()
                .with("warehouses", plan.warehouses)
                .with("virtual_users", plan.build_vu)
                .with("secs", (secs * 10.0).round() / 10.0)
                .with("database_bytes", size)
                .with("server", usage.to_json()),
        );
    }
    let mut db = pg::Conn::connect(&server.tpcc())?;
    let r = db.query("SELECT count(*) FROM warehouse")?;
    let warehouses: u32 = r
        .rows
        .first()
        .and_then(|r| r.first().cloned().flatten())
        .and_then(|v| v.parse().ok())
        .filter(|n| *n > 0)
        .ok_or("the database tpcc has no warehouses. Run the build step")?;
    add(result, "warehouses", warehouses.into());
    let mut failures = Vec::new();
    let conditions = tpcc::consistency(&mut db)?;
    let bad: u64 = conditions.iter().map(|c| c.failed).sum();
    say(&format!("check    before the runs: conditions 1 to 4, {bad} failures"));
    if bad > 0 {
        failures.push(format!("before the runs: {bad} units fail the consistency conditions"));
    }
    add(result, "check_before", tpcc::conditions_json(&conditions));
    let forms: Vec<&str> = ["procedures", "statements"].into_iter().filter(|f| has(f)).collect();
    if forms.is_empty() {
        return if failures.is_empty() { Ok(()) } else { Err(failures.join("; ")) };
    }
    let effective = |s: &Option<String>| s.clone().unwrap_or_else(|| server_sync.clone());
    let fsync = if plan.syncs.iter().any(|s| effective(s) != "off") {
        let dir = plan.sync_dir.clone().unwrap_or_else(|| {
            Path::new(&data_dir).parent().map_or(PathBuf::from(&data_dir), Path::to_path_buf)
        });
        let p50 = tpcc::fdatasync_p50(&dir, 200)?;
        say(&format!(
            "fsync    fdatasync p50 {:.1} us in {}",
            p50.as_secs_f64() * 1e6,
            dir.display()
        ));
        add(
            result,
            "fdatasync",
            Json::obj()
                .with("dir", dir.display().to_string())
                .with("rounds", 200u64)
                .with("write_bytes", 8192u64)
                .with("p50_us", (p50.as_secs_f64() * 1e7).round() / 10.0),
        );
        Some(p50)
    } else {
        None
    };
    let base = cgroup::idle_base(cg, cgroup::IDLE_BASE)?;
    add(result, "idle_base", base.to_json());
    let cores = tpcc::server_cores(cg);
    add(result, "server_cores", cores.into());
    let mut rows = Vec::new();
    for sync in &plan.syncs {
        let used = effective(sync);
        for &vu in &plan.vus {
            for form in &forms {
                say(&format!("run      {form}, {vu} virtual users, synchronous_commit {used}"));
                let mut row = Json::obj()
                    .with("form", *form)
                    .with("virtual_users", vu)
                    .with("synchronous_commit", used.as_str())
                    .with("synchronous_commit_set_by_driver", sync.is_some());
                let (nopm, tpm, usage, new_orders);
                if *form == "procedures" {
                    if let Some(v) = sync {
                        admin.simple(&format!("ALTER ROLE tpcc SET synchronous_commit = {v}"))?;
                    }
                    let script = tpcc::run_script(server, vu, plan.rampup, plan.duration)?;
                    let name = format!("run-{form}-{vu}-{used}");
                    let out = tpcc::run_hammerdb(
                        &plan.hammerdb,
                        &plan.work,
                        &name,
                        &script,
                        server,
                        cg,
                        say,
                    );
                    if sync.is_some() {
                        admin.simple("ALTER ROLE tpcc RESET synchronous_commit")?;
                    }
                    let out = out?;
                    let (n, t) = tpcc::parse_result(&out.lines).ok_or(format!(
                        "HammerDB printed no TEST RESULT, see {}",
                        out.log.display()
                    ))?;
                    let u = out.measured.ok_or(format!(
                        "HammerDB printed no start and end of the timed interval, see {}",
                        out.log.display()
                    ))?;
                    let profile = std::fs::read_to_string(out.work.join("hdbxtprofile.log"))
                        .map(|t| tpcc::parse_profile(&t))
                        .unwrap_or_default();
                    let mut times = Json::obj();
                    for (name, j) in profile {
                        times = times.with(&name, j);
                    }
                    row = row
                        .with(
                            "tpm_source",
                            "HammerDB TEST RESULT: pg_stat_database commits and rollbacks",
                        )
                        .with("time_profile", times)
                        .with(
                            "time_profile_note",
                            "HammerDB times each procedure call over the whole run, the ramp too",
                        );
                    new_orders = n as f64 * f64::from(plan.duration);
                    (nopm, tpm, usage) = (n as f64, t as f64, u);
                } else {
                    let s = tpcc::RunSettings {
                        warehouses,
                        terminals: vu,
                        rampup: Duration::from_secs(u64::from(plan.rampup) * 60),
                        duration: Duration::from_secs(u64::from(plan.duration) * 60),
                        seed: plan.seed,
                        sync: sync.clone(),
                    };
                    let run = tpcc::run_statements(&server.tpcc(), &s, cg)?;
                    let minutes = run.measured.as_secs_f64() / 60.0;
                    let mut times = Json::obj();
                    for (tx, name) in tpcc::TXS {
                        times = times.with(name, run.tally.latency[tx as usize].to_json());
                    }
                    row = row
                        .with(
                            "tpm_source",
                            "the transactions that the terminals finished in the interval",
                        )
                        .with("latency", times)
                        .with("new_order_rollbacks", run.tally.rolled_back)
                        .with("deliveries_skipped_districts", run.tally.deliveries_skipped)
                        .with("retries", run.tally.retries)
                        .with("errors", run.tally.errors)
                        .with("first_error", run.tally.first_error.clone());
                    if run.tally.errors > 0 {
                        failures.push(format!(
                            "statements with {vu} virtual users: {} errors, the first: {}",
                            run.tally.errors,
                            run.tally.first_error.clone().unwrap_or_default()
                        ));
                    }
                    new_orders = run.new_orders as f64;
                    nopm = run.new_orders as f64 / minutes;
                    tpm = run.tally.transactions() as f64 / minutes;
                    usage = run.usage;
                }
                let per_core = nopm / f64::from(cores);
                let cpu_per_no = (new_orders > 0.0).then(|| usage.cpu_usec as f64 / new_orders);
                let w_per_no =
                    usage.wbytes.filter(|_| new_orders > 0.0).map(|b| b as f64 / new_orders);
                let bound = fsync.filter(|_| used != "off").and_then(|p| tpcc::sync_bound(vu, p));
                let conditions = tpcc::consistency(&mut db)?;
                let bad: u64 = conditions.iter().map(|c| c.failed).sum();
                row = row
                    .with("nopm", nopm.round())
                    .with("tpm", tpm.round())
                    .with("nopm_per_core", per_core.round())
                    .with("server_cpu_usec_per_new_order", cpu_per_no.map(|v| v.round()))
                    .with("server_write_bytes_per_new_order", w_per_no.map(|v| v.round()))
                    .with("server_memory_peak", usage.memory_peak)
                    .with("sync_bound_nopm", bound.map(f64::round))
                    .with("server", usage.to_json())
                    .with("check", tpcc::conditions_json(&conditions));
                say(&format!(
                    "         {nopm:.0} NOPM ({per_core:.0} a core), {tpm:.0} TPM, cpu {} us a New-Order, sync bound {}",
                    cpu_per_no.map_or("-".to_owned(), |v| format!("{v:.0}")),
                    bound.map_or("-".to_owned(), |v| format!("{v:.0} NOPM"))
                ));
                say(&format!("         check: conditions 1 to 4, {bad} failures"));
                if bad > 0 {
                    failures.push(format!(
                        "{form} with {vu} virtual users, synchronous_commit {used}: {bad} units fail the consistency conditions"
                    ));
                }
                rows.push(row);
            }
        }
    }
    add(result, "rows", rows.into());
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("the run is not a result: {}", failures.join("; ")))
    }
}

/// The `fixed-set` command.
fn fixed_set_command(mut a: args::Args) -> Result<(), String> {
    let out = PathBuf::from(a.value("out").ok_or("usage: fixed-set needs --out DIR")?);
    let records: u64 = number(&mut a, "records", 100_000)?;
    let warehouses: u32 = number(&mut a, "warehouses", 2)?;
    let seed: u64 = number(&mut a, "seed", 1)?;
    a.finish()?;
    if records == 0 || warehouses == 0 {
        return Err("usage: --records and --warehouses must be at least 1".to_owned());
    }
    let files = [
        (out.join("ycsb"), "read1000.sql", ycsb::fixed_reads(records, 1000, seed)),
        (out.join("tpcc"), "neword100.sql", tpcc::fixed_new_orders(warehouses, 100, seed)),
    ];
    for (dir, name, text) in files {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join(name);
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("{}", path.display());
    }
    Ok(())
}

/// The `clickbench` command.
fn clickbench_command(mut a: args::Args) -> Result<(), String> {
    let dir = PathBuf::from(a.value("dir").ok_or("usage: clickbench needs --dir DIR")?);
    let engine = a.value("engine").ok_or("usage: clickbench needs --engine NAME")?;
    if !engine.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(format!(
            "usage: the engine name {engine:?} may have only letters, digits, _ and -"
        ));
    }
    let steps = a.value("steps").unwrap_or_else(|| "load,run,concurrent".to_owned());
    let steps: Vec<&str> = steps.split(',').collect();
    if let Some(bad) = steps.iter().find(|s| !["load", "run", "concurrent"].contains(s)) {
        return Err(format!("usage: unknown step {bad:?}, the steps are load, run and concurrent"));
    }
    let source = a.value("source");
    if steps.contains(&"load") && source.is_none() {
        return Err("usage: the load step needs --source FILE".to_owned());
    }
    let lib = a
        .value("lib")
        .map_or_else(|| dir.join("..").join("lib").join("benchmark-common.sh"), PathBuf::from);
    let unit = a.value("unit");
    let measure = match (unit, a.value("attach")) {
        (Some(u), None) => {
            clickbench::Measure::Server { path: Cgroup::of_unit(&u)?.path, unit: Some(u) }
        }
        (None, Some(p)) => {
            clickbench::Measure::Server { path: Cgroup::attach(Path::new(&p))?.path, unit: None }
        }
        (None, None) => clickbench::Measure::Own {
            prefix: format!("clickbench-{engine}"),
            cpus: a.value("cpus"),
        },
        _ => return Err("usage: give at most one of --unit and --attach".to_owned()),
    };
    let conn = a.value("conn");
    let duckdb_bin = a.value("duckdb-bin");
    let expected = a.value("answers");
    let save_answers = a.value("save-answers");
    let result_file = a.value("result-file");
    let smoke = a.flag("smoke");
    let json = a.flag("json");
    let report_dir = a.value("report");
    let machine = a.value("machine");
    a.finish()?;
    let system = clickbench::System::open(&dir)?;
    let say = |line: &str| {
        if !json {
            println!("{line}");
        }
    };
    let meta = report::Meta::now(&format!("clickbench-{engine}"), machine.clone(), None, smoke);
    let mut result = meta.to_json();
    if smoke {
        result =
            result.with("note", "smoke run: it shows that the driver works, it is not a baseline");
    }
    result = result
        .with("engine_name", engine.as_str())
        .with("dir", dir.display().to_string())
        .with("settings", system.env.to_json())
        .with("measure", measure.to_json())
        .with("steps", steps.iter().map(|s| (*s).to_owned()).collect::<Vec<String>>())
        .with(
            "kernel",
            std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default().trim(),
        );
    let answers_from = match (&measure, engine.as_str()) {
        (clickbench::Measure::Server { .. }, _) => Some(AnswersFrom::Server(pg::Config::parse(
            conn.as_deref().unwrap_or("user=bench dbname=test"),
        )?)),
        (clickbench::Measure::Own { .. }, "duckdb") => Some(AnswersFrom::DuckDb(vec![
            duckdb_bin.unwrap_or_else(|| "duckdb".to_owned()),
            "-readonly".to_owned(),
            dir.join("hits.db").display().to_string(),
        ])),
        _ => None,
    };
    let mut out = ClickbenchOut::default();
    let outcome = clickbench_steps(
        &system,
        &measure,
        &steps,
        (source.as_deref(), &lib),
        answers_from.as_ref(),
        (expected.as_deref(), save_answers.as_deref()),
        &mut result,
        &mut out,
        &say,
    );
    if let Err(e) = &outcome {
        result = result.with("error", e.as_str());
    }
    if let Some(path) = &result_file {
        let template_path = dir.join("template.json");
        let template = Json::parse(
            &std::fs::read_to_string(&template_path)
                .map_err(|e| format!("{}: {e}", template_path.display()))?,
        )?;
        let file = clickbench::result_file(
            &template,
            &report::today(),
            machine.as_deref().unwrap_or("unknown"),
            smoke,
            (out.load_time, out.data_size),
            out.concurrent,
            &out.times,
        )?;
        std::fs::write(path, file.pretty()).map_err(|e| format!("{path}: {e}"))?;
        say(&format!("result   {path}"));
    }
    if let Some(dir) = &report_dir {
        let (md, js) = report::write(Path::new(dir), &result)?;
        say(&format!("report   {}\n         {}", md.display(), js.display()));
    }
    if json {
        print!("{}", result.pretty());
    }
    outcome
}

/// Where the answers of an unmeasured pass come from.
enum AnswersFrom {
    Server(pg::Config),
    /// The `duckdb` program, its options and the database file.
    DuckDb(Vec<String>),
}

/// The numbers of the ClickBench result file.
#[derive(Default)]
struct ClickbenchOut {
    load_time: Option<f64>,
    data_size: Option<u64>,
    concurrent: Option<(Option<f64>, Option<f64>)>,
    times: Vec<Vec<Option<f64>>>,
}

/// The steps of the `clickbench` command, in the order of `bench_main` of the pin.
#[allow(clippy::too_many_arguments)]
fn clickbench_steps(
    system: &clickbench::System,
    measure: &clickbench::Measure,
    steps: &[&str],
    (source, lib): (Option<&str>, &Path),
    answers_from: Option<&AnswersFrom>,
    (expected, save_answers): (Option<&str>, Option<&str>),
    result: &mut Json,
    out: &mut ClickbenchOut,
    say: &dyn Fn(&str),
) -> Result<(), String> {
    let add = |result: &mut Json, key: &str, value: Json| {
        *result = std::mem::replace(result, Json::Null).with(key, value);
    };
    system.start()?;
    if steps.contains(&"load") {
        let source = Path::new(source.ok_or("the load step needs --source")?);
        let placed = system.place_source(source)?;
        say(&format!("load     {} as {}", source.display(), placed.display()));
        let (secs, usage) = system.load(measure)?;
        say(&format!("load     {secs:.1} s"));
        out.load_time = Some(secs);
        add(
            result,
            "load",
            Json::obj()
                .with("source", source.display().to_string())
                .with("load_time_s", secs)
                .with("usage", usage.to_json()),
        );
    }
    // The database of a server exists only after the load.
    match answers_from {
        Some(AnswersFrom::Server(conn)) => {
            let c = pg::Conn::connect(conn)?;
            add(result, "engine_version", Json::from(c.server_version.as_str()));
        }
        Some(AnswersFrom::DuckDb(argv)) => {
            let v = std::process::Command::new(&argv[0])
                .arg("--version")
                .output()
                .map_err(|e| format!("{}: {e}", argv[0]))?;
            add(result, "engine_version", Json::from(String::from_utf8_lossy(&v.stdout).trim()));
        }
        None => {}
    }
    if steps.contains(&"run") {
        if let clickbench::Measure::Server { .. } = measure {
            let cg = match measure {
                clickbench::Measure::Server { unit: Some(u), .. } => Cgroup::of_unit(u)?,
                clickbench::Measure::Server { path, .. } => Cgroup::attach(path)?,
                clickbench::Measure::Own { .. } => unreachable!(),
            };
            add(result, "idle_base", cgroup::idle_base(&cg, cgroup::IDLE_BASE)?.to_json());
        }
        let queries = system.queries()?;
        let mut results = Vec::new();
        let mut rows = Vec::new();
        for (i, sql) in queries.iter().enumerate() {
            let tries = system.run_query(measure, &format!("q{i}"), sql);
            let mut line = format!("{:<6}", tries.result.name);
            for t in &tries.times {
                match t {
                    Some(t) => line.push_str(&format!(" {t:>9.3}")),
                    None => line.push_str("      null"),
                }
            }
            if let Some(e) = &tries.result.error {
                line.push_str(&format!("  FAILED: {e}"));
            }
            say(&line);
            out.times.push(tries.times);
            results.push(tries.result);
        }
        let size = system.data_size()?;
        say(&format!("data-size {size}"));
        out.data_size = Some(size);
        add(result, "data_size_bytes", Json::from(size));
        if let Some(from) = answers_from {
            say("answers");
            for (r, sql) in results.iter_mut().zip(&queries) {
                if r.error.is_some() {
                    continue;
                }
                let answer = match from {
                    AnswersFrom::Server(conn) => pg::Conn::connect(conn)
                        .and_then(|mut c| {
                            c.query(sql).map(|rows| answers::Answer {
                                columns: rows.columns,
                                rows: rows.rows,
                            })
                        })
                        .map_err(String::from),
                    AnswersFrom::DuckDb(argv) => suite::duckdb_answer(argv.clone(), sql),
                };
                match answer {
                    Ok(a) => r.answer = Some(a),
                    Err(e) => r.error = Some(format!("the answer pass failed: {e}")),
                }
            }
            answer_files(&mut results, save_answers, expected, Some(&queries))?;
            if expected.is_some() {
                for (r, sql) in results.iter_mut().zip(&queries) {
                    if let Some(why) = clickbench::not_comparable(sql) {
                        r.check = None;
                        r.check_rule = Some(format!("not compared: {why}"));
                        say(&format!("{:<6} not compared: {why}", r.name));
                    }
                }
            }
        }
        for (r, t) in results.iter().zip(&out.times) {
            rows.push(
                r.to_json()
                    .with("tries_s", t.iter().map(|v| Json::from(*v)).collect::<Vec<Json>>()),
            );
        }
        if steps.contains(&"concurrent") {
            concurrent_step(system, measure, lib, result, out, say)?;
        }
        return finish_suite(&results, rows, expected, result, say);
    }
    if steps.contains(&"concurrent") {
        concurrent_step(system, measure, lib, result, out, say)?;
    }
    Ok(())
}

fn concurrent_step(
    system: &clickbench::System,
    measure: &clickbench::Measure,
    lib: &Path,
    result: &mut Json,
    out: &mut ClickbenchOut,
    say: &dyn Fn(&str),
) -> Result<(), String> {
    let duration = system.env.get("BENCH_CONCURRENT_DURATION").to_owned();
    say(&format!(
        "concurrent {} connections for {duration} s",
        system.env.get("BENCH_CONCURRENT_CONNECTIONS")
    ));
    let j = match system.concurrent(lib, measure)? {
        None => {
            say("concurrent off, BENCH_CONCURRENT_DURATION=0");
            out.concurrent = Some((None, None));
            Json::from("off: BENCH_CONCURRENT_DURATION=0")
        }
        Some(clickbench::Concurrent { qps, error_ratio: ratio, usage }) => {
            say(&format!(
                "concurrent {} queries per second, error ratio {}",
                qps.map_or("null".to_owned(), |v| format!("{v:.3}")),
                ratio.map_or("null".to_owned(), |v| format!("{v:.3}"))
            ));
            out.concurrent = Some((qps, ratio));
            Json::obj()
                .with("lib", lib.display().to_string())
                .with("qps", qps)
                .with("error_ratio", ratio)
                .with("usage", usage.to_json())
        }
    };
    *result = std::mem::replace(result, Json::Null).with("concurrent", j);
    Ok(())
}
