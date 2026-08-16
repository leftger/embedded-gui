use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 240;
const H: u32 = 140;
static MENU_ITEMS: [&str; 4] = ["Status", "Sensors", "Network", "About"];
static DROPDOWN_ITEMS: [&str; 4] = ["Fast", "Balanced", "Eco", "Turbo"];
static FEED_ITEMS: [&str; 5] = [
    "08:00 Sync complete",
    "09:15 Alert delivered",
    "10:05 Meeting reminder",
    "11:20 Battery saver tip",
    "12:30 Lunch timer",
];
static SHEET_ACTIONS: [&str; 3] = ["Open", "Later", "Dismiss"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("menu contract showcase", &settings);
    let mut gui = GuiContext::<24, 32, 24>::new(Rect::new(0, 0, W, H));

    gui.add_panel(Rect::new(4, 4, W - 8, H - 8), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(8, 8, 224, 10),
        "1 wrap | 2 select->dropdown | 3 back->dropdown | 4 back->sheet | 5 select->feed",
        Style::label(),
    )
    .unwrap();

    let menu = gui
        .add_menu(Rect::new(8, 22, 74, 48), &MENU_ITEMS, 0, Style::button())
        .unwrap();
    let dropdown = gui
        .add_dropdown(
            Rect::new(88, 22, 64, 14),
            &DROPDOWN_ITEMS,
            0,
            Style::button(),
        )
        .unwrap();
    let feed = gui
        .add_feed_timeline(
            Rect::new(88, 40, 144, 62),
            &FEED_ITEMS,
            0,
            3,
            false,
            Style::button(),
        )
        .unwrap();
    let sheet = gui
        .add_notification_action_sheet(
            Rect::new(8, 74, 74, 56),
            NotificationLevel::Info,
            "Notify",
            "Use Back to test close behavior.",
            &SHEET_ACTIONS,
            0,
            true,
            Style::panel(),
        )
        .unwrap();
    gui.set_focus(Some(menu)).unwrap();

    let mut contract = MenuContract::default();
    let mut focus_cycle = [menu, dropdown, feed, sheet].into_iter().cycle();

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
                    Keycode::Backspace => gui.handle_input(InputEvent::Back).unwrap(),
                    Keycode::Return | Keycode::Space => {
                        gui.handle_input(InputEvent::Select).unwrap()
                    }
                    Keycode::Tab => {
                        if let Some(next) = focus_cycle.next() {
                            gui.set_focus(Some(next)).unwrap();
                        }
                    }
                    Keycode::Num1 => {
                        contract.wrap_navigation = !contract.wrap_navigation;
                        gui.set_menu_contract(contract);
                    }
                    Keycode::Num2 => {
                        contract.select_opens_dropdown = !contract.select_opens_dropdown;
                        gui.set_menu_contract(contract);
                    }
                    Keycode::Num3 => {
                        contract.back_closes_dropdown = !contract.back_closes_dropdown;
                        gui.set_menu_contract(contract);
                    }
                    Keycode::Num4 => {
                        contract.back_closes_notification_sheet =
                            !contract.back_closes_notification_sheet;
                        gui.set_menu_contract(contract);
                    }
                    Keycode::Num5 => {
                        contract.select_toggles_feed_expanded =
                            !contract.select_toggles_feed_expanded;
                        gui.set_menu_contract(contract);
                    }
                    Keycode::R => {
                        gui.set_dropdown_open(dropdown, true).unwrap();
                        gui.set_notification_sheet_open(sheet, true).unwrap();
                        gui.set_feed_expanded(feed, false).unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
