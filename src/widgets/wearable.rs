//! Wearable and compact UI widgets and interaction controls.
//!
//! Includes `ContentIndicator`, `CrumbsIndicator`, `SelectionWidget`, and `ActionBar`.

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

use crate::{
    geometry::Rect,
    render::RenderCtx,
    style::Style,
    widget::{PropertyKey, PropertyValue, Widget},
};

/// Direction for content overflow indicator arrows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentIndicatorDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Content indicator arrow widget showing off-screen scrollable content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContentIndicatorWidget {
    pub direction: ContentIndicatorDirection,
    pub visible: bool,
    pub color: Rgb565,
    pub pulse_progress: f32,
}

impl ContentIndicatorWidget {
    pub const fn new(direction: ContentIndicatorDirection) -> Self {
        Self {
            direction,
            visible: true,
            color: Rgb565::new(31, 63, 31),
            pulse_progress: 1.0,
        }
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        if !self.visible || bounds.w == 0 || bounds.h == 0 {
            return Ok(());
        }

        let cx = bounds.x + (bounds.w as i32 / 2);
        let cy = bounds.y + (bounds.h as i32 / 2);
        let s = (bounds.w.min(bounds.h) as i32 / 3).max(2);

        // Draw chevron arrow
        match self.direction {
            ContentIndicatorDirection::Up => {
                ctx.draw_line(cx - s, cy + s / 2, cx, cy - s / 2, self.color)?;
                ctx.draw_line(cx, cy - s / 2, cx + s, cy + s / 2, self.color)?;
            }
            ContentIndicatorDirection::Down => {
                ctx.draw_line(cx - s, cy - s / 2, cx, cy + s / 2, self.color)?;
                ctx.draw_line(cx, cy + s / 2, cx + s, cy - s / 2, self.color)?;
            }
            ContentIndicatorDirection::Left => {
                ctx.draw_line(cx + s / 2, cy - s, cx - s / 2, cy, self.color)?;
                ctx.draw_line(cx - s / 2, cy, cx + s / 2, cy + s, self.color)?;
            }
            ContentIndicatorDirection::Right => {
                ctx.draw_line(cx - s / 2, cy - s, cx + s / 2, cy, self.color)?;
                ctx.draw_line(cx + s / 2, cy, cx - s / 2, cy + s, self.color)?;
            }
        }
        Ok(())
    }
}

impl Widget for ContentIndicatorWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::State => Some(PropertyValue::Bool(self.visible)),
            PropertyKey::Progress => Some(PropertyValue::Float(self.pulse_progress)),
            _ => None,
        }
    }
}

/// Crumbs pagination dots widget showing horizontal screen/card deck positions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrumbsIndicatorWidget {
    pub count: u8,
    pub active_index: u8,
    pub dot_radius: u8,
    pub dot_spacing: u8,
    pub active_color: Rgb565,
    pub inactive_color: Rgb565,
}

impl CrumbsIndicatorWidget {
    pub const fn new(count: u8, active_index: u8) -> Self {
        Self {
            count,
            active_index,
            dot_radius: 2,
            dot_spacing: 6,
            active_color: Rgb565::new(31, 63, 31),
            inactive_color: Rgb565::new(10, 20, 10),
        }
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        if self.count == 0 {
            return Ok(());
        }

        let total_w =
            (self.count as i32 - 1) * (self.dot_spacing as i32) + (self.dot_radius as i32 * 2);
        let start_x = bounds.x + (bounds.w as i32 - total_w) / 2 + self.dot_radius as i32;
        let cy = bounds.y + (bounds.h as i32 / 2);

        for i in 0..self.count {
            let cx = start_x + (i as i32 * self.dot_spacing as i32);
            let is_active = i == self.active_index;
            let color = if is_active {
                self.active_color
            } else {
                self.inactive_color
            };
            let r = if is_active {
                self.dot_radius as u32 + 1
            } else {
                self.dot_radius as u32
            };
            ctx.fill_circle(cx, cy, r, color)?;
        }
        Ok(())
    }
}

impl Widget for CrumbsIndicatorWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Selected => Some(PropertyValue::Int(self.active_index as i32)),
            _ => None,
        }
    }
}

/// Segmented multi-cell selection control (digits, PIN, time/date).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionWidget<'a, const MAX_CELLS: usize = 6> {
    pub cell_texts: [&'a str; MAX_CELLS],
    pub cell_count: usize,
    pub selected_cell: usize,
    pub is_active: bool,
    pub bump_offset_y: i8,
    pub slide_offset_x: i8,
    pub active_bg_color: Rgb565,
    pub active_text_color: Rgb565,
    pub inactive_bg_color: Rgb565,
    pub inactive_text_color: Rgb565,
}

