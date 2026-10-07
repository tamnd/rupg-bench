#!/bin/bash
# Fetch the TPC-H tools of pins.toml and build dbgen and qgen in $RUPG_BENCH_OPT/src/tpch-tools.
#
# The tools are under the TPC EULA, so the repository does not hold a copy. The pin is a commit of a repository that holds tpch_tools_3.0.1. The build uses makefile.suite of the kit with the settings on the command line, so a changed Makefile in the copy does not count. DATABASE=VECTORWISE makes qgen print a row count as the line `--LIMIT n`, which rupg-bench tpch turns into a LIMIT clause.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"

repo=$(pin tpch_tools repo)
commit=$(pin tpch_tools commit)
path=$(pin tpch_tools path)
src=$RUPG_BENCH_OPT/src/tpch-tools

apt_install build-essential
git_at "$repo" "$commit" "$src"
dbgen=$src/$path/dbgen
[ -f "$dbgen/makefile.suite" ] || die "$dbgen has no makefile.suite"
log "build dbgen and qgen with $JOBS jobs"
# The kit is old C and gcc prints many warnings, so they go to a log.
if ! make -s -C "$dbgen" -f makefile.suite -j"$JOBS" CC=gcc DATABASE=VECTORWISE MACHINE=LINUX WORKLOAD=TPCH dbgen qgen >"$src/build.log" 2>&1; then
    tail -20 "$src/build.log" >&2
    die "the build failed, see $src/build.log"
fi
for p in dbgen qgen; do
    [ -x "$dbgen/$p" ] || die "the build made no $dbgen/$p"
done
log "built $dbgen/dbgen and $dbgen/qgen at $commit"
printf '%s\n' "$dbgen"
