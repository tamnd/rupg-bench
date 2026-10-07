# rupg-bench

The benchmark harness for [rupg](https://github.com/tamnd/rupg).

[![ci](https://github.com/tamnd/rupg-bench/actions/workflows/ci.yml/badge.svg)](https://github.com/tamnd/rupg-bench/actions/workflows/ci.yml) [![license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)

This harness produces every performance number in a rupg report. It runs ClickBench, TPC-H, TPC-C, YCSB and pgbench against rupg and against PostgreSQL 19, ClickHouse, DuckDB, Umbra or CedarDB and SQLite, on the same machines, through the same transports, with the resources of each system measured in its own cgroup. It also holds the rules that decide what a number is allowed to claim.

It is a separate repository, in the way that rudb-bench is separate from rudb. It reaches every system through its client program, `libpq` or its library. It depends on rupg only through the `rupg` binary for the socket transport and the facade crate for the in-process transport, both at a pinned commit.

The procedure is [`spec/20-benchmarks.md`](https://github.com/tamnd/rupg/blob/main/spec/20-benchmarks.md), the targets are [`spec/02-the-goal.md`](https://github.com/tamnd/rupg/blob/main/spec/02-the-goal.md#210-when-a-gate-fails) section 2.10, and the published references are [`spec/03-baselines.md`](https://github.com/tamnd/rupg/blob/main/spec/03-baselines.md).

## Status

Early. The crate builds and CI is green, but no suite runs yet. `rupg-bench gates` prints the twelve gates of spec/02 section 2.10, `rupg-bench pins` prints the pinned versions, `rupg-bench measure` runs the cgroup v2 runner, and `rupg-bench load` runs the load cases. The first milestone, M0, builds the harness and measures every baseline that a later gate uses. Every command below is the planned interface from the spec.

## The gates

| Gate | Suite | Machine | Pass rule | Milestone |
|---|---|---|---|---|
| G2 | ClickBench | `4xl`, two instances | hot sum below ClickHouse's, measured in the same run | M4 |
| G3 | ClickBench | `4xl` | disk below 9.45 GB after load and checkpoint | M4 |
| G4 | YCSB A, B, C, F | `oltp` | 3x less server CPU per operation than PostgreSQL 19, 16 clients, no pipelining | M3 |
| G5 | TPC-C, statements | `oltp` | 3x NOPM per core of PostgreSQL 19, sync on | M5 |
| G6 | TPC-C, procedures | `oltp` | 10x NOPM per core of PostgreSQL 19, sync on | M7 |
| G7 | ClickBench | `4xl`, two instances | hot sum at most 1.753 s, cold sum at most 11.37 s | M8 |
| G8 | ClickBench | `4xl` | 10x lower peak memory and CPU time than each baseline, disk below 7.67 GB | M8 |
| G9 | TPC-H SF100 | `tpch` | hot total at most one tenth of DuckDB's | M8 |
| G10 | ClickBench | `metal`, two instances | hot sum at most 0.440 s | M8 |
| G11 | TPC-C | `cluster` | at least 0.8 N for N up to 16 | M11 |

G1 and G12 are compatibility gates and are decided in [rupg-compat](https://github.com/tamnd/rupg-compat).

## The suites

| Suite | What runs | Check on the answers |
|---|---|---|
| ClickBench | our fork at `e6bda4e`, 43 queries, three runs each, with true cold runs | each result equals PostgreSQL 19's, floats within a relative 1e-9 |
| TPC-H | `dbgen` 3.0.1 at SF1, SF10 and SF100, the 22 queries from `qgen -d` | the specification answer set at SF1, PostgreSQL 19 at SF10 and SF100 |
| TPC-C | HammerDB TPROC-C, procedures and statements, two warehouse counts, 5 minutes ramp and 20 minutes measured | consistency conditions 1 to 4 of clause 3.3.2 |
| YCSB | A, B, C and F, zipfian 0.99, 1 and 16 clients, and pipelined at a depth of 64 | each read returns the last acknowledged write |
| pgbench | a smoke test, never a gate | the balance sums and the history row count |
| Connections | 100, 1,000 and 10,000 connections, 60 s idle | none |
| Mixed | TPC-C with ClickBench on one server, and CH-benCHmark from M8 | the checks of each part |
| Scale out | TPC-C on N = 1, 2, 4, 8 and 16 nodes, ClickBench on N = 1, 2, 4 and 8 | the checks of each part |

A run whose check fails is not reported as a number. It is reported as a wrong-answer bug.

## Engines, pins and transports

| System | Version | How it runs | How rupg runs for that comparison |
|---|---|---|---|
| PostgreSQL 19 | `REL_19_STABLE` at `7d3d2db7`, then 19.0 | server, `psql` or `libpq` over a Unix socket | `rupg-server`, the same client over a Unix socket |
| ClickHouse | 26.9.12.8, the newest stable release on 7 October 2026 | server, `clickhouse-client` over TCP | `rupg-server`, `psql` over TCP |
| DuckDB | 1.5.6, then 2.0.0 when it is released | in process, the `duckdb` program | in process, the `rupg` program in embedded mode |
| Umbra or CedarDB | Umbra 26.09, CedarDB of 29 September 2026 | server, `psql` | `rupg-server`, the same |
| SQLite | 3.53.4 | in process | in process |

The exact pins are in [`pins.toml`](pins.toml), and `rupg-bench pins` prints them. HammerDB 6.0 and the TPC-H tools 3.0.1 are pinned there too. Umbra and CedarDB are used only where the license permits a published comparison. Each result records the exact version string of each system, the harness commit, the rupg commit, the configuration file, the kernel, the machine type and the date.

## The machines

| Name | Machine | Use |
|---|---|---|
| `4xl` | AWS `c6a.4xlarge`, 16 vCPU, 32 GiB, 500 GB gp2 | ClickBench, G2, G3, G7 and G8 |
| `metal` | AWS `c6a.metal`, 192 vCPU, 384 GiB | ClickBench, G10 |
| `tpch` | `4xl` for SF1 and SF10, and at least 64 vCPU, 256 GiB and local NVMe for SF100 | TPC-H, G9 |
| `oltp` | at least 32 cores, NVMe with power loss protection, `fdatasync` p50 below 100 µs | TPC-C, YCSB, connections, G4 to G6 |
| `cluster` | 16 `oltp` machines and 8 `c6a.4xlarge` | scale out, G11 |
| `gpc` | i9-13900K under WSL2 | daily runs, never a gate |
| `m4` | Apple M4 laptop | library and WebAssembly runs |

On `oltp` the server runs in a `cpuset` of 16 cores, and NOPM per core is NOPM divided by 16.

## Running it

The commands below are the planned interface. The harness is one Rust crate with a binary named `rupg-bench`, built with Rust 1.98.0.

```
cargo run --release -- machine up 4xl
cargo run --release -- clickbench --machine 4xl --rows full
cargo run --release -- tpch --scale 100 --machine tpch
cargo run --release -- tpcc --form procedures --machine oltp
cargo run --release -- ycsb --workload a --clients 16
cargo run --release -- connections --count 1000
cargo run --release -- load --case a
cargo run --release -- instructions
cargo run --release -- report
```

Each suite runs every system in its own cgroup v2 and reads `memory.peak`, `memory.stat`, `Pss`, `usage_usec`, `io.stat` and the bytes at rest, with an idle base over 10 s before each suite.

`rupg-bench measure` is this runner. With `--name` it starts a command in the new cgroup `/sys/fs/cgroup/rupg-bench/<name>`. With `--attach PATH` or `--unit UNIT` it measures a cgroup that exists, for example the cgroup of `postgresql@19-main.service`, while the client command runs outside it. `--idle` measures the idle base first, and `--du PATH` adds the bytes at rest. Before Linux 6.12 the kernel cannot reset `memory.peak`, so the peak counts from the creation of the cgroup, and the result says so. A driver then starts the system in a new cgroup for each suite.

```sh
rupg-bench measure --name duckdb --cpus 0-15 --du hits.db -- duckdb hits.db -c "SELECT count(*) FROM hits"
rupg-bench measure --unit postgresql@19-main --idle --du /var/lib/postgresql/19/main --json -- psql -c "SELECT 1"
```

`rupg-bench load` runs the load cases of spec/20 section 20.3.1 and then times the load command in the same runner. For case A it drops the caches, checks with `fincore` that no page of the source is resident, times `cat <source> > /dev/null` as the reference time `R`, drops the caches and checks again, and then times the load `L`. The result has `L`, `R` and `L / R`. For case B it reads the source twice, and `fincore` must report at least 99 percent of the pages resident. If fewer pages are resident, the run does the steps of case A and is recorded as case A. The load command must end with a checkpoint (spec/20 section 20.3.2). The command needs root, for `drop_caches`.

```sh
rupg-bench load --case a --source hits.parquet --name duckdb-load --du hits.db -- \
  duckdb hits.db -c "CREATE TABLE hits AS SELECT * FROM read_parquet('hits.parquet'); CHECKPOINT;"
```

## The reporting rules

1. A claim has five parts: one named machine, one metric, a baseline that we measured, the same transport on both sides, and the ratio. A claim without all five is not made.
2. Both sides are ours. A published number is a reference, and it is never one side of a ratio.
3. Durability is the same on both sides. A row with different durability is not published.
4. The gate row uses the default configuration. A row with more settings is labeled tuned and does not decide a gate.
5. A gate on `4xl` or `metal` passes only on two separate instances, and the worse number counts.
6. Each baseline that a gate used was measured within 30 days.
7. The load time includes every summary, the move to column segments and a checkpoint, and the run states whether it measured case A or case B.
8. A summary must never change an answer. G7 and G10 pass only when the row without summaries gives the same answers.
9. Misses are published with the same detail as passes, with the gap and the cause. A target never changes in the change that reports its miss.
10. A result that we later find wrong is corrected in a new report that names the old one, and the old report stays.
11. The ClickBench result is submitted only after the maintainers accept the summaries in the untuned category.

## Layout

```
pins.toml          versions of every system, the rupg commit
src/               suite drivers, the cgroup runner, the report generator
answers/           PostgreSQL 19 answers for ClickBench and TPC-H, TPC-H SF1 set
clickbench/        the rupg directory of our ClickBench fork
machines/          scripts for 4xl, metal, tpch, oltp and cluster
ratchet.toml       the instruction counts and the best number of each row
reports/<date>/    <commit>-<machine>-<suite>.md and a JSON file for each run
```

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). The milestones are issues with the label `kind/milestone`.

## License

Apache-2.0. See [LICENSE-APACHE](LICENSE-APACHE). The TPC-H and TPC-C numbers in this repository are not audited TPC results, and they are not comparable with published TPC results.
