//! Compact live telemetry sparkline trend chart.
//!
//! Provides ring-buffer history, min/max auto-scaling, polyline rendering, and baseline area fill
//! with zero dynamic heap allocations.

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx},
    style::Style,
    widget::{PropertyKey, PropertyValue, Widget},
};

/// Fixed-capacity ring-buffer sparkline chart widget for live telemetry display.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SparklineWidget<const CAP: usize = 32> {
    pub buffer: [i16; CAP],
    pub head: usize,
    pub count: usize,
    pub line_color: Rgb565,
    pub fill_color: Option<Rgb565>,
    pub min_val: Option<i16>,
    pub max_val: Option<i16>,
}

impl<const CAP: usize> Default for SparklineWidget<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> SparklineWidget<CAP> {
    /// Create a new empty sparkline widget.
    pub const fn new() -> Self {
        Self {
            buffer: [0; CAP],
            head: 0,
            count: 0,
            line_color: Rgb565::new(10, 63, 10), // Lime green
            fill_color: None,
            min_val: None,
            max_val: None,
        }
    }

    /// Push a new telemetry data point into the ring buffer in O(1).
    pub fn push(&mut self, val: i16) {
        if CAP == 0 {
            return;
        }
        self.buffer[self.head] = val;
        self.head = (self.head + 1) % CAP;
        if self.count < CAP {
            self.count += 1;
        }
    }

    /// Clear all data points.
    pub fn clear(&mut self) {
        self.head = 0;
        self.count = 0;
    }

    /// Configure explicit minimum and maximum value bounds instead of auto-scaling.
    pub const fn with_bounds(mut self, min_val: i16, max_val: i16) -> Self {
        self.min_val = Some(min_val);
        self.max_val = Some(max_val);
        self
    }

    /// Configure line color.
    pub const fn with_line_color(mut self, color: Rgb565) -> Self {
        self.line_color = color;
        self
    }

    /// Configure area fill color below the sparkline curve.
    pub const fn with_fill_color(mut self, color: Option<Rgb565>) -> Self {
        self.fill_color = color;
        self
    }

    /// Render sparkline within `bounds`.
    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        if self.count < 2 || bounds.w < 2 || bounds.h < 2 {
            return Ok(());
        }

        // Determine effective min/max
        let (min, max) = match (self.min_val, self.max_val) {
            (Some(mi), Some(ma)) => (mi, ma.max(mi + 1)),
            _ => {
                let mut mi = i16::MAX;
                let mut ma = i16::MIN;
                let start = if self.count < CAP { 0 } else { self.head };
                for i in 0..self.count {
                    let idx = (start + i) % CAP;
                    let v = self.buffer[idx];
                    if v < mi {
                        mi = v;
                    }
                    if v > ma {
                        ma = v;
                    }
                }
                if mi == ma {
                    ma = mi + 1;
                }
                (self.min_val.unwrap_or(mi), self.max_val.unwrap_or(ma))
            }
        };

        let range = (max - min) as f32;
        let baseline_y = bounds.bottom() - 1;
        let start_idx = if self.count < CAP { 0 } else { self.head };

        let num_points = self.count;
        let x_step = (bounds.w as f32 - 1.0) / (num_points as f32 - 1.0);

        let mut prev_pt: Option<(i32, i32)> = None;

        for i in 0..num_points {
            let idx = (start_idx + i) % CAP;
            let val = self.buffer[idx];
            let norm = ((val - min) as f32 / range).clamp(0.0, 1.0);

            let px = bounds.x + (i as f32 * x_step) as i32;
            let py = bounds.bottom() - 1 - (norm * (bounds.h as f32 - 1.0)) as i32;

            if let Some(fill) = self.fill_color {
                if baseline_y >= py {
                    ctx.draw_line(px, py, px, baseline_y, fill)?;
                }
            }

            if let Some((prev_x, prev_y)) = prev_pt {
                ctx.draw_line(prev_x, prev_y, px, py, self.line_color)?;
            }

            prev_pt = Some((px, py));
        }

        Ok(())
    }
}

impl<const CAP: usize> Widget for SparklineWidget<CAP> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Value => {
                if self.count == 0 {
                    None
                } else {
                    let last_idx = if self.head == 0 {
                        self.count - 1
                    } else {
                        self.head - 1
                    };
                    Some(PropertyValue::Int(self.buffer[last_idx] as i32))
                }
            }
            PropertyKey::Offset => Some(PropertyValue::Usize(self.count)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;
    use embedded_graphics_core::pixelcolor::RgbColor;

    #[test]
    fn test_sparkline_widget_push_and_bounds() {
        let mut sparkline = SparklineWidget::<4>::new();
        assert_eq!(sparkline.count, 0);

        sparkline.push(10);
        sparkline.push(20);
        sparkline.push(30);
        assert_eq!(sparkline.count, 3);
        assert_eq!(
            sparkline.get_property(PropertyKey::Value),
            Some(PropertyValue::Int(30))
        );

        sparkline.push(40);
        sparkline.push(50); // Wraps around
        assert_eq!(sparkline.count, 4);
        assert_eq!(
            sparkline.get_property(PropertyKey::Value),
            Some(PropertyValue::Int(50))
        );
    }

    #[test]
    fn test_sparkline_widget_render() {
        let mut sparkline = SparklineWidget::<5>::new()
            .with_line_color(Rgb565::RED)
            .with_fill_color(Some(Rgb565::BLUE));

        for &val in &[10, 30, 20, 50, 40] {
            sparkline.push(val);
        }

        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        assert!(sparkline.render(&mut ctx, Rect::new(0, 0, 20, 20)).is_ok());

        // Interior pixel below line should be filled with BLUE
        assert_eq!(fb.pixels()[15 * 20 + 4], Rgb565::BLUE);
        // Start pixel of line is RED
        assert_eq!(fb.pixels()[19 * 20], Rgb565::RED);
    }

    #[test]
    fn sparkline_clear_bounds_properties_and_small_render() {
        let mut spark = SparklineWidget::<4>::new()
            .with_bounds(-100, 100)
            .with_fill_color(None);
        assert_eq!(spark.min_val, Some(-100));
        assert_eq!(spark.max_val, Some(100));
        assert_eq!(
            spark.get_property(PropertyKey::Offset),
            Some(PropertyValue::Usize(0))
        );
        assert_eq!(spark.get_property(PropertyKey::Value), None);
        spark.push(5);
        spark.push(10);
        assert_eq!(
            spark.get_property(PropertyKey::Value),
            Some(PropertyValue::Int(10))
        );
        spark.clear();
        assert_eq!(spark.count, 0);

        let zero: SparklineWidget<0> = SparklineWidget::new();
        let mut zero_mut = zero;
        zero_mut.push(1);
        assert_eq!(zero_mut.count, 0);

        for v in [1, 2] {
            spark.push(v);
        }
        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        assert!(spark.render(&mut ctx, Rect::new(0, 0, 1, 20)).is_ok());
        assert!(spark.render(&mut ctx, Rect::new(0, 0, 20, 1)).is_ok());
        assert!(spark.render(&mut ctx, Rect::new(0, 0, 20, 20)).is_ok());
    }
}
