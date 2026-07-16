//! A RAM-backed, readback-capable framebuffer.
//!
//! [`Framebuffer`] implements [`DrawTarget`] **and** [`PixelRead`], so rendering
//! into it unlocks true per-pixel alpha compositing (`RenderCtx::*_true_alpha`)
//! instead of the ordered-dither approximation used for write-only displays.
//! The typical pattern is a software double buffer: render the UI into a
//! `Framebuffer`, then blit it to the physical panel once per frame.
//!
//! Storage is a fixed `[Rgb565; N]` array (no allocator). Pick `N >= W * H`;
//! put the buffer behind a `StaticCell` (or on the stack for host tooling).

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb565, RgbColor},
};

use crate::render::PixelRead;

/// A fixed-capacity RGB565 framebuffer. `N` is the backing array length and
/// must be at least `width * height`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Framebuffer<const N: usize> {
    pixels: [Rgb565; N],
    width: u32,
    height: u32,
}

impl<const N: usize> Framebuffer<N> {
    /// Create a `width × height` framebuffer cleared to black.
    ///
    /// Panics if `width * height > N`.
    pub fn new(width: u32, height: u32) -> Self {
        assert!(
            (width as usize) * (height as usize) <= N,
            "Framebuffer backing array too small for width * height",
        );
        Self {
            pixels: [Rgb565::BLACK; N],
            width,
            height,
        }
    }

    /// Fill the whole framebuffer with a single color.
    pub fn clear_color(&mut self, color: Rgb565) {
        let len = self.len();
        for p in self.pixels[..len].iter_mut() {
            *p = color;
        }
    }

    /// The active pixels, row-major, `width * height` long. Handy for blitting
    /// to a physical display.
    pub fn pixels(&self) -> &[Rgb565] {
        &self.pixels[..self.len()]
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    fn len(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    #[inline]
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return None;
        }
        Some((y as usize) * (self.width as usize) + (x as usize))
    }
}

impl<const N: usize> OriginDimensions for Framebuffer<N> {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl<const N: usize> DrawTarget for Framebuffer<N> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if let Some(idx) = self.index(point.x, point.y) {
                self.pixels[idx] = color;
            }
        }
        Ok(())
    }
}

impl<const N: usize> PixelRead for Framebuffer<N> {
    fn get_pixel(&self, point: Point) -> Rgb565 {
        match self.index(point.x, point.y) {
            Some(idx) => self.pixels[idx],
            None => Rgb565::BLACK,
        }
    }
}
