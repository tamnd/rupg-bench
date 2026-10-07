# tpch-sf0.1-duckdb on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-duckdb |
| machine | server3 |
| date | 2026-10-07 |
| commit | 68cad8fc |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| scale | 0.1 |
| engine_name | duckdb |
| steps | gen, load, run |
| runs | 3 |
| cold | page cache dropped, and a unit restarted, before the first run of each query |
| kernel | 6.8.0-106-generic |
| tpch_tools | /root/work/rupg-bench-opt/src/tpch-tools/tpch_tools_3.0.1/dbgen |
| engine_version | v1.5.6 (Variegata) 069cc9f9b5 |
| dbgen | the data was there |
| queries | qgen -d -s 0.1, written to /root/work/tpch/sf0.1/queries |

## engine

| Field | Value |
|---|---|
| mode | in process, a new duckdb process for each run |
| bin | /root/work/rupg-bench-opt/bin/duckdb |
| db | sf0.1.db |
| threads | null |
| cpus | null |

## load

| Field | Value |
|---|---|
| usage.wall_s | 24.743959 |
| usage.cpu_usec | 30130812 |
| usage.user_usec | 27991927 |
| usage.system_usec | 2138884 |
| usage.rbytes | 112513024 |
| usage.wbytes | 134529024 |
| usage.memory_peak_bytes | 655290368 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 232681472 |
| usage.pss_max_bytes | 453927936 |
| usage.memory_current_max_bytes | 654921728 |
| usage.samples | 200 |
| du_path | sf0.1.db |
| du_apparent_bytes | 120074240 |

## totals

| Field | Value |
|---|---|
| hot_s | 2.329 |
| cold_s | 7.234 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | ans-pg-sf0.1 |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | error |
|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 0.309, 0.085, 0.045 | 0.309 | 0.045 | 35188736 | 402784 | 317896 | 50364416 | 33675264 | 4 | ok | null |
| q2 | 0.333, 0.041, 0.034 | 0.333 | 0.034 | 30879744 | 393125 | 221146 | 45510656 | 31247360 | 44 | ok | null |
| q3 | 0.543, 0.101, 0.105 | 0.543 | 0.101 | 34369536 | 597750 | 444709 | 60870656 | 44546048 | 10 | ok | null |
| q4 | 0.205, 0.097, 0.132 | 0.205 | 0.097 | 33415168 | 287382 | 363539 | 51679232 | 39520256 | 5 | ok | null |
| q5 | 0.354, 0.128, 0.13 | 0.354 | 0.128 | 35889152 | 584643 | 571654 | 68001792 | 52884480 | 5 | ok | null |
| q6 | 0.16, 0.029, 0.042 | 0.16 | 0.029 | 31895552 | 222375 | 190206 | 44437504 | 32413696 | 1 | ok | null |
| q7 | 0.3, 0.117, 0.073 | 0.3 | 0.073 | 36286464 | 388228 | 284373 | 58740736 | 43995136 | 4 | ok | null |
| q8 | 0.332, 0.116, 0.117 | 0.332 | 0.116 | 37433344 | 374056 | 561470 | 57700352 | 42327040 | 2 | ok | null |
| q9 | 0.371, 0.585, 0.288 | 0.371 | 0.288 | 39096320 | 421336 | 1122410 | 75649024 | 55335936 | 175 | ok | null |
| q10 | 0.57, 0.215, 0.44 | 0.57 | 0.215 | 34566144 | 800300 | 1119333 | 128274432 | 113966080 | 20 | ok | null |
| q11 | 0.517, 0.034, 0.077 | 0.517 | 0.034 | 32985088 | 376410 | 250474 | 45797376 | 33646592 | 22 | ok | null |
| q12 | 0.281, 0.161, 0.036 | 0.281 | 0.036 | 33767424 | 348808 | 321382 | 49963008 | 35551232 | 2 | ok | null |
| q13 | 0.29, 0.122, 0.18 | 0.29 | 0.122 | 32972800 | 295089 | 464513 | 51286016 | 34485248 | 37 | ok | null |
| q14 | 0.188, 0.132, 0.092 | 0.188 | 0.092 | 33730560 | 245291 | 344194 | 49979392 | 37365760 | 1 | ok | null |
| q15 | 0.148, 0.037, 0.093 | 0.148 | 0.037 | 34918400 | 266562 | 218868 | 49246208 | 35904512 | 1 | ok | null |
| q16 | 0.2, 0.099, 0.21 | 0.2 | 0.099 | 30658560 | 271296 | 431702 | 52424704 | 41990144 | 2762 | ok | null |
| q17 | 0.357, 0.105, 0.097 | 0.357 | 0.097 | 36802560 | 422765 | 277106 | 55054336 | 37619712 | 1 | ok | null |
| q18 | 0.444, 0.306, 0.211 | 0.444 | 0.211 | 33927168 | 569305 | 619028 | 64315392 | 47244288 | 5 | ok | null |
| q19 | 0.316, 0.088, 0.047 | 0.316 | 0.047 | 33898496 | 377725 | 302102 | 50421760 | 35353600 | 1 | ok | null |
| q20 | 0.269, 0.13, 0.118 | 0.269 | 0.118 | 36847616 | 374351 | 501980 | 57225216 | 40559616 | 9 | ok | null |
| q21 | 0.429, 0.234, 0.231 | 0.429 | 0.231 | 35958784 | 664020 | 655253 | 62898176 | 47067136 | 47 | ok | null |
| q22 | 0.318, 0.124, 0.079 | 0.318 | 0.079 | 30076928 | 328513 | 319729 | 41140224 | 31647744 | 7 | ok | null |

