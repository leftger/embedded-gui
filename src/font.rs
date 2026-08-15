include!(concat!(env!("OUT_DIR"), "/generated_ascii_3x5.rs"));

/// Trait for custom font providers.
///
/// Consumers can implement this trait on custom font types (e.g. anti-aliased fonts,
/// vector text generators, external BDF/PSF font decoders, TTF parsers) and use
/// [`FontId::Dynamic`] or [`FontId::from`] to pass them to text styling.
pub trait Font: Send + Sync {
    /// Character horizontal advance in pixels.
    fn advance(&self) -> u32;

    /// Vertical line height in pixels.
    fn line_height(&self) -> u32;

    /// Render a single glyph by calling `draw_pixel(dx, dy)` for each active pixel
    /// in the glyph, where `(dx, dy)` are relative coordinates within the glyph bounding box.
    fn draw_glyph(&self, ch: char, draw_pixel: &mut dyn FnMut(i32, i32));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedFont {
    pub first_char: u8,
    pub advance: u8,
    pub line_height: u8,
    pub glyphs: &'static [[u8; 5]],
}

impl Font for PackedFont {
    fn advance(&self) -> u32 {
        self.advance as u32
    }

    fn line_height(&self) -> u32 {
        self.line_height as u32
    }

    fn draw_glyph(&self, ch: char, draw_pixel: &mut dyn FnMut(i32, i32)) {
        let code = ch as u32;
        let rows = if code >= self.first_char as u32 {
            let idx = (code as usize).saturating_sub(self.first_char as usize);
            self.glyphs.get(idx).copied().unwrap_or([0; 5])
        } else {
            [0; 5]
        };
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    draw_pixel(col, row as i32);
                }
            }
        }
    }
}

pub static ASCII_3X5_FONT: PackedFont = PackedFont {
    first_char: 32,
    advance: 4,
    line_height: 6,
    glyphs: &ASCII_3X5_GLYPHS,
};

pub static ASCII_4X7_FONT: PackedFont = PackedFont {
    first_char: 32,
    advance: 5,
    line_height: 8,
    glyphs: &ASCII_4X7_GLYPHS,
};

/// Bounding-box rendering operation for [`BitmapFont`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphOp {
    /// Draw a single pixel at relative offset `(dx, dy)`.
    Pixel(i32, i32),
    /// Draw a contiguous horizontal span of `len` pixels starting at `(dx, dy)`.
    Span(i32, i32, u32),
}

/// Flexible monospaced or packed raw bitmap font definition.
///
/// Unlike [`PackedFont`] (which is fixed to 3x5 5-row bitpacks), [`BitmapFont`]
/// supports arbitrary glyph dimensions (e.g., 8x8, 8x16, 12x16, 16x24),
/// configurable advance/line-height, and multi-byte row bit-masks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitmapFont {
    /// Bounding box width in pixels.
    pub width: u8,
    /// Bounding box height in pixels.
    pub height: u8,
    /// Character horizontal advance in pixels.
    pub advance: u8,
    /// Vertical line height in pixels.
    pub line_height: u8,
    /// The ASCII character code of the first glyph in the buffer (usually 32 / space).
    pub first_char: u8,
    /// Number of bytes per row for each glyph (e.g. 1 byte for width <= 8, 2 bytes for width <= 16).
    pub bytes_per_row: u8,
    /// Contiguous byte slice containing glyph bitmaps stored row by row (MSB left-to-right).
    pub glyphs: &'static [u8],
}

impl BitmapFont {
    /// Creates a new `BitmapFont` for standard 8x8 glyphs (1 byte per row, 8 rows per glyph).
    pub const fn new_8x8(
        first_char: u8,
        advance: u8,
        line_height: u8,
        glyphs: &'static [u8],
    ) -> Self {
        Self {
            width: 8,
            height: 8,
            advance,
            line_height,
            first_char,
            bytes_per_row: 1,
            glyphs,
        }
    }

    /// Creates a new `BitmapFont` for standard 8x16 glyphs (1 byte per row, 16 rows per glyph).
    pub const fn new_8x16(
        first_char: u8,
        advance: u8,
        line_height: u8,
        glyphs: &'static [u8],
    ) -> Self {
        Self {
            width: 8,
            height: 16,
            advance,
            line_height,
            first_char,
            bytes_per_row: 1,
            glyphs,
        }
    }

