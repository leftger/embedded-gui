# embedded-gui WebAssembly Demo

A minimal browser demo that renders an `embedded-gui` screen into an HTML
canvas with the same retained widget tree and RGB565 renderer used by
firmware targets.

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

## How it maps to firmware

The demo uses the exact `GuiContext` type and `render()` API a `no_std`
application uses. Only the final `DrawTarget` changes:

- Embedded: SPI/parallel display driver or a `Framebuffer` + `DisplayBackend`
- Browser: `WebSimulatorDisplay` → HTML canvas

Because `embedded-gui` renders to any `embedded-graphics`-compatible
`DrawTarget`, the same screen code can be compiled for both Cortex-M/RISC-V
firmware and `wasm32-unknown-unknown` without changing the widget tree.

## What is not included yet

This first demo is a static frame. The natural next step is wiring input
events (`InputEvent::Pointer`, keyboard/encoder events) from browser DOM
events into `GuiContext::handle_input`, and driving an animation loop with
`requestAnimationFrame`.
