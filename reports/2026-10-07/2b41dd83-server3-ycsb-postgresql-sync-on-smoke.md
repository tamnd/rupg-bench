# ycsb-postgresql-sync-on on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | ycsb-postgresql-sync-on |
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
| run | on |
| set_by_driver | true |

## load

| Field | Value |
|---|---|
| records | 10000 |
| secs | 2.818 |
| statements | CREATE TABLE, COPY FROM STDIN, VACUUM ANALYZE, CHECKPOINT |
| server.wall_s | 2.818018 |
| server.cpu_usec | 741628 |
| server.user_usec | 491723 |
| server.system_usec | 249905 |
| server.rbytes | 3424256 |
| server.wbytes | 21049344 |
| server.memory_peak_bytes | 90279936 |
| server.memory_peak_scope | since the cgroup was created |
| server.file_max_bytes | 69603328 |
| server.pss_max_bytes | 72999936 |
| server.memory_current_max_bytes | 89948160 |
| server.samples | 28 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.045088 |
| cpu_usec | 39063 |
| user_usec | 9765 |
| system_usec | 29297 |
| rbytes | 1376256 |
| wbytes | 221184 |
| memory_peak_bytes | 90279936 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 66793472 |
| pss_max_bytes | 53690368 |
| memory_current_max_bytes | 78360576 |
| samples | 98 |

## rows

| workload | clients | pipeline | ops | secs | ops_per_s | errors | first_error | server_cpu_usec_per_op | server_memory_peak | read.count | read.mean_us | read.p50_us | read.p95_us | read.p99_us | read.p999_us | read.max_us | update.count | update.mean_us | update.p50_us | update.p95_us | update.p99_us | update.p999_us | update.max_us | hottest_key.key | hottest_key.updates | hottest_key.updates_per_s | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.acknowledged_updates | check.fields_checked | check.fields_wrong | check.examples | read_modify_write.count | read_modify_write.mean_us | read_modify_write.p50_us | read_modify_write.p95_us | read_modify_write.p99_us | read_modify_write.p999_us | read_modify_write.max_us |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| a | 1 | 1 | 832 | 5.007 | 166 | 0 | null | 513.1 | 90279936 | 410 | 3427.3 | 155.6 | 19136.5 | 47972.4 | 64605.5 | 64605.5 | 422 | 8529.4 | 2687 | 43253.8 | 76021.8 | 154136.7 | 154136.7 | user2314253027668161298 | 18 | 3.6 | 5.05534 | 426883 | 205243 | 221640 | 348160 | 6725632 | 90279936 | since the cgroup was created | 43909120 | 55369728 | 57016320 | 46 | 422 | 397 | 0 |  |  |  |  |  |  |  |  |
| a | 16 | 1 | 2558 | 5.062 | 505 | 0 | null | 556.1 | 96415744 | 1298 | 15291.7 | 5767.2 | 65273.9 | 149946.4 | 202375.2 | 203218.4 | 1260 | 47949.7 | 33423.4 | 131072 | 232783.9 | 299991.1 | 299991.1 | user2314253027668161298 | 35 | 6.9 | 5.064266 | 1422583 | 776949 | 645634 | 0 | 8814592 | 96415744 | since the cgroup was created | 50339840 | 83573760 | 94433280 | 12 | 1260 | 1163 | 0 |  |  |  |  |  |  |  |  |
| b | 1 | 1 | 1962 | 5.004 | 392 | 0 | null | 201.6 | 96415744 | 1866 | 2305.3 | 92.7 | 11010 | 46137.3 | 78118.9 | 118133.7 | 96 | 7265.5 | 3145.7 | 42991.6 | 67684.5 | 67684.5 | 67684.5 | user2314253027668161298 | 6 | 1.2 | 5.017835 | 395615 | 231441 | 164175 | 319488 | 1146880 | 96415744 | since the cgroup was created | 52027392 | 56331264 | 65650688 | 40 | 96 | 95 | 0 |  |  |  |  |  |  |  |  |
| b | 16 | 1 | 4961 | 5.079 | 977 | 0 | null | 388.6 | 97021952 | 4731 | 15095.3 | 4882.4 | 69206 | 145752.1 | 299892.7 | 781956.8 | 230 | 38061.2 | 24903.7 | 100139 | 145752.1 | 212179.6 | 212179.6 | user2314253027668161298 | 10 | 2 | 5.104177 | 1927782 | 1009583 | 918199 | 0 | 2023424 | 97021952 | since the cgroup was created | 52514816 | 85125120 | 95809536 | 8 | 230 | 225 | 0 |  |  |  |  |  |  |  |  |
| c | 1 | 1 | 2094 | 5.005 | 418 | 0 | null | 161.4 | 97021952 | 2094 | 2271.3 | 79.4 | 9240.6 | 45088.8 | 96469 | 398010.3 |  |  |  |  |  |  |  |  |  |  | 5.08478 | 337941 | 231625 | 106316 | 0 | 0 | 97021952 | since the cgroup was created | 52592640 | 56376320 | 66146304 | 37 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| c | 16 | 1 | 8090 | 5.214 | 1552 | 0 | null | 328 | 120725504 | 8090 | 9942.8 | 2588.7 | 51642.4 | 85458.9 | 167772.2 | 287749.6 |  |  |  |  |  |  |  |  |  |  | 5.284805 | 2653802 | 1788671 | 865130 | 126976 | 229376 | 120725504 | since the cgroup was created | 53243904 | 109332480 | 119996416 | 9 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| f | 1 | 1 | 708 | 5.002 | 142 | 0 | null | 687.8 | 120725504 | 379 | 2485.7 | 133.1 | 11599.9 | 49020.9 | 75010.2 | 75010.2 |  |  |  |  |  |  |  | user2314253027668161298 | 16 | 3.2 | 5.0099 | 486972 | 277275 | 209696 | 0 | 3416064 | 120725504 | since the cgroup was created | 54054912 | 57175040 | 67555328 | 42 | 329 | 313 | 0 |  | 329 | 12332.9 | 3276.8 | 58196 | 98041.9 | 252020.5 | 252020.5 |
| f | 16 | 1 | 2037 | 5.289 | 385 | 0 | null | 874.3 | 120725504 | 1028 | 17655.6 | 6914 | 74973.2 | 160432.1 | 186646.5 | 211297.6 |  |  |  |  |  |  |  | user2314253027668161298 | 39 | 7.4 | 5.307529 | 1781008 | 947558 | 833449 | 1167360 | 4448256 | 120725504 | since the cgroup was created | 58478592 | 86146048 | 102748160 | 10 | 1009 | 935 | 0 |  | 1009 | 63058.2 | 51642.4 | 186646.5 | 278921.2 | 304087 | 383631.8 |

