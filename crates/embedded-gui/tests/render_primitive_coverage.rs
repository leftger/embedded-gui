//! Coverage tests for low-level `RenderCtx` primitive drawing APIs.

use embedded_graphics_core::{
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_gui::text::{BasicTextShaper, Line, ShapingConfig, Span, Text, TextDirection};
use embedded_gui::{
    AlphaLinearGradient, AlphaRadialGradient, BitmapFont, Border, DisplayPalette, Font, FontId,
    Framebuffer, ImageFit, ImageRef, InkRole, LayerState, LinearGradient, Rect, RenderCtx,
    RenderQuality, RoleColors, Shadow, StrokeCap, StrokeJoin, StrokeStyle, TextAlign, TextOverflow,
    TextOverflowPolicy, TextStyle, TextWrap, Transform2D, VerticalAlign,
};

fn fb(w: u32, h: u32) -> (Framebuffer<4096>, Rect) {
    let size = (w * h) as usize;
    assert!(size <= 4096);
    let mut buffer = Framebuffer::<4096>::new(w, h);
    buffer.clear_color(Rgb565::BLACK);
    (buffer, Rect::new(0, 0, w, h))
}

#[test]
fn render_fill_and_stroke_primitives() {
    let (mut buffer, viewport) = fb(64, 64);
    let mut ctx = RenderCtx::compositing(&mut buffer, viewport);

    ctx.fill_rect(Rect::new(1, 1, 10, 10), Rgb565::RED).unwrap();
    ctx.fill_rect_alpha(Rect::new(12, 1, 10, 10), Rgb565::GREEN, 128)
        .unwrap();
    ctx.fill_rounded_rect(Rect::new(1, 12, 10, 10), 3, Rgb565::BLUE)
        .unwrap();
    ctx.fill_rounded_rect_alpha(Rect::new(12, 12, 10, 10), 0, Rgb565::WHITE, 200)
        .unwrap();
    ctx.fill_rounded_rect_alpha(Rect::new(24, 12, 10, 10), 4, Rgb565::CYAN, 0)
        .unwrap();

    let vertical = LinearGradient::vertical(Rgb565::RED, Rgb565::BLUE);
    let horizontal = LinearGradient::horizontal(Rgb565::WHITE, Rgb565::BLACK);
    ctx.fill_rounded_rect_gradient_alpha(Rect::new(24, 1, 10, 10), 0, vertical, 255)
        .unwrap();
    ctx.fill_rounded_rect_gradient_alpha(Rect::new(35, 1, 10, 10), 2, horizontal, 128)
        .unwrap();

    ctx.stroke_rect(Rect::new(1, 24, 10, 10), Border::one(Rgb565::YELLOW))
        .unwrap();
    ctx.stroke_rect_alpha(Rect::new(12, 24, 10, 10), Border::one(Rgb565::YELLOW), 128)
        .unwrap();
    ctx.stroke_rounded_rect(Rect::new(24, 24, 10, 10), 2, Border::one(Rgb565::YELLOW))
        .unwrap();
    ctx.stroke_rounded_rect_alpha(
        Rect::new(35, 24, 10, 10),
        0,
        Border::one(Rgb565::YELLOW),
        128,
    )
    .unwrap();

    ctx.fill_circle(10, 40, 4, Rgb565::MAGENTA).unwrap();
    ctx.fill_circle(10, 52, 0, Rgb565::MAGENTA).unwrap();
    ctx.stroke_circle(24, 40, 3, Rgb565::WHITE).unwrap();
    ctx.stroke_circle(24, 52, 0, Rgb565::WHITE).unwrap();
    ctx.stroke_arc(35, 40, 4, 0, 90, Rgb565::CYAN).unwrap();
    ctx.stroke_arc_styled(
        35,
        52,
        4,
        90,
        0,
        StrokeStyle::new(Rgb565::CYAN)
            .with_width(2)
            .with_cap(StrokeCap::Round)
            .with_join(StrokeJoin::Round),
    )
    .unwrap();
    ctx.fill_sector_sweep(50, 40, 5, 0.0, 90.0, Rgb565::GREEN)
        .unwrap();
    ctx.fill_sector_sweep(58, 52, 5, 0.0, -270.0, Rgb565::BLUE)
        .unwrap();
    ctx.fill_sector_sweep(10, 52, 5, 10.0, 360.0, Rgb565::RED)
        .unwrap();

    ctx.fill_polygon(
        &[Point::new(20, 0), Point::new(26, 0), Point::new(23, 6)],
        Rgb565::new(31, 40, 0),
    )
    .unwrap();
    ctx.fill_polygon(
        &[Point::new(0, 0), Point::new(1, 1)],
        Rgb565::new(31, 40, 0),
    )
    .unwrap();

    ctx.draw_line(0, 40, 8, 48, Rgb565::GREEN).unwrap();
    ctx.draw_line_styled(0, 48, 8, 56, StrokeStyle::new(Rgb565::GREEN).with_width(2))
        .unwrap();
    ctx.draw_bezier_quad(
        Point::new(1, 1),
        Point::new(4, 6),
        Point::new(8, 1),
        StrokeStyle::new(Rgb565::WHITE).with_width(1),
    )
    .unwrap();
    ctx.draw_bezier_cubic(
        Point::new(1, 20),
        Point::new(4, 24),
        Point::new(8, 24),
        Point::new(12, 20),
        StrokeStyle::new(Rgb565::WHITE),
    )
    .unwrap();
}

#[test]
fn render_text_and_transforms() {
    let (mut buffer, viewport) = fb(64, 64);
    let mut ctx = RenderCtx::compositing(&mut buffer, viewport);

    ctx.draw_text(1, 1, "Hi\nThere", Rgb565::WHITE).unwrap();
    ctx.draw_text_with_font(1, 20, "AB", Rgb565::WHITE, FontId::Medium4x7)
        .unwrap();

    let style = TextStyle::new(Rgb565::CYAN)
        .with_align(TextAlign::Center)
        .with_vertical_align(VerticalAlign::Middle)
        .with_wrap(TextWrap::Character)
        .with_overflow(TextOverflow::Ellipsis)
        .with_overflow_policy(TextOverflowPolicy::WrapThenEllipsis { max_lines: 2 })
        .with_ellipsis_mode(embedded_gui::EllipsisMode::SingleGlyph)
        .with_max_lines(Some(2))
        .with_kerning(true);
    ctx.draw_text_in(Rect::new(20, 1, 30, 20), "abcdefghijklmnop", style)
        .unwrap();
    ctx.draw_text_in_with_font(
        Rect::new(1, 40, 20, 10),
        "word word",
        TextStyle::new(Rgb565::WHITE).with_wrap(TextWrap::Word),
        FontId::Tiny3x5,
    )
    .unwrap();

    let metrics = <RenderCtx<Framebuffer<4096>, embedded_gui::Dither>>::text_metrics("hello");
    assert!(metrics.width > 0);
    let metrics2 = <RenderCtx<Framebuffer<4096>, embedded_gui::Dither>>::text_metrics_with_font(
        "hello",
        FontId::Medium4x7,
    );
    assert!(metrics2.width > 0);
    let wrapped = <RenderCtx<Framebuffer<4096>, embedded_gui::Dither>>::text_metrics_wrapped(
        "hello world",
        8,
        TextWrap::Word,
    );
    assert!(wrapped.height >= metrics.height);
    let wrapped2 =
        <RenderCtx<Framebuffer<4096>, embedded_gui::Dither>>::text_metrics_wrapped_with_font(
            "hello",
            8,
            TextWrap::None,
            FontId::Tiny3x5,
        );
    assert!(wrapped2.width > 0);

    ctx.set_clip(Rect::new(0, 0, 50, 50));
    assert_eq!(ctx.clip(), Rect::new(0, 0, 50, 50));
    ctx.push_transform(Transform2D::translation(1.0, 1.0));
    ctx.translate(1.0, 0.0);
    ctx.scale(1.0, 1.0);
    ctx.rotate(0.0);
    ctx.skew(0.0, 0.0);
    ctx.pop_transform();
    ctx.push_layer(LayerState::normal());
    ctx.push_layer(LayerState::normal());
    ctx.pop_layer();
    ctx.pop_layer();
    assert_eq!(ctx.shadow_spread_for(0), 0);
    ctx.set_quality(RenderQuality::Low);
    assert_eq!(ctx.quality(), RenderQuality::Low);
    assert_eq!(ctx.shadow_spread_for(4), 0);
    ctx.set_quality(RenderQuality::Medium);
    assert_eq!(ctx.shadow_spread_for(4), 1);
    ctx.set_quality(RenderQuality::High);
    assert_eq!(ctx.shadow_spread_for(4), 4);
    ctx.set_backend_caps(ctx.backend_caps());
}

#[test]
fn render_images_masks_and_effects() {
    let (mut buffer, viewport) = fb(64, 64);
    let raw: [u16; 16] = [
        0xF800, 0x07E0, 0x001F, 0xFFFF, 0xF800, 0x07E0, 0x001F, 0xFFFF, 0xF800, 0x07E0, 0x001F,
        0xFFFF, 0xF800, 0x07E0, 0x001F, 0xFFFF,
    ];
    let img = ImageRef::new(4, 4, &raw);

    let mut ctx = RenderCtx::compositing(&mut buffer, viewport);
    ctx.draw_image(Rect::new(0, 0, 4, 4), img, ImageFit::Center)
        .unwrap();
    ctx.draw_image(Rect::new(4, 4, 8, 8), img, ImageFit::Stretch)
        .unwrap();
    ctx.draw_image_region(
        Rect::new(12, 0, 8, 8),
        img,
        ImageFit::Stretch,
        Rect::new(0, 0, 2, 2),
    )
    .unwrap();
    ctx.draw_image_transformed(Rect::new(20, 0, 8, 8), img, 1.2, 15.0)
        .unwrap();
    ctx.draw_image_transformed(Rect::new(30, 0, 0, 8), img, 0.0, 0.0)
        .unwrap();

    ctx.fill_rect_masked(Rect::new(0, 16, 8, 8), Rgb565::RED, |x, y| (x + y) % 2 == 0)
        .unwrap();
    let shadow = Shadow::soft();
    ctx.draw_drop_shadow(Rect::new(12, 20, 10, 10), 1, Rgb565::WHITE, 128, 1, 2)
        .unwrap();
    let _ = shadow;
    ctx.reverse_colour_rect(Rect::new(30, 16, 8, 8)).unwrap();
    ctx.blur_rect(Rect::new(30, 16, 8, 8), 0).unwrap();
}

#[test]
fn palette_ink_true_alpha_and_alpha_masks() {
    let (mut buffer, viewport) = fb(64, 64);
    let normal = RoleColors::new(
        Rgb565::BLACK,
        Rgb565::WHITE,
        Rgb565::CYAN,
        Rgb565::RED,
        Rgb565::new(16, 16, 16),
    );
    let palette = DisplayPalette::new(normal, normal);
    {
        let ctx = RenderCtx::new(&mut buffer, viewport).with_palette(palette);
        assert_eq!(ctx.ink(InkRole::Primary), Rgb565::WHITE);
    }

    let mut ctx = RenderCtx::compositing(&mut buffer, viewport);
    ctx.fill_rect_true_alpha(Rect::new(1, 1, 8, 8), Rgb565::RED, 128)
        .unwrap();
    ctx.fill_rounded_rect_true_alpha(Rect::new(10, 1, 8, 8), 2, Rgb565::BLUE, 0)
        .unwrap();
    ctx.fill_rounded_rect_true_alpha(Rect::new(20, 1, 8, 8), 3, Rgb565::BLUE, 128)
        .unwrap();

    let alpha_lin = AlphaLinearGradient::horizontal(Rgb565::RED, 255, Rgb565::BLUE, 0);
    ctx.fill_rounded_rect_alpha_gradient(Rect::new(1, 12, 10, 8), 2, &alpha_lin, 255)
        .unwrap();
    let alpha_lin_v = AlphaLinearGradient::vertical(Rgb565::WHITE, 128, Rgb565::BLACK, 255);
    ctx.fill_rounded_rect_alpha_gradient(Rect::new(12, 12, 10, 8), 0, &alpha_lin_v, 128)
        .unwrap();
    let alpha_radial =
        AlphaRadialGradient::new(0.5, 0.5, 10.0, Rgb565::WHITE, 255, Rgb565::GREEN, 0);
    ctx.fill_rounded_rect_radial_gradient(Rect::new(24, 12, 12, 12), 2, &alpha_radial, 200)
        .unwrap();

    let mask = [
        255u8, 128, 64, 0, 255, 128, 64, 0, 255, 128, 64, 0, 255, 128, 64, 0,
    ];
    ctx.fill_rect_alpha_mask(Rect::new(0, 24, 4, 4), &mask, 4, Rgb565::YELLOW, 255)
        .unwrap();
    ctx.fill_rect_alpha_mask(Rect::new(8, 24, 4, 4), &mask, 0, Rgb565::YELLOW, 255)
        .unwrap();
}

#[test]
fn render_text_model_and_shaped_text() {
    let (mut buffer, viewport) = fb(64, 64);
    let mut ctx = RenderCtx::compositing(&mut buffer, viewport);

    static SPANS: [Span; 2] = [Span::raw("Hello "), Span::raw("world\nwide")];
    static LINES: [Line; 1] = [Line::from_spans(&SPANS)];
    let text = Text::from_lines(&LINES)
        .aligned(TextAlign::Center)
        .vertical_aligned(VerticalAlign::Middle)
        .wrapped(TextWrap::Character)
        .line_spacing(1);
    ctx.draw_text_model_in(Rect::new(0, 0, 30, 20), text)
        .unwrap();
    ctx.draw_text_model_in(Rect::new(30, 30, 0, 0), text)
        .unwrap();

    ctx.draw_text_shaped_in::<BasicTextShaper, 16>(
        Rect::new(0, 20, 20, 8),
        "abc",
        TextStyle::new(Rgb565::WHITE),
        &BasicTextShaper,
        ShapingConfig {
            direction: TextDirection::Ltr,
            language_tag: None,
            enable_ligatures: false,
        },
    )
    .unwrap();
}

#[test]
fn render_text_with_every_font_family() {
    static GLYPHS: [u8; 16] = [
        0b00111100, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b01100110, 0, 0,
        0, 0, 0, 0, 0, 0, 0,
    ];
    static BITMAP: BitmapFont = BitmapFont::new_8x16(65, 8, 16, &GLYPHS);

    struct CustomFont;
    impl Font for CustomFont {
        fn advance(&self) -> u32 {
            4
        }
        fn line_height(&self) -> u32 {
            6
        }
        fn draw_glyph(&self, _ch: char, draw_pixel: &mut dyn FnMut(i32, i32)) {
            draw_pixel(0, 0);
            draw_pixel(1, 1);
        }
    }
    static CUSTOM: CustomFont = CustomFont;

    let (mut buffer, viewport) = fb(64, 64);
    let mut ctx = RenderCtx::compositing(&mut buffer, viewport);
    ctx.draw_text_with_font(0, 0, "A", Rgb565::WHITE, FontId::Scaled6x10)
        .unwrap();
    ctx.draw_text_with_font(0, 12, "A", Rgb565::WHITE, FontId::Vector(1))
        .unwrap();
    ctx.draw_text_with_font(0, 24, "A", Rgb565::WHITE, FontId::from(&BITMAP))
        .unwrap();
    ctx.draw_text_with_font(
        0,
        40,
        "A",
        Rgb565::WHITE,
        FontId::from(&CUSTOM as &'static dyn Font),
    )
    .unwrap();
    ctx.draw_text_with_font(20, 0, "A", Rgb565::WHITE, FontId::Vector(2))
        .unwrap();

    #[cfg(feature = "embedded-graphics")]
    {
        use embedded_graphics::mono_font::ascii::FONT_4X6;
        ctx.draw_text_with_font(20, 24, "A", Rgb565::WHITE, FontId::from(&FONT_4X6))
            .unwrap();
    }
}
