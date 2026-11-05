# ycsb-postgresql on server2-scale, 2026-11-05

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | ycsb-postgresql |
| machine | server2-scale |
| date | 2026-11-05 |
| commit | 9474c37f |
| smoke | false |
| engine_name | postgresql |
| cgroup | /sys/fs/cgroup/system.slice/m3w-yp.service |
| records | 100000 |
| field_count | 10 |
| field_length | 100 |
| distribution | scrambled zipfian, constant 0.99 |
| time_s | 45 |
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
| secs | 21.599 |
| statements | CREATE TABLE, COPY FROM STDIN, VACUUM ANALYZE, CHECKPOINT |
| server.wall_s | 21.599464 |
| server.cpu_usec | 11174676 |
| server.user_usec | 6516461 |
| server.system_usec | 4658214 |
| server.rbytes | 4096 |
| server.wbytes | 347951104 |
| server.memory_peak_bytes | 399060992 |
| server.memory_peak_scope | since the cgroup was created |
| server.file_max_bytes | 331857920 |
| server.pss_max_bytes | 176228352 |
| server.memory_current_max_bytes | 398770176 |
| server.samples | 49 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.07388 |
| cpu_usec | 9806 |
| user_usec | 9807 |
| system_usec | 0 |
| rbytes | 0 |
| wbytes | 28672 |
| memory_peak_bytes | 399060992 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 331972608 |
| pss_max_bytes | 111585280 |
| memory_current_max_bytes | 351027200 |
| samples | 30 |

## rows

| workload | clients | pipeline | ops | secs | ops_per_s | errors | first_error | server_cpu_usec_per_op | server_memory_peak | read.count | read.mean_us | read.p50_us | read.p95_us | read.p99_us | read.p999_us | read.max_us | update.count | update.mean_us | update.p50_us | update.p95_us | update.p99_us | update.p999_us | update.max_us | hottest_key.key | hottest_key.updates | hottest_key.updates_per_s | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.acknowledged_updates | check.fields_checked | check.fields_wrong | check.examples |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| b | 1 | 1 | 20377 | 45.061 | 452 | 0 | null | 433.3 | 416788480 | 19330 | 1224.8 | 121.3 | 626.7 | 9371.6 | 204472.3 | 581549.4 | 1047 | 20114.1 | 2064.4 | 99090.4 | 199229.4 | 385876 | 387984.9 | user6166968228214299628 | 27 | 0.6 | 45.071198 | 8829287 | 4649088 | 4180200 | 974848 | 20504576 | 416788480 | since the cgroup was created | 395538432 | 178490368 | 416788480 | 127 | 1047 | 999 | 0 |  |
| b | 4 | 1 | 67439 | 45.293 | 1489 | 0 | null | 372.5 | 455311360 | 64080 | 1498.6 | 107.5 | 905.2 | 64749.6 | 202375.2 | 793603 | 3359 | 24875.1 | 2703.4 | 106430.5 | 274726.9 | 476053.5 | 673951.5 | user6166968228214299628 | 132 | 2.9 | 45.463324 | 25122018 | 14522062 | 10599957 | 602112 | 37044224 | 455311360 | since the cgroup was created | 426041344 | 220792832 | 454504448 | 61 | 3359 | 2982 | 0 |  |
| c | 1 | 1 | 55329 | 45.012 | 1229 | 0 | null | 279.1 | 455311360 | 55329 | 810 | 91.6 | 266.2 | 3604.5 | 157286.4 | 666720 |  |  |  |  |  |  |  |  |  |  | 45.181404 | 15439784 | 10109041 | 5330743 | 0 | 491520 | 455311360 | since the cgroup was created | 416288768 | 215414784 | 425127936 | 91 | 0 | 0 | 0 |  |
| c | 4 | 1 | 177977 | 45.003 | 3955 | 0 | null | 245.4 | 455311360 | 177977 | 1002.6 | 88.6 | 387.1 | 6848.5 | 181403.6 | 869909 |  |  |  |  |  |  |  |  |  |  | 45.415696 | 43679026 | 27538100 | 16140926 | 0 | 0 | 455311360 | since the cgroup was created | 395038720 | 225822720 | 421486592 | 38 | 0 | 0 | 0 |  |

