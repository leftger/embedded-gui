# Size Harness

The `size_harness` crate is a standalone `no_std` firmware binary used to keep
`embedded-gui` flash/RAM growth visible to contributors and users.

It intentionally contains the same shape as a small real application:
a `GuiContext` with a panel, label, button, progress bar, and slider, rendered
to a no-op display inside an infinite loop.

## Why it lives outside the workspace

Like `web_demo/`, this crate has its own `[workspace]` and target directory so
embedded linker configuration does not leak into the main library's host CI
matrix.

## Build and measure

Install the ARM target and binutils if needed:

```bash
rustup target add thumbv7em-none-eabihf
# Ubuntu/Debian:
sudo apt-get install -y binutils-arm-none-eabi
```

Build the release ELF from the repository root:

```bash
cargo build --manifest-path size_harness/Cargo.toml --target thumbv7em-none-eabihf --release
arm-none-eabi-size size_harness/target/thumbv7em-none-eabihf/release/size_harness
```

The root `.cargo/config.toml` adds the `-Tlink.x` flag required by
`cortex-m-rt`; without it the produced ELF has empty `.text` sections and the
footprint report is meaningless.

## Current reference measurement

Measured from `master` with the release profile below (opt-level `z`, LTO,
`codegen-units = 1`, panic abort):

| Section | Bytes |
|---------|-------|
| `.text` (flash) | 71,360 |
| `.data` | 0 |
| `.bss` | 0 |
| Total flash (`text + data`) | 71,360 (~69.7 KiB) |

> `.data`/`.bss` are 0 because this harness keeps `GuiContext` on the stack.
> A real application must budget stack for the context plus any framebuffer,
> scratch buffers, and DMA buffers. To measure the framework's static state
> instead, place your `GuiContext` in a `static`/`static mut` and rerun
> `arm-none-eabi-size`.

## CI

The `size-harness` CI job builds this crate on every push/PR and writes the
`arm-none-eabi-size` output into the GitHub Actions step summary so a change
that grows flash by a meaningful amount is visible without downloading an ELF.
