# tpch-sf0.1-rupg on server2, 2026-11-07

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-rupg |
| machine | server2 |
| date | 2026-11-07 |
| commit | 8c4f3d18 |
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
| usage.wall_s | 215.283042 |
| usage.cpu_usec | 203716145 |
| usage.user_usec | 175690412 |
| usage.system_usec | 28025734 |
| usage.rbytes | 49152 |
| usage.wbytes | 349851648 |
| usage.memory_peak_bytes | 1740873728 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 260599808 |
| usage.pss_max_bytes | 1750294528 |
| usage.memory_current_max_bytes | 1740124160 |
| usage.samples | 1194 |
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
| wall_s | 10.089108 |
| cpu_usec | 3805 |
| user_usec | 0 |
| system_usec | 3805 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 1740873728 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 167677952 |
| pss_max_bytes | 726952960 |
| memory_current_max_bytes | 883204096 |
| samples | 39 |

## totals

| Field | Value |
|---|---|
| hot_s | 45.855423 |
| cold_s | 67.802516 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 17.498603, 0.416253, 0.490168 | 17.498603 | 0.416253 | 0 | 15394327 | 746539 | 1740873728 | 800038912 | 4 | not checked | null | null |
| q2 | 1.518821, 0.877644, 1.005382 | 1.518821 | 0.877644 | 0 | 1463563 | 1691267 | 1740873728 | 746291200 | 44 | not checked | null | null |
| q3 | 2.992809, 1.21741, 0.984741 | 2.992809 | 0.984741 | 0 | 2672801 | 2039914 | 1740873728 | 731680768 | 10 | not checked | null | null |
| q4 | 2.891816, 2.691478, 2.306962 | 2.891816 | 2.306962 | 0 | 2667576 | 4586282 | 1740873728 | 475418624 | 5 | not checked | null | null |
| q5 | 0.926267, 1.174989, 1.22325 | 0.926267 | 1.174989 | 0 | 781545 | 2225859 | 1740873728 | 492213248 | 5 | not checked | null | null |
| q6 | 0.121206, 0.120668, 0.198722 | 0.121206 | 0.120668 | 0 | 107564 | 295186 | 1740873728 | 441307136 | 1 | not checked | null | null |
| q7 | 8.515727, 7.87108, 7.406774 | 8.515727 | 7.406774 | 0 | 7875720 | 14335817 | 1740873728 | 471094272 | 4 | not checked | null | null |
| q8 | 1.008876, 0.858949, 0.709307 | 1.008876 | 0.709307 | 0 | 999630 | 1563043 | 1740873728 | 363440128 | 2 | not checked | null | null |
| q9 | 3.27233, 4.003668, 4.096836 | 3.27233 | 4.003668 | 0 | 3268364 | 8839031 | 1740873728 | 569456640 | 175 | not checked | null | null |
| q10 | 1.093692, 1.400875, 1.288881 | 1.093692 | 1.288881 | 0 | 1085764 | 2572545 | 1740873728 | 379701248 | 20 | not checked | null | null |
| q11 | 0.194042, 0.111872, 0.123697 | 0.194042 | 0.111872 | 0 | 184494 | 233084 | 1740873728 | 360269824 | 22 | not checked | null | null |
| q12 | 0.728746, 0.882153, 0.761494 | 0.728746 | 0.761494 | 0 | 708712 | 1401772 | 1740873728 | 360269824 | 2 | not checked | null | null |
| q13 | 2.390603, 2.283983, 2.19631 | 2.390603 | 2.19631 | 0 | 2363542 | 4413295 | 1740873728 | 438663168 | 37 | not checked | null | null |
| q14 | 0.378383, 0.234001, 0.220185 | 0.378383 | 0.220185 | 0 | 303245 | 448490 | 1740873728 | 438892544 | 1 | not checked | null | null |
| q15 | 0.216673, 0.170932, 0.118765 | 0.216673 | 0.118765 | 0 | 121283 | 204168 | 1740873728 | 472778752 | 1 | not checked | null | null |
| q16 | 0.701879, 0.527118, 0.524906 | 0.701879 | 0.524906 | 0 | 689737 | 974733 | 1740873728 | 474944512 | 2762 | not checked | null | null |
| q17 | 0.758074, 0.620378, 0.691003 | 0.758074 | 0.620378 | 0 | 734944 | 1105458 | 1740873728 | 476218368 | 1 | not checked | null | null |
| q18 | 3.406931, 3.488154, 3.096475 | 3.406931 | 3.096475 | 0 | 3342366 | 6434922 | 1740873728 | 579856384 | 5 | not checked | null | null |
| q19 | 0.883945, 0.607031, 0.51151 | 0.883945 | 0.51151 | 0 | 769146 | 1041320 | 1740873728 | 527877120 | 1 | not checked | null | null |
| q20 | 1.608765, 1.721671, 2.013402 | 1.608765 | 1.721671 | 0 | 1497112 | 3043616 | 1740873728 | 591148032 | 9 | not checked | null | null |
| q21 | 16.085105, 18.678846, 16.289488 | 16.085105 | 16.289488 | 0 | 15245194 | 30498531 | 1740873728 | 1028699136 | 47 | not checked | null | null |
| q22 | 0.609223, 0.39248, 0.579712 | 0.609223 | 0.39248 | 69632 | 582767 | 918392 | 1740873728 | 695239680 | 7 | not checked | null | null |

