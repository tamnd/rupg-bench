# instructions-tpch-duckdb on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | instructions-tpch-duckdb |
| machine | server3 |
| date | 2026-10-07 |
| commit | 68cad8fc |
| smoke | true |
| note | smoke run: it shows that the runner works, it is not the ratchet |
| set | tpch |
| engine_name | duckdb |
| queries_from | sf0.1/queries |
| repeat | 3 |
| kernel | 6.8.0-106-generic |

## engine

| Field | Value |
|---|---|
| mode | in process |
| client | /root/work/rupg-bench-opt/bin/duckdb, -readonly, sf0.1.db, -cmd, SET threads = 1, -c |

## counts

| Field | Value |
|---|---|
| base_runs | 85453700, 78966097, 79729237 |
| base | 78966097 |
| total_net | 2438500773 |

## queries

| query | net | runs |
|---|---|---|
| q1 | 181819056 | 269669650, 260785153, 261382172 |
| q2 | 60945836 | 142349281, 160537548, 139911933 |
| q3 | 98193528 | 178961079, 177159625, 177377126 |
| q4 | 81064510 | 163585313, 160030607, 160578261 |
| q5 | 113495494 | 194233285, 218421925, 192461591 |
| q6 | 54119204 | 133085301, 144319138, 133342754 |
| q7 | 119279456 | 198533299, 198245553, 204293015 |
| q8 | 111543941 | 191477000, 201397631, 190510038 |
| q9 | 208136240 | 288423051, 287102337, 287180989 |
| q10 | 259740350 | 348077112, 338706447, 354260612 |
| q11 | 34017323 | 115636087, 113286067, 112983420 |
| q12 | 94258836 | 173948529, 173224933, 174681379 |
| q13 | 155580995 | 239892226, 242172247, 234547092 |
| q14 | 65009526 | 158741533, 144971003, 143975623 |
| q15 | 58942360 | 138371987, 137908457, 141514159 |
| q16 | 52546632 | 132380132, 132522910, 131512729 |
| q17 | 47979014 | 130509455, 132247501, 126945111 |
| q18 | 147836301 | 229966196, 226802398, 226935627 |
| q19 | 113791916 | 192845399, 192758013, 197509120 |
| q20 | 99391587 | 179056009, 188361715, 178357684 |
| q21 | 235512478 | 317363626, 315669462, 314478575 |
| q22 | 45296190 | 124262287, 124480092, 125265430 |

