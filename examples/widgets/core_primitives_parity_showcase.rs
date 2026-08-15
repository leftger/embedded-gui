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
static CHART_SERIES: [f32; 8] = [0.1, 0.4, 0.2, 0.7, 0.5, 0.8, 0.3, 0.6];
static TABLE_ROWS: [&[&str]; 3] = [&["CPU", "64"], &["MEM", "73"], &["NET", "41"]];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("core primitives parity showcase", &settings);
    let mut gui = GuiContext::<24, 24, 24>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);
    let mut bars_mode = false;
    let mut clip_children = true;
    let mut gauge_value = 0.4f32;

    'running: loop {
        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Num1 => {
                        bars_mode = !bars_mode;
                        let mode = if bars_mode {
                            ChartMode::Bars
                        } else {
                            ChartMode::Line
                        };
                        gui.set_chart_decoration(ids.chart, mode, true, true, true)
                            .unwrap();
                    }
                    Keycode::Num2 => {
                        clip_children = !clip_children;
                        gui.set_flag(ids.clip_panel, WidgetFlags::CLIP_CHILDREN, clip_children)
                            .unwrap();
                    }
                    Keycode::Num3 => {
                        gauge_value = (gauge_value + 0.1).fract();
                        gui.set_gauge_value(ids.gauge, gauge_value).unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

struct Ids {
    chart: WidgetId,
    gauge: WidgetId,
    clip_panel: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 24, 24, 24>) -> Ids {
    gui.add_panel(Rect::new(6, 6, 228, 128), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(10, 10, 220, 10),
        "[1] chart mode [2] clip toggle [3] gauge value",
        Style::label(),
    )
    .unwrap();

    let chart = gui
        .add_chart(
            Rect::new(12, 24, 104, 46),
            &CHART_SERIES,
            0.0,
            1.0,
            Style::panel(),
        )
        .unwrap();
    gui.set_chart_style(chart, 2, true, true).unwrap();
    gui.set_chart_decoration(chart, ChartMode::Line, true, true, true)
        .unwrap();

    let gauge = gui
        .add_arc_gauge(
            Rect::new(124, 24, 48, 48),
            0.4,
            0.0,
            1.0,
            135,
            405,
            2,
            true,
            Style::progress(),
        )
        .unwrap();
    gui.set_gauge_ticks(gauge, 6, 2, true).unwrap();

    let table = gui
        .add_table(Rect::new(176, 24, 52, 46), &TABLE_ROWS, Style::panel())
        .unwrap();
    gui.set_table_style(table, true, 1, TextAlign::Center)
        .unwrap();

    let clip_panel = gui
        .add_panel(Rect::new(12, 78, 90, 42), Style::panel())
        .unwrap();
    let overflowing = gui
        .add_label(Rect::new(68, 14, 80, 10), "CLIP DEMO", Style::label())
        .unwrap();
    gui.add_child(clip_panel, overflowing).unwrap();

    Ids {
        chart,
        gauge,
        clip_panel,
    }
}
