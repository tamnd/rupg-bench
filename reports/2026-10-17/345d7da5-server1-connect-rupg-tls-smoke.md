# connect-rupg-tls on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792267173 |
| kernel | 6.8.0-101-generic |
| suite | connect-rupg-tls |
| machine | server1 |
| date | 2026-10-17 |
| commit | 345d7da5 |
| smoke | true |
| server | 127.0.0.1 port 55442 user postgres |
| transport | tls |
| client_auth | password |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 10.003839 |
| connections | 4352 |
| connections_per_second | 435 |
| latency.count | 4352 |
| latency.mean_us | 9188.6 |
| latency.p50_us | 6488.1 |
| latency.p95_us | 24772.6 |
| latency.p99_us | 52690.9 |
| latency.p999_us | 85983.2 |
| latency.max_us | 119668.2 |

