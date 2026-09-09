# Platform Bring-Up Guide

One recurring piece of community feedback about LVGL-style embedded UI stacks
is that the first board bring-up is the hardest part: the library itself is
fine once a screen is visible, but wiring the renderer to a concrete display
can feel undocumented. This guide is the `embedded-gui` answer to that
feedback.

`embedded-gui` deliberately keeps the same core for every target. A screen is
just:

1. A `GuiContext` with fixed capacities,
2. a `DrawTarget` that owns the pixels, and
3. a frame loop that renders and presents dirty regions.

Everything else (SPI timings, DMA channels, GPIOs, panel init sequences) stays
in your board crate where it belongs.

---

## 1. Dependencies

Add the crate with `default-features = false` and enable only the features your
UI needs:

```toml
[dependencies]
embedded-gui = { version = "0.2", default-features = false, features = ["libm", "rich-widgets"] }
```

Feature notes for bring-up:

| Feature | When to enable |
|---------|----------------|
| `rich-widgets` | Sliders, toggles, menus, gauges, text areas, keyboards, etc. |
| `libm` | Float math on targets without hardware FPU/libm support. |
| `embedded-graphics` | Interop with `embedded-graphics` fonts/drawables. |
| `embassy` | Async Embassy transfer/frame-clock integration. |
| `triple-buffering` | Extra swapchain slot for bursty displays. |

The crate itself remains `#![no_std]`; only the color type and font/rendering
interop features are additive.

## 2. Choose your display surface

`GuiContext::render()` writes to any `DrawTarget<Color = Rgb565>` from the
`embedded-graphics-core` ecosystem. There are three common bring-up surfaces:

### A. Direct draw target (simplest)

Use your display driver crate directly when it already implements `DrawTarget`
and you can afford to render straight into the controller's pixel stream.

```rust
#![no_std]

use embedded_gui::prelude::*;

// A driver from your BSP, for example:
// struct St7789<SPI, DC> { /* ... */ }
// impl<SPI, DC> DrawTarget for St7789<SPI, DC> {
//     type Color = Rgb565;
//     // ...
// }

fn main_loop(display: &mut impl embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>) {
    let mut gui = GuiContext::<32, 16, 16>::new(Rect::new(0, 0, 240, 240));

    gui.add_panel(Rect::new(0, 0, 240, 240), Style::panel()).unwrap();
    gui.add_label(Rect::new(10, 10, 100, 20), "Bring-up OK", Style::label()).unwrap();

    loop {
        gui.render(display).unwrap();
        // Wait for VSync / update only when dirty:
        // gui.render_dirty(display).unwrap();
    }
}
```

### B. Framebuffer + DMA/SPI backend

For double buffering, dirty-region transfers, and flicker-free updates, render
into an RGB565 `Framebuffer` and hand it to a display backend:

```rust
use embedded_gui::framebuffer::Framebuffer;
use embedded_gui::prelude::*;

// 240x240 RGB565 = 115,200 bytes. Pick a scratch/band buffer for partial updates.
let mut fb: Framebuffer<{ 240 * 240 }> = Framebuffer::new(240, 240);
let mut gui = GuiContext::<32, 16, 16>::new(Rect::new(0, 0, 240, 240));

gui.render(&mut fb).unwrap();

// fb.pixels_mut() is a plain row-major &mut [Rgb565].
// Send it to an SPI/parallel display, or implement DisplayBackend for
// ownership-based DMA transfers.
send_pixels_to_display(fb.pixels());
```

For asynchronous DMA, implement
[`DisplayBackend`](../crates/embedded-gui/src/display_backend.rs). The trait is
ownership-based: `start_dma_transfer(framebuffer)` returns a `DmaTransfer` token
that holds the buffer until `wait()`, so the compiler prevents rendering into a
buffer that DMA is still reading.

### C. Host simulator / browser

Iterate before hardware is ready:

```bash
cargo run --example simulator_menu --features std
```

or run the same widgets in a browser with the
[`web_demo/`](../web_demo/README.md) WebAssembly example.

## 3. Frame loop

A minimal deterministic loop looks like this:

```rust
loop {
    // 1. Poll input (encoder, keys, touch) and feed the context.
    //    gui.handle_input(InputEvent::Pointer { x, y, state, button })?;

    // 2. Advance timers for long-press, repeat, cursor blink, animations.
    //    gui.tick_input(16)?;

    // 3. Render only what changed. `render_dirty_buffered` is best for SPI.
    //    gui.render_dirty_buffered(&mut display, &mut scratch)?;

    // 4. Drain semantic events for application logic.
    //    while let Some(event) = gui.pop_event() { /* ... */ }

    // 5. Pace the frame (16 ms for 60 FPS, 33 ms for 30 FPS, etc.).
}
```

See [`docs/ux-priority-execution-plan.md`](./ux-priority-execution-plan.md)
and [`docs/interaction-behavior-contract.md`](./interaction-behavior-contract.md)
for the input/event contract details.

## 4. Bring-up checklist

- [ ] `cargo check` passes for your target with `--no-default-features` (see CI matrix for supported targets).
- [ ] A known-good `DrawTarget` (simulator or `Framebuffer`) renders labels/buttons.
- [ ] Display driver `DrawTarget::fill_solid` and `fill_contiguous` work; `embedded-gui` relies on them for fast paths.
- [ ] The `GuiContext` capacities (`NODES`, `EVENTS`, `DIRTY`) are large enough for your screen.
- [ ] The frame loop calls `tick_input` so long-press, repeat, and animations stay correct.
- [ ] Dirty rendering is used before optimizing SPI traffic; full repaints are fine for bring-up.
- [ ] The linker/panic config is in place before measuring flash (see [`memory-footprint.md`](./memory-footprint.md)).

## 5. Troubleshooting

- **Nothing renders** — confirm the `DrawTarget` color is `Rgb565` and the
  display controller is initialized with the same orientation/size as the
  `GuiContext` viewport.
- **Flicker** — switch from `gui.render(display)` to
  `render_dirty_buffered` with a small `Rgb565` scratch buffer.
- **`GuiError::WidgetsFull` / `EventsFull` / dirty full** — increase the
  const generics on `GuiContext::<NODES, EVENTS, DIRTY>`.
- **Flash/RAM unexpected** — read [`memory-footprint.md`](./memory-footprint.md)
  and the [`size_harness`](../size_harness/README.md); the root
  `.cargo/config.toml` contains the Cortex-M `-Tlink.x` flags needed for real
  ELF section measurements.
- **Touch/pointer offset** — hit-testing uses widget coordinates in the
  `GuiContext` viewport; scale raw touch coordinates to the same pixel space
  before creating `InputEvent::Pointer`.

## 6. Reference material in this repo

| Resource | Purpose |
|----------|---------|
| `crates/embedded-gui/examples/basics/simulator_menu.rs` | Host simulator with keyboard input and screen stack. |
| `crates/embedded-gui/examples/basics/dashboard_app.rs` | Common 320×240 dashboard shape. |
| `crates/embedded-gui/examples/integrations/embassy_gui_frame.rs` | Embassy `async` integration. |
| `crates/embedded-gui/examples/integrations/completion_swapchain_sim.rs` | Double-buffer/DMA swapchain simulation. |
| `size_harness/` | `no_std` firmware link target for footprint checks. |
| `crates/embedded-gui-studio/` | Live USB display agent workflow for real hardware bring-up. |
