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
const H: u32 = 132;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("event dispatch showcase", &settings);

    let mut gui = GuiContext::<16, 48, 16>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);
    gui.set_dispatch_policy(
        ids.guard,
        WidgetDispatchPolicy::stop(WidgetEventFilter::POINTER, EventPhaseMask::CAPTURE),
    )
    .unwrap();

    let mut stop_capture = true;
    let mut frame_count = 0i32;
    let mut click_count = 0i32;

    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Space => {
                        // Trigger a pointer-style activation against the target button.
                        gui.handle_input(InputEvent::Pointer {
                            x: 26,
                            y: 66,
                            state: PointerState::Pressed,
                            button: PointerButton::Primary,
                        })
                        .unwrap();
                    }
                    Keycode::Return => {
                        stop_capture = !stop_capture;
                        if stop_capture {
                            gui.set_dispatch_policy(
                                ids.guard,
                                WidgetDispatchPolicy::stop(
                                    WidgetEventFilter::POINTER,
                                    EventPhaseMask::CAPTURE,
                                ),
                            )
                            .unwrap();
                            gui.set_value_label(ids.mode, 1).unwrap();
                        } else {
                            gui.clear_dispatch_policy(ids.guard).unwrap();
                            gui.set_value_label(ids.mode, 0).unwrap();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        frame_count = (frame_count + 1) % 1000;
        gui.set_value_label(ids.frames, frame_count).unwrap();

        while let Some(event) = gui.pop_event() {
            match event {
                UiEvent::PointerPressed(id) if id == ids.button => {
                    gui.set_value_label(ids.pointer_hit, 1).unwrap();
                }
                UiEvent::Clicked(id) if id == ids.button => {
                    click_count += 1;
                    gui.set_value_label(ids.clicks, click_count).unwrap();
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
    guard: WidgetId,
    button: WidgetId,
    mode: WidgetId,
    pointer_hit: WidgetId,
    clicks: WidgetId,
    frames: WidgetId,
}

fn build_ui(gui: &mut GuiContext<'static, 16, 48, 16>) -> Ids {
    let root = gui
        .add_panel(Rect::new(6, 6, 208, 120), Style::panel())
        .unwrap();
    let guard = gui
        .add_panel(Rect::new(14, 34, 192, 84), Style::panel())
        .unwrap();
    gui.add_child(root, guard).unwrap();

    gui.add_label(
        Rect::new(12, 12, 196, 20),
        "SPACE=POINTER PRESS\nENTER=TOGGLE CAPTURE STOP",
        Style::label(),
    )
    .unwrap();
    let mode = gui
        .add_value_label(Rect::new(14, 36, 64, 12), "STOP", 1, Style::panel())
        .unwrap();
    let pointer_hit = gui
        .add_value_label(Rect::new(82, 36, 64, 12), "HIT", 0, Style::panel())
        .unwrap();
    let clicks = gui
        .add_value_label(Rect::new(150, 36, 56, 12), "CLICK", 0, Style::panel())
        .unwrap();
    let frames = gui
        .add_value_label(Rect::new(150, 104, 56, 12), "FRAME", 0, Style::panel())
        .unwrap();
    let button = gui
        .add_button(Rect::new(20, 56, 104, 18), "TARGET BUTTON", Style::button())
        .unwrap();
    gui.add_child(guard, button).unwrap();

    Ids {
        guard,
        button,
        mode,
        pointer_hit,
        clicks,
        frames,
    }
}
