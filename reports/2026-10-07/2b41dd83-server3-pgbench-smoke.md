# pgbench on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| cgroup | /sys/fs/cgroup/system.slice/system-postgresql.slice/postgresql@19-main.service |
| unix_time | 1791384356 |
| kernel | 6.8.0-106-generic |
| suite | pgbench |
| machine | server3 |
| date | 2026-10-07 |
| commit | 2b41dd83 |
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
| wall_s | 10.032522 |
| cpu_usec | 37631 |
| user_usec | 7259 |
| system_usec | 30373 |
| rbytes | 2244608 |
| wbytes | 2392064 |
| memory_peak_bytes | 80670720 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 59772928 |
| pss_max_bytes | 35897344 |
| memory_current_max_bytes | 69509120 |
| samples | 97 |

## pgbench

| Field | Value |
|---|---|
| transactions | 1045 |
| failed | 0 |
| latency_average_ms | 114.355 |
| tps | 34.978843 |

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
number of transactions actually processed: 1045
number of failed transactions: 0 (0.000%)
latency average = 114.355 ms
initial connection time = 286.120 ms
tps = 34.978843 (without initial connection time)
```

## run

| Field | Value |
|---|---|
| wall_s | 30.750013 |
| cpu_usec | 3435745 |
| user_usec | 2095562 |
| system_usec | 1340182 |
| rbytes | 8761344 |
| wbytes | 9592832 |
| memory_peak_bytes | 80670720 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 46084096 |
| pss_max_bytes | 54194176 |
| memory_current_max_bytes | 62607360 |
| samples | 180 |

## check

| Field | Value |
|---|---|
| sum_abalance | -145866 |
| sum_tbalance | -145866 |
| sum_bbalance | -145866 |
| history_rows | 1045 |
| transactions | 1045 |
| passed | true |
| failures |  |

