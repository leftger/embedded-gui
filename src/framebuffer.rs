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
    pixelcolor::{Gray8, GrayColor, Rgb565, RgbColor},
};

use crate::geometry::Rect;
use crate::render::PixelRead;

/// An 8-bit RGBA color representation (32-bit total).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rgba8888 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8888 {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb565(rgb: Rgb565, alpha: u8) -> Self {
        let r5 = rgb.r();
        let g6 = rgb.g();
        let b5 = rgb.b();
        let r8 = (r5 << 3) | (r5 >> 2);
        let g8 = (g6 << 2) | (g6 >> 4);
        let b8 = (b5 << 3) | (b5 >> 2);
        Self { r: r8, g: g8, b: b8, a: alpha }
    }

    pub fn to_rgb565(self) -> Rgb565 {
        Rgb565::new(self.r >> 3, self.g >> 2, self.b >> 3)
    }
}

impl embedded_graphics_core::pixelcolor::PixelColor for Rgba8888 {
    type Raw = embedded_graphics_core::pixelcolor::raw::RawU32;
}

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

    /// Mutable slice of active pixels.
    pub fn pixels_mut(&mut self) -> &mut [Rgb565] {
        let len = self.len();
        &mut self.pixels[..len]
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

    /// Apply Arm-2D Fast IIR Blur to the whole framebuffer.
    pub fn apply_iir_blur(&mut self, blur_degree: u8) {
        let rect = Rect::new(0, 0, self.width, self.height);
        self.blur_rect(rect, blur_degree);
    }

    /// Apply Arm-2D Fast IIR Blur to a sub-region `rect` of the framebuffer.
    pub fn blur_rect(&mut self, rect: Rect, blur_degree: u8) {
        if blur_degree == 0 || self.width == 0 || self.height == 0 {
            return;
        }

        let x0 = rect.x.max(0) as u32;
        let y0 = rect.y.max(0) as u32;
        let x1 = (rect.right() as u32).min(self.width);
        let y1 = (rect.bottom() as u32).min(self.height);

        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let alpha = 256 - (blur_degree as i32);
        let w = self.width as usize;

        // Pass 1: Horizontal Forward
        for y in y0..y1 {
            let row_start = y as usize * w;
            let idx0 = row_start + x0 as usize;
            let raw0 = self.pixels[idx0];
            let (r5, g6, b5) = (raw0.r(), raw0.g(), raw0.b());
            let mut acc_r = (((r5 << 3) | (r5 >> 2)) as i32) << 8;
            let mut acc_g = (((g6 << 2) | (g6 >> 4)) as i32) << 8;
            let mut acc_b = (((b5 << 3) | (b5 >> 2)) as i32) << 8;

            for x in x0..x1 {
                let idx = row_start + x as usize;
                let raw = self.pixels[idx];
                let (r5, g6, b5) = (raw.r(), raw.g(), raw.b());
                let r8 = ((r5 << 3) | (r5 >> 2)) as i32;
                let g8 = ((g6 << 2) | (g6 >> 4)) as i32;
                let b8 = ((b5 << 3) | (b5 >> 2)) as i32;

                acc_r += (((r8 << 8) - acc_r) * alpha) >> 8;
                acc_g += (((g8 << 8) - acc_g) * alpha) >> 8;
                acc_b += (((b8 << 8) - acc_b) * alpha) >> 8;

                let r_out = ((acc_r >> 8).clamp(0, 255) as u8) >> 3;
                let g_out = ((acc_g >> 8).clamp(0, 255) as u8) >> 2;
                let b_out = ((acc_b >> 8).clamp(0, 255) as u8) >> 3;
                self.pixels[idx] = Rgb565::new(r_out, g_out, b_out);
            }
        }

        // Pass 2: Horizontal Reverse
        for y in y0..y1 {
            let row_start = y as usize * w;
            let idx_last = row_start + (x1 - 1) as usize;
            let raw_last = self.pixels[idx_last];
            let (r5, g6, b5) = (raw_last.r(), raw_last.g(), raw_last.b());
            let mut acc_r = (((r5 << 3) | (r5 >> 2)) as i32) << 8;
            let mut acc_g = (((g6 << 2) | (g6 >> 4)) as i32) << 8;
            let mut acc_b = (((b5 << 3) | (b5 >> 2)) as i32) << 8;

            for x in (x0..x1).rev() {
                let idx = row_start + x as usize;
                let raw = self.pixels[idx];
                let (r5, g6, b5) = (raw.r(), raw.g(), raw.b());
                let r8 = ((r5 << 3) | (r5 >> 2)) as i32;
                let g8 = ((g6 << 2) | (g6 >> 4)) as i32;
                let b8 = ((b5 << 3) | (b5 >> 2)) as i32;

                acc_r += (((r8 << 8) - acc_r) * alpha) >> 8;
                acc_g += (((g8 << 8) - acc_g) * alpha) >> 8;
                acc_b += (((b8 << 8) - acc_b) * alpha) >> 8;

                let r_out = ((acc_r >> 8).clamp(0, 255) as u8) >> 3;
                let g_out = ((acc_g >> 8).clamp(0, 255) as u8) >> 2;
                let b_out = ((acc_b >> 8).clamp(0, 255) as u8) >> 3;
                self.pixels[idx] = Rgb565::new(r_out, g_out, b_out);
            }
        }

        // Pass 3: Vertical Forward
        for x in x0..x1 {
            let idx0 = y0 as usize * w + x as usize;
            let raw0 = self.pixels[idx0];
            let (r5, g6, b5) = (raw0.r(), raw0.g(), raw0.b());
            let mut acc_r = (((r5 << 3) | (r5 >> 2)) as i32) << 8;
            let mut acc_g = (((g6 << 2) | (g6 >> 4)) as i32) << 8;
            let mut acc_b = (((b5 << 3) | (b5 >> 2)) as i32) << 8;

            for y in y0..y1 {
                let idx = y as usize * w + x as usize;
                let raw = self.pixels[idx];
                let (r5, g6, b5) = (raw.r(), raw.g(), raw.b());
                let r8 = ((r5 << 3) | (r5 >> 2)) as i32;
                let g8 = ((g6 << 2) | (g6 >> 4)) as i32;
                let b8 = ((b5 << 3) | (b5 >> 2)) as i32;

                acc_r += (((r8 << 8) - acc_r) * alpha) >> 8;
                acc_g += (((g8 << 8) - acc_g) * alpha) >> 8;
                acc_b += (((b8 << 8) - acc_b) * alpha) >> 8;

                let r_out = ((acc_r >> 8).clamp(0, 255) as u8) >> 3;
                let g_out = ((acc_g >> 8).clamp(0, 255) as u8) >> 2;
                let b_out = ((acc_b >> 8).clamp(0, 255) as u8) >> 3;
                self.pixels[idx] = Rgb565::new(r_out, g_out, b_out);
            }
        }

        // Pass 4: Vertical Reverse
        for x in x0..x1 {
            let idx_last = (y1 - 1) as usize * w + x as usize;
            let raw_last = self.pixels[idx_last];
            let (r5, g6, b5) = (raw_last.r(), raw_last.g(), raw_last.b());
            let mut acc_r = (((r5 << 3) | (r5 >> 2)) as i32) << 8;
            let mut acc_g = (((g6 << 2) | (g6 >> 4)) as i32) << 8;
            let mut acc_b = (((b5 << 3) | (b5 >> 2)) as i32) << 8;

            for y in (y0..y1).rev() {
                let idx = y as usize * w + x as usize;
                let raw = self.pixels[idx];
                let (r5, g6, b5) = (raw.r(), raw.g(), raw.b());
                let r8 = ((r5 << 3) | (r5 >> 2)) as i32;
                let g8 = ((g6 << 2) | (g6 >> 4)) as i32;
                let b8 = ((b5 << 3) | (b5 >> 2)) as i32;

                acc_r += (((r8 << 8) - acc_r) * alpha) >> 8;
                acc_g += (((g8 << 8) - acc_g) * alpha) >> 8;
                acc_b += (((b8 << 8) - acc_b) * alpha) >> 8;

                let r_out = ((acc_r >> 8).clamp(0, 255) as u8) >> 3;
                let g_out = ((acc_g >> 8).clamp(0, 255) as u8) >> 2;
                let b_out = ((acc_b >> 8).clamp(0, 255) as u8) >> 3;
                self.pixels[idx] = Rgb565::new(r_out, g_out, b_out);
            }
        }
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

/// A fixed-capacity RGBA8888 framebuffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FramebufferRgba8888<const N: usize> {
    pixels: [Rgba8888; N],
    width: u32,
    height: u32,
}

impl<const N: usize> FramebufferRgba8888<N> {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(
            (width as usize) * (height as usize) <= N,
            "Framebuffer backing array too small for width * height",
        );
        Self {
            pixels: [Rgba8888::BLACK; N],
            width,
            height,
        }
    }

    pub fn clear_color(&mut self, color: Rgba8888) {
        let len = self.len();
        for p in self.pixels[..len].iter_mut() {
            *p = color;
        }
    }

    pub fn pixels(&self) -> &[Rgba8888] {
        &self.pixels[..self.len()]
    }

    pub fn pixels_mut(&mut self) -> &mut [Rgba8888] {
        let len = self.len();
        &mut self.pixels[..len]
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

    pub fn apply_iir_blur(&mut self, blur_degree: u8) {
        let rect = Rect::new(0, 0, self.width, self.height);
        self.blur_rect(rect, blur_degree);
    }

    pub fn blur_rect(&mut self, rect: Rect, blur_degree: u8) {
        if blur_degree == 0 || self.width == 0 || self.height == 0 {
            return;
        }
        let x0 = rect.x.max(0) as u32;
        let y0 = rect.y.max(0) as u32;
        let x1 = (rect.right() as u32).min(self.width);
        let y1 = (rect.bottom() as u32).min(self.height);

        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let alpha = 256 - (blur_degree as i32);
        let w = self.width as usize;

        // Pass 1: Horizontal Forward
        for y in y0..y1 {
            let row_start = y as usize * w;
            let p0 = self.pixels[row_start + x0 as usize];
            let mut acc_r = (p0.r as i32) << 8;
            let mut acc_g = (p0.g as i32) << 8;
            let mut acc_b = (p0.b as i32) << 8;
            let mut acc_a = (p0.a as i32) << 8;

            for x in x0..x1 {
                let idx = row_start + x as usize;
                let p = self.pixels[idx];
                acc_r += ((((p.r as i32) << 8) - acc_r) * alpha) >> 8;
                acc_g += ((((p.g as i32) << 8) - acc_g) * alpha) >> 8;
                acc_b += ((((p.b as i32) << 8) - acc_b) * alpha) >> 8;
                acc_a += ((((p.a as i32) << 8) - acc_a) * alpha) >> 8;

                self.pixels[idx] = Rgba8888 {
                    r: (acc_r >> 8).clamp(0, 255) as u8,
                    g: (acc_g >> 8).clamp(0, 255) as u8,
                    b: (acc_b >> 8).clamp(0, 255) as u8,
                    a: (acc_a >> 8).clamp(0, 255) as u8,
                };
            }
        }

        // Pass 2: Horizontal Reverse
        for y in y0..y1 {
            let row_start = y as usize * w;
            let p_last = self.pixels[row_start + (x1 - 1) as usize];
            let mut acc_r = (p_last.r as i32) << 8;
            let mut acc_g = (p_last.g as i32) << 8;
            let mut acc_b = (p_last.b as i32) << 8;
            let mut acc_a = (p_last.a as i32) << 8;

            for x in (x0..x1).rev() {
                let idx = row_start + x as usize;
                let p = self.pixels[idx];
                acc_r += ((((p.r as i32) << 8) - acc_r) * alpha) >> 8;
                acc_g += ((((p.g as i32) << 8) - acc_g) * alpha) >> 8;
                acc_b += ((((p.b as i32) << 8) - acc_b) * alpha) >> 8;
                acc_a += ((((p.a as i32) << 8) - acc_a) * alpha) >> 8;

                self.pixels[idx] = Rgba8888 {
                    r: (acc_r >> 8).clamp(0, 255) as u8,
                    g: (acc_g >> 8).clamp(0, 255) as u8,
                    b: (acc_b >> 8).clamp(0, 255) as u8,
                    a: (acc_a >> 8).clamp(0, 255) as u8,
                };
            }
        }

        // Pass 3: Vertical Forward
        for x in x0..x1 {
            let p0 = self.pixels[y0 as usize * w + x as usize];
            let mut acc_r = (p0.r as i32) << 8;
            let mut acc_g = (p0.g as i32) << 8;
            let mut acc_b = (p0.b as i32) << 8;
            let mut acc_a = (p0.a as i32) << 8;

            for y in y0..y1 {
                let idx = y as usize * w + x as usize;
                let p = self.pixels[idx];
                acc_r += ((((p.r as i32) << 8) - acc_r) * alpha) >> 8;
                acc_g += ((((p.g as i32) << 8) - acc_g) * alpha) >> 8;
                acc_b += ((((p.b as i32) << 8) - acc_b) * alpha) >> 8;
                acc_a += ((((p.a as i32) << 8) - acc_a) * alpha) >> 8;

                self.pixels[idx] = Rgba8888 {
                    r: (acc_r >> 8).clamp(0, 255) as u8,
                    g: (acc_g >> 8).clamp(0, 255) as u8,
                    b: (acc_b >> 8).clamp(0, 255) as u8,
                    a: (acc_a >> 8).clamp(0, 255) as u8,
                };
            }
        }

        // Pass 4: Vertical Reverse
        for x in x0..x1 {
            let p_last = self.pixels[(y1 - 1) as usize * w + x as usize];
            let mut acc_r = (p_last.r as i32) << 8;
            let mut acc_g = (p_last.g as i32) << 8;
            let mut acc_b = (p_last.b as i32) << 8;
            let mut acc_a = (p_last.a as i32) << 8;

            for y in (y0..y1).rev() {
                let idx = y as usize * w + x as usize;
                let p = self.pixels[idx];
                acc_r += ((((p.r as i32) << 8) - acc_r) * alpha) >> 8;
                acc_g += ((((p.g as i32) << 8) - acc_g) * alpha) >> 8;
                acc_b += ((((p.b as i32) << 8) - acc_b) * alpha) >> 8;
                acc_a += ((((p.a as i32) << 8) - acc_a) * alpha) >> 8;

                self.pixels[idx] = Rgba8888 {
                    r: (acc_r >> 8).clamp(0, 255) as u8,
                    g: (acc_g >> 8).clamp(0, 255) as u8,
                    b: (acc_b >> 8).clamp(0, 255) as u8,
                    a: (acc_a >> 8).clamp(0, 255) as u8,
                };
            }
        }
    }
}

impl<const N: usize> OriginDimensions for FramebufferRgba8888<N> {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl<const N: usize> DrawTarget for FramebufferRgba8888<N> {
    type Color = Rgba8888;
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

impl<const N: usize> PixelRead for FramebufferRgba8888<N> {
    fn get_pixel(&self, point: Point) -> Rgba8888 {
        match self.index(point.x, point.y) {
            Some(idx) => self.pixels[idx],
            None => Rgba8888::TRANSPARENT,
        }
    }
}

/// A fixed-capacity Gray8 framebuffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FramebufferGray8<const N: usize> {
    pixels: [Gray8; N],
    width: u32,
    height: u32,
}

impl<const N: usize> FramebufferGray8<N> {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(
            (width as usize) * (height as usize) <= N,
            "Framebuffer backing array too small for width * height",
        );
        Self {
            pixels: [Gray8::new(0); N],
            width,
            height,
        }
    }

    pub fn clear_color(&mut self, color: Gray8) {
        let len = self.len();
        for p in self.pixels[..len].iter_mut() {
            *p = color;
        }
    }

    pub fn pixels(&self) -> &[Gray8] {
        &self.pixels[..self.len()]
    }

    pub fn pixels_mut(&mut self) -> &mut [Gray8] {
        let len = self.len();
        &mut self.pixels[..len]
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

    pub fn apply_iir_blur(&mut self, blur_degree: u8) {
        let rect = Rect::new(0, 0, self.width, self.height);
        self.blur_rect(rect, blur_degree);
    }

    pub fn blur_rect(&mut self, rect: Rect, blur_degree: u8) {
        if blur_degree == 0 || self.width == 0 || self.height == 0 {
            return;
        }
        let x0 = rect.x.max(0) as u32;
        let y0 = rect.y.max(0) as u32;
        let x1 = (rect.right() as u32).min(self.width);
        let y1 = (rect.bottom() as u32).min(self.height);

        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let alpha = 256 - (blur_degree as i32);
        let w = self.width as usize;

        // Pass 1: Horizontal Forward
        for y in y0..y1 {
            let row_start = y as usize * w;
            let p0 = self.pixels[row_start + x0 as usize].luma();
            let mut acc = (p0 as i32) << 8;

            for x in x0..x1 {
                let idx = row_start + x as usize;
                let luma = self.pixels[idx].luma();
                acc += ((((luma as i32) << 8) - acc) * alpha) >> 8;
                self.pixels[idx] = Gray8::new((acc >> 8).clamp(0, 255) as u8);
            }
        }

        // Pass 2: Horizontal Reverse
        for y in y0..y1 {
            let row_start = y as usize * w;
            let p_last = self.pixels[row_start + (x1 - 1) as usize].luma();
            let mut acc = (p_last as i32) << 8;

            for x in (x0..x1).rev() {
                let idx = row_start + x as usize;
                let luma = self.pixels[idx].luma();
                acc += ((((luma as i32) << 8) - acc) * alpha) >> 8;
                self.pixels[idx] = Gray8::new((acc >> 8).clamp(0, 255) as u8);
            }
        }

        // Pass 3: Vertical Forward
        for x in x0..x1 {
            let p0 = self.pixels[y0 as usize * w + x as usize].luma();
            let mut acc = (p0 as i32) << 8;

            for y in y0..y1 {
                let idx = y as usize * w + x as usize;
                let luma = self.pixels[idx].luma();
                acc += ((((luma as i32) << 8) - acc) * alpha) >> 8;
                self.pixels[idx] = Gray8::new((acc >> 8).clamp(0, 255) as u8);
            }
        }

        // Pass 4: Vertical Reverse
        for x in x0..x1 {
            let p_last = self.pixels[(y1 - 1) as usize * w + x as usize].luma();
            let mut acc = (p_last as i32) << 8;

            for y in (y0..y1).rev() {
                let idx = y as usize * w + x as usize;
                let luma = self.pixels[idx].luma();
                acc += ((((luma as i32) << 8) - acc) * alpha) >> 8;
                self.pixels[idx] = Gray8::new((acc >> 8).clamp(0, 255) as u8);
            }
        }
    }
}

impl<const N: usize> OriginDimensions for FramebufferGray8<N> {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl<const N: usize> DrawTarget for FramebufferGray8<N> {
    type Color = Gray8;
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

impl<const N: usize> PixelRead for FramebufferGray8<N> {
    fn get_pixel(&self, point: Point) -> Gray8 {
        match self.index(point.x, point.y) {
            Some(idx) => self.pixels[idx],
            None => Gray8::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iir_blur_rgb565() {
        let mut fb = Framebuffer::<100>::new(10, 10);
        fb.clear_color(Rgb565::WHITE);
        for y in 3..7 {
            for x in 3..7 {
                fb.pixels_mut()[y * 10 + x] = Rgb565::BLACK;
            }
        }
        fb.apply_iir_blur(128);
        let center_pixel = fb.pixels()[5 * 10 + 5];
        assert_ne!(center_pixel, Rgb565::BLACK);
    }

    #[test]
    fn test_iir_blur_rgba8888() {
        let mut fb = FramebufferRgba8888::<100>::new(10, 10);
        fb.clear_color(Rgba8888::WHITE);
        for y in 3..7 {
            for x in 3..7 {
                fb.pixels_mut()[y * 10 + x] = Rgba8888::BLACK;
            }
        }
        fb.apply_iir_blur(128);
        let center_pixel = fb.pixels()[5 * 10 + 5];
        assert_ne!(center_pixel, Rgba8888::BLACK);
        assert!(center_pixel.r > 0);
    }

    #[test]
    fn test_iir_blur_gray8() {
        let mut fb = FramebufferGray8::<100>::new(10, 10);
        fb.clear_color(Gray8::new(255));
        for y in 3..7 {
            for x in 3..7 {
                fb.pixels_mut()[y * 10 + x] = Gray8::new(0);
            }
        }
        fb.apply_iir_blur(128);
        let center_pixel = fb.pixels()[5 * 10 + 5].luma();
        assert!(center_pixel > 0 && center_pixel < 255);
    }
}

