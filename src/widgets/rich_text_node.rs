//! Multi-Span Rich Text Flow Nodes.
//!
//! Provides `RichTextNodeWidget` and `TextSpan` for rendering structured multi-span
//! text blocks with inline badges and styled tags.

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};
use heapless::Vec;

use crate::{
    geometry::Rect,
    render::RenderCtx,
    style::Style,
    widget::{PropertyKey, PropertyValue, Widget},
};

/// Error indicating rich text node span capacity exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RichTextError;

/// An individual styled text span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextSpan<'a> {
    pub text: &'a str,
    pub color: Rgb565,
    pub background: Option<Rgb565>,
}

impl<'a> TextSpan<'a> {
    pub const fn plain(text: &'a str, color: Rgb565) -> Self {
        Self {
            text,
            color,
            background: None,
        }
    }

    pub const fn badge(text: &'a str, color: Rgb565, bg: Rgb565) -> Self {
        Self {
            text,
            color,
            background: Some(bg),
        }
    }
}

/// Multi-Span Rich Text Node widget supporting inline tag badges and colored text flows.
#[derive(Clone, Debug, PartialEq)]
pub struct RichTextNodeWidget<'a, const MAX_SPANS: usize = 6> {
    pub spans: Vec<TextSpan<'a>, MAX_SPANS>,
    pub line_spacing: u8,
}

impl<'a, const MAX_SPANS: usize> RichTextNodeWidget<'a, MAX_SPANS> {
    pub const fn new() -> Self {
        Self {
            spans: Vec::new(),
            line_spacing: 4,
        }
    }

    pub fn push_span(&mut self, span: TextSpan<'a>) -> Result<(), RichTextError> {
        self.spans.push(span).map_err(|_| RichTextError)
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        let mut cursor_x = bounds.x;
        let mut cursor_y = bounds.y;
        let font_w = 6;
        let font_h = 10;

        for span in &self.spans {
            let span_len = span.text.len() as i32;
            let span_px_w = span_len * font_w;

            // Simple line wrap if exceeding bounds width
            if cursor_x > bounds.x && cursor_x + span_px_w > bounds.right() {
                cursor_x = bounds.x;
                cursor_y += font_h + self.line_spacing as i32;
            }

            if let Some(bg) = span.background {
                let badge_rect = Rect::new(
                    cursor_x - 2,
                    cursor_y - 1,
                    span_px_w as u32 + 4,
                    font_h as u32 + 2,
                );
                ctx.fill_rounded_rect(badge_rect, 2, bg)?;
            }

            ctx.draw_text(cursor_x, cursor_y, span.text, span.color)?;
            cursor_x += span_px_w + 6;
        }

        Ok(())
    }
}

impl<'a, const MAX_SPANS: usize> Default for RichTextNodeWidget<'a, MAX_SPANS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const MAX_SPANS: usize> Widget for RichTextNodeWidget<'a, MAX_SPANS> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, _key: PropertyKey) -> Option<PropertyValue<'_>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_rich_text_node_render() {
        let mut node = RichTextNodeWidget::<4>::new();
        assert!(
            node.push_span(TextSpan::badge(
                "ALERT",
                Rgb565::new(31, 63, 31),
                Rgb565::new(31, 0, 0),
            ))
            .is_ok()
        );
        assert!(
            node.push_span(TextSpan::plain(
                "Sensor reading nominal",
                Rgb565::new(31, 63, 31),
            ))
            .is_ok()
        );

        let mut fb = Framebuffer::<24000>::new(200, 40);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 200, 40));
        assert!(node.render(&mut ctx, Rect::new(0, 0, 200, 40)).is_ok());
    }
}