impl<'a, const MAX_CELLS: usize> SelectionWidget<'a, MAX_CELLS> {
    pub const fn new(cell_texts: [&'a str; MAX_CELLS], cell_count: usize) -> Self {
        Self {
            cell_texts,
            cell_count,
            selected_cell: 0,
            is_active: true,
            bump_offset_y: 0,
            slide_offset_x: 0,
            active_bg_color: Rgb565::new(31, 63, 31),
            active_text_color: Rgb565::new(0, 0, 0),
            inactive_bg_color: Rgb565::new(3, 6, 3),
            inactive_text_color: Rgb565::new(31, 63, 31),
        }
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        if self.cell_count == 0 {
            return Ok(());
        }

        let cell_w = (bounds.w as i32 / self.cell_count as i32).max(1);
        let cell_h = bounds.h as i32;

        for i in 0..self.cell_count {
            let cx = bounds.x + (i as i32 * cell_w);
            let is_selected = i == self.selected_cell;

            let cell_rect = Rect::new(cx, bounds.y, cell_w as u32, cell_h as u32);
            let bg_color = if is_selected && self.is_active {
                self.active_bg_color
            } else {
                self.inactive_bg_color
            };
            let text_color = if is_selected && self.is_active {
                self.active_text_color
            } else {
                self.inactive_text_color
            };

            ctx.fill_rounded_rect(cell_rect, 2, bg_color)?;

            let text = if i < self.cell_texts.len() {
                self.cell_texts[i]
            } else {
                ""
            };
            let text_y = bounds.y
                + (if is_selected {
                    self.bump_offset_y as i32
                } else {
                    0
                });
            let text_x = cx + (cell_w / 2) - 4;

            ctx.draw_text(text_x, text_y + (cell_h / 2) - 3, text, text_color)?;
        }
        Ok(())
    }
}

impl<'a, const MAX_CELLS: usize> Widget for SelectionWidget<'a, MAX_CELLS> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Selected => Some(PropertyValue::Int(self.selected_cell as i32)),
            PropertyKey::State => Some(PropertyValue::Bool(self.is_active)),
            _ => None,
        }
    }
}

/// 3-Slot contextual Action Bar widget mapping hardware buttons (Up, Select, Down).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActionBarWidget<'a> {
    pub up_icon: Option<char>,
    pub select_icon: Option<char>,
    pub down_icon: Option<char>,
    pub up_label: Option<&'a str>,
    pub select_label: Option<&'a str>,
    pub down_label: Option<&'a str>,
    pub background_color: Rgb565,
    pub icon_color: Rgb565,
}

impl<'a> Default for ActionBarWidget<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ActionBarWidget<'a> {
    pub const fn new() -> Self {
        Self {
            up_icon: None,
            select_icon: None,
            down_icon: None,
            up_label: None,
            select_label: None,
            down_label: None,
            background_color: Rgb565::new(0, 0, 0),
            icon_color: Rgb565::new(31, 63, 31),
        }
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        ctx.fill_rounded_rect(bounds, 2, self.background_color)?;

        let slot_h = bounds.h as i32 / 3;
        let icon_x = bounds.x + (bounds.w as i32 / 2) - 3;

        // Slot 0: Up
        if let Some(lbl) = self.up_label {
            ctx.draw_text(icon_x, bounds.y + (slot_h / 2) - 3, lbl, self.icon_color)?;
        }

        // Slot 1: Select
        if let Some(lbl) = self.select_label {
            ctx.draw_text(
                icon_x,
                bounds.y + slot_h + (slot_h / 2) - 3,
                lbl,
                self.icon_color,
            )?;
        }

        // Slot 2: Down
        if let Some(lbl) = self.down_label {
            ctx.draw_text(
                icon_x,
                bounds.y + (slot_h * 2) + (slot_h / 2) - 3,
                lbl,
                self.icon_color,
            )?;
        }

        Ok(())
    }
}

impl<'a> Widget for ActionBarWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Text => self.select_label.map(PropertyValue::Str),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_content_indicator_render() {
        let indicator = ContentIndicatorWidget::new(ContentIndicatorDirection::Down);
        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        assert!(indicator.render(&mut ctx, Rect::new(0, 0, 20, 20)).is_ok());
    }

    #[test]
    fn test_crumbs_indicator_render() {
        let crumbs = CrumbsIndicatorWidget::new(4, 1);
        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        assert!(crumbs.render(&mut ctx, Rect::new(0, 0, 20, 20)).is_ok());
    }

    #[test]
    fn test_selection_widget_render() {
        let sel = SelectionWidget::<3>::new(["12", "34", "56"], 3);
        let mut fb = Framebuffer::<1200>::new(60, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 60, 20));
        assert!(sel.render(&mut ctx, Rect::new(0, 0, 60, 20)).is_ok());
    }
}
