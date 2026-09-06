//! Output boundary color conversion adapters for [`DrawTarget`].
//!
//! This module provides [`ColorConvertedDrawTarget`] and [`DrawTargetColorExt`], allowing
//! UI rendering pipelines that produce one pixel color format (such as [`Rgb565`]) to
//! draw directly into physical display drivers or buffers that require a different native
//! format (such as [`Bgr565`](embedded_graphics_core::pixelcolor::Bgr565),
//! [`Rgb888`](embedded_graphics_core::pixelcolor::Rgb888),
//! [`Gray8`](embedded_graphics_core::pixelcolor::Gray8), or
//! [`BinaryColor`](embedded_graphics_core::pixelcolor::BinaryColor)) with zero-heap,
//! on-the-fly conversion.

use core::marker::PhantomData;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::Dimensions,
    pixelcolor::{PixelColor, Rgb565},
    primitives::Rectangle,
};

/// A [`DrawTarget`] adapter that translates incoming pixels from format `CIn`
/// into the underlying display's native format `D::Color` using `From`/`Into`.
///
/// # Example
/// ```rust
/// use embedded_graphics_core::pixelcolor::{Rgb565, Rgb888};
/// use embedded_graphics_core::prelude::*;
/// use embedded_graphics_core::primitives::Rectangle;
/// use embedded_gui::adapter::DrawTargetColorExt;
///
/// # struct DummyDisplay;
/// # impl Dimensions for DummyDisplay {
/// #     fn bounding_box(&self) -> Rectangle { Rectangle::zero() }
/// # }
/// # impl DrawTarget for DummyDisplay {
/// #     type Color = Rgb888;
/// #     type Error = core::convert::Infallible;
/// #     fn draw_iter<I>(&mut self, _: I) -> Result<(), Self::Error> where I: IntoIterator<Item = Pixel<Self::Color>> { Ok(()) }
/// # }
/// let mut display = DummyDisplay;
/// let mut adapted = display.color_converted::<Rgb565>();
/// // Now `adapted` implements `DrawTarget<Color = Rgb565>`, ready for embedded-gui rendering!
/// ```
#[derive(Debug)]
pub struct ColorConvertedDrawTarget<'a, D: ?Sized, CIn = Rgb565> {
    _input_color: PhantomData<CIn>,
    target: &'a mut D,
}

impl<'a, D: ?Sized, CIn> ColorConvertedDrawTarget<'a, D, CIn> {
    /// Wraps a mutable target reference to convert colors on-the-fly from `CIn` to `D::Color`.
    #[inline]
    pub fn new(target: &'a mut D) -> Self {
        Self {
            target,
            _input_color: PhantomData,
        }
    }

    /// Borrow the inner draw target.
    #[inline]
    pub fn inner(&self) -> &D {
        self.target
    }

    /// Mutably borrow the inner draw target.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut D {
        self.target
    }
}

impl<'a, D: Dimensions + ?Sized, CIn> Dimensions for ColorConvertedDrawTarget<'a, D, CIn> {
    #[inline]
    fn bounding_box(&self) -> Rectangle {
        self.target.bounding_box()
    }
}

impl<'a, D: ?Sized, CIn> DrawTarget for ColorConvertedDrawTarget<'a, D, CIn>
where
    D: DrawTarget,
    CIn: PixelColor,
    D::Color: From<CIn>,
{
    type Color = CIn;
    type Error = D::Error;

    #[inline]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.target.draw_iter(
            pixels
                .into_iter()
                .map(|Pixel(pt, color)| Pixel(pt, D::Color::from(color))),
        )
    }

    #[inline]
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.target
            .fill_contiguous(area, colors.into_iter().map(D::Color::from))
    }

    #[inline]
    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.target.fill_solid(area, D::Color::from(color))
    }

    #[inline]
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.target.clear(D::Color::from(color))
    }
}

