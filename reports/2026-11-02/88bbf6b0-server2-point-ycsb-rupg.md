# ycsb-rupg on server2-point, 2026-11-02

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | ycsb-rupg |
| machine | server2-point |
| date | 2026-11-02 |
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
| server_version | 19.0 (rupg 0.4.18) |

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
| secs | 28.849 |
| statements | CREATE TABLE, COPY FROM STDIN, VACUUM ANALYZE, CHECKPOINT |
| server.wall_s | 28.849504 |
| server.cpu_usec | 22495716 |
| server.user_usec | 11555735 |
| server.system_usec | 10939980 |
| server.rbytes | 0 |
| server.wbytes | 273985536 |
| server.memory_peak_bytes | 569212928 |
| server.memory_peak_scope | since the cgroup was created |
| server.file_max_bytes | 222035968 |
| server.pss_max_bytes | 407607296 |
| server.memory_current_max_bytes | 568954880 |
| server.samples | 205 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.001212 |
| cpu_usec | 4088 |
| user_usec | 3626 |
| system_usec | 462 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 569212928 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 111935488 |
| pss_max_bytes | 250867712 |
| memory_current_max_bytes | 345350144 |
| samples | 75 |

## rows

| workload | clients | pipeline | ops | secs | ops_per_s | errors | first_error | server_cpu_usec_per_op | server_memory_peak | read.count | read.mean_us | read.p50_us | read.p95_us | read.p99_us | read.p999_us | read.max_us | update.count | update.mean_us | update.p50_us | update.p95_us | update.p99_us | update.p999_us | update.max_us | hottest_key.key | hottest_key.updates | hottest_key.updates_per_s | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.acknowledged_updates | check.fields_checked | check.fields_wrong | check.examples | read_modify_write.count | read_modify_write.mean_us | read_modify_write.p50_us | read_modify_write.p95_us | read_modify_write.p99_us | read_modify_write.p999_us | read_modify_write.max_us |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| a | 1 | 1 | 5671 | 60.006 | 95 | 0 | null | 3476 | 569212928 | 2841 | 6042.6 | 872.4 | 54001.7 | 95944.7 | 219152.4 | 287496 | 2830 | 15128.5 | 2981.9 | 77594.6 | 159383.6 | 276824.1 | 377392.4 | user6166968228214299628 | 94 | 1.6 | 60.036365 | 19712237 | 14709081 | 5003156 | 0 | 18022400 | 569212928 | since the cgroup was created | 115081216 | 255440896 | 353259520 | 477 | 2830 | 2530 | 0 |  |  |  |  |  |  |  |  |
| a | 16 | 1 | 11586 | 60.106 | 193 | 0 | null | 5542.4 | 569212928 | 5797 | 50240.4 | 12255.2 | 186646.5 | 291504.1 | 603979.8 | 3917847.6 | 5789 | 115720.8 | 96469 | 283115.5 | 406847.5 | 3103785 | 3302884 | user6166968228214299628 | 181 | 3 | 60.112555 | 64214673 | 43496922 | 20717751 | 0 | 22970368 | 569212928 | since the cgroup was created | 121372672 | 269839360 | 373121024 | 446 | 5789 | 5079 | 0 |  |  |  |  |  |  |  |  |
| b | 1 | 1 | 14337 | 60.002 | 239 | 0 | null | 2780.3 | 569212928 | 13628 | 3424.4 | 725 | 7176.2 | 69206 | 91750.4 | 253474.7 | 709 | 18666.4 | 4358.1 | 78118.9 | 138412 | 695258.5 | 695258.5 | user6166968228214299628 | 20 | 0.3 | 60.003136 | 39860661 | 35080039 | 4780623 | 0 | 4775936 | 569212928 | since the cgroup was created | 122421248 | 268340224 | 373141504 | 557 | 709 | 674 | 0 |  |  |  |  |  |  |  |  |
| b | 16 | 1 | 16185 | 60.09 | 269 | 0 | null | 3656.5 | 569212928 | 15396 | 56650.1 | 17170.4 | 196083.7 | 301989.9 | 633339.9 | 2093178.7 | 789 | 112065.3 | 93847.6 | 299892.7 | 536870.9 | 1397792.4 | 1397792.4 | user6166968228214299628 | 29 | 0.5 | 60.092293 | 59180993 | 49254806 | 9926187 | 0 | 5038080 | 569212928 | since the cgroup was created | 123469824 | 274217984 | 379891712 | 465 | 789 | 747 | 0 |  |  |  |  |  |  |  |  |
| c | 1 | 1 | 17964 | 60.048 | 299 | 0 | null | 2875.8 | 625139712 | 17964 | 3336.4 | 688.1 | 4915.2 | 70254.6 | 114294.8 | 3651843.6 |  |  |  |  |  |  |  |  |  |  | 60.05103 | 51660876 | 44045829 | 7615047 | 0 | 122118144 | 625139712 | since the cgroup was created | 245587968 | 392622080 | 624881664 | 449 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| c | 16 | 1 | 17300 | 60.108 | 288 | 0 | null | 3539.8 | 625139712 | 17300 | 55488.2 | 12648.4 | 204472.3 | 377487.4 | 583008.3 | 765299.2 |  |  |  |  |  |  |  |  |  |  | 60.111624 | 61237975 | 52002987 | 9234989 | 0 | 0 | 625139712 | since the cgroup was created | 122994688 | 272498688 | 379277312 | 476 | 0 | 0 | 0 |  |  |  |  |  |  |  |  |
| f | 1 | 1 | 4457 | 60.074 | 74 | 0 | null | 6398.4 | 625139712 | 2272 | 5192.1 | 1089.5 | 35913.7 | 77594.6 | 167772.2 | 175989.8 |  |  |  |  |  |  |  | user6166968228214299628 | 90 | 1.5 | 60.077028 | 28517703 | 22180414 | 6337288 | 0 | 13582336 | 625139712 | since the cgroup was created | 125091840 | 272518144 | 379883520 | 484 | 2185 | 1970 | 0 |  | 2185 | 22087.1 | 5636.1 | 86507.5 | 174063.6 | 312475.6 | 499426.4 |
| f | 16 | 1 | 7487 | 60.213 | 124 | 0 | null | 8537.9 | 625139712 | 3703 | 62916.2 | 14614.5 | 204472.3 | 364904.4 | 3321888.8 | 3692123.6 |  |  |  |  |  |  |  | user6166968228214299628 | 132 | 2.2 | 60.321014 | 63922907 | 43364675 | 20558231 | 0 | 16478208 | 625139712 | since the cgroup was created | 129286144 | 277396480 | 388759552 | 489 | 3784 | 3355 | 0 |  | 3784 | 192566.2 | 175112.2 | 419430.4 | 675282.9 | 3489660.9 | 3690754.4 |

