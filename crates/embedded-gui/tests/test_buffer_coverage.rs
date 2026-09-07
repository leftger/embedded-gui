//! Coverage for the std-only pixel test buffers and blend helpers.
#![cfg(feature = "std")]

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
    prelude::Point,
};
use embedded_gui::render::BlendMode;
use embedded_gui::{LayerCanvas, TestBuffer};

fn red() -> Rgb565 {
    Rgb565::new(31, 0, 0)
}

#[test]
fn test_buffer_pixel_ops_and_diffs() {
    let mut buf = TestBuffer::new(4, 4);
    assert_eq!(buf.pixel_at(-1, 0), None);
    assert_eq!(buf.pixel_at(0, -1), None);
    assert_eq!(buf.pixel_at(4, 0), None);
    assert_eq!(buf.pixel_at(0, 0), Some(Rgb565::BLACK));

    buf.clear_color(red());
    assert_eq!(buf.count_color(red()), 16);
    assert!(buf.has_non_background_pixel());

    buf.clear_color(Rgb565::BLACK);
    buf.draw_iter([Pixel(Point::new(1, 1), red())]).unwrap();
    buf.draw_iter([Pixel(Point::new(9, 9), red())]).unwrap();
    buf.assert_non_empty_rect(embedded_gui::Rect::new(0, 0, 4, 4));

    let digest = buf.digest();
    buf.assert_digest_eq(digest, "after-red");

    let other = TestBuffer::new(4, 4);
    assert!(buf.diff_bounding_region(&other).is_some());
    assert!(!buf.diff_regions::<8>(&other).is_empty());

    let same = buf.clone();
    assert_eq!(buf.diff_bounding_region(&same), None);
    assert_eq!(buf.diff_regions::<8>(&same).len(), 0);

    let sized = TestBuffer::new(5, 5);
    assert!(sized.diff_bounding_region(&buf).is_some());
    assert_eq!(sized.diff_regions::<4>(&buf).len(), 1);

    let tight = buf.clone();
    assert_eq!(tight.diff_regions::<0>(&other).len(), 0);
}

#[test]
fn test_buffer_composite_and_layer_canvas() {
    let mut base = TestBuffer::new(2, 2);
    base.clear_color(Rgb565::new(10, 20, 30));

    let mut overlay = TestBuffer::new(2, 2);
    overlay.clear_color(Rgb565::new(16, 32, 16));

    let mismatched = TestBuffer::new(3, 3);
    let mut target = base.clone();
    target.composite_from(&mismatched, BlendMode::Normal, 255);
    assert_eq!(target, base);

    let mut opacity_zero = base.clone();
    opacity_zero.composite_from(&overlay, BlendMode::Normal, 0);
    assert_eq!(opacity_zero, base);

    for mode in [
        BlendMode::Normal,
        BlendMode::Add,
        BlendMode::Multiply,
        BlendMode::Screen,
    ] {
        let mut blended = base.clone();
        blended.composite_from(&overlay, mode, 128);
        assert_ne!(blended, base);
    }

    let mut layer = LayerCanvas::new(2, 2);
    layer.clear(red());
    let mut sink = TestBuffer::new(2, 2);
    layer.composite_into(&mut sink, BlendMode::Normal, 255);
    assert!(sink.has_non_background_pixel());

    layer.target_mut().clear_color(Rgb565::BLACK);
    let mut sink2 = TestBuffer::new(2, 2);
    sink2.clear_color(red());
    layer.composite_into(&mut sink2, BlendMode::Normal, 255);
    assert_eq!(sink2.count_color(red()), 4);
}
