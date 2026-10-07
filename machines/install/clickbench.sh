#!/bin/bash
# Fetch the ClickBench commit of pins.toml into $RUPG_BENCH_OPT/src/ClickBench, with lib/ and the system directories only.
#
# Usage: clickbench.sh [SYSTEM...]. The default systems are the ones that spec/20 section 20.5.2 compares. The checkout is sparse because the full repository has the results of every system. rupg-bench clickbench runs in a copy of one system directory next to a copy of lib/, because the scripts write the data and the results in their directory.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"

repo=$(pin clickbench repo)
commit=$(pin clickbench commit)
src=$RUPG_BENCH_OPT/src/ClickBench
if [ $# -gt 0 ]; then
    systems=("$@")
else
    systems=(postgresql duckdb clickhouse umbra cedardb sqlite)
fi

apt_install git
if [ -d "$src/.git" ] && [ "$(git -C "$src" rev-parse HEAD)" = "$commit" ]; then
    git -C "$src" sparse-checkout add lib "${systems[@]}"
else
    log "fetch $repo at $commit"
    rm -rf "$src"
    mkdir -p "$src"
    git -C "$src" init -q
    git -C "$src" sparse-checkout set lib "${systems[@]}"
    git -C "$src" fetch -q --depth 1 --filter=blob:none "$repo" "$commit"
    git -C "$src" -c advice.detachedHead=false checkout -q FETCH_HEAD
fi
[ -f "$src/lib/benchmark-common.sh" ] || die "$src has no lib/benchmark-common.sh"
for s in "${systems[@]}"; do
    [ -x "$src/$s/query" ] || die "$src/$s has no query script"
done
log "ClickBench $commit in $src with ${systems[*]}"
printf '%s\n' "$src"
