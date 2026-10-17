# connect-rupg on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792272232 |
| kernel | 6.8.0-101-generic |
| suite | connect-rupg |
| machine | server1 |
| date | 2026-10-17 |
| commit | d3f4ee5 |
| smoke | true |
| server | /tmp port 55442 user postgres |
| transport | unix |
| client_auth | none |
| server_process | pid 1014889 and the processes below it |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 10.001241 |
| connections | 103633 |
| connections_per_second | 10362 |
| latency.count | 103633 |
| latency.mean_us | 385.4 |
| latency.p50_us | 158.7 |
| latency.p95_us | 938 |
| latency.p99_us | 3735.6 |
| latency.p999_us | 27656.2 |
| latency.max_us | 82995.6 |
| server_cpu_seconds | 14.01 |
| server_cpu_us_per_connection | 135.2 |

