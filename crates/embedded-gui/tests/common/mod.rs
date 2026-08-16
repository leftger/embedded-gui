use core::convert::Infallible;

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::Rgb565,
};

pub struct MockTarget {
    pub pixels: heapless::Vec<(i32, i32, Rgb565), 4096>,
    pub size: Size,
}

impl MockTarget {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: heapless::Vec::new(),
            size: Size::new(width, height),
        }
    }
}

impl DrawTarget for MockTarget {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            let _ = self.pixels.push((point.x, point.y, color));
        }
        Ok(())
    }
}

impl OriginDimensions for MockTarget {
    fn size(&self) -> Size {
        self.size
    }
}
