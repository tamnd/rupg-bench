#!/bin/bash
# Install the HammerDB release of pins.toml into $RUPG_BENCH_OPT/hammerdb.
#
# rupg-bench tpcc runs hammerdbcli from there for the procedure form of TPC-C (spec/20 section 20.8). HammerDB reaches PostgreSQL through libpq, so the script installs libpq5.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"

version=$(pin hammerdb version)
url=$(pin hammerdb url)
sha=$(pin hammerdb sha256)
dir=$RUPG_BENCH_OPT/hammerdb
mkdir -p "$RUPG_BENCH_OPT/download"

apt_install libpq5
tarball=$RUPG_BENCH_OPT/download/$(basename "$url")
fetch "$url" "$sha" "$tarball"
if [ ! -x "$dir/hammerdbcli" ] || [ "$(cat "$dir/VERSION" 2>/dev/null)" != "$version" ]; then
    rm -rf "$dir" "$dir.part"
    mkdir -p "$dir.part"
    tar -xzf "$tarball" -C "$dir.part" --strip-components 1
    echo "$version" >"$dir.part/VERSION"
    mv "$dir.part" "$dir"
fi
[ -x "$dir/hammerdbcli" ] || die "$dir has no hammerdbcli"
# librarycheck prints whether HammerDB can load libpq. hammerdbcli waits for more input at the end of stdin, so the input ends with exit.
check=$(
    cd "$dir" || exit 1
    printf 'librarycheck\nexit\n' | timeout 120 ./hammerdbcli 2>&1 | grep -A1 'library for PostgreSQL'
) || true
case $check in
*Success*) ;;
*) die "HammerDB cannot load libpq: $check" ;;
esac
log "HammerDB $version in $dir"
printf '%s\n' "$dir"
