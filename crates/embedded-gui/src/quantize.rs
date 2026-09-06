//! Low-color quantization, Bayer ordered dithering, and dirty band optimization.
//!
//! Enables smooth rendering of gradients, images, and anti-aliased geometry onto
//! monochrome OLEDs (1-bit), grayscale e-paper (2-bit / 4-bit), and RGB565 displays without banding.

use embedded_graphics_core::pixelcolor::Rgb565;

use crate::geometry::Rect;

/// Standard 4x4 Bayer dithering threshold matrix (values scaled to `0..=240`).
pub const BAYER_4X4: [[u8; 4]; 4] = [
    [0, 128, 32, 160],
    [192, 64, 224, 96],
    [48, 176, 16, 144],
    [240, 112, 208, 80],
];

/// Standard 2x2 Bayer dithering threshold matrix.
pub const BAYER_2X2: [[u8; 2]; 2] = [[0, 128], [192, 64]];

/// Quantizes an 8-bit grayscale intensity into 1-bit monochrome using 4x4 Bayer ordered dithering.
///
/// Returns `true` for ink/lit pixels, `false` for background.
#[inline]
pub fn bayer_1bit(x: u32, y: u32, intensity: u8) -> bool {
    let threshold = BAYER_4X4[(y % 4) as usize][(x % 4) as usize];
    intensity > threshold
}

/// Quantizes an 8-bit grayscale intensity into 2-bit grayscale (`0..=3`, e.g. for e-paper)
/// using ordered dithering.
#[inline]
pub fn bayer_2bit(x: u32, y: u32, intensity: u8) -> u8 {
    let threshold = BAYER_4X4[(y % 4) as usize][(x % 4) as usize] / 4;
    let scaled = (intensity as u16 * 3 + threshold as u16) / 255;
    scaled.min(3) as u8
}

/// Quantizes an 8-bit grayscale intensity into 4-bit grayscale (`0..=15`) using ordered dithering.
#[inline]
pub fn bayer_4bit(x: u32, y: u32, intensity: u8) -> u8 {
    let threshold = BAYER_4X4[(y % 4) as usize][(x % 4) as usize] / 16;
    let scaled = (intensity as u16 * 15 + threshold as u16) / 255;
    scaled.min(15) as u8
}

/// Dithers 24-bit RGB888 components down to 16-bit RGB565 to eliminate visible color banding.
#[inline]
pub fn bayer_dither_rgb565(x: u32, y: u32, r8: u8, g8: u8, b8: u8) -> Rgb565 {
    let dither = BAYER_4X4[(y % 4) as usize][(x % 4) as usize];

    // R: 8 bits to 5 bits (scale 31/255)
    let r_bias = dither >> 5;
    let r5 = ((r8 as u16 * 31 + r_bias as u16) / 255).min(31) as u8;

    // G: 8 bits to 6 bits (scale 63/255)
    let g_bias = dither >> 6;
    let g6 = ((g8 as u16 * 63 + g_bias as u16) / 255).min(63) as u8;

    // B: 8 bits to 5 bits (scale 31/255)
    let b_bias = dither >> 5;
    let b5 = ((b8 as u16 * 31 + b_bias as u16) / 255).min(31) as u8;

    Rgb565::new(r5, g6, b5)
}

/// Tracks dirty vertical bands to skip rendering and SPI bus transmission for unmodified bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyBandAccumulator {
    band_height: u32,
    total_height: u32,
    dirty_mask: u32,
}

impl DirtyBandAccumulator {
    /// Creates a new band tracker for a screen of `total_height` partitioned into bands of `band_height`.
    pub const fn new(band_height: u32, total_height: u32) -> Self {
        Self {
            band_height: if band_height == 0 { 16 } else { band_height },
            total_height,
            dirty_mask: 0,
        }
    }

    /// Total number of bands across the display height (up to 32).
    pub const fn total_bands(&self) -> usize {
        let count = self.total_height.div_ceil(self.band_height);
        if count > 32 { 32 } else { count as usize }
    }

    /// Marks the bands overlapping `rect` as dirty.
    pub fn mark_dirty_rect(&mut self, rect: Rect) {
        if rect.is_empty() || rect.bottom() <= 0 || rect.y >= self.total_height as i32 {
            return;
        }

        let start_y = rect.y.max(0) as u32;
        let end_y = (rect.bottom().max(0) as u32).min(self.total_height);

        let first_band = (start_y / self.band_height).min(31) as usize;
        let last_band = ((end_y.saturating_sub(1)) / self.band_height).min(31) as usize;

        for b in first_band..=last_band {
            self.dirty_mask |= 1 << b;
        }
    }

    /// Returns `true` if band `band_idx` contains modified pixels.
    pub const fn is_band_dirty(&self, band_idx: usize) -> bool {
        if band_idx >= 32 {
            false
        } else {
            (self.dirty_mask & (1 << band_idx)) != 0
        }
    }

    /// Total count of dirty bands this frame.
    pub const fn dirty_bands_count(&self) -> usize {
        self.dirty_mask.count_ones() as usize
    }

    /// Resets all dirty flags for the next frame.
    pub fn clear(&mut self) {
        self.dirty_mask = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bayer_1bit_monochrome() {
        // Pure black is always false
        assert!(!bayer_1bit(0, 0, 0));
        assert!(!bayer_1bit(1, 1, 0));

        // Pure white is always true
        assert!(bayer_1bit(0, 0, 255));
        assert!(bayer_1bit(2, 3, 255));

        // Mid gray produces half true, half false across 4x4 matrix
        let mut count = 0;
        for y in 0..4 {
            for x in 0..4 {
                if bayer_1bit(x, y, 128) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 8); // exactly 8 of 16 pixels lit
    }

    #[test]
    fn test_dirty_band_accumulator() {
        let mut acc = DirtyBandAccumulator::new(16, 240); // 15 bands
        assert_eq!(acc.total_bands(), 15);
        assert_eq!(acc.dirty_bands_count(), 0);

        // Mark a rect covering y=10..30 (overlaps band 0: 0..16, and band 1: 16..32)
        acc.mark_dirty_rect(Rect::new(0, 10, 100, 20));
        assert_eq!(acc.dirty_bands_count(), 2);
        assert!(acc.is_band_dirty(0));
        assert!(acc.is_band_dirty(1));
        assert!(!acc.is_band_dirty(2));

        acc.clear();
        assert_eq!(acc.dirty_bands_count(), 0);
    }
}
