//! The benchmark harness for rupg.
//!
//! The harness runs rupg and the baselines on the same machine, with the same transport, and reports the ratio. The rules are in `spec/02-the-goal.md` and `spec/20-benchmarks.md` of tamnd/rupg.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cgroup::Cgroup;
use crate::json::Json;

mod args;
mod cgroup;
mod gates;
mod json;
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
        "--version" | "-V" => println!("rupg-bench {}", env!("CARGO_PKG_VERSION")),
        "--help" | "-h" | "help" => println!("{USAGE}"),
        other => return Err(format!("usage: unknown command {other:?}\n{USAGE}")),
    }
    Ok(())
}

/// The `measure` command.
fn measure(mut a: args::Args) -> Result<(), String> {
    let name = a.value("name");
    let attach = a.value("attach");
    let unit = a.value("unit");
    let cpus = a.value("cpus");
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
    a.finish()?;
    let cg = match (name, attach, unit) {
        (Some(n), None, None) => {
            if idle.is_some() {
                return Err("usage: --idle needs --attach or --unit, because a new cgroup has nothing in it to measure".to_owned());
            }
            if argv.is_empty() {
                return Err("usage: --name needs a command after --".to_owned());
            }
            Cgroup::create(&n, cpus.as_deref())?
        }
        (None, Some(p), None) => Cgroup::attach(Path::new(&p))?,
        (None, None, Some(u)) => Cgroup::of_unit(&u)?,
        _ => return Err("usage: give one of --name, --attach and --unit".to_owned()),
    };
    if cpus.is_some() && !cg.owned() {
        return Err("usage: --cpus works only with --name".to_owned());
    }
    let mut out = Json::obj()
        .with("cgroup", cg.path.display().to_string())
        .with("unix_time", SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()))
        .with(
            "kernel",
            std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default().trim(),
        )
        .with("cpus", cpus.clone());
    let mut text = format!("cgroup          {}\n", cg.path.display());
    if let Some(d) = idle {
        let base = cgroup::idle_base(&cg, d)?;
        text.push_str(&format!("idle base over {} s:\n{}", d.as_secs_f64(), base.text()));
        out = out.with("idle_base", base.to_json());
    }
    if !argv.is_empty() {
        // In a new cgroup the command is the system. With --attach or --unit the command is the client, and it runs outside the cgroup that is measured.
        let mut cmd = if cg.owned() {
            cg.command(&argv)?
        } else {
            let mut c = std::process::Command::new(&argv[0]);
            c.args(&argv[1..]);
            c
        };
        let interval = cgroup::Interval::start(&cg)?;
        let status = cmd.status().map_err(|e| format!("{}: {e}", argv[0]));
        let usage = interval.finish()?;
        let status = status?;
        text.push_str(&format!(
            "command         {}\nexit            {status}\n{}",
            argv.join(" "),
            usage.text()
        ));
        out = out
            .with("command", argv.clone())
            .with("exit_code", status.code().map(i64::from))
            .with("run", usage.to_json());
        if cg.owned() {
            cg.kill_all()?;
        }
        if !status.success() {
            emit(json, &out, &text);
            cg.remove()?;
            return Err(format!("the command failed: {status}"));
        }
    }
    if let Some(path) = du {
        let bytes = cgroup::du_apparent(Path::new(&path))?;
        text.push_str(&format!("du {path}  {}\n", cgroup::mib(bytes)));
        out = out.with("du", Json::obj().with("path", path).with("apparent_bytes", bytes));
    }
    emit(json, &out, &text);
    cg.remove()
}

fn emit(json: bool, out: &Json, text: &str) {
    if json {
        print!("{}", out.pretty());
    } else {
        print!("{text}");
    }
}
