use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 160;
const H: u32 = 96;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("embedded-gui animation + dirty regions", &settings);

    display.clear(Rgb565::BLACK).unwrap();

    let mut gui = GuiContext::<12, 16, 12>::new(Rect::new(0, 0, W, H));
    gui.add_panel(Rect::new(4, 4, 152, 88), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(10, 10, 136, 10),
        "ANIMATION + DIRTY",
        Style::label(),
    )
    .unwrap();
    let progress = gui
        .add_progress_bar(Rect::new(10, 30, 120, 10), 0.0, Style::progress())
        .unwrap();
    let percent = gui
        .add_value_label(Rect::new(10, 48, 72, 12), "PCT", 0, Style::panel())
        .unwrap();
    let dirty_count = gui
        .add_value_label(Rect::new(10, 66, 72, 12), "DIRTY", 0, Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(88, 48, 58, 28),
        "ONLY\nDIRTY\nWIDGETS",
        Style::label(),
    )
    .unwrap();

    gui.render(&mut display).unwrap();
    gui.clear_dirty();

    let mut tween = Animation::new(0.0, 1.0, 900, Easing::InOutSine)
        .with_repeat_mode(RepeatMode::PingPong)
        .with_repeat_count(None);

    'running: loop {
        tween.tick(16);

        let value = tween.value();
        gui.set_progress(progress, value).unwrap();
        gui.set_value_label(percent, (value * 100.0) as i32)
            .unwrap();
        gui.set_value_label(dirty_count, gui.dirty_regions().len() as i32)
            .unwrap();
        gui.render_dirty(&mut display).unwrap();
        gui.clear_dirty();

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

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
