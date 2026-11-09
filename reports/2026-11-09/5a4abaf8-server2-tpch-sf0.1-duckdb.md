# tpch-sf0.1-duckdb on server2, 2026-11-09

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-duckdb |
| machine | server2 |
| date | 2026-11-09 |
| commit | 5a4abaf8 |
| smoke | false |
| scale | 0.1 |
| engine_name | duckdb |
| steps | gen, load, run |
| runs | 3 |
| cold | no: the first run is not cold |
| kernel | 6.8.0-147-generic |
| tpch_tools | /var/tmp/m3w-work/tp/kit/tpch_tools_3.0.1/dbgen |
| engine_version | v2.0.0-dev84237 (Development Version) cc7e7bac7f |
| dbgen | the data was there |
| queries | qgen -d -s 0.1, written to /var/tmp/m3w-work/tp/data/queries |

## engine

| Field | Value |
|---|---|
| mode | in process, a new duckdb process for each run |
| bin | /usr/local/bin/duckdb |
| db | /var/tmp/m3w-work/tp/sf01-duck.db |
| threads | null |
| cpus | null |

## load

| Field | Value |
|---|---|
| usage.wall_s | 42.383615 |
| usage.cpu_usec | 56210883 |
| usage.user_usec | 50681544 |
| usage.system_usec | 5529339 |
| usage.rbytes | 112496640 |
| usage.wbytes | 143491072 |
| usage.memory_peak_bytes | 586997760 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 219615232 |
| usage.pss_max_bytes | 387208192 |
| usage.memory_current_max_bytes | 586719232 |
| usage.samples | 393 |
| du_path | /var/tmp/m3w-work/tp/sf01-duck.db |
| du_apparent_bytes | 128462848 |

## totals

| Field | Value |
|---|---|
| hot_s | 4.881 |
| cold_s | 6.398 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 0.299, 0.294, 0.273 | 0.299 | 0.273 | 2166784 | 692117 | 756564 | 23711744 | 46148608 | 4 | not checked | null | null |
| q2 | 0.402, 0.189, 0.103 | 0.402 | 0.103 | 2920448 | 527275 | 1043979 | 21929984 | 44470272 | 44 | not checked | null | null |
| q3 | 0.311, 0.198, 0.287 | 0.311 | 0.198 | 1318912 | 1017740 | 1001328 | 30777344 | 54602752 | 10 | not checked | null | null |
| q4 | 0.298, 0.324, 0.223 | 0.298 | 0.223 | 167936 | 705615 | 1646184 | 32825344 | 57179136 | 5 | not checked | null | null |
| q5 | 0.227, 0.397, 0.219 | 0.227 | 0.219 | 131072 | 1255146 | 1202304 | 36302848 | 61599744 | 5 | not checked | null | null |
| q6 | 0.02, 0.093, 0.101 | 0.02 | 0.093 | 0 | 298611 | 834307 | 16269312 | 38392832 | 1 | not checked | null | null |
| q7 | 0.49, 0.315, 0.39 | 0.49 | 0.315 | 454656 | 998088 | 1658124 | 28307456 | 53541888 | 4 | not checked | null | null |
| q8 | 0.39, 0.209, 0.312 | 0.39 | 0.209 | 1052672 | 897590 | 1450518 | 27127808 | 51270656 | 2 | not checked | null | null |
| q9 | 0.293, 0.382, 0.509 | 0.293 | 0.382 | 262144 | 1059263 | 1818932 | 35377152 | 61889536 | 175 | not checked | null | null |
| q10 | 0.503, 0.693, 0.629 | 0.503 | 0.629 | 786432 | 793621 | 2375876 | 97202176 | 122391552 | 20 | not checked | null | null |
| q11 | 0.21, 0.183, 0.121 | 0.21 | 0.121 | 2514944 | 695595 | 1158903 | 21123072 | 44300288 | 22 | not checked | null | null |
| q12 | 0.213, 0.202, 0.296 | 0.213 | 0.202 | 720896 | 819261 | 1447819 | 22204416 | 47824896 | 2 | not checked | null | null |
| q13 | 0.298, 0.202, 0.21 | 0.298 | 0.202 | 2658304 | 980883 | 1292830 | 26251264 | 47452160 | 37 | not checked | null | null |
| q14 | 0.095, 0.11, 0.168 | 0.095 | 0.11 | 0 | 547886 | 952959 | 19853312 | 44677120 | 1 | not checked | null | null |
| q15 | 0.127, 0.192, 0.187 | 0.127 | 0.187 | 73728 | 501026 | 959138 | 19705856 | 46187520 | 1 | not checked | null | null |
| q16 | 0.386, 0.49, 0.5 | 0.386 | 0.49 | 0 | 969622 | 2312629 | 21700608 | 47379456 | 2762 | not checked | null | null |
| q17 | 0.503, 0.197, 0.276 | 0.503 | 0.197 | 4689920 | 1123397 | 825855 | 36589568 | 56664064 | 1 | not checked | null | null |
| q18 | 0.384, 0.116, 0.289 | 0.384 | 0.116 | 917504 | 766850 | 1414623 | 32821248 | 56996864 | 5 | not checked | null | null |
| q19 | 0.16, 0.041, 0.201 | 0.16 | 0.041 | 262144 | 222986 | 870844 | 20566016 | 45990912 | 1 | not checked | null | null |
| q20 | 0.201, 0.193, 0.204 | 0.201 | 0.193 | 0 | 464929 | 1043971 | 25812992 | 52681728 | 9 | not checked | null | null |
| q21 | 0.397, 0.195, 0.291 | 0.397 | 0.195 | 4816896 | 686744 | 944187 | 37396480 | 56052736 | 47 | not checked | null | null |
| q22 | 0.191, 0.183, 0.231 | 0.191 | 0.183 | 0 | 579787 | 1295525 | 18472960 | 44150784 | 7 | not checked | null | null |

