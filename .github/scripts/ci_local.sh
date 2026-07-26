#!/usr/bin/env bash
# Approximate GitHub Actions CI locally. Reads env from .github/workflows/ci.yml.
# Usage: from repo root: bash .github/scripts/ci_local.sh
# Env: CI_LOCAL_SKIP="publish,audit,msrv" to skip slow/optional jobs.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"

workflow=".github/workflows/ci.yml"
if [[ ! -f "$workflow" ]]; then
  echo "error: missing $workflow" >&2
  exit 1
fi

load_env() {
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*([A-Z_]+):[[:space:]]*(.*)$ ]] || continue
    key="${BASH_REMATCH[1]}"
    val="${BASH_REMATCH[2]}"
    val="${val%\"}"
    val="${val#\"}"
    val="${val//\\n/$'\n'}"
    export "$key=$val"
  done < <(awk '/^env:/{flag=1;next} /^jobs:/{flag=0} flag && /^  [A-Z]/' "$workflow")
}

should_skip() {
  local job="$1"
  [[ "${CI_LOCAL_SKIP:-}" == *"$job"* ]]
}

resolve_host_target() {
  if [[ -n "${CI_HOST_TARGET:-}" ]]; then
    local host
    host="$(rustc -vV | sed -n 's/^host: //p')"
    case "$CI_HOST_TARGET" in
      x86_64-unknown-linux-gnu | x86_64-apple-darwin | aarch64-apple-darwin)
        export CI_HOST_TARGET="$host"
        ;;
    esac
  fi
}

host_target_args() {
  if [[ -n "${CI_HOST_TARGET:-}" ]]; then
    echo "--target" "${CI_HOST_TARGET}"
  fi
}

run_check_no_std() {
  if [[ -z "${NO_STD_TARGET:-}" ]]; then
    echo "SKIP: no_std target not configured"
    return 0
  fi
  case "$(basename "$repo_root")" in
    bq25887)
      rustup target add thumbv8m.main-none-eabihf
      cargo check --target thumbv8m.main-none-eabihf
      cargo check --target thumbv8m.main-none-eabihf --features defmt-03
      cargo check --target thumbv8m.main-none-eabihf --features log
      ;;
    lis2de12)
      rustup target add thumbv8m.main-none-eabihf
      cargo check --target thumbv8m.main-none-eabihf
      cargo check --target thumbv8m.main-none-eabihf --features async
      cargo check --target thumbv8m.main-none-eabihf --features defmt-03
      cargo check --target thumbv8m.main-none-eabihf --features async,defmt-03
      ;;
    *)
      rustup target add "${NO_STD_TARGET}"
      # shellcheck disable=SC2086
      cargo check --lib ${NO_STD_CARGO_ARGS:---no-default-features} --target "${NO_STD_TARGET}"
      ;;
  esac
}

run_msrv() {
  if ! rustup toolchain list | grep -q "${MSRV}"; then
    rustup toolchain install "${MSRV}" --profile minimal
  fi
  bash .github/scripts/ci_msrv.sh
}

run_clippy() {
  # shellcheck disable=SC2086
  cargo clippy ${CLIPPY_ARGS:---lib --all-features} $(host_target_args) -- -W clippy::all
  if [[ -z "${NO_STD_TARGET:-}" ]]; then
    return 0
  fi
  rustup target add "${NO_STD_TARGET}"
  # shellcheck disable=SC2086
  cargo clippy --lib ${NO_STD_CARGO_ARGS:---no-default-features} --target "${NO_STD_TARGET}" -- -W clippy::all
}

run_docs() {
  if [[ -n "${CI_HOST_TARGET:-}" && "$(basename "$repo_root")" == "lis2de12" ]]; then
    export CI_DOCS_FEATURES="--features std"
  fi
  bash .github/scripts/ci_docs.sh
}

run_check() {
  # shellcheck disable=SC2086
  if [[ -n "${CI_HOST_TARGET:-}" ]]; then
    cargo check ${CHECK_ARGS:---lib --all-features} $(host_target_args)
  else
    cargo check ${CHECK_ARGS:---lib --all-features}
  fi
}

run_step() {
  local name="$1"
  shift
  echo ""
  echo "========== $name =========="
  "$@"
}

failures=0
record_failure() {
  echo "FAILED: $1" >&2
  failures=$((failures + 1))
}

try_step() {
  local name="$1"
  shift
  if should_skip "$name"; then
    echo "SKIP: $name"
    return 0
  fi
  if run_step "$name" "$@"; then
    echo "OK: $name"
  else
    record_failure "$name"
  fi
}

load_env
resolve_host_target
unset RUSTFLAGS 2>/dev/null || true

echo "Simulating CI for: $(basename "$repo_root")"
echo "MSRV=$MSRV NO_STD_TARGET=${NO_STD_TARGET:-<none>} CI_HOST_TARGET=${CI_HOST_TARGET:-<host>}"

try_step fmt cargo fmt --all -- --check
try_step check run_check
try_step check-no-std run_check_no_std
try_step msrv run_msrv
try_step clippy run_clippy
try_step test bash .github/scripts/ci_test.sh
try_step docs run_docs
try_step publish cargo publish --dry-run --allow-dirty

if command -v cargo-audit >/dev/null 2>&1; then
  try_step audit cargo audit
else
  echo "SKIP: audit (cargo-audit not installed)"
fi

if [[ -f .github/scripts/verify_license.sh ]]; then
  try_step license bash .github/scripts/verify_license.sh
fi

echo ""
if [[ "$failures" -eq 0 ]]; then
  echo "CI local simulation passed for $(basename "$repo_root")."
  exit 0
fi

echo "CI local simulation failed ($failures job(s)) for $(basename "$repo_root")." >&2
exit 1
