# clickbench-duckdb on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | clickbench-duckdb |
| machine | server3 |
| date | 2026-10-07 |
| commit | c935d991 |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| engine_name | duckdb |
| dir | /root/work/cbrun/duckdb |
| steps | load, run, concurrent |
| kernel | 6.8.0-106-generic |
| engine_version | v1.5.6 (Variegata) 069cc9f9b5 |
| data_size_bytes | 149434368 |

## settings

| Field | Value |
|---|---|
| BENCH_DOWNLOAD_SCRIPT | download-hits-parquet-single |
| BENCH_RESTARTABLE | no |
| BENCH_DURABLE | yes |
| BENCH_TRIES | 3 |
| BENCH_QUERIES_FILE | queries.sql |
| BENCH_CHECK_TIMEOUT | 300 |
| BENCH_CONCURRENT_CONNECTIONS | 10 |
| BENCH_CONCURRENT_DURATION | 60 |

## measure

| Field | Value |
|---|---|
| mode | in process, each query in a new cgroup |
| cgroup_prefix | clickbench-duckdb |
| cpus | null |

## load

| Field | Value |
|---|---|
| source | /root/work/data/hits_0.parquet |
| load_time_s | 28.591607 |
| usage.wall_s | 28.591732 |
| usage.cpu_usec | 31816663 |
| usage.user_usec | 26880348 |
| usage.system_usec | 4936314 |
| usage.rbytes | 145637376 |
| usage.wbytes | 149585920 |
| usage.memory_peak_bytes | 1361629184 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 75792384 |
| usage.pss_max_bytes | 1333648384 |
| usage.memory_current_max_bytes | 1361367040 |
| usage.samples | 169 |

## concurrent

| Field | Value |
|---|---|
| lib | /root/work/cbrun/duckdb/../lib/benchmark-common.sh |
| qps | 1.75 |
| error_ratio | 0.934 |
| usage.wall_s | 67.832049 |
| usage.cpu_usec | 146860513 |
| usage.user_usec | 55707560 |
| usage.system_usec | 91152952 |
| usage.rbytes | 166330368 |
| usage.wbytes | 151552 |
| usage.memory_peak_bytes | 338370560 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 165867520 |
| usage.pss_max_bytes | 183604224 |
| usage.memory_current_max_bytes | 333774848 |
| usage.samples | 614 |

## totals

