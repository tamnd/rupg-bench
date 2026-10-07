# instructions-tpcc-postgresql on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | instructions-tpcc-postgresql |
| machine | server3 |
| date | 2026-10-07 |
| commit | b41f7e63 |
| smoke | true |
| note | smoke run: it shows that the runner works, it is not the ratchet |
| set | tpcc |
| engine_name | postgresql |
| queries_from | /root/work/fixed2/tpcc |
| repeat | 3 |
| kernel | 6.8.0-106-generic |

## engine

| Field | Value |
|---|---|
| mode | server cgroup |
| cgroup | system.slice/system-postgresql.slice/postgresql@19-ycsb.service |
| client | /usr/lib/postgresql/19/bin/psql, -X, -q, -v, ON_ERROR_STOP=1, -h, /var/run/postgresql, -p, 5433, -U, tpcc, -d, tpcc, -o, /dev/null, -f, - |

## counts

| Field | Value |
|---|---|
| base_runs | 52594897, 52581519, 47058645 |
| base | 47058645 |
| total_net | 870376156 |

## queries

| query | net | runs |
|---|---|---|
| neword100 | 870376156 | 956960680, 918670439, 917434801 |

