# ycsb-rupg on server2-scale, 2026-11-05

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | ycsb-rupg |
| machine | server2-scale |
| date | 2026-11-05 |
| commit | 9474c37f |
| smoke | false |
| engine_name | rupg |
| cgroup | /sys/fs/cgroup/system.slice/m3w-yr.service |
| records | 100000 |
| field_count | 10 |
| field_length | 100 |
| distribution | scrambled zipfian, constant 0.99 |
| time_s | 45 |
| seed | 1 |
| kernel | 6.8.0-147-generic |
| server_version | 19.0 (rupg 0.4.20) |

## synchronous_commit

| Field | Value |
|---|---|
| server | on |
| run | on |
| set_by_driver | false |

## load

| Field | Value |
|---|---|
| records | 100000 |
| secs | 14.886 |
| statements | CREATE TABLE, COPY FROM STDIN, VACUUM ANALYZE, CHECKPOINT |
| server.wall_s | 14.885902 |
| server.cpu_usec | 12518927 |
| server.user_usec | 7576436 |
| server.system_usec | 4942491 |
| server.rbytes | 0 |
| server.wbytes | 221163520 |
| server.memory_peak_bytes | 752668672 |
| server.memory_peak_scope | since the cgroup was created |
| server.file_max_bytes | 221962240 |
| server.pss_max_bytes | 619345920 |
| server.memory_current_max_bytes | 752549888 |
| server.samples | 91 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.001874 |
| cpu_usec | 2202 |
| user_usec | 0 |
| system_usec | 2202 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 752668672 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 111861760 |
| pss_max_bytes | 233111552 |
| memory_current_max_bytes | 327905280 |
| samples | 85 |

## rows

| workload | clients | pipeline | ops | secs | ops_per_s | errors | first_error | server_cpu_usec_per_op | server_memory_peak | read.count | read.mean_us | read.p50_us | read.p95_us | read.p99_us | read.p999_us | read.max_us | update.count | update.mean_us | update.p50_us | update.p95_us | update.p99_us | update.p999_us | update.max_us | hottest_key.key | hottest_key.updates | hottest_key.updates_per_s | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.acknowledged_updates | check.fields_checked | check.fields_wrong | check.examples |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| b | 1 | 1 | 52348 | 45.076 | 1161 | 0 | null | 237.4 | 752668672 | 49748 | 397.8 | 71.7 | 213 | 2392.1 | 73400.3 | 266492.1 | 2600 | 9678.1 | 1359.9 | 72876 | 89653.2 | 184549.4 | 296885.4 | user6166968228214299628 | 88 | 2 | 45.079265 | 12429511 | 6890788 | 5538723 | 0 | 15769600 | 752668672 | since the cgroup was created | 113958912 | 237476864 | 334561280 | 335 | 2600 | 2324 | 0 |  |
| b | 4 | 1 | 145926 | 45.008 | 3242 | 0 | null | 195.4 | 752668672 | 138621 | 603 | 79.9 | 618.5 | 4390.9 | 77070.3 | 300268.1 | 7305 | 13123 | 2015.2 | 76546 | 101711.9 | 243269.6 | 402463.6 | user6166968228214299628 | 292 | 6.5 | 45.015281 | 28506742 | 14414405 | 14092338 | 0 | 43229184 | 752668672 | since the cgroup was created | 123396096 | 240872448 | 338952192 | 323 | 7305 | 6250 | 0 |  |
| c | 1 | 1 | 129530 | 45.017 | 2877 | 0 | null | 173.4 | 752668672 | 129530 | 344.9 | 66.6 | 140.3 | 2129.9 | 69206 | 285138.7 |  |  |  |  |  |  |  |  |  |  | 45.288006 | 22459187 | 12884652 | 9574534 | 0 | 0 | 752668672 | since the cgroup was created | 123396096 | 234434560 | 341127168 | 312 | 0 | 0 | 0 |  |
| c | 4 | 1 | 247077 | 45.096 | 5479 | 0 | null | 171.5 | 752668672 | 247077 | 718.2 | 68.1 | 288.8 | 4685.8 | 101711.9 | 870559.8 |  |  |  |  |  |  |  |  |  |  | 45.126119 | 42367526 | 23922540 | 18444985 | 0 | 0 | 752668672 | since the cgroup was created | 123396096 | 229112832 | 335982592 | 143 | 0 | 0 | 0 |  |

