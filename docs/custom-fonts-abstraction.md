# Custom Font Abstraction & Interop Guide

`embedded-gui` provides a unified, extensible font system designed for microcontrollers and `no_std` environments. It supports built-in packed fonts, custom monospaced/raw bitmap arrays, procedural dynamic font providers, and drop-in interop with `embedded-graphics` `MonoFont`s.

---

## 1. Overview & Architecture

Font rendering in `embedded-gui` is abstracted through the [`FontId`](../src/font.rs) enum and the [`Font`](../src/font.rs) trait.

| Font Mechanism | Type / Variant | Use Case & Description |
|----------------|----------------|------------------------|
| **Built-in Packed Fonts** | `FontId::Tiny3x5`, `FontId::Medium4x7`, `FontId::Scaled6x10` | Default lightweight 3x5 and 4x7 bitmap fonts built into the crate. |
| **Custom Bitmap Arrays** | `FontId::Bitmap(&'static BitmapFont)` | Flexible monospaced raw bitmap fonts (8x8, 8x16, 12x16, 16x24, etc.) without requiring `embedded-graphics`. |
| **Dynamic Trait Providers** | `FontId::Dynamic(&'static dyn Font)` | Arbitrary procedural, vector, or anti-aliased font generators implementing the `Font` trait. |
| **embedded-graphics Interop** | `FontId::MonoFont(&'static MonoFont<'static>)` | Transparent drop-in interop with `embedded-graphics::mono_font::*` (`FONT_6X10`, `PROFONT_*`, `u8g2_fonts`, etc.). |

---

## 2. The `Font` Trait

The [`Font`](../src/font.rs) trait defines character geometry and glyph rendering:

```rust
pub trait Font: Send + Sync {
    /// Character horizontal advance in pixels.
    fn advance(&self) -> u32;

    /// Vertical line height in pixels.
    fn line_height(&self) -> u32;

    /// Render a single glyph by calling `draw_pixel(dx, dy)` for each active pixel
    /// in the glyph, where `(dx, dy)` are relative coordinates within the glyph bounding box.
    fn draw_glyph(&self, ch: char, draw_pixel: &mut dyn FnMut(i32, i32));
}
```

Any type that implements `Font` can be converted into a `FontId` via `FontId::from(...)` or `.into()`.

---

## 3. Custom Raw Bitmap Fonts (`BitmapFont`)

The `BitmapFont` struct allows consumers to define custom monospaced or raw bitmap arrays with arbitrary dimensions:

```rust
use embedded_gui::font::BitmapFont;

// Define glyph bitmap array (e.g. 8x16 glyphs, 16 bytes per glyph, MSB left-to-right)
static MY_8X16_GLYPHS: [u8; 16 * 95] = [ ... ];

// Create static BitmapFont instance
static MY_CUSTOM_FONT: BitmapFont = BitmapFont::new_8x16(
    32, // ASCII code of first glyph (e.g. 32 / space)
    8,  // Character advance (pixels)
    16, // Line height (pixels)
    &MY_8X16_GLYPHS,
);

// Use in TextStyle or Widget styling
let font_id = FontId::from(&MY_CUSTOM_FONT);
let style = TextStyle::new(Rgb565::WHITE).with_font(font_id);
```

### Hardware Acceleration & Span Optimization

`BitmapFont` implements `draw_glyph_to` using `GlyphOp::Span`. When rendering to identity-transformed framebuffers, contiguous horizontal bit spans are automatically batch-drawn using `fill_rect`, drastically reducing pixel dispatch loops over SPI/I2C display controllers.

---

## 4. Procedural & Dynamic Font Providers

For procedural fonts, vector font decoders, or third-party font parsers, implement `Font` directly:

```rust
use embedded_gui::font::{Font, FontId};

struct ProceduralRingFont;

impl Font for ProceduralRingFont {
    fn advance(&self) -> u32 {
        12
    }
    fn line_height(&self) -> u32 {
        16
    }
    fn draw_glyph(&self, _ch: char, draw_pixel: &mut dyn FnMut(i32, i32)) {
        // Custom pixel generation logic
        for x in 0..10 {
            draw_pixel(x, 0);
            draw_pixel(x, 15);
        }
    }
}

static MY_PROCEDURAL_FONT: ProceduralRingFont = ProceduralRingFont;

// Pass trait reference to FontId
let font_id = FontId::from(&MY_PROCEDURAL_FONT as &'static dyn Font);
```

---

## 5. Drop-In `embedded-graphics` `MonoFont` Interop

When the `embedded-graphics` feature is enabled (default), standard monospaced fonts from `embedded-graphics` or crates like `u8g2_fonts`, `bdf`, or `profont` can be converted directly into `FontId`:

```rust
use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_9X15};
use embedded_gui::prelude::*;

// Use directly with with_font or FontId::from
let font1 = FontId::from(&FONT_6X10);
let font2 = FontId::from(&FONT_9X15);

let style = TextStyle::new(Rgb565::YELLOW).with_font(font1);
```

---

## 6. Runnable Showcase Example

A complete runnable showcase is provided in [`examples/basics/custom_font_showcase.rs`](../examples/basics/custom_font_showcase.rs):

```bash
cargo run --example custom_font_showcase --features std
```
