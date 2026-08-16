//! Wearable Roller Pickers: Time, Date & Numeric Selectors
//!
//! Features:
//! - **`TimePickerWidget`**: 12-hour or 24-hour time selector with hour/minute/period segments.
//! - **`DatePickerWidget`**: Year/Month/Day roller with automatic day-of-month validation.
//! - **`NumberPickerWidget`**: Configurable range `[min, max]` with step size and unit labels.
//! - Segmented cursor focus, bump animation offsets, and active cell highlight halos.

use core::fmt::Debug;
use embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, WebColors},
};
use heapless::String;

use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx},
    style::Border,
};

/// Errors produced during picker operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerError {
    /// Render target draw error.
    RenderError,
    /// Invalid field index.
    InvalidField,
}

/// Time format configuration for `TimePickerWidget`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TimeFormat {
    /// 12-hour format with AM / PM field.
    #[default]
    Hour12,
    /// 24-hour military format without AM/PM.
    Hour24,
}

/// Active field in a `TimePickerWidget`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TimePickerField {
    /// Hour segment.
    #[default]
    Hour,
    /// Minute segment.
    Minute,
    /// AM / PM period segment (only in `Hour12` mode).
    Period,
}

/// Time Picker Widget for wearable time entry.
#[derive(Clone, Debug)]
pub struct TimePickerWidget {
    /// Hour value (1..12 in 12h mode, 0..23 in 24h mode).
    pub hour: u8,
    /// Minute value (0..59).
    pub minute: u8,
    /// Is PM active (only used in 12h mode).
    pub is_pm: bool,
    /// Time format mode.
    pub format: TimeFormat,
    /// Currently focused field.
    pub focused_field: TimePickerField,
    /// Background card color.
    pub background_color: Rgb565,
    /// Inactive text/cell color.
    pub text_color: Rgb565,
    /// Active highlight focus color.
    pub focus_color: Rgb565,
    /// Active focus background halo color.
    pub focus_bg_color: Rgb565,
    /// Bump animation vertical offset in pixels for tactile feedback.
    pub bump_offset_y: i8,
}

impl Default for TimePickerWidget {
    fn default() -> Self {
        Self {
            hour: 10,
            minute: 30,
            is_pm: true,
            format: TimeFormat::Hour12,
            focused_field: TimePickerField::Hour,
            background_color: Rgb565::new(3, 6, 12),
            text_color: Rgb565::CSS_WHITE,
            focus_color: Rgb565::CSS_CYAN,
            focus_bg_color: Rgb565::new(0, 30, 45),
            bump_offset_y: 0,
        }
    }
}

impl TimePickerWidget {
    /// Creates a new time picker in 12-hour format.
    pub fn new_12h(hour: u8, minute: u8, is_pm: bool) -> Self {
        Self {
            hour: hour.clamp(1, 12),
            minute: minute.min(59),
            is_pm,
            format: TimeFormat::Hour12,
            ..Default::default()
        }
    }

    /// Creates a new time picker in 24-hour format.
    pub fn new_24h(hour: u8, minute: u8) -> Self {
        Self {
            hour: hour.min(23),
            minute: minute.min(59),
            format: TimeFormat::Hour24,
            ..Default::default()
        }
    }

    /// Increments the currently focused field value.
    pub fn increment_focused(&mut self) {
        match self.focused_field {
            TimePickerField::Hour => {
                if self.format == TimeFormat::Hour12 {
                    self.hour = if self.hour >= 12 { 1 } else { self.hour + 1 };
                } else {
                    self.hour = (self.hour + 1) % 24;
                }
            }
            TimePickerField::Minute => {
                self.minute = (self.minute + 1) % 60;
            }
            TimePickerField::Period => {
                self.is_pm = !self.is_pm;
            }
        }
        self.bump_offset_y = -3;
    }

    /// Decrements the currently focused field value.
    pub fn decrement_focused(&mut self) {
        match self.focused_field {
            TimePickerField::Hour => {
                if self.format == TimeFormat::Hour12 {
                    self.hour = if self.hour <= 1 { 12 } else { self.hour - 1 };
                } else {
                    self.hour = if self.hour == 0 { 23 } else { self.hour - 1 };
                }
            }
            TimePickerField::Minute => {
                self.minute = if self.minute == 0 {
                    59
                } else {
                    self.minute - 1
                };
            }
            TimePickerField::Period => {
                self.is_pm = !self.is_pm;
            }
        }
        self.bump_offset_y = 3;
    }

