#!/bin/bash
# Install the sqlite3 program of pins.toml into $RUPG_BENCH_OPT/bin.
#
# sqlite.org publishes a SHA3-256 hash for each download, not a SHA-256 hash, so the script checks that hash. The install script of the ClickBench pin installs sqlite3 from apt only when no sqlite3 is on the PATH, so put $RUPG_BENCH_OPT/bin first on the PATH and the pin runs the pinned version.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"

version=$(pin sqlite version)
url=$(pin sqlite url)
sha=$(pin sqlite sha3_256)
bin=$RUPG_BENCH_OPT/bin
mkdir -p "$bin" "$RUPG_BENCH_OPT/download"

command -v unzip >/dev/null || apt_install unzip
zip=$RUPG_BENCH_OPT/download/$(basename "$url")
fetch "$url" "$sha" "$zip" sha3
tmp=$(mktemp -d "$RUPG_BENCH_OPT/download/sqlite.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
unzip -q -o -j "$zip" sqlite3 -d "$tmp"
install -m 755 "$tmp/sqlite3" "$bin/sqlite3.part"
mv "$bin/sqlite3.part" "$bin/sqlite3"
got=$("$bin/sqlite3" --version)
case $got in
"$version "*) log "installed $bin/sqlite3: $got" ;;
*) die "$bin/sqlite3 reports $got, pins.toml wants $version" ;;
esac
