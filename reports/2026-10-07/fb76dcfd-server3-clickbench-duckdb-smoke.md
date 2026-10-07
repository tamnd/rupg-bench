# clickbench-duckdb on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | clickbench-duckdb |
| machine | server3 |
| date | 2026-10-07 |
| commit | fb76dcfd |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| engine_name | duckdb |
| dir | /root/work/rbdata/clickbench/duckdb |
| steps | load, run, concurrent |
| kernel | 6.8.0-106-generic |
| engine_version | v1.5.6 (Variegata) 069cc9f9b5 |
| data_size_bytes | 13381632 |

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
| source | /root/work/data/hits-smoke.parquet |
| load_time_s | 3.914014 |
| usage.wall_s | 3.914162 |
| usage.cpu_usec | 3699494 |
| usage.user_usec | 3096161 |
| usage.system_usec | 603332 |
| usage.rbytes | 36864 |
| usage.wbytes | 13393920 |
| usage.memory_peak_bytes | 262672384 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 13422592 |
| usage.pss_max_bytes | 269895680 |
| usage.memory_current_max_bytes | 262422528 |
| usage.samples | 35 |

## concurrent

| Field | Value |
|---|---|
| lib | /root/work/rbdata/clickbench/duckdb/../lib/benchmark-common.sh |
| qps | 3.5 |
| error_ratio | 0.87 |
| usage.wall_s | 63.392371 |
| usage.cpu_usec | 132962753 |
| usage.user_usec | 46472196 |
| usage.system_usec | 86490556 |
| usage.rbytes | 58884096 |
| usage.wbytes | 204800 |
| usage.memory_peak_bytes | 108437504 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 58212352 |
| usage.pss_max_bytes | 60941312 |
| usage.memory_current_max_bytes | 106541056 |
| usage.samples | 520 |

## totals

