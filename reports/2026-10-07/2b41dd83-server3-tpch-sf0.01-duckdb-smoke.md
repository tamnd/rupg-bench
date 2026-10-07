# tpch-sf0.01-duckdb on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | tpch-sf0.01-duckdb |
| machine | server3 |
| date | 2026-10-07 |
| commit | 2b41dd83 |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| scale | 0.01 |
| engine_name | duckdb |
| steps | load, run |
| runs | 3 |
| cold | page cache dropped, and a unit restarted, before the first run of each query |
| kernel | 6.8.0-106-generic |
| tpch_tools | /root/work/rupg-bench-opt/src/tpch-tools/tpch_tools_3.0.1/dbgen |
| engine_version | v1.5.6 (Variegata) 069cc9f9b5 |
| queries | qgen -d -s 0.01, written to /root/work/rbdata/tpch/sf0.01/queries |

## engine

| Field | Value |
|---|---|
| mode | in process, a new duckdb process for each run |
| bin | /root/work/rupg-bench-opt/bin/duckdb |
| db | /root/work/rbdata/tpch/sf0.01.db |
| threads | null |
| cpus | null |

## load

| Field | Value |
|---|---|
| usage.wall_s | 3.611612 |
| usage.cpu_usec | 2723550 |
| usage.user_usec | 2260089 |
| usage.system_usec | 463461 |
| usage.rbytes | 15241216 |
| usage.wbytes | 18726912 |
| usage.memory_peak_bytes | 129880064 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 33632256 |
| usage.pss_max_bytes | 115119104 |
| usage.memory_current_max_bytes | 127934464 |
| usage.samples | 37 |
| du_path | /root/work/rbdata/tpch/sf0.01.db |
| du_apparent_bytes | 15478784 |

## totals

| Field | Value |
|---|---|
| hot_s | 0.935 |
| cold_s | 4.82 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /root/work/rbdata/tpch/answers-sf0.01 |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | error |
|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 0.104, 0.017, 0.011 | 0.104 | 0.011 | 29171712 | 182488 | 194858 | 37134336 | 28016640 | 4 | ok | null |
| q2 | 0.2, 0.072, 0.061 | 0.2 | 0.061 | 29671424 | 172434 | 300359 | 41734144 | 23802880 | 4 | ok | null |
| q3 | 0.338, 0.054, 0.097 | 0.338 | 0.054 | 6111232 | 265095 | 266795 | 18042880 | 20034560 | 10 | ok | null |
| q4 | 0.283, 0.19, 0.075 | 0.283 | 0.075 | 6135808 | 261320 | 325697 | 16162816 | 21740544 | 5 | ok | null |
| q5 | 0.314, 0.032, 0.081 | 0.314 | 0.032 | 7417856 | 226586 | 223012 | 19144704 | 19428352 | 5 | ok | null |
| q6 | 0.085, 0.016, 0.009 | 0.085 | 0.009 | 27852800 | 255147 | 138199 | 35151872 | 23998464 | 1 | ok | null |
| q7 | 0.196, 0.139, 0.052 | 0.196 | 0.052 | 30883840 | 244060 | 258885 | 43581440 | 27906048 | 4 | ok | null |
| q8 | 0.2, 0.033, 0.049 | 0.2 | 0.033 | 33005568 | 239167 | 241838 | 45879296 | 32824320 | 2 | ok | null |
| q9 | 0.522, 0.082, 0.051 | 0.522 | 0.051 | 30466048 | 478694 | 255793 | 43696128 | 34204672 | 173 | ok | null |
| q10 | 0.25, 0.104, 0.068 | 0.25 | 0.068 | 30158848 | 349928 | 353112 | 50319360 | 39513088 | 20 | ok | null |
| q11 | 0.212, 0.025, 0.018 | 0.212 | 0.018 | 29913088 | 245836 | 152744 | 40095744 | 28240896 | 1 | ok | null |
| q12 | 0.137, 0.014, 0.14 | 0.137 | 0.014 | 29650944 | 227251 | 266275 | 38879232 | 26902528 | 2 | ok | null |
| q13 | 0.103, 0.073, 0.019 | 0.103 | 0.019 | 29003776 | 311387 | 282938 | 38711296 | 28805120 | 33 | ok | null |
| q14 | 0.29, 0.019, 0.023 | 0.29 | 0.019 | 29077504 | 271825 | 275551 | 37105664 | 27640832 | 1 | ok | null |
| q15 | 0.227, 0.104, 0.025 | 0.227 | 0.025 | 4182016 | 173985 | 210625 | 12742656 | 27586560 | 1 | ok | null |
| q16 | 0.147, 0.04, 0.026 | 0.147 | 0.026 | 12050432 | 198393 | 211956 | 26558464 | 26917888 | 296 | ok | null |
| q17 | 0.111, 0.039, 0.018 | 0.111 | 0.018 | 29216768 | 161428 | 177829 | 38027264 | 24092672 | 1 | ok | null |
| q18 | 0.179, 0.108, 0.148 | 0.179 | 0.108 | 30822400 | 199637 | 397614 | 45371392 | 32861184 | 2 | ok | null |
| q19 | 0.195, 0.03, 0.015 | 0.195 | 0.015 | 29253632 | 172502 | 135448 | 37543936 | 26272768 | 1 | ok | null |
| q20 | 0.247, 0.304, 0.067 | 0.247 | 0.067 | 31969280 | 202181 | 424199 | 44728320 | 33037312 | 1 | ok | null |
| q21 | 0.227, 0.143, 0.207 | 0.227 | 0.143 | 20967424 | 213627 | 427894 | 34492416 | 25544704 | 1 | ok | null |
| q22 | 0.253, 0.017, 0.103 | 0.253 | 0.017 | 29495296 | 286805 | 238825 | 39428096 | 29419520 | 7 | ok | null |

