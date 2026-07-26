#!/usr/bin/env bash
# Build docs with warnings denied (broken links, missing docs, etc.).
set -euo pipefail

export RUSTDOCFLAGS="${RUSTDOCFLAGS:--D warnings}"

if [ -n "${CI_HOST_TARGET:-}" ] && [ "${CI_DOCS_FEATURES:-}" != "" ]; then
  # shellcheck disable=SC2086
  cargo doc --lib --no-deps ${CI_DOCS_FEATURES} --target "${CI_HOST_TARGET}"
else
  # shellcheck disable=SC2086
  cargo doc ${DOCS_ARGS:---lib --no-deps --all-features}
fi
