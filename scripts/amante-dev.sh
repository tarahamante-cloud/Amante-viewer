#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../amante-viewer"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
# Match CI instead of bootstrapping a mutable stable toolchain during checks.
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-1.99.1}"
# Replit's audit loader is incompatible with rustup's current compiler TLS.
# Limit the adjustment to the child process; do not change workspace secrets.
case "${1:-check}" in
  check) exec env -u LD_AUDIT cargo check --profile ci --workspace --all-targets --locked ;;
  build) exec env -u LD_AUDIT cargo build --profile ci --locked -p aurora-viewer ;;
  test) exec env -u LD_AUDIT cargo test --profile ci --workspace --exclude aurora-viewer --locked "${2:-amante_security}" ;;
  full-test) exec env -u LD_AUDIT cargo test --profile ci --workspace --locked "${2:-amante_security}" ;;
  version) exec env -u LD_AUDIT ./target/ci/aurora-viewer --version ;;
  demo)
    if [[ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]]; then
      echo "A desktop display and a working Vulkan adapter are required; this is not a web app." >&2
      exit 1
    fi
    exec env -u LD_AUDIT AURORA_DEMO=1 cargo run --profile ci --locked -p aurora-viewer -- --title "Amante offline demo"
    ;;
  *) echo "Usage: bash scripts/amante-dev.sh {check|build|test [filter]|full-test [filter]|version|demo}" >&2; exit 2 ;;
esac
