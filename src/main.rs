//! The benchmark harness for rupg.
//!
//! The harness runs rupg and the baselines on the same machine, with the same transport, and reports the ratio. The rules are in `spec/02-the-goal.md` and `spec/20-benchmarks.md` of tamnd/rupg. At M0 the binary prints the gates and the baselines.

#![forbid(unsafe_code)]

use std::process::ExitCode;

mod gates;

const USAGE: &str = "usage: rupg-bench <gates | --version>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("gates") => {
            for gate in gates::GATES {
                println!("{:<4} {:<4} {:<28} {}", gate.id, gate.milestone, gate.suite, gate.target);
            }
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") => {
            println!("rupg-bench {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
