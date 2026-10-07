#!/bin/bash
# Build PostgreSQL at the pin of pins.toml and make it the cluster 19/main of postgresql-common.
#
# The ClickBench scripts of the pin start PostgreSQL with `systemctl start postgresql@$PGVERSION-main` and read /var/lib/postgresql/$PGVERSION/main. The pin is a commit, not a package, so this script builds the commit into /usr/lib/postgresql/19, where postgresql-common finds it. Then the ClickBench scripts run without change with PGVERSION=19.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"
need_root

repo=$(pin postgresql repo)
commit=$(pin postgresql commit)
major=19
prefix=/usr/lib/postgresql/$major
src=$RUPG_BENCH_OPT/src/postgres

apt_install build-essential bison flex pkg-config libreadline-dev zlib1g-dev libicu-dev libssl-dev \
    liburing-dev liblz4-dev libzstd-dev postgresql-common util-linux-extra

if [ -x "$prefix/bin/postgres" ] && [ "$(cat "$prefix/COMMIT" 2>/dev/null)" = "$commit" ]; then
    log "PostgreSQL at $commit is already in $prefix"
else
    git_at "$repo" "$commit" "$src"
    cd "$src"
    log "configure PostgreSQL at $commit"
    ./configure --quiet --prefix="$prefix" --with-openssl --with-icu --with-liburing --with-lz4 --with-zstd
    log "build with $JOBS jobs"
    make -s -j"$JOBS" world-bin
    make -s install-world-bin
    echo "$commit" >"$prefix/COMMIT"
    cd /
fi

# The cluster. pg_createcluster uses the binaries in /usr/lib/postgresql/19.
if ! pg_lsclusters -h | awk '{print $1"/"$2}' | grep -qx "$major/main"; then
    pg_createcluster "$major" main >/dev/null
fi
systemctl daemon-reload
systemctl restart "postgresql@$major-main"
sudo -u postgres psql -Atc 'SELECT version()'
