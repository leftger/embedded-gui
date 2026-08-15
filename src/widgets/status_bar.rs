//! Dynamic Wearable System Status Bar Widget
//!
//! Provides a standardized, configurable status bar displaying:
//! - Centered Clock time (12-hour or 24-hour mode)
//! - Battery percentage & battery gauge icon with optional charging lightning bolt glyph
//! - Bluetooth connectivity status indicator
//! - Do-Not-Disturb / Quiet Time indicator
//! - Unobstructed area integration for smooth slide-out and layout adaptation

use core::fmt::Debug;
use embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, WebColors},
};
use heapless::String;

use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx},
    round::UnobstructedArea,
    style::Border,
};

/// Errors produced during status bar operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusBarError {
    /// Render target failure.
    RenderError,
}

/// Status bar display modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StatusBarMode {
    /// Standard clock with battery and icons.
    #[default]
    ClockAndIcons,
    /// Clock only centered across the entire bar width.
    ClockOnly,
    /// Icons only (battery, BT, DND) without clock.
    IconsOnly,
}

/// Battery charging state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BatteryState {
    /// Discharging with percentage [0..100].
    #[default]
    Discharging,
    /// Actively charging with percentage [0..100].
    Charging,
    /// Fully charged (100%).
    Full,
}

/// Dynamic Wearable System Status Bar.
#[derive(Clone, Debug)]
pub struct StatusBarWidget {
    /// Current display mode.
    pub mode: StatusBarMode,
    /// Time text string (e.g. "10:42" or "10:42 AM").
    pub time_text: String<12>,
    /// Battery level percentage [0..100].
    pub battery_percent: u8,
    /// Battery charging state.
    pub battery_state: BatteryState,
    /// Bluetooth link connected flag.
    pub bluetooth_connected: bool,
    /// Do-Not-Disturb / Quiet Time active flag.
    pub dnd_active: bool,
    /// Background color of the status bar.
    pub background_color: Rgb565,
    /// Foreground text and icon color.
    pub foreground_color: Rgb565,
    /// Accent color for charging / connected indicators.
    pub accent_color: Rgb565,
    /// Optional bottom separator line color.
    pub separator_color: Option<Rgb565>,
    /// Standard bar height in pixels (typically 18..24).
    pub height: u16,
    /// Visibility toggle.
    pub is_visible: bool,
}

impl Default for StatusBarWidget {
    fn default() -> Self {
        let mut time_text = String::new();
        let _ = time_text.push_str("12:00");

        Self {
            mode: StatusBarMode::ClockAndIcons,
            time_text,
            battery_percent: 85,
            battery_state: BatteryState::Discharging,
            bluetooth_connected: true,
            dnd_active: false,
            background_color: Rgb565::new(2, 4, 8),
            foreground_color: Rgb565::CSS_WHITE,
            accent_color: Rgb565::CSS_CYAN,
            separator_color: Some(Rgb565::new(6, 12, 18)),
            height: 20,
            is_visible: true,
        }
    }
}

impl StatusBarWidget {
    /// Creates a new status bar with the specified initial time string.
    pub fn new(time_text: &str) -> Self {
        let mut widget = Self::default();
        widget.set_time(time_text);
        widget
    }

    /// Sets the time text.
    pub fn set_time(&mut self, time_str: &str) {
        self.time_text.clear();
        let _ = self.time_text.push_str(time_str);
    }

    /// Updates battery metrics.
    pub fn set_battery(&mut self, percent: u8, state: BatteryState) {
        self.battery_percent = percent.min(100);
        self.battery_state = state;
    }

    /// Applies the status bar bounds to an `UnobstructedArea`.
    pub fn apply_to_unobstructed_area(&self, area: &mut UnobstructedArea) {
        if self.is_visible && self.height > 0 {
            area.set_insets(self.height, 0, 0, 0);
        }
    }

