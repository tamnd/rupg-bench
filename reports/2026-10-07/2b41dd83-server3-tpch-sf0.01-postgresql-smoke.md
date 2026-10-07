# tpch-sf0.01-postgresql on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | tpch-sf0.01-postgresql |
| machine | server3 |
| date | 2026-10-07 |
| commit | 2b41dd83 |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| scale | 0.01 |
| engine_name | postgresql |
| steps | gen, load, run |
| runs | 3 |
| cold | page cache dropped, and a unit restarted, before the first run of each query |
| kernel | 6.8.0-106-generic |
| tpch_tools | /root/work/rupg-bench-opt/src/tpch-tools/tpch_tools_3.0.1/dbgen |
| engine_version | 19beta4 |
| dbgen | 30.5 s |
| queries | qgen -d -s 0.01, written to /root/work/rbdata/tpch/sf0.01/queries |

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
| usage.wall_s | 39.355385 |
| usage.cpu_usec | 12087275 |
| usage.user_usec | 8740804 |
| usage.system_usec | 3346471 |
| usage.rbytes | 272375808 |
| usage.wbytes | 60731392 |
| usage.memory_peak_bytes | 396636160 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 352534528 |
| usage.pss_max_bytes | 100992000 |
| usage.memory_current_max_bytes | 396210176 |
| usage.samples | 367 |
| rows.0.table | region |
| rows.0.rows | 5 |
| rows.1.table | nation |
| rows.1.rows | 25 |
| rows.2.table | part |
| rows.2.rows | 2000 |
| rows.3.table | supplier |
| rows.3.rows | 100 |
| rows.4.table | partsupp |
| rows.4.rows | 8000 |
| rows.5.table | customer |
| rows.5.rows | 1500 |
| rows.6.table | orders |
| rows.6.rows | 15000 |
| rows.7.table | lineitem |
| rows.7.rows | 60175 |
| du_path | /var/lib/postgresql/19/main |
| du_apparent_bytes | 2405987997 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.169964 |
| cpu_usec | 48267 |
| user_usec | 15273 |
| system_usec | 32994 |
| rbytes | 32768 |
| wbytes | 196608 |
| memory_peak_bytes | 396636160 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 336855040 |
| pss_max_bytes | 57139200 |
| memory_current_max_bytes | 352612352 |
| samples | 98 |

## totals

| Field | Value |
|---|---|
| hot_s | 3.601903 |
| cold_s | 19.776117 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | error |
|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 1.477232, 0.649355, 0.390804 | 1.477232 | 0.390804 | 13242368 | 363842 | 435425 | 76754944 | 51401728 | 4 | not checked | null |
| q2 | 0.599468, 0.024465, 0.019568 | 0.599468 | 0.019568 | 5611520 | 104329 | 33726 | 64626688 | 41968640 | 4 | not checked | null |
| q3 | 1.08388, 0.071606, 0.230695 | 1.08388 | 0.071606 | 17018880 | 180108 | 108394 | 81989632 | 50282496 | 10 | not checked | null |
| q4 | 0.643712, 0.094423, 0.159715 | 0.643712 | 0.094423 | 14655488 | 128009 | 89202 | 82911232 | 55079936 | 5 | not checked | null |
| q5 | 1.000663, 0.104688, 0.067076 | 1.000663 | 0.067076 | 14438400 | 125837 | 44752 | 75829248 | 45712384 | 5 | not checked | null |
| q6 | 0.778528, 0.097727, 0.109142 | 0.778528 | 0.097727 | 12271616 | 155001 | 63491 | 72966144 | 50460672 | 1 | not checked | null |
| q7 | 0.734427, 0.02628, 0.031383 | 0.734427 | 0.02628 | 16785408 | 161775 | 40311 | 80609280 | 48150528 | 4 | not checked | null |
| q8 | 1.439309, 0.075003, 0.126457 | 1.439309 | 0.075003 | 17047552 | 219075 | 39277 | 78532608 | 49329152 | 2 | not checked | null |
| q9 | 0.796649, 0.131207, 0.284365 | 0.796649 | 0.131207 | 17301504 | 162569 | 126974 | 88670208 | 55938048 | 173 | not checked | null |
| q10 | 1.273664, 0.052554, 0.134075 | 1.273664 | 0.052554 | 15757312 | 127388 | 72161 | 84373504 | 55239680 | 20 | not checked | null |
| q11 | 0.646066, 0.060787, 0.02132 | 0.646066 | 0.02132 | 5345280 | 79512 | 21066 | 61706240 | 40397824 | 1 | not checked | null |
| q12 | 0.910078, 0.10465, 0.086989 | 0.910078 | 0.086989 | 14725120 | 150208 | 113362 | 81993728 | 52297728 | 2 | not checked | null |
| q13 | 0.392468, 0.024252, 0.07856 | 0.392468 | 0.024252 | 5332992 | 66972 | 40621 | 64884736 | 42351616 | 33 | not checked | null |
| q14 | 0.315249, 0.075737, 0.075726 | 0.315249 | 0.075726 | 13344768 | 126418 | 67127 | 81784832 | 52040704 | 1 | not checked | null |
| q15 | 0.968587, 0.057823, 0.067618 | 0.968587 | 0.057823 | 12845056 | 145525 | 69754 | 76505088 | 50664448 | 1 | not checked | null |
| q16 | 0.324412, 0.019759, 0.074532 | 0.324412 | 0.019759 | 4001792 | 77486 | 21417 | 55246848 | 38998016 | 296 | not checked | null |
| q17 | 0.342245, 0.006722, 0.059153 | 0.342245 | 0.006722 | 4096000 | 30979 | 35601 | 54816768 | 38323200 | 1 | not checked | null |
| q18 | 1.060642, 0.265089, 0.160327 | 1.060642 | 0.160327 | 15388672 | 233814 | 232458 | 90497024 | 58427392 | 2 | not checked | null |
| q19 | 1.247327, 0.101425, 0.074277 | 1.247327 | 0.074277 | 13434880 | 166080 | 62805 | 80252928 | 49871872 | 1 | not checked | null |
| q20 | 2.880921, 1.970788, 1.936394 | 2.880921 | 1.936394 | 14213120 | 1199725 | 1935421 | 80244736 | 52429824 | 1 | not checked | null |
| q21 | 0.631091, 0.102603, 0.209712 | 0.631091 | 0.102603 | 14614528 | 113043 | 92151 | 82227200 | 50050048 | 1 | not checked | null |
| q22 | 0.229499, 0.025153, 0.009461 | 0.229499 | 0.009461 | 6074368 | 37114 | 47201 | 63053824 | 41117696 | 7 | not checked | null |

