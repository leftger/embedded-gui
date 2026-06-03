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

static CARD_A: [&str; 1] = ["ACTIVITY SUMMARY"];
static CARD_B: [&str; 1] = ["HEART RATE DETAIL"];
static CARD_C: [&str; 1] = ["SLEEP TREND"];

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
    card_1: WidgetId,
    card_2: WidgetId,
    card_3: WidgetId,
    reel: WidgetId,
}

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("cinematic peek/glance/carddeck showcase", &settings);

    let mut gui = GuiContext::<32, 48, 24>::new(Rect::new(0, 0, W, H));
    let ids = build_ui(&mut gui);
    let mut animator = WidgetAnimator::<64, 64>::new();
    let story_cards = [ids.card_1, ids.card_2, ids.card_3];
    let mut card_story =
        CardStory::new(&story_cards, TimelineMotionPreset::PeekIn).with_slide_px(18);
    let mut elapsed_ms: u32 = 0;
    let mut auto_step_ms: u32 = 0;
    let mut auto_glance_idx: usize = 0;
    let tokens = MotionTokens {
        peek_icon_duration_ms: 220,
        peek_text_stagger_ms: 70,
        peek_text_duration_ms: 140,
        glance_focus_bump_px: 5,
        glance_focus_slide_px: 10,
        glance_focus_duration_ms: 150,
        ..MotionTokens::default()
    };

    let icon = ImageRef::new(8, 8, &PEEK_ICON_PIXELS);
    let _ = setup_peek_timeline_with_tokens(&mut animator, ids.peek, None, None, 10, 14, tokens);
    gui.set_progress(ids.peek, 0.0).unwrap();
    card_story.apply(&mut gui).unwrap();
    let _ = gui.add_image(
        Rect::new(10, 14, 24, 24),
        icon,
        ImageFit::Stretch,
        Style::panel(),
    );

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
                    let _ = setup_launcher_glance_with_tokens(
                        &mut animator,
                        ids.glance_1,
                        &[ids.glance_2, ids.glance_3],
                        8,
                        42,
                        tokens,
                    );
                }
                1 => {
                    gui.set_glance_highlighted(ids.glance_1, false).unwrap();
                    gui.set_glance_highlighted(ids.glance_2, true).unwrap();
                    gui.set_glance_highlighted(ids.glance_3, false).unwrap();
                    let _ = setup_launcher_glance_with_tokens(
                        &mut animator,
                        ids.glance_2,
                        &[ids.glance_1, ids.glance_3],
                        8,
                        62,
                        tokens,
                    );
                }
                _ => {
                    gui.set_glance_highlighted(ids.glance_1, false).unwrap();
                    gui.set_glance_highlighted(ids.glance_2, false).unwrap();
                    gui.set_glance_highlighted(ids.glance_3, true).unwrap();
                    let _ = setup_launcher_glance_with_tokens(
                        &mut animator,
                        ids.glance_3,
                        &[ids.glance_1, ids.glance_2],
                        8,
                        82,
                        tokens,
                    );
                }
            }
            if let Some(transition) = card_story.next() {
                transition.animate(&mut animator, 104).unwrap();
                card_story.apply(&mut gui).unwrap();
            } else {
                let _ = card_story.jump_to(0);
                card_story.apply(&mut gui).unwrap();
            }
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
                        let _ = setup_peek_timeline_with_tokens(
                            &mut animator,
                            ids.peek,
                            None,
                            None,
                            10,
                            14,
                            tokens,
                        );
                    }
                    Keycode::Down => {
                        if let Some(transition) = card_story.next() {
                            transition.animate(&mut animator, 104).unwrap();
                            card_story.apply(&mut gui).unwrap();
                        }
                    }
                    Keycode::Up => {
                        if let Some(transition) = card_story.prev() {
                            transition.animate(&mut animator, 104).unwrap();
                            card_story.apply(&mut gui).unwrap();
                        }
                    }
                    Keycode::Num1 => {
                        gui.set_glance_highlighted(ids.glance_1, true).unwrap();
                        gui.set_glance_highlighted(ids.glance_2, false).unwrap();
                        gui.set_glance_highlighted(ids.glance_3, false).unwrap();
                        let _ = setup_launcher_glance_with_tokens(
                            &mut animator,
                            ids.glance_1,
                            &[ids.glance_2, ids.glance_3],
                            8,
                            42,
                            tokens,
                        );
                    }
                    Keycode::Num2 => {
                        gui.set_glance_highlighted(ids.glance_1, false).unwrap();
                        gui.set_glance_highlighted(ids.glance_2, true).unwrap();
                        gui.set_glance_highlighted(ids.glance_3, false).unwrap();
                        let _ = setup_launcher_glance_with_tokens(
                            &mut animator,
                            ids.glance_2,
                            &[ids.glance_1, ids.glance_3],
                            8,
                            62,
                            tokens,
                        );
                    }
                    Keycode::Num3 => {
                        gui.set_glance_highlighted(ids.glance_1, false).unwrap();
                        gui.set_glance_highlighted(ids.glance_2, false).unwrap();
                        gui.set_glance_highlighted(ids.glance_3, true).unwrap();
                        let _ = setup_launcher_glance_with_tokens(
                            &mut animator,
                            ids.glance_3,
                            &[ids.glance_1, ids.glance_2],
                            8,
                            82,
                            tokens,
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
        .add_glance_tile(
            Rect::new(8, 42, 90, 16),
            '*',
            "WORKOUT",
            "4.2 KM",
            Style::button(),
        )
        .unwrap();
    let glance_2 = gui
        .add_glance_tile(
            Rect::new(8, 62, 90, 16),
            '+',
            "WEATHER",
            "68F CLOUDY",
            Style::button(),
        )
        .unwrap();
    let glance_3 = gui
        .add_glance_tile(
            Rect::new(8, 82, 90, 16),
            '#',
            "MUSIC",
            "NOW PLAYING",
            Style::button(),
        )
        .unwrap();
    gui.set_glance_highlighted(glance_1, true).unwrap();

    let card_1 = gui
        .add_card_deck(Rect::new(104, 24, 108, 48), &CARD_A, 0, Style::panel())
        .unwrap();
    let card_2 = gui
        .add_card_deck(Rect::new(104, 24, 108, 48), &CARD_B, 0, Style::panel())
        .unwrap();
    let card_3 = gui
        .add_card_deck(Rect::new(104, 24, 108, 48), &CARD_C, 0, Style::panel())
        .unwrap();

    let sheet = SpriteSheet::new(ImageRef::new(16, 16, &REEL_PIXELS), 8, 8);
    let player = ReelPlayer::new(sheet, &REEL_FRAMES, true);
    let reel = gui
        .add_reel(
            Rect::new(138, 78, 42, 42),
            player,
            ImageFit::Stretch,
            Style::panel(),
        )
        .unwrap();

    Ids {
        peek,
        glance_1,
        glance_2,
        glance_3,
        card_1,
        card_2,
        card_3,
        reel,
    }
}
