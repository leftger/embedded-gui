//! End-to-end coverage for iOS/Flutter-style rubber-band scroll overscroll:
//! drag-time resistance past the content edge, and a spring-back to the
//! boundary on release — see `GuiContext::set_scroll_rubber_band`,
//! `ScrollState::scroll_by_rubber_band`, and
//! `SpringAnimator::rubber_band_snap_back`.

use embedded_gui::input::{InputEvent, PointerButton, PointerState};
use embedded_gui::prelude::*;

fn press_and_drag_up(gui: &mut GuiContext<8, 16, 8>, start_y: i32, steps: &[i32]) {
    gui.handle_input(InputEvent::Pointer {
        x: 10,
        y: start_y,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    for &y in steps {
        gui.handle_input(InputEvent::Pointer {
            x: 10,
            y,
            state: PointerState::Moved,
            button: PointerButton::Primary,
        })
        .unwrap();
    }
}

#[test]
fn rubber_band_disabled_keeps_the_existing_hard_clamp() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 50, 50));
    let view = gui
        .add_scroll_view(Rect::new(0, 0, 50, 50), 0, 200, Style::panel())
        .unwrap();

    // Drag downward from the top (offset already 0): every intermediate
    // step tries to push the offset negative.
    press_and_drag_up(&mut gui, 10, &[15, 20, 25, 30, 35]);

    // Default (rubber_band = false): hard-clamped at 0 throughout, exactly
    // like before this feature existed.
    assert_eq!(gui.scroll_offset(view), Some(0));

    gui.handle_input(InputEvent::Pointer {
        x: 10,
        y: 35,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.tick_input(16).unwrap();
    assert_eq!(gui.scroll_offset(view), Some(0));
}

#[test]
fn rubber_band_enabled_resists_overscroll_then_springs_back_on_release() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 50, 50));
    gui.set_scroll_rubber_band(true);
    let view = gui
        .add_scroll_view(Rect::new(0, 0, 50, 50), 0, 200, Style::panel())
        .unwrap();

    // Same drag as above, in five 5px steps (25px raw), simulating a real
    // stream of small pointer-move events rather than one big jump.
    press_and_drag_up(&mut gui, 10, &[15, 20, 25, 30, 35]);

    let overscrolled = gui.scroll_offset(view).unwrap();
    assert!(
        overscrolled < 0,
        "must be allowed to move past the top edge, got {overscrolled}"
    );
    assert!(
        overscrolled > -25,
        "quadratic resistance must move less than the raw 25px drag, got {overscrolled}"
    );

    gui.handle_input(InputEvent::Pointer {
        x: 10,
        y: 35,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();

    // Ticking forward must monotonically spring the offset back to 0 and
    // settle there, never overshooting past the boundary.
    let mut last = overscrolled;
    let mut settled = false;
    for _ in 0..300 {
        gui.tick_input(16).unwrap();
        while gui.pop_event().is_some() {}
        let current = gui.scroll_offset(view).unwrap();
        assert!(
            current >= last && current <= 0,
            "overshot or reversed while springing back: {current} after {last}"
        );
        last = current;
        if current == 0 {
            settled = true;
            break;
        }
    }
    assert!(settled, "spring-back never reached the boundary");

    // Stays settled afterward (no lingering oscillation or drift).
    gui.tick_input(16).unwrap();
    assert_eq!(gui.scroll_offset(view), Some(0));
}
