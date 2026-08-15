//! Actionable & Confirmation Dialog Widgets
//!
//! Provides modal dialogs and alert prompt cards with:
//! - Icon glyph header (Info, Warning, Error, Success, Question)
//! - Multi-line title & message body text
//! - Interactive action buttons with focus cursor and callback action IDs
//! - Automated contrast styling and rounded border frames

use core::fmt::Debug;
use embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, WebColors},
};
use heapless::{String, Vec};

use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx},
    style::Border,
};

/// Errors produced during dialog operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogError {
    /// Render target draw error.
    RenderError,
    /// Capacity exceeded for action buttons.
    CapacityExceeded,
}

/// Dialog icon types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DialogType {
    /// Informational dialog with cyan info icon.
    #[default]
    Info,
    /// Warning dialog with amber exclamation icon.
    Warning,
    /// Critical error dialog with red cross icon.
    Error,
    /// Success dialog with green checkmark icon.
    Success,
    /// Question / confirmation prompt with gold question icon.
    Question,
}

/// Action button descriptor in an actionable dialog.
#[derive(Clone, Debug)]
pub struct DialogAction {
    /// Text label for the action button.
    pub label: String<16>,
    /// Unique action identifier passed when triggered.
    pub action_id: u16,
    /// Is this a destructive / warning action (rendered in red).
    pub is_destructive: bool,
}

impl DialogAction {
    /// Creates a standard action button.
    pub fn new(label: &str, action_id: u16) -> Self {
        let mut text = String::new();
        let _ = text.push_str(label);
        Self {
            label: text,
            action_id,
            is_destructive: false,
        }
    }

    /// Creates a destructive action button.
    pub fn destructive(label: &str, action_id: u16) -> Self {
        let mut text = String::new();
        let _ = text.push_str(label);
        Self {
            label: text,
            action_id,
            is_destructive: true,
        }
    }
}

/// Self-contained actionable modal dialog.
#[derive(Clone, Debug)]
pub struct ActionableDialogWidget<const MAX_ACTIONS: usize = 3> {
    /// Dialog type icon and theme.
    pub dialog_type: DialogType,
    /// Dialog title.
    pub title: String<24>,
    /// Dialog message body.
    pub message: String<64>,
    /// Action buttons collection.
    pub actions: Vec<DialogAction, MAX_ACTIONS>,
    /// Index of currently selected/focused action.
    pub selected_action: usize,
    /// Background card color.
    pub background_color: Rgb565,
    /// Border stroke color.
    pub border_color: Rgb565,
}

impl<const MAX_ACTIONS: usize> ActionableDialogWidget<MAX_ACTIONS> {
    /// Creates a new actionable dialog.
    pub fn new(title: &str, message: &str, dialog_type: DialogType) -> Self {
        let mut title_str = String::new();
        let _ = title_str.push_str(title);

        let mut msg_str = String::new();
        let _ = msg_str.push_str(message);

        let border_color = match dialog_type {
            DialogType::Info => Rgb565::CSS_CYAN,
            DialogType::Warning => Rgb565::CSS_ORANGE,
            DialogType::Error => Rgb565::CSS_RED,
            DialogType::Success => Rgb565::CSS_GREEN,
            DialogType::Question => Rgb565::CSS_GOLD,
        };

        Self {
            dialog_type,
            title: title_str,
            message: msg_str,
            actions: Vec::new(),
            selected_action: 0,
            background_color: Rgb565::new(4, 8, 14),
            border_color,
        }
    }

    /// Adds an action button to the dialog.
    pub fn add_action(&mut self, action: DialogAction) -> Result<(), DialogError> {
        self.actions
            .push(action)
            .map_err(|_| DialogError::CapacityExceeded)
    }

    /// Selects the next action button to the right.
    pub fn select_next(&mut self) {
        if !self.actions.is_empty() {
            self.selected_action = (self.selected_action + 1) % self.actions.len();
        }
    }

    /// Selects the previous action button to the left.
    pub fn select_prev(&mut self) {
        if !self.actions.is_empty() {
            self.selected_action = if self.selected_action == 0 {
                self.actions.len() - 1
            } else {
                self.selected_action - 1
            };
        }
    }

    /// Gets the action ID of the currently selected button.
    pub fn current_action_id(&self) -> Option<u16> {
        self.actions.get(self.selected_action).map(|a| a.action_id)
    }

