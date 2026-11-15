# tpch-sf0.1-postgresql on server2, 2026-11-15

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-postgresql |
| machine | server2 |
| date | 2026-11-15 |
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
| usage.wall_s | 25.214138 |
| usage.cpu_usec | 22720387 |
| usage.user_usec | 17071076 |
| usage.system_usec | 5649312 |
| usage.rbytes | 28672 |
| usage.wbytes | 414887936 |
| usage.memory_peak_bytes | 533696512 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 498393088 |
| usage.pss_max_bytes | 274209792 |
| usage.memory_current_max_bytes | 533438464 |
| usage.samples | 178 |
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
| wall_s | 10.079648 |
| cpu_usec | 103847 |
| user_usec | 29739 |
| system_usec | 74108 |
| rbytes | 0 |
| wbytes | 8192 |
| memory_peak_bytes | 533696512 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 498192384 |
| pss_max_bytes | 217954304 |
| memory_current_max_bytes | 515338240 |
| samples | 76 |

## totals

| Field | Value |
|---|---|
| hot_s | 410.679801 |
| cold_s | 422.773597 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /var/tmp/m3w-work/tp/ans-d |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 1.40637, 1.510625, 1.381582 | 1.40637 | 1.381582 | 0 | 2845089 | 6449289 | 533696512 | 240777216 | 4 | ok | null | null |
| q2 | 0.193847, 0.172738, 0.367826 | 0.193847 | 0.172738 | 0 | 184398 | 456965 | 533696512 | 227790848 | 44 | ok | null | null |
| q3 | 0.481341, 0.424366, 0.399397 | 0.481341 | 0.399397 | 0 | 661569 | 1112231 | 536682496 | 255663104 | 10 | ok | null | null |
| q4 | 1.409005, 1.112542, 1.098461 | 1.409005 | 1.098461 | 0 | 1495220 | 2790422 | 565809152 | 255848448 | 5 | ok | null | null |
| q5 | 0.300197, 0.30665, 0.488664 | 0.300197 | 0.30665 | 0 | 481794 | 1178448 | 565809152 | 254513152 | 5 | ok | null | null |
| q6 | 0.307914, 0.300872, 0.289365 | 0.307914 | 0.289365 | 12288 | 431166 | 952379 | 565809152 | 225590272 | 1 | ok | null | null |
| q7 | 0.539372, 0.382421, 0.307614 | 0.539372 | 0.307614 | 0 | 736029 | 972337 | 565809152 | 241214464 | 4 | ok | null | null |
| q8 | 0.398388, 0.396991, 0.424306 | 0.398388 | 0.396991 | 0 | 587984 | 1125905 | 565809152 | 271536128 | 2 | ok | null | null |
| q9 | 1.524215, 1.481498, 1.379414 | 1.524215 | 1.379414 | 0 | 1540645 | 3624077 | 565809152 | 252231680 | 175 | ok | null | null |
| q10 | 0.676636, 0.610814, 0.881009 | 0.676636 | 0.610814 | 0 | 969392 | 2014964 | 565809152 | 250277888 | 20 | ok | null | null |
| q11 | 0.383921, 0.21218, 0.297295 | 0.383921 | 0.21218 | 0 | 182541 | 459428 | 565809152 | 230373376 | 22 | ok | null | null |
| q12 | 0.524059, 0.51516, 0.782421 | 0.524059 | 0.51516 | 0 | 661610 | 1847150 | 565809152 | 242885632 | 2 | ok | null | null |
| q13 | 0.801297, 0.801289, 0.796463 | 0.801297 | 0.796463 | 0 | 750334 | 1403600 | 565809152 | 227352576 | 37 | ok | null | null |
| q14 | 0.49669, 0.608504, 0.555133 | 0.49669 | 0.555133 | 0 | 723785 | 1677178 | 565809152 | 239705088 | 1 | ok | null | null |
| q15 | 0.767858, 0.383201, 0.372792 | 0.767858 | 0.372792 | 0 | 719244 | 811976 | 565809152 | 245049344 | 1 | ok | null | null |
| q16 | 0.285217, 0.279459, 0.212481 | 0.285217 | 0.212481 | 0 | 268470 | 413745 | 565809152 | 228037632 | 2762 | ok | null | null |
| q17 | 166.224923, 169.715402, 160.787066 | 166.224923 | 160.787066 | 0 | 150869811 | 306896263 | 565809152 | 229970944 | 1 | ok | null | null |
| q18 | 2.120875, 1.038056, 1.3716 | 2.120875 | 1.038056 | 0 | 1917846 | 2247412 | 565809152 | 262639616 | 5 | ok | null | null |
| q19 | 0.690854, 0.611547, 0.683312 | 0.690854 | 0.611547 | 12288 | 976114 | 2320093 | 565809152 | 233144320 | 1 | ok | null | null |
| q20 | 242.317338, 238.606162, 243.290453 | 242.317338 | 238.606162 | 438272 | 225416132 | 444481069 | 565809152 | 236591104 | 9 | ok | null | null |
| q21 | 0.708572, 0.59781, 0.508853 | 0.708572 | 0.508853 | 3514368 | 1275930 | 1910253 | 565809152 | 257917952 | 47 | ok | null | null |
| q22 | 0.214707, 0.120882, 0.197114 | 0.214707 | 0.120882 | 0 | 210686 | 306380 | 565809152 | 229325824 | 7 | ok | null | null |

