# embedded-gui WebAssembly Demo

A browser demo that renders an `embedded-gui` screen into an HTML canvas with
the same retained widget tree, RGB565 renderer, and input contract used by
firmware targets.

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

## Interactive input

The demo wires browser events into the same `GuiContext::handle_input` API a
firmware event loop uses:

| Browser event | `embedded-gui` input |
|---------------|----------------------|
| Pointer press/move/release (left button) | `InputEvent::Pointer` with `PointerState::{Pressed, Moved, Released}` |
| Arrow keys | `Up` / `Down` / `Left` / `Right` spatial navigation |
| Enter / Space | `Select` (activate focused widget) |
| Backspace / Escape | `Back` |

Try:

- Click **CLICK ME** to increment the click counter and progress bar.
- Click the **ENABLE** toggle; its checked state changes immediately.
- Focus the slider with pointer or arrow keys, then use **Left/Right** to
  change its value.
- Use the arrow keys to move focus between controls, then press **Enter**.

## How it maps to firmware

The demo uses the exact `GuiContext` type and `render()` API a `no_std`
application uses. Only the final `DrawTarget` changes:

- Embedded: SPI/parallel display driver or a `Framebuffer` + `DisplayBackend`
- Browser: `WebSimulatorDisplay` → HTML canvas

Because `embedded-gui` renders to any `embedded-graphics`-compatible
`DrawTarget`, the same screen code can be compiled for both Cortex-M/RISC-V
firmware and `wasm32-unknown-unknown` without changing the widget tree.

## What is not included yet

The demo renders on demand after each input event. It does not yet drive a
continuous animation loop with `requestAnimationFrame`, so motion presets and
timeline-driven widgets are not running at 60 FPS in the browser yet.
