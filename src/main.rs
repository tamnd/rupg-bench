//! The benchmark harness for rupg.
//!
//! The harness runs rupg and the baselines on the same machine, with the same transport, and reports the ratio. The rules are in `spec/02-the-goal.md` and `spec/20-benchmarks.md` of tamnd/rupg.

#![forbid(unsafe_code)]

use std::process::ExitCode;

mod args;
mod gates;
mod pins;
mod toml;

const USAGE: &str = "usage: rupg-bench <command> [options]

commands:
  gates                       print the twelve gates of spec/02 section 2.10
  pins [--file F] [--shell | --get SYSTEM.KEY]
                              print the pins of pins.toml, as shell assignments, or one value
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
        "--version" | "-V" => println!("rupg-bench {}", env!("CARGO_PKG_VERSION")),
        "--help" | "-h" | "help" => println!("{USAGE}"),
        other => return Err(format!("usage: unknown command {other:?}\n{USAGE}")),
    }
    Ok(())
}
