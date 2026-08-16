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

def run_showcase(example_name: str, frames_dir: Path, output_gif: Path, duration_ms: int = 33):
    print(f"\n--- Generating {example_name} -> {output_gif.name} ---")
    subprocess.run(
        ["cargo", "run", "--example", example_name, "--", "--record-gif"],
        cwd=REPO_ROOT,
        check=True,
    )
    compile_gif(frames_dir, output_gif, duration_ms)

def main():
    screenshots_dir = REPO_ROOT / "docs" / "screenshots"

    # 1. Grand Showcase: Rich controls, 2D grid layout, Béziers
    run_showcase(
        "rich_controls_grid_showcase",
        REPO_ROOT / "target" / "controls_frames",
        screenshots_dir / "rich_controls_grid_showcase.gif",
    )

    # 2. Pipeline & frosted glass showcase
    run_showcase(
        "graphics_pipeline_showcase",
        REPO_ROOT / "target" / "pipeline_frames",
        screenshots_dir / "frosted_glass_pipeline.gif",
    )

    # 3. Smartwatch & Wearable Suite
    run_showcase(
        "wearable_dialogs_pickers_status_showcase",
        REPO_ROOT / "target" / "wearable_frames",
        screenshots_dir / "wearable_suite_showcase.gif",
    )

    # 4. Smart Home IoT Dashboard
    run_showcase(
        "smart_home_dashboard",
        REPO_ROOT / "target" / "dashboard_frames",
        screenshots_dir / "smart_home_dashboard.gif",
    )

    # 5. Cinematic Transitions & Card Story Deck
    run_showcase(
        "cinematic_transitions_showcase",
        REPO_ROOT / "target" / "cinematic_frames",
        screenshots_dir / "cinematic_transitions.gif",
    )

if __name__ == "__main__":
    main()
