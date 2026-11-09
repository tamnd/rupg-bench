# tpch-sf0.1-postgresql on server2, 2026-11-09

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-postgresql |
| machine | server2 |
| date | 2026-11-09 |
| commit | 5a4abaf8 |
| smoke | false |
| scale | 0.1 |
| engine_name | postgresql |
| steps | gen, load, run |
| runs | 3 |
| cold | no: the first run is not cold |
| kernel | 6.8.0-147-generic |
| tpch_tools | /var/tmp/m3w-work/tp/kit/tpch_tools_3.0.1/dbgen |
| engine_version | 19beta4 |
| dbgen | the data was there |
| queries | qgen -d -s 0.1, written to /var/tmp/m3w-work/tp/data/queries |

## engine

| Field | Value |
|---|---|
| mode | server, a new connection for each run |
| host | 127.0.0.1 |
| port | 54329 |
| user | postgres |
| dbname | postgres |
| cgroup | /sys/fs/cgroup/system.slice/m3w-yp.service |
| unit | m3w-yp |

## load

| Field | Value |
|---|---|
| usage.wall_s | 28.899387 |
| usage.cpu_usec | 23144745 |
| usage.user_usec | 18033617 |
| usage.system_usec | 5111128 |
| usage.rbytes | 90112 |
| usage.wbytes | 414883840 |
| usage.memory_peak_bytes | 533872640 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 497930240 |
| usage.pss_max_bytes | 275033088 |
| usage.memory_current_max_bytes | 533610496 |
| usage.samples | 261 |
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
| wall_s | 10.000223 |
| cpu_usec | 37736 |
| user_usec | 0 |
| system_usec | 37737 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 533872640 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 497729536 |
| pss_max_bytes | 218859520 |
| memory_current_max_bytes | 517390336 |
| samples | 96 |

## totals

| Field | Value |
|---|---|
| hot_s | 452.902865 |
| cold_s | 473.050952 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /var/tmp/m3w-work/tp/ans-d |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 1.984622, 1.898256, 2.010298 | 1.984622 | 1.898256 | 0 | 3183581 | 7224168 | 533872640 | 231000064 | 4 | ok | null | null |
| q2 | 0.469818, 0.229786, 0.193618 | 0.469818 | 0.193618 | 0 | 435094 | 386856 | 533872640 | 229500928 | 44 | ok | null | null |
| q3 | 1.190686, 0.611661, 0.604727 | 1.190686 | 0.604727 | 0 | 1179098 | 1324516 | 536629248 | 249953280 | 10 | ok | null | null |
| q4 | 1.295393, 1.199274, 1.195323 | 1.295393 | 1.195323 | 0 | 1318425 | 2764495 | 565469184 | 257006592 | 5 | ok | null | null |
| q5 | 0.479768, 0.407797, 0.494849 | 0.479768 | 0.407797 | 0 | 342999 | 796536 | 565469184 | 254065664 | 5 | ok | null | null |
| q6 | 0.399072, 0.304956, 0.400508 | 0.399072 | 0.304956 | 12288 | 435144 | 964895 | 565469184 | 235760640 | 1 | ok | null | null |
| q7 | 0.677302, 0.41762, 0.486977 | 0.677302 | 0.41762 | 0 | 825220 | 783259 | 565469184 | 249513984 | 4 | ok | null | null |
| q8 | 0.410305, 0.504076, 0.706729 | 0.410305 | 0.504076 | 0 | 381772 | 892316 | 565469184 | 248128512 | 2 | ok | null | null |
| q9 | 1.497538, 1.585116, 1.505235 | 1.497538 | 1.505235 | 0 | 1820382 | 3178441 | 565469184 | 248126464 | 175 | ok | null | null |
| q10 | 0.808629, 0.615841, 0.700451 | 0.808629 | 0.615841 | 0 | 1082440 | 1780631 | 565469184 | 246084608 | 20 | ok | null | null |
| q11 | 0.273594, 0.213232, 0.209417 | 0.273594 | 0.209417 | 0 | 191527 | 380737 | 565469184 | 228665344 | 22 | ok | null | null |
| q12 | 0.887447, 0.78916, 0.811543 | 0.887447 | 0.78916 | 0 | 1097438 | 1947112 | 565469184 | 231380992 | 2 | ok | null | null |
| q13 | 0.987802, 0.901082, 0.88587 | 0.987802 | 0.88587 | 0 | 776897 | 1668943 | 565469184 | 226870272 | 37 | ok | null | null |
| q14 | 0.483953, 0.512183, 0.498652 | 0.483953 | 0.498652 | 0 | 692191 | 1337161 | 565469184 | 237700096 | 1 | ok | null | null |
| q15 | 0.406547, 0.50622, 0.505858 | 0.406547 | 0.505858 | 0 | 677530 | 1086483 | 565469184 | 242913280 | 1 | ok | null | null |
| q16 | 0.395786, 0.296932, 0.301463 | 0.395786 | 0.296932 | 0 | 280433 | 480543 | 565469184 | 226498560 | 2762 | ok | null | null |
| q17 | 211.002909, 192.616105, 183.298759 | 211.002909 | 183.298759 | 229376 | 173240342 | 341132532 | 565469184 | 229947392 | 1 | ok | null | null |
| q18 | 2.589778, 1.72608, 1.874265 | 2.589778 | 1.72608 | 0 | 2330289 | 3200723 | 565469184 | 243423232 | 5 | ok | null | null |
| q19 | 0.70136, 0.593016, 0.52022 | 0.70136 | 0.52022 | 0 | 992118 | 2123310 | 565469184 | 238637056 | 1 | ok | null | null |
| q20 | 244.790347, 255.685985, 309.495234 | 244.790347 | 255.685985 | 0 | 224369897 | 501841116 | 565469184 | 230443008 | 9 | ok | null | null |
| q21 | 0.99885, 0.793765, 0.705953 | 0.99885 | 0.705953 | 3366912 | 1668265 | 2673510 | 565469184 | 253974528 | 47 | ok | null | null |
| q22 | 0.319445, 0.290309, 0.132531 | 0.319445 | 0.132531 | 0 | 233073 | 321333 | 565469184 | 228226048 | 7 | ok | null | null |

