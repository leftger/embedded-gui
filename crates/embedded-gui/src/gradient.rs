//! Multi-stop linear gradient rasterizer adapted from `bevy_ui::gradients`.
//!
//! Provides zero-allocation, fixed-capacity color ramps with deterministic
//! integer-based interpolation for `#![no_std]` MCU displays and line buffers.

use embedded_graphics_core::pixelcolor::Rgb565;

use crate::colors::ColorOps;
pub use crate::style::GradientDirection;

/// A color stop along a gradient line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorStop {
    /// Target color at this stop.
    pub color: Rgb565,
    /// Normalized position along the gradient in `[0, 255]`.
    pub position: u8,
}

impl ColorStop {
    pub const fn new(color: Rgb565, position: u8) -> Self {
        Self { color, position }
    }
}

/// A multi-stop linear gradient holding up to `CAP` color stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiStopGradient<const CAP: usize = 4> {
    stops: [ColorStop; CAP],
    count: usize,
    pub direction: GradientDirection,
}

impl<const CAP: usize> MultiStopGradient<CAP> {
    /// Creates a new empty gradient with a given direction.
    pub const fn new(direction: GradientDirection) -> Self {
        Self {
            stops: [ColorStop::new(Rgb565::new(0, 0, 0), 0); CAP],
            count: 0,
            direction,
        }
    }

    /// Adds a color stop to the gradient. Returns false if capacity `CAP` is reached.
    pub fn add_stop(&mut self, stop: ColorStop) -> bool {
        if self.count >= CAP {
            return false;
        }
        // Insert maintaining sorted order by position
        let mut idx = self.count;
        while idx > 0 && self.stops[idx - 1].position > stop.position {
            self.stops[idx] = self.stops[idx - 1];
            idx -= 1;
        }
        self.stops[idx] = stop;
        self.count += 1;
        true
    }

    /// Number of active color stops.
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Whether any color stops are configured.
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Active color stops as a slice.
    pub fn stops(&self) -> &[ColorStop] {
        &self.stops[..self.count]
    }

    /// Samples the gradient at normalized position `t` in `[0, 255]`.
    pub fn sample_at(&self, t: u8) -> Rgb565 {
        if self.count == 0 {
            return Rgb565::new(0, 0, 0);
        }
        if self.count == 1 || t <= self.stops[0].position {
            return self.stops[0].color;
        }
        if t >= self.stops[self.count - 1].position {
            return self.stops[self.count - 1].color;
        }

        // Find the bounding segment
        for i in 0..self.count - 1 {
            let s0 = self.stops[i];
            let s1 = self.stops[i + 1];
            if t >= s0.position && t <= s1.position {
                let span = (s1.position - s0.position) as u16;
                if span == 0 {
                    return s0.color;
                }
                let local = (t - s0.position) as u16;
                let factor = ((local * 255) / span) as u8;
                return s0.color.lerp(s1.color, factor);
            }
        }

        self.stops[self.count - 1].color
    }

    /// Fills a row or column slice with sampled gradient pixels.
    pub fn fill_span(&self, span_length: usize, out_buffer: &mut [Rgb565]) {
        if out_buffer.is_empty() {
            return;
        }
        let len = out_buffer.len().min(span_length);
        if len == 1 {
            out_buffer[0] = self.sample_at(0);
            return;
        }
        let denom = (len - 1) as u32;
        for (i, pixel) in out_buffer[..len].iter_mut().enumerate() {
            let t = ((i as u32 * 255) / denom) as u8;
            *pixel = self.sample_at(t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::pixelcolor::RgbColor;

    #[test]
    fn test_multi_stop_gradient_sampling() {
        let red = Rgb565::new(31, 0, 0);
        let green = Rgb565::new(0, 63, 0);
        let blue = Rgb565::new(0, 0, 31);

        let mut grad = MultiStopGradient::<4>::new(GradientDirection::Horizontal);
        assert!(grad.add_stop(ColorStop::new(red, 0)));
        assert!(grad.add_stop(ColorStop::new(green, 128)));
        assert!(grad.add_stop(ColorStop::new(blue, 255)));

        assert_eq!(grad.sample_at(0), red);
        assert_eq!(grad.sample_at(128), green);
        assert_eq!(grad.sample_at(255), blue);

        // Clamping checks
        assert_eq!(grad.sample_at(0), red);
        assert_eq!(grad.sample_at(255), blue);

        // Midpoint between red and green (t=64)
        let mid = grad.sample_at(64);
        assert!(mid.r() > 10 && mid.g() > 20);
    }

    #[test]
    fn test_fill_span() {
        let red = Rgb565::new(31, 0, 0);
        let blue = Rgb565::new(0, 0, 31);

        let mut grad = MultiStopGradient::<2>::new(GradientDirection::Horizontal);
        grad.add_stop(ColorStop::new(red, 0));
        grad.add_stop(ColorStop::new(blue, 255));

        let mut buf = [Rgb565::new(0, 0, 0); 3];
        grad.fill_span(3, &mut buf);

        assert_eq!(buf[0], red);
        assert_eq!(buf[2], blue);
        assert!(buf[1].r() > 10 && buf[1].b() > 10);
    }
}
