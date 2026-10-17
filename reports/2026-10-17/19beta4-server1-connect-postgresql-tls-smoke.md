# connect-postgresql-tls on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792267183 |
| kernel | 6.8.0-101-generic |
| suite | connect-postgresql-tls |
| machine | server1 |
| date | 2026-10-17 |
| commit | 19beta4 |
| smoke | true |
| server | 127.0.0.1 port 54319 user postgres |
| transport | tls |
| client_auth | password |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 10.009095 |
| connections | 2197 |
| connections_per_second | 219.5 |
| latency.count | 2197 |
| latency.mean_us | 18211 |
| latency.p50_us | 14024.7 |
| latency.p95_us | 41943 |
| latency.p99_us | 60293.1 |
| latency.p999_us | 106430.5 |
| latency.max_us | 121206 |

