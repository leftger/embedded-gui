use embedded_graphics_core::{
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
    prelude::DrawTarget,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 200;
const H: u32 = 120;
const PRESS_X: i32 = 20;
const PRESS_Y: i32 = 42;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("long press input showcase", &settings);

    let mut gui = GuiContext::<12, 24, 12>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);
    gui.set_long_press_threshold_ms(600);

    let mut pointer_held = false;
    let mut clicks = 0i32;
    let mut long_presses = 0i32;

    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Space => {
                        if !pointer_held {
                            gui.handle_input(InputEvent::Pointer {
                                x: PRESS_X,
                                y: PRESS_Y,
                                state: PointerState::Pressed,
                                button: PointerButton::Primary,
                            })
                            .unwrap();
                            pointer_held = true;
                            gui.set_value_label(ids.state, 1).unwrap();
                        }
                    }
                    Keycode::Return => {
                        gui.handle_input(InputEvent::Pointer {
                            x: PRESS_X,
                            y: PRESS_Y,
                            state: PointerState::Released,
                            button: PointerButton::Primary,
                        })
                        .unwrap();
                        pointer_held = false;
                        gui.set_value_label(ids.state, 0).unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        gui.tick_input(16).unwrap();
        while let Some(event) = gui.pop_event() {
            match event {
                UiEvent::LongPressed(id) if id == ids.button => {
                    long_presses += 1;
                    gui.set_value_label(ids.long_count, long_presses).unwrap();
                }
                UiEvent::Clicked(id) if id == ids.button => {
                    clicks += 1;
                    gui.set_value_label(ids.click_count, clicks).unwrap();
                }
                UiEvent::PointerReleased(id) if id == ids.button => {
                    gui.set_value_label(ids.state, 0).unwrap();
                }
                _ => {}
            }
        }

        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

struct Ids {
    button: WidgetId,
    click_count: WidgetId,
    long_count: WidgetId,
    state: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 12, 24, 12>) -> Ids {
    gui.add_panel(Rect::new(6, 6, 188, 108), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(12, 12, 176, 20),
        "SPACE=PRESS/HOLD\nENTER=RELEASE",
        Style::label(),
    )
    .unwrap();
    let button = gui
        .add_button(Rect::new(20, 36, 96, 16), "PRESS TARGET", Style::button())
        .unwrap();
    let click_count = gui
        .add_value_label(Rect::new(20, 60, 80, 12), "CLICK", 0, Style::panel())
        .unwrap();
    let long_count = gui
        .add_value_label(Rect::new(108, 60, 80, 12), "LONG", 0, Style::panel())
        .unwrap();
    let state = gui
        .add_value_label(Rect::new(20, 78, 168, 12), "IDLE", 0, Style::panel())
        .unwrap();
    Ids {
        button,
        click_count,
        long_count,
        state,
    }
}
