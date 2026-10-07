# ycsb-postgresql on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | ycsb-postgresql |
| machine | server3 |
| date | 2026-10-07 |
| commit | c7a1474 |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| engine_name | postgresql |
| cgroup | /sys/fs/cgroup/system.slice/system-postgresql.slice/postgresql@19-ycsb.service |
| records | 100000 |
| field_count | 10 |
| field_length | 100 |
| distribution | scrambled zipfian, constant 0.99 |
| time_s | 10 |
| seed | 1 |
| kernel | 6.8.0-106-generic |
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
| secs | 43.32 |
| statements | CREATE TABLE, COPY FROM STDIN, VACUUM ANALYZE, CHECKPOINT |
| server.wall_s | 43.319902 |
| server.cpu_usec | 5811178 |
| server.user_usec | 3805322 |
| server.system_usec | 2005855 |
| server.rbytes | 970752 |
| server.wbytes | 275378176 |
| server.memory_peak_bytes | 476504064 |
| server.memory_peak_scope | since the cgroup was created |
| server.file_max_bytes | 420532224 |
| server.pss_max_bytes | 148979712 |
| server.memory_current_max_bytes | 475717632 |
| server.samples | 391 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.00025 |
| cpu_usec | 2755 |
| user_usec | 2755 |
| system_usec | 0 |
| rbytes | 0 |
| wbytes | 8192 |
| memory_peak_bytes | 476504064 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 420536320 |
| pss_max_bytes | 97382400 |
| memory_current_max_bytes | 441950208 |
| samples | 91 |

## rows

