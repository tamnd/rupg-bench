# TPC-H SF0.1 on server2: rupg, PostgreSQL 19, DuckDB and rudb

This file compares the TPC-H reports of 2026-11-15 in this directory. All the runs used the same machine, the same data and the same 22 queries of `qgen -d`.

## The five parts

1. **The machine.** server2: 6 vCPUs of an AMD EPYC processor at 2.0 GHz, 11 GiB of memory and kernel 6.8.0-147-generic. The machine is shared, and the load average was between 11 and 24 during the runs. So the times of one query can change by 30 percent from run to run.
2. **The metric.** The hot time of each query is the minimum of the second and third runs. The first run is not cold, because a drop of the page cache hurts the other users of the machine (`--no-cold`).
3. **The baselines.** PostgreSQL 19 (19beta4) from the oracle build of rupg-compat with `shared_buffers = 2GB` and `max_wal_size = 4GB`. DuckDB v2.0.0-dev84237 (cc7e7bac7f) with all threads. rudb 0.10.16 (74a1cc19), built from source with the release profile of its workspace.
4. **The transport.** rupg and PostgreSQL 19 use TCP on 127.0.0.1, with a new connection for each run. DuckDB and rudb run in process, with a new shell process for each run. The transport is not the same, so the ratios against DuckDB and rudb are not gate numbers. They show the distance to an analytical engine.
5. **The ratio.** The baseline number divided by the rupg number. A ratio above 1 means that rupg is faster.

The engine under test is rupg main at 8067c677 (#1585). All 22 answers of rupg, PostgreSQL 19 and rudb match the answers of DuckDB.

## Hot time of each query, in seconds

| Query | DuckDB | rudb | rupg | PostgreSQL 19 |
|---|---|---|---|---|
| q1 | 0.208 | 0.395 | 0.096 | 1.382 |
| q2 | 0.102 | 0.084 | 0.006 | 0.173 |
| q3 | 0.189 | 0.020 | 0.083 | 0.399 |
| q4 | 0.200 | 0.106 | 0.190 | 1.098 |
| q5 | 0.214 | 0.196 | 0.091 | 0.307 |
| q6 | 0.026 | 0.374 | 0.017 | 0.289 |
| q7 | 0.221 | 0.109 | 0.023 | 0.308 |
| q8 | 0.305 | 0.094 | 0.025 | 0.397 |
| q9 | 0.216 | 0.178 | 0.110 | 1.379 |
| q10 | 0.598 | 0.212 | 0.113 | 0.611 |
| q11 | 0.102 | 0.068 | 0.032 | 0.212 |
| q12 | 0.299 | 0.406 | 0.098 | 0.515 |
| q13 | 0.299 | 0.101 | 0.305 | 0.796 |
| q14 | 0.103 | 0.410 | 0.017 | 0.555 |
| q15 | 0.110 | 0.324 | 0.101 | 0.373 |
| q16 | 0.330 | 0.022 | 0.084 | 0.212 |
| q17 | 0.129 | 0.018 | 0.084 | 160.787 |
| q18 | 0.234 | 0.273 | 0.121 | 1.038 |
| q19 | 0.106 | 0.085 | 0.088 | 0.612 |
| q20 | 0.205 | 0.107 | 0.022 | 238.606 |
| q21 | 0.217 | 0.191 | 0.210 | 0.509 |
| q22 | 0.201 | 0.077 | 0.016 | 0.121 |
| hot total (s) | 4.614 | 3.851 | 1.930 | 410.680 |
| failed / wrong | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| hot CPU (s) | 29.519 | 12.879 | 4.832 | 785.452 |
| load wall (s) | 37.091 | 44.325 | 56.197 | 25.214 |
| peak memory (MB) | 596 | 558 | 501 | 566 |
| geomean (s) | 0.179 | 0.124 | 0.059 | 0.789 |
| total without q17, q20 (s) | 4.280 | 3.726 | 1.825 | 11.287 |

The peak memory is the peak of the cgroup of the engine during the load and the queries. The hot CPU is the CPU time of the cgroup for the hot runs of all the queries.

## Ratios

| Baseline | Hot sum ratio | Geometric mean ratio | Ratio without Q17 and Q20 | Queries where rupg is faster | CPU ratio |
|---|---|---|---|---|---|
| DuckDB | 2.39 | 3.04 | 2.35 | 21 of 22 | 6.11 |
| rudb | 1.99 | 2.10 | 2.04 | 15 of 22 | 2.67 |
| PostgreSQL 19 | 212.74 | 13.37 | 6.18 | 22 of 22 | 162.56 |

## Findings

- rupg is faster than PostgreSQL 19 on all 22 queries. Without Q17 and Q20, the sum is 6.2 times smaller (1.83 s against 11.29 s), and the CPU time of the hot runs is 162 times smaller. PostgreSQL 19 needs 161 s for Q17 and 239 s for Q20, because it runs their correlated subqueries once for each row.
- rupg is faster than DuckDB on 21 of 22 queries and faster than rudb on 15 of 22 queries. The sum of rupg is 2.4 times smaller than DuckDB and 2.0 times smaller than rudb.
- rudb is faster on Q17 (4.5x), Q3 (4.3x), Q16 (3.8x), Q13 (3.0x), Q4 (1.8x), Q21 (1.1x) and Q19 (1.0x). DuckDB is faster on Q13 only. These queries are the next work for rupg.
- The peak memory of rupg is 501 MB, against 566 MB for PostgreSQL 19, 596 MB for DuckDB and 558 MB for rudb. The load of rupg takes 56 s, against 25 s for PostgreSQL 19.
- Since the reports of 2026-11-09, the hot sum of rupg went from 23.5 s to 1.93 s, the peak memory from 1691 MB to 501 MB and the load from 258 s to 56 s.
