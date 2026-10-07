#!/bin/bash
# Install the DuckDB command line program of pins.toml into $RUPG_BENCH_OPT/bin.
#
# The script does not touch a duckdb that is already on the PATH. The drivers call $RUPG_BENCH_OPT/bin/duckdb by its full path.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"

version=$(pin duckdb version)
url=$(pin duckdb url)
sha=$(pin duckdb sha256)
bin=$RUPG_BENCH_OPT/bin
mkdir -p "$bin" "$RUPG_BENCH_OPT/download"

gz=$RUPG_BENCH_OPT/download/duckdb_cli-$version-linux-amd64.gz
fetch "$url" "$sha" "$gz"
gunzip -c "$gz" >"$bin/duckdb.part"
chmod 755 "$bin/duckdb.part"
mv "$bin/duckdb.part" "$bin/duckdb"
got=$("$bin/duckdb" --version)
case $got in
v"$version"*) log "installed $bin/duckdb: $got" ;;
*) die "$bin/duckdb reports $got, pins.toml wants $version" ;;
esac
