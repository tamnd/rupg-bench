#!/bin/bash
# Install the pinned systems and run one suite on this machine. The reports go to reports/ of the repository.
#
# Usage: run-suite.sh SUITE [MACHINE]. SUITE is pgbench, tpch, tpcc, ycsb, clickbench or fdatasync. MACHINE is the name in the reports (4xl, metal, tpch, oltp), and the host name by default.
#
# SMOKE=1 runs the suite at a tiny scale and marks every report as a smoke run. A smoke run shows that the scripts and the driver work. It is not a baseline.
#
# The sizes that spec/20 leaves to M0 have no default for a full run, so the script stops when they are not set:
#   tpcc: WAREHOUSES (a list, for example "1000 4000") and VU (spec/20 section 20.8)
#   ycsb: RECORDS (spec/20 section 20.9)
# Other settings: SCALES for tpch ("1"), ENGINES for clickbench and tpch ("postgresql duckdb"), HITS_DIR for clickbench (a directory with hits.tsv and hits.parquet, fetched when they are missing), SERVER_CPUS and CLIENT_CPUS for the cpuset of spec/20 section 20.2 (for example 0-15 and 16-31 on oltp).
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/lib.sh"
if [ "$(id -u)" -ne 0 ]; then
    exec sudo -E -H "$0" "$@"
fi

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    die "usage: run-suite.sh SUITE [MACHINE]"
fi
suite=$1
machine=${2:-$(hostname -s)}
smoke=${SMOKE:-0}
install=$RUPG_BENCH_ROOT/machines/install
reports=$RUPG_BENCH_ROOT/reports
mkdir -p "$RUPG_BENCH_DATA" "$reports"
export PATH=$RUPG_BENCH_OPT/bin:$PATH
# The ClickBench scripts of the pin read PGVERSION, and PostgreSQL is 19 here.
export PGVERSION=19

common=(--report "$reports" --machine "$machine")
if [ "$smoke" = 1 ]; then
    common+=(--smoke)
fi
need() {
    [ -n "${!1:-}" ] || die "set $1 for a full $suite run: spec/20 leaves it to the choice at M0"
}

# The cpuset of the server on oltp: the server unit gets SERVER_CPUS, and the driver and its clients run on CLIENT_CPUS.
server_cpus() {
    local unit=$1 dropin
    dropin=/etc/systemd/system/$unit.d
    if [ -n "${SERVER_CPUS:-}" ]; then
        mkdir -p "$dropin"
        printf '[Service]\nAllowedCPUs=%s\n' "$SERVER_CPUS" >"$dropin/rupg-bench-cpuset.conf"
        log "$unit runs on the CPUs $SERVER_CPUS"
    else
        rm -f "$dropin/rupg-bench-cpuset.conf"
    fi
    systemctl daemon-reload
    systemctl restart "$unit"
}
rb() {
    if [ -n "${CLIENT_CPUS:-}" ]; then
        taskset -c "$CLIENT_CPUS" "$harness" "$@"
    else
        "$harness" "$@"
    fi
}

log "run-suite $suite on $machine, smoke=$smoke"
harness=$("$install/harness.sh" | tail -1)
"$harness" pins --file "$RUPG_BENCH_ROOT/pins.toml" >&2

case $suite in
fdatasync)
    # The device of the PostgreSQL data directory, or of RUPG_BENCH_DATA before PostgreSQL is installed.
    dir=$RUPG_BENCH_DATA
    [ -d /var/lib/postgresql ] && dir=/var/lib/postgresql
    if [ "$smoke" = 1 ]; then
        "$RUPG_BENCH_ROOT/machines/fdatasync.sh" "$dir" 16m
    else
        "$RUPG_BENCH_ROOT/machines/fdatasync.sh" "$dir"
    fi
    ;;
