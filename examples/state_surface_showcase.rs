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

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("state surface showcase", &settings);
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, W, H));

    let surface = gui
        .add_state_surface(
            Rect::new(8, 8, 204, 104),
            SurfaceState::Loading,
            "NETWORK",
            "Connecting to companion service...",
            Some("Retry"),
            Style::panel(),
        )
        .unwrap();

    let mut elapsed_ms = 0u32;
    let mut state_idx = 0usize;
    let states = [
        SurfaceState::Loading,
        SurfaceState::Empty,
        SurfaceState::Error,
        SurfaceState::Offline,
        SurfaceState::Ready,
    ];
    let messages = [
        "Connecting to companion service...",
        "No data available yet.",
        "Sync failed. Try again.",
        "Watch is in offline mode.",
        "Everything looks good.",
    ];

    'running: loop {
        elapsed_ms = elapsed_ms.wrapping_add(16);
        gui.tick_state_surface(surface, 16, 0.8).unwrap();

        if elapsed_ms % 1500 == 0 {
            state_idx = (state_idx + 1) % states.len();
            gui.set_state_surface_state(surface, states[state_idx])
                .unwrap();
            gui.set_state_surface_message(surface, messages[state_idx])
                .unwrap();
            let action = if matches!(states[state_idx], SurfaceState::Ready | SurfaceState::Empty) {
                None
            } else {
                Some("Retry")
            };
            gui.set_state_surface_action(surface, action).unwrap();
        }

        display.clear(Rgb565::BLACK).unwrap();
        gui.render(&mut display).unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Space => {
                        state_idx = (state_idx + 1) % states.len();
                        gui.set_state_surface_state(surface, states[state_idx])
                            .unwrap();
                        gui.set_state_surface_message(surface, messages[state_idx])
                            .unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
