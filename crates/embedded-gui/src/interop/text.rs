//! Adapter for the `embedded-text` crate's `TextBox`.
//!
//! embedded-gui's own [`crate::text`] module (`Span`/`Line`/`Text`) stays the
//! default text path, rendered with [`crate::font`]'s no_std bitmap fonts.
//! This adapter is for content that needs `embedded-text`'s `TextBox`
//! instead: wrapping/justification/plugins (e.g. ANSI colors) driven by an
//! `embedded_graphics::mono_font::MonoFont`. Build a `TextBox` over an
//! embedded-gui [`Rect`] with [`text_box`] or [`text_box_with_style`], then
//! draw it during a widget's render pass with
//! [`crate::render::RenderCtx::draw_embedded_graphics`].
//!
//! ```
//! use embedded_gui::geometry::Rect;
//! use embedded_gui::interop::text::text_box;
//! use embedded_graphics::{
//!     mono_font::{ascii::FONT_6X10, MonoTextStyle},
//!     pixelcolor::{Rgb565, RgbColor},
//! };
//!
//! let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
//! let area = Rect::new(0, 0, 96, 32);
//! let text_box = text_box("Hello, embedded-text!", area, character_style);
//! assert_eq!(text_box.bounds.size.width, 96);
//! ```

use embedded_graphics::text::renderer::{CharacterStyle, TextRenderer};
use embedded_text::{TextBox, style::TextBoxStyle};

use crate::geometry::Rect;

/// Builds a default-styled `TextBox` over an embedded-gui [`Rect`].
pub fn text_box<'a, S>(content: &'a str, area: Rect, character_style: S) -> TextBox<'a, S>
where
    S: TextRenderer + CharacterStyle,
{
    TextBox::new(content, area.into(), character_style)
}

/// Builds a `TextBox` over an embedded-gui [`Rect`] with an explicit
/// `TextBoxStyle` (alignment, wrapping, line/paragraph spacing, ...).
pub fn text_box_with_style<'a, S>(
    content: &'a str,
    area: Rect,
    character_style: S,
    style: TextBoxStyle,
) -> TextBox<'a, S>
where
    S: TextRenderer + CharacterStyle,
{
    TextBox::with_textbox_style(content, area.into(), character_style, style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::{
        mono_font::{MonoTextStyle, ascii::FONT_6X10},
        pixelcolor::{Rgb565, RgbColor},
    };

    #[test]
    fn builds_text_box_over_rect() {
        let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        let area = Rect::new(2, 3, 96, 32);
        let text_box = text_box("Hello", area, character_style);
        assert_eq!(text_box.bounds.top_left.x, 2);
        assert_eq!(text_box.bounds.top_left.y, 3);
        assert_eq!(text_box.bounds.size.width, 96);
        assert_eq!(text_box.bounds.size.height, 32);
    }
}
