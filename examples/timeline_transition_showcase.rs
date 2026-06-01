use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(160, 96));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("timeline + transition showcase", &settings);

    let mut gui = GuiContext::<12, 12, 12>::new(Rect::new(0, 0, 160, 96));
    gui.add_panel(Rect::new(8, 8, 144, 80), Style::panel()).unwrap();
    let meter = gui
        .add_meter(Rect::new(20, 28, 40, 20), 0.0, 0.0, 1.0, Style::progress())
        .unwrap();
    let value = gui
        .add_value_label(Rect::new(20, 56, 70, 12), "ALPHA", 0, Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(72, 30, 74, 24),
        "SPACE:\nTRANSITION",
        Style::label(),
    )
    .unwrap();

    let mut seq = AnimationSequence::<4>::new();
    seq.push_animation(Animation::new(0.0, 1.0, 700, Easing::InOutSine))
        .unwrap();
    seq.push_delay(120).unwrap();
    seq.push_animation(Animation::new(1.0, 0.0, 700, Easing::InOutSine))
        .unwrap();
    let mut player = SequencePlayer::<2, 4>::new(seq);
    let mut transitions = ScreenTransitionRunner::new();
    let mut stack = ScreenStack::<4>::with_root(ScreenId::new(1)).unwrap();
    let mut lifecycle = heapless::Vec::<ScreenLifecycleEvent, 8>::new();
    let mut effect_idx = 0usize;
    let effects = [
        ScreenTransitionSpec::slide_left(420),
        ScreenTransitionSpec::fade(420),
        ScreenTransitionSpec::circular_reveal(420),
    ];

    'running: loop {
        player.tick(16).ok();
        if let Some(v) = player.active_value() {
            gui.set_meter_value(meter, v).unwrap();
        }

        transitions.tick(16);
        let alpha = transitions.active().map_or(0, |t| t.opacity_u8() as i32);
        gui.set_value_label(value, alpha).unwrap();

        display.clear(Rgb565::BLACK).unwrap();
        if let Some(active) = transitions.active() {
            render_transition_pair(&mut display, &gui, &gui, active, 160, 96).unwrap();
        } else {
            gui.render(&mut display).unwrap();
        }
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Escape,
                    ..
                } => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Space,
                    ..
                } => {
                    lifecycle.clear();
                    let command = if stack.current() == Some(ScreenId::new(1)) {
                        ScreenCommand::Push(ScreenId::new(2))
                    } else {
                        ScreenCommand::Pop
                    };
                    transitions
                        .apply(
                            &mut stack,
                            command,
                            effects[effect_idx],
                            &mut lifecycle,
                        )
                        .unwrap();
                    effect_idx = (effect_idx + 1) % effects.len();
                }
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
