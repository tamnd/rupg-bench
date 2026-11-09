# tpch-sf0.1-duckdb on server2, 2026-11-09

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-duckdb |
| machine | server2 |
| date | 2026-11-09 |
| commit | 5a4abaf8 |
| smoke | false |
| scale | 0.1 |
| engine_name | duckdb |
| steps | gen, load, run |
| runs | 3 |
| cold | no: the first run is not cold |
| kernel | 6.8.0-147-generic |
| tpch_tools | /var/tmp/m3w-work/tp/kit/tpch_tools_3.0.1/dbgen |
| engine_version | v2.0.0-dev84237 (Development Version) cc7e7bac7f |
| dbgen | the data was there |
| queries | qgen -d -s 0.1, written to /var/tmp/m3w-work/tp/data/queries |

## engine

| Field | Value |
|---|---|
| mode | in process, a new duckdb process for each run |
| bin | /usr/local/bin/duckdb |
| db | /var/tmp/m3w-work/tp/sf01-duck.db |
| threads | 1 |
| cpus | null |

## load

| Field | Value |
|---|---|
| usage.wall_s | 44.281618 |
| usage.cpu_usec | 46481510 |
| usage.user_usec | 40083625 |
| usage.system_usec | 6397885 |
| usage.rbytes | 111546368 |
| usage.wbytes | 143491072 |
| usage.memory_peak_bytes | 632700928 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 240082944 |
| usage.pss_max_bytes | 430726144 |
| usage.memory_current_max_bytes | 632520704 |
| usage.samples | 407 |
| du_path | /var/tmp/m3w-work/tp/sf01-duck.db |
| du_apparent_bytes | 128462848 |

## totals

| Field | Value |
|---|---|
| hot_s | 4.447 |
| cold_s | 6.856 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /var/tmp/m3w-work/tp/ans-d |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 0.627, 0.205, 0.291 | 0.627 | 0.205 | 2138112 | 687854 | 1248231 | 20414464 | 41893888 | 4 | ok | null | null |
| q2 | 0.309, 0.118, 0.177 | 0.309 | 0.118 | 1753088 | 395522 | 1067850 | 20176896 | 43905024 | 44 | ok | null | null |
| q3 | 0.199, 0.12, 0.191 | 0.199 | 0.12 | 139264 | 356758 | 822627 | 27521024 | 49970176 | 10 | ok | null | null |
| q4 | 0.292, 0.21, 0.217 | 0.292 | 0.21 | 389120 | 439733 | 863788 | 31363072 | 57269248 | 5 | ok | null | null |
| q5 | 0.196, 0.294, 0.196 | 0.196 | 0.196 | 0 | 413871 | 848595 | 33292288 | 59846656 | 5 | ok | null | null |
| q6 | 0.092, 0.092, 0.19 | 0.092 | 0.092 | 0 | 339978 | 617051 | 15347712 | 39589888 | 1 | ok | null | null |
| q7 | 0.411, 0.399, 0.325 | 0.411 | 0.325 | 774144 | 525757 | 1181938 | 23986176 | 49609728 | 4 | ok | null | null |
| q8 | 0.282, 0.221, 0.475 | 0.282 | 0.221 | 131072 | 642679 | 1213050 | 22249472 | 48864256 | 2 | ok | null | null |
| q9 | 0.501, 0.53, 0.402 | 0.501 | 0.402 | 69632 | 580545 | 1376238 | 28913664 | 55574528 | 175 | ok | null | null |
| q10 | 0.616, 0.391, 0.613 | 0.616 | 0.391 | 81920 | 892686 | 2429468 | 90800128 | 117317632 | 20 | ok | null | null |
| q11 | 0.193, 0.195, 0.187 | 0.193 | 0.187 | 155648 | 455148 | 1511298 | 18751488 | 44572672 | 22 | ok | null | null |
| q12 | 0.294, 0.188, 0.224 | 0.294 | 0.188 | 110592 | 653141 | 1044573 | 19836928 | 46125056 | 2 | ok | null | null |
| q13 | 0.213, 0.289, 0.403 | 0.213 | 0.289 | 0 | 346707 | 1175313 | 22392832 | 47688704 | 37 | ok | null | null |
| q14 | 0.021, 0.024, 0.034 | 0.021 | 0.024 | 0 | 386650 | 733734 | 18395136 | 43751424 | 1 | ok | null | null |
| q15 | 0.118, 0.191, 0.188 | 0.118 | 0.188 | 73728 | 398868 | 757757 | 17530880 | 44362752 | 1 | ok | null | null |
| q16 | 0.374, 0.498, 0.321 | 0.374 | 0.321 | 0 | 510670 | 1516640 | 20754432 | 46868480 | 2762 | ok | null | null |
| q17 | 1.011, 0.309, 0.193 | 1.011 | 0.193 | 12382208 | 902982 | 842281 | 45772800 | 59958272 | 1 | ok | null | null |
| q18 | 0.133, 0.202, 0.209 | 0.133 | 0.202 | 253952 | 302049 | 1122610 | 25812992 | 49294336 | 5 | ok | null | null |
| q19 | 0.116, 0.11, 0.103 | 0.116 | 0.103 | 184320 | 567052 | 766205 | 18448384 | 43870208 | 1 | ok | null | null |
| q20 | 0.201, 0.199, 0.291 | 0.201 | 0.199 | 0 | 328774 | 935793 | 22515712 | 47927296 | 9 | ok | null | null |
| q21 | 0.474, 0.206, 0.192 | 0.474 | 0.192 | 8126464 | 540153 | 1064402 | 38805504 | 54624256 | 47 | ok | null | null |
| q22 | 0.183, 0.081, 0.272 | 0.183 | 0.081 | 0 | 468014 | 561029 | 17985536 | 42440704 | 7 | ok | null | null |

