# m1 on server1, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | m1 |
| machine | server1 |
| date | 2026-10-07 |
| commit | 238dd515 |
| smoke | true |
| rupg_commit | 238dd515594a39e198c40375cf4236a282390753 |
| rupg_version | 0.0.10 |
| cores | 4 |
| os | linux |
| kernel | 6.8.0-101-generic |
| dir | run |

## empty

| Field | Value |
|---|---|
| file_bytes | 83935232 |
| allocated_bytes | 307200 |

## latency

| Field | Value |
|---|---|
| warmup | 1000 |
| commits | 10000 |
| value_bytes | 100 |
| latency.p50_us | 3097.975 |
| latency.p90_us | 9204.773 |
| latency.p99_us | 35891.058 |
| latency.p999_us | 90242.369 |
| latency.max_us | 197978.176 |
| latency.mean_us | 5095.02992 |

## throughput

| Field | Value |
|---|---|
| writers | 4 |
| seconds | 30.070982 |
| commits | 16278 |
| commits_per_second | 541.319202 |
| writer_min | 4056 |
| writer_max | 4092 |

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
| fill_seconds | 19.972809 |
| page_cache_dropped | true |
| open_seconds | 9.404246 |
| replay_gib_per_second | 0.10676 |

