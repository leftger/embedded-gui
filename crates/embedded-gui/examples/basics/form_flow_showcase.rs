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
const H: u32 = 136;
static LEVELS: [&str; 3] = ["LOW", "MED", "HIGH"];
static MODES: [&str; 3] = ["AUTO", "MANUAL", "SAFE"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("form flow showcase", &settings);

    let mut gui = GuiContext::<32, 96, 32>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);
    gui.set_state_transition_duration_ms(80);

    'running: loop {
        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Up => gui.handle_input(InputEvent::Up).unwrap(),
                    Keycode::Down => gui.handle_input(InputEvent::Down).unwrap(),
                    Keycode::Left => gui.handle_input(InputEvent::Left).unwrap(),
                    Keycode::Right => gui.handle_input(InputEvent::Right).unwrap(),
                    Keycode::Return => gui.handle_input(InputEvent::Select).unwrap(),
                    Keycode::Backspace => gui.handle_input(InputEvent::Back).unwrap(),
                    Keycode::Num1 => gui.textarea_insert_char(ids.note, '1').unwrap(),
                    Keycode::Num2 => gui.textarea_insert_char(ids.note, '2').unwrap(),
                    Keycode::Num3 => gui.textarea_insert_char(ids.note, '3').unwrap(),
                    _ => {}
                },
                _ => {}
            }
        }

        gui.tick_input(16).unwrap();
        while let Some(event) = gui.pop_event() {
            if let UiEvent::Activate(id) = event {
                if id == ids.apply {
                    let summary = if gui.checked_value(ids.enable).unwrap_or(false) {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    };
                    let _ = gui.set_textarea_text(ids.status, summary);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

struct Ids {
    enable: WidgetId,
    note: WidgetId,
    apply: WidgetId,
    status: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 32, 96, 32>) -> Ids {
    gui.add_panel(Rect::new(6, 6, 228, 124), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(10, 10, 220, 18),
        "UP/DOWN focus  LEFT/RIGHT value  ENTER select\nBACKSPACE closes dropdown  1/2/3 type",
        Style::label(),
    )
    .unwrap();

    let enable = gui
        .add_checkbox(Rect::new(12, 38, 100, 16), "ENABLE", false, Style::button())
        .unwrap();
    let _mode = gui
        .add_dropdown(Rect::new(116, 38, 108, 16), &MODES, 0, Style::button())
        .unwrap();
    let _level = gui
        .add_roller(Rect::new(12, 58, 100, 28), &LEVELS, 1, Style::button())
        .unwrap();
    let note = gui
        .add_textarea(Rect::new(116, 58, 108, 28), "N0TE", "NOTE", Style::panel())
        .unwrap();
    let apply = gui
        .add_button(Rect::new(12, 92, 100, 16), "APPLY", Style::button())
        .unwrap();
    let status = gui
        .add_textarea(Rect::new(116, 92, 108, 16), "IDLE", "-", Style::panel())
        .unwrap();
    gui.set_textarea_capabilities(status, true, true, false)
        .unwrap();

    Ids {
        enable,
        note,
        apply,
        status,
    }
}
