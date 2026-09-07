//! Render advanced widget kinds that need images, sheets, timelines, or
//! notification-style data, plus mutate a few states before rendering.

use embedded_gui::prelude::*;

mod common;
use common::MockTarget;

#[test]
fn renders_advanced_widgets() {
    static IMAGE_PX: [u16; 16] = [0xFFFF; 16];
    static ITEMS: [&str; 3] = ["Feed 1", "Feed 2", "Feed 3"];
    static TITLES: [&str; 3] = ["Card 1", "Card 2", "Card 3"];
    static ACTIONS: [&str; 2] = ["OK", "Cancel"];
    static SUGGESTIONS: [&str; 2] = ["alpha", "beta"];
    static REEL_FRAMES: [ReelFrame; 2] = [
        ReelFrame {
            sprite_index: 0,
            duration_ms: 10,
        },
        ReelFrame {
            sprite_index: 1,
            duration_ms: 10,
        },
    ];
    static RLE_DATA: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

    let image = ImageRef::new(4, 4, &IMAGE_PX);
    let sheet = SpriteSheet::new(image, 2, 2);
    let reel = ReelPlayer::new(sheet, &REEL_FRAMES, true);

    let mut gui = GuiContext::<64, 32, 32>::new(Rect::new(0, 0, 320, 1000));
    let mut y = 0u32;
    let mut rect = |height: u32| {
        let r = Rect::new(0, y as i32, 320, height);
        y += height + 2;
        r
    };

    let image_id = gui
        .add_image(rect(30), image, ImageFit::Center, Style::panel())
        .unwrap();
    let _ = gui
        .add_image(rect(30), image, ImageFit::Stretch, Style::panel())
        .unwrap();

    let peek = gui
        .add_peek_reveal(rect(40), image, "Peek", "Subtitle", Style::panel())
        .unwrap();
    let glance = gui
        .add_glance_tile(rect(40), 'g', "Glance", "Sub", Style::panel())
        .unwrap();
    let card_deck = gui
        .add_card_deck(rect(50), &TITLES, 0, Style::panel())
        .unwrap();
    let reel_id = gui
        .add_reel(rect(50), reel, ImageFit::Stretch, Style::panel())
        .unwrap();
    let state = gui
        .add_state_surface(
            rect(50),
            SurfaceState::Loading,
            "Title",
            "Message",
            None,
            Style::panel(),
        )
        .unwrap();
    let heads_up = gui
        .add_heads_up_banner(
            rect(30),
            NotificationLevel::Warning,
            "Heads up",
            500,
            Style::panel(),
        )
        .unwrap();
    let sheet_widget = gui
        .add_notification_action_sheet(
            rect(60),
            NotificationLevel::Info,
            "Title",
            "Body",
            &ACTIONS,
            0,
            true,
            Style::panel(),
        )
        .unwrap();
    let feed = gui
        .add_feed_timeline(rect(80), &ITEMS, 0, 3, false, Style::panel())
        .unwrap();
    let _rle = gui
        .add_rle_player(rect(40), &RLE_DATA, 2, 2, 4, 10, Style::panel())
        .unwrap();
    let auto = gui
        .add_autocomplete_widget(rect(40), &SUGGESTIONS, Style::panel())
        .unwrap();

    gui.set_progress(peek, 0.6).unwrap();
    gui.set_glance_highlighted(glance, true).unwrap();
    gui.set_card_deck_selected(card_deck, 2).unwrap();
    gui.set_state_surface_state(state, SurfaceState::Error)
        .unwrap();
    gui.set_state_surface_message(state, "Failed").unwrap();
    gui.set_state_surface_action(state, Some("Retry")).unwrap();
    gui.set_state_surface_busy_phase(state, 0.25).unwrap();
    gui.tick_state_surface(state, 100, 1.0).unwrap();
    gui.tick_reel(reel_id, 25).unwrap();
    gui.set_heads_up_ttl(heads_up, 200).unwrap();
    gui.tick_heads_up(heads_up, 50).unwrap();
    gui.set_notification_sheet_open(sheet_widget, false)
        .unwrap();
    gui.set_notification_sheet_selected(sheet_widget, 1)
        .unwrap();
    gui.set_feed_selected(feed, 2).unwrap();
    gui.set_feed_expanded(feed, true).unwrap();
    gui.set_autocomplete_text(auto, "al").unwrap();
    gui.insert_autocomplete_char(auto, 'p').unwrap();
    gui.autocomplete_confirm_selection(auto).unwrap();

    // Exercise the RLE player tick on the next render.
    gui.handle_input(InputEvent::Down).unwrap();
    gui.tick_input(15).unwrap();

    let mut target = MockTarget::new(320, 1000);
    gui.render(&mut target).unwrap();
    assert!(!target.pixels.is_empty());
    let _ = image_id;
}
