#!/usr/bin/env bash
# MSRV build with retries. Avoids rust-cache to reduce SIGPIPE/stale-artifact flakes.
set -euo pipefail

export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
export CARGO_NET_RETRY=10

MSRV="${MSRV:?MSRV must be set}"
CHECK_ARGS="${CHECK_ARGS:---lib --all-features}"

# device-driver-generation invokes rustfmt from build.rs; MSRV installs minimal profile only.
rustup component add --toolchain "${MSRV}" rustfmt

run_check() {
  if [ -n "${CI_HOST_TARGET:-}" ]; then
    # shellcheck disable=SC2086
    cargo "+${MSRV}" check ${CHECK_ARGS} --target "${CI_HOST_TARGET}"
  else
    # shellcheck disable=SC2086
    cargo "+${MSRV}" check ${CHECK_ARGS}
  fi
}

max_attempts=3
for attempt in $(seq 1 "$max_attempts"); do
  if run_check; then
    exit 0
  fi
  if [ "$attempt" -eq "$max_attempts" ]; then
    echo "MSRV check failed after ${max_attempts} attempts." >&2
    exit 1
  fi
  echo "MSRV check failed (attempt ${attempt}/${max_attempts}); cleaning and retrying in 10s..." >&2
  cargo clean
  sleep 10
done
