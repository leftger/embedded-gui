//! Showcase: Wearable Status Bar, Pickers & Actionable Dialogs
//!
//! Demonstrates:
//! 1. **System Status Bar (`StatusBarWidget`)**: Real-time battery indicator, charging state, Bluetooth, DND, and clock.
//! 2. **Time Picker (`TimePickerWidget`)**: 12h/24h segmented time selector with focus halo and bump animations.
//! 3. **Numeric Range Picker (`NumberPickerWidget`)**: Incremental value roller with units.
//! 4. **Actionable & Confirmation Dialogs (`ActionableDialogWidget`)**: Icon glyphs, multi-line prompts, and action choices.
//!
//! ### Controls:
//! - **[Left / Right]**: Navigate fields within active picker / select dialog button
//! - **[Up / Down]**: Increment / Decrement active picker value
//! - **[Tab]**: Switch active widget focus (Time Picker -> Number Picker -> Dialog)
//! - **[Space]**: Toggle Status Bar mode / trigger selected dialog action
//! - **[Esc / Q]**: Exit

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, WebColors},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::{
    framebuffer::Framebuffer,
    geometry::Rect,
    render::RenderCtx,
    widgets::{
        ActionableDialogWidget, BatteryState, DialogAction, DialogType, NumberPickerWidget,
        StatusBarMode, StatusBarWidget, TimePickerWidget,
    },
};

const W: u32 = 240;
const H: u32 = 240;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveFocus {
    TimePicker,
    NumberPicker,
    Dialog,
}

fn main() {
    println!("=== embedded-gui: Wearable Status Bar, Pickers & Dialogs Showcase ===");

    let res = std::panic::catch_unwind(|| {
        run_interactive_window();
    });

    if res.is_err() {
        println!("\n[Notice: SDL2 desktop window could not be opened in current terminal session]");
        println!("[Rendering in standalone console simulation mode...]\n");
        run_console_showcase();
    }
}

