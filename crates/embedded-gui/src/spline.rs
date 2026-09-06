//! Catmull-Rom and Cardinal cubic spline curves adapted from `bevy_curve::cubic_splines`.
//!
//! Provides smooth interpolation passing directly through control points, ideal for
//! sensor plots, smoothed vector paths, and continuous animation tracks in `#![no_std]`.

use embedded_graphics_core::geometry::Point;

/// A cubic spline evaluator using the Catmull-Rom formulation.
///
/// In a Catmull-Rom spline, the curve passes directly through every control point
/// (unlike Bézier curves where intermediate control points only act as attractors).
#[derive(Debug, Clone, Copy)]
pub struct CatmullRomSpline<'a> {
    points: &'a [Point],
}

impl<'a> CatmullRomSpline<'a> {
    /// Creates a new Catmull-Rom spline from a slice of control points.
    /// Returns `None` if the slice contains fewer than 2 points.
    pub const fn new(points: &'a [Point]) -> Option<Self> {
        if points.len() < 2 {
            None
        } else {
            Some(Self { points })
        }
    }

    /// Evaluates the spline at normalized parameter `t` in `[0.0, 1.0]`.
    pub fn evaluate(&self, t: f32) -> Point {
        self.evaluate_cardinal(t, 0.0)
    }

    /// Evaluates a Cardinal spline with a given `tension` parameter (-1.0 to 1.0, 0.0 is standard Catmull-Rom).
    pub fn evaluate_cardinal(&self, t: f32, tension: f32) -> Point {
        let n = self.points.len();
        if n == 0 {
            return Point::zero();
        }
        if n == 1 {
            return self.points[0];
        }

        let t = t.clamp(0.0, 1.0);
        let num_segments = (n - 1) as f32;
        let scaled_t = t * num_segments;
        let segment_idx = (scaled_t as usize).min(n - 2);
        let seg_t = scaled_t - segment_idx as f32;

        let p0 = if segment_idx == 0 {
            // Extrapolate phantom point before start: 2 * p1 - p2
            Point::new(
                2 * self.points[0].x - self.points[1].x,
                2 * self.points[0].y - self.points[1].y,
            )
        } else {
            self.points[segment_idx - 1]
        };

        let p1 = self.points[segment_idx];
        let p2 = self.points[segment_idx + 1];

        let p3 = if segment_idx + 2 >= n {
            // Extrapolate phantom point after end: 2 * p2 - p1
            Point::new(
                2 * self.points[n - 1].x - self.points[n - 2].x,
                2 * self.points[n - 1].y - self.points[n - 2].y,
            )
        } else {
            self.points[segment_idx + 2]
        };

        let s = (1.0 - tension) * 0.5;

        let t2 = seg_t * seg_t;
        let t3 = t2 * seg_t;

        // Basis functions for Cardinal/Catmull-Rom
        let h1 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h2 = -2.0 * t3 + 3.0 * t2;
        let h3 = t3 - 2.0 * t2 + seg_t;
        let h4 = t3 - t2;

        let m1_x = s * (p2.x - p0.x) as f32;
        let m1_y = s * (p2.y - p0.y) as f32;
        let m2_x = s * (p3.x - p1.x) as f32;
        let m2_y = s * (p3.y - p1.y) as f32;

        let x = h1 * p1.x as f32 + h2 * p2.x as f32 + h3 * m1_x + h4 * m2_x;
        let y = h1 * p1.y as f32 + h2 * p2.y as f32 + h3 * m1_y + h4 * m2_y;

        Point::new((x + 0.5) as i32, (y + 0.5) as i32)
    }

    /// Evaluates `samples` evenly distributed points along the spline into a fixed buffer.
    pub fn sample_into(&self, buffer: &mut [Point]) {
        if buffer.is_empty() {
            return;
        }
        if buffer.len() == 1 {
            buffer[0] = self.evaluate(0.0);
            return;
        }
        let denom = (buffer.len() - 1) as f32;
        for (i, p) in buffer.iter_mut().enumerate() {
            *p = self.evaluate(i as f32 / denom);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catmull_rom_passes_through_control_points() {
        let points = [
            Point::new(0, 0),
            Point::new(10, 20),
            Point::new(20, 10),
            Point::new(30, 30),
        ];
        let spline = CatmullRomSpline::new(&points).unwrap();

        // Start point t=0.0
        assert_eq!(spline.evaluate(0.0), Point::new(0, 0));

        // Point 1 at t=1/3
        let p1 = spline.evaluate(1.0 / 3.0);
        assert!((p1.x - 10).abs() <= 1 && (p1.y - 20).abs() <= 1);

        // Point 2 at t=2/3
        let p2 = spline.evaluate(2.0 / 3.0);
        assert!((p2.x - 20).abs() <= 1 && (p2.y - 10).abs() <= 1);

        // End point t=1.0
        assert_eq!(spline.evaluate(1.0), Point::new(30, 30));
    }

    #[test]
    fn test_catmull_rom_sampling() {
        let points = [Point::new(0, 0), Point::new(100, 100)];
        let spline = CatmullRomSpline::new(&points).unwrap();

        let mut samples = [Point::zero(); 5];
        spline.sample_into(&mut samples);

        assert_eq!(samples[0], Point::new(0, 0));
        assert_eq!(samples[2], Point::new(50, 50));
        assert_eq!(samples[4], Point::new(100, 100));
    }
}
