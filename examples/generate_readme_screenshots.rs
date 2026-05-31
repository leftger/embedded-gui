use embedded_graphics_core::{
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use embedded_gui::prelude::*;

const DASHBOARD_W: u32 = 192;
const DASHBOARD_H: u32 = 120;
const FONT_W: u32 = 240;
const FONT_H: u32 = 140;

static TABS: [&str; 3] = ["SYS", "GFX", "NET"];
static SETTINGS_ITEMS: [&str; 6] = ["DITHER", "AUDIO", "RADAR", "VIBRO", "DEBUG", "ABOUT"];

fn main() {
    if let Err(err) = run() {
        eprintln!("failed to generate screenshots: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("docs/screenshots")?;

    let mut dashboard_display = SimulatorDisplay::<Rgb565>::new(Size::new(DASHBOARD_W, DASHBOARD_H));
    draw_dashboard_showcase(&mut dashboard_display);
    let dashboard_settings = OutputSettingsBuilder::new().scale(4).build();
    dashboard_display
        .to_rgb_output_image(&dashboard_settings)
        .save_png("docs/screenshots/dashboard.png")?;

    let mut fonts_display = SimulatorDisplay::<Rgb565>::new(Size::new(FONT_W, FONT_H));
    draw_font_showcase(&mut fonts_display);
    let font_settings = OutputSettingsBuilder::new().scale(4).build();
    fonts_display
        .to_rgb_output_image(&font_settings)
        .save_png("docs/screenshots/fonts.png")?;

    println!("wrote docs/screenshots/dashboard.png and docs/screenshots/fonts.png");
    Ok(())
}

fn draw_dashboard_showcase(display: &mut SimulatorDisplay<Rgb565>) {
    let mut gui = GuiContext::<32, 32, 24>::new(Rect::new(0, 0, DASHBOARD_W, DASHBOARD_H));
    gui.add_themed_panel(Rect::new(4, 4, 184, 112)).expect("panel");
    gui.add_themed_label(Rect::new(12, 10, 168, 10), "SETTINGS")
        .expect("label");
    gui.add_themed_tabs(Rect::new(12, 26, 104, 14), &TABS, 1)
        .expect("tabs");
    let scroll = gui
        .add_themed_scroll_view(Rect::new(12, 44, 104, 54), 0, 96)
        .expect("scroll view");
    let list = gui
        .add_themed_list(Rect::new(4, 4, 92, 42), &SETTINGS_ITEMS, 2, 4)
        .expect("list");
    gui.add_child(scroll, list).expect("child");
    gui.add_themed_toggle(Rect::new(124, 28, 52, 14), "AA", true)
        .expect("toggle");
    gui.add_themed_slider(Rect::new(124, 50, 52, 14), 0.68, 0.0, 1.0)
        .expect("slider");
    gui.add_themed_value_label(Rect::new(124, 70, 52, 12), "FPS", 60)
        .expect("value");
    gui.add_themed_icon_button(Rect::new(124, 88, 52, 14), 'D', "DIALOG")
        .expect("icon button");
    gui.render(display).expect("render");
}

fn draw_font_showcase(display: &mut SimulatorDisplay<Rgb565>) {
    let mut ctx = RenderCtx::new(display, Rect::new(0, 0, FONT_W, FONT_H));
    ctx.fill_rect(Rect::new(0, 0, FONT_W, FONT_H), Rgb565::new(2, 4, 8))
        .expect("fill");

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
    outer
        .render(Rect::new(10, 10, 220, 120), &mut ctx)
        .expect("panel");

    let text_area = outer.content_area(Rect::new(10, 10, 220, 120));
    let body = text_area.inset(EdgeInsets::all(2));
    ctx.draw_text_in(
        Rect::new(body.x, body.y, body.w, 8),
        "MIXED",
        TextStyle::new(Rgb565::YELLOW).with_align(TextAlign::Center),
    )
    .expect("title");

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
    ctx.draw_text_model_in(
        Rect::new(body.x, body.y + 10, body.w, body.h - 10),
        text,
    )
    .expect("text model");
    ctx.draw_text_in(
        Rect::new(4, 2, FONT_W - 8, 8),
        "Mixed tiny + large spans",
        TextStyle::new(Rgb565::WHITE).with_align(TextAlign::Center),
    )
    .expect("hint");
}