pgbench)
    "$install/postgresql.sh"
    server_cpus postgresql@19-main
    if [ "$smoke" = 1 ]; then
        rb pgbench --unit postgresql@19-main --scale 1 --clients 4 --time 30 "${common[@]}"
    else
        rb pgbench --unit postgresql@19-main --scale "${SCALE:-100}" --clients "${CLIENTS:-16}" \
            --time "${TIME:-300}" "${common[@]}"
    fi
    ;;
tpch)
    engines=${ENGINES:-postgresql duckdb}
    "$install/duckdb.sh"
    [[ " $engines " == *" postgresql "* ]] && "$install/postgresql.sh"
    tools=$("$install/tpch-tools.sh" | tail -1)
    scales=${SCALES:-1}
    [ "$smoke" = 1 ] && scales=0.01
    for sf in $scales; do
        data=$RUPG_BENCH_DATA/tpch/sf$sf
        answers=$RUPG_BENCH_DATA/tpch/answers-sf$sf
        mkdir -p "$data"
        # At SF1 the answers of the kit check every engine. At a larger scale PostgreSQL 19 gives the answers (spec/20 section 20.7), so it runs first.
        check=()
        [ "$sf" = 1 ] && check=(--answers "$tools/answers")
        steps=gen,load,run
        for engine in $engines; do
            case $engine in
            postgresql)
                server_cpus postgresql@19-main
                rb tpch --tools "$tools" --scale "$sf" --data "$data" --steps "$steps" --engine postgresql \
                    --unit postgresql@19-main --du /var/lib/postgresql/19/main "${check[@]}" \
                    --save-answers "$answers" "${common[@]}"
                [ "$sf" = 1 ] || check=(--answers "$answers")
                ;;
            duckdb)
                cpus=()
                [ -n "${SERVER_CPUS:-}" ] && cpus=(--cpus "$SERVER_CPUS")
                rm -f "$data.db"
                rb tpch --tools "$tools" --scale "$sf" --data "$data" --steps "$steps" --duckdb "$data.db" \
                    --duckdb-bin "$RUPG_BENCH_OPT/bin/duckdb" "${cpus[@]}" "${check[@]}" "${common[@]}"
                ;;
            *) die "the tpch suite has no engine $engine" ;;
            esac
            steps=load,run
        done
    done
    ;;
tpcc)
    "$install/postgresql.sh"
    "$install/hammerdb.sh"
    "$install/tpcc-roles.sh"
    server_cpus postgresql@19-main
    if [ "$smoke" = 1 ]; then
        warehouses=2 vu=2 rampup=1 duration=1
    else
        need WAREHOUSES
        need VU
        warehouses=${WAREHOUSES:?} vu=${VU:?} rampup=${RAMPUP:-5} duration=${DURATION:-20}
    fi
    for w in $warehouses; do
        rb tpcc --unit postgresql@19-main --warehouses "$w" --vu "$vu" --rampup "$rampup" \
            --duration "$duration" --sync on,off --sync-dir /var/lib/postgresql/19/main "${common[@]}"
    done
    ;;
ycsb)
    "$install/postgresql.sh"
    server_cpus postgresql@19-main
    if [ "$smoke" = 1 ]; then
        records=10000 rows=1,16 time=5
    else
        need RECORDS
        records=${RECORDS:?} rows=1,16,16x64 time=${TIME:-60}
    fi
    # Both durability rows of spec/20 section 20.9. The first run loads usertable, and the second runs on it.
    steps=load,run
    for sync in on off; do
        rb ycsb --unit postgresql@19-main --records "$records" --rows "$rows" --time "$time" --sync "$sync" \
            --steps "$steps" "${common[@]}"
        steps=run
    done
    ;;
