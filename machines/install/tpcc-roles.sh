#!/bin/bash
# Make the roles that HammerDB needs in the PostgreSQL 19 cluster: hammerdb, a superuser that builds the schema, and tpcc, the owner of the schema.
#
# Usage: tpcc-roles.sh [CLUSTER]. The cluster is main by default. HammerDB makes the database tpcc and the role tpcc itself, so it needs a superuser. The roles log in on the Unix socket with a password, not with trust, because a superuser with trust would let every user of the machine in. The password is random and is kept in /etc/rupg-bench/tpcc.pass, which only root can read. rupg-bench tpcc reads it from there.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"
need_root

major=19
cluster=${1:-main}
passfile=/etc/rupg-bench/tpcc.pass
mkdir -p "$(dirname "$passfile")"
if [ ! -s "$passfile" ]; then
    (umask 077 && head -c 24 /dev/urandom | base64 | tr -dc 'A-Za-z0-9' >"$passfile")
fi
pass=$(cat "$passfile")
port=$(pg_lsclusters -h | awk -v v="$major" -v c="$cluster" '$1 == v && $2 == c { print $3 }')
[ -n "$port" ] || die "there is no cluster $major/$cluster"

# The rules for the socket. They go first, so the peer rule of the package does not reject the roles. The method password asks the client for the password in clear text over the socket, so the small client of rupg-bench can use it too.
hba=/etc/postgresql/$major/$cluster/pg_hba.conf
for role in tpcc hammerdb; do
    if ! grep -qE "^local[[:space:]]+all[[:space:]]+${role}[[:space:]]+password" "$hba"; then
        sed -i "1i local   all             $role                                   password" "$hba"
    fi
done
psql_su() {
    sudo -u postgres psql -h /var/run/postgresql -p "$port" -qAt "$@"
}
psql_su -c "SELECT pg_reload_conf()" >/dev/null
if [ "$(psql_su -c "SELECT count(*) FROM pg_roles WHERE rolname = 'hammerdb'")" = 0 ]; then
    psql_su -c "CREATE ROLE hammerdb LOGIN SUPERUSER"
fi
# The password goes on standard input, so it is not on a command line that ps shows. It has only letters and digits, so the quotes are safe.
set_password() {
    printf "ALTER ROLE %s PASSWORD '%s';\n" "$1" "$pass" | psql_su -v ON_ERROR_STOP=1 -f -
}
set_password hammerdb
if [ "$(psql_su -c "SELECT count(*) FROM pg_roles WHERE rolname = 'tpcc'")" != 0 ]; then
    set_password tpcc
fi
log "roles hammerdb and tpcc in $major/$cluster on port $port, password in $passfile"
