use core::convert::Infallible;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb565, RgbColor},
};

use crate::{
    geometry::Rect,
    render::{
        PixelRead, WindowedDrawTarget,
        task::{DrawTaskQueue, SoftwareDrawUnit, dispatch_draw_tasks},
    },
};

/// A fixed-capacity row-band buffer for partial rendering.
/// Allows rendering large displays in narrow horizontal slices (e.g. 10-20 lines),
/// requiring only a small fraction of the SRAM (e.g., 2–8 KB instead of 150 KB).
#[derive(Clone, Debug)]
pub struct PartialBandBuffer<const N: usize> {
    buffer: [Rgb565; N],
    width: usize,
    height: usize,
}

impl<const N: usize> PartialBandBuffer<N> {
    pub const fn new(width: usize, height: usize) -> Self {
        Self {
            buffer: [Rgb565::BLACK; N],
            width,
            height,
        }
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn buffer(&self) -> &[Rgb565] {
        &self.buffer[..self.width * self.height]
    }

    pub fn buffer_mut(&mut self) -> &mut [Rgb565] {
        let len = self.width * self.height;
        &mut self.buffer[..len]
    }

    pub fn clear_color(&mut self, color: Rgb565) {
        let len = self.width * self.height;
        self.buffer[..len].fill(color);
    }

    /// Renders draw tasks overlapping `dirty_rect` in horizontal slices of height `self.height`,
    /// flushing each rendered band directly to a [`WindowedDrawTarget`].
    pub fn render_tasks_banded<D, const CAP: usize>(
        &mut self,
        _viewport: Rect,
        dirty_rect: Rect,
        tasks: &DrawTaskQueue<'_, CAP>,
        target: &mut D,
        clear_color: Option<Rgb565>,
    ) -> Result<(), D::Error>
    where
        D: WindowedDrawTarget<Color = Rgb565>,
    {
        if dirty_rect.w == 0 || dirty_rect.h == 0 {
            return Ok(());
        }

        let mut y = dirty_rect.y;
        let y_end = dirty_rect.y + dirty_rect.h as i32;

        while y < y_end {
            let band_h = ((y_end - y) as u32).min(self.height as u32);
            let band_rect = Rect::new(dirty_rect.x, y, dirty_rect.w, band_h);

            let len = self.width * self.height;
            if let Some(c) = clear_color {
                self.buffer[..len].fill(c);
            } else {
                self.buffer[..len].fill(Rgb565::BLACK);
            }

            // Render tasks that intersect this band
            for task in tasks.as_slice() {
                let tb = task.bounds();
                let overlap = tb.intersection(band_rect);
                if !overlap.is_empty() {
                    // Create task clipped to band buffer local coordinates
                    let mut local_target = BandTargetWrapper {
                        parent: self,
                        band_origin: Point::new(band_rect.x, band_rect.y),
                    };
                    let mut fallback = SoftwareDrawUnit;
                    let mut queue = DrawTaskQueue::<1>::new();
                    let _ = queue.push(*task);
                    let mut units = [];
                    let _ =
                        dispatch_draw_tasks(&queue, &mut local_target, &mut units, &mut fallback);
                }
            }

            // Set window on hardware controller and flush band slice
            let eg_rect = embedded_graphics_core::primitives::Rectangle::new(
                Point::new(band_rect.x, band_rect.y),
                Size::new(band_rect.w, band_rect.h),
            );
            target.set_window(&eg_rect)?;

            // Stream active pixels
            let count = (band_rect.w * band_rect.h) as usize;
            target.draw_iter((0..count).map(|i| {
                let px = (i % band_rect.w as usize) as i32 + band_rect.x;
                let py = (i / band_rect.w as usize) as i32 + band_rect.y;
                Pixel(Point::new(px, py), self.buffer[i])
            }))?;

            y += band_h as i32;
        }

        Ok(())
    }
}

impl<const N: usize> OriginDimensions for PartialBandBuffer<N> {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl<const N: usize> DrawTarget for PartialBandBuffer<N> {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0
                && coord.y >= 0
                && (coord.x as usize) < self.width
                && (coord.y as usize) < self.height
            {
                let idx = (coord.y as usize) * self.width + (coord.x as usize);
                if idx < N {
                    self.buffer[idx] = color;
                }
            }
        }
        Ok(())
    }
}

