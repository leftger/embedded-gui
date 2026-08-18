#!/usr/bin/env python3
"""Generates the demo project's binary-ish art so the repo carries no opaque blobs.

Emits a seven-segment BDF font, three 1-bit icon layers as BMPs, and an OBJ
solid. Run from anywhere: paths are resolved relative to this file.
"""

import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# --- Seven-segment digit font -------------------------------------------------

CELL_W, CELL_H = 18, 30
STROKE = 4
MARGIN = 2

SEGMENTS = {
    #      a      b      c      d      e      f      g
    "0": "abcdef",
    "1": "bc",
    "2": "abdeg",
    "3": "abcdg",
    "4": "bcfg",
    "5": "acdfg",
    "6": "acdefg",
    "7": "abc",
    "8": "abcdefg",
    "9": "abcdfg",
}


def segment_pixels(name):
    """Returns the pixel spans of one seven-segment bar within the cell."""
    left, right = MARGIN, CELL_W - MARGIN - 1
    top, bottom = MARGIN, CELL_H - MARGIN - 1
    middle = CELL_H // 2
    if name == "a":
        return [(x, y) for x in range(left, right + 1) for y in range(top, top + STROKE)]
    if name == "d":
        return [(x, y) for x in range(left, right + 1) for y in range(bottom - STROKE + 1, bottom + 1)]
    if name == "g":
        return [
            (x, y)
            for x in range(left, right + 1)
            for y in range(middle - STROKE // 2, middle - STROKE // 2 + STROKE)
        ]
    if name == "b":
        return [(x, y) for x in range(right - STROKE + 1, right + 1) for y in range(top, middle + 1)]
    if name == "c":
        return [(x, y) for x in range(right - STROKE + 1, right + 1) for y in range(middle, bottom + 1)]
    if name == "f":
        return [(x, y) for x in range(left, left + STROKE) for y in range(top, middle + 1)]
    if name == "e":
        return [(x, y) for x in range(left, left + STROKE) for y in range(middle, bottom + 1)]
    raise ValueError(name)


def render_digit(digit):
    grid = [[0] * CELL_W for _ in range(CELL_H)]
    for seg in SEGMENTS[digit]:
        for x, y in segment_pixels(seg):
            grid[y][x] = 1
    return grid


def write_bdf(path):
    bytes_per_row = (CELL_W + 7) // 8
    lines = [
        "STARTFONT 2.1",
        "FONT -demo-sevenseg-medium-r-normal--30-300-72-72-m-180-iso8859-1",
        f"SIZE {CELL_H} 72 72",
        f"FONTBOUNDINGBOX {CELL_W} {CELL_H} 0 0",
        "STARTPROPERTIES 2",
        f"FONT_ASCENT {CELL_H}",
        "FONT_DESCENT 0",
        "ENDPROPERTIES",
        f"CHARS {len(SEGMENTS)}",
    ]
    for digit in sorted(SEGMENTS):
        grid = render_digit(digit)
        lines += [
            f"STARTCHAR {digit}",
            f"ENCODING {ord(digit)}",
            "SWIDTH 600 0",
            f"DWIDTH {CELL_W} 0",
            f"BBX {CELL_W} {CELL_H} 0 0",
            "BITMAP",
        ]
        for row in grid:
            value = 0
            for x, bit in enumerate(row):
                if bit:
                    value |= 1 << (bytes_per_row * 8 - 1 - x)
            lines.append(f"{value:0{bytes_per_row * 2}X}")
        lines.append("ENDCHAR")
    lines.append("ENDFONT")
    path.write_text("\n".join(lines) + "\n")


# --- 1-bit icon layers --------------------------------------------------------


def write_bmp(path, grid):
    """Writes a 24-bit BMP; ink pixels are black, paper is white."""
    height = len(grid)
    width = len(grid[0])
    row_bytes = width * 3
    padding = (4 - row_bytes % 4) % 4
    pixel_data = bytearray()
    for row in reversed(grid):  # BMP rows run bottom-up
        for value in row:
            pixel_data += b"\x00\x00\x00" if value else b"\xff\xff\xff"
        pixel_data += b"\x00" * padding
    header = struct.pack(
        "<2sIHHIIiiHHIIiiII",
        b"BM",
        14 + 40 + len(pixel_data),
        0,
        0,
        14 + 40,
        40,
        width,
        height,
        1,
        24,
        0,
        len(pixel_data),
        2835,
        2835,
        0,
        0,
    )
    path.write_bytes(header + pixel_data)


def blank(width, height):
    return [[0] * width for _ in range(height)]


def battery_shell():
    grid = blank(24, 12)
    for x in range(0, 21):
        grid[0][x] = grid[11][x] = 1
    for y in range(12):
        grid[y][0] = grid[y][20] = 1
    for y in range(4, 8):
        grid[y][21] = grid[y][22] = 1
    return grid


def battery_fill():
    grid = blank(24, 12)
    for y in range(3, 9):
        for x in range(3, 12):
            grid[y][x] = 1
    return grid


def battery_bolt():
    grid = blank(24, 12)
    bolt = [
        (14, 2), (15, 2), (16, 2),
        (13, 3), (14, 3), (15, 3),
        (12, 4), (13, 4), (14, 4), (15, 4), (16, 4),
        (13, 5), (14, 5), (15, 5),
        (12, 6), (13, 6), (14, 6),
        (13, 7), (14, 7), (15, 7),
        (14, 8), (15, 8),
    ]
    for x, y in bolt:
        grid[y][x] = 1
    return grid


# --- Mesh ---------------------------------------------------------------------


def write_obj(path):
    """A beveled octahedral gem: small enough to rasterize on an MCU."""
    vertices = [
        (0.0, 1.0, 0.0),
        (1.0, 0.0, 0.0),
        (0.0, 0.0, 1.0),
        (-1.0, 0.0, 0.0),
        (0.0, 0.0, -1.0),
        (0.0, -1.0, 0.0),
    ]
    faces = [
        (1, 2, 3), (1, 3, 4), (1, 4, 5), (1, 5, 2),
        (6, 3, 2), (6, 4, 3), (6, 5, 4), (6, 2, 5),
    ]
    lines = ["# demo gem generated by make_demo_assets.py"]
    lines += [f"v {x} {y} {z}" for x, y, z in vertices]
    lines += [f"f {a} {b} {c}" for a, b, c in faces]
    path.write_text("\n".join(lines) + "\n")


def main():
    (ROOT / "assets" / "fonts").mkdir(parents=True, exist_ok=True)
    (ROOT / "assets" / "icons").mkdir(parents=True, exist_ok=True)
    (ROOT / "assets" / "meshes").mkdir(parents=True, exist_ok=True)

    write_bdf(ROOT / "assets" / "fonts" / "sevenseg30.bdf")
    write_bmp(ROOT / "assets" / "icons" / "batt_shell.bmp", battery_shell())
    write_bmp(ROOT / "assets" / "icons" / "batt_fill.bmp", battery_fill())
    write_bmp(ROOT / "assets" / "icons" / "batt_bolt.bmp", battery_bolt())
    write_obj(ROOT / "assets" / "meshes" / "gem.obj")
    print("wrote demo assets under", ROOT / "assets")


if __name__ == "__main__":
    main()
