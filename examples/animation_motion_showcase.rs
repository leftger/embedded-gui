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
const FRAME_MS: u32 = 16;
const STAGE_COUNT: i32 = 4;

static TABS: [&str; 4] = ["NAV", "FX", "IO", "SYS"];
static ITEMS: [&str; 8] = ["ALPHA", "BETA", "GAMMA", "DELTA", "SIGMA", "OMEGA", "ION", "ARC"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("animation motion showcase", &settings);
    let mut gui = GuiContext::<48, 48, 32>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);

    let mut animator = WidgetAnimator::<64, 64>::new();
    let mut stage = 0_i32;
    start_stage(&mut gui, &mut animator, &ids, stage);

    'running: loop {
        animator.tick(FRAME_MS, &mut gui).unwrap();
        gui.tick_spinner(ids.spinner, FRAME_MS, 0.45).unwrap();

        if animator.active_count() == 0 {
            stage = (stage + 1) % STAGE_COUNT;
            start_stage(&mut gui, &mut animator, &ids, stage);
        }

        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Space => start_stage(&mut gui, &mut animator, &ids, stage),
                    Keycode::Tab => {
                        stage = (stage + 1) % STAGE_COUNT;
                        start_stage(&mut gui, &mut animator, &ids, stage);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(FRAME_MS as u64));
    }
}

struct Ids {
    stage_label: WidgetId,
    progress: WidgetId,
    meter: WidgetId,
    slider: WidgetId,
    scroll: WidgetId,
    sweep_panel: WidgetId,
    tabs: WidgetId,
    dropdown: WidgetId,
    roller: WidgetId,
    gauge: WidgetId,
    spinner: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 48, 48, 32>) -> Ids {
    gui.add_panel(Rect::new(6, 6, 228, 128), Style::panel()).unwrap();
    gui.add_label(
        Rect::new(12, 10, 216, 8),
        "SPACE replay stage | TAB next stage",
        Style::label(),
    )
    .unwrap();

    let stage_label = gui
        .add_value_label(Rect::new(12, 20, 68, 10), "STAGE", 0, Style::panel())
        .unwrap();
    let progress = gui
        .add_progress_bar(Rect::new(12, 34, 94, 10), 0.0, Style::progress())
        .unwrap();
    let meter = gui
        .add_meter(Rect::new(12, 48, 44, 26), 0.0, 0.0, 1.0, Style::progress())
        .unwrap();
    let slider = gui
        .add_slider(Rect::new(62, 54, 44, 12), 0.0, 0.0, 1.0, Style::button())
        .unwrap();

    let scroll = gui
        .add_scroll_view(Rect::new(12, 78, 94, 48), 0, 176, Style::panel())
        .unwrap();
    let list = gui
        .add_list(Rect::new(4, 4, 86, 160), &ITEMS, 0, 8, Style::button())
        .unwrap();
    gui.add_child(scroll, list).unwrap();

    let sweep_panel = gui
        .add_panel(Rect::new(112, 24, 48, 28), Style::panel())
        .unwrap();
    let tabs = gui
        .add_tabs(Rect::new(164, 24, 64, 12), &TABS, 0, Style::button())
        .unwrap();
    let dropdown = gui
        .add_dropdown(Rect::new(164, 40, 64, 12), &ITEMS, 0, Style::button())
        .unwrap();
    let roller = gui
        .add_roller(Rect::new(112, 58, 48, 64), &ITEMS, 0, Style::button())
        .unwrap();
    let gauge = gui
        .add_gauge(Rect::new(164, 58, 34, 34), 0.0, 0.0, 1.0, Style::progress())
        .unwrap();
    gui.set_gauge_ticks(gauge, 7, 2, true).unwrap();
    let spinner = gui
        .add_spinner(Rect::new(202, 58, 26, 26), 0.0, Style::progress())
        .unwrap();

    Ids {
        stage_label,
        progress,
        meter,
        slider,
        scroll,
        sweep_panel,
        tabs,
        dropdown,
        roller,
        gauge,
        spinner,
    }
}