clickbench)
    engines=${ENGINES:-postgresql duckdb}
    src=$("$install/clickbench.sh" | tail -1)
    hits=${HITS_DIR:-$RUPG_BENCH_DATA/hits}
    run=$RUPG_BENCH_DATA/clickbench
    mkdir -p "$hits" "$run"
    rm -rf "${run:?}/lib"
    cp -r "$src/lib" "$run/lib"
    answers=$run/answers-postgresql
    first=yes
    # The concurrent test of the pin runs for 600 s. A smoke run shortens it to 60 s unless the setting is in the environment.
    if [ "$smoke" = 1 ]; then
        export BENCH_CONCURRENT_DURATION=${BENCH_CONCURRENT_DURATION:-60}
    fi
    for engine in $engines; do
        rm -rf "${run:?}/$engine"
        cp -r "$src/$engine" "$run/$engine"
        case $engine in
        postgresql)
            "$install/postgresql.sh"
            if [ "$smoke" = 1 ]; then
                # The first 100,000 rows of hits.tsv, with no download of the whole file. head closes the pipe early, so curl fails by design, and the line count is the check.
                source=$hits/hits-smoke.tsv
                if [ ! -f "$source" ]; then
                    { curl -fsSL https://datasets.clickhouse.com/hits_compatible/hits.tsv.gz | gzip -dc | head -n 100000 >"$source.part"; } || true
                    [ "$(wc -l <"$source.part")" -eq 100000 ] || die "the download of the first rows of hits.tsv failed"
                    mv "$source.part" "$source"
                fi
            else
                [ -f "$hits/hits.tsv" ] || "$run/lib/download-hits-tsv" "$hits"
                source=$hits/hits.tsv
                # The settings that postgresql/install of the pin writes after its apt install. The harness builds PostgreSQL from the pin, so it does not run that script.
                mem=$(awk '/MemTotal/ {print $2}' /proc/meminfo)
                threads=$(nproc)
                cat >/etc/postgresql/19/main/conf.d/clickbench.conf <<CONF
shared_buffers=$((mem / 4))kB
max_worker_processes=$((threads + 15))
max_parallel_workers=$threads
max_parallel_maintenance_workers=$((threads / 2))
max_parallel_workers_per_gather=$((threads / 2))
max_wal_size=32GB
work_mem=64MB
effective_cache_size = $((mem - mem / 4))kB
CONF
            fi
            server_cpus postgresql@19-main
            rb clickbench --dir "$run/postgresql" --engine postgresql --unit postgresql@19-main --source "$source" \
                --save-answers "$answers" --result-file "$run/postgresql-result.json" "${common[@]}"
            ;;
        duckdb)
            "$install/duckdb.sh"
            check=()
            if [ "$smoke" = 1 ]; then
                # The first 100,000 rows of the first file of the partitioned parquet set. They are not known to be the rows of hits-smoke.tsv, so a smoke run does not check the answers against PostgreSQL.
                source=$hits/hits-smoke.parquet
                if [ ! -f "$source" ]; then
                    [ -f "$hits/hits_0.parquet" ] ||
                        curl -fsSL --retry 3 -o "$hits/hits_0.parquet" https://datasets.clickhouse.com/hits_compatible/athena_partitioned/hits_0.parquet
                    "$RUPG_BENCH_OPT/bin/duckdb" -c "COPY (SELECT * FROM '$hits/hits_0.parquet' LIMIT 100000) TO '$source.part' (FORMAT parquet)"
                    mv "$source.part" "$source"
                fi
            else
                [ -f "$hits/hits.parquet" ] || "$run/lib/download-hits-parquet-single" "$hits"
                source=$hits/hits.parquet
                [ "$first" = no ] && [ -d "$answers" ] && check=(--answers "$answers")
            fi
            cpus=()
            [ -n "${SERVER_CPUS:-}" ] && cpus=(--cpus "$SERVER_CPUS")
            rb clickbench --dir "$run/duckdb" --engine duckdb --source "$source" \
                --duckdb-bin "$RUPG_BENCH_OPT/bin/duckdb" "${cpus[@]}" "${check[@]}" \
                --result-file "$run/duckdb-result.json" "${common[@]}"
            ;;
        *) die "the clickbench suite runs postgresql and duckdb. The driver does not run $engine yet." ;;
        esac
        first=no
    done
    ;;
*) die "unknown suite $suite: use pgbench, tpch, tpcc, ycsb, clickbench or fdatasync" ;;
esac
log "run-suite $suite done, the reports are in $reports"
