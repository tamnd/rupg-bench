# clickbench-postgresql on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | clickbench-postgresql |
| machine | server3 |
| date | 2026-10-07 |
| commit | fb76dcfd |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| engine_name | postgresql |
| dir | /root/work/rbdata/clickbench/postgresql |
| steps | load, run, concurrent |
| kernel | 6.8.0-106-generic |
| engine_version | 19beta4 |
| data_size_bytes | 1664993329 |

## settings

| Field | Value |
|---|---|
| BENCH_DOWNLOAD_SCRIPT | download-hits-tsv |
| BENCH_RESTARTABLE | yes |
| BENCH_DURABLE | yes |
| BENCH_TRIES | 3 |
| BENCH_QUERIES_FILE | queries.sql |
| BENCH_CHECK_TIMEOUT | 300 |
| BENCH_CONCURRENT_CONNECTIONS | 10 |
| BENCH_CONCURRENT_DURATION | 60 |

## measure

| Field | Value |
|---|---|
| mode | server, the cgroup of the server |
| cgroup | /sys/fs/cgroup/system.slice/system-postgresql.slice/postgresql@19-main.service |
| unit | postgresql@19-main |

## load

| Field | Value |
|---|---|
| source | /root/work/data/hits-smoke.tsv |
| load_time_s | 34.281468 |
| usage.wall_s | 34.281597 |
| usage.cpu_usec | 9238365 |
| usage.user_usec | 7487402 |
| usage.system_usec | 1750962 |
| usage.rbytes | 9695232 |
| usage.wbytes | 147116032 |
| usage.memory_peak_bytes | 213397504 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 185061376 |
| usage.pss_max_bytes | 104433664 |
| usage.memory_current_max_bytes | 213135360 |
| usage.samples | 244 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.001305 |
| cpu_usec | 54076 |
| user_usec | 15055 |
| system_usec | 39020 |
| rbytes | 1318912 |
| wbytes | 204800 |
| memory_peak_bytes | 213397504 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 130445312 |
| pss_max_bytes | 47579136 |
| memory_current_max_bytes | 143269888 |
| samples | 83 |

## concurrent

| Field | Value |
|---|---|
| lib | /root/work/rbdata/clickbench/postgresql/../lib/benchmark-common.sh |
| qps | 2.733 |
| error_ratio | 0 |
| usage.wall_s | 68.761292 |
| usage.cpu_usec | 42013047 |
| usage.user_usec | 30469096 |
| usage.system_usec | 11543950 |
| usage.rbytes | 91508736 |
| usage.wbytes | 12263424 |
| usage.memory_peak_bytes | 256962560 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 205729792 |
| usage.pss_max_bytes | 156075008 |
| usage.memory_current_max_bytes | 252461056 |
| usage.samples | 174 |

## totals

