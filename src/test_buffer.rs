use core::convert::Infallible;

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
};

use heapless::Vec;

use crate::{geometry::Rect, present::PresentRegion, render::BlendMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestBuffer {
    size: Size,
    pixels: std::vec::Vec<Rgb565>,
}

impl TestBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            size: Size::new(width, height),
            pixels: std::vec![Rgb565::BLACK; width.saturating_mul(height) as usize],
        }
    }

    pub fn pixel_at(&self, x: i32, y: i32) -> Option<Rgb565> {
        if x < 0 || y < 0 || x >= self.size.width as i32 || y >= self.size.height as i32 {
            return None;
        }
        self.pixels
            .get(y as usize * self.size.width as usize + x as usize)
            .copied()
    }

    pub fn clear_color(&mut self, color: Rgb565) {
        for pixel in &mut self.pixels {
            *pixel = color;
        }
    }

    pub fn count_color(&self, color: Rgb565) -> usize {
        self.pixels.iter().filter(|&&pixel| pixel == color).count()
    }

    pub fn has_non_background_pixel(&self) -> bool {
        self.pixels.iter().any(|&pixel| pixel != Rgb565::BLACK)
    }

    pub fn assert_non_empty_rect(&self, rect: Rect) {
        let clipped = rect.intersection(Rect::new(0, 0, self.size.width, self.size.height));
        for y in clipped.y..clipped.bottom() {
            for x in clipped.x..clipped.right() {
                if self
                    .pixel_at(x, y)
                    .is_some_and(|pixel| pixel != Rgb565::BLACK)
                {
                    return;
                }
            }
        }
        panic!("expected non-background pixel in {:?}", rect);
    }

    pub fn digest(&self) -> u64 {
        self.pixels.iter().enumerate().fold(0u64, |acc, (idx, &c)| {
            acc.wrapping_mul(16_777_619)
                ^ idx as u64
                ^ ((c.r() as u64) << 32)
                ^ ((c.g() as u64) << 40)
                ^ ((c.b() as u64) << 48)
        })
    }

    pub fn assert_digest_eq(&self, expected: u64, label: &str) {
        let actual = self.digest();
        assert_eq!(
            actual, expected,
            "visual digest mismatch for {}: expected {expected:#x}, got {actual:#x}",
            label
        );
    }

    pub fn diff_bounding_region(&self, previous: &Self) -> Option<PresentRegion> {
        if self.size != previous.size {
            return Some(PresentRegion::new(
                0,
                0,
                self.size.width as usize,
                self.size.height as usize,
            ));
        }

        let mut bounds = Rect::empty();
        for y in 0..self.size.height as i32 {
            for x in 0..self.size.width as i32 {
                if self.pixel_at(x, y) != previous.pixel_at(x, y) {
                    let pixel = Rect::new(x, y, 1, 1);
                    bounds = if bounds.is_empty() {
                        pixel
                    } else {
                        bounds.union(pixel)
                    };
                }
            }
        }

        (!bounds.is_empty()).then_some(bounds.into())
    }

    pub fn diff_regions<const N: usize>(&self, previous: &Self) -> Vec<PresentRegion, N> {
        let mut regions = Vec::new();
        let Some(bounds) = self.diff_bounding_region(previous) else {
            return regions;
        };

        if self.size != previous.size {
            let _ = regions.push(bounds);
            return regions;
        }

        for y in 0..self.size.height as i32 {
            let mut min_x = self.size.width as i32;
            let mut max_x = -1;
            for x in 0..self.size.width as i32 {
                if self.pixel_at(x, y) != previous.pixel_at(x, y) {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                }
            }

            if max_x >= min_x {
                let row =
                    PresentRegion::new(min_x as usize, y as usize, (max_x - min_x + 1) as usize, 1);
                if regions.push(row).is_err() {
                    regions.clear();
                    let _ = regions.push(bounds);
                    return regions;
                }
            }
        }

        regions
    }

    pub fn composite_from(&mut self, overlay: &Self, mode: BlendMode, opacity: u8) {
        if self.size != overlay.size {
            return;
        }
        if opacity == 0 {
            return;
        }
        for (idx, src) in overlay.pixels.iter().copied().enumerate() {
            if src == Rgb565::BLACK {
                continue;
            }
            let dst = self.pixels[idx];
            let blended = blend_pixel(src, dst, mode);
            self.pixels[idx] = lerp_pixel(dst, blended, opacity);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerCanvas {
    inner: TestBuffer,
}

impl LayerCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            inner: TestBuffer::new(width, height),
        }
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.inner.clear_color(color);
    }

    pub fn target_mut(&mut self) -> &mut TestBuffer {
        &mut self.inner
    }

    pub fn composite_into(&self, target: &mut TestBuffer, mode: BlendMode, opacity: u8) {
        target.composite_from(&self.inner, mode, opacity);
    }
}

fn lerp_pixel(a: Rgb565, b: Rgb565, t: u8) -> Rgb565 {
    let t = t as u16;
    let inv = 255u16.saturating_sub(t);
    let r = ((a.r() as u16 * inv) + (b.r() as u16 * t)) / 255;
    let g = ((a.g() as u16 * inv) + (b.g() as u16 * t)) / 255;
    let bb = ((a.b() as u16 * inv) + (b.b() as u16 * t)) / 255;
    Rgb565::new(r as u8, g as u8, bb as u8)
}

fn blend_pixel(src: Rgb565, dst: Rgb565, mode: BlendMode) -> Rgb565 {
    match mode {
        BlendMode::Normal => src,
        BlendMode::Add => Rgb565::new(
            src.r().saturating_add(dst.r()),
            src.g().saturating_add(dst.g()),
            src.b().saturating_add(dst.b()),
        ),
        BlendMode::Multiply => Rgb565::new(
            ((src.r() as u16 * dst.r() as u16) / 31) as u8,
            ((src.g() as u16 * dst.g() as u16) / 63) as u8,
            ((src.b() as u16 * dst.b() as u16) / 31) as u8,
        ),
        BlendMode::Screen => Rgb565::new(
            (31 - ((31 - src.r() as u16) * (31 - dst.r() as u16) / 31)) as u8,
            (63 - ((63 - src.g() as u16) * (63 - dst.g() as u16) / 63)) as u8,
            (31 - ((31 - src.b() as u16) * (31 - dst.b() as u16) / 31)) as u8,
        ),
    }
}

impl DrawTarget for TestBuffer {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x < 0
                || point.y < 0
                || point.x >= self.size.width as i32
                || point.y >= self.size.height as i32
            {
                continue;
            }
            let idx = point.y as usize * self.size.width as usize + point.x as usize;
            if let Some(pixel) = self.pixels.get_mut(idx) {
                *pixel = color;
            }
        }
        Ok(())
    }
}

impl OriginDimensions for TestBuffer {
    fn size(&self) -> Size {
        self.size
    }
}
