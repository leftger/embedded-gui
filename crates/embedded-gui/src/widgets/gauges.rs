use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx, StrokeStyle},
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// Progress bar widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressBarWidget {
    pub value: f32,
}

impl ProgressBarWidget {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }
}

impl Widget for ProgressBarWidget {
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

/// Color threshold arc band definition for an arc gauge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaugeThreshold {
    pub min_val: f32,
    pub max_val: f32,
    pub color: Rgb565,
}

/// Rotary arc gauge widget with needle, sweep angles, and threshold color zones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArcGaugeWidget<const MAX_THRESHOLDS: usize = 4> {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub start_deg: i32,
    pub end_deg: i32,
    pub thickness: u8,
    pub track_color: Rgb565,
    pub needle_color: Rgb565,
    pub hub_color: Rgb565,
    pub thresholds: [Option<GaugeThreshold>; MAX_THRESHOLDS],
    pub threshold_count: usize,
}

impl<const MAX_THRESHOLDS: usize> Default for ArcGaugeWidget<MAX_THRESHOLDS> {
    fn default() -> Self {
        Self::new(0.0, 0.0, 100.0)
    }
}

impl<const MAX_THRESHOLDS: usize> ArcGaugeWidget<MAX_THRESHOLDS> {
    /// Create a new arc gauge with 270° sweep (135° to 405°).
    pub const fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            start_deg: 135,
            end_deg: 405,
            thickness: 3,
            track_color: Rgb565::new(6, 12, 6),
            needle_color: Rgb565::new(31, 0, 0),
            hub_color: Rgb565::new(31, 63, 31),
            thresholds: [None; MAX_THRESHOLDS],
            threshold_count: 0,
        }
    }

    /// Set sweep angles in degrees.
    pub const fn with_angles(mut self, start_deg: i32, end_deg: i32) -> Self {
        self.start_deg = start_deg;
        self.end_deg = end_deg;
        self
    }

    /// Set track and needle colors.
    pub const fn with_colors(mut self, track: Rgb565, needle: Rgb565) -> Self {
        self.track_color = track;
        self.needle_color = needle;
        self
    }

    /// Add a colored threshold band (e.g. green normal range, red warning zone).
    pub fn add_threshold(&mut self, min_val: f32, max_val: f32, color: Rgb565) -> bool {
        if self.threshold_count < MAX_THRESHOLDS {
            self.thresholds[self.threshold_count] = Some(GaugeThreshold {
                min_val,
                max_val,
                color,
            });
            self.threshold_count += 1;
            true
        } else {
            false
        }
    }

    /// Render the rotary arc gauge into `bounds`.
    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        if bounds.w < 4 || bounds.h < 4 {
            return Ok(());
        }

        let cx = bounds.x + (bounds.w as i32 / 2);
        let cy = bounds.y + (bounds.h as i32 / 2);
        let r = (bounds.w.min(bounds.h) / 2).saturating_sub(self.thickness as u32 + 1);

        // 1. Draw base track
        ctx.stroke_arc_styled(
            cx,
            cy,
            r,
            self.start_deg,
            self.end_deg,
            StrokeStyle::new(self.track_color).with_width(self.thickness),
        )?;

        // 2. Draw threshold bands
        let range = (self.max - self.min).max(1e-5);
        let sweep = (self.end_deg - self.start_deg) as f32;

        for t in self.thresholds.iter().take(self.threshold_count).flatten() {
            let frac_start = ((t.min_val - self.min) / range).clamp(0.0, 1.0);
            let frac_end = ((t.max_val - self.min) / range).clamp(0.0, 1.0);
            let b_start = self.start_deg + (frac_start * sweep) as i32;
            let b_end = self.start_deg + (frac_end * sweep) as i32;
            if b_start != b_end {
                ctx.stroke_arc_styled(
                    cx,
                    cy,
                    r,
                    b_start,
                    b_end,
                    StrokeStyle::new(t.color).with_width(self.thickness),
                )?;
            }
        }

        // 3. Draw needle
        let norm = ((self.value - self.min) / range).clamp(0.0, 1.0);
        let needle_deg = self.start_deg as f32 + norm * sweep;
        let rad = needle_deg * (core::f32::consts::PI / 180.0);
        let needle_len = r as f32 * 0.85;

        let nx = cx + (needle_len * rad.cos()) as i32;
        let ny = cy + (needle_len * rad.sin()) as i32;

        ctx.draw_line(cx, cy, nx, ny, self.needle_color)?;
        ctx.fill_circle(cx, cy, 2, self.hub_color)?;

        Ok(())
    }
}

