#!/usr/bin/env bash
# Runs unit tests, each integration test target, optional extra commands, and an integration check.
set -euo pipefail

cargo_test() {
  if [ -n "${CI_HOST_TARGET:-}" ]; then
    cargo test --target "$CI_HOST_TARGET" "$@"
  else
    cargo test "$@"
  fi
}

UNIT_ARGS="${CI_UNIT_TEST_ARGS:---lib --all-features}"
INTEGRATION_ARGS="${CI_INTEGRATION_TEST_ARGS:---all-features}"

echo "::group::Unit tests"
# shellcheck disable=SC2086
cargo_test ${UNIT_ARGS}

if [ -d tests ] && compgen -G "tests/*.rs" >/dev/null; then
  echo "::endgroup::"
  echo "::group::Integration tests"
  for test_file in tests/*.rs; do
    test_name="$(basename "$test_file" .rs)"
    echo "Running integration test: ${test_name}"
    # shellcheck disable=SC2086
    cargo_test --test "${test_name}" ${INTEGRATION_ARGS}
  done
fi

if [ -n "${CI_EXTRA_TEST_COMMANDS:-}" ]; then
  echo "::endgroup::"
  echo "::group::Feature tests"
  while IFS= read -r cmd; do
    [ -z "$cmd" ] && continue
    [[ "$cmd" == "|" ]] && continue
    echo "Running: ${cmd}"
    # shellcheck disable=SC2086
    eval "${cmd}"
  done <<< "${CI_EXTRA_TEST_COMMANDS}"
fi

echo "::endgroup::"
if [ "${CI_INTEGRATION_CHECK:-true}" = "true" ]; then
  echo "::group::Integration check"
  check_cmd="${CI_INTEGRATION_CHECK_CMD:-cargo build --examples --all-features}"
  echo "Running: ${check_cmd}"
  # shellcheck disable=SC2086
  eval "${check_cmd}"
  echo "::endgroup::"
fi

echo "All test and integration checks passed."
