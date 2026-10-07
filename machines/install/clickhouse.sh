#!/bin/bash
# Install the ClickHouse release of pins.toml into $RUPG_BENCH_OPT/bin.
#
# Usage: clickhouse.sh [--system]. The script checks the sha256 of the release archive and copies the one clickhouse binary out of it. With --system it then runs `clickhouse install` from that binary, which makes /usr/bin/clickhouse, the user clickhouse and /etc/clickhouse-server. The install script of the ClickBench pin fetches the newest build with `curl https://clickhouse.com/ | sh` only when /usr/bin/clickhouse does not exist, so after --system the pin runs the pinned version. Use --system only on a machine of the benchmark, because it changes the system.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"

system=no
case ${1:-} in
"") ;;
--system) system=yes ;;
*) die "usage: clickhouse.sh [--system]" ;;
esac
version=$(pin clickhouse version)
url=$(pin clickhouse url)
sha=$(pin clickhouse sha256)
bin=$RUPG_BENCH_OPT/bin
mkdir -p "$bin" "$RUPG_BENCH_OPT/download"

tgz=$RUPG_BENCH_OPT/download/clickhouse-common-static-$version-amd64.tgz
fetch "$url" "$sha" "$tgz"
tmp=$(mktemp -d "$RUPG_BENCH_OPT/download/clickhouse.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
tar -xzf "$tgz" -C "$tmp" "clickhouse-common-static-$version/usr/bin/clickhouse"
install -m 755 "$tmp/clickhouse-common-static-$version/usr/bin/clickhouse" "$bin/clickhouse.part"
mv "$bin/clickhouse.part" "$bin/clickhouse"
got=$("$bin/clickhouse" local --version)
case $got in
*" $version "* | *" $version") log "installed $bin/clickhouse: $got" ;;
*) die "$bin/clickhouse reports $got, pins.toml wants $version" ;;
esac

if [ "$system" = yes ]; then
    need_root
    if [ -x /usr/bin/clickhouse ]; then
        have=$(/usr/bin/clickhouse local --version)
        case $have in
        *" $version "* | *" $version") log "/usr/bin/clickhouse is already $version" ;;
        *) die "/usr/bin/clickhouse reports $have, pins.toml wants $version. Remove it first." ;;
        esac
    else
        log "clickhouse install --noninteractive"
        (cd "$tmp" && "$bin/clickhouse" install --noninteractive >"$RUPG_BENCH_OPT/download/clickhouse-install.log" 2>&1) ||
            die "clickhouse install failed, see $RUPG_BENCH_OPT/download/clickhouse-install.log"
        log "installed /usr/bin/clickhouse: $(/usr/bin/clickhouse local --version)"
    fi
fi
