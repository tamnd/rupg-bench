//! The benchmark harness for rupg.
//!
//! The harness runs rupg and the baselines on the same machine, with the same transport, and reports the ratio. The rules are in `spec/02-the-goal.md` and `spec/20-benchmarks.md` of tamnd/rupg.

#![forbid(unsafe_code)]

use std::path::Path;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cgroup::Cgroup;
use crate::json::Json;

mod answers;
mod args;
mod cgroup;
mod gates;
mod instructions;
mod json;
mod load;
mod pg;
mod pgbench;
mod pins;
mod report;
mod toml;

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
  instructions --queries PATH --set NAME (--duckdb DB [--duckdb-bin B] [--threads N] | --unit UNIT | --attach PATH)
          [--bin DIR] [--conn WORDS] [--repeat N] [--ratchet FILE] [--save FILE] [--smoke] [--json] [--report DIR [--machine M]]
                              count the instructions retired for each query with perf stat (spec/21 section 21.14).
                              PATH is a file with one query on each line or a directory of <name>.sql files.
                              --duckdb counts the duckdb process. --unit and --attach count the server cgroup on all
                              CPUs while psql from --bin sends the query. Each query runs --repeat times (3) and the
                              minimum counts, minus the count of SELECT 1. --ratchet compares with [instructions.NAME]
                              of a ratchet file and fails over the budgets (3 percent a query, 1 percent in total).
                              --save writes that table for this run.
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
        "pgbench" => pgbench_command(a)?,
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
    let pairs: Vec<(std::path::PathBuf, std::path::PathBuf)> = if expected.is_dir() {
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
    let conn = pg::Config::parse(
        &a.value("conn").unwrap_or_else(|| "user=bench dbname=bench".to_owned()),
    )?;
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
        client.extend(["-o".to_owned(), "/dev/null".to_owned(), "-c".to_owned()]);
        instructions::Engine::Server { cgroup, client }
    };
    a.finish()?;
    let set = instructions::read_set(Path::new(&queries))?;
    if set.is_empty() {
        return Err(format!("{queries} has no query"));
    }
    let meta = report::Meta::now(&format!("instructions-{set_name}"), machine, None, smoke);
    let mut result = meta.to_json();
    if smoke {
        result =
            result.with("note", "smoke run: it shows that the runner works, it is not the ratchet");
    }
    result = result
        .with("set", set_name.as_str())
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
    result = result.with("counts", counts.to_json());
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
