//! Coverage for RAM framebuffer operations and pixel readback helpers.

use embedded_graphics_core::pixelcolor::{Gray8, Rgb565, RgbColor};
use embedded_gui::geometry::Rect;
use embedded_gui::{
    Framebuffer, FramebufferGray8, FramebufferRgba8888, FramebufferSlice, Rgba8888,
};

#[test]
fn rgb565_framebuffer_operations() {
    let mut fb = Framebuffer::<16>::new(4, 4);
    assert_eq!(fb.width(), 4);
    assert_eq!(fb.height(), 4);
    assert_eq!(fb.pixels().len(), 16);
    assert!(fb.pixels().iter().all(|&p| p == Rgb565::BLACK));

    fb.clear_color(Rgb565::new(10, 20, 30));
    assert!(fb.pixels().iter().all(|&p| p == Rgb565::new(10, 20, 30)));

    fb.pixels_mut()[0] = Rgb565::new(31, 0, 0);
    assert_eq!(fb.pixels()[0], Rgb565::new(31, 0, 0));

    fb.invert_rect(Rect::new(0, 0, 4, 4));
    assert_eq!(fb.pixels()[0], Rgb565::new(0, 63, 31));

    fb.invert_rect(Rect::new(-5, -5, 2, 2));
    fb.invert_rect(Rect::new(100, 100, 2, 2));
    fb.invert_rect(Rect::new(2, 2, 0, 0));

    fb.blur_rect(Rect::new(0, 0, 4, 4), 0);
    fb.blur_rect(Rect::new(0, 0, 4, 4), 2);
    fb.apply_iir_blur(1);
    fb.apply_iir_blur(3);

    fb.reverse_colour_rect(Rect::new(0, 0, 4, 4));
    fb.apply_reverse_colour();
}

#[test]
fn rgba_and_gray_framebuffers() {
    let mut rgba = FramebufferRgba8888::<16>::new(4, 4);
    rgba.clear_color(Rgba8888::new(1, 2, 3, 255));
    assert_eq!(rgba.pixels().len(), 16);
    rgba.pixels_mut()[0] = Rgba8888::new(4, 5, 6, 128);
    rgba.blur_rect(Rect::new(0, 0, 4, 4), 2);
    rgba.apply_iir_blur(1);
    rgba.reverse_colour_rect(Rect::new(0, 0, 4, 4));
    rgba.apply_reverse_colour();

    let mut gray = FramebufferGray8::<16>::new(4, 4);
    gray.clear_color(Gray8::new(128));
    assert_eq!(gray.pixels().len(), 16);
    gray.pixels_mut()[0] = Gray8::new(64);
    gray.blur_rect(Rect::new(0, 0, 4, 4), 1);
    gray.apply_iir_blur(2);

    let conv = Rgba8888::from_rgb565(Rgb565::new(31, 63, 31), 255);
    assert_eq!(conv.r, 255);
    assert_eq!(conv.g, 255);
    assert_eq!(conv.b, 255);
    assert_eq!(conv.a, 255);
    let back = conv.to_rgb565();
    assert!(back.r() >= 31);
}

#[test]
fn framebuffer_slice_operations() {
    let mut storage = [Rgb565::BLACK; 16];
    let mut slice = FramebufferSlice::new(&mut storage, 4, 4);
    slice.clear_color(Rgb565::new(5, 10, 15));
    assert_eq!(slice.pixels().len(), 16);
    slice.pixels_mut()[0] = Rgb565::new(31, 0, 0);
    assert_eq!(slice.pixels()[0], Rgb565::new(31, 0, 0));
}
