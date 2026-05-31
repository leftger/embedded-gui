use core::convert::Infallible;

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
};

use heapless::Vec;

use crate::{geometry::Rect, present::PresentRegion};

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
