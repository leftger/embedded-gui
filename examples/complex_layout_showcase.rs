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
    let mut window = Window::new("complex layout showcase", &settings);
    let mut gui = GuiContext::<32, 64, 32>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);

    relayout(&mut gui, &ids, false);
    let mut use_flex = false;

    'running: loop {
        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Space => {
                        use_flex = !use_flex;
                        relayout(&mut gui, &ids, use_flex);
                        let _ = gui.set_value_label(ids.mode, if use_flex { 1 } else { 0 });
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
    stat_a: WidgetId,
    stat_b: WidgetId,
    stat_c: WidgetId,
    panel_a: WidgetId,
    panel_b: WidgetId,
    panel_c: WidgetId,
    mode: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 32, 64, 32>) -> Ids {
    let root = gui
        .add_panel(Rect::new(6, 6, 228, 128), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(10, 10, 220, 18),
        "SPACE toggles between intrinsic and flex layout",
        Style::label(),
    )
    .unwrap();
    let mode = gui
        .add_value_label(Rect::new(10, 28, 80, 12), "MODE", 0, Style::panel())
        .unwrap();

    let sidebar = gui
        .add_panel(Rect::new(10, 44, 64, 80), Style::panel())
        .unwrap();
    let content = gui
        .add_panel(Rect::new(78, 44, 154, 80), Style::panel())
        .unwrap();
    gui.add_child(root, sidebar).unwrap();
    gui.add_child(root, content).unwrap();

    let stat_a = gui
        .add_value_label(Rect::new(0, 0, 1, 1), "CPU", 42, Style::panel())
        .unwrap();
    let stat_b = gui
        .add_value_label(Rect::new(0, 0, 1, 1), "RAM", 68, Style::panel())
        .unwrap();
    let stat_c = gui
        .add_value_label(Rect::new(0, 0, 1, 1), "NET", 17, Style::panel())
        .unwrap();
    gui.add_child(sidebar, stat_a).unwrap();
    gui.add_child(sidebar, stat_b).unwrap();
    gui.add_child(sidebar, stat_c).unwrap();

    let panel_a = gui
        .add_button(Rect::new(0, 0, 1, 1), "CHART", Style::button())
        .unwrap();
    let panel_b = gui
        .add_button(Rect::new(0, 0, 1, 1), "DETAILS", Style::button())
        .unwrap();
    let panel_c = gui
        .add_button(Rect::new(0, 0, 1, 1), "ACTIONS", Style::button())
        .unwrap();
    gui.add_child(content, panel_a).unwrap();
    gui.add_child(content, panel_b).unwrap();
    gui.add_child(content, panel_c).unwrap();

    Ids {
        stat_a,
        stat_b,
        stat_c,
        panel_a,
        panel_b,
        panel_c,
        mode,
    }
}

fn relayout(gui: &mut GuiContext<'static, 32, 64, 32>, ids: &Ids, use_flex: bool) {
    let _ = gui.apply_layout(
        LinearLayout::column().with_gap(2).with_padding(EdgeInsets::all(2)),
        Rect::new(0, 0, 64, 80),
        &[ids.stat_a, ids.stat_b, ids.stat_c],
    );

    if use_flex {
        let items = [
            LayoutItem::length(18).with_grow(0),
            LayoutItem::fill_weight(2),
            LayoutItem::fill_weight(1),
        ];
        let _ = gui.apply_layout_flex(
            LinearLayout::column().with_gap(2).with_padding(EdgeInsets::all(2)),
            Rect::new(0, 0, 154, 80),
            &[ids.panel_a, ids.panel_b, ids.panel_c],
            &items,
            true,
            true,
        );
    } else {
        let _ = gui.apply_layout_intrinsic_with_cross(
            LinearLayout::column().with_gap(2).with_padding(EdgeInsets::all(2)),
            Rect::new(0, 0, 154, 80),
            &[ids.panel_a, ids.panel_b, ids.panel_c],
            true,
        );
    }
    let _ = gui.set_value_label(ids.mode, if use_flex { 1 } else { 0 });
}
