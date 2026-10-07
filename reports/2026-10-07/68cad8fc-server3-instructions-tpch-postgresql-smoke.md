# instructions-tpch-postgresql on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | instructions-tpch-postgresql |
| machine | server3 |
| date | 2026-10-07 |
| commit | 68cad8fc |
| smoke | true |
| note | smoke run: it shows that the runner works, it is not the ratchet |
| set | tpch |
| engine_name | postgresql |
| queries_from | sf0.1/queries |
| repeat | 3 |
| kernel | 6.8.0-106-generic |

## engine

| Field | Value |
|---|---|
| mode | server cgroup |
| cgroup | system.slice/system-postgresql.slice/postgresql@19-main.service |
| client | /usr/lib/postgresql/19/bin/psql, -X, -q, -v, ON_ERROR_STOP=1, -h, /var/run/postgresql, -p, 5432, -U, bench, -d, bench, -o, /dev/null, -c |

## counts

| Field | Value |
|---|---|
| base_runs | 10036857, 11131517, 9738733 |
| base | 9738733 |
| total_net | 448092314188 |

## queries

| query | net | runs |
|---|---|---|
| q1 | 4666200445 | 4805816706, 4684140124, 4675939178 |
| q2 | 137510227 | 214394335, 147248960, 147350793 |
| q3 | 409997873 | 633722528, 419736606, 420303335 |
| q4 | 1163081099 | 1184725598, 1172819832, 1181413698 |
| q5 | 273716682 | 283455415, 285499834, 294910769 |
| q6 | 561976697 | 579882805, 574000486, 571715430 |
| q7 | 357781365 | 383515677, 367520098, 371898346 |
| q8 | 348791144 | 384812398, 358529877, 380352899 |
| q9 | 1297726179 | 1324263693, 1307464912, 1319558251 |
| q10 | 884738950 | 922135723, 902916050, 894477683 |
| q11 | 126439219 | 136177952, 136829295, 139976275 |
| q12 | 952359084 | 979897531, 973525681, 962097817 |
| q13 | 528730303 | 540058399, 538469036, 539299525 |
| q14 | 583471849 | 597384429, 595647571, 593210582 |
| q15 | 541563024 | 571583980, 566594690, 551301757 |
| q16 | 245122014 | 259910899, 258034873, 254860747 |
| q17 | 131801210041 | 131909920667, 131868920191, 131810948774 |
| q18 | 1802274860 | 1916037798, 1860434588, 1812013593 |
| q19 | 982056385 | 993731104, 995286143, 991795118 |
| q20 | 299390712409 | 299445067677, 299400451142, 299506542180 |
| q21 | 850001119 | 923765069, 868339709, 859739852 |
| q22 | 186853220 | 267481230, 202739468, 196591953 |

