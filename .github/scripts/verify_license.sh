#!/usr/bin/env bash
set -euo pipefail

test -f LICENSE
test -f LICENSE-MIT
test -f LICENSE-APACHE
grep -q 'MIT OR Apache-2.0' Cargo.toml

OUT="$(mktemp)"
trap 'rm -f "$OUT"' EXIT

licensee detect . >"$OUT"

grep -A6 '^LICENSE-MIT:' "$OUT" | grep -q 'Confidence:    100.00%'
grep -A6 '^LICENSE-MIT:' "$OUT" | grep -q 'License:       MIT'
grep -A6 '^LICENSE-APACHE:' "$OUT" | grep -q 'Confidence:    100.00%'
grep -A6 '^LICENSE-APACHE:' "$OUT" | grep -q 'License:       Apache-2.0'

echo "GitHub licensee check passed (MIT + Apache-2.0 at 100% confidence)."