/// Extension trait on any [`DrawTarget`] to conveniently wrap it in a
/// [`ColorConvertedDrawTarget`].
pub trait DrawTargetColorExt: DrawTarget {
    /// Adapts this draw target to accept input color `CIn`, automatically
    /// converting each pixel to `Self::Color` via `From`.
    #[inline]
    fn color_converted<CIn>(&mut self) -> ColorConvertedDrawTarget<'_, Self, CIn>
    where
        CIn: PixelColor,
        Self::Color: From<CIn>,
    {
        ColorConvertedDrawTarget::new(self)
    }
}

impl<D: DrawTarget> DrawTargetColorExt for D {}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::{
        geometry::{OriginDimensions, Point, Size},
        pixelcolor::{BinaryColor, Gray8, Rgb565, Rgb888, RgbColor},
    };

    struct MockBuffer<C, const N: usize> {
        pixels: [C; N],
        size: Size,
    }

    impl<C: Copy, const N: usize> MockBuffer<C, N> {
        fn new(default: C, width: u32, height: u32) -> Self {
            Self {
                pixels: [default; N],
                size: Size::new(width, height),
            }
        }
    }

    impl<C, const N: usize> OriginDimensions for MockBuffer<C, N> {
        fn size(&self) -> Size {
            self.size
        }
    }

    impl<C: Copy + PixelColor, const N: usize> DrawTarget for MockBuffer<C, N> {
        type Color = C;
        type Error = core::convert::Infallible;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            let w = self.size.width as i32;
            let h = self.size.height as i32;
            for Pixel(pt, color) in pixels {
                if pt.x >= 0 && pt.x < w && pt.y >= 0 && pt.y < h {
                    let idx = (pt.y * w + pt.x) as usize;
                    if idx < N {
                        self.pixels[idx] = color;
                    }
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_rgb565_to_rgb888_adapter() {
        let mut buffer = MockBuffer::<Rgb888, 4>::new(Rgb888::BLACK, 2, 2);
        {
            let mut adapted = buffer.color_converted::<Rgb565>();
            let red_565 = Rgb565::RED;
            adapted
                .draw_iter([
                    Pixel(Point::new(0, 0), red_565),
                    Pixel(Point::new(1, 1), Rgb565::WHITE),
                ])
                .unwrap();
        }

        assert_eq!(buffer.pixels[0], Rgb888::RED);
        assert_eq!(buffer.pixels[1], Rgb888::BLACK);
        assert_eq!(buffer.pixels[3], Rgb888::WHITE);
    }

    #[test]
    fn test_rgb565_to_gray8_adapter() {
        let mut buffer = MockBuffer::<Gray8, 2>::new(Gray8::new(0), 2, 1);
        {
            let mut adapted = buffer.color_converted::<Rgb565>();
            adapted
                .draw_iter([
                    Pixel(Point::new(0, 0), Rgb565::BLACK),
                    Pixel(Point::new(1, 0), Rgb565::WHITE),
                ])
                .unwrap();
        }

        assert_eq!(buffer.pixels[0], Gray8::new(0));
        assert_eq!(buffer.pixels[1], Gray8::new(255));
    }

    #[test]
    fn test_rgb565_to_binary_adapter() {
        let mut buffer = MockBuffer::<BinaryColor, 2>::new(BinaryColor::Off, 2, 1);
        {
            let mut adapted = buffer.color_converted::<Rgb565>();
            adapted
                .draw_iter([
                    Pixel(Point::new(0, 0), Rgb565::BLACK),
                    Pixel(Point::new(1, 0), Rgb565::WHITE),
                ])
                .unwrap();
        }

        assert_eq!(buffer.pixels[0], BinaryColor::Off);
        assert_eq!(buffer.pixels[1], BinaryColor::On);
    }

    #[test]
    fn test_dimensions_delegation() {
        let mut buffer = MockBuffer::<Rgb888, 4>::new(Rgb888::BLACK, 2, 2);
        let adapted = buffer.color_converted::<Rgb565>();
        assert_eq!(
            adapted.bounding_box(),
            Rectangle::new(Point::zero(), Size::new(2, 2))
        );
    }
}
