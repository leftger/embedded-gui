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
static ITEMS: [&str; 3] = ["ONE", "TWO", "THREE"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("raw key input showcase", &settings);

    let mut gui = GuiContext::<24, 64, 24>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);
    gui.set_widget_key_input_policy(
        ids.button,
        WidgetKeyInputPolicy {
            raw_select: true,
            raw_back: false,
        },
    )
    .unwrap();
    gui.set_widget_key_input_policy(
        ids.dropdown,
        WidgetKeyInputPolicy {
            raw_select: false,
            raw_back: true,
        },
    )
    .unwrap();
    gui.set_focus(Some(ids.button)).unwrap();

    'running: loop {
        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Tab => gui.handle_input(InputEvent::Down).unwrap(),
                    Keycode::S => gui.handle_input(InputEvent::SelectPressed).unwrap(),
                    Keycode::Return => gui.handle_input(InputEvent::SelectReleased).unwrap(),
                    Keycode::B => gui.handle_input(InputEvent::BackPressed).unwrap(),
                    Keycode::Backspace => gui.handle_input(InputEvent::BackReleased).unwrap(),
                    _ => {}
                },
                _ => {}
            }
        }

        gui.tick_input(16).unwrap();
        while let Some(event) = gui.pop_event() {
            match event {
                UiEvent::Pressed(id) if id == ids.button || id == ids.dropdown => {
                    gui.set_value_label(ids.pressed_count, gui.value_of(ids.pressed_count) + 1)
                        .unwrap();
                }
                UiEvent::Released(id) if id == ids.button || id == ids.dropdown => {
                    gui.set_value_label(ids.released_count, gui.value_of(ids.released_count) + 1)
                        .unwrap();
                }
                UiEvent::Clicked(id) if id == ids.button => {
                    gui.set_value_label(ids.clicked_count, gui.value_of(ids.clicked_count) + 1)
                        .unwrap();
                }
                UiEvent::Back => {
                    gui.set_value_label(ids.back_count, gui.value_of(ids.back_count) + 1)
                        .unwrap();
                }
                UiEvent::Closed(id) if id == ids.dropdown => {
                    gui.set_value_label(ids.closed_count, gui.value_of(ids.closed_count) + 1)
                        .unwrap();
                }
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

struct Ids {
    button: WidgetId,
    dropdown: WidgetId,
    pressed_count: WidgetId,
    released_count: WidgetId,
    clicked_count: WidgetId,
    back_count: WidgetId,
    closed_count: WidgetId,
}

impl Ids {
    fn counters(&self) -> [WidgetId; 5] {
        [
            self.pressed_count,
            self.released_count,
            self.clicked_count,
            self.back_count,
            self.closed_count,
        ]
    }
}

trait CounterAccess {
    fn value_of(&self, id: WidgetId) -> i32;
}

impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize> CounterAccess
    for GuiContext<'a, NODES, EVENTS, DIRTY>
{
    fn value_of(&self, id: WidgetId) -> i32 {
        self.widgets()
            .iter()
            .find(|w| w.id == id)
            .and_then(|w| {
                if let WidgetKind::ValueLabel { value, .. } = w.kind {
                    Some(value)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }
}

fn build_ui(gui: &mut GuiContext<'static, 24, 64, 24>) -> Ids {
    gui.add_panel(Rect::new(6, 6, 208, 116), Style::panel()).unwrap();
    gui.add_label(
        Rect::new(10, 10, 198, 24),
        "TAB focus\nS=SelectPressed ENTER=SelectReleased\nB=BackPressed BACKSPACE=BackReleased",
        Style::label(),
    )
    .unwrap();

    let button = gui
        .add_button(Rect::new(12, 42, 94, 16), "RAW SELECT", Style::button())
        .unwrap();
    let dropdown = gui
        .add_dropdown(Rect::new(112, 42, 96, 16), &ITEMS, 0, Style::button())
        .unwrap();
    gui.set_dropdown_open(dropdown, true).unwrap();

    let pressed_count = gui
        .add_value_label(Rect::new(12, 64, 62, 12), "PRESS", 0, Style::panel())
        .unwrap();
    let released_count = gui
        .add_value_label(Rect::new(78, 64, 62, 12), "RELEASE", 0, Style::panel())
        .unwrap();
    let clicked_count = gui
        .add_value_label(Rect::new(144, 64, 62, 12), "CLICK", 0, Style::panel())
        .unwrap();
    let back_count = gui
        .add_value_label(Rect::new(12, 80, 94, 12), "BACK", 0, Style::panel())
        .unwrap();
    let closed_count = gui
        .add_value_label(Rect::new(112, 80, 96, 12), "CLOSED", 0, Style::panel())
        .unwrap();

    let ids = Ids {
        button,
        dropdown,
        pressed_count,
        released_count,
        clicked_count,
        back_count,
        closed_count,
    };

    for id in ids.counters() {
        gui.set_value_label(id, 0).unwrap();
    }
    ids
}
