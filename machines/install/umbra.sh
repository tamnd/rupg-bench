#!/bin/bash
# Pull the Umbra image of pins.toml and check its digest.
#
# The ClickBench pin pulls the image by its tag, and a tag can move. So the script pulls the tag and stops when the digest is not the digest of pins.toml. It also writes the digest to $RUPG_BENCH_OPT/umbra.digest for the result. The license of Umbra is not verified (pins.toml), so a result is published only after the license is checked.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"
need_root

image=$(pin umbra image)
want=$(pin umbra digest)
command -v docker >/dev/null || apt_install docker.io
log "docker pull $image"
docker pull -q "$image" >/dev/null
digest=$(docker image inspect --format '{{index .RepoDigests 0}}' "$image")
[ "${digest#*@}" = "$want" ] || die "$image has the digest ${digest#*@}, pins.toml wants $want. The tag moved."
mkdir -p "$RUPG_BENCH_OPT"
printf '%s\n' "$digest" >"$RUPG_BENCH_OPT/umbra.digest"
log "pulled $image as $digest"
log "warning: the license of Umbra is not verified, so do not publish an Umbra result until it is"
