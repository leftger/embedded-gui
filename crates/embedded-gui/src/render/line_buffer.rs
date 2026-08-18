use core::convert::Infallible;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb565, RgbColor},
};

use crate::geometry::Rect;

/// A minimal-RAM line-buffer streamer designed for microcontrollers with < 2 KB of SRAM.
/// Renders display contents in narrow scanline bands (e.g. 1, 4, 8, 16 lines),
/// streaming each slice directly to the hardware display controller.
#[derive(Clone, Debug)]
pub struct LineBufferRenderer<const N: usize> {
    buffer: [Rgb565; N],
    width: usize,
    lines: usize,
    current_y: i32,
}

impl<const N: usize> LineBufferRenderer<N> {
    pub const fn new(width: usize, lines: usize) -> Self {
        Self {
            buffer: [Rgb565::BLACK; N],
            width,
            lines,
            current_y: 0,
        }
    }

    #[inline]
    pub const fn lines(&self) -> usize {
        self.lines
    }

    #[inline]
    pub const fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub const fn current_y(&self) -> i32 {
        self.current_y
    }

    pub fn clear(&mut self, color: Rgb565) {
        let len = (self.width * self.lines).min(N);
        self.buffer[..len].fill(color);
    }

    pub fn buffer(&self) -> &[Rgb565] {
        let len = (self.width * self.lines).min(N);
        &self.buffer[..len]
    }

    pub fn buffer_mut(&mut self) -> &mut [Rgb565] {
        let len = (self.width * self.lines).min(N);
        &mut self.buffer[..len]
    }

    /// Iterates over a viewport in slices of `lines` height, rendering each slice via `render_slice`
    /// and flushing the slice to the display target.
    pub fn render_stream<D, F>(
        &mut self,
        target: &mut D,
        viewport: Rect,
        clear_color: Option<Rgb565>,
        mut render_slice: F,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        F: FnMut(&mut ScanlineTarget<'_, N>, Rect) -> Result<(), Infallible>,
    {
        let mut y = viewport.y;
        let y_end = viewport.y + viewport.h as i32;

        while y < y_end {
            let slice_h = ((y_end - y) as u32).min(self.lines as u32);
            let slice_rect = Rect::new(viewport.x, y, viewport.w.min(self.width as u32), slice_h);

            self.current_y = y;
            if let Some(c) = clear_color {
                self.clear(c);
            } else {
                self.clear(Rgb565::BLACK);
            }

            {
                let mut scanline_target = ScanlineTarget {
                    renderer: self,
                    slice_rect,
                };
                let _ = render_slice(&mut scanline_target, slice_rect);
            }

            // Flush the current slice buffer to the target
            let width = self.width;
            let buffer = &self.buffer;
            let pixels = (0..slice_h).flat_map(|row| {
                let y_coord = y + row as i32;
                let row_start = (row as usize) * width;
                (0..slice_rect.w).map(move |col| {
                    let x_coord = viewport.x + col as i32;
                    let color = buffer[row_start + col as usize];
                    Pixel(Point::new(x_coord, y_coord), color)
                })
            });

            target.draw_iter(pixels)?;
            y += slice_h as i32;
        }

        Ok(())
    }
}

/// A clipped DrawTarget view into the active scanline slice.
pub struct ScanlineTarget<'a, const N: usize> {
    renderer: &'a mut LineBufferRenderer<N>,
    slice_rect: Rect,
}

impl<'a, const N: usize> OriginDimensions for ScanlineTarget<'a, N> {
    fn size(&self) -> Size {
        Size::new(self.slice_rect.w, self.slice_rect.h)
    }
}

impl<'a, const N: usize> DrawTarget for ScanlineTarget<'a, N> {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let min_x = self.slice_rect.x;
        let max_x = min_x + self.slice_rect.w as i32;
        let min_y = self.slice_rect.y;
        let max_y = min_y + self.slice_rect.h as i32;
        let width = self.renderer.width;
        let lines = self.renderer.lines;

        for Pixel(pt, color) in pixels {
            if pt.x >= min_x && pt.x < max_x && pt.y >= min_y && pt.y < max_y {
                let local_x = (pt.x - min_x) as usize;
                let local_y = (pt.y - min_y) as usize;
                if local_x < width && local_y < lines {
                    let idx = local_y * width + local_x;
                    if idx < N {
                        self.renderer.buffer[idx] = color;
                    }
                }
            }
        }

        Ok(())
    }
}