    /// Renders the actionable dialog.
    pub fn render<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        bounds: Rect,
    ) -> Result<(), DialogError>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        if bounds.is_empty() {
            return Ok(());
        }

        // 1. Draw Container Box
        ctx.fill_rounded_rect(bounds, 6, self.background_color)
            .map_err(|_| DialogError::RenderError)?;
        ctx.stroke_rounded_rect(bounds, 6, Border::one(self.border_color))
            .map_err(|_| DialogError::RenderError)?;

        // 2. Icon Badge & Title Header
        let header_y = bounds.y + 10;
        let (icon_symbol, icon_color) = match self.dialog_type {
            DialogType::Info => ("(i)", Rgb565::CSS_CYAN),
            DialogType::Warning => ("(!)", Rgb565::CSS_ORANGE),
            DialogType::Error => ("(X)", Rgb565::CSS_RED),
            DialogType::Success => ("(V)", Rgb565::CSS_GREEN),
            DialogType::Question => ("(?)", Rgb565::CSS_GOLD),
        };

        ctx.draw_text(bounds.x + 12, header_y, icon_symbol, icon_color)
            .map_err(|_| DialogError::RenderError)?;
        ctx.draw_text(bounds.x + 32, header_y, &self.title, Rgb565::CSS_WHITE)
            .map_err(|_| DialogError::RenderError)?;

        // 3. Message Body Text
        ctx.draw_text(
            bounds.x + 12,
            header_y + 18,
            &self.message,
            Rgb565::new(20, 40, 30),
        )
        .map_err(|_| DialogError::RenderError)?;

        // 4. Action Buttons along bottom
        if !self.actions.is_empty() {
            let num_acts = self.actions.len() as i32;
            let spacing = 6i32;
            let total_spacing = spacing * (num_acts - 1);
            let btn_w = ((bounds.w as i32 - 24 - total_spacing) / num_acts).max(36) as u32;
            let btn_h = 20u32;
            let btn_y = bounds.bottom() - 10 - btn_h as i32;
            let start_x = bounds.x + 12;

            for (i, action) in self.actions.iter().enumerate() {
                let is_selected = i == self.selected_action;
                let bx = start_x + (i as i32 * (btn_w as i32 + spacing));
                let btn_rect = Rect::new(bx, btn_y, btn_w, btn_h);

                let bg = if is_selected {
                    if action.is_destructive {
                        Rgb565::new(30, 4, 4)
                    } else {
                        Rgb565::new(0, 35, 45)
                    }
                } else {
                    Rgb565::new(6, 12, 18)
                };

                let border = if is_selected {
                    if action.is_destructive {
                        Rgb565::CSS_RED
                    } else {
                        Rgb565::CSS_CYAN
                    }
                } else {
                    Rgb565::new(10, 20, 30)
                };

                ctx.fill_rounded_rect(btn_rect, 3, bg)
                    .map_err(|_| DialogError::RenderError)?;
                ctx.stroke_rounded_rect(btn_rect, 3, Border::one(border))
                    .map_err(|_| DialogError::RenderError)?;

                let text_x = btn_rect.x + (btn_rect.w as i32 - (action.label.len() as i32 * 4)) / 2;
                let text_y = btn_rect.y + 6;
                let text_color = if is_selected {
                    Rgb565::CSS_WHITE
                } else {
                    Rgb565::new(15, 30, 25)
                };

                ctx.draw_text(text_x, text_y, &action.label, text_color)
                    .map_err(|_| DialogError::RenderError)?;
            }
        }

        Ok(())
    }
}

/// Standard 2-button confirmation dialog.
#[derive(Clone, Debug)]
pub struct ConfirmationDialogWidget {
    /// Inner actionable dialog instance.
    pub dialog: ActionableDialogWidget<2>,
}

impl ConfirmationDialogWidget {
    /// Creates a confirmation prompt with Confirm and Cancel buttons.
    pub fn new(title: &str, message: &str, confirm_id: u16, cancel_id: u16) -> Self {
        let mut dialog = ActionableDialogWidget::new(title, message, DialogType::Question);
        let _ = dialog.add_action(DialogAction::new("CANCEL", cancel_id));
        let _ = dialog.add_action(DialogAction::new("CONFIRM", confirm_id));
        dialog.selected_action = 1; // Default to confirm
        Self { dialog }
    }

    /// Renders the confirmation dialog.
    pub fn render<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        bounds: Rect,
    ) -> Result<(), DialogError>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        self.dialog.render(ctx, bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_dialog_actions_and_render() {
        let screen = Rect::new(0, 0, 240, 240);
        let mut fb = Framebuffer::<{ 240 * 240 }>::new(240, 240);
        let mut ctx = RenderCtx::new(&mut fb, screen);

        let mut dialog = ActionableDialogWidget::<3>::new(
            "DELETE ENTRY?",
            "This action cannot be undone.",
            DialogType::Warning,
        );
        assert!(dialog.add_action(DialogAction::new("CANCEL", 1)).is_ok());
        assert!(
            dialog
                .add_action(DialogAction::destructive("DELETE", 2))
                .is_ok()
        );

        assert_eq!(dialog.current_action_id(), Some(1));
        dialog.select_next();
        assert_eq!(dialog.current_action_id(), Some(2));

        let res = dialog.render(&mut ctx, Rect::new(10, 60, 220, 100));
        assert!(res.is_ok());
    }

    #[test]
    fn test_confirmation_dialog() {
        let confirm = ConfirmationDialogWidget::new("SYNC DATA", "Upload 12 pending logs?", 10, 20);
        assert_eq!(confirm.dialog.actions.len(), 2);
    }
}