    /// Renders the status bar inside the provided bounds.
    pub fn render<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        bounds: Rect,
    ) -> Result<(), StatusBarError>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        if !self.is_visible || bounds.is_empty() {
            return Ok(());
        }

        // 1. Background fill
        ctx.fill_rect(bounds, self.background_color)
            .map_err(|_| StatusBarError::RenderError)?;

        // 2. Optional bottom separator line
        if let Some(sep_color) = self.separator_color {
            ctx.fill_rect(
                Rect::new(bounds.x, bounds.bottom() - 1, bounds.w, 1),
                sep_color,
            )
            .map_err(|_| StatusBarError::RenderError)?;
        }

        let center_y = bounds.y + (bounds.h as i32 / 2);

        // 3. Render Clock Time (Center)
        if self.mode == StatusBarMode::ClockAndIcons || self.mode == StatusBarMode::ClockOnly {
            let char_width = 4;
            let text_w = self.time_text.len() as i32 * char_width;
            let time_x = bounds.x + (bounds.w as i32 - text_w) / 2;
            let time_y = center_y - 3;
            ctx.draw_text(time_x, time_y, &self.time_text, self.foreground_color)
                .map_err(|_| StatusBarError::RenderError)?;
        }

        // 4. Render Left-Side Icons (Bluetooth & DND)
        if self.mode == StatusBarMode::ClockAndIcons || self.mode == StatusBarMode::IconsOnly {
            let mut left_cursor = bounds.x + 6;

            if self.bluetooth_connected {
                // Bluetooth glyph icon (5x7 diamond / antenna)
                let bt_color = self.accent_color;
                let bx = left_cursor;
                let by = center_y - 4;

                let _ = ctx.stroke_rect(Rect::new(bx, by, 6, 8), Border::one(bt_color));
                let _ = ctx.draw_text(bx + 1, by + 1, "B", bt_color);
                left_cursor += 10;
            }

            if self.dnd_active {
                // Moon crescent / dot glyph for DND
                let mx = left_cursor;
                let my = center_y - 3;
                let _ = ctx.fill_circle(mx + 3, my + 3, 3, Rgb565::CSS_GOLD);
                let _ = ctx.fill_circle(mx + 4, my + 2, 2, self.background_color);
            }

            // 5. Render Right-Side Icons (Battery gauge & percent)
            let right_cursor = bounds.right() - 6;

            // Battery shell: 16x8 rectangle with 2x4 terminal nipple
            let batt_w = 16u32;
            let batt_h = 8u32;
            let batt_x = right_cursor - (batt_w as i32);
            let batt_y = center_y - 4;

            let shell_rect = Rect::new(batt_x, batt_y, batt_w, batt_h);
            ctx.stroke_rect(shell_rect, Border::one(self.foreground_color))
                .map_err(|_| StatusBarError::RenderError)?;

            // Terminal nipple on right edge
            ctx.fill_rect(
                Rect::new(batt_x + batt_w as i32, batt_y + 2, 2, 4),
                self.foreground_color,
            )
            .map_err(|_| StatusBarError::RenderError)?;

            // Fill battery level inside
            let max_fill = batt_w.saturating_sub(4);
            let fill_w = ((self.battery_percent as u32 * max_fill) / 100).max(1);
            let fill_color = if self.battery_percent <= 15 {
                Rgb565::CSS_RED
            } else if self.battery_state == BatteryState::Charging {
                Rgb565::CSS_GREEN
            } else {
                self.foreground_color
            };

            ctx.fill_rect(
                Rect::new(batt_x + 2, batt_y + 2, fill_w, batt_h - 4),
                fill_color,
            )
            .map_err(|_| StatusBarError::RenderError)?;

            // Charging lightning bolt indicator
            if self.battery_state == BatteryState::Charging {
                ctx.draw_text(batt_x - 8, batt_y + 1, "~", Rgb565::CSS_YELLOW)
                    .map_err(|_| StatusBarError::RenderError)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_status_bar_render_and_insets() {
        let screen = Rect::new(0, 0, 240, 240);
        let mut fb = Framebuffer::<{ 240 * 240 }>::new(240, 240);
        let mut ctx = RenderCtx::new(&mut fb, screen);

        let mut status_bar = StatusBarWidget::new("09:41");
        status_bar.set_battery(72, BatteryState::Charging);
        status_bar.dnd_active = true;

        let bar_bounds = Rect::new(0, 0, 240, 20);
        let res = status_bar.render(&mut ctx, bar_bounds);
        assert!(res.is_ok());

        let mut area = UnobstructedArea::new(screen);
        status_bar.apply_to_unobstructed_area(&mut area);
        assert_eq!(area.visible_rect(), Rect::new(0, 20, 240, 220));
    }
}
