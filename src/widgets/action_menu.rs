//! Hierarchical Action Menu and Cascading Action Sheets.
//!
//! Provides `ActionMenuWidget` (contextual cascading action menu with submenus and highlight cursor).

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};
use heapless::Vec;

use crate::{
    geometry::Rect,
    render::RenderCtx,
    style::{Border, Style},
    widget::{PropertyKey, PropertyValue, Widget},
};

/// Error indicating action menu capacity exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionMenuError;

/// A single item in an Action Menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionMenuItem<'a> {
    pub label: &'a str,
    pub action_id: u16,
    pub is_submenu: bool,
}

/// Cascading Hierarchical Action Menu widget.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionMenuWidget<'a, const MAX_ITEMS: usize = 8> {
    pub title: Option<&'a str>,
    pub items: Vec<ActionMenuItem<'a>, MAX_ITEMS>,
    pub selected_index: usize,
    pub background_color: Rgb565,
    pub text_color: Rgb565,
    pub selected_bg_color: Rgb565,
    pub selected_text_color: Rgb565,
    pub accent_color: Rgb565,
}

impl<'a, const MAX_ITEMS: usize> ActionMenuWidget<'a, MAX_ITEMS> {
    pub const fn new(title: Option<&'a str>) -> Self {
        Self {
            title,
            items: Vec::new(),
            selected_index: 0,
            background_color: Rgb565::new(3, 6, 9),
            text_color: Rgb565::new(31, 63, 31),
            selected_bg_color: Rgb565::new(0, 45, 30),
            selected_text_color: Rgb565::new(31, 63, 31),
            accent_color: Rgb565::new(0, 35, 30),
        }
    }

    pub fn add_item(
        &mut self,
        label: &'a str,
        action_id: u16,
        is_submenu: bool,
    ) -> Result<(), ActionMenuError> {
        self.items
            .push(ActionMenuItem {
                label,
                action_id,
                is_submenu,
            })
            .map_err(|_| ActionMenuError)
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        ctx.fill_rounded_rect(bounds, 4, self.background_color)?;
        ctx.stroke_rounded_rect(bounds, 4, Border::one(self.accent_color))?;

        let mut y = bounds.y + 4;

        // Title (if present)
        if let Some(title) = self.title {
            ctx.draw_text(bounds.x + 8, y, title, Rgb565::new(15, 30, 20))?;
            y += 14;
        }

        let item_h = 16;
        for (i, item) in self.items.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let item_rect = Rect::new(bounds.x + 4, y, bounds.w.saturating_sub(8), item_h as u32);

            if is_selected {
                ctx.fill_rounded_rect(item_rect, 2, self.selected_bg_color)?;
            }

            let fg = if is_selected {
                self.selected_text_color
            } else {
                self.text_color
            };
            ctx.draw_text(bounds.x + 10, y + 3, item.label, fg)?;

            // Submenu chevron hint '>'
            if item.is_submenu {
                ctx.draw_text(bounds.right() - 14, y + 3, ">", fg)?;
            }

            y += item_h + 2;
        }

        Ok(())
    }
}

impl<'a, const MAX_ITEMS: usize> Widget for ActionMenuWidget<'a, MAX_ITEMS> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Selected => Some(PropertyValue::Int(self.selected_index as i32)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_action_menu_render() {
        let mut menu = ActionMenuWidget::<4>::new(Some("SETTINGS"));
        assert!(menu.add_item("Wi-Fi", 1, true).is_ok());
        assert!(menu.add_item("Bluetooth", 2, true).is_ok());
        assert!(menu.add_item("Restart", 3, false).is_ok());

        let mut fb = Framebuffer::<24000>::new(160, 100);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 160, 100));
        assert!(menu.render(&mut ctx, Rect::new(0, 0, 160, 100)).is_ok());
    }
}
