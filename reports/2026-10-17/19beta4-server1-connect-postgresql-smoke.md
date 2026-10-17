# connect-postgresql on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792272244 |
| kernel | 6.8.0-101-generic |
| suite | connect-postgresql |
| machine | server1 |
| date | 2026-10-17 |
| commit | 19beta4 |
| smoke | true |
| server | /tmp port 54319 user postgres |
| transport | unix |
| client_auth | none |
| server_process | pid 121969 and the processes below it |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 10.032279 |
| connections | 3659 |
| connections_per_second | 364.7 |
| latency.count | 3659 |
| latency.mean_us | 10961.1 |
| latency.p50_us | 7733.2 |
| latency.p95_us | 36700.2 |
| latency.p99_us | 44826.6 |
| latency.p999_us | 61603.8 |
| latency.max_us | 100894.6 |
| server_cpu_seconds | 34.02 |
| server_cpu_us_per_connection | 9297.6 |

