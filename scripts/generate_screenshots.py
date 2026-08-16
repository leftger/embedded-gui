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

def compile_gif(frames_dir: Path, output_gif: Path, duration_ms: int = 33):
    raw_files = sorted(frames_dir.glob("frame_*.raw"))
    if not raw_files:
        print(f"Error: No recorded frames found in {frames_dir}!")
        return

    images = []
    for f in raw_files:
        raw_bytes = f.read_bytes()
        img = Image.frombytes("RGB", (W, H), raw_bytes)
        images.append(img)

    output_gif.parent.mkdir(parents=True, exist_ok=True)
    images[0].save(
        output_gif,
        save_all=True,
        append_images=images[1:],
        duration=duration_ms,
        loop=0,
        optimize=True,
    )
    print(f"-> Successfully generated {output_gif} ({len(images)} frames, {os.path.getsize(output_gif)} bytes)")

def main():
    # 1. Pipeline & frosted glass showcase
    print("1. Generating frosted glass & pipeline showcase GIF...")
    subprocess.run(
        ["cargo", "run", "--example", "graphics_pipeline_showcase", "--", "--record-gif"],
        cwd=REPO_ROOT,
        check=True,
    )
    compile_gif(
        REPO_ROOT / "target" / "pipeline_frames",
        REPO_ROOT / "docs" / "screenshots" / "frosted_glass_pipeline.gif",
    )

    # 2. Rich controls & 2D GridLayout showcase
    print("\n2. Generating rich controls, 2D GridLayout & Bézier showcase GIF...")
    subprocess.run(
        ["cargo", "run", "--example", "rich_controls_grid_showcase", "--", "--record-gif"],
        cwd=REPO_ROOT,
        check=True,
    )
    compile_gif(
        REPO_ROOT / "target" / "controls_frames",
        REPO_ROOT / "docs" / "screenshots" / "rich_controls_grid_showcase.gif",
    )

if __name__ == "__main__":
    main()
