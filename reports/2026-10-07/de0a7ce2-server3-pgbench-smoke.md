# pgbench on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| cgroup | /sys/fs/cgroup/system.slice/system-postgresql.slice/postgresql@19-main.service |
| unix_time | 1791363827 |
| kernel | 6.8.0-106-generic |
| suite | pgbench |
| machine | server3 |
| date | 2026-10-07 |
| commit | de0a7ce2 |
| smoke | true |
| postgresql_commit | 7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333 |
| pgbench_version | pgbench (PostgreSQL) 19beta4 |
| server_version | 19beta4 |

## settings

| Field | Value |
|---|---|
| scale | 1 |
| clients | 4 |
| jobs | 4 |
| time_s | 30 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.03181 |
| cpu_usec | 26703 |
| user_usec | 0 |
| system_usec | 26704 |
| rbytes | 0 |
| wbytes | 2408448 |
| memory_peak_bytes | 80031744 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 61829120 |
| pss_max_bytes | 34781184 |
| memory_current_max_bytes | 71446528 |
| samples | 20 |

## pgbench

| Field | Value |
|---|---|
| transactions | 588 |
| failed | 0 |
| latency_average_ms | 202.763 |
| tps | 19.727433 |

output:

```
pgbench (19beta4)
transaction type: <builtin: TPC-B (sort of)>
scaling factor: 1
query mode: simple
number of clients: 4
number of threads: 4
maximum number of tries: 1
duration: 30 s
number of transactions actually processed: 588
number of failed transactions: 0 (0.000%)
latency average = 202.763 ms
initial connection time = 432.160 ms
tps = 19.727433 (without initial connection time)
```

## run

| Field | Value |
|---|---|
| wall_s | 31.112581 |
| cpu_usec | 2373405 |
| user_usec | 1382255 |
| system_usec | 991149 |
| rbytes | 602112 |
| wbytes | 5218304 |
| memory_peak_bytes | 83308544 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 65441792 |
| pss_max_bytes | 58931200 |
| memory_current_max_bytes | 82362368 |
| samples | 44 |

## check

| Field | Value |
|---|---|
| sum_abalance | 141710 |
| sum_tbalance | 141710 |
| sum_bbalance | 141710 |
| history_rows | 588 |
| transactions | 588 |
| passed | true |
| failures |  |

