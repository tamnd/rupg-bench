# connect-postgresql on server1, 2026-10-14

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1791983953 |
| kernel | 6.8.0-101-generic |
| suite | connect-postgresql |
| machine | server1 |
| date | 2026-10-14 |
| commit | 7d3d2db7 |
| smoke | true |
| server | /home/tam/work/pg19-oracle/sock port 54419 user postgres |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 30.027743 |
| connections | 6682 |
| connections_per_second | 222.5 |
| latency.count | 6682 |
| latency.mean_us | 17956.3 |
| latency.p50_us | 12320.8 |
| latency.p95_us | 48234.5 |
| latency.p99_us | 76021.8 |
| latency.p999_us | 153092.1 |
| latency.max_us | 194393.5 |

