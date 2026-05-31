use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 192;
const H: u32 = 120;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("embedded-gui text + layout showcase", &settings);

    draw_showcase(&mut display);

    'running: loop {
        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Escape,
                    ..
                } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn draw_showcase(display: &mut SimulatorDisplay<Rgb565>) {
    display.clear(Rgb565::BLACK).unwrap();

    let mut ctx = RenderCtx::new(display, Rect::new(0, 0, W, H));
    let shell_style = Style {
        background: Some(Rgb565::new(1, 3, 6)),
        gradient: Some(LinearGradient::vertical(
            Rgb565::new(3, 6, 10),
            Rgb565::new(0, 1, 4),
        )),
        font: FontId::Scaled6x10,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::CYAN,
        opacity: 255,
        corner_radius: 2,
        shadow: Some(Shadow::soft()),
        border: Border::one(Rgb565::new(9, 18, 24)),
        padding: EdgeInsets::all(4),
    };
    let shell = Block::styled(shell_style)
        .title("CORE API")
        .title_align(TextAlign::Center);
    shell.render(Rect::new(4, 4, 184, 112), &mut ctx).unwrap();

    let header = Rect::new(10, 10, 172, 16);
    let header_spans = [
        Span::styled("TEXT", TextStyle::new(Rgb565::CYAN)),
        Span::styled(" + ", TextStyle::new(Rgb565::WHITE)),
        Span::styled("LAYOUT", TextStyle::new(Rgb565::YELLOW)),
    ];
    let header_lines = [Line::from_spans(&header_spans).aligned(TextAlign::Center)];
    ctx.draw_text_model_in(
        header,
        Text::from_lines(&header_lines).vertical_aligned(VerticalAlign::Middle),
    )
    .unwrap();

    let layout = LinearLayout::row()
        .with_gap(4)
        .with_padding(EdgeInsets::all(0));
    let specs = [
        LayoutItem::length(50),
        LayoutItem::fill_weight(2),
        LayoutItem::ratio(3, 10),
    ];
    let mut cards = [Rect::empty(); 3];
    layout.arrange_items(Rect::new(10, 32, 172, 72), &specs, &mut cards);

    draw_card(
        &mut ctx,
        cards[0],
        "LEFT\nTOP",
        TextStyle {
            color: Rgb565::WHITE,
            font: FontId::Tiny3x5,
            opacity: 255,
            align: TextAlign::Left,
            vertical_align: VerticalAlign::Top,
            wrap: TextWrap::None,
            line_spacing: 1,
        },
        Rgb565::new(3, 7, 11),
    );

    draw_card(
        &mut ctx,
        cards[1],
        "CENTERED\nMULTILINE",
        TextStyle {
            color: Rgb565::YELLOW,
            font: FontId::Tiny3x5,
            opacity: 255,
            align: TextAlign::Center,
            vertical_align: VerticalAlign::Middle,
            wrap: TextWrap::None,
            line_spacing: 1,
        },
        Rgb565::new(6, 6, 2),
    );

    draw_card(
        &mut ctx,
        cards[2],
        "WRAPPED TEXT THAT CLIPS INSIDE",
        TextStyle {
            color: Rgb565::GREEN,
            font: FontId::Tiny3x5,
            opacity: 255,
            align: TextAlign::Right,
            vertical_align: VerticalAlign::Bottom,
            wrap: TextWrap::Character,
            line_spacing: 1,
        },
        Rgb565::new(2, 7, 3),
    );
}

fn draw_card<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    text: &str,
    text_style: TextStyle,
    bg: Rgb565,
) where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    let style = Style {
        background: Some(bg),
        gradient: None,
        font: text_style.font,
        foreground: Rgb565::WHITE,
        text: text_style.color,
        accent: Rgb565::CYAN,
        opacity: 255,
        corner_radius: 2,
        shadow: None,
        border: Border::one(Rgb565::new(12, 20, 22)),
        padding: EdgeInsets::all(3),
    };
    let block = Block::styled(style);
    block.render(rect, ctx).unwrap();
    ctx.draw_text_in(block.inner(rect), text, text_style)
        .unwrap();
}
