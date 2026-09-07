//! Compass heading tape and dial widget for wearable & outdoor MCU displays.
//!
//! Provides zero-allocation rendering of horizontal cardinal direction tapes
//! (N, NE, E, SE, S, SW, W, NW) and heading degrees (0°..360°).

use core::fmt::Write as _;
use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};
use heapless::String;

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx, TextAlign, TextStyle},
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

const CARDINALS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

/// Cardinal direction helper function.
pub fn cardinal_for_heading(heading_deg: f32) -> &'static str {
    let norm = (heading_deg % 360.0 + 360.0) % 360.0;
    let idx = (((norm + 22.5) / 45.0) as usize) % 8;
    CARDINALS[idx]
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompassMode {
    /// Ribbon tape scrolling left/right with center cursor.
    #[default]
    HorizontalTape,
    /// Compact numeric degree readout with cardinal direction indicator.
    CompactDial,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompassWidget {
    /// Compass heading in degrees (0.0 .. 360.0).
    pub heading_deg: f32,
    /// Display mode (HorizontalTape or CompactDial).
    pub mode: CompassMode,
    /// Center needle / cursor color.
    pub needle_color: Rgb565,
    /// Standard label and numeric text color.
    pub text_color: Rgb565,
    /// Border and tick mark color.
    pub tick_color: Rgb565,
    /// Highlight color for cardinal directions (N, E, S, W).
    pub cardinal_color: Rgb565,
}

impl Default for CompassWidget {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl CompassWidget {
    /// Create a new compass widget with the specified heading in degrees.
    ///
    /// Heading is normalized to `0.0..360.0` degrees.
    pub fn new(heading_deg: f32) -> Self {
        Self {
            heading_deg: (heading_deg % 360.0 + 360.0) % 360.0,
            mode: CompassMode::HorizontalTape,
            needle_color: Rgb565::new(31, 0, 0),
            text_color: Rgb565::new(31, 63, 31),
            tick_color: Rgb565::new(20, 40, 20),
            cardinal_color: Rgb565::new(0, 63, 63),
        }
    }

    /// Set the rendering mode.
    pub fn with_mode(mut self, mode: CompassMode) -> Self {
        self.mode = mode;
        self
    }

    /// Override colors for needle, text, ticks, and cardinal highlights.
    pub fn with_colors(
        mut self,
        needle: Rgb565,
        text: Rgb565,
        tick: Rgb565,
        cardinal: Rgb565,
    ) -> Self {
        self.needle_color = needle;
        self.text_color = text;
        self.tick_color = tick;
        self.cardinal_color = cardinal;
        self
    }

    /// Returns cardinal direction string ("N", "NE", "E", etc.) for current heading.
    pub fn cardinal_name(&self) -> &'static str {
        cardinal_for_heading(self.heading_deg)
    }

    /// Render compass widget inside `bounds`.
    pub fn render<D, C>(&self, bounds: Rect, ctx: &mut RenderCtx<'_, D, C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        if bounds.w == 0 || bounds.h == 0 {
            return Ok(());
        }

        match self.mode {
            CompassMode::HorizontalTape => self.render_tape(bounds, ctx),
            CompassMode::CompactDial => self.render_dial(bounds, ctx),
        }
    }

    fn render_tape<D, C>(&self, bounds: Rect, ctx: &mut RenderCtx<'_, D, C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        // Top and bottom border lines
        ctx.draw_line(
            bounds.x,
            bounds.y,
            bounds.right() - 1,
            bounds.y,
            self.tick_color,
        )?;
        ctx.draw_line(
            bounds.x,
            bounds.bottom() - 1,
            bounds.right() - 1,
            bounds.bottom() - 1,
            self.tick_color,
        )?;

        let center_x = bounds.x + (bounds.w as i32 / 2);

        // Center vertical needle / cursor line
        ctx.draw_line(
            center_x,
            bounds.y,
            center_x,
            bounds.bottom() - 1,
            self.needle_color,
        )?;

        let min_deg_offset = -(center_x - bounds.x);
        let max_deg_offset = bounds.right() - 1 - center_x;

        let start_step = ((self.heading_deg + min_deg_offset as f32) / 15.0).floor() as i32;
        let end_step = ((self.heading_deg + max_deg_offset as f32) / 15.0).ceil() as i32;

        for step in start_step..=end_step {
            let tick_deg_unwrapped = step * 15;
            let diff = (tick_deg_unwrapped as f32) - self.heading_deg;
            let x = center_x + diff.round() as i32;

            if x >= bounds.x && x < bounds.right() {
                let tick_deg = tick_deg_unwrapped.rem_euclid(360);
                let is_major = tick_deg % 45 == 0;

                if is_major {
                    let label = match tick_deg {
                        0 => "N",
                        45 => "NE",
                        90 => "E",
                        135 => "SE",
                        180 => "S",
                        225 => "SW",
                        270 => "W",
                        315 => "NW",
                        _ => "",
                    };

                    let label_color = match tick_deg {
                        0 | 90 | 180 | 270 => self.cardinal_color,
                        _ => self.text_color,
                    };

                    if x != center_x {
                        ctx.draw_line(x, bounds.y, x, bounds.y + 3, self.tick_color)?;
                        ctx.draw_line(
                            x,
                            bounds.bottom() - 4,
                            x,
                            bounds.bottom() - 1,
                            self.tick_color,
                        )?;
                    }

                    let text_rect = Rect::new(x - 10, bounds.y + 1, 20, bounds.h.saturating_sub(2));
                    ctx.draw_text_in(
                        text_rect,
                        label,
                        TextStyle::new(label_color).with_align(TextAlign::Center),
                    )?;
                } else if x != center_x {
                    ctx.draw_line(x, bounds.y, x, bounds.y + 2, self.tick_color)?;
                    ctx.draw_line(
                        x,
                        bounds.bottom() - 3,
                        x,
                        bounds.bottom() - 1,
                        self.tick_color,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn render_dial<D, C>(&self, bounds: Rect, ctx: &mut RenderCtx<'_, D, C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        let deg_int = (self.heading_deg.round() as i32).rem_euclid(360);
        let mut deg_str: String<16> = String::new();
        let _ = write!(&mut deg_str, "{}°", deg_int);

        let card_str = self.cardinal_name();
        let card_color = match deg_int {
            0 | 90 | 180 | 270 => self.cardinal_color,
            _ => self.text_color,
        };

        if bounds.h >= 14 {
            let half_h = bounds.h / 2;
            let top_rect = Rect::new(bounds.x, bounds.y, bounds.w, half_h);
            ctx.draw_text_in(
                top_rect,
                deg_str.as_str(),
                TextStyle::new(self.text_color).with_align(TextAlign::Center),
            )?;

            let bot_rect = Rect::new(
                bounds.x,
                bounds.y + half_h as i32,
                bounds.w,
                bounds.h - half_h,
            );
            ctx.draw_text_in(
                bot_rect,
                card_str,
                TextStyle::new(card_color).with_align(TextAlign::Center),
            )?;
        } else {
            let mut combined: String<16> = String::new();
            let _ = write!(&mut combined, "{} {}", card_str, deg_str.as_str());
            ctx.draw_text_in(
                bounds,
                combined.as_str(),
                TextStyle::new(self.text_color).with_align(TextAlign::Center),
            )?;
        }

        Ok(())
    }
}

impl Widget for CompassWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Value => Some(PropertyValue::Float(self.heading_deg)),
            PropertyKey::Text => Some(PropertyValue::Str(self.cardinal_name())),
            _ => None,
        }
    }

    fn set_property<'a>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Value, PropertyValue::Float(v)) => {
                self.heading_deg = (v % 360.0 + 360.0) % 360.0;
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;

    #[test]
    fn test_compass_heading_normalization() {
        let c1 = CompassWidget::new(450.0);
        assert_eq!(c1.heading_deg, 90.0);

        let c2 = CompassWidget::new(-90.0);
        assert_eq!(c2.heading_deg, 270.0);

        let c3 = CompassWidget::new(-450.0);
        assert_eq!(c3.heading_deg, 270.0);

        let c4 = CompassWidget::new(360.0);
        assert_eq!(c4.heading_deg, 0.0);

        let mut c5 = CompassWidget::new(0.0);
        assert!(
            c5.set_property(PropertyKey::Value, PropertyValue::Float(-45.0))
                .is_ok()
        );
        assert_eq!(c5.heading_deg, 315.0);
    }

    #[test]
    fn test_cardinal_lookup() {
        assert_eq!(cardinal_for_heading(0.0), "N");
        assert_eq!(cardinal_for_heading(90.0), "E");
        assert_eq!(cardinal_for_heading(180.0), "S");
        assert_eq!(cardinal_for_heading(270.0), "W");
        assert_eq!(cardinal_for_heading(45.0), "NE");
        assert_eq!(cardinal_for_heading(135.0), "SE");
        assert_eq!(cardinal_for_heading(225.0), "SW");
        assert_eq!(cardinal_for_heading(315.0), "NW");
    }

    #[test]
    fn test_compass_render_smoke() {
        let mut fb = Framebuffer::<1024>::new(64, 16);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 64, 16));

        let compass_tape = CompassWidget::new(180.0);
        assert!(
            compass_tape
                .render(Rect::new(0, 0, 64, 16), &mut ctx)
                .is_ok()
        );

        let compass_dial = CompassWidget::new(270.0).with_mode(CompassMode::CompactDial);
        assert!(
            compass_dial
                .render(Rect::new(0, 0, 64, 16), &mut ctx)
                .is_ok()
        );
    }
}
