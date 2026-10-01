#!/usr/bin/env bash
# Keeps routine NuxtJP checks V8-free and makes the full native gate explicit.
set -euo pipefail

repository_root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$repository_root"
tier="${1:-standard}"
case "$tier" in standard | full | docs | feature-matrix) ;; *)
  echo "usage: $0 [standard|full|docs|feature-matrix]" >&2; exit 2 ;;
esac

require_path_capacity() {
  local minimum=$1 label=$2 target=$3 path available
  local -a paths=("$target")
  if grep -qi microsoft /proc/sys/kernel/osrelease 2>/dev/null; then
    [[ -d /mnt/c ]] || { echo "WSL host storage is unavailable." >&2; exit 1; }
    paths+=(/mnt/c)
  fi
  for path in "${paths[@]}"; do
    available="$(df -Pk "$path" | awk 'END { print $4 }')"
    [[ "$available" =~ ^[0-9]+$ ]] || { echo "Cannot inspect $path." >&2; exit 1; }
    if ((available < minimum * 1024 * 1024)); then
      echo "$label requires ${minimum} GiB free on $path." >&2
      exit 1
    fi
  done
}

require_path_capacity 50 "NuxtJP verification" "${TMPDIR:-/tmp}"
verification_dir="$(mktemp -d "${TMPDIR:-/tmp}/nuxt-v8-verify.XXXXXX")"
trap 'rm -rf -- "$verification_dir"' EXIT
if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then
  export CARGO_TARGET_DIR="$verification_dir/cargo-target"
fi
[[ "$CARGO_TARGET_DIR" == /* && ! -L "$CARGO_TARGET_DIR" ]] || {
  echo "CARGO_TARGET_DIR must be an absolute non-symlink path." >&2; exit 1;
}
mkdir -p -- "$CARGO_TARGET_DIR"
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_NET_OFFLINE=true

require_capacity() {
  require_path_capacity "$1" "$2" "$CARGO_TARGET_DIR"
}

cargo_stage() {
  require_capacity 50 "Cargo stage"
  cargo "$@"
}

prepare_v8() {
  require_capacity 75 "NuxtJP V8 verification"
  local source="${RUSTY_V8_ARCHIVE:-}"
  local target="$verification_dir/librusty_v8.a"
  local expected="dd548eecb862fb8e746fc0ba4e0d926af94b2ea69299abefbbaac5fa9e3d8799"
  [[ "$source" == /* && -f "$source" && ! -L "$source" ]] || {
    echo "RUSTY_V8_ARCHIVE must name the reviewed local archive." >&2; exit 1;
  }
  cp --no-dereference -- "$source" "$target"
  chmod 400 "$target"
  [[ "$(sha256sum "$target" | awk '{ print $1 }')" == "$expected" ]] || {
    echo "RUSTY_V8_ARCHIVE digest is not approved." >&2; exit 1;
  }
  export RUSTY_V8_ARCHIVE="$target"
}

case "$tier" in
  standard)
    cargo_stage fmt --all -- --check
    cargo_stage clippy --locked --offline --no-default-features \
      --lib --bins --tests -- -D warnings
    cargo_stage test --locked --offline --no-default-features
    ;;
  full)
    prepare_v8
    cargo_stage fmt --all -- --check
    cargo_stage clippy --locked --offline --all-targets --all-features -- -D warnings
    cargo_stage test --locked --offline --all-targets --all-features
    ;;
  docs)
    prepare_v8
    DOCS_RS=1 RUSTDOCFLAGS="-D warnings" cargo_stage doc --locked --offline \
      --all-features --no-deps
    ;;
  feature-matrix)
    cargo_stage check --locked --offline --no-default-features --lib --bins --tests
    prepare_v8
    cargo_stage check --locked --offline --no-default-features \
      --features v8-engine --lib --bins --tests
    ;;
esac

echo "NuxtJP V8 worker ${tier} verification passed."
