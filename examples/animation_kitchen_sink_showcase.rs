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
static TABS: [&str; 3] = ["CPU", "GFX", "NET"];
static ITEMS: [&str; 5] = ["ONE", "TWO", "THREE", "FOUR", "FIVE"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("animation kitchen sink showcase", &settings);
    let mut gui = GuiContext::<32, 32, 24>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);

    let mut animator = WidgetAnimator::<32, 32>::new();
    start_batch(&mut animator, &ids);

    'running: loop {
        animator.tick(16, &mut gui).unwrap();
        gui.tick_spinner(ids.spinner, 16, 0.35).unwrap();
        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Space => start_batch(&mut animator, &ids),
                    _ => {}
                },
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

struct Ids {
    progress: WidgetId,
    meter: WidgetId,
    slider: WidgetId,
    scroll: WidgetId,
    panel: WidgetId,
    tabs: WidgetId,
    dropdown: WidgetId,
    roller: WidgetId,
    gauge: WidgetId,
    spinner: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 32, 32, 24>) -> Ids {
    gui.add_panel(Rect::new(6, 6, 228, 128), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(12, 10, 216, 8),
        "SPACE restart animation batch",
        Style::label(),
    )
    .unwrap();

    let progress = gui
        .add_progress_bar(Rect::new(12, 24, 92, 10), 0.0, Style::progress())
        .unwrap();
    let meter = gui
        .add_meter(Rect::new(12, 38, 42, 26), 0.0, 0.0, 1.0, Style::progress())
        .unwrap();
    let slider = gui
        .add_slider(Rect::new(60, 44, 44, 12), 0.0, 0.0, 1.0, Style::button())
        .unwrap();
    let scroll = gui
        .add_scroll_view(Rect::new(12, 68, 92, 44), 0, 140, Style::panel())
        .unwrap();
    let list = gui
        .add_list(Rect::new(4, 4, 84, 120), &ITEMS, 0, 4, Style::button())
        .unwrap();
    gui.add_child(scroll, list).unwrap();

    let panel = gui
        .add_panel(Rect::new(110, 24, 48, 26), Style::panel())
        .unwrap();
    let tabs = gui
        .add_tabs(Rect::new(162, 24, 66, 12), &TABS, 0, Style::button())
        .unwrap();
    let dropdown = gui
        .add_dropdown(Rect::new(162, 40, 66, 12), &ITEMS, 0, Style::button())
        .unwrap();
    let roller = gui
        .add_roller(Rect::new(110, 54, 48, 58), &ITEMS, 0, Style::button())
        .unwrap();
    let gauge = gui
        .add_gauge(Rect::new(162, 58, 34, 34), 0.0, 0.0, 1.0, Style::progress())
        .unwrap();
    let spinner = gui
        .add_spinner(Rect::new(202, 58, 26, 26), 0.0, Style::progress())
        .unwrap();
    gui.set_gauge_ticks(gauge, 6, 2, true).unwrap();

    Ids {
        progress,
        meter,
        slider,
        scroll,
        panel,
        tabs,
        dropdown,
        roller,
        gauge,
        spinner,
    }
}

fn start_batch(animator: &mut WidgetAnimator<32, 32>, ids: &Ids) {
    let _ = animator.ping_pong_progress(ids.progress, 0.05, 0.95, 1200, Easing::InOutSine);
    let _ = animator.animate_meter(ids.meter, 0.0, 1.0, 1000, Easing::InOutCubic);
    let _ = animator.animate_slider_value(ids.slider, 0.0, 1.0, 900, Easing::InOutSine);
    let _ = animator.animate_scroll_offset_y(ids.scroll, 0, 80, 1200, Easing::OutCubic);
    let _ = animator.animate_widget_x(ids.panel, 110, 128, 900, Easing::InOutBack);
    let _ = animator.animate_widget_width(ids.panel, 48, 40, 900, Easing::InOutSine);
    let _ = animator.animate_widget_height(ids.panel, 26, 32, 900, Easing::InOutSine);
    let _ = animator.bind_property(
        ids.panel,
        AnimatedProperty::WidgetY,
        Animation::new(24.0, 32.0, 700, Easing::InOutSine)
            .with_repeat_mode(RepeatMode::PingPong)
            .with_repeat_count(None),
    );
    let _ = animator.animate_tab_selected(ids.tabs, 0, 2, 1200, Easing::InOutSine);
    let _ = animator.animate_dropdown_selected(ids.dropdown, 0, 4, 1200, Easing::InOutSine);
    let _ = animator.animate_roller_selected(ids.roller, 0, 4, 1200, Easing::InOutSine);
    let _ = animator.animate_gauge_value(ids.gauge, 0.0, 1.0, 1200, Easing::OutExpo);
    let _ = animator.animate_spinner_phase(ids.spinner, 0.0, 2.0, 1200, Easing::Linear);
    let _ = animator.stagger_widget_x(
        &[ids.tabs, ids.dropdown],
        162,
        168,
        900,
        120,
        Easing::OutSine,
    );
    let _ = animator.preset_fade_in_up(ids.panel, 32, 24, 700);
    let _ = animator.preset_attention_shake(ids.panel, 128, 2, 700);
    let _ = animator.animate_corner_radius(ids.panel, 0, 4, 900, Easing::InOutSine);
    let _ = animator.animate_accent_color(
        ids.panel,
        Rgb565::new(0, 42, 31),
        Rgb565::new(31, 31, 0),
        900,
        Easing::InOutSine,
    );
    let _ = animator.animate_widget_path(
        ids.spinner,
        &[
            PathPoint::new(202.0, 58.0),
            PathPoint::new(206.0, 62.0),
            PathPoint::new(202.0, 66.0),
            PathPoint::new(198.0, 62.0),
            PathPoint::new(202.0, 58.0),
        ],
        1200,
        Easing::InOutSine,
    );
}