    /// Get the raw byte slice for a character's row data.
    pub fn glyph_bytes(&self, ch: char) -> Option<&'static [u8]> {
        let code = ch as u32;
        if code < self.first_char as u32 {
            return None;
        }
        let idx = (code - self.first_char as u32) as usize;
        let bytes_per_glyph = self.height as usize * self.bytes_per_row as usize;
        let start = idx * bytes_per_glyph;
        let end = start + bytes_per_glyph;
        if end <= self.glyphs.len() {
            Some(&self.glyphs[start..end])
        } else {
            None
        }
    }

    /// Renders a glyph by emitting [`GlyphOp`] commands to a single closure callback.
    pub fn draw_glyph_to<F>(&self, ch: char, mut emit: F)
    where
        F: FnMut(GlyphOp),
    {
        if let Some(data) = self.glyph_bytes(ch) {
            let bpr = self.bytes_per_row as usize;
            for row in 0..(self.height as usize) {
                let row_data = &data[row * bpr..(row + 1) * bpr];
                let mut span_start: Option<usize> = None;
                let mut span_len = 0u32;

                for col in 0..(self.width as usize) {
                    let byte_idx = col / 8;
                    let bit_idx = 7 - (col % 8);
                    let is_set =
                        byte_idx < row_data.len() && (row_data[byte_idx] & (1 << bit_idx)) != 0;

                    if is_set {
                        if span_start.is_none() {
                            span_start = Some(col);
                            span_len = 1;
                        } else {
                            span_len += 1;
                        }
                    } else if let Some(start) = span_start {
                        if span_len == 1 {
                            emit(GlyphOp::Pixel(start as i32, row as i32));
                        } else {
                            emit(GlyphOp::Span(start as i32, row as i32, span_len));
                        }
                        span_start = None;
                        span_len = 0;
                    }
                }
                if let Some(start) = span_start {
                    if span_len == 1 {
                        emit(GlyphOp::Pixel(start as i32, row as i32));
                    } else {
                        emit(GlyphOp::Span(start as i32, row as i32, span_len));
                    }
                }
            }
        }
    }
}

impl Font for BitmapFont {
    fn advance(&self) -> u32 {
        self.advance as u32
    }

    fn line_height(&self) -> u32 {
        self.line_height as u32
    }

    fn draw_glyph(&self, ch: char, draw_pixel: &mut dyn FnMut(i32, i32)) {
        self.draw_glyph_to(ch, |op| match op {
            GlyphOp::Pixel(dx, dy) => draw_pixel(dx, dy),
            GlyphOp::Span(dx, dy, len) => {
                for col in 0..len {
                    draw_pixel(dx + col as i32, dy);
                }
            }
        });
    }
}

#[cfg(feature = "embedded-graphics")]
impl Font for embedded_graphics::mono_font::MonoFont<'static> {
    fn advance(&self) -> u32 {
        self.character_size.width + self.character_spacing
    }

    fn line_height(&self) -> u32 {
        self.character_size.height
    }

    fn draw_glyph(&self, ch: char, draw_pixel: &mut dyn FnMut(i32, i32)) {
        use embedded_graphics::Drawable;
        use embedded_graphics::draw_target::DrawTarget;
        use embedded_graphics::geometry::{OriginDimensions, Point, Size};
        use embedded_graphics::mono_font::MonoTextStyle;
        use embedded_graphics::pixelcolor::BinaryColor;
        use embedded_graphics::text::Text;

        struct Collector<'a> {
            f: &'a mut dyn FnMut(i32, i32),
        }

        impl OriginDimensions for Collector<'_> {
            fn size(&self) -> Size {
                Size::new(u32::MAX, u32::MAX)
            }
        }

        impl DrawTarget for Collector<'_> {
            type Color = BinaryColor;
            type Error = core::convert::Infallible;

            fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
            where
                I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
            {
                for embedded_graphics::Pixel(pos, color) in pixels {
                    if color.is_on() {
                        (self.f)(pos.x, pos.y);
                    }
                }
                Ok(())
            }
        }

        let text_style = MonoTextStyle::new(self, BinaryColor::On);
        let mut buf = [0u8; 4];
        let ch_str = ch.encode_utf8(&mut buf);
        let mut collector = Collector { f: draw_pixel };
        let _ = Text::new(ch_str, Point::zero(), text_style).draw(&mut collector);
    }
}

