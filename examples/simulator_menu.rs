use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;
use heapless::Vec;

const W: u32 = 160;
const H: u32 = 96;
const MAIN: ScreenId = ScreenId::new(1);
const SETTINGS: ScreenId = ScreenId::new(2);
const HUD: ScreenId = ScreenId::new(3);
static MAIN_ITEMS: [&str; 4] = ["PLAY", "SETTINGS", "HUD", "QUIT"];
static SETTINGS_ITEMS: [&str; 3] = ["VIDEO", "AUDIO", "INPUT"];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("embedded-gui menu", &settings);

    let mut gui = GuiContext::<16, 16, 12>::new(Rect::new(0, 0, W, H));
    let mut lifecycle = Vec::<ScreenLifecycleEvent, 8>::new();
    let mut screens = ScreenStack::<4>::with_root_lifecycle(MAIN, &mut lifecycle).unwrap();
    let mut main_menu = None;
    let mut settings_toggle = None;
    build_screen(
        &mut gui,
        screens.current().unwrap(),
        &mut main_menu,
        &mut settings_toggle,
    );

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
                    Keycode::Return | Keycode::Space => {
                        gui.handle_input(InputEvent::Select).unwrap()
                    }
                    Keycode::Backspace => gui.handle_input(InputEvent::Back).unwrap(),
                    _ => {}
                },
                _ => {}
            }
        }

        while let Some(event) = gui.pop_event() {
            match event {
                UiEvent::Activate(id) => {
                    if Some(id) == main_menu {
                        match gui.menu_selected(id) {
                            Some(1) => {
                                screens
                                    .apply_lifecycle(ScreenCommand::Push(SETTINGS), &mut lifecycle)
                                    .unwrap();
                            }
                            Some(2) => {
                                screens
                                    .apply_lifecycle(ScreenCommand::Push(HUD), &mut lifecycle)
                                    .unwrap();
                            }
                            Some(3) => break 'running,
                            _ => {}
                        }
                    } else if Some(id) == settings_toggle {
                        // The context toggles this automatically on activation.
                    }
                }
                UiEvent::Back => {
                    if screens.len() > 1 {
                        screens
                            .apply_lifecycle(ScreenCommand::Pop, &mut lifecycle)
                            .unwrap();
                    }
                }
                _ => {}
            }

            if !matches!(
                event,
                UiEvent::FocusChanged { .. } | UiEvent::ValueChanged(_)
            ) {
                main_menu = None;
                settings_toggle = None;
                build_screen(
                    &mut gui,
                    screens.current().unwrap_or(MAIN),
                    &mut main_menu,
                    &mut settings_toggle,
                );
            }
        }
        lifecycle.clear();

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn build_screen(
    gui: &mut GuiContext<'static, 16, 16, 12>,
    screen: ScreenId,
    main_menu: &mut Option<WidgetId>,
    settings_toggle: &mut Option<WidgetId>,
) {
    gui.clear_widgets().unwrap();
    gui.add_panel(Rect::new(4, 4, 152, 88), Style::panel())
        .unwrap();

    match screen {
        MAIN => {
            gui.add_label(Rect::new(10, 10, 140, 12), "EMBEDDED GUI", Style::label())
                .unwrap();
            *main_menu = Some(
                gui.add_menu(Rect::new(10, 26, 96, 44), &MAIN_ITEMS, 0, Style::button())
                    .unwrap(),
            );
            gui.add_progress_bar(Rect::new(10, 78, 96, 8), 0.72, Style::progress())
                .unwrap();
        }
        SETTINGS => {
            gui.add_label(Rect::new(10, 10, 140, 12), "SETTINGS", Style::label())
                .unwrap();
            let toggle = gui
                .add_toggle(Rect::new(10, 26, 98, 14), "DITHER", true, Style::button())
                .unwrap();
            *settings_toggle = Some(toggle);
            gui.add_slider(Rect::new(10, 46, 98, 14), 0.65, 0.0, 1.0, Style::button())
                .unwrap();
            gui.add_list(
                Rect::new(112, 24, 38, 44),
                &SETTINGS_ITEMS,
                0,
                3,
                Style::panel(),
            )
            .unwrap();
            gui.add_label(Rect::new(10, 78, 130, 8), "BACKSPACE: BACK", Style::label())
                .unwrap();
        }
        HUD => {
            gui.add_label(Rect::new(10, 10, 140, 12), "HUD PREVIEW", Style::label())
                .unwrap();
            gui.add_value_label(Rect::new(10, 28, 82, 12), "HP", 87, Style::panel())
                .unwrap();
            gui.add_checkbox(Rect::new(10, 46, 84, 14), "RADAR", true, Style::button())
                .unwrap();
            gui.add_icon_button(Rect::new(10, 66, 84, 14), '>', "START", Style::button())
                .unwrap();
            gui.add_label(Rect::new(100, 28, 46, 34), "3D\nSAFE", Style::label())
                .unwrap();
        }
        _ => {
            gui.add_label(Rect::new(10, 10, 140, 12), "UNKNOWN SCREEN", Style::label())
                .unwrap();
        }
    }
}
