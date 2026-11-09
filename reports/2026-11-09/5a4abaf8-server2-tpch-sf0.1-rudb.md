# tpch-sf0.1-rudb on server2, 2026-11-09

This is a measured run with the rules of spec/20.

| Field | Value |
|---|---|
| suite | tpch-sf0.1-rudb |
| machine | server2 |
| date | 2026-11-09 |
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
| usage.wall_s | 53.998559 |
| usage.cpu_usec | 49143782 |
| usage.user_usec | 41498363 |
| usage.system_usec | 7645419 |
| usage.rbytes | 113356800 |
| usage.wbytes | 119472128 |
| usage.memory_peak_bytes | 445747200 |
| usage.memory_peak_scope | since the cgroup was created |
| usage.file_max_bytes | 169852928 |
| usage.pss_max_bytes | 324304896 |
| usage.memory_current_max_bytes | 431939584 |
| usage.samples | 501 |
| du_path | /var/tmp/m3w-work/tp/sf01-rudb.db |
| du_apparent_bytes | 58883343 |

## totals

| Field | Value |
|---|---|
| hot_s | 5.069075 |
| cold_s | 7.252404 |
| failed | 0 |
| checked | 22 |
| wrong | 0 |
| answers_from | /var/tmp/m3w-work/tp/ans-d |

## results

| query | runs_s | cold_s | hot_s | cold_rbytes | cold_cpu_usec | hot_cpu_usec | memory_peak_bytes | pss_max_bytes | rows | answer | answer_rule | error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| q1 | 0.980278, 0.780166, 0.707587 | 0.980278 | 0.707587 | 1511424 | 857138 | 1387709 | 6197248 | 18874368 | 4 | ok | null | null |
| q2 | 0.288806, 0.087981, 0.014103 | 0.288806 | 0.014103 | 753664 | 223461 | 118013 | 4481024 | 16576512 | 44 | ok | null | null |
| q3 | 0.094659, 0.090133, 0.093268 | 0.094659 | 0.090133 | 0 | 103055 | 257721 | 5328896 | 17864704 | 10 | ok | null | null |
| q4 | 0.210999, 0.196158, 0.211076 | 0.210999 | 0.196158 | 32768 | 267562 | 405031 | 4530176 | 18395136 | 5 | ok | null | null |
| q5 | 0.20282, 0.12431, 0.292416 | 0.20282 | 0.12431 | 0 | 196637 | 477836 | 6258688 | 19915776 | 5 | ok | null | null |
| q6 | 0.707315, 0.701826, 0.709947 | 0.707315 | 0.701826 | 0 | 600826 | 1313103 | 2981888 | 16305152 | 1 | ok | null | null |
| q7 | 0.130456, 0.167214, 0.204082 | 0.130456 | 0.167214 | 126976 | 110873 | 284008 | 8024064 | 21382144 | 4 | ok | null | null |
| q8 | 0.300536, 0.109391, 0.197434 | 0.300536 | 0.109391 | 28672 | 200183 | 375118 | 8716288 | 24283136 | 2 | ok | null | null |
| q9 | 0.393188, 0.198058, 0.203786 | 0.393188 | 0.198058 | 221184 | 234222 | 280285 | 11829248 | 25873408 | 175 | ok | null | null |
| q10 | 0.502803, 0.298469, 0.473616 | 0.502803 | 0.298469 | 786432 | 332168 | 819680 | 13258752 | 23068672 | 20 | ok | null | null |
| q11 | 0.0961, 0.018362, 0.010616 | 0.0961 | 0.010616 | 192512 | 43149 | 47349 | 3731456 | 15024128 | 22 | ok | null | null |
| q12 | 0.990788, 0.970656, 0.591072 | 0.990788 | 0.591072 | 528384 | 827427 | 1604930 | 10407936 | 22138880 | 2 | ok | null | null |
| q13 | 0.102868, 0.038321, 0.192159 | 0.102868 | 0.038321 | 3624960 | 59268 | 400263 | 9859072 | 19067904 | 37 | ok | null | null |
| q14 | 0.719423, 0.690032, 0.615904 | 0.719423 | 0.615904 | 0 | 623359 | 1298272 | 3383296 | 19341312 | 1 | ok | null | null |
| q15 | 0.709665, 0.671348, 0.701995 | 0.709665 | 0.671348 | 0 | 746814 | 1276393 | 3862528 | 18070528 | 1 | ok | null | null |
| q16 | 0.091691, 0.020149, 0.095406 | 0.091691 | 0.020149 | 12288 | 89274 | 186095 | 5419008 | 16035840 | 2762 | ok | null | null |
| q17 | 0.022404, 0.171927, 0.168719 | 0.022404 | 0.168719 | 86016 | 97438 | 232361 | 4734976 | 16759808 | 1 | ok | null | null |
| q18 | 0.215578, 0.173436, 0.184945 | 0.215578 | 0.173436 | 200704 | 282292 | 365586 | 35176448 | 43750400 | 5 | ok | null | null |
| q19 | 0.024492, 0.088337, 0.016752 | 0.024492 | 0.016752 | 69632 | 99987 | 127040 | 4022272 | 18209792 | 1 | ok | null | null |
| q20 | 0.168142, 0.093084, 0.032013 | 0.168142 | 0.032013 | 0 | 169992 | 213456 | 6406144 | 20914176 | 9 | ok | null | null |
| q21 | 0.224515, 0.113968, 0.181759 | 0.224515 | 0.113968 | 1761280 | 125208 | 383666 | 10915840 | 22645760 | 47 | ok | null | null |
| q22 | 0.074878, 0.00953, 0.082498 | 0.074878 | 0.00953 | 253952 | 77792 | 107763 | 3751936 | 16397312 | 7 | ok | null | null |

