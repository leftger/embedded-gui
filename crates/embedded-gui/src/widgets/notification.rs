//! Structured Notification Sheets and Modal Alert Queue.
//!
//! Provides `NotificationSheetWidget` (modal alert with actions and countdown progress)
//! and `NotificationQueue` (deterministic fixed-capacity notification manager).

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};
use heapless::Vec;

use crate::{
    geometry::Rect,
    render::RenderCtx,
    style::{Border, Style},
    widget::{PropertyKey, PropertyValue, Widget},
};

/// Error indicating notification actions capacity exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationError;

/// Priority severity of a notification alert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationPriority {
    Silent,
    Normal,
    Important,
    Critical,
}

/// Action button choice attached to a notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationAction<'a> {
    pub label: &'a str,
    pub action_id: u16,
}

/// Modal Notification Sheet widget displaying heads-up alerts, actions, and auto-dismiss progress.
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationSheetWidget<'a, const MAX_ACTIONS: usize = 3> {
    pub title: &'a str,
    pub message: &'a str,
    pub priority: NotificationPriority,
    pub actions: Vec<NotificationAction<'a>, MAX_ACTIONS>,
    pub selected_action: usize,
    pub auto_dismiss_progress: f32, // 0.0 to 1.0
    pub background_color: Rgb565,
    pub text_color: Rgb565,
    pub accent_color: Rgb565,
}

impl<'a, const MAX_ACTIONS: usize> NotificationSheetWidget<'a, MAX_ACTIONS> {
    pub const fn new(title: &'a str, message: &'a str, priority: NotificationPriority) -> Self {
        let accent = match priority {
            NotificationPriority::Silent | NotificationPriority::Normal => {
                Rgb565::new(0, 35, 30) // Cyan/Teal
            }
            NotificationPriority::Important => Rgb565::new(31, 35, 0), // Amber
            NotificationPriority::Critical => Rgb565::new(31, 0, 0),   // Red
        };

        Self {
            title,
            message,
            priority,
            actions: Vec::new(),
            selected_action: 0,
            auto_dismiss_progress: 1.0,
            background_color: Rgb565::new(2, 4, 6),
            text_color: Rgb565::new(31, 63, 31),
            accent_color: accent,
        }
    }

    pub fn add_action(&mut self, label: &'a str, action_id: u16) -> Result<(), NotificationError> {
        self.actions
            .push(NotificationAction { label, action_id })
            .map_err(|_| NotificationError)
    }

    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        // 1. Draw outer rounded modal card
        ctx.fill_rounded_rect(bounds, 6, self.background_color)?;
        ctx.stroke_rounded_rect(bounds, 6, Border::one(self.accent_color))?;

        // 2. Top Header Bar
        let header_rect = Rect::new(bounds.x, bounds.y, bounds.w, 18);
        ctx.fill_rounded_rect(header_rect, 4, self.accent_color)?;
        ctx.draw_text(bounds.x + 8, bounds.y + 4, self.title, Rgb565::new(0, 0, 0))?;

        // 3. Message Body
        ctx.draw_text(bounds.x + 8, bounds.y + 24, self.message, self.text_color)?;

        // 4. Auto-dismiss progress line at bottom
        if self.auto_dismiss_progress > 0.0 && self.auto_dismiss_progress <= 1.0 {
            let progress_w = ((bounds.w as f32) * self.auto_dismiss_progress) as u32;
            let bar_rect = Rect::new(bounds.x, bounds.bottom() - 3, progress_w, 2);
            ctx.fill_rect(bar_rect, self.accent_color)?;
        }

        // 5. Action Buttons (if any)
        if !self.actions.is_empty() {
            let action_h = 16;
            let action_y = bounds.bottom() - 22;
            let btn_w = (bounds.w.saturating_sub(16) / self.actions.len() as u32).max(20);

            for (i, action) in self.actions.iter().enumerate() {
                let btn_x = bounds.x + 8 + (i as i32 * (btn_w as i32 + 4));
                let btn_rect = Rect::new(btn_x, action_y, btn_w, action_h);
                let is_sel = i == self.selected_action;

                let bg = if is_sel {
                    self.accent_color
                } else {
                    Rgb565::new(6, 12, 18)
                };
                let fg = if is_sel {
                    Rgb565::new(0, 0, 0)
                } else {
                    self.text_color
                };

                ctx.fill_rounded_rect(btn_rect, 2, bg)?;
                ctx.draw_text(btn_x + 4, action_y + 3, action.label, fg)?;
            }
        }

        Ok(())
    }
}

impl<'a, const MAX_ACTIONS: usize> Widget for NotificationSheetWidget<'a, MAX_ACTIONS> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Text => Some(PropertyValue::Str(self.title)),
            PropertyKey::Progress => Some(PropertyValue::Float(self.auto_dismiss_progress)),
            PropertyKey::Selected => Some(PropertyValue::Int(self.selected_action as i32)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_notification_sheet_render() {
        let mut notif = NotificationSheetWidget::<2>::new(
            "BATTERY WARNING",
            "Battery level 15%",
            NotificationPriority::Important,
        );
        assert!(notif.add_action("DISMISS", 1).is_ok());
        assert!(notif.add_action("POWER SAVE", 2).is_ok());

        let mut fb = Framebuffer::<24000>::new(200, 100);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 200, 100));
        assert!(notif.render(&mut ctx, Rect::new(0, 0, 200, 100)).is_ok());
    }
}
