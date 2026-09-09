# Memory Footprint Transparency

LVGL's HN discussion included a reminder that a 32 KB RAM / 128 KB flash
requirement is not insignificant for many MCUs. `embedded-gui` targets the
same class of hardware, so memory numbers should be public, reproducible, and
kept visible in CI.

## Reference measurement

The [`size_harness`](../size_harness/README.md) builds a small real firmware
binary (panel, label, button, progress bar, slider) with the recommended
embedded release profile. Current numbers for `thumbv7em-none-eabihf`:

| Metric | Value |
|--------|-------|
| Flash (`.text` + `.data`) | 71,360 B (~69.7 KiB) |
| Static RAM (`.data` + `.bss`) | 0 B in harness |
| Release profile | `opt-level = "z"`, LTO, `codegen-units = 1`, `panic = "abort"` |
| Crate features | `default-features = false`, `features = ["libm", "rich-widgets"]` |
| MCU target | `thumbv7em-none-eabihf` |

The harness stores `GuiContext` on the stack, so the static RAM column is 0.
Firmware RAM budgeting must add:

- stack space for the `GuiContext` you instantiate (or move it to a `static`
  to count it as BSS),
- an RGB565 framebuffer if you use one (`width × height × 2` bytes),
- scratch buffers passed to `render_dirty_buffered`,
- DMA/SPI buffers and any display driver state.

The `.cargo/config.toml` at the repository root now supplies the `-Tlink.x`
linker flag needed by `cortex-m-rt`. Without it the size harness ELF was not
fully linked and reported `0` bytes — the exact kind of misleading footprint
data this effort is meant to remove.

## Reproduce locally

```bash
rustup target add thumbv7em-none-eabihf
sudo apt-get install -y binutils-arm-none-eabi

cargo build --manifest-path size_harness/Cargo.toml --target thumbv7em-none-eabihf --release
arm-none-eabi-size size_harness/target/thumbv7em-none-eabihf/release/size_harness
```

The CI `size-harness` job runs the same commands and writes the output to the
GitHub Actions step summary for every push and pull request.

## How to keep your own footprint small

- Use `default-features = false`; do not enable `std`, `image-decode`, or
  optional interop crates unless your app uses them.
- Keep `GuiContext::<NODES, EVENTS, DIRTY>` capacities as small as the screen
  actually needs; each capacity directly sizes fixed `heapless::Vec`s.
- Prefer dirty/buffered rendering for SPI displays; full repaints waste CPU and
  bus time, not just memory.
- Keep fonts/bitmaps in `static` data and let the linker garbage-collect unused
  glyphs (`opt-level = "z"` + LTO already helps).
- For very small targets, the `rich-widgets` feature can be disabled; core
  labels, buttons, progress bars, and sliders in the size harness use it, so
  removing it lowers flash further.

## Community context

This document is part of a set of community-driven improvements. If you have a
different MCU class or feature set in mind, clone the size harness, change the
widgets/features, and share the `arm-none-eabi-size` output in a PR — visible
numbers make trade-offs easier for everyone.
