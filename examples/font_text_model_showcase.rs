use embedded_graphics_core::{
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
    prelude::DrawTarget,
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
    let mut window = Window::new(
        "font text-model showcase (1 tiny, 2 mixed, 3 large)",
        &settings,
    );
    let mut mode = 2u8;

    draw_showcase(&mut display, mode);

    'running: loop {
        window.update(&display);
        let mut dirty = false;
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
                    mode = 1;
                    dirty = true;
                }
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Num2,
                    ..
                } => {
                    mode = 2;
                    dirty = true;
                }
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Num3,
                    ..
                } => {
                    mode = 3;
                    dirty = true;
                }
                _ => {}
            }
        }
        if dirty {
            draw_showcase(&mut display, mode);
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn draw_showcase<D>(display: &mut D, mode: u8)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    let mut ctx = RenderCtx::new(display, Rect::new(0, 0, W, H));
    ctx.fill_rect(Rect::new(0, 0, W, H), Rgb565::new(2, 4, 8))
        .unwrap();

    let outer = Block::styled(Style {
        background: Some(Rgb565::new(8, 14, 22)),
        gradient: Some(LinearGradient::vertical(
            Rgb565::new(10, 20, 28),
            Rgb565::new(3, 6, 12),
        )),
        font: FontId::Scaled6x10,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::CYAN,
        opacity: 255,
        corner_radius: 4,
        shadow: Some(Shadow::soft()),
        border: Border::one(Rgb565::new(14, 26, 30)),
        padding: EdgeInsets::all(3),
    })
    .title("FONT MODEL")
    .title_align(TextAlign::Center);
    outer.render(Rect::new(10, 10, 220, 120), &mut ctx).unwrap();

    let text_area = outer.content_area(Rect::new(10, 10, 220, 120));
    let body = text_area.inset(EdgeInsets::all(2));
    let title = match mode {
        1 => "TINY",
        3 => "LARGE",
        _ => "MIXED",
    };
    ctx.draw_text_in(
        Rect::new(body.x, body.y, body.w, 8),
        title,
        TextStyle::new(Rgb565::YELLOW).with_align(TextAlign::Center),
    )
    .unwrap();
    draw_mode_text(
        &mut ctx,
        Rect::new(body.x, body.y + 10, body.w, body.h - 10),
        mode,
    );

    ctx.draw_text_in(
        Rect::new(4, 2, W - 8, 8),
        "[1] tiny  [2] mixed  [3] large",
        TextStyle::new(Rgb565::WHITE).with_align(TextAlign::Center),
    )
    .unwrap();
}

fn draw_mode_text<D>(ctx: &mut RenderCtx<'_, D>, area: Rect, mode: u8)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    match mode {
        1 => {
            let spans = [
                Span::styled(
                    "tiny-only wrapping keeps dense info readable. ",
                    TextStyle::new(Rgb565::WHITE).with_font(FontId::Tiny3x5),
                ),
                Span::styled(
                    "All spans are tiny.",
                    TextStyle::new(Rgb565::CYAN).with_font(FontId::Tiny3x5),
                ),
            ];
            let lines = [Line::from_spans(&spans).aligned(TextAlign::Left)];
            let text = Text::from_lines(&lines).wrapped(TextWrap::Character);
            ctx.draw_text_model_in(area, text).unwrap();
        }
        3 => {
            let spans = [
                Span::styled(
                    "Large glyphs consume width quickly. ",
                    TextStyle::new(Rgb565::WHITE).with_font(FontId::Scaled6x10),
                ),
                Span::styled(
                    "Wrapping reacts to font advance.",
                    TextStyle::new(Rgb565::YELLOW).with_font(FontId::Scaled6x10),
                ),
            ];
            let lines = [Line::from_spans(&spans).aligned(TextAlign::Left)];
            let text = Text::from_lines(&lines).wrapped(TextWrap::Character);
            ctx.draw_text_model_in(area, text).unwrap();
        }
        _ => {
            let line1 = [
                Span::styled(
                    "tiny prefix ",
                    TextStyle::new(Rgb565::WHITE).with_font(FontId::Tiny3x5),
                ),
                Span::styled(
                    "LARGE MID ",
                    TextStyle::new(Rgb565::new(31, 48, 0)).with_font(FontId::Scaled6x10),
                ),
                Span::styled(
                    "tiny suffix ",
                    TextStyle::new(Rgb565::WHITE).with_font(FontId::Tiny3x5),
                ),
                Span::styled(
                    "ACCENT",
                    TextStyle::new(Rgb565::new(0, 54, 28)).with_font(FontId::Scaled6x10),
                ),
            ];
            let line2 = [
                Span::styled(
                    "centered mixed line",
                    TextStyle::new(Rgb565::WHITE).with_font(FontId::Tiny3x5),
                ),
                Span::styled(
                    " ++ ",
                    TextStyle::new(Rgb565::new(31, 48, 0)).with_font(FontId::Scaled6x10),
                ),
                Span::styled(
                    "font-aware wrap",
                    TextStyle::new(Rgb565::new(0, 54, 28)).with_font(FontId::Scaled6x10),
                ),
            ];
            let lines = [
                Line::from_spans(&line1).aligned(TextAlign::Left),
                Line::from_spans(&line2).aligned(TextAlign::Center),
            ];
            let text = Text::from_lines(&lines).wrapped(TextWrap::Character);
            ctx.draw_text_model_in(area, text).unwrap();
        }
    }
}
