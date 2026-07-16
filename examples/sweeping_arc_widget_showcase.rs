//! Host demo of the `SweepingArc` widget: a `WidgetAnimator` drives its
//! `progress` 0 → 1 on a loop, rendered into a 96×64 SDL window (the Nitro
//! Revolver panel size). Proves the crate-side widget in isolation — no
//! Markham font/overlay, just the arc sweep + rounded "window".
//!
//! Run: `cargo run --example sweeping_arc_widget_showcase`

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 96;
const H: u32 = 64;
const FRAME_MS: u32 = 33; // ~30 Hz, matching the device
const DURATION_MS: u32 = 5000;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(8).build();
    let mut window = Window::new("SweepingArc widget", &settings);

    let mut gui = GuiContext::<4, 4, 8>::new(Rect::new(0, 0, W, H));
    let arc = gui
        .add_sweeping_arc(
            Rect::new(0, 0, W, H),
            0.0,
            60,                     // arc_radius
            12,                     // frame_inset
            4,                      // corner_radius
            Rgb565::new(5, 10, 5),  // dark-grey background
            Rgb565::new(28, 0, 15), // pink/magenta sweep
            Rgb565::BLACK,          // frame "window"
            Style::panel(),
        )
        .unwrap();

    let mut animator = WidgetAnimator::<4, 4>::new();
    animator
        .animate_progress(arc, 0.0, 1.0, DURATION_MS, Easing::Linear)
        .unwrap();

    'running: loop {
        if animator.active_count() == 0 {
            animator
                .animate_progress(arc, 0.0, 1.0, DURATION_MS, Easing::Linear)
                .unwrap();
        }
        animator.tick(FRAME_MS, &mut gui).unwrap();

        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Escape,
                    ..
                } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(FRAME_MS as u64));
    }
}