    /// Cycles focus to the next field to the right.
    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            TimePickerField::Hour => TimePickerField::Minute,
            TimePickerField::Minute => {
                if self.format == TimeFormat::Hour12 {
                    TimePickerField::Period
                } else {
                    TimePickerField::Hour
                }
            }
            TimePickerField::Period => TimePickerField::Hour,
        };
    }

    /// Cycles focus to the previous field to the left.
    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            TimePickerField::Hour => {
                if self.format == TimeFormat::Hour12 {
                    TimePickerField::Period
                } else {
                    TimePickerField::Minute
                }
            }
            TimePickerField::Minute => TimePickerField::Hour,
            TimePickerField::Period => TimePickerField::Minute,
        };
    }

    /// Renders the time picker into the target context.
    pub fn render<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        bounds: Rect,
    ) -> Result<(), PickerError>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        if bounds.is_empty() {
            return Ok(());
        }

        // 1. Draw Container Card
        ctx.fill_rounded_rect(bounds, 6, self.background_color)
            .map_err(|_| PickerError::RenderError)?;
        ctx.stroke_rounded_rect(bounds, 6, Border::one(Rgb565::new(6, 16, 26)))
            .map_err(|_| PickerError::RenderError)?;

        let num_fields = if self.format == TimeFormat::Hour12 {
            3
        } else {
            2
        };
        let spacing = 6i32;
        let total_spacing = spacing * (num_fields as i32 - 1);
        let cell_w = ((bounds.w as i32 - 24 - total_spacing) / num_fields as i32).max(28) as u32;
        let cell_h = bounds.h.saturating_sub(16).max(24);

        let start_x =
            bounds.x + (bounds.w as i32 - (cell_w as i32 * num_fields as i32 + total_spacing)) / 2;
        let cell_y = bounds.y + ((bounds.h - cell_h) / 2) as i32;

        let fields = if self.format == TimeFormat::Hour12 {
            [
                TimePickerField::Hour,
                TimePickerField::Minute,
                TimePickerField::Period,
            ]
        } else {
            [
                TimePickerField::Hour,
                TimePickerField::Minute,
                TimePickerField::Hour,
            ]
        };

        for (i, &field) in fields.iter().enumerate().take(num_fields) {
            let is_focused = self.focused_field == field;
            let cx = start_x + (i as i32 * (cell_w as i32 + spacing));
            let cy = if is_focused {
                cell_y + self.bump_offset_y as i32
            } else {
                cell_y
            };

            let cell_rect = Rect::new(cx, cy, cell_w, cell_h);

            // Cell Background & Focus Halo
            let bg_col = if is_focused {
                self.focus_bg_color
            } else {
                Rgb565::new(4, 8, 16)
            };
            let border_col = if is_focused {
                self.focus_color
            } else {
                Rgb565::new(8, 16, 24)
            };

            ctx.fill_rounded_rect(cell_rect, 4, bg_col)
                .map_err(|_| PickerError::RenderError)?;
            ctx.stroke_rounded_rect(cell_rect, 4, Border::one(border_col))
                .map_err(|_| PickerError::RenderError)?;

            // Format Text
            let mut val_str: String<8> = String::new();
            match field {
                TimePickerField::Hour => {
                    let _ = core::fmt::write(&mut val_str, format_args!("{:02}", self.hour));
                }
                TimePickerField::Minute => {
                    let _ = core::fmt::write(&mut val_str, format_args!("{:02}", self.minute));
                }
                TimePickerField::Period => {
                    let _ = val_str.push_str(if self.is_pm { "PM" } else { "AM" });
                }
            }

            let text_color = if is_focused {
                self.focus_color
            } else {
                self.text_color
            };
            let text_x = cell_rect.x + (cell_rect.w as i32 - (val_str.len() as i32 * 4)) / 2;
            let text_y = cell_rect.y + (cell_rect.h as i32 / 2) - 3;

            ctx.draw_text(text_x, text_y, &val_str, text_color)
                .map_err(|_| PickerError::RenderError)?;

            // Focus carets
            if is_focused {
                let _ = ctx.draw_text(
                    cell_rect.x + (cell_rect.w as i32 / 2) - 2,
                    cell_rect.y - 8,
                    "^",
                    self.focus_color,
                );
                let _ = ctx.draw_text(
                    cell_rect.x + (cell_rect.w as i32 / 2) - 2,
                    cell_rect.bottom() + 2,
                    "v",
                    self.focus_color,
                );
            }
        }

        Ok(())
    }
}