impl<const N: usize> PixelRead for PartialBandBuffer<N> {
    fn get_pixel(&self, point: Point) -> Self::Color {
        if point.x >= 0
            && point.y >= 0
            && (point.x as usize) < self.width
            && (point.y as usize) < self.height
        {
            let idx = (point.y as usize) * self.width + (point.x as usize);
            if idx < N {
                return self.buffer[idx];
            }
        }
        Rgb565::BLACK
    }
}

/// Helper wrapper that translates global coordinates into local band buffer coordinates.
struct BandTargetWrapper<'a, const N: usize> {
    parent: &'a mut PartialBandBuffer<N>,
    band_origin: Point,
}

impl<'a, const N: usize> OriginDimensions for BandTargetWrapper<'a, N> {
    fn size(&self) -> Size {
        self.parent.size()
    }
}

impl<'a, const N: usize> DrawTarget for BandTargetWrapper<'a, N> {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let ox = self.band_origin.x;
        let oy = self.band_origin.y;
        let w = self.parent.width;
        let h = self.parent.height;
        for Pixel(coord, color) in pixels {
            let lx = coord.x - ox;
            let ly = coord.y - oy;
            if lx >= 0 && ly >= 0 && (lx as usize) < w && (ly as usize) < h {
                let idx = (ly as usize) * w + (lx as usize);
                if idx < N {
                    self.parent.buffer[idx] = color;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::task::DrawTask;
    use embedded_graphics_core::primitives::Rectangle;

    struct MockWindowedTarget {
        pixels: [Rgb565; 400],
        window: Option<Rectangle>,
    }

    impl OriginDimensions for MockWindowedTarget {
        fn size(&self) -> Size {
            Size::new(20, 20)
        }
    }

    impl DrawTarget for MockWindowedTarget {
        type Color = Rgb565;
        type Error = Infallible;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            for Pixel(pt, color) in pixels {
                if pt.x >= 0 && pt.y >= 0 && pt.x < 20 && pt.y < 20 {
                    self.pixels[(pt.y * 20 + pt.x) as usize] = color;
                }
            }
            Ok(())
        }
    }

    impl WindowedDrawTarget for MockWindowedTarget {
        fn set_window(&mut self, rect: &Rectangle) -> Result<(), Self::Error> {
            self.window = Some(*rect);
            Ok(())
        }
    }

    #[test]
    fn test_partial_band_buffer_render() {
        let mut band = PartialBandBuffer::<100>::new(20, 5);
        assert_eq!(band.width(), 20);
        assert_eq!(band.height(), 5);

        let mut queue = DrawTaskQueue::<2>::new();
        queue
            .push(DrawTask::Fill {
                rect: Rect::new(0, 0, 20, 10),
                color: Rgb565::GREEN,
                radius: 0,
                opacity: 255,
            })
            .unwrap();

        let mut target = MockWindowedTarget {
            pixels: [Rgb565::BLACK; 400],
            window: None,
        };

        band.render_tasks_banded(
            Rect::new(0, 0, 20, 20),
            Rect::new(0, 0, 20, 10),
            &queue,
            &mut target,
            Some(Rgb565::BLACK),
        )
        .unwrap();

        assert_eq!(target.pixels[0], Rgb565::GREEN);
        assert_eq!(target.pixels[20 * 9 + 5], Rgb565::GREEN);
        assert_eq!(target.pixels[20 * 11 + 5], Rgb565::BLACK);
    }
}
