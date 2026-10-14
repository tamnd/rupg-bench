# connect-rupg on server1, 2026-10-14

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1791983923 |
| kernel | 6.8.0-101-generic |
| suite | connect-rupg |
| machine | server1 |
| date | 2026-10-14 |
| commit | 178a60a6 |
| smoke | true |
| server | /tmp port 55441 user postgres |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 30.005178 |
| connections | 54571 |
| connections_per_second | 1818.7 |
| latency.count | 54571 |
| latency.mean_us | 2195.8 |
| latency.p50_us | 1081.3 |
| latency.p95_us | 7110.7 |
| latency.p99_us | 22151.2 |
| latency.p999_us | 46661.6 |
| latency.max_us | 131625.5 |

