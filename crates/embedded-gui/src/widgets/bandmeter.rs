//! Segmented VU-style band level meter widget.
//!
//! Renders N equal-width (or equal-height) segments from a baseline,
//! colored by threshold zones (green/yellow/red), for audio, signal,
//! battery, or any level indicator use case.

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx},
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// Orientation of the band meter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeterOrientation {
    /// Segments grow left-to-right.
    Horizontal,
    /// Segments grow bottom-to-top.
    Vertical,
}

/// Segmented VU-style band level meter widget.
///
/// `SEGMENTS` is the maximum number of segments (const generic, default 16).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BandMeterWidget<const SEGMENTS: usize = 16> {
    /// Normalized level value in `0.0 ..= 1.0`.
    pub value: f32,
    /// Optional peak-hold marker position in `0.0 ..= 1.0`.
    pub peak: Option<f32>,
    /// Number of segments actually used (clamped to `SEGMENTS`).
    pub segments: usize,
    /// Rendering orientation.
    pub orientation: MeterOrientation,
    /// Color for segments below `warn_threshold`.
    pub normal_color: Rgb565,
    /// Color for segments between `warn_threshold` and `over_threshold`.
    pub warn_color: Rgb565,
    /// Color for segments at or above `over_threshold`.
    pub over_color: Rgb565,
    /// Fraction at which warn zone begins (default `0.75`).
    pub warn_threshold: f32,
    /// Fraction at which over zone begins (default `0.90`).
    pub over_threshold: f32,
    /// Pixel gap between segments (default `1`).
    pub gap: u32,
}

