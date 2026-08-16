# embedded-gui

[![crates.io](https://img.shields.io/crates/v/embedded-gui.svg)](https://crates.io/crates/embedded-gui)
[![docs.rs](https://img.shields.io/docsrs/embedded-gui)](https://docs.rs/embedded-gui)
[![CI](https://github.com/leftger/embedded-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/leftger/embedded-gui/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

`embedded-gui` is a lightweight, deterministic, zero-allocation (`no_std`) GUI & HUD framework for microcontrollers and [`embedded-graphics`](https://crates.io/crates/embedded-graphics) displays.

Heavily inspired by modern wearable and smartwatch UI frameworks—its animation model, interaction contracts, and cinematic motion primitives draw from fluid, tactile embedded design patterns. **LVGL** serves as a secondary influence for widget composition, layout rules, and state-variant styling.

---

## Key Capabilities

- **Zero-Allocation (`no_std`)**: Built entirely on fixed-capacity data structures (`heapless`) with strict memory bounds and deterministic execution times.
- **Rich Built-in Widgets**: Buttons, Sliders, Dropdowns, Toggles, Checkboxes, Gauges, Meters, Sweeping Arcs, Plotters/Charts, TextAreas, On-Screen Keyboards, and Circular Lists.
- **Native Custom Widgets**: Extensible third-party widget support via type-erased `WidgetStorage<'a>` and object-safe `Widget` trait contracts.
- **Unified Motion Engine**: Tactile spatial easing curves (`moook`), spring dynamics, timeline keyframing, property mutator bindings, and screen stack transitions (flip-card, peek/glance, shutter, portal).
- **Decoupled Rendering Engine**: Bounding-box dirty tracking, opacity layering, software IIR blur, subpixel anti-aliasing, and custom display backends.
- **Async DMA & Double/Triple Buffering**: Zero-copy presentation via `CompletionSlot` and `StandardSwapChain`, fully compatible with Embassy `async/await` or bare-metal superloop polling.
- **Multi-Target Tested**: Continuously verified across ARM Cortex-M0/M0+ (`thumbv6m`), Cortex-M4F/M7F (`thumbv7em`), Cortex-M33/M55 (`thumbv8m.main`), and RISC-V (`riscv32imac`).

---

## Quick Start

```rust
use embedded_graphics::pixelcolor::Rgb565;
use embedded_gui::prelude::*;

// 1. Create a fixed-capacity GUI context (Max Widgets, Focus Group Capacity, Dirty Rects)
let mut gui = GuiContext::<16, 4, 8>::new(Rect::new(0, 0, 320, 240));

// 2. Spawn widgets using the fluent builder pattern
let status_label = gui.spawn(
    WidgetBuilder::new(Rect::new(10, 10, 150, 20))
        .with_style_class("header")
        .build()
)?;

// 3. Mutate properties dynamically using the generic property engine
gui.set_widget_property(status_label, PropertyKey::Text, PropertyValue::Text("SYSTEM OK"))?;

// 4. Render only dirty regions to your embedded-graphics DrawTarget
gui.render(&mut display)?;
```

---

## Visual Showcase

### Accelerated Graphics Pipeline & Frosted Glass Blur
![Frosted glass and graphics pipeline showcase](docs/screenshots/frosted_glass_pipeline.gif)

### Tactile Cinematic Motion & Transitions
![Animation and transition showcase](docs/screenshots/motion.gif)

### Flip-card Screen Stack Transitions
![Flipcard selection transition](docs/screenshots/flipcard.gif)

### Launcher Glance Tiles & Card Story Deck
![Cinematic peek glance carddeck showcase](docs/screenshots/cinematic.gif)

### Dashboard UI Composition
![Dashboard UI screenshot](docs/screenshots/dashboard.png)

### Mixed Typography & Font Models
![Mixed font showcase screenshot](docs/screenshots/fonts.png)

---

## Feature Architecture

### 1. Widgets & Layouts
- **Controls**: Buttons, Icon Buttons, Sliders, Toggles, Checkboxes, Dropdowns, Rollers.
- **Data & Display**: Progress Bars, Gauges, Meters, Sweeping Arcs, Plotters/Line Charts, Bar Charts, Busy Wheels.
- **Structure & Layout**: Linear Layouts (Row/Column with spacing & constraints), Panels, Tabs, Cards, Dialogs, Circular Lists.
- **Input & Text**: TextAreas (word wrap, selection, undo/redo), On-Screen Keyboards.

### 2. Motion Framework (`src/motion/`)
- **Easing & Physics**: Standard Easings (Linear, Quad, Cubic, Sine, Exponential) + Spatial Easing (`moook_curve`), Spring Physics, Inertia.
- **Timelines & Keyframes**: Multi-track property keyframing and sequence controllers.
- **Screen Stack Transitions**: Slide, Fade, Portal, Shutter, Modal Overlay, Round-Flip Card.

### 3. Render Engine (`src/render/`)
- **Dirty Region Tracking**: Merges overlapping invalidate rectangles to minimize SPI/I2C/Parallel bus transfers.
- **Compositing**: Software alpha blending, opacity stacks, subpixel anti-aliasing, and IIR blur filters (RGB565, RGBA8888, GRAY8).

### 4. Input & Semantics (`src/input/`)
- Multi-device event mapping: Rotary Encoders (CW/CCW/Press), D-Pad/Keyboards (Arrow keys, Select, Back), Touch/Pointer (Tap, Long Press, Drag, Flick).
- Configurable per-widget focus navigation, raw key policies, and event routing phases (Capture, Target, Bubble).

---

## Documentation & Guides

Detailed architecture specifications and integration guides are available in [`docs/`](./docs/):

- 🔤 **[Custom Font Abstraction & Interop Guide](./docs/custom-fonts-abstraction.md)**: Drop-in custom bitmap fonts (`BitmapFont`), `Font` trait abstraction, and `embedded-graphics` `MonoFont` interop.
- 🎬 **[Animation Presets Guide](./docs/animation-presets.md)**: Easing curves, spring physics, and timeline keyframing specifications.
- 🔀 **[Transition Presets Guide](./docs/transition-presets.md)**: Screen stack slide, fade, portal, and flip-card transition rules.
- 🎹 **[TextArea & Keybindings Specification](./docs/textarea-input-keybindings.md)**: Input policies, key bindings, and text editing behavior.
- 🎯 **[Interaction Behavior Contract](./docs/interaction-behavior-contract.md)**: Focus management, event bubble paths, and pointer semantics.

---

## Examples Directory

The repository includes showcase examples categorized under `examples/`:

| Directory | Purpose & Highlights |
|-----------|----------------------|
| **`examples/basics/`** | Core layout rules, custom font drop-in interop, dashboard layout, form flows, interaction semantics, raw key input, and keyboard navigation (`custom_font_showcase.rs`, `dashboard_app.rs`, `complex_layout_showcase.rs`). |
| **`examples/widgets/`** | Comprehensive widget showcases, gauges, sweeping arcs, alpha blending, and visual quality benchmarks (`widgets_showcase.rs`, `visual_quality_showcase.rs`, `sweeping_arc_widget_showcase.rs`). |
| **`examples/motion/`** | Motion framework, spring physics, dirty-region animation, timeline keyframing, and cinematic peek/glance cards (`animation_motion_showcase.rs`, `cinematic_peek_glance_carddeck_showcase.rs`). |
| **`examples/integrations/`** | Third-party interop, Embassy async frames, DMA swapchain simulation, and 3D graphics overlays (`embassy_gui_frame.rs`, `completion_swapchain_sim.rs`, `embedded_3dgfx_overlay.rs`). |

Run any example using Cargo:
```bash
cargo run --example dashboard_app --features std
cargo run --example animation_motion_showcase --features std
```

---

## Cargo Features

| Feature | Description |
|---------|-------------|
| `embedded-graphics` | *(Default)* Transparent support for `embedded-graphics` `MonoFont` references (`&FONT_6X10`, `&FONT_9X15`) in styles and text rendering. |
| `libm` | Provides floating-point math support (`f32::sin`, `cos`, `round`, `sqrt`) when building for `no_std` targets without standard library floats. |
| `rich-widgets` | Enables advanced visual widgets including Gauges, Plotters, TextAreas, and On-Screen Keyboards. |
| `embedded-text` | Enables interoperability adapters for `embedded-text` `TextBox`. |
| `embedded-layout` | Enables interoperability adapters for `embedded-layout` `View` alignment. |
| `embassy` | Adds `EmbassyWaitTransfer` and `FrameClock` for Embassy async executor integration. |
| `triple-buffering` | Enables triple-buffer swapchain for bursty display frame rates. |

---

## v0.2.0 Breaking Changes & Migration

Version `0.2.0` introduces a flexible, trait-based font system supporting custom raw bitmap arrays (`BitmapFont`) and dynamic font providers (`Font` trait):

- **`FontId` Enum Variants**: Added `FontId::Bitmap(&'static BitmapFont)` and `FontId::Dynamic(&'static dyn Font)`. Exhaustive `match` statements on `FontId` must include these new variants or a wildcard fallback arm (`_ => ...`).
- **Non-`const` Geometry Methods**: `FontId::advance()` and `FontId::line_height()` are now standard `fn` methods instead of `const fn` to allow dispatching to dynamic trait references.
- **Enhanced `CustomFont` Support**: Legacy 3x5 `PackedFont` usage can be upgraded to the new [`BitmapFont`](./docs/custom-fonts-abstraction.md) struct (`BitmapFont::new_8x16`, `new_8x8`, or custom dimensions) for arbitrary glyph sizes and `fill_rect` span acceleration.

---

## License

Dual-licensed under either of:

- **MIT License** ([`LICENSE-MIT`](./LICENSE-MIT))
- **Apache License, Version 2.0** ([`LICENSE-APACHE`](./LICENSE-APACHE))

at your option.
