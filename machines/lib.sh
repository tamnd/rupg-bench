#!/bin/bash
# Shared functions for the machine scripts. Source this file. It does not run anything.
#
# The scripts read every version from pins.toml at the root of the repository, so a pin changes in one place.

RUPG_BENCH_ROOT=${RUPG_BENCH_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}
PINS_FILE=${PINS_FILE:-$RUPG_BENCH_ROOT/pins.toml}
# Installed programs and fetched sources go here. The data of a run goes under RUPG_BENCH_DATA.
RUPG_BENCH_OPT=${RUPG_BENCH_OPT:-/opt/rupg-bench}
RUPG_BENCH_DATA=${RUPG_BENCH_DATA:-/var/lib/rupg-bench}
# The build jobs. The test servers are shared, so the default is 4.
JOBS=${JOBS:-4}

log() {
    printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" >&2
}

die() {
    log "error: $*"
    exit 1
}

need_root() {
    [ "$(id -u)" -eq 0 ] || die "run this script as root"
}

# pin <table> <key>: print one string value of pins.toml. Fail if it is missing.
pin() {
    local value
    value=$(awk -v table="[$1]" -v key="$2" '
        /^\[/ { in_table = ($0 == table); next }
        in_table && $1 == key && $2 == "=" {
            line = $0
            sub(/^[^=]*= *"/, "", line)
            sub(/".*$/, "", line)
            print line
            exit
        }' "$PINS_FILE")
    [ -n "$value" ] || die "pins.toml has no $1.$2"
    printf '%s\n' "$value"
}

# fetch <url> <sha256 or sha3-256> <file> [sha3]: download a file and check its hash.
fetch() {
    local url=$1 want=$2 out=$3 kind=${4:-sha256} got
    if [ ! -f "$out" ]; then
        log "fetch $url"
        curl -fsSL --retry 3 -o "$out.part" "$url"
        mv "$out.part" "$out"
    fi
    if [ "$kind" = sha3 ]; then
        got=$(openssl dgst -sha3-256 -r "$out" | cut -d' ' -f1)
    else
        got=$(sha256sum "$out" | cut -d' ' -f1)
    fi
    if [ "$got" != "$want" ]; then
        rm -f "$out"
        die "$out has hash $got, pins.toml wants $want"
    fi
}

# git_at <repo> <commit> <dir>: check out one commit of a repository without its history.
git_at() {
    local repo=$1 commit=$2 dir=$3
    if [ -d "$dir/.git" ] && [ "$(git -C "$dir" rev-parse HEAD)" = "$commit" ]; then
        return 0
    fi
    rm -rf "$dir"
    mkdir -p "$dir"
    git -C "$dir" init -q
    git -C "$dir" fetch -q --depth 1 "$repo" "$commit"
    git -C "$dir" -c advice.detachedHead=false checkout -q FETCH_HEAD
}

# apt_install <package>...: install packages. Another apt on the machine holds the lock for a time, so wait up to 10 minutes for it.
apt_install() {
    local missing=() p
    for p in "$@"; do
        dpkg-query -W -f='${Status}' "$p" 2>/dev/null | grep -q 'install ok installed' || missing+=("$p")
    done
    [ ${#missing[@]} -eq 0 ] && return 0
    log "apt install ${missing[*]}"
    # A broken source that is not ours must not stop the install, so a failed update is only a warning.
    apt-get -o DPkg::Lock::Timeout=600 update -q >/dev/null 2>&1 || log "warning: apt-get update failed, the install uses the old package lists"
    # needrestart must not restart the services of other users of the machine.
    NEEDRESTART_SUSPEND=1 DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=600 install -y -q "${missing[@]}" >/dev/null
}