#[derive(Clone, Copy)]
pub enum FontId {
    Tiny3x5,
    Medium4x7,
    Scaled6x10,
    Vector(u8),
    Custom(&'static PackedFont),
    Bitmap(&'static BitmapFont),
    Dynamic(&'static dyn Font),
    #[cfg(feature = "embedded-graphics")]
    MonoFont(&'static embedded_graphics::mono_font::MonoFont<'static>),
}

impl core::fmt::Debug for FontId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Tiny3x5 => write!(f, "Tiny3x5"),
            Self::Medium4x7 => write!(f, "Medium4x7"),
            Self::Scaled6x10 => write!(f, "Scaled6x10"),
            Self::Vector(scale) => f.debug_tuple("Vector").field(scale).finish(),
            Self::Custom(font) => f.debug_tuple("Custom").field(font).finish(),
            Self::Bitmap(font) => f.debug_tuple("Bitmap").field(font).finish(),
            Self::Dynamic(_) => f.write_str("Dynamic"),
            #[cfg(feature = "embedded-graphics")]
            Self::MonoFont(font) => f.debug_tuple("MonoFont").field(font).finish(),
        }
    }
}

impl PartialEq for FontId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tiny3x5, Self::Tiny3x5) => true,
            (Self::Medium4x7, Self::Medium4x7) => true,
            (Self::Scaled6x10, Self::Scaled6x10) => true,
            (Self::Vector(a), Self::Vector(b)) => a == b,
            (Self::Custom(a), Self::Custom(b)) => core::ptr::eq(*a, *b),
            (Self::Bitmap(a), Self::Bitmap(b)) => core::ptr::eq(*a, *b),
            (Self::Dynamic(a), Self::Dynamic(b)) => core::ptr::eq(
                *a as *const dyn Font as *const (),
                *b as *const dyn Font as *const (),
            ),
            #[cfg(feature = "embedded-graphics")]
            (Self::MonoFont(a), Self::MonoFont(b)) => core::ptr::eq(*a, *b),
            _ => false,
        }
    }
}

impl Eq for FontId {}

impl FontId {
    pub fn advance(self) -> u32 {
        match self {
            Self::Tiny3x5 => 4,
            Self::Medium4x7 => 5,
            Self::Scaled6x10 => 7,
            Self::Vector(scale) => (8 * scale) as u32,
            Self::Custom(font) => font.advance as u32,
            Self::Bitmap(font) => font.advance as u32,
            Self::Dynamic(font) => font.advance(),
            #[cfg(feature = "embedded-graphics")]
            Self::MonoFont(font) => font.character_size.width + font.character_spacing,
        }
    }

    pub fn line_height(self) -> u32 {
        match self {
            Self::Tiny3x5 => 6,
            Self::Medium4x7 => 8,
            Self::Scaled6x10 => 11,
            Self::Vector(scale) => (12 * scale) as u32,
            Self::Custom(font) => font.line_height as u32,
            Self::Bitmap(font) => font.line_height as u32,
            Self::Dynamic(font) => font.line_height(),
            #[cfg(feature = "embedded-graphics")]
            Self::MonoFont(font) => font.character_size.height,
        }
    }
}

pub const fn packed_font(font: FontId) -> &'static PackedFont {
    match font {
        FontId::Tiny3x5 => &ASCII_3X5_FONT,
        FontId::Medium4x7 => &ASCII_4X7_FONT,
        FontId::Scaled6x10 => &ASCII_3X5_FONT,
        FontId::Vector(_) => &ASCII_3X5_FONT,
        FontId::Custom(font) => font,
        FontId::Bitmap(_) => &ASCII_3X5_FONT,
        FontId::Dynamic(_) => &ASCII_3X5_FONT,
        #[cfg(feature = "embedded-graphics")]
        FontId::MonoFont(_) => &ASCII_3X5_FONT,
    }
}

impl From<&'static PackedFont> for FontId {
    fn from(font: &'static PackedFont) -> Self {
        FontId::Custom(font)
    }
}

impl From<&'static BitmapFont> for FontId {
    fn from(font: &'static BitmapFont) -> Self {
        FontId::Bitmap(font)
    }
}

impl From<&'static dyn Font> for FontId {
    fn from(font: &'static dyn Font) -> Self {
        FontId::Dynamic(font)
    }
}

#[cfg(feature = "embedded-graphics")]
impl From<&'static embedded_graphics::mono_font::MonoFont<'static>> for FontId {
    fn from(font: &'static embedded_graphics::mono_font::MonoFont<'static>) -> Self {
        FontId::MonoFont(font)
    }
}

