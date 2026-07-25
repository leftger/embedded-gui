use embedded_graphics_core::pixelcolor::{Gray8, GrayColor, Rgb565, RgbColor};
use embedded_gui::{
    AlphaLinearGradient, AlphaRadialGradient, Framebuffer, FramebufferGray8, FramebufferRgba8888,
    ImageRef, Rect, RenderCtx, Rgba8888, TileMode, TileRef, Transform2D,
};

#[test]
fn test_iir_blur_all_formats() {
    // RGB565
    let mut fb_rgb = Framebuffer::<400>::new(20, 20);
    fb_rgb.clear_color(Rgb565::WHITE);
    for y in 5..15 {
        for x in 5..15 {
            fb_rgb.pixels_mut()[y * 20 + x] = Rgb565::BLACK;
        }
    }
    fb_rgb.blur_rect(Rect::new(0, 0, 20, 20), 160);
    assert_ne!(fb_rgb.pixels()[10 * 20 + 10], Rgb565::BLACK);

    // RGBA8888
    let mut fb_rgba = FramebufferRgba8888::<400>::new(20, 20);
    fb_rgba.clear_color(Rgba8888::WHITE);
    for y in 5..15 {
        for x in 5..15 {
            fb_rgba.pixels_mut()[y * 20 + x] = Rgba8888::new(0, 0, 0, 255);
        }
    }
    fb_rgba.apply_iir_blur(160);
    assert!(fb_rgba.pixels()[10 * 20 + 10].r > 0);

    // Gray8
    let mut fb_gray = FramebufferGray8::<400>::new(20, 20);
    fb_gray.clear_color(Gray8::new(255));
    for y in 5..15 {
        for x in 5..15 {
            fb_gray.pixels_mut()[y * 20 + x] = Gray8::new(0);
        }
    }
    fb_gray.blur_rect(Rect::new(0, 0, 20, 20), 160);
    assert!(fb_gray.pixels()[10 * 20 + 10].luma() > 0);
}

#[test]
fn test_render_ctx_blur_rect() {
    let mut fb = Framebuffer::<400>::new(20, 20);
    fb.clear_color(Rgb565::WHITE);
    for y in 5..15 {
        for x in 5..15 {
            fb.pixels_mut()[y * 20 + x] = Rgb565::BLACK;
        }
    }

    {
        let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 20, 20));
        ctx.blur_rect(Rect::new(2, 2, 16, 16), 128).unwrap();
    }

    let center_px = fb.pixels()[10 * 20 + 10];
    assert_ne!(center_px, Rgb565::BLACK);
}

#[test]
fn test_alpha_gradients_and_masking() {
    let mut fb = Framebuffer::<900>::new(30, 30);
    fb.clear_color(Rgb565::BLACK);

    let lin_grad = AlphaLinearGradient::vertical(Rgb565::RED, 255, Rgb565::BLUE, 128);
    let rad_grad = AlphaRadialGradient::new(0.5, 0.5, 10.0, Rgb565::WHITE, 255, Rgb565::GREEN, 0);

    let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 30, 30));

    // Test rounded rect linear alpha gradient fill
    ctx.fill_rounded_rect_alpha_gradient(Rect::new(0, 0, 15, 15), 3, &lin_grad, 255)
        .unwrap();

    // Test rounded rect radial alpha gradient fill
    ctx.fill_rounded_rect_radial_gradient(Rect::new(15, 0, 15, 15), 3, &rad_grad, 200)
        .unwrap();

    // Test 8-bit alpha mask blending
    let mask: [u8; 16] = [
        255, 128, 64, 0,
        255, 128, 64, 0,
        255, 128, 64, 0,
        255, 128, 64, 0,
    ];
    ctx.fill_rect_alpha_mask(Rect::new(0, 15, 4, 4), &mask, 4, Rgb565::YELLOW, 255)
        .unwrap();

    // Test drop shadow and card fill
    ctx.draw_drop_shadow(Rect::new(18, 18, 8, 8), 2, Rgb565::WHITE, 180, 1, 2)
        .unwrap();

    ctx.draw_card_fill(Rect::new(18, 18, 8, 8), 2, &lin_grad, Rgb565::WHITE, 128, 2)
        .unwrap();
}

#[test]
fn test_tile_transforms_and_2xssaa() {
    let mut fb = Framebuffer::<400>::new(20, 20);
    fb.clear_color(Rgb565::BLACK);

    // Create a 4x4 test texture
    let raw_pixels: [u16; 16] = [
        0xF800, 0x07E0, 0x001F, 0xFFFF,
        0xF800, 0x07E0, 0x001F, 0xFFFF,
        0xF800, 0x07E0, 0x001F, 0xFFFF,
        0xF800, 0x07E0, 0x001F, 0xFFFF,
    ];

    let tile_repeat = TileRef::new(4, 4, &raw_pixels, TileMode::Repeat);
    let tile_mirror = TileRef::new(4, 4, &raw_pixels, TileMode::Mirror);

    // Test tile sampling modes
    assert!(tile_repeat.get_pixel(5, 5).is_some());
    assert!(tile_mirror.get_pixel(-1, -1).is_some());

    let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 20, 20));

    // Test 2D transform inverse
    let tr = Transform2D::rotation(45.0).then(Transform2D::scale(1.5, 1.5));
    let inv = tr.inverse();
    assert!(inv.is_some());

    // Test 2xSSAA tile rendering
    ctx.draw_tile_transformed_ssaa(Rect::new(2, 2, 8, 8), tile_repeat, tr, 255, true)
        .unwrap();

    // Test 2xSSAA image rendering
    let img = ImageRef::new(4, 4, &raw_pixels);
    ctx.draw_image_transformed_ssaa(Rect::new(10, 10, 8, 8), img, 1.2, 30.0, 255, true)
        .unwrap();
}

#[test]
fn test_reverse_colour_and_line_masks() {
    let mut fb = Framebuffer::<400>::new(20, 20);
    fb.clear_color(Rgb565::WHITE);

    // Test reverse colour on sub-rect
    fb.reverse_colour_rect(Rect::new(5, 5, 10, 10));
    assert_eq!(fb.pixels()[5 * 20 + 5], Rgb565::BLACK);

    {
        let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 20, 20));
        ctx.reverse_colour_rect(Rect::new(5, 5, 10, 10)).unwrap();
    }
    assert_eq!(fb.pixels()[5 * 20 + 5], Rgb565::WHITE); // Inverted back to white

    // Test line masks
    let h_mask = [255, 128, 64, 0];
    let v_mask = [255, 128, 64, 0];
    let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 20, 20));
    ctx.fill_rect_horizontal_line_mask(Rect::new(0, 0, 4, 4), &h_mask, Rgb565::RED, 255).unwrap();
    ctx.fill_rect_vertical_line_mask(Rect::new(10, 0, 4, 4), &v_mask, Rgb565::BLUE, 255).unwrap();
}

#[test]
fn test_widgets_busy_wheel_and_gauge() {
    use embedded_gui::{BusyWheel, GaugeWidget};

    let mut fb = Framebuffer::<1600>::new(40, 40);
    fb.clear_color(Rgb565::BLACK);

    let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 40, 40));

    let spinner = BusyWheel::new(20, 20, 12);
    spinner.draw(&mut ctx).unwrap();

    let gauge = GaugeWidget::new(Rect::new(0, 0, 30, 30), 0.0, 100.0);
    gauge.draw(&mut ctx).unwrap();
}

