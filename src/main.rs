//! The benchmark harness for rupg.
//!
//! The harness runs rupg and the baselines on the same machine, with the same transport, and reports the ratio. The rules are in `spec/02-the-goal.md` and `spec/20-benchmarks.md` of tamnd/rupg.

#![forbid(unsafe_code)]

use std::path::Path;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cgroup::Cgroup;
use crate::json::Json;

mod args;
mod cgroup;
mod gates;
mod json;
mod load;
mod pins;
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
