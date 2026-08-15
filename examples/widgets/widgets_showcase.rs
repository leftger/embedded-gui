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
const LEFT_GROUP: FocusGroupId = FocusGroupId::new(1);
const RIGHT_GROUP: FocusGroupId = FocusGroupId::new(2);
static LIST_ITEMS: [&str; 6] = ["ALPHA", "BETA", "GAMMA", "DELTA", "EPS", "ZETA"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("embedded-gui widgets showcase", &settings);

    let mut gui = GuiContext::<24, 24, 16>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);

    'running: loop {
        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Tab => toggle_group(&mut gui),
                    Keycode::Up => gui.handle_input(InputEvent::Up).unwrap(),
                    Keycode::Down => gui.handle_input(InputEvent::Down).unwrap(),
                    Keycode::Left => gui.handle_input(InputEvent::Left).unwrap(),
                    Keycode::Right => gui.handle_input(InputEvent::Right).unwrap(),
                    Keycode::Return | Keycode::Space => {
                        gui.handle_input(InputEvent::Select).unwrap()
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        while let Some(event) = gui.pop_event() {
            match event {
                UiEvent::ValueChanged(id) if id == ids.slider => {
                    let pct = (gui.slider_value(ids.slider).unwrap_or(0.0) * 100.0) as i32;
                    gui.set_value_label(ids.value_label, pct).unwrap();
                }
                UiEvent::ValueChanged(id) if id == ids.list => {
                    let selected = gui.list_selected(ids.list).unwrap_or(0) as i32;
                    gui.set_value_label(ids.list_value, selected).unwrap();
                }
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

struct ShowcaseIds {
    slider: WidgetId,
    value_label: WidgetId,
    list: WidgetId,
    list_value: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 24, 24, 16>) -> ShowcaseIds {
    gui.add_panel(Rect::new(4, 4, 184, 112), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(10, 10, 172, 10),
        "WIDGETS: TAB SWITCHES GROUP",
        Style::label(),
    )
    .unwrap();

    let toggle = gui
        .add_toggle(Rect::new(10, 28, 74, 14), "POWER", true, Style::button())
        .unwrap();
    let checkbox = gui
        .add_checkbox(Rect::new(10, 46, 74, 14), "SYNC", false, Style::button())
        .unwrap();
    let slider = gui
        .add_slider(Rect::new(10, 66, 74, 14), 0.42, 0.0, 1.0, Style::button())
        .unwrap();
    let value_label = gui
        .add_value_label(Rect::new(10, 86, 74, 14), "VOL", 42, Style::panel())
        .unwrap();

    let list = gui
        .add_list(
            Rect::new(96, 28, 82, 52),
            &LIST_ITEMS,
            0,
            4,
            Style::button(),
        )
        .unwrap();
    let list_value = gui
        .add_value_label(Rect::new(96, 86, 82, 14), "SEL", 0, Style::panel())
        .unwrap();
    let icon = gui
        .add_icon_button(Rect::new(96, 102, 82, 12), '>', "LAUNCH", Style::button())
        .unwrap();

    for id in [toggle, checkbox, slider, value_label] {
        gui.set_focus_group(id, LEFT_GROUP).unwrap();
    }
    for id in [list, list_value, icon] {
        gui.set_focus_group(id, RIGHT_GROUP).unwrap();
    }
    gui.set_active_focus_group(Some(LEFT_GROUP));

    ShowcaseIds {
        slider,
        value_label,
        list,
        list_value,
    }
}

fn toggle_group(gui: &mut GuiContext<'static, 24, 24, 16>) {
    let next = if gui.focus().is_some_and(|id| {
        gui.widgets()
            .iter()
            .find(|node| node.id == id)
            .is_some_and(|node| node.focus_group == LEFT_GROUP)
    }) {
        RIGHT_GROUP
    } else {
        LEFT_GROUP
    };
    gui.set_active_focus_group(Some(next));
}
