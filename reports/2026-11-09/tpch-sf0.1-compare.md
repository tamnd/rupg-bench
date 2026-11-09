# TPC-H SF0.1 on server2: rupg, PostgreSQL 19, DuckDB and rudb

This file compares the TPC-H reports of 2026-11-09 in this directory. All the runs used the same machine, the same data and the same 22 queries of `qgen -d`.

## The five parts

1. **The machine.** server2: 6 vCPUs of an AMD EPYC processor at 2.0 GHz, 11 GiB of memory and kernel 6.8.0-147-generic. The machine is shared, and the load average was between 11 and 24 during the runs. So the times of one query can change by 30 percent from run to run.
2. **The metric.** The hot time of each query is the minimum of the second and third runs. The first run is not cold, because a drop of the page cache hurts the other users of the machine (`--no-cold`).
3. **The baselines.** PostgreSQL 19 from the oracle build of rupg-compat with `shared_buffers = 2GB` and `max_wal_size = 4GB`. DuckDB v2.0.0-dev84237 (cc7e7bac7f) with all threads, and with `SET threads = 1`. rudb 0.10.16 (74a1cc19), built from source with the release profile of its workspace.
4. **The transport.** rupg and PostgreSQL 19 use TCP on 127.0.0.1, with a new connection for each run. DuckDB and rudb run in process, with a new shell process for each run. The transport is not the same, so the ratios against DuckDB and rudb are not gate numbers. They show the distance to an analytical engine.
5. **The ratio.** The baseline number divided by the rupg number. A ratio above 1 means that rupg is faster.

The engine under test is rupg main at 4b8f9ff6 (#1433). All 22 answers of rupg, PostgreSQL 19 and rudb match the answers of DuckDB.

## Hot time of each query, in seconds

| Query | DuckDB | DuckDB 1 thread | rudb | rupg | PostgreSQL 19 |
|---|---|---|---|---|---|
| q1 | 0.273 | 0.205 | 0.708 | 0.397 | 1.898 |
| q2 | 0.103 | 0.118 | 0.014 | 0.702 | 0.194 |
| q3 | 0.198 | 0.120 | 0.090 | 0.894 | 0.605 |
| q4 | 0.223 | 0.210 | 0.196 | 0.784 | 1.195 |
| q5 | 0.219 | 0.196 | 0.124 | 1.214 | 0.408 |
| q6 | 0.093 | 0.092 | 0.702 | 0.172 | 0.305 |
| q7 | 0.315 | 0.325 | 0.167 | 1.093 | 0.418 |
| q8 | 0.209 | 0.221 | 0.109 | 1.086 | 0.504 |
| q9 | 0.382 | 0.402 | 0.198 | 2.875 | 1.505 |
| q10 | 0.629 | 0.391 | 0.298 | 1.410 | 0.616 |
| q11 | 0.121 | 0.187 | 0.011 | 0.189 | 0.209 |
| q12 | 0.202 | 0.188 | 0.591 | 0.887 | 0.789 |
| q13 | 0.202 | 0.289 | 0.038 | 1.811 | 0.886 |
| q14 | 0.110 | 0.024 | 0.616 | 0.305 | 0.499 |
| q15 | 0.187 | 0.188 | 0.671 | 0.188 | 0.506 |
| q16 | 0.490 | 0.321 | 0.020 | 0.710 | 0.297 |
| q17 | 0.197 | 0.193 | 0.169 | 0.716 | 183.299 |
| q18 | 0.116 | 0.202 | 0.173 | 3.607 | 1.726 |
| q19 | 0.041 | 0.103 | 0.017 | 0.276 | 0.520 |
| q20 | 0.193 | 0.199 | 0.032 | 1.318 | 255.686 |
| q21 | 0.195 | 0.192 | 0.114 | 2.489 | 0.706 |
| q22 | 0.183 | 0.081 | 0.010 | 0.399 | 0.133 |
| hot total (s) | 4.881 | 4.447 | 5.069 | 23.521 | 452.903 |
| failed / wrong | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| hot CPU (s) | 28.307 | 23.700 | 11.962 | 42.407 | 878.290 |
| load wall (s) | 42.384 | 44.282 | 53.999 | 257.878 | 28.899 |
| peak memory (MB) | 587 | 633 | 446 | 1691 | 565 |
| geomean (s) | 0.190 | 0.175 | 0.108 | 0.758 | 0.944 |
| total without q17, q20 (s) | 4.491 | 4.055 | 4.868 | 21.487 | 13.918 |

The peak memory is the peak of the cgroup of the engine during the load and the queries. The hot CPU is the CPU time of the cgroup for the hot runs of all the queries.

## Ratios

| Baseline | Hot sum ratio | Geometric mean ratio | Ratio without Q17 and Q20 | Queries where rupg is faster |
|---|---|---|---|---|
| DuckDB | 0.21 | 0.25 | 0.21 | 0 of 22 |
| DuckDB 1 thread | 0.19 | 0.23 | 0.19 | 1 of 22 |
| rudb | 0.22 | 0.14 | 0.23 | 4 of 22 |
| PostgreSQL 19 | 19.26 | 1.24 | 0.65 | 9 of 22 |

## Findings

- PostgreSQL 19 needs 183 s for Q17 and 256 s for Q20. It runs their correlated subqueries once for each row, and the tables have only the keys of the specification. Without these two queries, PostgreSQL 19 is faster than rupg: 13.9 s against 21.5 s.
- rupg is faster than PostgreSQL 19 on 9 of 22 queries. The largest gaps where rupg is slower are Q2 (3.6x), Q21 (3.5x), Q22 (3.0x), Q5 (3.0x) and Q7 (2.6x).
- DuckDB and rudb are about 5 times faster than rupg on the sum, and they are faster on almost every query. With one thread, DuckDB is not slower, because the machine has only 6 vCPUs and other jobs use them.
- rupg uses 1691 MB at its peak. PostgreSQL 19 uses 565 MB, DuckDB 587 MB and rudb 446 MB. The load of rupg takes 258 s, against 29 s for PostgreSQL 19.
- rudb needed two changes to run in the harness. Its `COPY FROM` does not accept the `|` at the end of each line of a `.tbl` file, so its data has the lines without it. Its shell writes the line of `.timer on` to stderr, so a wrapper joins stderr to stdout. DuckDB writes that line to stdout.

## After this run

rupg #1442 reduces the rows of a join with an uncorrelated `IN` subquery. In a separate A/B on the same machine, Q18 went from 4.62 s to 2.10 s, and the other queries did not change.
