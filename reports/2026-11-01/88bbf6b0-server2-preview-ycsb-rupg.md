# ycsb-rupg on server2-preview, 2026-11-01

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | ycsb-rupg |
| machine | server2-preview |
| date | 2026-11-01 |
| commit | 88bbf6b0 |
| smoke | false |
| engine_name | rupg |
| cgroup | /sys/fs/cgroup/system.slice/m3w-yr.service |
| records | 100000 |
| field_count | 10 |
| field_length | 100 |
| distribution | scrambled zipfian, constant 0.99 |
| time_s | 60 |
| seed | 1 |
| kernel | 6.8.0-147-generic |
| server_version | 19.0 (rupg 0.4.17) |

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
| secs | 16.521 |
| statements | CREATE TABLE, COPY FROM STDIN, VACUUM ANALYZE, CHECKPOINT |
| server.wall_s | 16.520817 |
| server.cpu_usec | 15704354 |
| server.user_usec | 10316944 |
| server.system_usec | 5387410 |
| server.rbytes | 0 |
| server.wbytes | 219774976 |
| server.memory_peak_bytes | 572583936 |
| server.memory_peak_scope | since the cgroup was created |
| server.file_max_bytes | 219803648 |
| server.pss_max_bytes | 421431296 |
| server.memory_current_max_bytes | 572583936 |
| server.samples | 164 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.003423 |
| cpu_usec | 1900 |
| user_usec | 1900 |
| system_usec | 0 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 572583936 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 110047232 |
| pss_max_bytes | 256723968 |
| memory_current_max_bytes | 349159424 |
| samples | 99 |

## rows

| workload | clients | pipeline | ops | secs | ops_per_s | errors | first_error | server_cpu_usec_per_op | server_memory_peak | read.count | read.mean_us | read.p50_us | read.p95_us | read.p99_us | read.p999_us | read.max_us | update.count | update.mean_us | update.p50_us | update.p95_us | update.p99_us | update.p999_us | update.max_us | hottest_key.key | hottest_key.updates | hottest_key.updates_per_s | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.acknowledged_updates | check.fields_checked | check.fields_wrong | check.examples | read_modify_write.count | read_modify_write.mean_us | read_modify_write.p50_us | read_modify_write.p95_us | read_modify_write.p99_us | read_modify_write.p999_us | read_modify_write.max_us |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| a | 1 | 1 | 57 | 60.49 | 1 | 0 | null | 988990.9 | 572583936 | 24 | 1118943.2 | 994050 | 1115684.9 | 4612892.6 | 4612892.6 | 4612892.6 | 33 | 1019254.9 | 1006633 | 1174405.1 | 1299512.5 | 1299512.5 | 1299512.5 | user6166968228214299628 | 1 | 0 | 60.493499 | 56372484 | 54815807 | 1556677 | 0 | 172032 | 572583936 | since the cgroup was created | 110088192 | 431425536 | 524546048 | 563 | 33 | 33 | 0 |  |  |  |  |  |  |  |  |
| a | 16 | 1 | 16 | 89.225 | 0 | 0 | null | 2714567.4 | 3089735680 | 8 | 80394387 | 82141249.5 | 89224971.1 | 89224971.1 | 89224971.1 | 89224971.1 | 8 | 84491098.6 | 82678120.4 | 88224786.2 | 88224786.2 | 88224786.2 | 88224786.2 | user6166968228214299628 | 0 | 0 | 89.228844 | 43433078 | 26801201 | 16631877 | 0 | 36864 | 3089735680 | since the cgroup was created | 110096384 | 2990940160 | 3089473536 | 370 | 8 | 8 | 0 |  |  |  |  |  |  |  |  |
| b | 1 | 1 | 61 | 60.727 | 1 | 0 | null | 967569.9 | 3090550784 | 56 | 987929.8 | 998244.4 | 1182793.7 | 1209050.9 | 1209050.9 | 1209050.9 | 5 | 1080539 | 1023410.2 | 1189043.6 | 1189043.6 | 1189043.6 | 1189043.6 | user6166968228214299628 | 0 | 0 | 60.798592 | 59021761 | 57956708 | 1065052 | 0 | 28672 | 3090550784 | since the cgroup was created | 110104576 | 2990997504 | 3090292736 | 218 | 5 | 5 | 0 |  |  |  |  |  |  |  |  |
| b | 16 | 1 | 81 | 71.122 | 1 | 0 | null | 1367942.4 | 3333140480 | 74 | 13271158.7 | 12683575.3 | 27514634.2 | 35391086.4 | 35391086.4 | 35391086.4 | 7 | 14036141.6 | 14629732.4 | 23006024.6 | 23006024.6 | 23006024.6 | 23006024.6 | user6166968228214299628 | 2 | 0 | 71.204881 | 110803337 | 101271704 | 9531633 | 0 | 110104576 | 3333140480 | since the cgroup was created | 216997888 | 3120732160 | 3329667072 | 125 | 7 | 7 | 0 |  |  |  |  |  |  |  |  |
| c | 1 | 1 | 54 | 60.792 | 1 | 0 | null | 1029432.5 | 3333140480 | 54 | 1125752.1 | 1115684.9 | 1384120.3 | 1494683 | 1494683 | 1494683 |  |  |  |  |  |  |  |  |  |  | 60.810359 | 55589353 | 54402342 | 1187011 | 0 | 0 | 3333140480 | since the cgroup was created | 110108672 | 3010778112 | 3109359616 | 106 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| c | 16 | 1 | 82 | 64.788 | 1 | 0 | null | 1338463.5 | 3333140480 | 82 | 12261575.4 | 10938744.8 | 22817013.8 | 32190972.6 | 32190972.6 | 32190972.6 |  |  |  |  |  |  |  |  |  |  | 64.890955 | 109754009 | 103291840 | 6462169 | 0 | 0 | 3333140480 | since the cgroup was created | 110108672 | 3022867456 | 3115376640 | 49 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| f | 1 | 1 | 34 | 60.414 | 1 | 0 | null | 1604853.9 | 3333140480 | 20 | 1192185.5 | 1132462.1 | 1484783.6 | 1506328.1 | 1506328.1 | 1506328.1 |  |  |  |  |  |  |  | user6166968228214299628 | 1 | 0 | 60.415621 | 54565031 | 53259775 | 1305255 | 0 | 73728 | 3333140480 | since the cgroup was created | 110125056 | 3004594176 | 3103158272 | 107 | 14 | 14 | 0 |  | 14 | 2612151.8 | 2600468.5 | 3024962.3 | 3024962.3 | 3024962.3 | 3024962.3 |
| f | 16 | 1 | 35 | 73 | 0 | 0 | null | 2543961.7 | 3333140480 | 19 | 23569635.2 | 18924699.6 | 46997688.2 | 46997688.2 | 46997688.2 | 46997688.2 |  |  |  |  |  |  |  | user6166968228214299628 | 1 | 0 | 73.014205 | 89038661 | 81889849 | 7148812 | 0 | 81920 | 3333140480 | since the cgroup was created | 110141440 | 3010782208 | 3109433344 | 95 | 16 | 16 | 0 |  | 16 | 40433120.1 | 38117834.8 | 64086008 | 64086008 | 64086008 | 64086008 |

