#!/bin/bash
# Build rupg-bench with the Rust release of pins.toml and copy the binary to $RUPG_BENCH_OPT/bin/rupg-bench.
#
# The script installs rustup when the machine has none, then the toolchain of rust-toolchain.toml. rustup-init is checked against the sha256 file that static.rust-lang.org publishes next to it. That checks the download, not a pin, and the toolchain itself is pinned. The build uses $JOBS jobs and the lock file of the repository. The script prints the path of the binary.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/../lib.sh"

version=$(pin rust version)
bin=$RUPG_BENCH_OPT/bin
mkdir -p "$bin" "$RUPG_BENCH_OPT/download"
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck source=/dev/null
    . "$HOME/.cargo/env"
fi
if ! command -v rustup >/dev/null; then
    apt_install build-essential curl
    url=https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init
    init=$RUPG_BENCH_OPT/download/rustup-init
    rm -f "$init"
    fetch "$url" "$(curl -fsSL --retry 3 "$url.sha256" | cut -d' ' -f1)" "$init"
    chmod 755 "$init"
    "$init" -y -q --profile minimal --default-toolchain "$version" --no-modify-path >/dev/null
    # shellcheck source=/dev/null
    . "$HOME/.cargo/env"
fi
rustup toolchain install "$version" --profile minimal >/dev/null 2>&1
cd "$RUPG_BENCH_ROOT"
log "build rupg-bench with Rust $version and $JOBS jobs"
CARGO_BUILD_JOBS=$JOBS cargo "+$version" build --release --locked --quiet
install -m 755 target/release/rupg-bench "$bin/rupg-bench.part"
mv "$bin/rupg-bench.part" "$bin/rupg-bench"
log "installed $bin/rupg-bench: $("$bin/rupg-bench" --version)"
printf '%s\n' "$bin/rupg-bench"
