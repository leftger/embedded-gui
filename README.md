# embedded-gui

<p align="center">
  <img src="assets/aztec_rustacean.png" alt="embedded-gui" width="100%">
</p>

[![crates.io](https://img.shields.io/crates/v/embedded-gui.svg)](https://crates.io/crates/embedded-gui)
[![docs.rs](https://img.shields.io/docsrs/embedded-gui)](https://docs.rs/embedded-gui)
[![CI](https://github.com/leftger/embedded-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/leftger/embedded-gui/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

`embedded-gui` is a lightweight, deterministic, zero-allocation (`no_std`) GUI & HUD framework for microcontrollers and [`embedded-graphics`](https://crates.io/crates/embedded-graphics) displays.

Heavily inspired by modern wearable and smartwatch UI frameworks—its animation model, interaction contracts, and cinematic motion primitives draw from fluid, tactile embedded design patterns for widget composition, layout rules, and state-variant styling.

---

## Key Capabilities

- **Zero-Allocation (`no_std`)**: Built entirely on fixed-capacity data structures (`heapless`) with strict memory bounds and deterministic execution times.
- **Declarative KDL GUI Markup & Codegen**: Author complex UI screens in clean [KDL markup](https://kdl.dev) and compile directly into zero-allocation `#![no_std]` Rust code at build time with `include_gui!` or `gui_kdl!`.
- **2D Grid Layout Engine**: CSS-style track resolution (`"140px 1fr 2fr auto"`), cell placement, and multi-track spans (`col_span`, `row_span`).
- **Rich Built-in Widgets**: Buttons, Sliders, Dropdowns, Toggles, Checkboxes, Gauges, Meters, Sweeping Arcs, Plotters/Charts, TextAreas, On-Screen Keyboards, and Circular Lists.
- **Native Custom Widgets**: Extensible third-party widget support via type-erased `WidgetStorage<'a>` and object-safe `Widget` trait contracts.
- **Unified Motion Engine**: Tactile spatial easing curves (`moook`), spring dynamics, timeline keyframing, property mutator bindings, and screen stack transitions (flip-card, peek/glance, shutter, portal).
- **Decoupled Rendering Engine**: Bounding-box dirty tracking, opacity layering, software IIR blur, subpixel anti-aliasing, and custom display backends.
- **Async DMA & Double/Triple Buffering**: Zero-copy presentation via `CompletionSlot` and `StandardSwapChain`, fully compatible with Embassy `async/await` or bare-metal superloop polling.
- **Multi-Target Tested**: Continuously verified across ARM Cortex-M0/M0+ (`thumbv6m`), Cortex-M4F/M7F (`thumbv7em`), Cortex-M33/M55 (`thumbv8m.main`), and RISC-V (`riscv32imac`).

---

## Quick Start

### 1. Declarative KDL Markup (New in v0.2.1)

Author your UI screen in declarative KDL (`ui/dashboard.kdl`):
```kdl
screen id="Dashboard" width=320 height=240 theme="dark" {
    grid cols="140px 1fr" rows="24px 1fr 48px" gap=6 padding=8 {
        banner col=0 row=0 col_span=2 text="Living Room Climate"
        spinbox id="TempSetpoint" col=0 row=1 min=100 max=350 value=225 digits=4 decimals=1
        scale id="RoomGauge" col=1 row=1 mode="radial" min=10.0 max=40.0 value=22.5
        button id="FanBtn" col=0 row=2 text="FAN HIGH"
        toggle id="EcoMode" col=1 row=2 checked=true
    }
}
```

<div align="center">
  <img src="https://raw.githubusercontent.com/leftger/embedded-gui/master/docs/screenshots/kdl_generated_screen.png" width="420" alt="Generated KDL Screen Render"><br/>
  <sub><b>Zero-Allocation Result:</b> 2D Grid Layout, decimal spinbox, tachometer scale, button, and toggle compiled directly into pure <code>no_std</code> Rust.</sub>
</div>
<br/>

Include and compile directly into zero-allocation `#![no_std]` Rust code:
```rust
use embedded_gui::prelude::*;

// Embeds and compiles the KDL file at build time into DashboardApp and DashboardWidgets
include_gui!("ui/dashboard.kdl");

fn main() {
    let mut gui = GuiContext::<64, 32, 16>::new(Rect::new(0, 0, 320, 240));
    let app = DashboardApp::build(&mut gui).expect("failed to build UI");

    // Strongly-typed widget IDs generated automatically:
    // app.widgets.room_gauge
    // app.widgets.temp_setpoint
    // app.widgets.fan_btn
    // app.widgets.eco_mode
}
```

*For complete syntax, full widget catalog, and styling options, see the [Declarative KDL GUI Codegen Guide](./docs/kdl-gui-codegen-guide.md).*

### 2. Programmatic Fluent Builder API

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

<div align="center">

### Grand Showcase: 2D Grid Layout, Rich Controls & Vector Béziers
*Zero-allocation 2D `GridLayout` (fractional `fr` & fixed `px` tracks), radial tachometer & linear graduated scales, interactive table grid with cell navigation, precision decimal spinbox, and stroked Bézier vector paths.*

<img src="https://raw.githubusercontent.com/leftger/embedded-gui/master/docs/screenshots/rich_controls_grid_showcase.gif" width="640" alt="Grand Showcase: Rich Controls, 2D Grid, and Bézier Curves">

<br/><br/>

<table>
  <tr>
    <th width="50%" align="center">Accelerated Graphics Pipeline & Frosted Glass</th>
    <th width="50%" align="center">Smart Home & Industrial IoT Telemetry Dashboard</th>
  </tr>
  <tr>
    <td align="center">
      <img src="https://raw.githubusercontent.com/leftger/embedded-gui/master/docs/screenshots/frosted_glass_pipeline.gif" width="360" alt="Frosted Glass Pipeline"><br/>
      <sub><b>Hardware-accelerated 2D pipeline:</b> Moving IIR frosted glass blur overlays, alpha linear/radial gradients, and scanline blits.</sub>
    </td>
    <td align="center">
      <img src="https://raw.githubusercontent.com/leftger/embedded-gui/master/docs/screenshots/smart_home_dashboard.gif" width="360" alt="Smart Home Dashboard"><br/>
      <sub><b>Glassmorphic IoT Dashboard:</b> HVAC radial gauges, color temperature sliders, live power sparkline curve with area fills.</sub>
    </td>
  </tr>
  <tr>
    <th align="center">Wearable Subsystems, Pickers & Status Bar</th>
    <th align="center">Fluid Cinematic Transitions & Card Story Deck</th>
  </tr>
  <tr>
    <td align="center">
      <img src="https://raw.githubusercontent.com/leftger/embedded-gui/master/docs/screenshots/wearable_suite_showcase.gif" width="360" alt="Wearable Suite"><br/>
      <sub><b>Wearable OS:</b> Real-time status bar (charging battery, Bluetooth, clock), 12h/24h time picker, number roller, and action dialogs.</sub>
    </td>
    <td align="center">
      <img src="https://raw.githubusercontent.com/leftger/embedded-gui/master/docs/screenshots/cinematic_transitions.gif" width="360" alt="Cinematic Transitions"><br/>
      <sub><b>Cinematic Motion:</b> Spatial <code>moook</code> easing, 3D card story transitions, daily fitness rings, and sleep stage bar charts.</sub>
    </td>
  <tr>
    <th colspan="2" align="center">Embedded GUI Studio: Live KDL Editor & 60 FPS Motion Previewer</th>
  </tr>
  <tr>
    <td colspan="2" align="center">
      <img src="https://raw.githubusercontent.com/leftger/embedded-gui/master/docs/screenshots/embedded_gui_studio_preview.gif" width="700" alt="Embedded GUI Studio Preview"><br/>
      <sub><b>Cross-Platform Studio IDE:</b> Real-time KDL code editing, 60 FPS motion playback timeline, dynamic simulated LCD display, and instant <code>no_std</code> Rust codegen preview.</sub>
    </td>
  </tr>
</table>

</div>

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

- 📐 **[Declarative KDL GUI Markup & Codegen Guide](./docs/kdl-gui-codegen-guide.md)**: Complete reference manual for KDL screen syntax, 2D grid layouts, full widget catalog, vector Bézier paths, and zero-allocation Rust codegen.
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
