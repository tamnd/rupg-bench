#!/bin/bash
# Measure the fdatasync p50 of the device under DIR with fio, as spec/20 section 20.2 asks for the choice of the oltp machine.
#
# Usage: fdatasync.sh DIR [SIZE]. fio writes SIZE (256m by default) in blocks of 4 KiB to a file in DIR, with one fdatasync after each write and a queue depth of 1 (--rw=write --bs=4k --fdatasync=1 --iodepth=1). The script prints the p50, p95 and p99 of the fdatasync time in microseconds and removes the file. The oltp machine needs a p50 below 100 us. This is a requirement for the choice of the machine, not a benchmark result.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/lib.sh"

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    die "usage: fdatasync.sh DIR [SIZE]"
fi
dir=$1
size=${2:-256m}
[ -d "$dir" ] || die "$dir is not a directory"
command -v fio >/dev/null || apt_install fio
command -v python3 >/dev/null || apt_install python3
out=$(mktemp)
trap 'rm -f "$out" "$dir/rupg-bench-fdatasync.0.0"' EXIT
log "fio in $dir, $size, bs 4k, fdatasync after each write"
fio --name=rupg-bench-fdatasync --directory="$dir" --rw=write --bs=4k --size="$size" --fdatasync=1 \
    --iodepth=1 --ioengine=sync --output-format=json --output="$out" >/dev/null
python3 -I - "$out" "$dir" <<'PY'
import json, sys
job = json.load(open(sys.argv[1]))["jobs"][0]
p = job["sync"]["lat_ns"]["percentile"]
us = lambda k: p[k] / 1000
print(f"fdatasync dir={sys.argv[2]} syncs={job['sync']['lat_ns']['N']} "
      f"p50_us={us('50.000000'):.1f} p95_us={us('95.000000'):.1f} p99_us={us('99.000000'):.1f}")
PY
