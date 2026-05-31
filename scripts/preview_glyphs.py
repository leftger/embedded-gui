#!/usr/bin/env python3
"""Preview 3x5 glyph override files in terminal."""

from __future__ import annotations

import sys
from pathlib import Path


def parse_key(token: str) -> str | None:
    token = token.strip()
    if token.lower() == "space":
        return " "
    if len(token) == 1:
        return token
    return None


def load_glyphs(path: Path) -> dict[str, list[str]]:
    glyphs: dict[str, list[str]] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or ":" not in line:
            continue
        key_raw, rows_raw = line.split(":", 1)
        key = parse_key(key_raw)
        if key is None:
            continue
        rows = [r.strip() for r in rows_raw.split(",")]
        if len(rows) != 5:
            continue
        if any(len(r) != 3 or any(c not in "01" for c in r) for r in rows):
            continue
        glyphs[key] = rows
    return glyphs


def render(rows: list[str]) -> list[str]:
    return ["".join("#" if c == "1" else "." for c in row) for row in rows]


def main() -> int:
    if len(sys.argv) < 3:
        print("Usage: preview_glyphs.py <glyph-file> <chars>")
        return 1

    path = Path(sys.argv[1])
    chars = sys.argv[2]
    if not path.exists():
        print(f"File not found: {path}")
        return 1

    glyphs = load_glyphs(path)
    for ch in chars:
        label = "space" if ch == " " else ch
        print(f"\n[{label}]")
        rows = glyphs.get(ch)
        if rows is None:
            print("(no override)")
            continue
        for row in render(rows):
            print(row)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
