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
const H: u32 = 130;

static CARDS: [&str; 3] = ["ACTIVITY SUMMARY", "HEART RATE DETAIL", "SLEEP TREND"];

static PEEK_ICON_PIXELS: [u16; 64] = [0xFFFF; 64];
static REEL_PIXELS: [u16; 256] = [0x07E0; 256];
static REEL_FRAMES: [ReelFrame; 4] = [
    ReelFrame {
        sprite_index: 0,
        duration_ms: 80,
    },
    ReelFrame {
        sprite_index: 1,
        duration_ms: 80,
    },
    ReelFrame {
        sprite_index: 2,
        duration_ms: 80,
    },
    ReelFrame {
        sprite_index: 3,
        duration_ms: 80,
    },
];

#[derive(Clone, Copy)]
struct Ids {
    peek: WidgetId,
    glance_1: WidgetId,
    glance_2: WidgetId,
    glance_3: WidgetId,
    carddeck: WidgetId,
    reel: WidgetId,
}

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("cinematic peek/glance/carddeck showcase", &settings);

    let mut gui = GuiContext::<32, 48, 24>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);
    let mut animator = WidgetAnimator::<64, 64>::new();
    let mut deck_state = CardDeckState::new(CARDS.len());
    let mut elapsed_ms: u32 = 0;
    let mut auto_step_ms: u32 = 0;
    let mut auto_glance_idx: usize = 0;

    let icon = ImageRef::new(8, 8, &PEEK_ICON_PIXELS);
    let _ = setup_peek_timeline(&mut animator, ids.peek, None, None, 10, 14);
    gui.set_progress(ids.peek, 0.0).unwrap();
    let _ = gui.add_image(Rect::new(10, 14, 24, 24), icon, ImageFit::Stretch, Style::panel());

    'running: loop {
        elapsed_ms = elapsed_ms.wrapping_add(16);
        auto_step_ms = auto_step_ms.wrapping_add(16);
        animator.tick(16, &mut gui).unwrap();
        gui.tick_reel(ids.reel, 16).unwrap();
        gui.set_progress(ids.peek, ((elapsed_ms % 1200) as f32) / 1200.0)
            .unwrap();
        if auto_step_ms >= 1100 {
            auto_step_ms = 0;
            auto_glance_idx = (auto_glance_idx + 1) % 3;
            match auto_glance_idx {
                0 => {
                    gui.set_glance_highlighted(ids.glance_1, true).unwrap();
                    gui.set_glance_highlighted(ids.glance_2, false).unwrap();
                    gui.set_glance_highlighted(ids.glance_3, false).unwrap();
                    let _ = setup_launcher_glance(
                        &mut animator,
                        ids.glance_1,
                        &[ids.glance_2, ids.glance_3],
                        8,
                        42,
                    );
                }
                1 => {
                    gui.set_glance_highlighted(ids.glance_1, false).unwrap();
                    gui.set_glance_highlighted(ids.glance_2, true).unwrap();
                    gui.set_glance_highlighted(ids.glance_3, false).unwrap();
                    let _ = setup_launcher_glance(
                        &mut animator,
                        ids.glance_2,
                        &[ids.glance_1, ids.glance_3],
                        8,
                        62,
                    );
                }
                _ => {
                    gui.set_glance_highlighted(ids.glance_1, false).unwrap();
                    gui.set_glance_highlighted(ids.glance_2, false).unwrap();
                    gui.set_glance_highlighted(ids.glance_3, true).unwrap();
                    let _ = setup_launcher_glance(
                        &mut animator,
                        ids.glance_3,
                        &[ids.glance_1, ids.glance_2],
                        8,
                        82,
                    );
                }
            }
            if deck_state.move_next().is_none() {
                let _ = deck_state.move_prev();
                let _ = deck_state.move_prev();
            }
            gui.set_card_deck_selected(ids.carddeck, deck_state.current())
                .unwrap();
            setup_card_story(&mut gui, &[ids.carddeck], &deck_state).unwrap();
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
                        let _ = setup_peek_timeline(&mut animator, ids.peek, None, None, 10, 14);
                    }
                    Keycode::Down => {
                        if deck_state.move_next().is_some() {
                            gui.set_card_deck_selected(ids.carddeck, deck_state.current())
                                .unwrap();
                            setup_card_story(&mut gui, &[ids.carddeck], &deck_state).unwrap();
                        }
                    }
                    Keycode::Up => {
                        if deck_state.move_prev().is_some() {
                            gui.set_card_deck_selected(ids.carddeck, deck_state.current())
                                .unwrap();
                            setup_card_story(&mut gui, &[ids.carddeck], &deck_state).unwrap();
                        }
                    }
                    Keycode::Num1 => {
                        gui.set_glance_highlighted(ids.glance_1, true).unwrap();
                        gui.set_glance_highlighted(ids.glance_2, false).unwrap();
                        gui.set_glance_highlighted(ids.glance_3, false).unwrap();
                        let _ = setup_launcher_glance(
                            &mut animator,
                            ids.glance_1,
                            &[ids.glance_2, ids.glance_3],
                            8,
                            42,
                        );
                    }
                    Keycode::Num2 => {
                        gui.set_glance_highlighted(ids.glance_1, false).unwrap();
                        gui.set_glance_highlighted(ids.glance_2, true).unwrap();
                        gui.set_glance_highlighted(ids.glance_3, false).unwrap();
                        let _ = setup_launcher_glance(
                            &mut animator,
                            ids.glance_2,
                            &[ids.glance_1, ids.glance_3],
                            8,
                            62,
                        );
                    }
                    Keycode::Num3 => {
                        gui.set_glance_highlighted(ids.glance_1, false).unwrap();
                        gui.set_glance_highlighted(ids.glance_2, false).unwrap();
                        gui.set_glance_highlighted(ids.glance_3, true).unwrap();
                        let _ = setup_launcher_glance(
                            &mut animator,
                            ids.glance_3,
                            &[ids.glance_1, ids.glance_2],
                            8,
                            82,
                        );
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn build_ui(gui: &mut GuiContext<'static, 32, 48, 24>) -> Ids {
    gui.add_panel(Rect::new(4, 4, 212, 122), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(8, 8, 204, 10),
        "auto-cycling demo | 1/2/3 focus, up/down cards, space replay",
        Style::label(),
    )
    .unwrap();

    let peek = gui
        .add_peek_reveal(
            Rect::new(8, 20, 90, 18),
            ImageRef::new(8, 8, &PEEK_ICON_PIXELS),
            "NEXT",
            "RUN CLUB",
            Style::panel(),
        )
        .unwrap();

    let glance_1 = gui
        .add_glance_tile(Rect::new(8, 42, 90, 16), '*', "WORKOUT", "4.2 KM", Style::button())
        .unwrap();
    let glance_2 = gui
        .add_glance_tile(Rect::new(8, 62, 90, 16), '+', "WEATHER", "68F CLOUDY", Style::button())
        .unwrap();
    let glance_3 = gui
        .add_glance_tile(Rect::new(8, 82, 90, 16), '#', "MUSIC", "NOW PLAYING", Style::button())
        .unwrap();
    gui.set_glance_highlighted(glance_1, true).unwrap();

    let carddeck = gui
        .add_card_deck(Rect::new(104, 24, 108, 48), &CARDS, 0, Style::panel())
        .unwrap();

    let sheet = SpriteSheet::new(ImageRef::new(16, 16, &REEL_PIXELS), 8, 8);
    let player = ReelPlayer::new(sheet, &REEL_FRAMES, true);
    let reel = gui
        .add_reel(Rect::new(138, 78, 42, 42), player, ImageFit::Stretch, Style::panel())
        .unwrap();

    Ids {
        peek,
        glance_1,
        glance_2,
        glance_3,
        carddeck,
        reel,
    }
}
