# tpch-sf0.1-rupg on server2, 2026-11-15

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-rupg |
| machine | server2 |
| date | 2026-11-15 |
| commit | 5a4abaf8 |
| smoke | false |
| scale | 0.1 |
| engine_name | rupg |
| steps | gen, load, run |
| runs | 3 |
| cold | no: the first run is not cold |
| kernel | 6.8.0-147-generic |
| tpch_tools | /var/tmp/m3w-work/tp/kit/tpch_tools_3.0.1/dbgen |
| engine_version | 19.0 (rupg 0.4.20) |
| dbgen | the data was there |
| queries | qgen -d -s 0.1, written to /var/tmp/m3w-work/tp/data/queries |

## engine

| Field | Value |
|---|---|
| mode | server, a new connection for each run |
| host | 127.0.0.1 |
| port | 5596 |
| user | postgres |
| dbname | postgres |
| cgroup | /sys/fs/cgroup/system.slice/m3w-yr.service |
| unit | m3w-yr |

## load

| Field | Value |
|---|---|
| usage.wall_s | 56.196503 |
| usage.cpu_usec | 53223173 |
| usage.user_usec | 39616096 |
| usage.system_usec | 13607076 |
| usage.rbytes | 0 |
| usage.wbytes | 284364800 |
| usage.memory_peak_bytes | 500948992 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 188801024 |
| usage.pss_max_bytes | 402083840 |
| usage.memory_current_max_bytes | 500813824 |
| usage.samples | 509 |
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

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.01916 |
| cpu_usec | 2245 |
| user_usec | 0 |
| system_usec | 2245 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 500948992 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 95928320 |
| pss_max_bytes | 48683008 |
| memory_current_max_bytes | 128069632 |
| samples | 91 |

## totals

| Field | Value |
|---|---|
| hot_s | 1.930471 |
| cold_s | 4.423975 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /var/tmp/m3w-work/tp/ans-d |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 0.165872, 0.09563, 0.106559 | 0.165872 | 0.09563 | 0 | 161854 | 140616 | 500948992 | 57419776 | 4 | ok | null | null |
| q2 | 0.012646, 0.006404, 0.017402 | 0.012646 | 0.006404 | 0 | 11880 | 16195 | 500948992 | 63002624 | 44 | ok | null | null |
| q3 | 0.097526, 0.083118, 0.110511 | 0.097526 | 0.083118 | 0 | 97301 | 164822 | 500948992 | 78670848 | 10 | ok | null | null |
| q4 | 0.824492, 0.190195, 0.199538 | 0.824492 | 0.190195 | 0 | 817652 | 371282 | 500948992 | 82229248 | 5 | ok | null | null |
| q5 | 0.199853, 0.090923, 0.279183 | 0.199853 | 0.090923 | 0 | 193281 | 297640 | 500948992 | 137123840 | 5 | ok | null | null |
| q6 | 0.07941, 0.016668, 0.102769 | 0.07941 | 0.016668 | 0 | 70514 | 25739 | 500948992 | 138737664 | 1 | ok | null | null |
| q7 | 0.024458, 0.036423, 0.02297 | 0.024458 | 0.02297 | 0 | 22472 | 41748 | 500948992 | 140564480 | 4 | ok | null | null |
| q8 | 0.104504, 0.025045, 0.372127 | 0.104504 | 0.025045 | 0 | 100154 | 428421 | 500948992 | 154018816 | 2 | ok | null | null |
| q9 | 0.20868, 0.12535, 0.109596 | 0.20868 | 0.109596 | 0 | 193322 | 228276 | 500948992 | 128239616 | 175 | ok | null | null |
| q10 | 0.117596, 0.112735, 0.276233 | 0.117596 | 0.112735 | 0 | 114354 | 367251 | 500948992 | 142326784 | 20 | ok | null | null |
| q11 | 0.079399, 0.080182, 0.031945 | 0.079399 | 0.031945 | 0 | 79294 | 99216 | 500948992 | 147397632 | 22 | ok | null | null |
| q12 | 0.090058, 0.097918, 0.108625 | 0.090058 | 0.097918 | 0 | 89911 | 178413 | 500948992 | 152066048 | 2 | ok | null | null |
| q13 | 0.799447, 0.305358, 0.491421 | 0.799447 | 0.305358 | 0 | 590382 | 568026 | 500948992 | 169233408 | 37 | ok | null | null |
| q14 | 0.119284, 0.106474, 0.016992 | 0.119284 | 0.016992 | 0 | 24901 | 110896 | 500948992 | 102976512 | 1 | ok | null | null |
| q15 | 0.092331, 0.113103, 0.100633 | 0.092331 | 0.100633 | 0 | 84286 | 106396 | 500948992 | 103078912 | 1 | ok | null | null |
| q16 | 0.189043, 0.084205, 0.096079 | 0.189043 | 0.084205 | 0 | 80638 | 96504 | 500948992 | 103087104 | 2762 | ok | null | null |
| q17 | 0.105998, 0.196208, 0.083984 | 0.105998 | 0.083984 | 0 | 94924 | 150639 | 500948992 | 126408704 | 1 | ok | null | null |
| q18 | 0.306841, 0.287607, 0.12088 | 0.306841 | 0.12088 | 0 | 225814 | 388610 | 500948992 | 148618240 | 5 | ok | null | null |
| q19 | 0.093097, 0.08811, 0.23121 | 0.093097 | 0.08811 | 0 | 86051 | 477246 | 500948992 | 150199296 | 1 | ok | null | null |
| q20 | 0.192779, 0.109061, 0.021526 | 0.192779 | 0.021526 | 0 | 189493 | 108640 | 500948992 | 110918656 | 9 | ok | null | null |
| q21 | 0.404309, 0.209864, 0.279052 | 0.404309 | 0.209864 | 0 | 398994 | 419082 | 500948992 | 136625152 | 47 | ok | null | null |
| q22 | 0.116353, 0.109192, 0.015772 | 0.116353 | 0.015772 | 0 | 7289 | 46228 | 500948992 | 136624128 | 7 | ok | null | null |

