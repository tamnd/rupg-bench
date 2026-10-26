# m1 on server2, 2026-10-26

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | m1 |
| machine | server2 |
| date | 2026-10-26 |
| commit | 3c5cb50b |
| smoke | false |
| rupg_commit | 3c5cb50b9fd4e55106fd698b7b55fb88e1f4108e |
| rupg_version | 0.2.3 |
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
| latency.p50_us | 2433.973 |
| latency.p90_us | 4306.859 |
| latency.p99_us | 9013.312 |
| latency.p999_us | 17044.5 |
| latency.max_us | 60572.929 |
| latency.mean_us | 2839.603484 |
| fdatasync_syncs | 1000 |
| fdatasync.p50_us | 2096.202 |
| fdatasync.p90_us | 3494.713 |
| fdatasync.p99_us | 5924.006 |
| fdatasync.p999_us | 10009.981 |
| fdatasync.max_us | 11078.165 |
| fdatasync.mean_us | 2363.518604 |
| device_write_bytes | 51961856 |

## throughput

| Field | Value |
|---|---|
| writers | 6 |
| seconds | 30.004606 |
| commits | 32002 |
| commits_per_second | 1066.569592 |
| writer_min | 5299 |
| writer_max | 5368 |
| device_write_bytes | 47828992 |

## recovery

| Field | Value |
|---|---|
| rows | 256 |
| value_bytes | 2000 |
| rounds | 2098 |
| row_bytes_written | 1074176000 |
| log_blocks | 2098 |
| log_bytes | 1078036320 |
| file_bytes | 1375780864 |
| fill_seconds | 18.88768 |
| page_cache_dropped | true |
| open_seconds | 4.563479 |
| replay_gib_per_second | 0.220007 |
| device_write_bytes | 1085358080 |

## ring

| Field | Value |
|---|---|
| writers | 6 |
| value_bytes | 2000 |
| rows_per_commit | 100 |
| load_bytes | 1073741824 |
| commit_seconds | 10 |
| default.ring_bytes | 67108864 |
| default.load_seconds | 19.659264 |
| default.load_slower | 0 |
| default.commits_per_second | 896.120951 |
| default.commits_fewer | 0 |
| default.enough | true |
| worker.ring_bytes | 50331648 |
| worker.load_seconds | 21.773088 |
| worker.load_slower | 0.107523 |
| worker.commits_per_second | 827.414651 |
| worker.commits_fewer | 0.076671 |
| worker.enough | false |
| smallest.ring_bytes | 16777216 |
| smallest.load_seconds | 19.893421 |
| smallest.load_slower | 0.011911 |
| smallest.commits_per_second | 1021.129673 |
| smallest.commits_fewer | -0.1395 |
| smallest.enough | true |
| device_write_bytes | 6832193536 |

## postgresql

| Field | Value |
|---|---|
| server_version | 18.6 |
| settings | fsync=on full_page_writes=on shared_buffers=131072 synchronous_commit=on wal_sync_method=fdatasync |
| data_directory | /var/lib/rupgpg/data |
| value_bytes | 100 |
| seconds | 30 |
| curve.0.clients | 1 |
| curve.0.commits_per_second | 709.233855 |
| curve.0.latency_ms | 1.41 |
| curve.1.clients | 6 |
| curve.1.commits_per_second | 2302.561842 |
| curve.1.latency_ms | 2.606 |
| curve.2.clients | 8 |
| curve.2.commits_per_second | 2938.289609 |
| curve.2.latency_ms | 2.723 |
| curve.3.clients | 32 |
| curve.3.commits_per_second | 7753.75747 |
| curve.3.latency_ms | 4.127 |
| device_write_bytes | null |

