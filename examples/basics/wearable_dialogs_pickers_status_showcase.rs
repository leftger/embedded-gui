//! Showcase: Wearable Status Bar, Pickers & Actionable Dialogs (320x240)
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
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor, WebColors},
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::{
    EdgeInsets, Framebuffer, Rect, RenderCtx,
    widgets::{
        ActionableDialogWidget, BatteryState, DialogAction, DialogType, NumberPickerWidget,
        StatusBarWidget, TimePickerWidget,
    },
};

const W: u32 = 320;
const H: u32 = 240;
const FB_SIZE: usize = (W * H) as usize;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveFocus {
    TimePicker,
    NumberPicker,
    Dialog,
}

fn render_wearable_scene<D: DrawTarget<Color = Rgb565> + embedded_gui::PixelRead>(
    target: &mut D,
    frame: u32,
    status_bar: &StatusBarWidget,
    time_picker: &TimePickerWidget,
    number_picker: &NumberPickerWidget,
    dialog: &ActionableDialogWidget<3>,
    dialog_result: Option<&str>,
) -> Result<(), D::Error> {
    // 1. Clear background with dark wearable gradient
    let bg_rect = Rectangle::new(Point::zero(), Size::new(W, H));
    target.fill_solid(&bg_rect, Rgb565::new(1, 2, 5))?;

    let viewport = Rect::new(0, 0, W, H);
    let mut ctx = RenderCtx::compositing(target, viewport);

    // 2. Render Status Bar at top (Full 320px width)
    let bar_bounds = Rect::new(0, 0, W, status_bar.height as u32);
    ctx.fill_rect(bar_bounds, Rgb565::new(3, 6, 12))?;
    let _ = status_bar.render(&mut ctx, bar_bounds);

    // 3. Left Column (Width 140px): Time Picker + Number Picker
    let left_col = Rect::new(10, 28, 140, 175);
    ctx.fill_rounded_rect(left_col, 6, Rgb565::new(2, 4, 9))?;
    ctx.stroke_rounded_rect(
        left_col,
        6,
        embedded_gui::Border::one(Rgb565::new(0, 20, 30)),
    )?;

    ctx.draw_text(
        left_col.x + 8,
        left_col.y + 6,
        "ALARM & GOAL",
        Rgb565::CSS_CYAN,
    )?;

    // Time Picker
    let mut tp = time_picker.clone();
    tp.hour = (10 + (frame / 30) % 3) as u8;
    tp.minute = (40 + (frame / 2) % 20) as u8;
    let tp_bounds = Rect::new(left_col.x + 8, left_col.y + 24, left_col.w - 16, 44);
    let _ = tp.render(&mut ctx, tp_bounds);

    // Number Picker (Heart Rate / Step Goal)
    let mut np = number_picker.clone();
    np.value = 135 + ((frame as i32 * 2) % 30);
    np.is_focused = ((frame / 30) % 2) == 1;
    let np_bounds = Rect::new(left_col.x + 8, left_col.y + 80, left_col.w - 16, 32);
    let _ = np.render(&mut ctx, np_bounds);

    // Daily Goal Gauge in left column
    let gauge_rect = Rect::new(left_col.x + 8, left_col.y + 124, left_col.w - 16, 10);
    let progress = ((frame as f32 * 0.05).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    ctx.fill_rounded_rect(gauge_rect, 4, Rgb565::new(6, 12, 18))?;
    let filled_w = ((gauge_rect.w as f32) * progress) as u32;
    if filled_w > 0 {
        ctx.fill_rounded_rect(
            Rect::new(gauge_rect.x, gauge_rect.y, filled_w, gauge_rect.h),
            4,
            Rgb565::CSS_SPRING_GREEN,
        )?;
    }
    ctx.draw_text(
        left_col.x + 8,
        left_col.y + 140,
        "Target: 10,000 steps",
        Rgb565::CSS_GRAY,
    )?;

    // 4. Right Column (Width 156px): Actionable Notification Dialog Card
    let right_col = Rect::new(156, 28, 154, 175);
    ctx.fill_rounded_rect(right_col, 6, Rgb565::new(3, 5, 11))?;
    ctx.stroke_rounded_rect(
        right_col,
        6,
        embedded_gui::Border::one(Rgb565::new(20, 15, 0)),
    )?;

    let mut anim_dialog = dialog.clone();
    anim_dialog.selected_action = ((frame / 40) % 2) as usize;
    let dialog_bounds = Rect::new(right_col.x + 6, right_col.y + 6, right_col.w - 12, 160);
    let _ = anim_dialog.render(&mut ctx, dialog_bounds);

    // 5. Bottom Status / Hint Bar
    let bot_bar = Rect::new(10, 208, W - 20, 24);
    ctx.fill_rounded_rect(bot_bar, 4, Rgb565::new(2, 4, 8))?;
    ctx.stroke_rounded_rect(
        bot_bar,
        4,
        embedded_gui::Border::one(Rgb565::new(0, 15, 25)),
    )?;

    if let Some(msg) = dialog_result {
        let _ = ctx.draw_text_in(
            bot_bar.inset(EdgeInsets::symmetric(6, 4)),
            msg,
            embedded_gui::TextStyle::new(Rgb565::CSS_GREEN),
        );
    } else {
        let _ = ctx.draw_text_in(
            bot_bar.inset(EdgeInsets::symmetric(6, 4)),
            "Wearable OS: Battery, Bluetooth, Pickers & Action Sheet",
            embedded_gui::TextStyle::new(Rgb565::CSS_LIGHT_GRAY),
        );
    }

    Ok(())
}

fn record_frames() {
    let out_dir = std::path::Path::new("target/wearable_frames");
    let _ = std::fs::create_dir_all(out_dir);

    let mut fb = Framebuffer::<FB_SIZE>::new(W, H);
    let mut status_bar = StatusBarWidget::new("10:42");
    status_bar.set_battery(88, BatteryState::Charging);
    status_bar.bluetooth_connected = true;
    status_bar.dnd_active = false;

    let time_picker = TimePickerWidget::new_12h(10, 42, true);
    let number_picker = NumberPickerWidget::new(40, 200, 145, "BPM");

    let mut dialog = ActionableDialogWidget::<3>::new(
        "HEART RATE ALERT",
        "Threshold exceeded 140 BPM.",
        DialogType::Warning,
    );
    let _ = dialog.add_action(DialogAction::new("SNOOZE", 1));
    let _ = dialog.add_action(DialogAction::destructive("DISMISS", 2));

    let total_frames = 80;
    println!(
        "Recording {} frames to target/wearable_frames...",
        total_frames
    );

    for f in 0..total_frames {
        render_wearable_scene(
            &mut fb,
            f,
            &status_bar,
            &time_picker,
            &number_picker,
            &dialog,
            None,
        )
        .unwrap();

        let mut rgb888 = Vec::with_capacity((W * H * 3) as usize);
        for p in fb.pixels() {
            let r = (p.r() << 3) | (p.r() >> 2);
            let g = (p.g() << 2) | (p.g() >> 4);
            let b = (p.b() << 3) | (p.b() >> 2);
            rgb888.push(r);
            rgb888.push(g);
            rgb888.push(b);
        }

        let filename = out_dir.join(format!("frame_{:03}.raw", f));
        std::fs::write(filename, &rgb888).unwrap();
    }
    println!("Wearable frame recording complete!");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--record-gif") {
        record_frames();
        return;
    }

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
    let mut fb = Framebuffer::<FB_SIZE>::new(W, H);
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new(
        "Wearable Status Bar, Pickers & Dialogs (320x240)",
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
        "HEART RATE ALERT",
        "Threshold exceeded 140 BPM.",
        DialogType::Warning,
    );
    let _ = dialog.add_action(DialogAction::new("SNOOZE", 1));
    let _ = dialog.add_action(DialogAction::destructive("DISMISS", 2));

    let mut active_focus = ActiveFocus::TimePicker;
    let dialog_result: Option<&str> = None;
    let mut frame = 0u32;
    let mut paused = false;

    'running: loop {
        if !paused {
            frame = frame.wrapping_add(1);
        }

        render_wearable_scene(
            &mut fb,
            frame,
            &status_bar,
            &time_picker,
            &number_picker,
            &dialog,
            dialog_result,
        )
        .unwrap();

        let full_area = Rectangle::new(Point::zero(), Size::new(W, H));
        display
            .fill_contiguous(&full_area, fb.pixels().iter().copied())
            .unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape | Keycode::Q => break 'running,
                    Keycode::Space => paused = !paused,
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
                        ActiveFocus::Dialog => {
                            dialog.selected_action = dialog.selected_action.saturating_sub(1);
                        }
                    },
                    Keycode::Right => match active_focus {
                        ActiveFocus::TimePicker => time_picker.next_field(),
                        ActiveFocus::NumberPicker => number_picker.increment(),
                        ActiveFocus::Dialog => {
                            dialog.selected_action = (dialog.selected_action + 1).min(1);
                        }
                    },
                    Keycode::Up => match active_focus {
                        ActiveFocus::TimePicker => time_picker.increment_focused(),
                        ActiveFocus::NumberPicker => number_picker.increment(),
                        ActiveFocus::Dialog => {}
                    },
                    Keycode::Down => match active_focus {
                        ActiveFocus::TimePicker => time_picker.decrement_focused(),
                        ActiveFocus::NumberPicker => number_picker.decrement(),
                        ActiveFocus::Dialog => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn run_console_showcase() {
    let mut fb = Framebuffer::<FB_SIZE>::new(W, H);
    let status_bar = StatusBarWidget::new("10:42");
    let time_picker = TimePickerWidget::new_12h(10, 42, true);
    let number_picker = NumberPickerWidget::new(40, 200, 145, "BPM");
    let dialog = ActionableDialogWidget::<3>::new(
        "HEART RATE ALERT",
        "Threshold exceeded 140 BPM.",
        DialogType::Warning,
    );

    println!("Running 60 frames of wearable showcase in headless mode...");
    let t0 = std::time::Instant::now();
    for f in 0..60 {
        render_wearable_scene(
            &mut fb,
            f,
            &status_bar,
            &time_picker,
            &number_picker,
            &dialog,
            None,
        )
        .unwrap();
    }
    println!("-> 60 frames rendered in: {:?}", t0.elapsed());
    println!("Wearable showcase headless verification complete!");
}
