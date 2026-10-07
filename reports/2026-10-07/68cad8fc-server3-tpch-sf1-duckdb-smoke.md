# tpch-sf1-duckdb on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | tpch-sf1-duckdb |
| machine | server3 |
| date | 2026-10-07 |
| commit | 68cad8fc |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| scale | 1 |
| engine_name | duckdb |
| steps | gen, load, run |
| runs | 3 |
| cold | page cache dropped, and a unit restarted, before the first run of each query |
| kernel | 6.8.0-106-generic |
| tpch_tools | /root/work/rupg-bench-opt/src/tpch-tools/tpch_tools_3.0.1/dbgen |
| engine_version | v1.5.6 (Variegata) 069cc9f9b5 |
| dbgen | the data was there |
| queries | qgen -d -s 1, written to /root/work/tpch/sf1/queries |

## engine

| Field | Value |
|---|---|
| mode | in process, a new duckdb process for each run |
| bin | /root/work/rupg-bench-opt/bin/duckdb |
| db | sf1.db |
| threads | null |
| cpus | null |

## load

| Field | Value |
|---|---|
| usage.wall_s | 317.97629 |
| usage.cpu_usec | 369190913 |
| usage.user_usec | 344937925 |
| usage.system_usec | 24252988 |
| usage.rbytes | 1206829056 |
| usage.wbytes | 1164902400 |
| usage.memory_peak_bytes | 2886647808 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 477151232 |
| usage.pss_max_bytes | 2490030080 |
| usage.memory_current_max_bytes | 2868465664 |
| usage.samples | 891 |
| du_path | sf1.db |
| du_apparent_bytes | 1158688768 |

## totals

| Field | Value |
|---|---|
| hot_s | 12.723 |
| cold_s | 19.995 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /root/work/rupg-bench-opt/src/tpch-tools/tpch_tools_3.0.1/dbgen/answers |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | error |
|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 1.305, 0.726, 0.687 | 1.305 | 0.687 | 92164096 | 1107866 | 1671186 | 165662720 | 93681664 | 4 | ok | null |
| q2 | 0.859, 0.354, 0.291 | 0.859 | 0.291 | 36597760 | 906287 | 861700 | 68259840 | 51505152 | 100 | ok | null |
| q3 | 1.466, 1.149, 0.897 | 1.466 | 0.897 | 86593536 | 1192430 | 1390720 | 172437504 | 106147840 | 10 | ok | null |
| q4 | 0.996, 0.442, 0.67 | 0.996 | 0.442 | 69775360 | 907403 | 1482185 | 132485120 | 84205568 | 5 | ok | null |
| q5 | 0.898, 0.781, 0.606 | 0.898 | 0.606 | 89329664 | 1009254 | 1584408 | 177721344 | 109910016 | 5 | ok | null |
| q6 | 0.573, 0.294, 0.198 | 0.573 | 0.198 | 78327808 | 477225 | 656641 | 141168640 | 81210368 | 1 | ok | null |
| q7 | 0.727, 0.496, 0.637 | 0.727 | 0.496 | 97800192 | 1158417 | 1464491 | 204210176 | 127633408 | 4 | ok | null |
| q8 | 0.827, 0.693, 0.846 | 0.827 | 0.693 | 105639936 | 944720 | 1753011 | 202895360 | 117622784 | 2 | ok | null |
| q9 | 1.685, 1.778, 1.869 | 1.685 | 1.778 | 114159616 | 2325614 | 4979975 | 340492288 | 245851136 | 175 | ok | null |
| q10 | 1.695, 1.545, 0.963 | 1.695 | 0.963 | 86630400 | 2196591 | 3005164 | 214949888 | 148724736 | 20 | ok | null |
| q11 | 0.293, 0.151, 0.098 | 0.293 | 0.098 | 35549184 | 473938 | 432952 | 60534784 | 45494272 | 1048 | ok | null |
| q12 | 0.671, 0.459, 0.428 | 0.671 | 0.428 | 80027648 | 946039 | 1242939 | 152731648 | 92636160 | 2 | ok | null |
| q13 | 1.448, 1.158, 1.303 | 1.448 | 1.158 | 59904000 | 1765040 | 3448533 | 162025472 | 121973760 | 42 | ok | null |
| q14 | 0.71, 0.431, 0.708 | 0.71 | 0.431 | 85831680 | 867307 | 1624678 | 173191168 | 107682816 | 1 | ok | null |
| q15 | 0.599, 0.296, 0.376 | 0.599 | 0.296 | 87232512 | 831723 | 937424 | 163409920 | 96033792 | 1 | ok | null |
| q16 | 0.616, 0.247, 0.299 | 0.616 | 0.247 | 33280000 | 734311 | 1029202 | 91893760 | 78976000 | 18314 | ok | null |
| q17 | 0.519, 0.311, 0.505 | 0.519 | 0.311 | 79548416 | 858478 | 1243465 | 150016000 | 91304960 | 1 | ok | null |
| q18 | 0.953, 0.877, 0.672 | 0.953 | 0.672 | 64598016 | 1839246 | 3265183 | 183369728 | 138829824 | 57 | ok | null |
| q19 | 0.688, 0.564, 0.506 | 0.688 | 0.506 | 90894336 | 984732 | 1574562 | 183476224 | 111898624 | 1 | ok | null |
| q20 | 0.921, 0.523, 0.406 | 0.921 | 0.406 | 92307456 | 960081 | 1233448 | 177905664 | 104924160 | 186 | ok | null |
| q21 | 1.211, 1.074, 0.996 | 1.211 | 0.996 | 78823424 | 1780151 | 3581639 | 193224704 | 132343808 | 100 | ok | null |
| q22 | 0.335, 0.137, 0.123 | 0.335 | 0.123 | 35520512 | 518641 | 652406 | 59199488 | 42386432 | 7 | ok | null |

