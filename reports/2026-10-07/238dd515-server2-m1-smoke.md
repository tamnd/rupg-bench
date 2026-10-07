# m1 on server2, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | m1 |
| machine | server2 |
| date | 2026-10-07 |
| commit | 238dd515 |
| smoke | true |
| rupg_commit | 238dd515594a39e198c40375cf4236a282390753 |
| rupg_version | 0.0.10 |
| cores | 6 |
| os | linux |
| kernel | 6.8.0-136-generic |
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
| latency.p50_us | 2028.857 |
| latency.p90_us | 3365.191 |
| latency.p99_us | 6218.538 |
| latency.p999_us | 13762.019 |
| latency.max_us | 46800.628 |
| latency.mean_us | 2271.06197 |

## throughput

| Field | Value |
|---|---|
| writers | 6 |
| seconds | 30.011143 |
| commits | 37237 |
| commits_per_second | 1240.772456 |
| writer_min | 6181 |
| writer_max | 6219 |

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
| fill_seconds | 16.172204 |
| page_cache_dropped | true |
| open_seconds | 4.086647 |
| replay_gib_per_second | 0.245678 |

