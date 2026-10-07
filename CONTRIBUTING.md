# Contributing

This repository holds the benchmark harness, the baselines and the reports that decide the performance gates of rupg. The engine is in [tamnd/rupg](https://github.com/tamnd/rupg).

## Before you start

Read [`spec/02-the-goal.md`](https://github.com/tamnd/rupg/blob/main/spec/02-the-goal.md), [`spec/03-baselines.md`](https://github.com/tamnd/rupg/blob/main/spec/03-baselines.md) and [`spec/20-benchmarks.md`](https://github.com/tamnd/rupg/blob/main/spec/20-benchmarks.md) in the engine repository. They define the gates, the baselines and the rules for a run.

## The rules for a number

A number that does not have all five parts is not reported as a result.

1. **The machine.** One named machine type, recorded with the CPU, the kernel and the disk.
2. **The metric.** The metric of the suite. For ClickBench, the hot sum is the sum over the 43 queries of the minimum of three runs.
3. **The baseline.** A number that we measured on the same machine type, with the engine at a named version and a named configuration. A published number is a sanity check. It is not a baseline.
4. **The transport.** rupg and the baseline use the same transport: in process, a Unix socket, or TCP.
5. **The ratio.** The baseline number divided by the rupg number, against each baseline.

A run on a shared CI runner is not a measurement. CI checks only that the harness still works.

## Running the checks

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features
cargo test
```

## Writing style

The README, the reports and the issues use ASD-STE100 technical English. Write short sentences in the active voice. Use one line for each paragraph. Do not use the em dash or the en dash.

## License

By contributing, you agree that your contribution is licensed under Apache-2.0. Do not copy code under the AGPL or another copyleft license into this repository.
