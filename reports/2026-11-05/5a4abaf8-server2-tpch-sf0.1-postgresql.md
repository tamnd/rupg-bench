# tpch-sf0.1-postgresql on server2, 2026-11-05

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-postgresql |
| machine | server2 |
| date | 2026-11-05 |
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
| usage.wall_s | 27.401697 |
| usage.cpu_usec | 23459180 |
| usage.user_usec | 17853201 |
| usage.system_usec | 5605978 |
| usage.rbytes | 0 |
| usage.wbytes | 414863360 |
| usage.memory_peak_bytes | 533078016 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 498384896 |
| usage.pss_max_bytes | 275185664 |
| usage.memory_current_max_bytes | 533045248 |
| usage.samples | 142 |
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
| wall_s | 10.000525 |
| cpu_usec | 188009 |
| user_usec | 106062 |
| system_usec | 81947 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 533078016 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 498184192 |
| pss_max_bytes | 219268096 |
| memory_current_max_bytes | 514682880 |
| samples | 79 |

## totals

| Field | Value |
|---|---|
| hot_s | 424.705782 |
| cold_s | 418.950945 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 2.064907, 1.387758, 1.393398 | 2.064907 | 1.387758 | 0 | 3720405 | 5870874 | 533078016 | 233232384 | 4 | not checked | null | null |
| q2 | 0.758939, 0.109343, 0.111953 | 0.758939 | 0.109343 | 0 | 729949 | 146629 | 533078016 | 229604352 | 44 | not checked | null | null |
| q3 | 0.730919, 0.299265, 0.380148 | 0.730919 | 0.299265 | 0 | 1008051 | 1052108 | 536076288 | 256811008 | 10 | not checked | null | null |
| q4 | 0.901402, 1.197693, 0.914629 | 0.901402 | 0.914629 | 0 | 1040410 | 2511270 | 565092352 | 258494464 | 5 | not checked | null | null |
| q5 | 0.491566, 0.302766, 0.402449 | 0.491566 | 0.302766 | 0 | 582636 | 809762 | 565092352 | 246265856 | 5 | not checked | null | null |
| q6 | 0.223507, 0.385326, 0.304661 | 0.223507 | 0.304661 | 12288 | 315101 | 1137297 | 565092352 | 232365056 | 1 | not checked | null | null |
| q7 | 0.404482, 0.390498, 0.377853 | 0.404482 | 0.377853 | 0 | 571920 | 1180523 | 565092352 | 247503872 | 4 | not checked | null | null |
| q8 | 0.372025, 0.378592, 0.417886 | 0.372025 | 0.378592 | 0 | 612382 | 1216547 | 565092352 | 256779264 | 2 | not checked | null | null |
| q9 | 1.11585, 1.016734, 1.083057 | 1.11585 | 1.016734 | 0 | 1366566 | 3065978 | 565092352 | 247540736 | 175 | not checked | null | null |
| q10 | 0.686877, 0.586765, 0.327429 | 0.686877 | 0.327429 | 0 | 1163360 | 1675473 | 565092352 | 267011072 | 20 | not checked | null | null |
| q11 | 0.19532, 0.199407, 0.275699 | 0.19532 | 0.199407 | 0 | 187370 | 450504 | 565092352 | 229844992 | 22 | not checked | null | null |
| q12 | 0.680457, 0.623908, 0.608826 | 0.680457 | 0.608826 | 0 | 817494 | 1901925 | 565092352 | 235805696 | 2 | not checked | null | null |
| q13 | 0.734663, 0.709207, 1.108434 | 0.734663 | 0.709207 | 0 | 652152 | 1534904 | 565092352 | 227225600 | 37 | not checked | null | null |
| q14 | 0.38128, 0.470642, 0.390602 | 0.38128 | 0.390602 | 0 | 703122 | 1251560 | 565092352 | 252619776 | 1 | not checked | null | null |
| q15 | 0.30088, 0.287262, 0.528701 | 0.30088 | 0.287262 | 0 | 621301 | 1332246 | 565092352 | 224924672 | 1 | not checked | null | null |
| q16 | 0.228011, 0.200032, 0.276658 | 0.228011 | 0.200032 | 0 | 194689 | 455967 | 565092352 | 228187136 | 2762 | not checked | null | null |
| q17 | 166.787953, 172.17491, 168.20057 | 166.787953 | 168.20057 | 0 | 148982540 | 308869774 | 565092352 | 245306368 | 1 | not checked | null | null |
| q18 | 1.877123, 1.709849, 1.893963 | 1.877123 | 1.709849 | 0 | 1686420 | 3206779 | 565092352 | 253479936 | 5 | not checked | null | null |
| q19 | 0.501361, 0.523772, 0.421529 | 0.501361 | 0.421529 | 0 | 1027775 | 1638654 | 565092352 | 241712128 | 1 | not checked | null | null |
| q20 | 238.407146, 245.766947, 248.978599 | 238.407146 | 245.766947 | 0 | 221031476 | 452049049 | 565092352 | 242317312 | 9 | not checked | null | null |
| q21 | 0.702592, 0.699156, 0.59117 | 0.702592 | 0.59117 | 0 | 1091326 | 1806002 | 565092352 | 249618432 | 47 | not checked | null | null |
| q22 | 0.403685, 0.214378, 0.201353 | 0.403685 | 0.201353 | 0 | 145494 | 373936 | 565092352 | 231780352 | 7 | not checked | null | null |