| workload | clients | pipeline | ops | secs | ops_per_s | errors | first_error | server_cpu_usec_per_op | server_memory_peak | read.count | read.mean_us | read.p50_us | read.p95_us | read.p99_us | read.p999_us | read.max_us | update.count | update.mean_us | update.p50_us | update.p95_us | update.p99_us | update.p999_us | update.max_us | hottest_key.key | hottest_key.updates | hottest_key.updates_per_s | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.acknowledged_updates | check.fields_checked | check.fields_wrong | check.examples | read_modify_write.count | read_modify_write.mean_us | read_modify_write.p50_us | read_modify_write.p95_us | read_modify_write.p99_us | read_modify_write.p999_us | read_modify_write.max_us |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| a | 1 | 1 | 1175 | 10.004 | 117 | 0 | null | 568.5 | 476504064 | 594 | 3968.5 | 251.9 | 20316.2 | 59244.5 | 134978.7 | 134978.7 | 581 | 13154.6 | 4358.1 | 58196 | 117440.5 | 165644.4 | 165644.4 | user6166968228214299628 | 28 | 2.8 | 10.054025 | 667950 | 282112 | 385839 | 0 | 11657216 | 476504064 | since the cgroup was created | 421134336 | 99959808 | 444829696 | 71 | 581 | 544 | 0 |  |  |  |  |  |  |  |  |
| a | 16 | 1 | 4244 | 10.161 | 418 | 0 | null | 613.8 | 495841280 | 2146 | 16688.4 | 6455.3 | 65273.9 | 150994.9 | 287309.8 | 454837.9 | 2098 | 59468.4 | 43253.8 | 178257.9 | 299892.7 | 492830.7 | 572304.3 | user6166968228214299628 | 59 | 5.8 | 10.181317 | 2604901 | 1346907 | 1257994 | 4096 | 19906560 | 495841280 | since the cgroup was created | 436035584 | 149303296 | 493346816 | 25 | 2098 | 1921 | 0 |  |  |  |  |  |  |  |  |
| a | 16 | 64 | 6592 | 11.142 | 592 | 0 | null | 434.7 | 502247424 | 3270 | 1610676.9 | 1585446.9 | 2449473.5 | 2852126.7 | 3053453.3 | 3081100.3 | 3322 | 1648527.4 | 1619001.3 | 2550136.8 | 2868903.9 | 3070230.5 | 3086286.3 | user6166968228214299628 | 133 | 11.9 | 11.199972 | 2865632 | 1501921 | 1363712 | 0 | 23216128 | 502247424 | since the cgroup was created | 443215872 | 172719104 | 501592064 | 35 | 3322 | 2933 | 0 |  |  |  |  |  |  |  |  |
| b | 1 | 1 | 1380 | 10.003 | 138 | 0 | null | 306.5 | 502247424 | 1297 | 6599.2 | 1982.5 | 37748.7 | 55836.7 | 100139 | 133998 | 83 | 17353.6 | 8650.8 | 58196 | 109023.6 | 109023.6 | 109023.6 | user6166968228214299628 | 4 | 0.4 | 10.010411 | 422915 | 230416 | 192499 | 0 | 1048576 | 502247424 | since the cgroup was created | 443256832 | 147604480 | 467312640 | 67 | 83 | 83 | 0 |  |  |  |  |  |  |  |  |
| b | 16 | 1 | 8991 | 10.472 | 859 | 0 | null | 390.7 | 502571008 | 8510 | 15797.4 | 5636.1 | 69206 | 149946.4 | 261095.4 | 559990.6 | 481 | 55260.6 | 31719.4 | 162529.3 | 278921.2 | 513093.1 | 513093.1 | user6166968228214299628 | 15 | 1.4 | 10.514276 | 3513146 | 1867880 | 1645266 | 0 | 8208384 | 502571008 | since the cgroup was created | 443785216 | 194169856 | 500617216 | 23 | 481 | 461 | 0 |  |  |  |  |  |  |  |  |
| b | 16 | 64 | 22975 | 10.648 | 2158 | 0 | null | 163.3 | 504098816 | 21808 | 450471.8 | 392167.4 | 1069547.5 | 1593835.5 | 2382364.7 | 2687528.8 | 1167 | 575931.5 | 499122.2 | 1283457 | 1761607.7 | 2600468.5 | 2687521.2 | user6166968228214299628 | 40 | 3.8 | 10.763027 | 3751323 | 2209307 | 1542017 | 0 | 7806976 | 504098816 | since the cgroup was created | 444788736 | 201072640 | 503128064 | 18 | 1167 | 1074 | 0 |  |  |  |  |  |  |  |  |
| c | 1 | 1 | 845 | 10.002 | 84 | 0 | null | 372.7 | 504098816 | 845 | 11832.8 | 6029.3 | 45088.8 | 61079.6 | 92183.4 | 92183.4 |  |  |  |  |  |  |  |  |  |  | 10.003639 | 314920 | 182188 | 132732 | 0 | 16384 | 504098816 | since the cgroup was created | 444796928 | 162560000 | 468914176 | 79 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| c | 16 | 1 | 12699 | 10.506 | 1209 | 0 | null | 251.9 | 504098816 | 12699 | 12675.4 | 4882.4 | 57409.5 | 97517.6 | 268435.5 | 735355.6 |  |  |  |  |  |  |  |  |  |  | 10.560977 | 3198525 | 1809114 | 1389410 | 0 | 2514944 | 504098816 | since the cgroup was created | 444874752 | 201925632 | 501895168 | 14 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| c | 16 | 64 | 49683 | 10.746 | 4623 | 0 | null | 87.2 | 504098816 | 49683 | 211578.4 | 156237.8 | 603979.8 | 1090519 | 1904214 | 2103024.4 |  |  |  |  |  |  |  |  |  |  | 10.762982 | 4330335 | 2863736 | 1466598 | 0 | 5242880 | 504098816 | since the cgroup was created | 444956672 | 210733056 | 502505472 | 14 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| f | 1 | 1 | 850 | 10.018 | 85 | 0 | null | 675.2 | 504098816 | 412 | 5112.7 | 231.4 | 30015.5 | 56361 | 98220.1 | 98220.1 |  |  |  |  |  |  |  | user6166968228214299628 | 15 | 1.5 | 10.024015 | 573932 | 273046 | 300887 | 0 | 7667712 | 504098816 | since the cgroup was created | 445243392 | 162529280 | 469237760 | 60 | 438 | 423 | 0 |  | 438 | 18057.1 | 3883 | 86507.5 | 112197.6 | 220006.2 | 220006.2 |
| f | 16 | 1 | 3947 | 10.479 | 377 | 0 | null | 706 | 507092992 | 1933 | 16444.4 | 5603.3 | 74973.2 | 159383.6 | 251658.2 | 278528.7 |  |  |  |  |  |  |  | user6166968228214299628 | 78 | 7.4 | 10.493012 | 2786763 | 1554007 | 1232755 | 0 | 14508032 | 507092992 | since the cgroup was created | 446513152 | 199400448 | 504475648 | 13 | 2014 | 1836 | 0 |  | 2014 | 65790.7 | 47448.1 | 191889.4 | 289407 | 484442.1 | 495561 |
| f | 16 | 64 | 6101 | 11.26 | 542 | 0 | null | 474.8 | 507912192 | 3044 | 1135964.4 | 1056964.6 | 2071986.2 | 2499805.2 | 3003121.7 | 3009860.5 |  |  |  |  |  |  |  | user6166968228214299628 | 123 | 10.9 | 11.489445 | 2896563 | 1555492 | 1341071 | 0 | 16474112 | 507912192 | since the cgroup was created | 448282624 | 201719808 | 506830848 | 23 | 3057 | 2725 | 0 |  | 3057 | 1206048.2 | 1132462.1 | 2181038.1 | 2617245.7 | 2969567.2 | 3011877.1 |

