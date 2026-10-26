# m1 on server2, 2026-10-25

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | m1 |
| machine | server2 |
| date | 2026-10-25 |
| commit | 2a5296ed |
| smoke | true |
| rupg_commit | 2a5296ed69345ec35519eab7fb8be25e3b0a9615 |
| rupg_version | 0.1.53 |
| cores | 6 |
| os | linux |
| kernel | 6.8.0-147-generic |
| dir | /root/m1-run |

## empty

| Field | Value |
|---|---|
| file_bytes | 83935232 |
| allocated_bytes | 307200 |
| device_write_bytes | 311296 |

## latency

| Field | Value |
|---|---|
| warmup | 1000 |
| commits | 10000 |
| value_bytes | 100 |
| latency.p50_us | 2385.933 |
| latency.p90_us | 4446.69 |
| latency.p99_us | 8413.764 |
| latency.p999_us | 18078.542 |
| latency.max_us | 48884.428 |
| latency.mean_us | 2774.214574 |
| fdatasync_syncs | 1000 |
| fdatasync.p50_us | 2087.767 |
| fdatasync.p90_us | 4567.155 |
| fdatasync.p99_us | 8657.438 |
| fdatasync.p999_us | 25661.905 |
| fdatasync.max_us | 27020.471 |
| fdatasync.mean_us | 2554.630752 |
| device_write_bytes | 52019200 |

## throughput

| Field | Value |
|---|---|
| writers | 6 |
| seconds | 30.008444 |
| commits | 33622 |
| commits_per_second | 1120.417968 |
| writer_min | 5572 |
| writer_max | 5644 |
| device_write_bytes | 48922624 |

## recovery

| Field | Value |
|---|---|
| rows | 256 |
| value_bytes | 2000 |
| rounds | 2098 |
| row_bytes_written | 1074176000 |
| log_blocks | 2098 |
| log_bytes | 1078036320 |
| file_bytes | 1526775808 |
| fill_seconds | 19.126163 |
| page_cache_dropped | true |
| open_seconds | 4.082801 |
| replay_gib_per_second | 0.245909 |
| device_write_bytes | 1085407232 |

## ring

| Field | Value |
|---|---|
| writers | 6 |
| value_bytes | 2000 |
| rows_per_commit | 100 |
| load_bytes | 1073741824 |
| default_ring_bytes | 67108864 |
| default_seconds | 17.984226 |
| small_ring_bytes | 16777216 |
| small_seconds | 22.678722 |
| small_slower | 0.261034 |
| small_enough | false |
| device_write_bytes | 4541157376 |

