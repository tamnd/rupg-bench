# rupg-bench

The benchmark harness for [rupg](https://github.com/tamnd/rupg).

[![ci](https://github.com/tamnd/rupg-bench/actions/workflows/ci.yml/badge.svg)](https://github.com/tamnd/rupg-bench/actions/workflows/ci.yml) [![license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)

This harness produces every performance number in a rupg report. It runs ClickBench, TPC-H, TPC-C, YCSB and pgbench against rupg and against PostgreSQL 19, ClickHouse, DuckDB, Umbra or CedarDB and SQLite, on the same machines, through the same transports, with the resources of each system measured in its own cgroup. It also holds the rules that decide what a number is allowed to claim.

It is a separate repository, in the way that rudb-bench is separate from rudb. It reaches every system through its client program, `libpq` or its library. It depends on rupg only through the `rupg` binary for the socket transport and the facade crate for the in-process transport, both at a pinned commit.

The procedure is [`spec/20-benchmarks.md`](https://github.com/tamnd/rupg/blob/main/spec/20-benchmarks.md), the targets are [`spec/02-the-goal.md`](https://github.com/tamnd/rupg/blob/main/spec/02-the-goal.md#210-when-a-gate-fails) section 2.10, and the published references are [`spec/03-baselines.md`](https://github.com/tamnd/rupg/blob/main/spec/03-baselines.md).

## Status

Early. The crate builds and CI is green. ClickBench, TPC-H, TPC-C, YCSB and pgbench run, and the other suites do not run yet. `rupg-bench gates` prints the twelve gates of spec/02 section 2.10, `rupg-bench pins` prints the pinned versions, `rupg-bench measure` runs the cgroup v2 runner, `rupg-bench load` runs the load cases, `rupg-bench answers` checks answer sets, `rupg-bench report` writes the report files, `rupg-bench pgbench` runs the pgbench smoke run with its check, `rupg-bench instructions` counts the instructions of each query, `rupg-bench tpch` runs TPC-H, `rupg-bench ycsb` runs YCSB, `rupg-bench tpcc` runs TPC-C, and `rupg-bench clickbench` runs ClickBench. The first milestone, M0, builds the harness and measures every baseline that a later gate uses. Every command below is the planned interface from the spec.

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
| YCSB | A, B, C and F, zipfian 0.99, 1 and 16 clients, and pipelined at a depth of 64 | each read returns one row of its key, each update changes one row, and after the run each updated field holds its last acknowledged write |
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
cargo run --release -- tpcc --unit postgresql@19-main --steps procedures,statements --sync on,off
cargo run --release -- ycsb --unit postgresql@19-main --workloads a --rows 16
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

`rupg-bench answers` compares two answer sets as multisets of rows (spec/20 section 20.5). An answer set is a directory with one file `<query>.tsv` for each query: a header line with the column names, then the rows in the text format of PostgreSQL `COPY`, with `\N` for NULL. So `COPY (<query>) TO STDOUT WITH (HEADER)` writes one. Floats match within a relative 1e-9. An expected file `<query>.out` is a file of the TPC-H answer set at SF1, and its values match with the rules of TPC-H clause 2.1.3.5: equal column values and counts, sums within 100, and averages and ratios within 1 percent. The TPC-H tools are under the TPC EULA, so the harness fetches the answer set at the pin of `pins.toml` and this repository does not hold a copy.

```sh
rupg-bench answers --expected tpch_tools_3.0.1/dbgen/answers --actual out/
```

`rupg-bench report` writes `reports/<date>/<commit>-<machine>-<suite>.json` and a Markdown file with the same name from the `--json` output of a command. The Markdown file comes from the JSON file only. A smoke run has `-smoke` at the end of its name and a line at the top that says that its numbers are not baselines.

```sh
rupg-bench measure --unit cron.service --idle --json > idle.json
rupg-bench report --result idle.json --suite idle-base --machine server3 --smoke
```

`rupg-bench pgbench` runs the smoke test of spec/21 section 21.4.6 against a running PostgreSQL server. It runs `pgbench -i` at the scale, measures the idle base of the server cgroup over 10 s, and runs the TPC-B like script of pgbench for a fixed time while it measures the server cgroup. Then it checks that the sums of `abalance`, `tbalance` and `bbalance` are equal, and that `pgbench_history` has one row for each transaction that pgbench reports. A run that fails the check is a wrong answer and the command fails. The harness talks to the server with its own small client for protocol 3.0, so it needs no `libpq`. `machines/install/postgresql.sh` makes the role and the database `bench`, allows that role on the Unix socket with no password, and turns on the io accounting of the unit.

```sh
rupg-bench pgbench --unit postgresql@19-main --scale 1 --clients 4 --time 30 --smoke --report reports --machine server3
```

`rupg-bench instructions` is the instruction count runner of spec/21 section 21.14. It runs each query of a set under `perf stat -e instructions`. For DuckDB, perf counts the `duckdb` process with one thread. For a server, perf counts the cgroup of the server on all CPUs while `psql` reads the query on its standard input and sends each statement, so the count is the work of the server and not the work of the client. `--password-file` gives `psql` a password in `PGPASSWORD`. Each query runs three times and the minimum counts. The count of `SELECT 1` in the same way is subtracted, so the net count is the work of the query. A set is a file with one query on each line, named `q0` to `q42` for ClickBench, or a directory of `<name>.sql` files. A server needs `--engine NAME`, and the report name has the set and the engine. `--ratchet ratchet.toml` fails when one query rises by more than 3 percent or the total by more than 1 percent. `--save` writes the table for the ratchet. The ratchet is filled only on the dedicated runner, so [`ratchet.toml`](ratchet.toml) has no table yet. `rupg-bench fixed-set --out DIR` writes the YCSB and TPC-C parts of the fixed set: `DIR/ycsb/read1000.sql` has 1,000 point reads with scrambled zipfian keys, and `DIR/tpcc/neword100.sql` has 100 New-Orders in plain statements, with 5 + 2n statements each. The ClickBench part is the 43 queries on the first 1,000,000 rows of `hits`, and the TPC-H part is the 22 queries of `qgen -d` at SF 0.1. Each New-Order set adds 100 orders, so the database grows a little with each run.

```sh
rupg-bench instructions --set clickbench --queries queries.sql --duckdb hits.db --duckdb-bin duckdb
rupg-bench instructions --set clickbench --queries queries.sql --engine postgresql --unit postgresql@19-main --ratchet ratchet.toml
rupg-bench fixed-set --out fixed
rupg-bench instructions --set tpcc --queries fixed/tpcc --engine postgresql --unit postgresql@19-main --conn "user=tpcc dbname=tpcc" --password-file /etc/rupg-bench/tpcc.pass
```

`rupg-bench tpch` is the TPC-H driver of spec/20 section 20.7. `machines/install/tpch-tools.sh` builds `dbgen` and `qgen` 3.0.1 from the pin. The step `gen` runs `dbgen` into the data directory and writes the 22 queries of `qgen -d` to `DATA/queries`. Q15 uses the approved variant A of the kit (a `WITH` clause in place of the view). The step `load` creates the 8 tables with the types, primary keys and foreign keys of the specification and loads the same `.tbl` files into each system. For a server it adds the keys after the rows, then runs `VACUUM ANALYZE` and `CHECKPOINT`. The step `run` runs each query 3 times. The first run is cold: the driver drops the page cache, and for a server it also restarts the unit. The hot time of a query is the smallest of the other runs. Each query runs in its own cgroup, or in the cgroup of the server unit, and the result has the counters of each run. `--answers` checks the answers against the `.out` files of the kit at SF1, or against `.tsv` files from `--save-answers` of another run.

```sh
rupg-bench tpch --tools tpch_tools_3.0.1/dbgen --scale 1 --data sf1 --duckdb sf1.db --answers tpch_tools_3.0.1/dbgen/answers
rupg-bench tpch --tools tpch_tools_3.0.1/dbgen --scale 1 --data sf1 --engine postgresql --unit postgresql@19-main --answers tpch_tools_3.0.1/dbgen/answers
```

`rupg-bench ycsb` is the YCSB driver of spec/20 section 20.9. It follows the core workload of YCSB: the table `usertable` with ten text fields of 100 bytes, keys made from the FNV hash of the record number, and the scrambled zipfian distribution with the constant 0.99. The step `load` makes the table and loads `--records` rows with `COPY`, then runs `VACUUM ANALYZE` and `CHECKPOINT`. The step `run` runs each workload of `--workloads` (A, B, C and F) for each row of `--rows` for `--time` seconds. A row is a client count, such as `16`, or a client count and a pipeline depth, such as `16x64`. Each client has its own connection and thread, and sends prepared statements with the extended protocol. With a pipeline, up to that many statements wait for their results on each connection, each with its own Sync, as in the pipeline mode of `libpq`. The server cgroup is measured over each row, with the idle base first, so the result has the server CPU for each operation and `memory.peak`. It also has the throughput, the p50, p95, p99 and p99.9 latency of each operation, and the update rate of the hottest key. `--sync on` or `--sync off` sets `synchronous_commit` for each connection, for the two durability rows. Each read must return one row of its key, and each update must change one row. After each row the driver reads every field that the row updated, and the field must hold an update that no later acknowledged update replaced. A row that fails a check is a wrong answer and not a number.

```sh
rupg-bench ycsb --unit postgresql@19-main --records 1000000 --rows 1,16,16x64 --time 60 --sync on
```

`rupg-bench tpcc` is the TPC-C driver of spec/20 section 20.8. It has two forms on the schema that HammerDB builds. The form `procedures` runs HammerDB TPROC-C, which calls the stored procedures that HammerDB makes in the database. The form `statements` is a driver in the harness, because HammerDB has no form with plain statements for PostgreSQL. Its terminals send prepared statements, so a New-Order makes 5 + 2n round trips: the customer and warehouse read with `BEGIN`, the district update, the insert into `orders`, the insert into `new_order`, a read of each item and an update of each stock row, and one insert of all order lines with `COMMIT`. The items are sorted, so two New-Orders do not wait on each other in a cycle, and 1 percent of New-Orders roll back on an unused item. The mix is the HammerDB mix, with no keying or think time. The step `build` drops the database `tpcc` and builds `--warehouses` with HammerDB. Each run has a ramp of `--rampup` minutes and counts `--duration` minutes, for each count of `--vu` and each value of `--sync`. NOPM is the change of `sum(d_next_o_id)` over the counted minutes, as HammerDB counts it. The server cgroup is measured over the same minutes, with the idle base first, so the result has NOPM per core, the server CPU and the device write bytes for each New-Order, and `memory.peak`. The statement form gives the p50, p95 and p99 of each transaction, and the procedure form gives the time profile of HammerDB. For a run with `synchronous_commit` on, the result prints the sync bound `0.45 x 60 x T / s`, with `s` the `fdatasync` p50 that the driver measures next to the data directory. The conditions 1 to 4 of clause 3.3.2 are checked before the runs and after each run, and a run that fails one is not a result. The step `check` only checks them. HammerDB logs in with a password, so run `machines/install/hammerdb.sh` and `machines/install/tpcc-roles.sh` first.

```sh
rupg-bench tpcc --unit postgresql@19-main --warehouses 100 --vu 16 --rampup 5 --duration 20 --sync on,off
```

`rupg-bench clickbench` is the ClickBench driver of spec/20 section 20.5. It runs the scripts of one system directory of the ClickBench pin, in a copy of that directory next to a copy of `lib/`, because the scripts write the data and the results in their directory. `machines/install/clickbench.sh` fetches the pin with only `lib/` and the system directories. The driver does what `bench_main` of the pin does, in the same order: `./start`, `./load` on `--source` and a `sync`, then each query of `queries.sql` with the tries of `bench_run_query`, then `./data-size`, then the concurrent test `bench_concurrent_qps` of the pin. Before the first try of each query it does the cold cycle of the pin: `./stop`, `drop_caches` and `./start` for a server, and `drop_caches` only for a system with `BENCH_RESTARTABLE=no`. The settings come from the `export` lines of `benchmark.sh`, and the environment overrides a setting of the form `${NAME:-default}`, for example `BENCH_CONCURRENT_DURATION`. With `--unit` or `--attach` the driver measures the server cgroup, with the idle base first. With neither, each try runs in a new cgroup. The `query` script prints the result in the format of its client, so the answers come from a second pass that is not timed: through the PostgreSQL protocol for a server, or with `duckdb` on `hits.db`. `--save-answers` keeps them, and `--answers` checks them against the answers of another engine, as spec/20 section 20.5 checks every engine against PostgreSQL 19. `--result-file` writes the result in the format of ClickBench from `template.json`. The PostgreSQL scripts of the pin need `PGVERSION=19`.

```sh
PGVERSION=19 rupg-bench clickbench --dir cb/postgresql --engine postgresql --unit postgresql@19-main --source hits.tsv --save-answers ans-pg
rupg-bench clickbench --dir cb/duckdb --engine duckdb --source hits.parquet --duckdb-bin /opt/rupg-bench/bin/duckdb --answers ans-pg
```

The report of a YCSB run with `--sync` has the setting in its name, as in `ycsb-postgresql-sync-on`, and the report of a TPC-C run has the warehouse count, as in `tpcc-postgresql-w100`. So the runs of one suite do not write the same file.

## The machine scripts

The scripts in `machines/` start a machine, install the pins, run a suite and stop the machine. They read every version from `pins.toml`. Each install script in `machines/install/` checks the hash of what it downloads against the pin and checks the version that the program reports. `clickhouse.sh` copies the binary of the pinned release to `/opt/rupg-bench/bin`, and with `--system` it also runs `clickhouse install`, so the install script of the ClickBench pin finds `/usr/bin/clickhouse` and does not fetch the newest build. `sqlite.sh` checks the SHA3-256 hash that sqlite.org publishes. `umbra.sh` pulls the pinned image tag and stops if its digest is not the digest of `pins.toml`. The installer of CedarDB always installs the newest release, so `cedardb.sh` cannot pin it: it keeps the installer, records the version that the server reports and warns when that is not the pin. The licenses of Umbra and CedarDB are not verified, so their results are not published until they are. `harness.sh` builds `rupg-bench` with the pinned Rust.

`machines/run-suite.sh SUITE [MACHINE]` runs on the machine itself. It builds the harness, installs what the suite needs and runs the driver, and the reports go to `reports/`. The suites are `pgbench`, `tpch`, `tpcc`, `ycsb`, `clickbench` and `fdatasync`. `fdatasync` runs `fio` with the settings of spec/20 section 20.2 and prints the p50, which decides if a machine can be `oltp`. `SMOKE=1` runs a tiny scale and marks every report as a smoke run. spec/20 leaves the TPC-C warehouse counts, the TPC-C virtual users and the YCSB record count to the choice at M0, so a full run stops until `WAREHOUSES`, `VU` and `RECORDS` are set. `SERVER_CPUS` puts the server in a `cpuset` and `CLIENT_CPUS` puts the driver on the other cores, as on `oltp`.

`machines/instance.sh NAME SUITE [KEY=VALUE...]` starts the AWS machine `NAME` (`4xl`, `metal`, `tpch` or `oltp`) with Ubuntu 24.04 and a 500 GB gp2 root volume, copies the repository to it, runs `run-suite.sh` with the settings, copies the reports back and terminates the machine on exit, also after an error. It needs `AWS_REGION`, `KEY_NAME`, `SSH_KEY` and `SECURITY_GROUP`. The `oltp` machine and the `tpch` machine for SF100 are not chosen yet, so they need `INSTANCE_TYPE`, or `HOST` for a machine that exists. `DRY_RUN=1` prints the AWS command and stops.

```sh
machines/instance.sh 4xl clickbench
machines/instance.sh tpch tpch SCALES="1 10"
INSTANCE_TYPE=<type> machines/instance.sh oltp tpcc WAREHOUSES="<n> <4n>" VU=<v> SERVER_CPUS=0-15 CLIENT_CPUS=16-31
HOST=root@server3 MACHINE=server3 machines/instance.sh 4xl pgbench SMOKE=1
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
answers/           PostgreSQL 19 answers for ClickBench and TPC-H (the TPC-H SF1 set is fetched)
clickbench/        the rupg directory of our ClickBench fork
machines/          the install scripts, run-suite.sh and instance.sh for 4xl, metal, tpch and oltp
ratchet.toml       the instruction counts and the best number of each row
reports/<date>/    <commit>-<machine>-<suite>.md and a JSON file for each run
```

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). The milestones are issues with the label `kind/milestone`.

## License

Apache-2.0. See [LICENSE-APACHE](LICENSE-APACHE). The TPC-H and TPC-C numbers in this repository are not audited TPC results, and they are not comparable with published TPC results.