| Field | Value |
|---|---|
| hot_s | 19.269 |
| cold_s | 76.904 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error | tries_s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q0 | 1.368, 0.157, 0.385 | 1.368 | 0.157 | 84328448 | 329290 | 248672 | 140648448 | 41860096 | 1 | not checked | null | null | 1.368, 0.157, 0.385 |
| q1 | 1.619, 0.519, 0.553 | 1.619 | 0.519 | 84811776 | 432094 | 317961 | 145272832 | 41953280 | 1 | not checked | null | null | 1.619, 0.519, 0.553 |
| q2 | 1.594, 0.3, 0.271 | 1.594 | 0.271 | 84779008 | 426262 | 363197 | 141512704 | 41368576 | 1 | not checked | null | null | 1.594, 0.3, 0.271 |
| q3 | 1.507, 0.483, 0.316 | 1.507 | 0.316 | 84705280 | 467415 | 262107 | 146419712 | 41827328 | 1 | not checked | null | null | 1.507, 0.483, 0.316 |
| q4 | 1.284, 0.311, 0.813 | 1.284 | 0.311 | 84619264 | 352356 | 301451 | 147148800 | 44349440 | 1 | not checked | null | null | 1.284, 0.311, 0.813 |
| q5 | 2.858, 1.526, 0.319 | 2.858 | 0.319 | 84877312 | 701623 | 568013 | 144715776 | 46117888 | 1 | not checked | null | null | 2.858, 1.526, 0.319 |
| q6 | 1.761, 0.128, 0.592 | 1.761 | 0.128 | 84713472 | 347747 | 316971 | 139812864 | 43210752 | 1 | not checked | null | null | 1.761, 0.128, 0.592 |
| q7 | 2.104, 1.027, 0.518 | 2.104 | 0.518 | 85233664 | 496695 | 295402 | 142299136 | 42521600 | 3 | not checked | null | null | 2.104, 1.027, 0.518 |
| q8 | 3.188, 0.905, 2.065 | 3.188 | 0.905 | 85028864 | 671705 | 659610 | 152051712 | 46796800 | 10 | not checked | null | null | 3.188, 0.905, 2.065 |
| q9 | 1.825, 0.643, 0.766 | 1.825 | 0.643 | 85442560 | 551982 | 534532 | 148275200 | 47372288 | 10 | not checked | null | null | 1.825, 0.643, 0.766 |
| q10 | 1.18, 0.295, 0.494 | 1.18 | 0.295 | 85393408 | 386834 | 353662 | 139243520 | 42803200 | 6 | not checked | null | null | 1.18, 0.295, 0.494 |
| q11 | 1.631, 0.52, 0.747 | 1.631 | 0.52 | 85180416 | 376051 | 442250 | 146309120 | 42807296 | 10 | not checked | null | null | 1.631, 0.52, 0.747 |
| q12 | 2.785, 0.31, 0.72 | 2.785 | 0.31 | 85180416 | 625855 | 379765 | 144719872 | 43953152 | 10 | not checked | null | null | 2.785, 0.31, 0.72 |
| q13 | 1.412, 0.72, 0.206 | 1.412 | 0.206 | 85229568 | 425084 | 432302 | 147427328 | 43926528 | 10 | not checked | null | null | 1.412, 0.72, 0.206 |
| q14 | 1.793, 0.304, 0.156 | 1.793 | 0.156 | 85405696 | 533744 | 432929 | 140337152 | 42461184 | 10 | not checked | null | null | 1.793, 0.304, 0.156 |
| q15 | 1.483, 0.365, 0.619 | 1.483 | 0.365 | 84848640 | 434320 | 377425 | 145170432 | 42177536 | 10 | not checked | null | null | 1.483, 0.365, 0.619 |
| q16 | 1.788, 0.458, 0.754 | 1.788 | 0.458 | 84869120 | 600978 | 681085 | 145698816 | 42941440 | 10 | not checked | null | null | 1.788, 0.458, 0.754 |
| q17 | 1.353, 0.228, 0.616 | 1.353 | 0.228 | 85291008 | 503858 | 473476 | 137826304 | 42696704 | 10 | not checked | null | null | 1.353, 0.228, 0.616 |
| q18 | 1.658, 0.497, 0.349 | 1.658 | 0.349 | 85196800 | 701830 | 569145 | 147865600 | 45043712 | 10 | not checked | null | null | 1.658, 0.497, 0.349 |
| q19 | 1.029, 0.232, 0.356 | 1.029 | 0.232 | 84611072 | 431115 | 318029 | 140750848 | 40905728 | 0 | not checked | null | null | 1.029, 0.232, 0.356 |
| q20 | 1.253, 0.194, 0.244 | 1.253 | 0.194 | 84865024 | 613932 | 432085 | 143314944 | 40813568 | 1 | not checked | null | null | 1.253, 0.194, 0.244 |
| q21 | 1.036, 0.489, 0.325 | 1.036 | 0.325 | 85381120 | 368168 | 422259 | 142016512 | 42260480 | 0 | not checked | null | null | 1.036, 0.489, 0.325 |
| q22 | 1.405, 0.836, 0.491 | 1.405 | 0.491 | 85962752 | 675811 | 843029 | 152756224 | 51625984 | 10 | not checked | null | null | 1.405, 0.836, 0.491 |
| q23 | 1.999, 0.591, 0.398 | 1.999 | 0.398 | 84791296 | 683937 | 463446 | 142774272 | 42245120 | 0 | not checked | null | null | 1.999, 0.591, 0.398 |
| q24 | 1.922, 0.261, 0.314 | 1.922 | 0.261 | 84692992 | 642734 | 338790 | 139157504 | 41833472 | 10 | not checked | null | null | 1.922, 0.261, 0.314 |
| q25 | 1.003, 0.276, 0.426 | 1.003 | 0.276 | 84774912 | 361193 | 352878 | 145104896 | 41755648 | 10 | not checked | null | null | 1.003, 0.276, 0.426 |
| q26 | 1.212, 0.478, 0.184 | 1.212 | 0.184 | 84656128 | 400230 | 333255 | 144375808 | 42241024 | 10 | not checked | null | null | 1.212, 0.478, 0.184 |
| q27 | 1.146, 0.421, 0.66 | 1.146 | 0.421 | 85725184 | 473950 | 664299 | 155443200 | 53170176 | 0 | not checked | null | null | 1.146, 0.421, 0.66 |
| q28 | 5.077, 3.522, 3.262 | 5.077 | 3.262 | 85938176 | 2261562 | 3542510 | 142888960 | 43191296 | 0 | not checked | null | null | 5.077, 3.522, 3.262 |
| q29 | 1.695, 0.846, 0.797 | 1.695 | 0.797 | 84738048 | 831988 | 1069527 | 153571328 | 53014528 | 1 | not checked | null | null | 1.695, 0.846, 0.797 |
| q30 | 1.574, 0.307, 0.256 | 1.574 | 0.256 | 85442560 | 533235 | 313404 | 147054592 | 43535360 | 10 | not checked | null | null | 1.574, 0.307, 0.256 |
| q31 | 2.215, 0.464, 0.278 | 2.215 | 0.278 | 85442560 | 584993 | 371043 | 147861504 | 44619776 | 10 | not checked | null | null | 2.215, 0.464, 0.278 |
| q32 | 2.31, 1.019, 1.158 | 2.31 | 1.019 | 85204992 | 826917 | 1132896 | 156643328 | 50298880 | 10 | not checked | null | null | 2.31, 1.019, 1.158 |
| q33 | 1.14, 0.367, 0.427 | 1.14 | 0.367 | 84905984 | 484697 | 513386 | 147001344 | 45036544 | 10 | not checked | null | null | 1.14, 0.367, 0.427 |
| q34 | 2.225, 0.409, 0.405 | 2.225 | 0.405 | 85204992 | 603054 | 519984 | 148520960 | 45734912 | 10 | not checked | null | null | 2.225, 0.409, 0.405 |
| q35 | 2.207, 0.94, 0.497 | 2.207 | 0.497 | 84905984 | 655046 | 536422 | 144932864 | 42963968 | 10 | not checked | null | null | 2.207, 0.94, 0.497 |
| q36 | 1.671, 0.502, 0.862 | 1.671 | 0.502 | 85585920 | 845675 | 502505 | 153251840 | 52759552 | 0 | not checked | null | null | 1.671, 0.502, 0.862 |
| q37 | 1.388, 0.556, 0.393 | 1.388 | 0.393 | 85614592 | 517520 | 560496 | 157241344 | 52308992 | 0 | not checked | null | null | 1.388, 0.556, 0.393 |
| q38 | 1.606, 0.422, 0.394 | 1.606 | 0.394 | 85778432 | 604902 | 588632 | 157708288 | 50183168 | 0 | not checked | null | null | 1.606, 0.422, 0.394 |
| q39 | 2.798, 0.827, 0.407 | 2.798 | 0.407 | 85626880 | 552960 | 603855 | 157827072 | 50567168 | 0 | not checked | null | null | 2.798, 0.827, 0.407 |
| q40 | 1.734, 0.346, 0.469 | 1.734 | 0.346 | 85692416 | 643552 | 678167 | 153382912 | 52759552 | 0 | not checked | null | null | 1.734, 0.346, 0.469 |
| q41 | 1.781, 0.658, 0.384 | 1.781 | 0.384 | 85340160 | 656224 | 579362 | 157933568 | 52618240 | 0 | not checked | null | null | 1.781, 0.658, 0.384 |
| q42 | 1.287, 0.206, 0.242 | 1.287 | 0.206 | 85286912 | 536175 | 499020 | 151982080 | 52610048 | 0 | not checked | null | null | 1.287, 0.206, 0.242 |

