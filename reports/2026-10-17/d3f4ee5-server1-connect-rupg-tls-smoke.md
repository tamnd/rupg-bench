# connect-rupg-tls on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792272255 |
| kernel | 6.8.0-101-generic |
| suite | connect-rupg-tls |
| machine | server1 |
| date | 2026-10-17 |
| commit | d3f4ee5 |
| smoke | true |
| server | 127.0.0.1 port 55442 user postgres |
| transport | tls |
| client_auth | password |
| server_process | pid 1014889 and the processes below it |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 10.0047 |
| connections | 11111 |
| connections_per_second | 1110.6 |
| latency.count | 11111 |
| latency.mean_us | 3599.9 |
| latency.p50_us | 2392.1 |
| latency.p95_us | 9306.1 |
| latency.p99_us | 24903.7 |
| latency.p999_us | 38535.2 |
| latency.max_us | 92790.2 |
| server_cpu_seconds | 22.1 |
| server_cpu_us_per_connection | 1989 |

