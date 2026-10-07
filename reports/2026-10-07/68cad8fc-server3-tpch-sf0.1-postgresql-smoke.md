# tpch-sf0.1-postgresql on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-postgresql |
| machine | server3 |
| date | 2026-10-07 |
| commit | 68cad8fc |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| scale | 0.1 |
| engine_name | postgresql |
| steps | gen, load, run |
| runs | 3 |
| cold | page cache dropped, and a unit restarted, before the first run of each query |
| kernel | 6.8.0-106-generic |
| tpch_tools | /root/work/rupg-bench-opt/src/tpch-tools/tpch_tools_3.0.1/dbgen |
| engine_version | 19beta4 |
| dbgen | the data was there |
| queries | qgen -d -s 0.1, written to /root/work/tpch/sf0.1/queries |

## engine

| Field | Value |
|---|---|
| mode | server, a new connection for each run |
| host | /var/run/postgresql |
| port | 5432 |
| user | bench |
| dbname | bench |
| cgroup | /sys/fs/cgroup/system.slice/system-postgresql.slice/postgresql@19-main.service |
| unit | postgresql@19-main |

## load

| Field | Value |
|---|---|
| usage.wall_s | 69.035015 |
| usage.cpu_usec | 27973663 |
| usage.user_usec | 20524087 |
| usage.system_usec | 7449576 |
| usage.rbytes | 269713408 |
| usage.wbytes | 422686720 |
| usage.memory_peak_bytes | 1828225024 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 1782439936 |
| usage.pss_max_bytes | 242919424 |
| usage.memory_current_max_bytes | 1828118528 |
| usage.samples | 260 |
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
| du_path | /var/lib/postgresql/19/main |
| du_apparent_bytes | 1882750078 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.020643 |
| cpu_usec | 5644 |
| user_usec | 1650 |
| system_usec | 3994 |
| rbytes | 0 |
| wbytes | 12288 |
| memory_peak_bytes | 1828225024 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 449437696 |
| pss_max_bytes | 172923904 |
| memory_current_max_bytes | 386658304 |
| samples | 43 |

## totals

| Field | Value |
|---|---|
| hot_s | 720.092964 |
| cold_s | 1018.98956 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | error |
|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 3.699873, 1.711605, 2.837189 | 3.699873 | 1.711605 | 95744000 | 1856890 | 3051911 | 169148416 | 53943296 | 4 | not checked | null |
| q2 | 1.115773, 0.089176, 0.139574 | 1.115773 | 0.089176 | 22274048 | 251118 | 195405 | 105340928 | 62285824 | 44 | not checked | null |
| q3 | 7.066778, 0.794397, 0.434619 | 7.066778 | 0.434619 | 132718592 | 1858622 | 633701 | 306528256 | 153391104 | 10 | not checked | null |
| q4 | 3.133193, 1.74661, 1.418131 | 3.133193 | 1.418131 | 117149696 | 1582309 | 1425041 | 240590848 | 100473856 | 5 | not checked | null |
| q5 | 6.169753, 0.39254, 0.325834 | 6.169753 | 0.325834 | 106921984 | 1948749 | 387656 | 242798592 | 115609600 | 5 | not checked | null |
| q6 | 2.148201, 0.77968, 0.520845 | 2.148201 | 0.520845 | 95379456 | 679647 | 691635 | 168271872 | 53144576 | 1 | not checked | null |
| q7 | 7.757914, 0.409815, 0.61072 | 7.757914 | 0.409815 | 131936256 | 1809086 | 505114 | 297406464 | 148281344 | 4 | not checked | null |
| q8 | 6.873766, 0.284351, 0.46038 | 6.873766 | 0.284351 | 132407296 | 1406818 | 465593 | 293265408 | 141461504 | 2 | not checked | null |
| q9 | 5.194661, 1.987717, 2.324611 | 5.194661 | 1.987717 | 135716864 | 1603913 | 1908035 | 261201920 | 104538112 | 175 | not checked | null |
| q10 | 2.771156, 1.48674, 1.155721 | 2.771156 | 1.155721 | 120721408 | 1164238 | 1348908 | 232816640 | 89790464 | 20 | not checked | null |
| q11 | 0.997151, 0.281911, 0.186747 | 0.997151 | 0.186747 | 17555456 | 314180 | 199789 | 93425664 | 60438528 | 22 | not checked | null |
| q12 | 5.399685, 1.30683, 1.064678 | 5.399685 | 1.064678 | 117063680 | 1127176 | 1082941 | 209612800 | 71996416 | 2 | not checked | null |
| q13 | 1.85567, 0.93036, 0.865376 | 1.85567 | 0.865376 | 24436736 | 495897 | 727156 | 109457408 | 64451584 | 37 | not checked | null |
| q14 | 3.179467, 1.186853, 1.240715 | 3.179467 | 1.186853 | 99028992 | 855173 | 1094935 | 182644736 | 61325312 | 1 | not checked | null |
| q15 | 2.485891, 0.768152, 0.423686 | 2.485891 | 0.423686 | 95789056 | 1055053 | 793157 | 172253184 | 55231488 | 1 | not checked | null |
| q16 | 1.080106, 0.229299, 0.33081 | 1.080106 | 0.229299 | 8396800 | 228583 | 174517 | 73781248 | 46959616 | 2762 | not checked | null |
| q17 | 291.987536, 236.442472, 417.262821 | 291.987536 | 236.442472 | 100372480 | 134403695 | 254129544 | 172015616 | 54751232 | 1 | not checked | null |
| q18 | 7.493631, 1.781786, 2.178705 | 7.493631 | 1.781786 | 131932160 | 2165099 | 1532988 | 315850752 | 178846720 | 5 | not checked | null |
| q19 | 1.826068, 1.315876, 1.35183 | 1.826068 | 1.315876 | 98168832 | 857135 | 1081086 | 166604800 | 51707904 | 1 | not checked | null |
| q20 | 647.61879, 465.327813, 612.635388 | 647.61879 | 465.327813 | 103677952 | 182130125 | 392430941 | 158281728 | 47134720 | 9 | not checked | null |
| q21 | 7.696026, 2.810882, 3.191259 | 7.696026 | 2.810882 | 122368000 | 1505537 | 1462732 | 224534528 | 81711104 | 47 | not checked | null |
| q22 | 1.438472, 0.337683, 0.119684 | 1.438472 | 0.119684 | 26329088 | 293862 | 258728 | 102907904 | 63129600 | 7 | not checked | null |