| Field | Value |
|---|---|
| hot_s | 9.755 |
| cold_s | 18.405 |
| failed | 0 |
| checked | 41 |
| wrong | 0 |
| answers_from | ans-pg |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error | tries_s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q0 | 0.08, 0.002, 0.002 | 0.08 | 0.002 | 28545024 | 348202 | 212290 | 34865152 | 22823936 | 1 | ok | null | null | 0.08, 0.002, 0.002 |
| q1 | 0.066, 0.056, 0.012 | 0.066 | 0.012 | 30302208 | 266074 | 203487 | 37871616 | 22556672 | 1 | ok | null | null | 0.066, 0.056, 0.012 |
| q2 | 0.09, 0.018, 0.015 | 0.09 | 0.015 | 30007296 | 299177 | 307587 | 38215680 | 25945088 | 1 | ok | null | null | 0.09, 0.018, 0.015 |
| q3 | 0.052, 0.091, 0.066 | 0.052 | 0.066 | 4689920 | 226830 | 685657 | 34168832 | 25596928 | 1 | ok | null | null | 0.052, 0.091, 0.066 |
| q4 | 0.547, 0.104, 0.16 | 0.547 | 0.104 | 30810112 | 373044 | 506187 | 45658112 | 32877568 | 1 | ok | null | null | 0.547, 0.104, 0.16 |
| q5 | 0.198, 0.077, 0.05 | 0.198 | 0.05 | 30257152 | 253292 | 369256 | 49123328 | 35060736 | 1 | ok | null | null | 0.198, 0.077, 0.05 |
| q6 | 0.171, 0.003, 0.01 | 0.171 | 0.003 | 25395200 | 219875 | 209120 | 31703040 | 23770112 | 1 | ok | null | null | 0.171, 0.003, 0.01 |
| q7 | 0.133, 0.011, 0.015 | 0.133 | 0.011 | 31690752 | 421084 | 360583 | 39313408 | 23031808 | 5 | ok | null | null | 0.133, 0.011, 0.015 |
| q8 | 0.218, 0.224, 0.36 | 0.218 | 0.224 | 5697536 | 393460 | 860710 | 28532736 | 39387136 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 2) | null | 0.218, 0.224, 0.36 |
| q9 | 0.463, 0.512, 0.328 | 0.463 | 0.328 | 30756864 | 555785 | 1021712 | 54857728 | 38269952 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 0.463, 0.512, 0.328 |
| q10 | 0.208, 0.118, 0.04 | 0.208 | 0.04 | 22683648 | 301875 | 394171 | 37752832 | 26651648 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 2) | null | 0.208, 0.118, 0.04 |
| q11 | 0.191, 0.169, 0.039 | 0.191 | 0.039 | 31825920 | 353189 | 383572 | 47603712 | 36700160 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 0.191, 0.169, 0.039 |
| q12 | 0.218, 0.108, 0.123 | 0.218 | 0.108 | 32092160 | 357060 | 493437 | 53682176 | 38245376 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 2) | null | 0.218, 0.108, 0.123 |
| q13 | 0.269, 0.17, 0.097 | 0.269 | 0.097 | 33468416 | 347688 | 455087 | 62148608 | 47880192 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 2) | null | 0.269, 0.17, 0.097 |
| q14 | 0.36, 0.154, 0.051 | 0.36 | 0.051 | 34213888 | 417111 | 391204 | 54935552 | 37692416 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 0.36, 0.154, 0.051 |
| q15 | 0.468, 0.144, 0.119 | 0.468 | 0.119 | 33525760 | 341078 | 526384 | 50200576 | 36943872 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 2) | null | 0.468, 0.144, 0.119 |
| q16 | 0.565, 0.239, 0.237 | 0.565 | 0.237 | 16166912 | 374115 | 701933 | 48676864 | 47697920 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 0.565, 0.239, 0.237 |
| q17 | 0.45, 0.181, 0.138 | 0.45 | 0.138 | 6688768 | 405546 | 637366 | 37994496 | 41197568 | 10 | ok | the number of rows only: LIMIT with no sort key in the output fixes no row | null | 0.45, 0.181, 0.138 |
| q18 | 0.863, 0.576, 0.543 | 0.863 | 0.543 | 38969344 | 1055580 | 1399681 | 107687936 | 89840640 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 4) | null | 0.863, 0.576, 0.543 |
| q19 | 0.07, 0.034, 0.005 | 0.07 | 0.005 | 28827648 | 280561 | 234523 | 36212736 | 24047616 | 0 | ok | null | null | 0.07, 0.034, 0.005 |
| q20 | 0.411, 0.612, 0.315 | 0.411 | 0.315 | 54132736 | 652857 | 1171025 | 97943552 | 61569024 | 1 | ok | null | null | 0.411, 0.612, 0.315 |
| q21 | 0.378, 0.251, 0.239 | 0.378 | 0.239 | 55373824 | 620100 | 792803 | 105148416 | 67940352 | 1 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 0.378, 0.251, 0.239 |
| q22 | 0.455, 0.294, 0.377 | 0.455 | 0.294 | 55873536 | 586122 | 909881 | 114921472 | 74319872 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 4) | null | 0.455, 0.294, 0.377 |
| q23 | 1.292, 0.64, 0.824 | 1.292 | 0.64 | 110710784 | 1220696 | 1789671 | 237793280 | 145713152 | 10 | ok | the number of rows only: LIMIT with no sort key in the output fixes no row | null | 1.292, 0.64, 0.824 |
| q24 | 0.199, 0.133, 0.11 | 0.199 | 0.11 | 36630528 | 357932 | 445991 | 56360960 | 37795840 | 10 | ok | the number of rows only: LIMIT with no sort key in the output fixes no row | null | 0.199, 0.133, 0.11 |
| q25 | 0.133, 0.128, 0.032 | 0.133 | 0.032 | 29159424 | 257868 | 385369 | 41836544 | 30640128 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 1) | null | 0.133, 0.128, 0.032 |
| q26 | 0.246, 0.167, 0.185 | 0.246 | 0.167 | 35053568 | 384518 | 502170 | 53583872 | 35805184 | 10 | ok | the number of rows only: LIMIT with no sort key in the output fixes no row | null | 0.246, 0.167, 0.185 |
| q27 | 0.359, 0.233, 0.233 | 0.359 | 0.233 | 53645312 | 651088 | 947294 | 100159488 | 64519168 | 2 | not checked | not compared: STRLEN counts bytes, and length in the PostgreSQL query counts characters | null | 0.359, 0.233, 0.233 |
| q28 | 1.672, 1.312, 1.523 | 1.672 | 1.312 | 48726016 | 2309515 | 4626503 | 106221568 | 76108800 | 2 | not checked | not compared: STRLEN counts bytes, and length in the PostgreSQL query counts characters | null | 1.672, 1.312, 1.523 |
| q29 | 0.199, 0.102, 0.077 | 0.199 | 0.077 | 28884992 | 392213 | 446126 | 40914944 | 28622848 | 1 | ok | null | null | 0.199, 0.102, 0.077 |
| q30 | 0.352, 0.123, 0.095 | 0.352 | 0.095 | 34680832 | 314405 | 363636 | 57470976 | 44059648 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 0.352, 0.123, 0.095 |
| q31 | 0.369, 0.267, 0.145 | 0.369 | 0.145 | 42463232 | 373890 | 577645 | 80130048 | 55445504 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 0.369, 0.267, 0.145 |
| q32 | 0.85, 0.584, 0.571 | 0.85 | 0.571 | 42110976 | 891890 | 1817556 | 155504640 | 133455872 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 0.85, 0.584, 0.571 |
| q33 | 1.536, 1.056, 1.04 | 1.536 | 1.04 | 52649984 | 1756164 | 2358062 | 181739520 | 147643392 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 2) | null | 1.536, 1.056, 1.04 |
| q34 | 1.035, 0.842, 0.651 | 1.035 | 0.651 | 53329920 | 1512043 | 2163946 | 187052032 | 152428544 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 3) | null | 1.035, 0.842, 0.651 |
| q35 | 0.291, 0.252, 0.186 | 0.291 | 0.186 | 29818880 | 425263 | 615612 | 49201152 | 39035904 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 5) | null | 0.291, 0.252, 0.186 |
| q36 | 0.493, 0.517, 0.616 | 0.493 | 0.517 | 44707840 | 758255 | 1532958 | 130691072 | 92812288 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 2) | null | 0.493, 0.517, 0.616 |
| q37 | 0.185, 0.092, 0.122 | 0.185 | 0.092 | 32518144 | 249997 | 527830 | 55939072 | 29549568 | 10 | ok | all sort keys, and the rows that do not tie with the last row (sort key columns 2) | null | 0.185, 0.092, 0.122 |
| q38 | 0.236, 0.111, 0.108 | 0.236 | 0.108 | 44474368 | 409120 | 622698 | 79126528 | 52377600 | 10 | ok | all sort keys, and the rows that do not tie with the first or the last row (sort key columns 2) | null | 0.236, 0.111, 0.108 |
| q39 | 1.221, 0.576, 0.627 | 1.221 | 0.576 | 58818560 | 1595717 | 2373207 | 205615104 | 163727360 | 10 | ok | all sort keys, and the rows that do not tie with the first or the last row (sort key columns 6) | null | 1.221, 0.576, 0.627 |
| q40 | 0.292, 0.073, 0.087 | 0.292 | 0.073 | 40615936 | 416054 | 576319 | 61943808 | 42127360 | 10 | ok | all sort keys, and the rows that do not tie with the first or the last row (sort key columns 3) | null | 0.292, 0.073, 0.087 |
| q41 | 0.262, 0.042, 0.113 | 0.262 | 0.042 | 35176448 | 337529 | 412382 | 52207616 | 35828736 | 0 | ok | all sort keys, and the rows that do not tie with the first or the last row (sort key columns 3) | null | 0.262, 0.042, 0.113 |
| q42 | 0.259, 0.156, 0.048 | 0.259 | 0.048 | 30711808 | 452144 | 473137 | 44593152 | 33061888 | 10 | ok | all sort keys, and the rows that do not tie with the first or the last row (sort key columns 1) | null | 0.259, 0.156, 0.048 |

