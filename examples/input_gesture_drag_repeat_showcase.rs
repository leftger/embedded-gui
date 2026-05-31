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
static ITEMS: [&str; 10] = [
    "ALPHA", "BETA", "GAMMA", "DELTA", "EPS", "ZETA", "ETA", "THETA", "IOTA", "KAPPA",
];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("input gesture + drag + repeat showcase", &settings);
    let mut gui = GuiContext::<24, 48, 24>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);

    gui.set_long_press_threshold_ms(450);
    gui.set_press_repeat_timing(650, 140);

    let mut hold_active = false;
    let mut drag_active = false;
    let mut pointer_x = 18;
    let mut pointer_y = 38;
    let mut clicked_count = 0;
    let mut activated_count = 0;
    let mut long_count = 0;
    let mut gesture_count = 0;

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
                        if !hold_active {
                            hold_active = true;
                            pointer_x = 18;
                            pointer_y = 38;
                            gui.handle_input(InputEvent::Pointer {
                                x: pointer_x,
                                y: pointer_y,
                                state: PointerState::Pressed,
                                button: PointerButton::Primary,
                            })
                            .unwrap();
                        }
                    }
                    Keycode::Return => {
                        if hold_active || drag_active {
                            gui.handle_input(InputEvent::Pointer {
                                x: pointer_x,
                                y: pointer_y,
                                state: PointerState::Released,
                                button: PointerButton::Primary,
                            })
                            .unwrap();
                        }
                        hold_active = false;
                        drag_active = false;
                    }
                    Keycode::D => {
                        if !drag_active {
                            drag_active = true;
                            pointer_x = 142;
                            pointer_y = 46;
                            gui.handle_input(InputEvent::Pointer {
                                x: pointer_x,
                                y: pointer_y,
                                state: PointerState::Pressed,
                                button: PointerButton::Primary,
                            })
                            .unwrap();
                        }
                    }
                    Keycode::Up => {
                        if drag_active {
                            pointer_y = (pointer_y - 6).max(36);
                            gui.handle_input(InputEvent::Pointer {
                                x: pointer_x,
                                y: pointer_y,
                                state: PointerState::Moved,
                                button: PointerButton::Primary,
                            })
                            .unwrap();
                        }
                    }
                    Keycode::Down => {
                        if drag_active {
                            pointer_y = (pointer_y + 6).min(106);
                            gui.handle_input(InputEvent::Pointer {
                                x: pointer_x,
                                y: pointer_y,
                                state: PointerState::Moved,
                                button: PointerButton::Primary,
                            })
                            .unwrap();
                        }
                    }
                    Keycode::Right => {
                        if hold_active {
                            pointer_x = (pointer_x + 8).min(118);
                            gui.handle_input(InputEvent::Pointer {
                                x: pointer_x,
                                y: pointer_y,
                                state: PointerState::Moved,
                                button: PointerButton::Primary,
                            })
                            .unwrap();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        gui.tick_input(16).unwrap();
        while let Some(event) = gui.pop_event() {
            match event {
                UiEvent::Clicked(id) if id == ids.repeat_button => {
                    clicked_count += 1;
                    gui.set_value_label(ids.clicked, clicked_count).unwrap();
                }
                UiEvent::Activate(id) if id == ids.repeat_button => {
                    activated_count += 1;
                    gui.set_value_label(ids.activated, activated_count).unwrap();
                }
                UiEvent::LongPressed(id) if id == ids.repeat_button => {
                    long_count += 1;
                    gui.set_value_label(ids.long_pressed, long_count).unwrap();
                }
                UiEvent::Gesture(id) if id == ids.repeat_button => {
                    gesture_count += 1;
                    gui.set_value_label(ids.gesture, gesture_count).unwrap();
                }
                UiEvent::Scroll { id, .. } if id == ids.scroll => {
                    let value = gui.scroll_offset(ids.scroll).unwrap_or(0);
                    gui.set_value_label(ids.scroll_value, value).unwrap();
                }
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

struct Ids {
    repeat_button: WidgetId,
    scroll: WidgetId,
    clicked: WidgetId,
    activated: WidgetId,
    long_pressed: WidgetId,
    gesture: WidgetId,
    scroll_value: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 24, 48, 24>) -> Ids {
    let shell = flat_style(Rgb565::new(5, 8, 20), Rgb565::new(10, 18, 26), Rgb565::new(0, 42, 31));
    let card = flat_style(Rgb565::new(9, 14, 28), Rgb565::new(12, 22, 30), Rgb565::new(10, 56, 31));
    let stat = flat_style(Rgb565::new(8, 10, 22), Rgb565::new(10, 18, 28), Rgb565::new(10, 56, 31));
    let button_style = flat_style(Rgb565::new(16, 24, 31), Rgb565::new(12, 22, 30), Rgb565::new(10, 56, 31));

    gui.add_panel(Rect::new(6, 6, 228, 128), shell).unwrap();
    gui.add_label(
        Rect::new(10, 10, 220, 8),
        "SPACE hold  RIGHT gesture  ENTER release",
        Style::label(),
    )
    .unwrap();
    gui.add_label(
        Rect::new(10, 20, 220, 8),
        "D drag mode  UP/DOWN drag-scroll",
        Style::label(),
    )
    .unwrap();

    let repeat_button = gui
        .add_button(Rect::new(12, 34, 112, 16), "HOLD TARGET", button_style)
        .unwrap();

    let clicked = gui
        .add_value_label(Rect::new(12, 54, 54, 12), "CLICK", 0, stat)
        .unwrap();
    let activated = gui
        .add_value_label(Rect::new(70, 54, 54, 12), "ACT", 0, stat)
        .unwrap();
    let long_pressed = gui
        .add_value_label(Rect::new(12, 70, 54, 12), "LONG", 0, stat)
        .unwrap();
    let gesture = gui
        .add_value_label(Rect::new(70, 70, 54, 12), "GEST", 0, stat)
        .unwrap();

    let scroll = gui
        .add_scroll_view(Rect::new(136, 34, 92, 72), 0, 180, card)
        .unwrap();
    let list = gui
        .add_list(Rect::new(4, 4, 84, 160), &ITEMS, 0, 8, card)
        .unwrap();
    gui.add_child(scroll, list).unwrap();
    let scroll_value = gui
        .add_value_label(Rect::new(136, 110, 92, 12), "SCROLL", 0, stat)
        .unwrap();

    Ids {
        repeat_button,
        scroll,
        clicked,
        activated,
        long_pressed,
        gesture,
        scroll_value,
    }
}

fn flat_style(background: Rgb565, border: Rgb565, accent: Rgb565) -> Style {
    Style {
        background: Some(background),
        gradient: None,
        font: FontId::Medium4x7,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent,
        opacity: 255,
        corner_radius: 1,
        shadow: None,
        border: Border::one(border),
        padding: EdgeInsets::all(1),
    }
}
