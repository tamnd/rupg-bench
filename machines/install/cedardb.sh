#!/bin/bash
# Install CedarDB with the installer that the ClickBench pin uses, and record the version that it installed.
#
# Usage: cedardb.sh DIR. DIR is the cedardb directory of the ClickBench copy, where the pin keeps the binary and the database. The installer at https://get.cedardb.com always installs the newest release, so this install cannot be pinned. The script keeps a copy of the installer with its sha256, and writes the version that the binary reports to $RUPG_BENCH_OPT/cedardb.version. A run whose version is not the one in pins.toml says so. The installer adds the systemd unit cedardb.service, so use the script only on a machine of the benchmark. The license of CedarDB is not verified (pins.toml), so a result is published only after the license is checked.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"
need_root

[ $# -eq 1 ] || die "usage: cedardb.sh DIR"
dir=$(cd "$1" && pwd)
want=$(pin cedardb version)
mkdir -p "$RUPG_BENCH_OPT/download"
apt_install postgresql-client
installer=$RUPG_BENCH_OPT/download/cedardb-install-$(date -u +%Y%m%d).sh
curl -fsSL --retry 3 -o "$installer" https://get.cedardb.com
log "installer sha256 $(sha256sum "$installer" | cut -d' ' -f1)"
systemctl stop cedardb.service 2>/dev/null || true
rm -rf "$dir/db"
bash "$installer" -y --install-dir "$dir" --db-dir "$dir/db" --with-systemd=system
systemctl start cedardb.service
got=""
for _ in $(seq 60); do
    got=$(PGHOST=/tmp PGUSER=postgres psql -Atc 'SELECT version()' 2>/dev/null) && break
    sleep 1
done
[ -n "$got" ] || die "CedarDB did not answer SELECT version() in 60 s"
printf '%s\n' "$got" >"$RUPG_BENCH_OPT/cedardb.version"
case $got in
*"$want"*) log "installed CedarDB: $got" ;;
*) log "warning: CedarDB reports $got, pins.toml names $want. Record this version with the result." ;;
esac
log "warning: the license of CedarDB is not verified, so do not publish a CedarDB result until it is"
