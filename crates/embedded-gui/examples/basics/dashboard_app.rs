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

const W: u32 = 192;
const H: u32 = 120;
const BOOT: ScreenId = ScreenId::new(1);
const MAIN: ScreenId = ScreenId::new(2);
const SETTINGS: ScreenId = ScreenId::new(3);
const DIALOG: ScreenId = ScreenId::new(4);

static TABS: [&str; 3] = ["SYS", "GFX", "NET"];
static SETTINGS_ITEMS: [&str; 6] = ["DITHER", "AUDIO", "RADAR", "VIBRO", "DEBUG", "ABOUT"];

#[derive(Default)]
struct DashboardIds {
    boot_progress: Option<WidgetId>,
    load_meter: Option<WidgetId>,
    fps_value: Option<WidgetId>,
    present_value: Option<WidgetId>,
    toast: Option<WidgetId>,
}

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("embedded-gui dashboard app", &settings);
    let mut gui = GuiContext::<32, 32, 24>::new(Rect::new(0, 0, W, H));
    let mut lifecycle = Vec::<ScreenLifecycleEvent, 16>::new();
    let mut screens = ScreenStack::<6>::with_root_lifecycle(BOOT, &mut lifecycle).unwrap();
    let mut ids = DashboardIds::default();
    let mut boot = Tween::new(0.0, 1.0, 1800, Easing::Smoothstep);
    let mut load = Tween::new(0.15, 0.95, 2200, Easing::EaseOut);
    rebuild(&mut gui, screens.current().unwrap(), &mut ids);

    'running: loop {
        if screens.current() == Some(BOOT) {
            if boot.tick(16) {
                screens
                    .apply_lifecycle(ScreenCommand::Replace(MAIN), &mut lifecycle)
                    .unwrap();
                rebuild(&mut gui, MAIN, &mut ids);
            } else if let Some(progress) = ids.boot_progress {
                gui.set_progress(progress, boot.value()).unwrap();
            }
        } else if screens.current() == Some(MAIN) {
            if load.tick(16) {
                load.reset();
            }
            if let Some(meter) = ids.load_meter {
                gui.set_meter_value(meter, load.value()).unwrap();
            }
            if let Some(fps) = ids.fps_value {
                gui.set_value_label(fps, 60).unwrap();
            }
            if let Some(toast) = ids.toast {
                gui.tick_toast(toast, 16).unwrap();
            }
        }

        if let Some(present) = ids.present_value {
            gui.set_value_label(present, gui.present_regions().count() as i32)
                .unwrap();
        }

        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        gui.clear_dirty();
        lifecycle.clear();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Space => {
                        if screens.current() == Some(MAIN) {
                            screens
                                .apply_lifecycle(ScreenCommand::Push(SETTINGS), &mut lifecycle)
                                .unwrap();
                            rebuild(&mut gui, SETTINGS, &mut ids);
                        }
                    }
                    Keycode::D => {
                        screens
                            .apply_lifecycle(ScreenCommand::Push(DIALOG), &mut lifecycle)
                            .unwrap();
                        rebuild(&mut gui, DIALOG, &mut ids);
                    }
                    Keycode::Backspace => {
                        if screens.len() > 1 {
                            screens
                                .apply_lifecycle(ScreenCommand::Pop, &mut lifecycle)
                                .unwrap();
                            rebuild(&mut gui, screens.current().unwrap_or(MAIN), &mut ids);
                        }
                    }
                    Keycode::Up => gui.handle_input(InputEvent::Up).unwrap(),
                    Keycode::Down => gui.handle_input(InputEvent::Down).unwrap(),
                    Keycode::Left => gui.handle_input(InputEvent::Left).unwrap(),
                    Keycode::Right => gui.handle_input(InputEvent::Right).unwrap(),
                    Keycode::Return => gui.handle_input(InputEvent::Select).unwrap(),
                    _ => {}
                },
                _ => {}
            }
        }
        while gui.pop_event().is_some() {}

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn rebuild(gui: &mut GuiContext<'static, 32, 32, 24>, screen: ScreenId, ids: &mut DashboardIds) {
    *ids = DashboardIds::default();
    gui.clear_widgets().unwrap();
    gui.add_themed_panel(Rect::new(4, 4, 184, 112)).unwrap();

    match screen {
        BOOT => {
            gui.add_themed_label(Rect::new(16, 28, 160, 14), "BOOTING DASHBOARD")
                .unwrap();
            ids.boot_progress = Some(
                gui.add_themed_progress_bar(Rect::new(24, 58, 144, 10), 0.0)
                    .unwrap(),
            );
        }
        MAIN => {
            gui.add_themed_label(Rect::new(12, 10, 168, 10), "MAIN DASHBOARD")
                .unwrap();
            ids.load_meter = Some(
                gui.add_themed_meter(Rect::new(12, 28, 62, 42), 0.35, 0.0, 1.0)
                    .unwrap(),
            );
            ids.fps_value = Some(
                gui.add_themed_value_label(Rect::new(84, 28, 72, 12), "FPS", 60)
                    .unwrap(),
            );
            ids.present_value = Some(
                gui.add_themed_value_label(Rect::new(84, 44, 72, 12), "REG", 0)
                    .unwrap(),
            );
            gui.add_themed_button(Rect::new(84, 64, 72, 14), "SPACE SETTINGS")
                .unwrap();
            gui.add_themed_icon_button(Rect::new(12, 82, 70, 14), 'D', "DIALOG")
                .unwrap();
            ids.toast = Some(
                gui.add_themed_toast(Rect::new(88, 84, 88, 16), "LIVE VALUES", 4000)
                    .unwrap(),
            );
        }
        SETTINGS => {
            gui.add_themed_label(Rect::new(12, 10, 168, 10), "SETTINGS")
                .unwrap();
            gui.add_themed_tabs(Rect::new(12, 26, 104, 14), &TABS, 0)
                .unwrap();
            let scroll = gui
                .add_themed_scroll_view(Rect::new(12, 44, 104, 54), 0, 96)
                .unwrap();
            let list = gui
                .add_themed_list(Rect::new(4, 4, 92, 42), &SETTINGS_ITEMS, 0, 4)
                .unwrap();
            gui.add_child(scroll, list).unwrap();
            gui.add_themed_toggle(Rect::new(124, 28, 52, 14), "AA", true)
                .unwrap();
            gui.add_themed_slider(Rect::new(124, 50, 52, 14), 0.5, 0.0, 1.0)
                .unwrap();
            gui.add_themed_label(Rect::new(122, 82, 58, 18), "BACKSPACE\nRETURNS")
                .unwrap();
        }
        DIALOG => {
            gui.add_themed_dialog(
                Rect::new(32, 28, 128, 58),
                "CONFIRM",
                "THIS IS A MODAL DIALOG OVERLAY. BACKSPACE CLOSES IT.",
            )
            .unwrap();
        }
        _ => {}
    }
}
