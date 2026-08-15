//! Timeline Connective RelBar and Peek Layer widgets.
//!
//! Provides `TimelineNodeWidget` (relationship connector bar between chronological pins)
//! and `PeekBannerWidget` (reactive peek overlay for heads-up alerts and upcoming events).

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

use crate::{
    geometry::Rect,
    render::{RenderCtx, StrokeStyle},
    round::UnobstructedArea,
    style::Style,
    widget::{PropertyKey, PropertyValue, Widget},
};

/// Temporal state of a timeline pin node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineNodeState {
    Past,
    ActiveNow,
    Upcoming,
    Future,
}

/// A vertical relationship connector bar and node dot linking chronological events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineNodeWidget {
    pub state: TimelineNodeState,
    pub node_radius: u8,
    pub bar_width: u8,
    pub has_top_bar: bool,
    pub has_bottom_bar: bool,
    pub active_color: Rgb565,
    pub inactive_color: Rgb565,
    pub line_color: Rgb565,
}

impl TimelineNodeWidget {
    pub const fn new(state: TimelineNodeState) -> Self {
        Self {
            state,
            node_radius: 4,
            bar_width: 2,
            has_top_bar: true,
            has_bottom_bar: true,
            active_color: Rgb565::new(31, 40, 0), // Amber / Gold
            inactive_color: Rgb565::new(10, 20, 10),
            line_color: Rgb565::new(8, 16, 8),
        }
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        let cx = bounds.x + (bounds.w as i32 / 2);
        let cy = bounds.y + (bounds.h as i32 / 2);
        let r = self.node_radius as u32;

        // Top connector line
        if self.has_top_bar {
            let stroke = StrokeStyle::new(self.line_color).with_width(self.bar_width);
            ctx.draw_line_styled(cx, bounds.y, cx, cy - r as i32, stroke)?;
        }

        // Bottom connector line
        if self.has_bottom_bar {
            let stroke = StrokeStyle::new(self.line_color).with_width(self.bar_width);
            ctx.draw_line_styled(cx, cy + r as i32, cx, bounds.bottom(), stroke)?;
        }

        // Center Node Dot
        let (dot_color, dot_r) = match self.state {
            TimelineNodeState::ActiveNow => (self.active_color, r + 1),
            TimelineNodeState::Upcoming => (self.active_color, r),
            TimelineNodeState::Past | TimelineNodeState::Future => (self.inactive_color, r),
        };

        ctx.fill_circle(cx, cy, dot_r, dot_color)?;
        if matches!(self.state, TimelineNodeState::ActiveNow) {
            ctx.stroke_circle(cx, cy, dot_r + 2, Rgb565::new(31, 63, 31))?;
        }

        Ok(())
    }
}

impl Widget for TimelineNodeWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Selected => Some(PropertyValue::Int(self.state as i32)),
            _ => None,
        }
    }
}

/// Reactive Peek banner ribbon for upcoming pins, alerts, or status hints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PeekBannerWidget<'a> {
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub icon_char: Option<char>,
    pub height: u16,
    pub is_expanded: bool,
    pub background_color: Rgb565,
    pub text_color: Rgb565,
    pub accent_color: Rgb565,
}

impl<'a> PeekBannerWidget<'a> {
    pub const fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            icon_char: None,
            height: 28,
            is_expanded: false,
            background_color: Rgb565::new(4, 8, 12),
            text_color: Rgb565::new(31, 63, 31),
            accent_color: Rgb565::new(0, 45, 25), // Teal / Emerald
        }
    }

    /// Adapts an [`UnobstructedArea`] to account for this peek banner.
    pub fn apply_to_unobstructed_area(&self, area: &mut UnobstructedArea) {
        let h = if self.is_expanded {
            self.height * 2
        } else {
            self.height
        };
        area.inset_top = h;
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        // Background banner card
        ctx.fill_rounded_rect(bounds, 4, self.background_color)?;

        // Left accent indicator stripe
        let stripe = Rect::new(bounds.x, bounds.y, 4, bounds.h);
        ctx.fill_rounded_rect(stripe, 2, self.accent_color)?;

        // Title and icon
        let text_x = bounds.x + 10;
        let text_y = bounds.y + (bounds.h as i32 / 2) - 4;
        ctx.draw_text(text_x, text_y, self.title, self.text_color)?;

        if let Some(sub) = self.subtitle {
            if self.is_expanded {
                ctx.draw_text(text_x, text_y + 12, sub, Rgb565::new(20, 40, 20))?;
            }
        }

        Ok(())
    }
}

impl<'a> Widget for PeekBannerWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Text => Some(PropertyValue::Str(self.title)),
            PropertyKey::Expanded => Some(PropertyValue::Bool(self.is_expanded)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_timeline_node_render() {
        let node = TimelineNodeWidget::new(TimelineNodeState::ActiveNow);
        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        assert!(node.render(&mut ctx, Rect::new(0, 0, 20, 20)).is_ok());
    }

    #[test]
    fn test_peek_banner_render() {
        let peek = PeekBannerWidget::new("Meeting in 10m");
        let mut fb = Framebuffer::<1200>::new(100, 12);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 100, 12));
        assert!(peek.render(&mut ctx, Rect::new(0, 0, 100, 12)).is_ok());
    }
}
