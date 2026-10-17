# connect-postgresql-tls on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792272266 |
| kernel | 6.8.0-101-generic |
| suite | connect-postgresql-tls |
| machine | server1 |
| date | 2026-10-17 |
| commit | 19beta4 |
| smoke | true |
| server | 127.0.0.1 port 54319 user postgres |
| transport | tls |
| client_auth | password |
| server_process | pid 121969 and the processes below it |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 10.016968 |
| connections | 2351 |
| connections_per_second | 234.7 |
| latency.count | 2351 |
| latency.mean_us | 17030 |
| latency.p50_us | 12189.7 |
| latency.p95_us | 43515.9 |
| latency.p99_us | 53477.4 |
| latency.p999_us | 72351.7 |
| latency.max_us | 77061.2 |
| server_cpu_seconds | 32.02 |
| server_cpu_us_per_connection | 13619.7 |

