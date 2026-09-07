use embedded_graphics_core::pixelcolor::Rgb565;
#[cfg(feature = "embedded-graphics")]
use embedded_graphics_core::pixelcolor::RgbColor;

use crate::font::FontId;

pub const CHAR_WIDTH: u32 = 4;
pub const CHAR_HEIGHT: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrap {
    None,
    Character,
    Word,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOverflow {
    Clip,
    Ellipsis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EllipsisMode {
    ThreeDots,
    SingleGlyph,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOverflowPolicy {
    Global(TextOverflow),
    WrapThenEllipsis { max_lines: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextStyle {
    pub color: Rgb565,
    pub font: FontId,
    pub opacity: u8,
    pub align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub overflow_policy: TextOverflowPolicy,
    pub kerning: bool,
    pub max_lines: Option<u8>,
    pub ellipsis: EllipsisMode,
    pub line_spacing: u8,
}

impl TextStyle {
    pub const fn new(color: Rgb565) -> Self {
        Self {
            color,
            font: FontId::Tiny3x5,
            opacity: 255,
            align: TextAlign::Left,
            vertical_align: VerticalAlign::Top,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            overflow_policy: TextOverflowPolicy::Global(TextOverflow::Clip),
            kerning: false,
            max_lines: None,
            ellipsis: EllipsisMode::ThreeDots,
            line_spacing: 1,
        }
    }

    pub const fn centered(mut self) -> Self {
        self.align = TextAlign::Center;
        self.vertical_align = VerticalAlign::Middle;
        self
    }

    pub const fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub const fn with_vertical_align(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = align;
        self
    }

    pub const fn with_wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub const fn with_line_spacing(mut self, spacing: u8) -> Self {
        self.line_spacing = spacing;
        self
    }

    pub const fn with_overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self.overflow_policy = TextOverflowPolicy::Global(overflow);
        self
    }

    pub const fn with_kerning(mut self, kerning: bool) -> Self {
        self.kerning = kerning;
        self
    }

    pub const fn with_max_lines(mut self, max_lines: Option<u8>) -> Self {
        self.max_lines = max_lines;
        self
    }

    pub const fn with_ellipsis_mode(mut self, ellipsis: EllipsisMode) -> Self {
        self.ellipsis = ellipsis;
        self
    }

    pub const fn with_overflow_policy(mut self, policy: TextOverflowPolicy) -> Self {
        self.overflow_policy = policy;
        self
    }

    pub const fn with_opacity(mut self, opacity: u8) -> Self {
        self.opacity = opacity;
        self
    }

    pub const fn with_font_id(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    pub fn with_font(mut self, font: impl Into<FontId>) -> Self {
        self.font = font.into();
        self
    }
}

#[cfg(feature = "embedded-graphics")]
impl From<&embedded_graphics::mono_font::MonoTextStyle<'static, Rgb565>> for TextStyle {
    fn from(mono_style: &embedded_graphics::mono_font::MonoTextStyle<'static, Rgb565>) -> Self {
        let mut style = TextStyle::new(mono_style.text_color.unwrap_or(Rgb565::WHITE));
        style.font = FontId::MonoFont(mono_style.font);
        style
    }
}

#[cfg(feature = "embedded-graphics")]
impl From<embedded_graphics::mono_font::MonoTextStyle<'static, Rgb565>> for TextStyle {
    fn from(mono_style: embedded_graphics::mono_font::MonoTextStyle<'static, Rgb565>) -> Self {
        Self::from(&mono_style)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextMetrics {
    pub width: u32,
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::pixelcolor::RgbColor;

    #[test]
    fn text_style_builder_mutations() {
        let mut style = TextStyle::new(Rgb565::RED);
        style = style.centered();
        assert_eq!(style.align, TextAlign::Center);
        assert_eq!(style.vertical_align, VerticalAlign::Middle);

        style = style
            .with_align(TextAlign::Right)
            .with_vertical_align(VerticalAlign::Bottom)
            .with_wrap(TextWrap::Word)
            .with_line_spacing(2)
            .with_overflow(TextOverflow::Ellipsis)
            .with_kerning(true)
            .with_max_lines(Some(2))
            .with_ellipsis_mode(EllipsisMode::SingleGlyph)
            .with_overflow_policy(TextOverflowPolicy::WrapThenEllipsis { max_lines: 3 })
            .with_opacity(100)
            .with_font_id(FontId::Tiny3x5)
            .with_font(FontId::Tiny3x5);
        assert_eq!(style.align, TextAlign::Right);
        assert_eq!(style.vertical_align, VerticalAlign::Bottom);
        assert_eq!(style.wrap, TextWrap::Word);
        assert_eq!(style.line_spacing, 2);
        assert_eq!(style.overflow, TextOverflow::Ellipsis);
        assert!(style.kerning);
        assert_eq!(style.max_lines, Some(2));
        assert_eq!(style.ellipsis, EllipsisMode::SingleGlyph);
        assert_eq!(style.opacity, 100);
        assert_eq!(style.font, FontId::Tiny3x5);
    }

    #[cfg(feature = "embedded-graphics")]
    #[test]
    fn text_style_from_mono_style() {
        use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_4X6};
        let mono = MonoTextStyle::new(&FONT_4X6, Rgb565::GREEN);
        let style = TextStyle::from(&mono);
        assert_eq!(style.color, Rgb565::GREEN);
        assert!(matches!(style.font, FontId::MonoFont(_)));
        let style2 = TextStyle::from(mono);
        assert_eq!(style2.color, Rgb565::GREEN);
    }
}
