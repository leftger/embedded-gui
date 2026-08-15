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
static ACTIONS: [&str; 3] = ["Open", "Snooze", "Dismiss"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("notification primitives showcase", &settings);
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, W, H));

    gui.add_panel(Rect::new(4, 4, W - 8, H - 8), Style::panel())
        .unwrap();
    let banner = gui
        .add_heads_up_banner(
            Rect::new(10, 10, 200, 16),
            NotificationLevel::Info,
            "Message from phone: Build finished.",
            1800,
            Style::panel(),
        )
        .unwrap();
    let sheet = gui
        .add_notification_action_sheet(
            Rect::new(20, 34, 180, 76),
            NotificationLevel::Warning,
            "Battery low",
            "Battery below 10%. Choose an action.",
            &ACTIONS,
            0,
            true,
            Style::panel(),
        )
        .unwrap();

    let mut selected = 0usize;

    'running: loop {
        gui.tick_heads_up(banner, 16).unwrap();

        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Left => {
                        selected = selected.saturating_sub(1);
                        gui.set_notification_sheet_selected(sheet, selected)
                            .unwrap();
                    }
                    Keycode::Right => {
                        selected = (selected + 1).min(ACTIONS.len() - 1);
                        gui.set_notification_sheet_selected(sheet, selected)
                            .unwrap();
                    }
                    Keycode::Space => {
                        gui.set_notification_sheet_open(sheet, false).unwrap();
                        gui.set_heads_up_ttl(banner, 1200).unwrap();
                    }
                    Keycode::R => {
                        gui.set_notification_sheet_open(sheet, true).unwrap();
                        gui.set_notification_sheet_selected(sheet, 0).unwrap();
                        selected = 0;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
