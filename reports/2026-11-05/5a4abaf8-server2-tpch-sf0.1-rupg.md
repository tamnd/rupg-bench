# tpch-sf0.1-rupg on server2, 2026-11-05

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-rupg |
| machine | server2 |
| date | 2026-11-05 |
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
| usage.wall_s | 265.906138 |
| usage.cpu_usec | 210432408 |
| usage.user_usec | 182979365 |
| usage.system_usec | 27453044 |
| usage.rbytes | 151552 |
| usage.wbytes | 349851648 |
| usage.memory_peak_bytes | 1714466816 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 260468736 |
| usage.pss_max_bytes | 1692511232 |
| usage.memory_current_max_bytes | 1714294784 |
| usage.samples | 697 |
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
| wall_s | 10.000244 |
| cpu_usec | 4869 |
| user_usec | 400 |
| system_usec | 4469 |
| rbytes | 0 |
| wbytes | 0 |
| memory_peak_bytes | 1714466816 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 167591936 |
| pss_max_bytes | 726921216 |
| memory_current_max_bytes | 882905088 |
| samples | 48 |

## totals

| Field | Value |
|---|---|
| hot_s | 135.419932 |
| cold_s | 184.740882 |
| failed | 0 |
| checked | 0 |
| wrong | 0 |
| answers_from | null |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 23.300781, 1.896666, 0.349743 | 23.300781 | 0.349743 | 1392640 | 20936304 | 2835734 | 1714466816 | 816372736 | 4 | not checked | null | null |
| q2 | 0.888182, 0.946798, 0.610509 | 0.888182 | 0.610509 | 0 | 2017448 | 2640797 | 1714466816 | 915517440 | 44 | not checked | null | null |
| q3 | 5.020026, 7.631187, 5.715622 | 5.020026 | 5.715622 | 0 | 5076499 | 10436535 | 1714466816 | 786334720 | 10 | not checked | null | null |
| q4 | 8.296508, 7.778911, 7.797997 | 8.296508 | 7.778911 | 0 | 7568081 | 13824568 | 1714466816 | 1115367424 | 5 | not checked | null | null |
| q5 | 5.323129, 6.106941, 5.394107 | 5.323129 | 5.394107 | 0 | 4759858 | 8928292 | 1714466816 | 1109142528 | 5 | not checked | null | null |
| q6 | 0.283474, 0.295322, 0.188699 | 0.283474 | 0.188699 | 0 | 133175 | 305026 | 1714466816 | 888285184 | 1 | not checked | null | null |
| q7 | 11.785888, 13.570744, 13.065821 | 11.785888 | 13.065821 | 0 | 10049502 | 22940846 | 1714466816 | 873589760 | 4 | not checked | null | null |
| q8 | 3.98902, 4.151121, 4.68571 | 3.98902 | 4.151121 | 0 | 3601208 | 7847195 | 1714466816 | 779401216 | 2 | not checked | null | null |
| q9 | 17.105949, 17.489344, 15.611808 | 17.105949 | 15.611808 | 0 | 15176623 | 29346969 | 1714466816 | 981040128 | 175 | not checked | null | null |
| q10 | 6.706675, 4.323241, 2.331417 | 6.706675 | 2.331417 | 0 | 4381693 | 5999942 | 1714466816 | 907056128 | 20 | not checked | null | null |
| q11 | 0.755991, 0.871126, 0.870476 | 0.755991 | 0.870476 | 0 | 444530 | 1537465 | 1714466816 | 660550656 | 22 | not checked | null | null |
| q12 | 6.086101, 5.101402, 7.570223 | 6.086101 | 5.101402 | 0 | 4837335 | 10731293 | 1714466816 | 894452736 | 2 | not checked | null | null |
| q13 | 2.824129, 2.204428, 2.794159 | 2.824129 | 2.204428 | 0 | 2087530 | 4382666 | 1714466816 | 709277696 | 37 | not checked | null | null |
| q14 | 4.097803, 5.420325, 4.789257 | 4.097803 | 4.789257 | 0 | 3528409 | 8388664 | 1714466816 | 773691392 | 1 | not checked | null | null |
| q15 | 8.783522, 5.221203, 3.210156 | 8.783522 | 3.210156 | 0 | 7550418 | 6158781 | 1714466816 | 940219392 | 1 | not checked | null | null |
| q16 | 0.808849, 0.604266, 0.799198 | 0.808849 | 0.604266 | 0 | 662476 | 1260036 | 1714466816 | 940162048 | 2762 | not checked | null | null |
| q17 | 7.718538, 4.317943, 4.707271 | 7.718538 | 4.317943 | 0 | 6237245 | 7269574 | 1714466816 | 1429615616 | 1 | not checked | null | null |
| q18 | 8.573748, 7.183208, 8.701467 | 8.573748 | 7.183208 | 0 | 6950683 | 13403875 | 1714466816 | 1481250816 | 5 | not checked | null | null |
| q19 | 4.092656, 12.126906, 7.913678 | 4.092656 | 7.913678 | 0 | 3825001 | 22182799 | 2072854528 | 2075960320 | 1 | not checked | null | null |
| q20 | 2.803804, 2.953405, 2.988831 | 2.803804 | 2.953405 | 0 | 3371041 | 5141982 | 2085687296 | 2086069248 | 9 | not checked | null | null |
| q21 | 53.780423, 39.988727, 49.000272 | 53.780423 | 39.988727 | 0 | 44537539 | 76648083 | 2255257600 | 2099211264 | 47 | not checked | null | null |
| q22 | 1.715687, 1.087005, 1.085228 | 1.715687 | 1.085228 | 0 | 1334441 | 1857830 | 2255257600 | 1986841600 | 7 | not checked | null | null |

