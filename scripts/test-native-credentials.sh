#!/usr/bin/env bash
# Never points tests at the desktop user's existing credential store.
set -euo pipefail
workspace="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$(dirname "${BASH_SOURCE[0]}")/.."
bus_runner="$(command -v dbus-run-session || true)"
if [[ -z "$bus_runner" ]]; then
  for candidate in /nix/store/*-dbus-*/bin/dbus-run-session; do
    [[ -x "$candidate" ]] && bus_runner="$candidate" && break
  done
fi
if [[ -z "$bus_runner" ]] || { [[ "${AMANTE_SKIP_NATIVE_STORE:-0}" != 1 ]] && ! command -v gnome-keyring-daemon >/dev/null; }; then
  echo "Requires dbus-run-session and gnome-keyring-daemon." >&2
  exit 1
fi
bus_config=()
if [[ -f "$(dirname "$bus_runner")/../share/dbus-1/session.conf" ]]; then
  bus_config=("--config-file=$(dirname "$bus_runner")/../share/dbus-1/session.conf")
fi
temp="$(mktemp -d)"
trap 'rm -rf "$temp"' EXIT
chmod 700 "$temp"
mkdir -m 700 "$temp/home" "$temp/runtime"
env -u DBUS_SESSION_BUS_ADDRESS -u GNOME_KEYRING_CONTROL \
  PATH="$(dirname "$bus_runner"):$PATH" \
  HOME="$temp/home" XDG_DATA_HOME="$temp/home/data" \
  XDG_CONFIG_HOME="$temp/home/config" XDG_CACHE_HOME="$temp/home/cache" \
  XDG_RUNTIME_DIR="$temp/runtime" AMANTE_ISOLATED_KEYRING_TEST=1 \
  CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" \
  AMANTE_WORKSPACE="$workspace" AMANTE_TEST_RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" \
  "$bus_runner" "${bus_config[@]}" -- bash -euo pipefail -c '
    # A public synthetic test passphrase, not a user credential.
    if [[ "${AMANTE_SKIP_NATIVE_STORE:-0}" != 1 ]]; then
      printf "%s" "isolated-test-only" | gnome-keyring-daemon --unlock --components=secrets >/dev/null
      env -u LD_AUDIT GLIBC_TUNABLES=glibc.rtld.optional_static_tls=65536 \
        RUSTUP_HOME="$AMANTE_TEST_RUSTUP_HOME" \
        cargo +1.99.1 test --manifest-path "$AMANTE_WORKSPACE/amante-viewer/Cargo.toml" \
        --profile ci --locked -p amante-credentials --test native_store -- --ignored
    else
      echo "Native credential-store round trip explicitly skipped; testing SLURL bus only."
    fi
    env -u LD_AUDIT GLIBC_TUNABLES=glibc.rtld.optional_static_tls=65536 \
      RUSTUP_HOME="$AMANTE_TEST_RUSTUP_HOME" \
      cargo +1.99.1 test --manifest-path "$AMANTE_WORKSPACE/amante-viewer/Cargo.toml" \
      --profile ci --locked -p amante-credentials --test linux_bus -- --test-threads=1
  '