pub fn get_vector_glyph(ch: char) -> &'static [(u8, u8)] {
    match ch {
        ' ' => &[],
        '0' => &[
            (2, 0),
            (6, 0),
            (6, 10),
            (2, 10),
            (2, 0),
            (0xFF, 0xFF),
            (2, 10),
            (6, 0),
        ],
        '1' => &[
            (4, 0),
            (4, 10),
            (0xFF, 0xFF),
            (2, 2),
            (4, 0),
            (0xFF, 0xFF),
            (2, 10),
            (6, 10),
        ],
        '2' => &[(2, 0), (6, 0), (6, 5), (2, 5), (2, 10), (6, 10)],
        '3' => &[
            (2, 0),
            (6, 0),
            (6, 10),
            (2, 10),
            (0xFF, 0xFF),
            (2, 5),
            (6, 5),
        ],
        '4' => &[(2, 0), (2, 5), (6, 5), (0xFF, 0xFF), (6, 0), (6, 10)],
        '5' => &[(6, 0), (2, 0), (2, 5), (6, 5), (6, 10), (2, 10)],
        '6' => &[(6, 0), (2, 0), (2, 10), (6, 10), (6, 5), (2, 5)],
        '7' => &[(2, 0), (6, 0), (2, 10)],
        '8' => &[
            (2, 0),
            (6, 0),
            (6, 10),
            (2, 10),
            (2, 0),
            (0xFF, 0xFF),
            (2, 5),
            (6, 5),
        ],
        '9' => &[(6, 5), (2, 5), (2, 0), (6, 0), (6, 10), (2, 10)],
        'A' | 'a' => &[(2, 10), (4, 0), (6, 10), (0xFF, 0xFF), (3, 5), (5, 5)],
        'B' | 'b' => &[
            (2, 0),
            (5, 0),
            (6, 2),
            (6, 4),
            (5, 5),
            (2, 5),
            (5, 5),
            (6, 6),
            (6, 8),
            (5, 10),
            (2, 10),
            (2, 0),
        ],
        'C' | 'c' => &[(6, 0), (2, 0), (2, 10), (6, 10)],
        'D' | 'd' => &[(2, 0), (5, 0), (6, 3), (6, 7), (5, 10), (2, 10), (2, 0)],
        'E' | 'e' => &[
            (6, 0),
            (2, 0),
            (2, 10),
            (6, 10),
            (0xFF, 0xFF),
            (2, 5),
            (5, 5),
        ],
        'F' | 'f' => &[(6, 0), (2, 0), (2, 10), (0xFF, 0xFF), (2, 5), (5, 5)],
        'G' | 'g' => &[(6, 2), (6, 0), (2, 0), (2, 10), (6, 10), (6, 5), (4, 5)],
        'H' | 'h' => &[
            (2, 0),
            (2, 10),
            (0xFF, 0xFF),
            (6, 0),
            (6, 10),
            (0xFF, 0xFF),
            (2, 5),
            (6, 5),
        ],
        'I' | 'i' => &[
            (4, 0),
            (4, 10),
            (0xFF, 0xFF),
            (2, 0),
            (6, 0),
            (0xFF, 0xFF),
            (2, 10),
            (6, 10),
        ],
        'J' | 'j' => &[(6, 0), (6, 8), (4, 10), (2, 8)],
        'K' | 'k' => &[(2, 0), (2, 10), (0xFF, 0xFF), (6, 0), (2, 5), (6, 10)],
        'L' | 'l' => &[(2, 0), (2, 10), (6, 10)],
        'M' | 'm' => &[(2, 10), (2, 0), (4, 5), (6, 0), (6, 10)],
        'N' | 'n' => &[(2, 10), (2, 0), (6, 10), (6, 0)],
        'O' | 'o' => &[(2, 0), (6, 0), (6, 10), (2, 10), (2, 0)],
        'P' | 'p' => &[(2, 10), (2, 0), (6, 0), (6, 5), (2, 5)],
        'Q' | 'q' => &[
            (2, 0),
            (6, 0),
            (6, 10),
            (2, 10),
            (2, 0),
            (0xFF, 0xFF),
            (4, 7),
            (7, 10),
        ],
        'R' | 'r' => &[
            (2, 10),
            (2, 0),
            (6, 0),
            (6, 5),
            (2, 5),
            (0xFF, 0xFF),
            (4, 5),
            (6, 10),
        ],
        'S' | 's' => &[(6, 0), (2, 0), (2, 5), (6, 5), (6, 10), (2, 10)],
        'T' | 't' => &[(2, 0), (6, 0), (0xFF, 0xFF), (4, 0), (4, 10)],
        'U' | 'u' => &[(2, 0), (2, 10), (6, 10), (6, 0)],
        'V' | 'v' => &[(2, 0), (4, 10), (6, 0)],
        'W' | 'w' => &[(2, 0), (2, 10), (4, 5), (6, 10), (6, 0)],
        'X' | 'x' => &[(2, 0), (6, 10), (0xFF, 0xFF), (6, 0), (2, 10)],
        'Y' | 'y' => &[(2, 0), (4, 5), (6, 0), (0xFF, 0xFF), (4, 5), (4, 10)],
        'Z' | 'z' => &[(2, 0), (6, 0), (2, 10), (6, 10)],
        '-' => &[(2, 5), (6, 5)],
        '+' => &[(2, 5), (6, 5), (0xFF, 0xFF), (4, 2), (4, 8)],
        '.' => &[(4, 9), (4, 10)],
        ':' => &[(4, 2), (4, 3), (0xFF, 0xFF), (4, 7), (4, 8)],
        '/' => &[(2, 10), (6, 0)],
        _ => &[(2, 0), (6, 0), (6, 10), (2, 10), (2, 0), (2, 0), (6, 10)],
    }
}