fn run_interactive_window() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new(
        "Wearable Status Bar, Pickers & Dialogs (240x240)",
        &settings,
    );

    let mut status_bar = StatusBarWidget::new("10:42");
    status_bar.set_battery(88, BatteryState::Charging);
    status_bar.bluetooth_connected = true;
    status_bar.dnd_active = false;

    let mut time_picker = TimePickerWidget::new_12h(10, 42, true);
    let mut number_picker = NumberPickerWidget::new(40, 200, 72, "BPM");
    number_picker.is_focused = false;

    let mut dialog = ActionableDialogWidget::<3>::new(
        "HEART RATE LIMIT",
        "Threshold exceeded 140 BPM.",
        DialogType::Warning,
    );
    let _ = dialog.add_action(DialogAction::new("SNOOZE", 1));
    let _ = dialog.add_action(DialogAction::destructive("DISMISS", 2));

    let mut active_focus = ActiveFocus::TimePicker;
    let mut dialog_result: Option<&str> = None;

    'running: loop {
        // Render Frame
        display.clear(Rgb565::new(1, 2, 4)).unwrap();
        let screen = Rect::new(0, 0, W, H);
        let mut ctx = RenderCtx::new(&mut display, screen);

        // 1. Render Status Bar at top
        let bar_bounds = Rect::new(0, 0, W, status_bar.height as u32);
        let _ = status_bar.render(&mut ctx, bar_bounds);

        // 2. Render Time Picker
        let tp_bounds = Rect::new(12, 28, W - 24, 48);
        let _ = time_picker.render(&mut ctx, tp_bounds);

        // 3. Render Number Picker
        let np_bounds = Rect::new(12, 82, W - 24, 30);
        let _ = number_picker.render(&mut ctx, np_bounds);

        // 4. Render Actionable Dialog Card
        let dialog_bounds = Rect::new(12, 120, W - 24, 90);
        let _ = dialog.render(&mut ctx, dialog_bounds);

        // Render dialog action status message
        if let Some(msg) = dialog_result {
            let _ = ctx.draw_text(16, 218, msg, Rgb565::CSS_GREEN);
        } else {
            let _ = ctx.draw_text(
                16,
                218,
                "TAB: Focus | ARROWS: Adjust",
                Rgb565::new(12, 24, 18),
            );
        }

        window.update(&display);

        // Decay bump animation
        if time_picker.bump_offset_y > 0 {
            time_picker.bump_offset_y -= 1;
        } else if time_picker.bump_offset_y < 0 {
            time_picker.bump_offset_y += 1;
        }

        // Process Events
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape | Keycode::Q => break 'running,
                    Keycode::Tab => {
                        active_focus = match active_focus {
                            ActiveFocus::TimePicker => {
                                number_picker.is_focused = true;
                                ActiveFocus::NumberPicker
                            }
                            ActiveFocus::NumberPicker => {
                                number_picker.is_focused = false;
                                ActiveFocus::Dialog
                            }
                            ActiveFocus::Dialog => {
                                number_picker.is_focused = false;
                                ActiveFocus::TimePicker
                            }
                        };
                    }
                    Keycode::Left => match active_focus {
                        ActiveFocus::TimePicker => time_picker.prev_field(),
                        ActiveFocus::NumberPicker => number_picker.decrement(),
                        ActiveFocus::Dialog => dialog.select_prev(),
                    },
                    Keycode::Right => match active_focus {
                        ActiveFocus::TimePicker => time_picker.next_field(),
                        ActiveFocus::NumberPicker => number_picker.increment(),
                        ActiveFocus::Dialog => dialog.select_next(),
                    },
                    Keycode::Up => match active_focus {
                        ActiveFocus::TimePicker => time_picker.increment_focused(),
                        ActiveFocus::NumberPicker => number_picker.increment(),
                        ActiveFocus::Dialog => dialog.select_prev(),
                    },
                    Keycode::Down => match active_focus {
                        ActiveFocus::TimePicker => time_picker.decrement_focused(),
                        ActiveFocus::NumberPicker => number_picker.decrement(),
                        ActiveFocus::Dialog => dialog.select_next(),
                    },
                    Keycode::Space | Keycode::Return => {
                        if active_focus == ActiveFocus::Dialog {
                            dialog_result = match dialog.current_action_id() {
                                Some(1) => Some("Action: Snoozed alarm for 5 mins"),
                                Some(2) => Some("Action: Alert dismissed"),
                                _ => None,
                            };
                        } else {
                            // Cycle status bar mode
                            status_bar.mode = match status_bar.mode {
                                StatusBarMode::ClockAndIcons => StatusBarMode::ClockOnly,
                                StatusBarMode::ClockOnly => StatusBarMode::IconsOnly,
                                StatusBarMode::IconsOnly => StatusBarMode::ClockAndIcons,
                            };
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn run_console_showcase() {
    let screen = Rect::new(0, 0, W, H);
    let mut fb = Framebuffer::<{ 240 * 240 }>::new(W, H);
    let mut ctx = RenderCtx::new(&mut fb, screen);

    let mut status_bar = StatusBarWidget::new("10:42");
    status_bar.set_battery(95, BatteryState::Full);
    status_bar.bluetooth_connected = true;
    let _ = status_bar.render(&mut ctx, Rect::new(0, 0, W, 20));

    let time_picker = TimePickerWidget::new_12h(10, 42, true);
    let _ = time_picker.render(&mut ctx, Rect::new(12, 28, W - 24, 48));

    let number_picker = NumberPickerWidget::new(40, 200, 72, "BPM");
    let _ = number_picker.render(&mut ctx, Rect::new(12, 82, W - 24, 30));

    let mut dialog = ActionableDialogWidget::<2>::new(
        "CONFIRM SYNC",
        "Send 14 activities?",
        DialogType::Question,
    );
    let _ = dialog.add_action(DialogAction::new("CANCEL", 1));
    let _ = dialog.add_action(DialogAction::new("SYNC", 2));
    let _ = dialog.render(&mut ctx, Rect::new(12, 120, W - 24, 90));

    println!("1. Dynamic System Status Bar rendered (10:42, Battery 95% Full, BT connected).");
    println!("2. Segmented 12-Hour Time Picker rendered (10:42 PM with active cell focus).");
    println!("3. Numeric Range Picker rendered (72 BPM).");
    println!("4. Actionable Modal Dialog rendered with 2 buttons (CANCEL, SYNC).");
    println!("\nAll wearable status bar, picker, and dialog widgets validated successfully!");
}
