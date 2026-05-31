use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 240;
const H: u32 = 140;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("visual quality showcase (1/2/3 + Q compare)", &settings);
    let mut quality = RenderQuality::High;
    let mut compare_mode = false;

    draw_showcase(&mut display, quality, compare_mode);

    'running: loop {
        window.update(&display);
        let mut needs_redraw = false;
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Escape,
                    ..
                } => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Num1,
                    ..
                } => {
                    quality = RenderQuality::Low;
                    compare_mode = false;
                    needs_redraw = true;
                }
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Num2,
                    ..
                } => {
                    quality = RenderQuality::Medium;
                    compare_mode = false;
                    needs_redraw = true;
                }
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Num3,
                    ..
                } => {
                    quality = RenderQuality::High;
                    compare_mode = false;
                    needs_redraw = true;
                }
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Q,
                    ..
                } => {
                    compare_mode = !compare_mode;
                    needs_redraw = true;
                }
                _ => {}
            }
        }
        if needs_redraw {
            draw_showcase(&mut display, quality, compare_mode);
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn draw_showcase(
    display: &mut SimulatorDisplay<Rgb565>,
    quality: RenderQuality,
    compare_mode: bool,
) {
    display.clear(Rgb565::BLACK).unwrap();
    let mut ctx = RenderCtx::new(display, Rect::new(0, 0, W, H));
    ctx.set_quality(quality);
    ctx.fill_rect(Rect::new(0, 0, W, H), Rgb565::new(2, 4, 9))
        .unwrap();
    if compare_mode {
        ctx.fill_rounded_rect_alpha(Rect::new(6, 12, 228, 116), 8, Rgb565::new(8, 16, 24), 255)
            .unwrap();
        draw_quality_card(&mut ctx, Rect::new(10, 18, 70, 108), RenderQuality::Low);
        draw_quality_card(&mut ctx, Rect::new(85, 18, 70, 108), RenderQuality::Medium);
        draw_quality_card(&mut ctx, Rect::new(160, 18, 70, 108), RenderQuality::High);
    } else {
        // Bright backdrop panel makes shadow falloff easy to see.
        ctx.fill_rounded_rect_alpha(Rect::new(10, 12, 220, 116), 8, Rgb565::new(8, 16, 24), 255)
            .unwrap();
        draw_quality_card(&mut ctx, Rect::new(16, 18, 208, 100), quality);
    }
    draw_help_text(&mut ctx, quality, compare_mode);
}

fn draw_quality_card<D>(ctx: &mut RenderCtx<'_, D>, area: Rect, quality: RenderQuality)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    ctx.set_quality(quality);
    let label = match quality {
        RenderQuality::Low => "LOW",
        RenderQuality::Medium => "MED",
        RenderQuality::High => "HIGH",
    };

    let shell = Block::styled(Style {
        background: Some(Rgb565::new(10, 20, 28)),
        gradient: Some(LinearGradient::vertical(
            Rgb565::new(12, 24, 31),
            Rgb565::new(4, 10, 16),
        )),
        font: FontId::Scaled6x10,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::CYAN,
        opacity: 255,
        corner_radius: 5,
        shadow: Some(Shadow {
            color: Rgb565::new(0, 0, 6),
            opacity: 220,
            offset_x: 4,
            offset_y: 4,
            spread: 5,
        }),
        border: Border::one(Rgb565::new(10, 20, 24)),
        padding: EdgeInsets::all(3),
    })
    .title(label)
    .title_align(TextAlign::Center);
    shell.render(area, ctx).unwrap();

    let inner = shell.content_area(area);
    let badge = match quality {
        RenderQuality::Low => ("NO SHADOW", Rgb565::new(31, 8, 8)),
        RenderQuality::Medium => ("SINGLE PASS", Rgb565::new(31, 31, 4)),
        RenderQuality::High => ("MULTI PASS", Rgb565::new(8, 63, 8)),
    };
    let badge_w = inner.w.min(96);
    let badge_x = inner.x + (inner.w.saturating_sub(badge_w) as i32 / 2);
    ctx.fill_rounded_rect_alpha(
        Rect::new(badge_x, inner.y + 2, badge_w, 10),
        2,
        badge.1,
        255,
    )
    .unwrap();
    ctx.draw_text_in(
        Rect::new(badge_x, inner.y + 4, badge_w, 6),
        badge.0,
        TextStyle::new(Rgb565::BLACK).with_align(TextAlign::Center),
    )
    .unwrap();

    // Light reference plate to make shadow offsets obvious.
    ctx.fill_rounded_rect_alpha(
        Rect::new(inner.x + 4, inner.y + 16, inner.w.saturating_sub(8), 28),
        3,
        Rgb565::new(18, 26, 31),
        255,
    )
    .unwrap();

    let tile_rect = Rect::new(inner.x + 10, inner.y + 20, inner.w.saturating_sub(20), 18);
    let tile = Block::styled(Style {
        background: Some(Rgb565::new(0, 0, 28)),
        gradient: Some(LinearGradient::vertical(
            Rgb565::new(0, 0, 31),
            Rgb565::new(0, 0, 16),
        )),
        font: FontId::Tiny3x5,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::CYAN,
        opacity: 220,
        corner_radius: 4,
        shadow: Some(Shadow {
            color: Rgb565::BLACK,
            opacity: 220,
            offset_x: 3,
            offset_y: 3,
            spread: 5,
        }),
        border: Border::one(Rgb565::WHITE),
        padding: EdgeInsets::all(1),
    });
    tile.render(tile_rect, ctx).unwrap();
    ctx.draw_text_in(
        tile.inner(tile_rect),
        "tile",
        TextStyle::new(Rgb565::WHITE).with_align(TextAlign::Center),
    )
    .unwrap();

    ctx.draw_text_in(
        Rect::new(inner.x + 2, inner.y + 52, inner.w.saturating_sub(4), 16),
        "shadow tracks\nthe tile bounds",
        TextStyle::new(Rgb565::WHITE)
            .with_align(TextAlign::Center)
            .with_wrap(TextWrap::Character),
    )
    .unwrap();
}

fn draw_help_text<D>(ctx: &mut RenderCtx<'_, D>, quality: RenderQuality, compare_mode: bool)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    let indicator = if compare_mode {
        "[Q] compare ON   [1/2/3] single mode"
    } else {
        match quality {
            RenderQuality::Low => "[1]*low*  [2] med   [3] high   [Q] compare",
            RenderQuality::Medium => "[1] low   [2]*med* [3] high   [Q] compare",
            RenderQuality::High => "[1] low   [2] med   [3]*high*  [Q] compare",
        }
    };
    ctx.draw_text_in(
        Rect::new(8, 2, W - 16, 12),
        indicator,
        TextStyle::new(Rgb565::WHITE).with_align(TextAlign::Center),
    )
    .unwrap();
}
