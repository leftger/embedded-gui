# Embedded GUI Studio

A cross-platform desktop & web GUI studio for declarative KDL GUI editing, 60 FPS live animation preview, and `#![no_std]` Rust code generation for `embedded-gui`.

<div align="center">
  <img src="../../docs/screenshots/embedded_gui_studio_preview.gif" width="720" alt="Embedded GUI Studio Preview">
</div>

## Features

- **Live KDL Editor**: Write declarative GUI screens in KDL with real-time parser validation and inline error diagnostics.
- **🖥 Visual Preview**: Real-time 2D rendering of simulated embedded displays (with 1x, 1.5x, 2x zoom) with accurate CSS-like grid track calculations (`fr`, `px`, `auto`, `gap`, `padding`).
- **⏱ 60 FPS Animation Engine**: Live playback toolbar with Play/Pause, speed multipliers (0.5x, 1.0x, 2.0x), reset, and timeline scrubber slider to preview dynamic waveforms, oscillating gauges, and rotating spinners in real time.
- **🦀 Generated Rust**: Instant, zero-allocation `#![no_std]` Rust code generator preview with one-click clipboard copy.
- **🌲 AST Inspector**: Structured view of screen metrics, grid tracks, and placed widget hierarchy.
- **Starter Presets**: Built-in starter templates (Oscilloscope, Motion Kitchen Sink, Smart Thermostat, Sensor Dashboard).

## Running Locally

To launch the studio GUI:

```bash
cargo run -p embedded-gui-studio
```
