//! Slint-style scanline / span-buffered software rasterizer with fixed-point antialiasing.
//!
//! Evaluates horizontal scanline spans for rounded rectangles, pills, and borders using
//! fixed-point integer arithmetic and integer square root (`isqrt`), computing subpixel
//! boundary coverage (0..=255) for smooth rendering without floating-point overhead.

use embedded_graphics_core::pixelcolor::Rgb565;

use crate::geometry::Rect;
use crate::render::lerp_rgb565;

/// Subpixel coverage and span for a single scanline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanlineSpan {
    /// Starting X coordinate of the interior solid span.
    pub x_start: i32,
    /// Ending X coordinate of the interior solid span (exclusive).
    pub x_end: i32,
    /// Left boundary subpixel coverage (0..=255) at `x_start - 1`.
    pub left_coverage: u8,
    /// Right boundary subpixel coverage (0..=255) at `x_end`.
    pub right_coverage: u8,
}

/// Fixed-point rounded rectangle span evaluator.
pub struct FixedSpanRasterizer;

impl FixedSpanRasterizer {
    /// Computes the horizontal span and edge coverage for a rounded rectangle on scanline `y`.
    ///
    /// If `y` is outside the rectangle, returns `None`.
    pub fn eval_rounded_rect_span(rect: Rect, radius: u8, y: i32) -> Option<ScanlineSpan> {
        if y < rect.y || y >= rect.y + rect.h as i32 {
            return None;
        }

        let w = rect.w as i32;
        let h = rect.h as i32;
        let r = (radius as i32).min(w / 2).min(h / 2);

        if r == 0 {
            // Sharp corners: full span, 0 boundary fringe
            return Some(ScanlineSpan {
                x_start: rect.x,
                x_end: rect.x + w,
                left_coverage: 0,
                right_coverage: 0,
            });
        }

        let rel_y = y - rect.y;
        let corner_dy = if rel_y < r {
            r - 1 - rel_y
        } else if rel_y >= h - r {
            rel_y - (h - r)
        } else {
            -1 // Middle straight section
        };

        if corner_dy < 0 {
            // Straight vertical edge section
            Some(ScanlineSpan {
                x_start: rect.x,
                x_end: rect.x + w,
                left_coverage: 0,
                right_coverage: 0,
            })
        } else {
            // Corner section: evaluate circle chord dx = sqrt(r^2 - dy^2)
            // Using 4-bit shifted fixed point (16 units per pixel)
            let r_fixed = (r << 4) as u32;
            let dy_fixed = (corner_dy << 4) as u32 + 8; // center of scanline
            let r2 = r_fixed * r_fixed;
            let dy2 = dy_fixed * dy_fixed;

            let dx_fixed = if r2 > dy2 { (r2 - dy2).isqrt() } else { 0 };

            let corner_indent_fixed = r_fixed.saturating_sub(dx_fixed);
            let indent_pixels = (corner_indent_fixed >> 4) as i32;
            let subpixel_frac = (corner_indent_fixed & 0x0F) as u8;

            // Invert subpixel fraction into coverage: 0 frac => full pixel coverage
            let edge_cov = 255u8.saturating_sub(subpixel_frac * 16);

            let x_start = rect.x + indent_pixels + 1;
            let x_end = (rect.x + w - indent_pixels - 1).max(x_start);

            Some(ScanlineSpan {
                x_start,
                x_end,
                left_coverage: edge_cov,
                right_coverage: edge_cov,
            })
        }
    }

    /// Renders an antialiased rounded rectangle span directly into a scanline buffer slice.
    pub fn render_span_into_line(
        line: &mut [Rgb565],
        line_offset_x: i32,
        span: ScanlineSpan,
        color: Rgb565,
        opacity: u8,
    ) {
        let len = line.len() as i32;

        // Left fringe pixel
        if span.left_coverage > 0 {
            let lx = span.x_start - 1 - line_offset_x;
            if lx >= 0 && lx < len {
                let cov = ((span.left_coverage as u16 * opacity as u16) / 255) as u8;
                if cov > 0 {
                    let idx = lx as usize;
                    line[idx] = lerp_rgb565(line[idx], color, cov);
                }
            }
        }

        // Interior solid span
        let start_idx = (span.x_start - line_offset_x).clamp(0, len) as usize;
        let end_idx = (span.x_end - line_offset_x).clamp(0, len) as usize;

        if start_idx < end_idx {
            if opacity == 255 {
                line[start_idx..end_idx].fill(color);
            } else if opacity > 0 {
                for pixel in &mut line[start_idx..end_idx] {
                    *pixel = lerp_rgb565(*pixel, color, opacity);
                }
            }
        }

        // Right fringe pixel
        if span.right_coverage > 0 {
            let rx = span.x_end - line_offset_x;
            if rx >= 0 && rx < len {
                let cov = ((span.right_coverage as u16 * opacity as u16) / 255) as u8;
                if cov > 0 {
                    let idx = rx as usize;
                    line[idx] = lerp_rgb565(line[idx], color, cov);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::pixelcolor::RgbColor;

    #[test]
    fn test_fixed_span_evaluator_sharp() {
        let rect = Rect::new(10, 10, 50, 20);
        let span = FixedSpanRasterizer::eval_rounded_rect_span(rect, 0, 15).unwrap();
        assert_eq!(span.x_start, 10);
        assert_eq!(span.x_end, 60);
        assert_eq!(span.left_coverage, 0);
        assert_eq!(span.right_coverage, 0);
    }

    #[test]
    fn test_fixed_span_evaluator_rounded() {
        let rect = Rect::new(10, 10, 50, 20);
        // Middle line (y=20): straight section
        let mid_span = FixedSpanRasterizer::eval_rounded_rect_span(rect, 5, 20).unwrap();
        assert_eq!(mid_span.x_start, 10);
        assert_eq!(mid_span.x_end, 60);

        // Top corner line (y=10): corner section
        let top_span = FixedSpanRasterizer::eval_rounded_rect_span(rect, 5, 10).unwrap();
        assert!(top_span.x_start > 10);
        assert!(top_span.x_end < 60);
        assert!(top_span.left_coverage > 0);
    }

    #[test]
    fn test_render_span_into_line() {
        let mut line = [Rgb565::new(0, 0, 0); 20];
        let span = ScanlineSpan {
            x_start: 5,
            x_end: 15,
            left_coverage: 128,
            right_coverage: 128,
        };
        FixedSpanRasterizer::render_span_into_line(&mut line, 0, span, Rgb565::new(31, 0, 0), 255);

        // Center pixels should be full red
        assert_eq!(line[10], Rgb565::new(31, 0, 0));
        // Boundary fringe should be partially blended
        assert!(line[4].r() > 0 && line[4].r() < 31);
        assert!(line[15].r() > 0 && line[15].r() < 31);
        // Outside pixels should remain untouched
        assert_eq!(line[0], Rgb565::new(0, 0, 0));
    }
}
