use core::fmt::Write as _;
use embedded_graphics_core::pixelcolor::{Rgb565, WebColors};
use heapless::String;

use crate::{
    block::Block,
    geometry::Rect,
    render::{CHAR_WIDTH, Compositor, RenderCtx, TextAlign, TextStyle},
    style::{Border, VisualState, WidgetStyle},
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// High-precision numeric spinbox widget with digit-level cursor selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpinboxWidget {
    pub value: i32,
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub digits: u8,
    pub decimals: u8,
    pub focused_digit: u8,
}

impl SpinboxWidget {
    pub const fn new(min: i32, max: i32, value: i32) -> Self {
        Self {
            value,
            min,
            max,
            step: 1,
            digits: 4,
            decimals: 0,
            focused_digit: 0,
        }
    }

    pub const fn with_decimals(mut self, decimals: u8) -> Self {
        self.decimals = decimals;
        self
    }

    pub const fn with_digits(mut self, digits: u8) -> Self {
        self.digits = if digits == 0 { 1 } else { digits };
        self
    }

    pub const fn with_step(mut self, step: i32) -> Self {
        self.step = step;
        self
    }

    /// Step value based on currently active focused digit (10^focused_digit).
    pub fn current_digit_multiplier(&self) -> i32 {
        let mut mult: i32 = 1;
        for _ in 0..self.focused_digit {
            mult = mult.saturating_mul(10);
        }
        mult
    }

    /// Increments value at the current digit place.
    pub fn increment(&mut self) {
        let delta = self.current_digit_multiplier();
        self.value = self.value.saturating_add(delta).clamp(self.min, self.max);
    }

    /// Decrements value at the current digit place.
    pub fn decrement(&mut self) {
        let delta = self.current_digit_multiplier();
        self.value = self.value.saturating_sub(delta).clamp(self.min, self.max);
    }

    /// Moves cursor to previous (more significant / left) digit.
    pub fn prev_digit(&mut self) {
        if self.focused_digit + 1 < self.digits {
            self.focused_digit += 1;
        }
    }

    /// Moves cursor to next (less significant / right) digit.
    pub fn next_digit(&mut self) {
        if self.focused_digit > 0 {
            self.focused_digit -= 1;
        }
    }

    pub fn format_text(&self, out: &mut String<16>) {
        out.clear();
        let abs_val = self.value.abs();
        let sign = if self.value < 0 { "-" } else { "" };

        if self.decimals == 0 {
            let _ = write!(
                out,
                "{}{:0width$}",
                sign,
                abs_val,
                width = self.digits as usize
            );
        } else {
            let mut divisor = 1;
            for _ in 0..self.decimals {
                divisor *= 10;
            }
            let int_part = abs_val / divisor;
            let frac_part = abs_val % divisor;
            let _ = write!(
                out,
                "{}{:0w$}.{:0d$}",
                sign,
                int_part,
                frac_part,
                w = (self.digits.saturating_sub(self.decimals)) as usize,
                d = self.decimals as usize
            );
        }
    }

    pub fn render<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        rect: Rect,
        style: WidgetStyle,
        state: VisualState,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        let resolved = style.resolve(state);
        let block = Block::styled(resolved);
        block.render(rect, ctx)?;

        let inner = block.inner(rect);
        let mut formatted: String<16> = String::new();
        self.format_text(&mut formatted);

        let char_w = CHAR_WIDTH;
        let line_h = resolved.font.line_height();

        let total_chars = formatted.len() as u32;
        let text_w = total_chars * char_w;
        let start_x = inner.x + (inner.w.saturating_sub(text_w) / 2) as i32;
        let start_y = inner.y + (inner.h.saturating_sub(line_h) / 2) as i32;

        // Draw formatted number
        ctx.draw_text_in(
            Rect::new(start_x, start_y, text_w, line_h),
            formatted.as_str(),
            TextStyle::new(resolved.text).with_font(resolved.font),
        )?;

        // Highlight focused digit box / underline
        if self.focused_digit < self.digits {
            let digit_from_right = self.focused_digit as u32
                + if self.decimals > 0 && self.focused_digit >= self.decimals {
                    1
                } else {
                    0
                };
            let char_idx = total_chars
                .saturating_sub(1)
                .saturating_sub(digit_from_right);
            let digit_x = start_x + (char_idx * char_w) as i32;
            let underline = Rect::new(digit_x, start_y + line_h as i32 + 1, char_w, 2);
            ctx.fill_rect(underline, Rgb565::CSS_CYAN)?;
        }

        // Draw increment / decrement indicator chevrons on sides if width allows
        if inner.w >= 80 {
            let left_btn = Rect::new(inner.x + 2, inner.y + 2, 14, inner.h.saturating_sub(4));
            let right_btn = Rect::new(
                inner.right() - 16,
                inner.y + 2,
                14,
                inner.h.saturating_sub(4),
            );
            ctx.stroke_rect(left_btn, Border::one(Rgb565::CSS_GRAY))?;
            ctx.stroke_rect(right_btn, Border::one(Rgb565::CSS_GRAY))?;
            ctx.draw_text_in(
                left_btn,
                "-",
                TextStyle::new(resolved.text)
                    .with_font(resolved.font)
                    .with_align(TextAlign::Center),
            )?;
            ctx.draw_text_in(
                right_btn,
                "+",
                TextStyle::new(resolved.text)
                    .with_font(resolved.font)
                    .with_align(TextAlign::Center),
            )?;
        }

        Ok(())
    }
}

impl Widget for SpinboxWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &crate::style::Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Value => Some(PropertyValue::Int(self.value)),
            PropertyKey::Min => Some(PropertyValue::Int(self.min)),
            PropertyKey::Max => Some(PropertyValue::Int(self.max)),
            _ => None,
        }
    }

    fn set_property<'a>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Value, PropertyValue::Int(v)) => {
                self.value = v.clamp(self.min, self.max);
                Ok(())
            }
            (PropertyKey::Min, PropertyValue::Int(m)) => {
                self.min = m;
                Ok(())
            }
            (PropertyKey::Max, PropertyValue::Int(m)) => {
                self.max = m;
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}
