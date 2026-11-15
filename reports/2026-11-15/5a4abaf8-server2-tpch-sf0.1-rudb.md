# tpch-sf0.1-rudb on server2, 2026-11-15

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-rudb |
| machine | server2 |
| date | 2026-11-15 |
| commit | 5a4abaf8 |
| smoke | false |
| scale | 0.1 |
| engine_name | rudb |
| steps | gen, load, run |
| runs | 3 |
| cold | no: the first run is not cold |
| kernel | 6.8.0-147-generic |
| tpch_tools | /var/tmp/m3w-work/tp/kit/tpch_tools_3.0.1/dbgen |
| engine_version | rudb 0.10.16 |
| dbgen | the data was there |
| queries | qgen -d -s 0.1, written to /var/tmp/m3w-work/tp/data-u/queries |

## engine

| Field | Value |
|---|---|
| mode | in process, a new duckdb process for each run |
| bin | /var/tmp/m3w-work/yb/rudbw |
| db | /var/tmp/m3w-work/tp/sf01-rudb.db |
| threads | null |
| cpus | null |

## load

| Field | Value |
|---|---|
| usage.wall_s | 44.324979 |
| usage.cpu_usec | 43361892 |
| usage.user_usec | 36877362 |
| usage.system_usec | 6484529 |
| usage.rbytes | 123613184 |
| usage.wbytes | 119554048 |
| usage.memory_peak_bytes | 557604864 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 234315776 |
| usage.pss_max_bytes | 371622912 |
| usage.memory_current_max_bytes | 551329792 |
| usage.samples | 406 |
| du_path | /var/tmp/m3w-work/tp/sf01-rudb.db |
| du_apparent_bytes | 60345588 |

## totals

| Field | Value |
|---|---|
| hot_s | 3.850696 |
| cold_s | 4.788039 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /var/tmp/m3w-work/tp/ans-d |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 0.70303, 0.395285, 0.469507 | 0.70303 | 0.395285 | 2174976 | 921843 | 1259334 | 10092544 | 21627904 | 4 | ok | null | null |
| q2 | 0.103576, 0.084332, 0.085137 | 0.103576 | 0.084332 | 958464 | 102677 | 406572 | 5021696 | 11694080 | 44 | ok | null | null |
| q3 | 0.170133, 0.019523, 0.101281 | 0.170133 | 0.019523 | 114688 | 120294 | 150467 | 7774208 | 16718848 | 10 | ok | null | null |
| q4 | 0.121425, 0.185276, 0.106478 | 0.121425 | 0.106478 | 0 | 228272 | 417427 | 6615040 | 19316736 | 5 | ok | null | null |
| q5 | 0.115467, 0.202639, 0.195647 | 0.115467 | 0.195647 | 0 | 201608 | 493159 | 8953856 | 22513664 | 5 | ok | null | null |
| q6 | 0.295768, 0.373689, 0.500002 | 0.295768 | 0.373689 | 16384 | 577252 | 1240908 | 4833280 | 17340416 | 1 | ok | null | null |
| q7 | 0.103199, 0.190132, 0.108635 | 0.103199 | 0.108635 | 24576 | 186827 | 539620 | 10289152 | 20584448 | 4 | ok | null | null |
| q8 | 0.104009, 0.094439, 0.192559 | 0.104009 | 0.094439 | 0 | 128992 | 392545 | 9805824 | 23690240 | 2 | ok | null | null |
| q9 | 0.291704, 0.197235, 0.177503 | 0.291704 | 0.177503 | 131072 | 252882 | 467463 | 14372864 | 27449344 | 175 | ok | null | null |
| q10 | 0.208933, 0.211513, 0.296725 | 0.208933 | 0.211513 | 0 | 376451 | 709637 | 15421440 | 27265024 | 20 | ok | null | null |
| q11 | 0.080643, 0.068081, 0.079754 | 0.080643 | 0.068081 | 81920 | 93185 | 427399 | 4825088 | 16689152 | 22 | ok | null | null |
| q12 | 0.591302, 0.579857, 0.406481 | 0.591302 | 0.406481 | 12288 | 792743 | 1279970 | 10682368 | 24188928 | 2 | ok | null | null |
| q13 | 0.239526, 0.106007, 0.101129 | 0.239526 | 0.101129 | 53248 | 255148 | 292511 | 7901184 | 15923200 | 37 | ok | null | null |
| q14 | 0.405872, 0.41272, 0.409984 | 0.405872 | 0.409984 | 0 | 542505 | 1086603 | 5062656 | 20355072 | 1 | ok | null | null |
| q15 | 0.384414, 0.324181, 0.519611 | 0.384414 | 0.324181 | 0 | 523124 | 1440299 | 5959680 | 19930112 | 1 | ok | null | null |
| q16 | 0.115456, 0.100342, 0.022444 | 0.115456 | 0.022444 | 163840 | 185778 | 280576 | 8478720 | 20250624 | 2762 | ok | null | null |
| q17 | 0.097874, 0.104474, 0.018475 | 0.097874 | 0.018475 | 262144 | 171873 | 291945 | 5775360 | 20969472 | 1 | ok | null | null |
| q18 | 0.298864, 0.273369, 0.378064 | 0.298864 | 0.273369 | 0 | 395763 | 764148 | 50196480 | 60708864 | 5 | ok | null | null |
| q19 | 0.020687, 0.084595, 0.09189 | 0.020687 | 0.084595 | 0 | 40118 | 192951 | 4788224 | 18438144 | 1 | ok | null | null |
| q20 | 0.113959, 0.106615, 0.169995 | 0.113959 | 0.106615 | 0 | 70350 | 263171 | 7704576 | 23955456 | 9 | ok | null | null |
| q21 | 0.207192, 0.191086, 0.221475 | 0.207192 | 0.191086 | 32768 | 206391 | 369501 | 11698176 | 24248320 | 47 | ok | null | null |
| q22 | 0.015006, 0.078101, 0.077214 | 0.015006 | 0.077214 | 0 | 95509 | 113069 | 4235264 | 16267264 | 7 | ok | null | null |

