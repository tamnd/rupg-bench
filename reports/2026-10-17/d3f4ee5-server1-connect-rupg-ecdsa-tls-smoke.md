# connect-rupg-ecdsa-tls on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792272277 |
| kernel | 6.8.0-101-generic |
| suite | connect-rupg-ecdsa-tls |
| machine | server1 |
| date | 2026-10-17 |
| commit | d3f4ee5 |
| smoke | true |
| server | 127.0.0.1 port 55443 user postgres |
| transport | tls |
| client_auth | password |
| server_process | pid 1014907 and the processes below it |

## connect

| Field | Value |
|---|---|
| clients | 4 |
| seconds | 10.002408 |
| connections | 18106 |
| connections_per_second | 1810.2 |
| latency.count | 18106 |
| latency.mean_us | 2208.9 |
| latency.p50_us | 1474.6 |
| latency.p95_us | 5374 |
| latency.p99_us | 16384 |
| latency.p999_us | 26607.6 |
| latency.max_us | 34461.6 |
| server_cpu_seconds | 14.03 |
| server_cpu_us_per_connection | 774.9 |

