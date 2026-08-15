//! Demonstrates the optional `embedded-layout` and `embedded-text` adapters.
//!
//! Run with:
//!   cargo run --example interop_showcase --features embedded-text,embedded-layout
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    prelude::*,
};
use embedded_graphics_core::{draw_target::DrawTarget, geometry::Size, pixelcolor::Rgb565};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::interop::{layout::RectView, text::text_box_with_style};
// Import embedded-gui's items individually rather than via its prelude glob:
// both embedded-gui and embedded-layout export a type/trait named `Align`
// (a layout enum here, a trait there), and gluing both preludes together
// makes that name ambiguous.
use embedded_gui::{
    Block, Border, EdgeInsets, FontId, Rect, RenderCtx, Style, TextAlign, TextStyle,
};
use embedded_layout::{
    layout::linear::{FixedMargin, LinearLayout},
    prelude::*,
};
use embedded_text::style::{HeightMode, TextBoxStyleBuilder, VerticalOverdraw};

const W: u32 = 192;
const H: u32 = 120;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("embedded-gui interop showcase", &settings);

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

    let shell_area = Rect::new(4, 4, W - 8, H - 8);
    let mut ctx = RenderCtx::new(display, Rect::new(0, 0, W, H));
    let shell_style = Style {
        background: Some(Rgb565::new(1, 3, 6)),
        gradient: None,
        font: FontId::Scaled6x10,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::CYAN,
        opacity: 255,
        corner_radius: 2,
        shadow: None,
        border: Border::one(Rgb565::new(9, 18, 24)),
        padding: EdgeInsets::all(4),
    };
    Block::styled(shell_style)
        .title("INTEROP")
        .title_align(TextAlign::Center)
        .render(shell_area, &mut ctx)
        .unwrap();

    // 1. Use embedded-layout's own LinearLayout (with its `DistributeFill`
    //    spacing) to arrange two content slots, instead of embedded-gui's
    //    LinearLayout/Constraint solver.
    let content_area = Rect::new(10, 22, W - 20, H - 32);
    let mut slots = [
        RectView::from(Rect::new(0, 0, content_area.w, 24)),
        RectView::from(Rect::new(0, 0, content_area.w, 24)),
    ];
    let parent = RectView::from(content_area);
    let arranged = LinearLayout::vertical(Views::new(&mut slots))
        .with_spacing(FixedMargin(6))
        .arrange()
        .align_to(&parent, horizontal::Left, vertical::Top);
    let label_area: Rect = arranged.inner()[0].into();
    let text_area: Rect = arranged.inner()[1].into();

    ctx.draw_text_in(
        label_area,
        "arranged by embedded-layout:",
        TextStyle::new(Rgb565::YELLOW),
    )
    .unwrap();

    // 2. Draw an embedded-text `TextBox` (word-wrapped, justified) into the
    //    second slot via the generic `draw_embedded_graphics` escape hatch.
    let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN);
    let style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
        .alignment(embedded_text::alignment::HorizontalAlignment::Justified)
        .build();
    let text_box = text_box_with_style(
        "TextBox from embedded-text, wrapped and justified.",
        text_area,
        character_style,
        style,
    );
    ctx.draw_embedded_graphics(&text_box).unwrap();
}