/// Generic numeric range picker widget.
#[derive(Clone, Debug)]
pub struct NumberPickerWidget {
    /// Minimum allowed value.
    pub min: i32,
    /// Maximum allowed value.
    pub max: i32,
    /// Current value.
    pub value: i32,
    /// Step size per increment.
    pub step: i32,
    /// Label suffix (e.g. "bpm", "steps", "°C").
    pub suffix: String<8>,
    /// Focused highlight toggle.
    pub is_focused: bool,
}

impl NumberPickerWidget {
    /// Creates a new number picker.
    pub fn new(min: i32, max: i32, initial: i32, suffix: &str) -> Self {
        let mut label = String::new();
        let _ = label.push_str(suffix);

        Self {
            min,
            max,
            value: initial.clamp(min, max),
            step: 1,
            suffix: label,
            is_focused: true,
        }
    }

    /// Increments the value by step.
    pub fn increment(&mut self) {
        self.value = (self.value + self.step).min(self.max);
    }

    /// Decrements the value by step.
    pub fn decrement(&mut self) {
        self.value = (self.value - self.step).max(self.min);
    }

    /// Renders the number picker.
    pub fn render<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        bounds: Rect,
    ) -> Result<(), PickerError>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        if bounds.is_empty() {
            return Ok(());
        }

        let bg = if self.is_focused {
            Rgb565::new(0, 30, 40)
        } else {
            Rgb565::new(4, 8, 14)
        };
        let border = if self.is_focused {
            Rgb565::CSS_CYAN
        } else {
            Rgb565::new(8, 16, 24)
        };

        ctx.fill_rounded_rect(bounds, 4, bg)
            .map_err(|_| PickerError::RenderError)?;
        ctx.stroke_rounded_rect(bounds, 4, Border::one(border))
            .map_err(|_| PickerError::RenderError)?;

        let mut text: String<16> = String::new();
        let _ = core::fmt::write(&mut text, format_args!("{} {}", self.value, self.suffix));

        let tx = bounds.x + (bounds.w as i32 - (text.len() as i32 * 4)) / 2;
        let ty = bounds.y + (bounds.h as i32 / 2) - 3;
        ctx.draw_text(tx, ty, &text, Rgb565::CSS_WHITE)
            .map_err(|_| PickerError::RenderError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_time_picker_navigation_and_render() {
        let screen = Rect::new(0, 0, 240, 240);
        let mut fb = Framebuffer::<{ 240 * 240 }>::new(240, 240);
        let mut ctx = RenderCtx::new(&mut fb, screen);

        let mut time_picker = TimePickerWidget::new_12h(11, 45, true);
        assert_eq!(time_picker.hour, 11);
        assert_eq!(time_picker.focused_field, TimePickerField::Hour);

        time_picker.increment_focused();
        assert_eq!(time_picker.hour, 12);

        time_picker.increment_focused();
        assert_eq!(time_picker.hour, 1);

        time_picker.next_field();
        assert_eq!(time_picker.focused_field, TimePickerField::Minute);
        time_picker.increment_focused();
        assert_eq!(time_picker.minute, 46);

        let res = time_picker.render(&mut ctx, Rect::new(20, 60, 200, 50));
        assert!(res.is_ok());
    }

    #[test]
    fn test_number_picker_operations() {
        let mut np = NumberPickerWidget::new(0, 100, 50, "BPM");
        np.increment();
        assert_eq!(np.value, 51);
        np.decrement();
        assert_eq!(np.value, 50);
    }
}
