# embedded-gui Animated WebAssembly Showcase

A multi-screen browser demo that renders `embedded-gui` widgets into real
320×240 `GuiContext` screens and animates between them with the same
transition effects firmware can use.

**Live demo:** <https://leftger.github.io/embedded-gui/>

Community context: LVGL users noted that a great way to iterate on an
embedded UI is to run the same UI in a browser (WASM/emscripten) rather than
deploying to the board on every change. This crate is the `embedded-gui`
version of that workflow.

## Build

Install the WebAssembly target and `wasm-pack`:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Then build the demo package:

```bash
cd web_demo
wasm-pack build --target web --release
```

This creates `web_demo/pkg/`.

## Run

Serve the directory locally (any static file server works):

```bash
cd web_demo
python3 -m http.server 8000
```

Open <http://localhost:8000/>. The demo creates a 320×240 RGB565 UI at 2×
canvas scale and flushes it to the page.

## What the showcase includes

Each screen is a separate `GuiContext` with its own widget layout:

| Screen | Highlights |
|--------|------------|
| **Controls** | Button, toggle, checkbox, icon button, progress bar, slider, value labels, tabs |
| **Lists & Menus** | Menu, list, dropdown, roller |
| **Data & Gauges** | Chart, plotter, arc gauge, gauge |
| **Text & Input** | Textarea, table, keyboard |
| **Motion** | Carousel, card deck |
| **Overlays & Feedback** | State surface, heads-up banner, notification sheet |

Navigation uses embedded-gui's real screen-transition renderer with effects
that rotate per screen: **PushMoook, slide, shutter, round flip, port-hole,
wipe, circular reveal, fade, and zoom**.

While a screen is visible, its in-widget motion runs continuously: progress
bars breathe, gauges sweep, tabs/rollers/dropdowns auto-cycle, carousels drift,
and the state-surface/heads-up widgets animate on their own timelines. If no
input is received for a few seconds the demo auto-advances to the next screen.

## Interactive input

The demo wires browser events into the same `GuiContext::handle_input` API a
firmware event loop uses:

| Browser event | `embedded-gui` action |
|---------------|------------------------|
| Pointer press/release (left button) | `InputEvent::Pointer` with `PointerState::{Pressed, Released}` |
| Arrow keys | `Up` / `Down` / `Left` / `Right` spatial navigation |
| Enter / Space | `Select` (activate focused widget) |
| Backspace / Escape | `Back` |
| `PREV` / `NEXT` buttons | Animated screen transition |
| Shift + Left / Right | Animated screen transition |

Try:

- Leave a screen alone for a moment and watch the widgets animate themselves.
- Cycle through all five category screens and watch the transition effect change.
- Click **CLICK ME** to increment the click counter.
- Click the **ENABLE** toggle; its checked state changes immediately.
- Use arrow keys + **Enter** to interact with menus, dropdowns, and lists.

## How it maps to firmware

The demo uses the exact `GuiContext` type, `render()` API, and
`render_transition_pair` transition renderer a `no_std` application uses. Only
the final `DrawTarget` changes:

- Embedded: SPI/parallel display driver or a `Framebuffer` + `DisplayBackend`
- Browser: `WebSimulatorDisplay` → HTML canvas

Because `embedded-gui` renders to any `embedded-graphics`-compatible
`DrawTarget`, the same screen code can be compiled for both Cortex-M/RISC-V
firmware and `wasm32-unknown-unknown` without changing the widget tree.

## What is not included yet

The demo drives the animation loop at ~30 FPS and covers the most visible
in-widget motion paths. It is not yet running full cinematic keyframe decks,
spring physics, or every motion preset on every screen.
