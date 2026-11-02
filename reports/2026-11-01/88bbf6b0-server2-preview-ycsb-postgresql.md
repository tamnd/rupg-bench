# ycsb-postgresql on server2-preview, 2026-11-01

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | ycsb-postgresql |
| machine | server2-preview |
| date | 2026-11-01 |
| commit | 88bbf6b0 |
| smoke | false |
| engine_name | postgresql |
| cgroup | /sys/fs/cgroup/system.slice/m3w-yp.service |
| records | 100000 |
| field_count | 10 |
| field_length | 100 |
| distribution | scrambled zipfian, constant 0.99 |
| time_s | 60 |
| seed | 1 |
| kernel | 6.8.0-147-generic |
| server_version | 19beta4 |

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
| secs | 15.702 |
| statements | CREATE TABLE, COPY FROM STDIN, VACUUM ANALYZE, CHECKPOINT |
| server.wall_s | 15.701812 |
| server.cpu_usec | 11409300 |
| server.user_usec | 7010706 |
| server.system_usec | 4398595 |
| server.rbytes | 0 |
| server.wbytes | 347934720 |
| server.memory_peak_bytes | 397983744 |
| server.memory_peak_scope | since the cgroup was created |
| server.file_max_bytes | 331853824 |
| server.pss_max_bytes | 174466048 |
| server.memory_current_max_bytes | 397877248 |
| server.samples | 59 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.006016 |
| cpu_usec | 14895 |
| user_usec | 4616 |
| system_usec | 10280 |
| rbytes | 0 |
| wbytes | 8192 |
| memory_peak_bytes | 397983744 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 331968512 |
| pss_max_bytes | 109674496 |
| memory_current_max_bytes | 349954048 |
| samples | 55 |

## rows

| workload | clients | pipeline | ops | secs | ops_per_s | errors | first_error | server_cpu_usec_per_op | server_memory_peak | read.count | read.mean_us | read.p50_us | read.p95_us | read.p99_us | read.p999_us | read.max_us | update.count | update.mean_us | update.p50_us | update.p95_us | update.p99_us | update.p999_us | update.max_us | hottest_key.key | hottest_key.updates | hottest_key.updates_per_s | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.acknowledged_updates | check.fields_checked | check.fields_wrong | check.examples | read_modify_write.count | read_modify_write.mean_us | read_modify_write.p50_us | read_modify_write.p95_us | read_modify_write.p99_us | read_modify_write.p999_us | read_modify_write.max_us |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| a | 1 | 1 | 5862 | 60.078 | 98 | 0 | null | 1234.7 | 397983744 | 2931 | 1931.5 | 191.5 | 1703.9 | 71827.5 | 189792.3 | 396594.4 | 2931 | 18559.1 | 1957.9 | 97517.6 | 193986.6 | 492830.7 | 680891 | user6166968228214299628 | 97 | 1.6 | 60.282929 | 7237787 | 3724568 | 3513219 | 0 | 49483776 | 397983744 | since the cgroup was created | 364683264 | 141485056 | 384696320 | 172 | 2931 | 2617 | 0 |  |  |  |  |  |  |  |  |
| a | 16 | 1 | 43784 | 60.105 | 728 | 0 | null | 1037.7 | 564350976 | 21923 | 7736.6 | 389.1 | 71827.5 | 125829.1 | 289407 | 888191.4 | 21861 | 36152.7 | 6389.8 | 173015 | 267386.9 | 402653.2 | 773048.6 | user6166968228214299628 | 766 | 12.7 | 60.301288 | 45436661 | 25039979 | 20396682 | 0 | 125288448 | 564350976 | since the cgroup was created | 453922816 | 298461184 | 559886336 | 40 | 21861 | 17801 | 0 |  |  |  |  |  |  |  |  |
| b | 1 | 1 | 25635 | 60.004 | 427 | 0 | null | 411.4 | 564350976 | 24392 | 1609.9 | 105.5 | 946.2 | 66846.7 | 219152.4 | 761663.3 | 1243 | 16488.5 | 1949.7 | 87556.1 | 176160.8 | 499122.2 | 519700.1 | user6166968228214299628 | 38 | 0.6 | 60.182756 | 10545848 | 6120955 | 4424893 | 0 | 16834560 | 564350976 | since the cgroup was created | 460341248 | 224346112 | 482066432 | 143 | 1243 | 1143 | 0 |  |  |  |  |  |  |  |  |
| b | 16 | 1 | 91881 | 60.228 | 1526 | 0 | null | 543.2 | 591122432 | 87257 | 8970.5 | 206.8 | 76021.8 | 176160.8 | 394264.6 | 990486.6 | 4624 | 38022.6 | 6520.8 | 175112.2 | 293601.3 | 402653.2 | 493482.8 | user6166968228214299628 | 172 | 2.9 | 60.443587 | 49908678 | 30058639 | 19850040 | 0 | 36995072 | 591122432 | since the cgroup was created | 465428480 | 325419008 | 590229504 | 18 | 4624 | 4048 | 0 |  |  |  |  |  |  |  |  |
| c | 1 | 1 | 58497 | 60 | 975 | 0 | null | 304.2 | 591122432 | 58497 | 1012.6 | 103.9 | 448.5 | 13041.7 | 109051.9 | 1082338.3 |  |  |  |  |  |  |  |  |  |  | 60.00973 | 17793052 | 11445322 | 6347730 | 0 | 425984 | 591122432 | since the cgroup was created | 465428480 | 239418368 | 486584320 | 129 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| c | 16 | 1 | 164183 | 60.208 | 2727 | 0 | null | 388.1 | 591122432 | 164183 | 5838.4 | 175.1 | 61079.6 | 100663.3 | 276824.1 | 876757.6 |  |  |  |  |  |  |  |  |  |  | 60.586951 | 63714877 | 38451107 | 25263771 | 0 | 0 | 591122432 | since the cgroup was created | 465453056 | 271435776 | 520613888 | 32 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| f | 1 | 1 | 10219 | 60.003 | 170 | 0 | null | 1001.9 | 591122432 | 5177 | 1578.5 | 144.4 | 1998.8 | 71303.2 | 168820.7 | 387122.1 |  |  |  |  |  |  |  | user6166968228214299628 | 193 | 3.2 | 60.007082 | 10238815 | 5408659 | 4830157 | 0 | 49324032 | 591122432 | since the cgroup was created | 467443712 | 229799936 | 488779776 | 194 | 5042 | 4381 | 0 |  | 5042 | 10272.2 | 1441.8 | 74973.2 | 102760.4 | 198180.9 | 491415.8 |
| f | 16 | 1 | 38194 | 60.322 | 633 | 0 | null | 1226 | 616083456 | 19029 | 8380.7 | 370.7 | 73400.3 | 165675 | 371195.9 | 781370.7 |  |  |  |  |  |  |  | user6166968228214299628 | 723 | 12 | 60.890743 | 46825932 | 26400313 | 20425620 | 0 | 102301696 | 616083456 | since the cgroup was created | 490377216 | 328164352 | 614449152 | 42 | 19165 | 15571 | 0 |  | 19165 | 41825 | 7602.2 | 181403.6 | 287309.8 | 497025 | 885493 |

