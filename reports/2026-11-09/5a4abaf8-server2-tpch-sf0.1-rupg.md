# tpch-sf0.1-rupg on server2, 2026-11-09

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-rupg |
| machine | server2 |
| date | 2026-11-09 |
| commit | 5a4abaf8 |
| smoke | false |
| scale | 0.1 |
| engine_name | rupg |
| steps | gen, load, run |
| runs | 3 |
| cold | no: the first run is not cold |
| kernel | 6.8.0-147-generic |
| tpch_tools | /var/tmp/m3w-work/tp/kit/tpch_tools_3.0.1/dbgen |
| engine_version | 19.0 (rupg 0.4.20) |
| dbgen | the data was there |
| queries | qgen -d -s 0.1, written to /var/tmp/m3w-work/tp/data/queries |

## engine

| Field | Value |
|---|---|
| mode | server, a new connection for each run |
| host | 127.0.0.1 |
| port | 5596 |
| user | postgres |
| dbname | postgres |
| cgroup | /sys/fs/cgroup/system.slice/m3w-yr.service |
| unit | m3w-yr |

## load

| Field | Value |
|---|---|
| usage.wall_s | 257.877664 |
| usage.cpu_usec | 233016354 |
| usage.user_usec | 194547681 |
| usage.system_usec | 38468673 |
| usage.rbytes | 1589248 |
| usage.wbytes | 349851648 |
| usage.memory_peak_bytes | 1691070464 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 219111424 |
| usage.pss_max_bytes | 1634833408 |
| usage.memory_current_max_bytes | 1690939392 |
| usage.samples | 1643 |
| rows.0.table | region |
| rows.0.rows | 5 |
| rows.1.table | nation |
| rows.1.rows | 25 |
| rows.2.table | part |
| rows.2.rows | 20000 |
| rows.3.table | supplier |
| rows.3.rows | 1000 |
| rows.4.table | partsupp |
| rows.4.rows | 80000 |
| rows.5.table | customer |
| rows.5.rows | 15000 |
| rows.6.table | orders |
| rows.6.rows | 150000 |
| rows.7.table | lineitem |
| rows.7.rows | 600572 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.071195 |
| cpu_usec | 71679 |
| user_usec | 58351 |
| system_usec | 13328 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 1691070464 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 166350848 |
| pss_max_bytes | 738911232 |
| memory_current_max_bytes | 893657088 |
| samples | 75 |

## totals

| Field | Value |
|---|---|
| hot_s | 23.521116 |
| cold_s | 47.688558 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /var/tmp/m3w-work/tp/ans-d |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 18.315386, 0.406567, 0.397107 | 18.315386 | 0.397107 | 0 | 15042600 | 585496 | 1691070464 | 814432256 | 4 | ok | null | null |
| q2 | 1.932753, 0.701608, 0.824575 | 1.932753 | 0.701608 | 0 | 1734760 | 1410078 | 1691070464 | 827584512 | 44 | ok | null | null |
| q3 | 3.402681, 0.910395, 0.893758 | 3.402681 | 0.893758 | 0 | 2848724 | 1599366 | 1691070464 | 809843712 | 10 | ok | null | null |
| q4 | 0.911962, 0.78423, 0.794646 | 0.911962 | 0.78423 | 0 | 784222 | 1375297 | 1691070464 | 440791040 | 5 | ok | null | null |
| q5 | 1.324239, 1.411681, 1.213718 | 1.324239 | 1.213718 | 0 | 1187306 | 1993157 | 1691070464 | 460196864 | 5 | ok | null | null |
| q6 | 0.119787, 0.171885, 0.213894 | 0.119787 | 0.171885 | 0 | 112207 | 282650 | 1691070464 | 447953920 | 1 | ok | null | null |
| q7 | 1.002241, 1.092893, 1.120576 | 1.002241 | 1.092893 | 0 | 1004045 | 1714124 | 1691070464 | 373574656 | 4 | ok | null | null |
| q8 | 1.29028, 1.086157, 1.397777 | 1.29028 | 1.086157 | 0 | 1101033 | 2062584 | 1691070464 | 378015744 | 2 | ok | null | null |
| q9 | 2.874993, 2.874846, 3.485354 | 2.874993 | 2.874846 | 0 | 2588818 | 5739818 | 1691070464 | 377752576 | 175 | ok | null | null |
| q10 | 1.401769, 1.41381, 1.409824 | 1.401769 | 1.409824 | 0 | 1281445 | 2286544 | 1691070464 | 290402304 | 20 | ok | null | null |
| q11 | 0.2135, 0.189096, 0.207477 | 0.2135 | 0.189096 | 0 | 125608 | 312946 | 1691070464 | 214490112 | 22 | ok | null | null |
| q12 | 1.002837, 1.009473, 0.886582 | 1.002837 | 0.886582 | 0 | 936303 | 1592867 | 1691070464 | 218602496 | 2 | ok | null | null |
| q13 | 2.077942, 1.994735, 1.810911 | 2.077942 | 1.810911 | 0 | 2002513 | 3447234 | 1691070464 | 288378880 | 37 | ok | null | null |
| q14 | 0.386774, 0.305447, 0.3131 | 0.386774 | 0.305447 | 0 | 302693 | 614552 | 1691070464 | 298449920 | 1 | ok | null | null |
| q15 | 0.299404, 0.187847, 0.207723 | 0.299404 | 0.187847 | 0 | 198186 | 353200 | 1691070464 | 239881216 | 1 | ok | null | null |
| q16 | 0.802956, 0.724506, 0.710436 | 0.802956 | 0.710436 | 0 | 630320 | 1025588 | 1691070464 | 274041856 | 2762 | ok | null | null |
| q17 | 0.899106, 0.716397, 0.793279 | 0.899106 | 0.716397 | 0 | 840683 | 1214518 | 1691070464 | 348502016 | 1 | ok | null | null |
| q18 | 3.97218, 3.606714, 4.316108 | 3.97218 | 3.606714 | 0 | 3427765 | 6953169 | 1691070464 | 551508992 | 5 | ok | null | null |
| q19 | 0.300456, 0.275899, 0.287499 | 0.300456 | 0.275899 | 0 | 287387 | 549336 | 1691070464 | 457034752 | 1 | ok | null | null |
| q20 | 1.693754, 1.318206, 1.377936 | 1.693754 | 1.318206 | 0 | 1453444 | 2354553 | 1691070464 | 526044160 | 9 | ok | null | null |
| q21 | 3.072112, 2.488607, 2.709612 | 3.072112 | 2.488607 | 0 | 2870434 | 4184652 | 1691070464 | 534969344 | 47 | ok | null | null |
| q22 | 0.391447, 0.398948, 0.789863 | 0.391447 | 0.398948 | 0 | 306184 | 754806 | 1691070464 | 304196608 | 7 | ok | null | null |