impl<const MAX_THRESHOLDS: usize> Widget for ArcGaugeWidget<MAX_THRESHOLDS> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Value => Some(PropertyValue::Float(self.value)),
            PropertyKey::Min => Some(PropertyValue::Float(self.min)),
            PropertyKey::Max => Some(PropertyValue::Float(self.max)),
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
                self.value = v.clamp(self.min, self.max);
                Ok(())
            }
            (PropertyKey::Min, PropertyValue::Float(m)) => {
                self.min = m;
                Ok(())
            }
            (PropertyKey::Max, PropertyValue::Float(m)) => {
                self.max = m;
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
    use embedded_graphics_core::pixelcolor::WebColors;

    #[test]
    fn test_arc_gauge_properties() {
        let mut gauge = ArcGaugeWidget::<3>::new(25.0, 0.0, 100.0);
        assert_eq!(
            gauge.get_property(PropertyKey::Value),
            Some(PropertyValue::Float(25.0))
        );

        assert!(
            gauge
                .set_property(PropertyKey::Value, PropertyValue::Float(75.0))
                .is_ok()
        );
        assert_eq!(
            gauge.get_property(PropertyKey::Value),
            Some(PropertyValue::Float(75.0))
        );
    }

    #[test]
    fn test_arc_gauge_render() {
        let mut gauge = ArcGaugeWidget::<2>::new(50.0, 0.0, 100.0);
        gauge.add_threshold(0.0, 60.0, Rgb565::new(0, 63, 0));
        gauge.add_threshold(60.0, 100.0, Rgb565::new(31, 0, 0));

        let mut fb = Framebuffer::<900>::new(30, 30);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 30, 30));
        assert!(gauge.render(&mut ctx, Rect::new(0, 0, 30, 30)).is_ok());

        // Hub pixel at center (15, 15) should be hub_color (WHITE)
        assert_eq!(fb.pixels()[15 * 30 + 15], Rgb565::new(31, 63, 31));
    }

    #[test]
    fn test_progress_bar_properties_and_gauges_extra_paths() {
        let mut bar = ProgressBarWidget::new(1.5);
        assert_eq!(bar.value, 1.0);
        assert_eq!(
            bar.get_property(PropertyKey::Progress),
            Some(PropertyValue::Float(1.0))
        );
        assert!(
            bar.set_property(PropertyKey::Value, PropertyValue::Float(-1.0))
                .is_ok()
        );
        assert_eq!(bar.value, 0.0);
        assert!(
            bar.set_property(PropertyKey::Min, PropertyValue::Float(0.0))
                .is_err()
        );

        let mut gauge = ArcGaugeWidget::<2>::default()
            .with_angles(180, 360)
            .with_colors(Rgb565::new(1, 2, 3), Rgb565::new(4, 5, 6));
        assert_eq!(gauge.start_deg, 180);
        assert_eq!(gauge.end_deg, 360);
        assert!(gauge.add_threshold(10.0, 20.0, Rgb565::CSS_GREEN));
        assert!(gauge.add_threshold(20.0, 30.0, Rgb565::CSS_RED));
        assert!(!gauge.add_threshold(30.0, 40.0, Rgb565::CSS_BLUE));

        assert!(
            gauge
                .set_property(PropertyKey::Min, PropertyValue::Float(-10.0))
                .is_ok()
        );
        assert!(
            gauge
                .set_property(PropertyKey::Max, PropertyValue::Float(110.0))
                .is_ok()
        );
        assert!(
            gauge
                .set_property(PropertyKey::Offset, PropertyValue::Usize(0))
                .is_err()
        );

        let mut fb = Framebuffer::<900>::new(30, 30);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 30, 30));
        assert!(gauge.render(&mut ctx, Rect::new(0, 0, 30, 30)).is_ok());
        assert!(gauge.render(&mut ctx, Rect::new(0, 0, 2, 2)).is_ok());

        let mut equal = ArcGaugeWidget::<1>::new(50.0, 0.0, 100.0);
        assert!(equal.add_threshold(50.0, 50.0, Rgb565::CSS_RED));
        assert!(equal.render(&mut ctx, Rect::new(0, 0, 30, 30)).is_ok());
    }
}
