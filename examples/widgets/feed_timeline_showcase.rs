use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 220;
const H: u32 = 120;
static FEED: [&str; 8] = [
    "06:15 Run reminder",
    "07:30 New weather alert",
    "08:05 Meeting starts soon",
    "09:00 Hydration goal",
    "10:20 Battery saver hint",
    "11:10 Lunch checklist",
    "12:45 Walk milestone",
    "14:00 Standup notes",
];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("feed timeline showcase", &settings);
    let mut gui = GuiContext::<12, 16, 12>::new(Rect::new(0, 0, W, H));

    gui.add_panel(Rect::new(4, 4, W - 8, H - 8), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(10, 10, 200, 8),
        "UP/DOWN select  SPACE expand/collapse",
        Style::label(),
    )
    .unwrap();

    let feed = gui
        .add_feed_timeline(
            Rect::new(10, 24, 200, 88),
            &FEED,
            0,
            4,
            false,
            Style::button(),
        )
        .unwrap();
    gui.set_focus(Some(feed)).unwrap();

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
                    Keycode::Space | Keycode::Return => {
                        gui.handle_input(InputEvent::Select).unwrap()
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
