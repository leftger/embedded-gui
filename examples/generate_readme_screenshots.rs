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
const MOTION_W: u32 = 220;
const MOTION_H: u32 = 128;
const FLIP_W: u32 = 176;
const FLIP_H: u32 = 176;

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

    let mut dashboard_display =
        SimulatorDisplay::<Rgb565>::new(Size::new(DASHBOARD_W, DASHBOARD_H));
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

    let motion_settings = OutputSettingsBuilder::new().scale(4).build();
    for frame_idx in 0..32 {
        let mut motion_display = SimulatorDisplay::<Rgb565>::new(Size::new(MOTION_W, MOTION_H));
        let t = frame_idx as f32 / 31.0;
        draw_motion_showcase(&mut motion_display, t);
        let path = format!("docs/screenshots/motion_{frame_idx:02}.png");
        motion_display
            .to_rgb_output_image(&motion_settings)
            .save_png(path)?;
    }
    let mut motion_poster = SimulatorDisplay::<Rgb565>::new(Size::new(MOTION_W, MOTION_H));
    draw_motion_showcase(&mut motion_poster, 0.55);
    motion_poster
        .to_rgb_output_image(&motion_settings)
        .save_png("docs/screenshots/motion.png")?;

    let flip_settings = OutputSettingsBuilder::new().scale(4).build();
    const FLIP_FRAMES: usize = 40;
    for frame_idx in 0..FLIP_FRAMES {
        let mut flip_display = SimulatorDisplay::<Rgb565>::new(Size::new(FLIP_W, FLIP_H));
        let t = frame_idx as f32 / (FLIP_FRAMES - 1) as f32;
        draw_flipcard_showcase(&mut flip_display, t);
        let path = format!("docs/screenshots/flipcard_{frame_idx:02}.png");
        flip_display
            .to_rgb_output_image(&flip_settings)
            .save_png(path)?;
    }

    println!("wrote dashboard/fonts/motion/flipcard PNGs to docs/screenshots");
    Ok(())
}

fn draw_dashboard_showcase(display: &mut SimulatorDisplay<Rgb565>) {
    let mut gui = GuiContext::<32, 32, 24>::new(Rect::new(0, 0, DASHBOARD_W, DASHBOARD_H));
    gui.add_themed_panel(Rect::new(4, 4, 184, 112))
        .expect("panel");
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
    ctx.draw_text_model_in(Rect::new(body.x, body.y + 10, body.w, body.h - 10), text)
        .expect("text model");
    ctx.draw_text_in(
        Rect::new(4, 2, FONT_W - 8, 8),
        "Mixed tiny + large spans",
        TextStyle::new(Rgb565::WHITE).with_align(TextAlign::Center),
    )
    .expect("hint");
}

