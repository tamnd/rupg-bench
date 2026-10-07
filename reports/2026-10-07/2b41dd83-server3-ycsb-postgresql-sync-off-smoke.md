# ycsb-postgresql-sync-off on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | ycsb-postgresql-sync-off |
| machine | server3 |
| date | 2026-10-07 |
| commit | 2b41dd83 |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| engine_name | postgresql |
| cgroup | /sys/fs/cgroup/system.slice/system-postgresql.slice/postgresql@19-main.service |
| records | 10000 |
| field_count | 10 |
| field_length | 100 |
| distribution | scrambled zipfian, constant 0.99 |
| time_s | 5 |
| seed | 1 |
| kernel | 6.8.0-106-generic |
| server_version | 19beta4 |

## synchronous_commit

| Field | Value |
|---|---|
| server | on |
| run | off |
| set_by_driver | true |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.000368 |
| cpu_usec | 7110 |
| user_usec | 4618 |
| system_usec | 2493 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 120725504 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 58519552 |
| pss_max_bytes | 53413888 |
| memory_current_max_bytes | 67522560 |
| samples | 91 |

## rows

| workload | clients | pipeline | ops | secs | ops_per_s | errors | first_error | server_cpu_usec_per_op | server_memory_peak | read.count | read.mean_us | read.p50_us | read.p95_us | read.p99_us | read.p999_us | read.max_us | update.count | update.mean_us | update.p50_us | update.p95_us | update.p99_us | update.p999_us | update.max_us | hottest_key.key | hottest_key.updates | hottest_key.updates_per_s | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.acknowledged_updates | check.fields_checked | check.fields_wrong | check.examples | read_modify_write.count | read_modify_write.mean_us | read_modify_write.p50_us | read_modify_write.p95_us | read_modify_write.p99_us | read_modify_write.p999_us | read_modify_write.max_us |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| a | 1 | 1 | 335 | 5.056 | 66 | 0 | null | 711.2 | 120725504 | 163 | 14433 | 7012.4 | 57409.5 | 89129 | 135141.9 | 135141.9 | 172 | 15707.4 | 7012.4 | 59244.5 | 159383.6 | 228962.5 | 228962.5 | user2314253027668161298 | 7 | 1.4 | 5.091261 | 238253 | 129751 | 108501 | 774144 | 180224 | 120725504 | since the cgroup was created | 41586688 | 55630848 | 53202944 | 32 | 172 | 170 | 0 |  |  |  |  |  |  |  |  |
| a | 16 | 1 | 2942 | 5.122 | 574 | 0 | null | 504.1 | 120725504 | 1483 | 26047.6 | 13107.2 | 94371.8 | 210763.8 | 267386.9 | 270734.1 | 1459 | 28665.7 | 14286.8 | 103284.7 | 210763.8 | 295698.4 | 361792 | user2314253027668161298 | 40 | 7.8 | 5.190999 | 1483027 | 903817 | 579210 | 0 | 524288 | 120725504 | since the cgroup was created | 42831872 | 85303296 | 85700608 | 9 | 1459 | 1343 | 0 |  |  |  |  |  |  |  |  |
| b | 1 | 1 | 1122 | 5.004 | 224 | 0 | null | 210.3 | 120725504 | 1064 | 4484.4 | 95.2 | 27918.3 | 63176.7 | 78643.2 | 86982.4 | 58 | 3871.5 | 288.8 | 22282.2 | 36005 | 36005 | 36005 | user2314253027668161298 | 5 | 1 | 5.005509 | 235928 | 147020 | 88909 | 0 | 131072 | 120725504 | since the cgroup was created | 42958848 | 56737792 | 55508992 | 45 | 58 | 57 | 0 |  |  |  |  |  |  |  |  |
| b | 16 | 1 | 3789 | 5.064 | 748 | 0 | null | 350.7 | 120725504 | 3605 | 21368.8 | 9830.4 | 77594.6 | 152043.5 | 301989.9 | 441390.2 | 184 | 17994.3 | 9109.5 | 68157.4 | 131072 | 162695.7 | 162695.7 | user2314253027668161298 | 8 | 1.6 | 5.181291 | 1328624 | 809802 | 518822 | 0 | 172032 | 120725504 | since the cgroup was created | 43319296 | 86349824 | 85282816 | 9 | 184 | 181 | 0 |  |  |  |  |  |  |  |  |
| c | 1 | 1 | 356 | 5.042 | 71 | 0 | null | 295.6 | 120725504 | 356 | 14158.4 | 6979.6 | 56361 | 94371.8 | 162511.2 | 162511.2 |  |  |  |  |  |  |  |  |  |  | 5.087849 | 105230 | 46777 | 58454 | 0 | 0 | 120725504 | since the cgroup was created | 43327488 | 56489984 | 55574528 | 43 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| c | 16 | 1 | 3643 | 5.086 | 716 | 0 | null | 346.3 | 120725504 | 3643 | 22156.3 | 12058.6 | 80740.4 | 158335 | 283115.5 | 326352.9 |  |  |  |  |  |  |  |  |  |  | 5.09293 | 1261541 | 808735 | 452806 | 0 | 8192 | 120725504 | since the cgroup was created | 43327488 | 106685440 | 109477888 | 11 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| f | 1 | 1 | 1030 | 5.008 | 206 | 0 | null | 454.3 | 120725504 | 542 | 3543.1 | 88.1 | 19005.4 | 73924.6 | 100394.7 | 100394.7 |  |  |  |  |  |  |  | user2314253027668161298 | 23 | 4.6 | 5.009804 | 467943 | 375628 | 92314 | 385024 | 712704 | 120725504 | since the cgroup was created | 44662784 | 80392192 | 79933440 | 45 | 488 | 461 | 0 |  | 488 | 6322.6 | 202.8 | 45875.2 | 90177.5 | 159996.2 | 159996.2 |
| f | 16 | 1 | 2413 | 5.111 | 472 | 0 | null | 699.5 | 120725504 | 1207 | 22524 | 11468.8 | 77070.3 | 197132.3 | 299892.7 | 351898.6 |  |  |  |  |  |  |  | user2314253027668161298 | 43 | 8.4 | 5.355243 | 1687781 | 1029412 | 658369 | 8192 | 450560 | 120725504 | since the cgroup was created | 45228032 | 85773312 | 88133632 | 12 | 1206 | 1114 | 0 |  | 1206 | 44454.8 | 27394 | 122683.4 | 250609.7 | 358613 | 408001.2 |

