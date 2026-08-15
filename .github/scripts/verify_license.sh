#!/usr/bin/env bash
set -euo pipefail

test -f LICENSE
test -f LICENSE-MIT
test -f LICENSE-APACHE
grep -q 'MIT OR Apache-2.0' Cargo.toml

if ! command -v licensee >/dev/null 2>&1; then
    echo "Basic license files verified. (install 'licensee' for 100% confidence deep scan)."
    exit 0
fi

OUT="$(mktemp)"
trap 'rm -f "$OUT"' EXIT

licensee detect . >"$OUT"

grep -A6 '^LICENSE-MIT:' "$OUT" | grep -q 'Confidence:    100.00%'
grep -A6 '^LICENSE-MIT:' "$OUT" | grep -q 'License:       MIT'
grep -A6 '^LICENSE-APACHE:' "$OUT" | grep -q 'Confidence:    100.00%'
grep -A6 '^LICENSE-APACHE:' "$OUT" | grep -q 'License:       Apache-2.0'

echo "GitHub licensee check passed (MIT + Apache-2.0 at 100% confidence)."