impl<const SEGMENTS: usize> Default for BandMeterWidget<SEGMENTS> {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl<const SEGMENTS: usize> BandMeterWidget<SEGMENTS> {
    /// Create a new band meter with `value` in `0.0 ..= 1.0` and sensible defaults.
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            peak: None,
            segments: SEGMENTS,
            orientation: MeterOrientation::Horizontal,
            // Green
            normal_color: Rgb565::new(0, 50, 0),
            // Yellow/orange
            warn_color: Rgb565::new(31, 40, 0),
            // Red
            over_color: Rgb565::new(31, 0, 0),
            warn_threshold: 0.75,
            over_threshold: 0.90,
            gap: 1,
        }
    }

    /// Override the number of rendered segments (clamped to `SEGMENTS`).
    pub fn with_segments(mut self, n: usize) -> Self {
        self.segments = n.min(SEGMENTS);
        self
    }

    /// Set the meter orientation.
    pub fn with_orientation(mut self, o: MeterOrientation) -> Self {
        self.orientation = o;
        self
    }

    /// Set an optional peak-hold marker position.
    pub fn with_peak(mut self, p: f32) -> Self {
        self.peak = Some(p.clamp(0.0, 1.0));
        self
    }

    /// Override the three threshold colors.
    pub fn with_colors(mut self, normal: Rgb565, warn: Rgb565, over: Rgb565) -> Self {
        self.normal_color = normal;
        self.warn_color = warn;
        self.over_color = over;
        self
    }

    /// Override the warn and over threshold fractions.
    pub fn with_thresholds(mut self, warn: f32, over: f32) -> Self {
        self.warn_threshold = warn;
        self.over_threshold = over;
        self
    }

    /// Set the pixel gap between segments.
    pub fn with_gap(mut self, gap: u32) -> Self {
        self.gap = gap;
        self
    }

    /// Return the segment color for normalized position `frac` (0.0 ..= 1.0).
    #[inline]
    fn segment_color(&self, frac: f32) -> Rgb565 {
        if frac >= self.over_threshold {
            self.over_color
        } else if frac >= self.warn_threshold {
            self.warn_color
        } else {
            self.normal_color
        }
    }

    /// Render the meter into `bounds`.
    pub fn render<D, C>(&self, bounds: Rect, ctx: &mut RenderCtx<'_, D, C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        let n = self.segments;
        if n == 0 || bounds.w == 0 || bounds.h == 0 {
            return Ok(());
        }

        // Number of lit segments
        let lit = ((self.value * n as f32) as usize).min(n);

        match self.orientation {
            MeterOrientation::Horizontal => {
                // Each segment column width (accounting for gaps)
                let total_gaps = self.gap * (n as u32).saturating_sub(1);
                let available = bounds.w.saturating_sub(total_gaps);
                let seg_w = (available / n as u32).max(1);

                for i in 0..n {
                    let frac = (i as f32 + 0.5) / n as f32;
                    let color = self.segment_color(frac);

                    let x_offset = i as u32 * (seg_w + self.gap);
                    let x = bounds.x + x_offset as i32;
                    let rect = Rect::new(x, bounds.y, seg_w, bounds.h);

                    if i < lit {
                        ctx.fill_rect(rect, color)?;
                    }
                }

                // Peak marker: 2-px-wide vertical line
                if let Some(p) = self.peak {
                    let peak_x = bounds.x + (p * (bounds.w.saturating_sub(2)) as f32) as i32;
                    ctx.draw_line(
                        peak_x,
                        bounds.y,
                        peak_x,
                        bounds.bottom() - 1,
                        self.segment_color(p),
                    )?;
                    if peak_x + 1 < bounds.right() {
                        ctx.draw_line(
                            peak_x + 1,
                            bounds.y,
                            peak_x + 1,
                            bounds.bottom() - 1,
                            self.segment_color(p),
                        )?;
                    }
                }
            }

            MeterOrientation::Vertical => {
                // Each segment row height (accounting for gaps)
                let total_gaps = self.gap * (n as u32).saturating_sub(1);
                let available = bounds.h.saturating_sub(total_gaps);
                let seg_h = (available / n as u32).max(1);

                // Segments grow from the bottom upward;
                // i=0 is the bottom-most (lowest level) segment.
                for i in 0..n {
                    let frac = (i as f32 + 0.5) / n as f32;
                    let color = self.segment_color(frac);

                    let y_offset = (n - 1 - i) as u32 * (seg_h + self.gap);
                    let y = bounds.y + y_offset as i32;
                    let rect = Rect::new(bounds.x, y, bounds.w, seg_h);

                    if i < lit {
                        ctx.fill_rect(rect, color)?;
                    }
                }

                // Peak marker: 2-px-tall horizontal line
                if let Some(p) = self.peak {
                    let peak_y =
                        bounds.bottom() - 1 - (p * (bounds.h.saturating_sub(2)) as f32) as i32;
                    ctx.draw_line(
                        bounds.x,
                        peak_y,
                        bounds.right() - 1,
                        peak_y,
                        self.segment_color(p),
                    )?;
                    if peak_y > bounds.y {
                        ctx.draw_line(
                            bounds.x,
                            peak_y - 1,
                            bounds.right() - 1,
                            peak_y - 1,
                            self.segment_color(p),
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl<const SEGMENTS: usize> Widget for BandMeterWidget<SEGMENTS> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Value | PropertyKey::Progress => Some(PropertyValue::Float(self.value)),
            _ => None,
        }
    }

    fn set_property<'a>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Value | PropertyKey::Progress, PropertyValue::Float(v)) => {
                self.value = v.clamp(0.0, 1.0);
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
    fn test_band_meter_segments() {
        // Segment count is clamped to the const SEGMENTS
        let m = BandMeterWidget::<8>::new(0.5).with_segments(100);
        assert_eq!(m.segments, 8, "segments must be clamped to SEGMENTS");

        let m2 = BandMeterWidget::<8>::new(0.5).with_segments(4);
        assert_eq!(m2.segments, 4);

        // Property round-trip: Value and Progress are aliases
        let mut m3 = BandMeterWidget::<16>::new(0.3);
        assert_eq!(
            m3.get_property(PropertyKey::Value),
            Some(PropertyValue::Float(0.3))
        );
        assert!(
            m3.set_property(PropertyKey::Progress, PropertyValue::Float(0.8))
                .is_ok()
        );
        assert_eq!(
            m3.get_property(PropertyKey::Progress),
            Some(PropertyValue::Float(0.8))
        );

        // Out-of-range values are clamped
        assert!(
            m3.set_property(PropertyKey::Value, PropertyValue::Float(1.5))
                .is_ok()
        );
        assert_eq!(
            m3.get_property(PropertyKey::Value),
            Some(PropertyValue::Float(1.0))
        );
    }

    #[test]
    fn test_band_meter_render_horizontal() {
        // 8 segments at 50% → 4 lit, 4 dark
        let meter = BandMeterWidget::<8>::new(0.5).with_gap(0).with_colors(
            Rgb565::new(0, 50, 0),
            Rgb565::new(31, 40, 0),
            Rgb565::new(31, 0, 0),
        );

        // 64 × 8 framebuffer
        let mut fb = Framebuffer::<512>::new(64, 8);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 64, 8));
        assert!(meter.render(Rect::new(0, 0, 64, 8), &mut ctx).is_ok());

        // gap=0, 8 segments over 64 px → each segment is 8 px wide.
        // Segments 0-3 are lit (pixels x=0..31), segments 4-7 are dark (x=32..63).
        let pixels = fb.pixels();

        // First pixel of lit segment 0 (row 0, col 0) — should be non-black
        assert_ne!(
            pixels[0],
            Rgb565::new(0, 0, 0),
            "lit segment pixel should not be black"
        );
        // First pixel of unlit segment 4 (row 0, col 32) — should remain black
        assert_eq!(
            pixels[32],
            Rgb565::new(0, 0, 0),
            "unlit segment pixel should remain black"
        );
    }
}
