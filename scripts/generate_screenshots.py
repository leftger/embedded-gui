#!/usr/bin/env python3
"""
Generate animated GIFs and screenshots for embedded-gui documentation.
"""

import os
import subprocess
from pathlib import Path
from PIL import Image

W, H = 320, 240
REPO_ROOT = Path(__file__).resolve().parent.parent
FRAMES_DIR = REPO_ROOT / "target" / "pipeline_frames"
OUTPUT_GIF = REPO_ROOT / "docs" / "screenshots" / "frosted_glass_pipeline.gif"

def main():
    print("1. Running showcase in record-frames mode...")
    subprocess.run(
        ["cargo", "run", "--example", "graphics_pipeline_showcase", "--", "--record-gif"],
        cwd=REPO_ROOT,
        check=True,
    )

    print("2. Compiling raw frames into animated GIF...")
    raw_files = sorted(FRAMES_DIR.glob("frame_*.raw"))
    if not raw_files:
        print("Error: No recorded frames found!")
        return

    images = []
    for f in raw_files:
        raw_bytes = f.read_bytes()
        img = Image.frombytes("RGB", (W, H), raw_bytes)
        images.append(img)

    OUTPUT_GIF.parent.mkdir(parents=True, exist_ok=True)
    images[0].save(
        OUTPUT_GIF,
        save_all=True,
        append_images=images[1:],
        duration=33,  # ~30 fps
        loop=0,
        optimize=True,
    )
    print(f"-> Successfully generated {OUTPUT_GIF} ({len(images)} frames, {os.path.getsize(OUTPUT_GIF)} bytes)")

if __name__ == "__main__":
    main()