fn start_stage(
    gui: &mut GuiContext<'static, 48, 48, 32>,
    animator: &mut WidgetAnimator<64, 64>,
    ids: &Ids,
    stage: i32,
) {
    gui.set_value_label(ids.stage_label, stage).unwrap();

    match stage {
        // Stage 0: long horizontal/vertical sweeps + scrolling motion.
        0 => {
            let _ = animator.animate_scroll_offset_y(ids.scroll, 0, 120, 1450, Easing::InOutSine);
            let _ = animator.animate_widget_x(ids.sweep_panel, 112, 132, 1200, Easing::InOutBack);
            let _ = animator.animate_widget_y(ids.sweep_panel, 24, 36, 1200, Easing::InOutSine);
            let _ = animator.animate_gauge_value(ids.gauge, 0.0, 1.0, 1200, Easing::OutExpo);
            let _ = animator.animate_spinner_phase(ids.spinner, 0.0, 2.2, 1300, Easing::Linear);
        }
        // Stage 1: orbit/path and accent-color shifts.
        1 => {
            let _ = animator.animate_widget_path(
                ids.spinner,
                &[
                    PathPoint::new(202.0, 58.0),
                    PathPoint::new(210.0, 64.0),
                    PathPoint::new(202.0, 74.0),
                    PathPoint::new(194.0, 64.0),
                    PathPoint::new(202.0, 58.0),
                ],
                1400,
                Easing::InOutSine,
            );
            let _ = animator.animate_corner_radius(ids.sweep_panel, 0, 6, 1200, Easing::InOutSine);
            let _ = animator.animate_accent_color(
                ids.sweep_panel,
                Rgb565::new(0, 28, 31),
                Rgb565::new(31, 18, 0),
                1200,
                Easing::InOutSine,
            );
            let _ = animator.preset_attention_shake(ids.sweep_panel, 112, 3, 800);
            let _ = animator.animate_scroll_offset_y(ids.scroll, 120, 24, 1200, Easing::OutCubic);
        }
        // Stage 2: pulse/compression with fade + springy values.
        2 => {
            let _ = animator.preset_fade_in_up(ids.sweep_panel, 36, 24, 760);
            let _ = animator.animate_widget_width(ids.sweep_panel, 48, 40, 920, Easing::InOutSine);
            let _ = animator.animate_widget_height(ids.sweep_panel, 28, 34, 920, Easing::InOutSine);
            let _ = animator.animate_opacity(ids.sweep_panel, 80, 255, 920, Easing::InOutSine);
            let _ = animator.animate_meter(ids.meter, 0.0, 1.0, 950, Easing::OutElastic);
            let _ = animator.animate_slider_value(ids.slider, 0.0, 1.0, 950, Easing::InOutBack);
            let _ = animator.animate_progress(ids.progress, 0.0, 1.0, 950, Easing::InOutSine);
        }
        // Stage 3: selector choreography + staggered sweeps.
        _ => {
            let _ = animator.animate_tab_selected(ids.tabs, 0, 3, 1100, Easing::InOutSine);
            let _ = animator.animate_dropdown_selected(ids.dropdown, 0, 7, 1100, Easing::InOutSine);
            let _ = animator.animate_roller_selected(ids.roller, 0, 7, 1100, Easing::OutCubic);
            let _ = animator.stagger_widget_x(
                &[ids.tabs, ids.dropdown],
                164,
                170,
                850,
                120,
                Easing::OutSine,
            );
            let _ = animator.animate_widget_path(
                ids.sweep_panel,
                &[
                    PathPoint::new(112.0, 24.0),
                    PathPoint::new(120.0, 30.0),
                    PathPoint::new(130.0, 24.0),
                    PathPoint::new(122.0, 20.0),
                    PathPoint::new(112.0, 24.0),
                ],
                1150,
                Easing::InOutSine,
            );
        }
    }
}