pub fn glyph_rows(font: FontId, ch: char) -> [u8; 5] {
    let packed = packed_font(font);
    let code = ch as u32;
    if code >= packed.first_char as u32 {
        let idx = (code as usize).saturating_sub(packed.first_char as usize);
        if idx < packed.glyphs.len() {
            return packed.glyphs[idx];
        }
    }
    let fallback = b'?'.saturating_sub(packed.first_char) as usize;
    packed.glyphs.get(fallback).copied().unwrap_or([0; 5])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    static MY_CUSTOM_GLYPHS: [[u8; 5]; 2] = [
        [0b111, 0b101, 0b111, 0b101, 0b101], // Space / 'A'
        [0b111, 0b111, 0b111, 0b111, 0b111],
    ];

    static MY_CUSTOM_FONT: PackedFont = PackedFont {
        first_char: 32,
        advance: 8,
        line_height: 12,
        glyphs: &MY_CUSTOM_GLYPHS,
    };

    static MY_BITMAP_GLYPHS: [u8; 16] = [
        // 'A' (8x16)
        0b00111100, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b01100110,
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
        0b00000000, 0b00000000,
    ];

    static MY_BITMAP_FONT: BitmapFont =
        BitmapFont::new_8x16(65 /* 'A' */, 8, 16, &MY_BITMAP_GLYPHS);

    struct CustomFontImpl;
    impl Font for CustomFontImpl {
        fn advance(&self) -> u32 {
            10
        }
        fn line_height(&self) -> u32 {
            14
        }
        fn draw_glyph(&self, _ch: char, draw_pixel: &mut dyn FnMut(i32, i32)) {
            draw_pixel(0, 0);
            draw_pixel(1, 1);
        }
    }

    static DYN_FONT_INSTANCE: CustomFontImpl = CustomFontImpl;

    #[test]
    fn test_custom_font_id() {
        let font_id = FontId::Custom(&MY_CUSTOM_FONT);
        assert_eq!(font_id.advance(), 8);
        assert_eq!(font_id.line_height(), 12);
        assert_eq!(packed_font(font_id).first_char, 32);
        assert_eq!(
            glyph_rows(font_id, ' '),
            [0b111, 0b101, 0b111, 0b101, 0b101]
        );
    }

    #[test]
    fn test_bitmap_font() {
        let font_id = FontId::from(&MY_BITMAP_FONT);
        assert_eq!(font_id.advance(), 8);
        assert_eq!(font_id.line_height(), 16);

        let mut pixels = Vec::new();
        MY_BITMAP_FONT.draw_glyph('A', &mut |x, y| pixels.push((x, y)));
        assert!(!pixels.is_empty());
        assert!(pixels.contains(&(2, 0))); // 0b00111100 has bit at col 2
    }

    #[test]
    fn test_dynamic_font() {
        let font_id = FontId::from(&DYN_FONT_INSTANCE as &'static dyn Font);
        assert_eq!(font_id.advance(), 10);
        assert_eq!(font_id.line_height(), 14);
    }
}
