# Hardware Acceleration

Community feedback around embedded GUI libraries frequently mentions silicon
2D engines: NXP PXP, STM32 DMA2D/Chrom-ART, ESP32 DMA, and similar blitters.
This document explains the acceleration abstraction `embedded-gui` already
exposes and how to wire a chip-specific engine into the render pipeline.

---

## 1. Where the abstraction lives

There are two complementary layers:

1. **`Hardware2DAccelerator`** (`crates/embedded-gui/src/render/accelerator.rs`)
   models the raw operations a silicon 2D engine performs on a row-major RGB565
   buffer:

   - `fill_rect(dest, dest_stride, rect, color)` — solid fill (e.g. DMA2D R2M)
   - `copy_rect(src, src_stride, src_rect, dest, dest_stride, dest_pos)` — M2M blit
   - `blend_rect(fg, fg_stride, fg_rect, fg_alpha, bg, bg_stride, bg_pos)` — alpha blend

   `Software2DAccelerator` is the fallback implementation; it lets the same code
   run on targets without a blitter.

2. **`DrawUnit<D>`** (`crates/embedded-gui/src/render/task.rs`) is the
   rasterizer plug-in point used by `dispatch_draw_tasks`. A draw unit inspects
   a [`DrawTask`] and either executes it or returns `false` so a later unit or
   the software fallback handles it.

`HardwareAcceleratorDrawUnit<A>` is provided as a convenience wrapper that
routes solid rectangle fills to the target's fast `fill_solid` path. For
engines that operate directly on a framebuffer slice, implement your own
`DrawUnit<Framebuffer<N>>` and call the `Hardware2DAccelerator` methods on
`fb.pixels_mut()`.

## 2. Minimal hardware draw unit

```rust
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_gui::framebuffer::Framebuffer;
use embedded_gui::prelude::*;
use embedded_gui::render::{DrawTask, DrawUnit, Hardware2DAccelerator};

struct PxpDrawUnit<A> {
    accelerator: A,
}

impl<A: Hardware2DAccelerator, const N: usize> DrawUnit<Framebuffer<N>>
    for PxpDrawUnit<A>
{
    fn can_handle(&self, task: &DrawTask<'_>) -> bool {
        matches!(
            task,
            DrawTask::Fill {
                radius: 0,
                opacity: 255,
                ..
            }
        )
    }

    fn execute(
        &mut self,
        task: &DrawTask<'_>,
        target: &mut Framebuffer<N>,
    ) -> Result<(), core::convert::Infallible> {
        if let DrawTask::Fill { rect, color, .. } = task {
            self.accelerator.fill_rect(
                target.pixels_mut(),
                target.width() as usize,
                *rect,
                *color,
            );
        }
        Ok(())
    }
}
```

Then dispatch:

```rust
let mut pxp = PxpDrawUnit {
    accelerator: MyPxpDriver::new(),
};
let mut fallback = SoftwareDrawUnit;
let mut units: [&mut dyn DrawUnit<Framebuffer<{ 320 * 240 }>>; 1] = [&mut pxp];

dispatch_draw_tasks(&queue, &mut framebuffer, &mut units, &mut fallback)?;
```

The `DrawTaskQueue` is populated during UI traversal, so a hardware unit can
accelerate opaque fills, and the `SoftwareDrawUnit` fallback continues to
handle gradients, rounded corners, text, arcs, and shadows.

## 3. Wiring notes by silicon

| Engine | Typical integration |
|--------|---------------------|
| STM32 DMA2D / Chrom-ART | `fill_rect` → Register-to-Memory; `copy_rect` → Memory-to-Memory; `blend_rect` → PFC with alpha. Operates on internal SRAM/SDRAM framebuffers. |
| NXP PXP | `fill_rect` → PXP as a 2D filler; `copy_rect`/`blend_rect` through the AS background/blit path. |
| ESP32 (GDMA/LCD) | May accelerate memory copies and block transfers; check whether your specific SoC exposes a general-purpose 2D engine or only LCD DMA. |
| Raspberry Pi Pico / no engine | `Software2DAccelerator`; rely on `fill_solid`/`fill_contiguous` fast paths in the display driver. |

Always measure: on small panels, software rendering is often fast enough, and
the win from a blitter is reduced SPI traffic and freeing the CPU for other
tasks rather than a visible framerate jump.

## 4. Current status

- The `Hardware2DAccelerator` trait and software fallback are implemented.
- The render task queue has a first-class draw-unit dispatch path.
- Board-specific HAL adapters (e.g. an STM32 DMA2D or NXP PXP adapter crate)
  are intentionally kept out of the core crate, matching the project's
  dependency-free `no_std` design.

If you build an adapter for your MCU, the best home is a small companion crate
that depends on `embedded-gui` and your HAL.
