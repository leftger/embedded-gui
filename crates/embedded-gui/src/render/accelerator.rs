//! Hardware 2D graphics acceleration trait hooks (STM32 DMA2D / Chrom-ART, ESP32 DMA, NXP PXP).
//! Allows bare-metal silicon accelerators to execute solid rectangle fills, block blits, and alpha
//! blending directly without burning CPU cycles.

use embedded_graphics_core::geometry::Point;
use embedded_graphics_core::pixelcolor::Rgb565;

use crate::geometry::Rect;
use crate::render::lerp_rgb565;

/// Trait for hardware-accelerated 2D blitting and rasterization engines.
pub trait Hardware2DAccelerator {
    /// Accelerated solid color rectangle fill (e.g. DMA2D Register-to-Memory mode).
    fn fill_rect(&mut self, dest: &mut [Rgb565], dest_stride: usize, rect: Rect, color: Rgb565);

    /// Accelerated memory-to-memory block transfer / blit (e.g. DMA2D M2M mode).
    fn copy_rect(
        &mut self,
        src: &[Rgb565],
        src_stride: usize,
        src_rect: Rect,
        dest: &mut [Rgb565],
        dest_stride: usize,
        dest_pos: Point,
    );

    /// Accelerated memory-to-memory block transfer with alpha blending.
    #[allow(clippy::too_many_arguments)]
    fn blend_rect(
        &mut self,
        fg: &[Rgb565],
        fg_stride: usize,
        fg_rect: Rect,
        fg_alpha: u8,
        bg: &mut [Rgb565],
        bg_stride: usize,
        bg_pos: Point,
    );
}

/// Fallback software implementation of [`Hardware2DAccelerator`] when no dedicated silicon engine is active.
#[derive(Clone, Copy, Debug, Default)]
pub struct Software2DAccelerator;

impl Hardware2DAccelerator for Software2DAccelerator {
    fn fill_rect(&mut self, dest: &mut [Rgb565], dest_stride: usize, rect: Rect, color: Rgb565) {
        let x0 = rect.x.max(0) as usize;
        let y0 = rect.y.max(0) as usize;
        let w = rect.w as usize;
        let h = rect.h as usize;

        for dy in 0..h {
            let row_idx = (y0 + dy) * dest_stride + x0;
            if row_idx + w <= dest.len() {
                dest[row_idx..row_idx + w].fill(color);
            }
        }
    }

    fn copy_rect(
        &mut self,
        src: &[Rgb565],
        src_stride: usize,
        src_rect: Rect,
        dest: &mut [Rgb565],
        dest_stride: usize,
        dest_pos: Point,
    ) {
        let sx0 = src_rect.x.max(0) as usize;
        let sy0 = src_rect.y.max(0) as usize;
        let dx0 = dest_pos.x.max(0) as usize;
        let dy0 = dest_pos.y.max(0) as usize;
        let w = src_rect.w as usize;
        let h = src_rect.h as usize;

        for dy in 0..h {
            let src_idx = (sy0 + dy) * src_stride + sx0;
            let dest_idx = (dy0 + dy) * dest_stride + dx0;
            if src_idx + w <= src.len() && dest_idx + w <= dest.len() {
                dest[dest_idx..dest_idx + w].copy_from_slice(&src[src_idx..src_idx + w]);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn blend_rect(
        &mut self,
        fg: &[Rgb565],
        fg_stride: usize,
        fg_rect: Rect,
        fg_alpha: u8,
        bg: &mut [Rgb565],
        bg_stride: usize,
        bg_pos: Point,
    ) {
        let fx0 = fg_rect.x.max(0) as usize;
        let fy0 = fg_rect.y.max(0) as usize;
        let bx0 = bg_pos.x.max(0) as usize;
        let by0 = bg_pos.y.max(0) as usize;
        let w = fg_rect.w as usize;
        let h = fg_rect.h as usize;

        for dy in 0..h {
            let fg_idx = (fy0 + dy) * fg_stride + fx0;
            let bg_idx = (by0 + dy) * bg_stride + bx0;
            for dx in 0..w {
                if fg_idx + dx < fg.len() && bg_idx + dx < bg.len() {
                    let fg_color = fg[fg_idx + dx];
                    let bg_color = bg[bg_idx + dx];
                    bg[bg_idx + dx] = lerp_rgb565(bg_color, fg_color, fg_alpha);
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
    fn test_software_accelerator_fill_and_copy() {
        let mut dest = [Rgb565::BLACK; 64];
        let mut accel = Software2DAccelerator;

        // Fill a 4x4 rect at (2, 2) with stride 8
        accel.fill_rect(&mut dest, 8, Rect::new(2, 2, 4, 4), Rgb565::WHITE);

        assert_eq!(dest[2 * 8 + 2], Rgb565::WHITE);
        assert_eq!(dest[2 * 8 + 5], Rgb565::WHITE);
        assert_eq!(dest[0], Rgb565::BLACK);

        // Blit from dest to another buffer
        let mut target = [Rgb565::BLACK; 64];
        accel.copy_rect(
            &dest,
            8,
            Rect::new(2, 2, 4, 4),
            &mut target,
            8,
            Point::new(0, 0),
        );

        assert_eq!(target[0], Rgb565::WHITE);
        assert_eq!(target[3], Rgb565::WHITE);
    }
}