| Field | Value |
|---|---|
| hot_s | 1.594 |
| cold_s | 9.132 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error | tries_s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q0 | 0.067, 0.012, 0.008 | 0.067 | 0.008 | 25632768 | 137433 | 246352 | 31547392 | 21795840 | 1 | not checked | null | null | 0.067, 0.012, 0.008 |
| q1 | 0.072, 0.004, 0.005 | 0.072 | 0.004 | 27348992 | 160109 | 215769 | 33718272 | 22667264 | 1 | not checked | null | null | 0.072, 0.004, 0.005 |
| q2 | 0.134, 0.005, 0.007 | 0.134 | 0.005 | 26923008 | 151282 | 314057 | 33902592 | 23191552 | 1 | not checked | null | null | 0.134, 0.005, 0.007 |
| q3 | 0.076, 0.003, 0.012 | 0.076 | 0.003 | 26890240 | 131837 | 247861 | 33198080 | 22914048 | 1 | not checked | null | null | 0.076, 0.003, 0.012 |
| q4 | 0.153, 0.017, 0.051 | 0.153 | 0.017 | 27525120 | 291031 | 297992 | 36306944 | 27634688 | 1 | not checked | null | null | 0.153, 0.017, 0.051 |
| q5 | 0.349, 0.046, 0.17 | 0.349 | 0.046 | 26787840 | 285252 | 637323 | 35401728 | 26393600 | 1 | not checked | null | null | 0.349, 0.046, 0.17 |
| q6 | 0.079, 0.002, 0.001 | 0.079 | 0.001 | 25559040 | 220553 | 255524 | 32190464 | 23062528 | 1 | not checked | null | null | 0.079, 0.002, 0.001 |
| q7 | 0.102, 0.003, 0.008 | 0.102 | 0.003 | 28532736 | 191422 | 180508 | 35176448 | 26274816 | 3 | not checked | null | null | 0.102, 0.003, 0.008 |
| q8 | 0.208, 0.039, 0.058 | 0.208 | 0.039 | 28266496 | 221596 | 226750 | 38662144 | 29763584 | 10 | not checked | null | null | 0.208, 0.039, 0.058 |
| q9 | 0.306, 0.092, 0.077 | 0.306 | 0.077 | 28614656 | 369481 | 356899 | 43483136 | 33848320 | 10 | not checked | null | null | 0.306, 0.092, 0.077 |
| q10 | 0.129, 0.078, 0.076 | 0.129 | 0.076 | 28012544 | 280301 | 278385 | 38432768 | 29679616 | 6 | not checked | null | null | 0.129, 0.078, 0.076 |
| q11 | 0.13, 0.055, 0.012 | 0.13 | 0.012 | 28540928 | 219233 | 268901 | 38248448 | 29533184 | 10 | not checked | null | null | 0.13, 0.055, 0.012 |
| q12 | 0.236, 0.022, 0.019 | 0.236 | 0.019 | 27299840 | 158255 | 201773 | 36823040 | 28739584 | 10 | not checked | null | null | 0.236, 0.022, 0.019 |
| q13 | 0.203, 0.115, 0.021 | 0.203 | 0.021 | 28151808 | 327780 | 439949 | 42713088 | 30523392 | 10 | not checked | null | null | 0.203, 0.115, 0.021 |
| q14 | 0.171, 0.055, 0.067 | 0.171 | 0.055 | 28254208 | 182318 | 174789 | 37842944 | 27942912 | 10 | not checked | null | null | 0.171, 0.055, 0.067 |
| q15 | 0.234, 0.091, 0.017 | 0.234 | 0.017 | 29466624 | 226675 | 392072 | 39034880 | 30528512 | 10 | not checked | null | null | 0.234, 0.091, 0.017 |
| q16 | 0.181, 0.078, 0.072 | 0.181 | 0.072 | 29839360 | 218862 | 279296 | 42340352 | 29254656 | 10 | not checked | null | null | 0.181, 0.078, 0.072 |
| q17 | 0.219, 0.049, 0.067 | 0.219 | 0.049 | 29417472 | 181674 | 259116 | 41058304 | 26881024 | 10 | not checked | null | null | 0.219, 0.049, 0.067 |
| q18 | 0.382, 0.12, 0.083 | 0.382 | 0.083 | 30588928 | 312616 | 396767 | 46833664 | 37959680 | 10 | not checked | null | null | 0.382, 0.12, 0.083 |
| q19 | 0.066, 0.002, 0.001 | 0.066 | 0.001 | 25628672 | 179601 | 155615 | 32141312 | 24007680 | 0 | not checked | null | null | 0.066, 0.002, 0.001 |
| q20 | 0.187, 0.059, 0.094 | 0.187 | 0.059 | 28741632 | 269224 | 386030 | 38092800 | 26613760 | 1 | not checked | null | null | 0.187, 0.059, 0.094 |
| q21 | 0.157, 0.098, 0.01 | 0.157 | 0.01 | 30793728 | 228460 | 299702 | 40316928 | 25862144 | 0 | not checked | null | null | 0.157, 0.098, 0.01 |
| q22 | 0.551, 0.172, 0.132 | 0.551 | 0.132 | 35098624 | 552894 | 576590 | 54136832 | 37594112 | 10 | not checked | null | null | 0.551, 0.172, 0.132 |
| q23 | 0.319, 0.191, 0.111 | 0.319 | 0.111 | 38371328 | 435411 | 541699 | 58363904 | 41062400 | 0 | not checked | null | null | 0.319, 0.191, 0.111 |
| q24 | 0.135, 0.053, 0.062 | 0.135 | 0.053 | 29577216 | 301376 | 288765 | 38359040 | 27669504 | 10 | not checked | null | null | 0.135, 0.053, 0.062 |
| q25 | 0.193, 0.011, 0.011 | 0.193 | 0.011 | 26525696 | 234614 | 203716 | 33918976 | 25489408 | 10 | not checked | null | null | 0.193, 0.011, 0.011 |
| q26 | 0.138, 0.01, 0.007 | 0.138 | 0.007 | 27721728 | 182139 | 182637 | 35708928 | 26066944 | 10 | not checked | null | null | 0.138, 0.01, 0.007 |
| q27 | 0.705, 0.069, 0.03 | 0.705 | 0.03 | 29741056 | 594870 | 384608 | 38666240 | 28446720 | 0 | not checked | null | null | 0.705, 0.069, 0.03 |
| q28 | 0.516, 0.138, 0.341 | 0.516 | 0.138 | 29589504 | 560119 | 548785 | 41676800 | 31040512 | 0 | not checked | null | null | 0.516, 0.138, 0.341 |
| q29 | 0.193, 0.075, 0.059 | 0.193 | 0.059 | 27344896 | 243505 | 233594 | 37683200 | 29268992 | 1 | not checked | null | null | 0.193, 0.075, 0.059 |
| q30 | 0.287, 0.019, 0.028 | 0.287 | 0.019 | 30986240 | 517562 | 403440 | 41562112 | 31823872 | 10 | not checked | null | null | 0.287, 0.019, 0.028 |
| q31 | 0.231, 0.028, 0.028 | 0.231 | 0.028 | 31604736 | 242786 | 346647 | 45088768 | 30018560 | 10 | not checked | null | null | 0.231, 0.028, 0.028 |
| q32 | 0.344, 0.15, 0.091 | 0.344 | 0.091 | 31076352 | 266409 | 512123 | 53051392 | 43110400 | 10 | not checked | null | null | 0.344, 0.15, 0.091 |
| q33 | 0.22, 0.067, 0.077 | 0.22 | 0.067 | 29253632 | 353167 | 262843 | 46751744 | 36676608 | 10 | not checked | null | null | 0.22, 0.067, 0.077 |
| q34 | 0.316, 0.167, 0.115 | 0.316 | 0.115 | 29274112 | 419832 | 560310 | 46993408 | 30162944 | 10 | not checked | null | null | 0.316, 0.167, 0.115 |
| q35 | 0.21, 0.024, 0.016 | 0.21 | 0.016 | 29683712 | 319631 | 222236 | 39751680 | 30268416 | 10 | not checked | null | null | 0.21, 0.024, 0.016 |
| q36 | 0.16, 0.068, 0.007 | 0.16 | 0.007 | 26480640 | 277232 | 280187 | 32694272 | 24232960 | 0 | not checked | null | null | 0.16, 0.068, 0.007 |
| q37 | 0.104, 0.012, 0.007 | 0.104 | 0.007 | 26136576 | 256461 | 235148 | 32628736 | 24344576 | 0 | not checked | null | null | 0.104, 0.012, 0.007 |
| q38 | 0.107, 0.004, 0.002 | 0.107 | 0.002 | 26193920 | 271243 | 257313 | 32632832 | 21793792 | 0 | not checked | null | null | 0.107, 0.004, 0.002 |
| q39 | 0.218, 0.012, 0.011 | 0.218 | 0.011 | 27025408 | 189705 | 202780 | 33685504 | 23945216 | 0 | not checked | null | null | 0.218, 0.012, 0.011 |
| q40 | 0.196, 0.009, 0.004 | 0.196 | 0.004 | 28336128 | 237966 | 207817 | 34594816 | 24378368 | 0 | not checked | null | null | 0.196, 0.009, 0.004 |
| q41 | 0.099, 0.005, 0.008 | 0.099 | 0.005 | 27303936 | 200679 | 259846 | 33984512 | 24275968 | 0 | not checked | null | null | 0.099, 0.005, 0.008 |
| q42 | 0.049, 0.004, 0.004 | 0.049 | 0.004 | 26284032 | 172891 | 182267 | 32628736 | 24460288 | 0 | not checked | null | null | 0.049, 0.004, 0.004 |

