use embedded_graphics_core::{
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
    prelude::DrawTarget,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 220;
const H: u32 = 128;
static ITEMS: [&str; 4] = ["ALPHA", "BETA", "GAMMA", "DELTA"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("interaction semantics showcase", &settings);

    let mut gui = GuiContext::<24, 64, 24>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);
    gui.set_state_transition_duration_ms(90);

    let mut pointer_down = false;
    let mut clicked = 0;
    let mut closed = 0;
    let mut released = 0;

    'running: loop {
        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Tab => {
                        gui.handle_input(InputEvent::Down).unwrap();
                    }
                    Keycode::Return => {
                        gui.handle_input(InputEvent::Select).unwrap();
                    }
                    Keycode::Backspace => {
                        gui.handle_input(InputEvent::Back).unwrap();
                    }
                    // SPACE toggles a pointer hold over the top button.
                    Keycode::Space => {
                        pointer_down = !pointer_down;
                        gui.handle_input(InputEvent::Pointer {
                            x: 16,
                            y: 58,
                            state: if pointer_down {
                                PointerState::Pressed
                            } else {
                                PointerState::Released
                            },
                            button: PointerButton::Primary,
                        })
                        .unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        gui.tick_input(16).unwrap();
        while let Some(event) = gui.pop_event() {
            match event {
                UiEvent::Clicked(id) if id == ids.button => {
                    clicked += 1;
                    gui.set_value_label(ids.clicks, clicked).unwrap();
                }
                UiEvent::Closed(id) if id == ids.dropdown => {
                    closed += 1;
                    gui.set_value_label(ids.closed, closed).unwrap();
                }
                UiEvent::Released(id) if id == ids.button => {
                    released += 1;
                    gui.set_value_label(ids.released, released).unwrap();
                }
                _ => {}
            }
        }

        gui.set_value_label(ids.transitions, gui.active_state_transitions() as i32)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

struct Ids {
    button: WidgetId,
    dropdown: WidgetId,
    clicks: WidgetId,
    closed: WidgetId,
    released: WidgetId,
    transitions: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 24, 64, 24>) -> Ids {
    gui.add_panel(Rect::new(6, 6, 208, 116), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(10, 10, 200, 20),
        "TAB focus  ENTER select  BACKSPACE closes dropdown\nSPACE pointer press/release",
        Style::label(),
    )
    .unwrap();

    let button = gui
        .add_button(Rect::new(12, 50, 96, 18), "PRESS TARGET", Style::button())
        .unwrap();
    let dropdown = gui
        .add_dropdown(Rect::new(112, 50, 96, 18), &ITEMS, 0, Style::button())
        .unwrap();

    let clicks = gui
        .add_value_label(Rect::new(12, 76, 46, 12), "CLICK", 0, Style::panel())
        .unwrap();
    let closed = gui
        .add_value_label(Rect::new(62, 76, 46, 12), "CLOSE", 0, Style::panel())
        .unwrap();
    let released = gui
        .add_value_label(Rect::new(112, 76, 46, 12), "REL", 0, Style::panel())
        .unwrap();
    let transitions = gui
        .add_value_label(Rect::new(162, 76, 46, 12), "TRANS", 0, Style::panel())
        .unwrap();

    Ids {
        button,
        dropdown,
        clicks,
        closed,
        released,
        transitions,
    }
}
