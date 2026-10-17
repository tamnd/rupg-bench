# connect-postgresql-ecdsa-tls on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792272288 |
| kernel | 6.8.0-101-generic |
| suite | connect-postgresql-ecdsa-tls |
| machine | server1 |
| date | 2026-10-17 |
| commit | 19beta4 |
| smoke | true |
| server | 127.0.0.1 port 54329 user postgres |
| transport | tls |
| client_auth | password |
| server_process | pid 1014924 and the processes below it |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 10.008104 |
| connections | 2796 |
| connections_per_second | 279.4 |
| latency.count | 2796 |
| latency.mean_us | 14310 |
| latency.p50_us | 9764.9 |
| latency.p95_us | 43253.8 |
| latency.p99_us | 49020.9 |
| latency.p999_us | 58982.4 |
| latency.max_us | 59060.8 |
| server_cpu_seconds | 31.94 |
| server_cpu_us_per_connection | 11423.5 |

