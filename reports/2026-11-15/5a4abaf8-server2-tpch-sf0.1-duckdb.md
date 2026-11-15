# tpch-sf0.1-duckdb on server2, 2026-11-15

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-duckdb |
| machine | server2 |
| date | 2026-11-15 |
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
| usage.wall_s | 37.091418 |
| usage.cpu_usec | 49052918 |
| usage.user_usec | 42971218 |
| usage.system_usec | 6081700 |
| usage.rbytes | 112496640 |
| usage.wbytes | 143491072 |
| usage.memory_peak_bytes | 595906560 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 240967680 |
| usage.pss_max_bytes | 375111680 |
| usage.memory_current_max_bytes | 595726336 |
| usage.samples | 329 |
| du_path | /var/tmp/m3w-work/tp/sf01-duck.db |
| du_apparent_bytes | 128462848 |

## totals

| Field | Value |
|---|---|
| hot_s | 4.614 |
| cold_s | 7.255 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 0.679, 0.289, 0.208 | 0.679 | 0.208 | 2162688 | 440664 | 1280849 | 23715840 | 46372864 | 4 | not checked | null | null |
| q2 | 0.406, 0.116, 0.102 | 0.406 | 0.102 | 1347584 | 821794 | 861068 | 20377600 | 43211776 | 44 | not checked | null | null |
| q3 | 0.115, 0.199, 0.189 | 0.115 | 0.189 | 581632 | 419927 | 1039384 | 30519296 | 55710720 | 10 | not checked | null | null |
| q4 | 0.303, 0.2, 0.511 | 0.303 | 0.2 | 167936 | 733354 | 1354869 | 32690176 | 58357760 | 5 | not checked | null | null |
| q5 | 0.198, 0.293, 0.214 | 0.198 | 0.214 | 69632 | 440735 | 1188359 | 36065280 | 62289920 | 5 | not checked | null | null |
| q6 | 0.182, 0.026, 0.088 | 0.182 | 0.026 | 0 | 354120 | 697111 | 16478208 | 41125888 | 1 | not checked | null | null |
| q7 | 0.308, 0.221, 0.417 | 0.308 | 0.221 | 454656 | 717575 | 1596896 | 28274688 | 53669888 | 4 | not checked | null | null |
| q8 | 0.242, 0.317, 0.305 | 0.242 | 0.305 | 0 | 512211 | 1344730 | 26095616 | 52460544 | 2 | not checked | null | null |
| q9 | 0.615, 0.224, 0.216 | 0.615 | 0.216 | 0 | 1380296 | 1620878 | 34205696 | 60997632 | 175 | not checked | null | null |
| q10 | 0.822, 0.658, 0.598 | 0.822 | 0.598 | 0 | 1064081 | 1930976 | 96358400 | 122947584 | 20 | not checked | null | null |
| q11 | 0.221, 0.102, 0.172 | 0.221 | 0.102 | 155648 | 849369 | 1002199 | 18706432 | 43610112 | 22 | not checked | null | null |
| q12 | 0.401, 0.299, 0.458 | 0.401 | 0.299 | 241664 | 533927 | 1399125 | 22036480 | 47072256 | 2 | not checked | null | null |
| q13 | 0.282, 0.382, 0.299 | 0.282 | 0.299 | 36864 | 530688 | 1567475 | 23027712 | 48217088 | 37 | not checked | null | null |
| q14 | 0.117, 0.103, 0.193 | 0.117 | 0.103 | 0 | 320128 | 1375647 | 19775488 | 45980672 | 1 | not checked | null | null |
| q15 | 0.12, 0.177, 0.11 | 0.12 | 0.11 | 73728 | 528155 | 1221643 | 19726336 | 45992960 | 1 | not checked | null | null |
| q16 | 0.474, 0.33, 0.405 | 0.474 | 0.33 | 0 | 859586 | 1502935 | 21295104 | 46901248 | 2762 | not checked | null | null |
| q17 | 0.579, 0.282, 0.129 | 0.579 | 0.129 | 0 | 1271811 | 1348749 | 31903744 | 57154560 | 1 | not checked | null | null |
| q18 | 0.204, 0.234, 0.422 | 0.204 | 0.234 | 131072 | 733509 | 1087560 | 33914880 | 57581568 | 5 | not checked | null | null |
| q19 | 0.291, 0.106, 0.282 | 0.291 | 0.106 | 0 | 593520 | 1085090 | 20865024 | 46972928 | 1 | not checked | null | null |
| q20 | 0.21, 0.205, 0.281 | 0.21 | 0.205 | 0 | 684323 | 1024241 | 24760320 | 52099072 | 9 | not checked | null | null |
| q21 | 0.283, 0.217, 0.29 | 0.283 | 0.217 | 0 | 554217 | 1285289 | 31215616 | 56532992 | 47 | not checked | null | null |
| q22 | 0.203, 0.71, 0.201 | 0.203 | 0.201 | 0 | 627818 | 2704319 | 18255872 | 44965888 | 7 | not checked | null | null |