fn draw_flipcard_showcase(display: &mut SimulatorDisplay<Rgb565>, t: f32) {
    let mut gui_a = GuiContext::<16, 16, 8>::new(Rect::new(0, 0, FLIP_W, FLIP_H));
    gui_a
        .add_themed_panel(Rect::new(8, 8, FLIP_W - 16, FLIP_H - 16))
        .expect("panel");
    gui_a
        .add_themed_label(Rect::new(16, 18, FLIP_W - 32, 10), "DAILY ACTIVITY")
        .expect("title");
    gui_a
        .add_themed_progress_bar(Rect::new(16, 36, FLIP_W - 32, 10), 0.72)
        .expect("steps progress");
    gui_a
        .add_themed_label(Rect::new(16, 48, FLIP_W - 32, 8), "STEPS  72%")
        .expect("steps label");
    gui_a
        .add_themed_progress_bar(Rect::new(16, 62, FLIP_W - 32, 10), 0.55)
        .expect("cal progress");
    gui_a
        .add_themed_label(Rect::new(16, 74, FLIP_W - 32, 8), "CALS   55%")
        .expect("cal label");
    gui_a
        .add_themed_slider(Rect::new(16, 90, FLIP_W - 32, 10), 0.72, 0.0, 1.0)
        .expect("slider");
    gui_a
        .add_themed_value_label(Rect::new(16, 108, FLIP_W - 32, 10), "BPM", 74)
        .expect("bpm");
    gui_a
        .add_themed_icon_button(Rect::new(16, 124, FLIP_W - 32, 14), 'A', "SELECT CARD")
        .expect("button");

    let mut gui_b = GuiContext::<16, 16, 8>::new(Rect::new(0, 0, FLIP_W, FLIP_H));
    gui_b
        .add_themed_panel(Rect::new(8, 8, FLIP_W - 16, FLIP_H - 16))
        .expect("panel");
    gui_b
        .add_themed_label(Rect::new(16, 18, FLIP_W - 32, 10), "HEART RATE")
        .expect("title");
    gui_b
        .add_gauge(
            Rect::new(FLIP_W as i32 / 2 - 24, 32, 48, 48),
            0.62,
            0.0,
            1.0,
            Style::progress(),
        )
        .expect("gauge");
    gui_b
        .add_themed_value_label(Rect::new(16, 86, FLIP_W - 32, 10), "MAX", 142)
        .expect("max bpm");
    gui_b
        .add_themed_value_label(Rect::new(16, 100, FLIP_W - 32, 10), "AVG", 88)
        .expect("avg bpm");
    gui_b
        .add_themed_toggle(Rect::new(16, 116, FLIP_W - 32, 14), "ALERTS", true)
        .expect("toggle");
    gui_b
        .add_themed_icon_button(Rect::new(16, 136, FLIP_W - 32, 14), 'B', "BACK")
        .expect("button");

    let active = ActiveScreenTransition {
        from: Some(ScreenId(0)),
        to: Some(ScreenId(1)),
        effect: ScreenTransitionEffect::RoundFlipLeft,
        origin: ScreenTransitionOrigin::Center,
        progress: t,
    };
    render_transition_pair(display, &gui_a, &gui_b, active, FLIP_W, FLIP_H)
        .expect("flip transition");
}

fn draw_motion_showcase(display: &mut SimulatorDisplay<Rgb565>, t: f32) {
    let mut gui = GuiContext::<24, 24, 16>::new(Rect::new(0, 0, MOTION_W, MOTION_H));
    let eased = apply_easing(t, Easing::InOutSine);
    let panel_x = 14 + (eased * 96.0).round() as i32;
    let panel_y = 20 + (apply_easing(t, Easing::OutBounce) * 32.0).round() as i32;
    let progress = apply_easing(t, Easing::InOutCubic);
    let gauge = apply_easing(t, Easing::OutExpo);
    let opacity = (80.0 + 175.0 * apply_easing(t, Easing::InOutSine)).round() as u8;

    gui.add_themed_panel(Rect::new(6, 6, MOTION_W - 12, MOTION_H - 12))
        .expect("shell");
    gui.add_themed_label(
        Rect::new(12, 10, MOTION_W - 24, 10),
        "ANIMATION + TRANSITION CAPABILITIES",
    )
    .expect("title");
    gui.add_themed_progress_bar(Rect::new(12, 28, 122, 10), progress)
        .expect("progress");
    gui.add_gauge(
        Rect::new(140, 22, 26, 26),
        gauge,
        0.0,
        1.0,
        Style::progress(),
    )
    .expect("gauge");
    gui.add_spinner(Rect::new(170, 24, 20, 20), progress, Style::progress())
        .expect("spinner");
    let card = gui
        .add_themed_panel(Rect::new(panel_x, panel_y, 72, 28))
        .expect("card");
    gui.set_widget_opacity(card, opacity).expect("opacity");
    gui.add_themed_label(Rect::new(panel_x + 6, panel_y + 8, 58, 10), "MOTION")
        .expect("card label");
    gui.add_themed_value_label(Rect::new(12, 44, 62, 12), "FPS", 60)
        .expect("fps");
    gui.add_themed_slider(Rect::new(12, 62, 122, 12), progress, 0.0, 1.0)
        .expect("slider");
    gui.add_themed_icon_button(Rect::new(12, 82, 90, 14), 'A', "CIRCULAR REVEAL")
        .expect("button");

    gui.render(display).expect("render");
}
