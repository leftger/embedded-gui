use embedded_gui::prelude::*;

#[test]
fn focus_moves_between_buttons() {
    let mut gui = GuiContext::<4, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let first = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    let second = gui
        .add_button(Rect::new(0, 12, 30, 10), "TWO", Style::button())
        .unwrap();

    assert_eq!(gui.focus(), Some(first));
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.focus(), Some(second));
    gui.handle_input(InputEvent::Up).unwrap();
    assert_eq!(gui.focus(), Some(first));
}

#[test]
fn widget_flags_control_focus_rendering_and_pointer_hits() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 80, 40));
    let first = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    let second = gui
        .add_button(Rect::new(0, 12, 30, 10), "TWO", Style::button())
        .unwrap();
    let label = gui
        .add_label(Rect::new(40, 0, 30, 10), "LBL", Style::label())
        .unwrap();

    assert_eq!(gui.focus(), Some(first));
    gui.set_disabled(first, true).unwrap();
    assert_eq!(gui.focus(), Some(second));
    assert!(gui.has_flag(first, WidgetFlags::DISABLED).unwrap());

    while gui.pop_event().is_some() {}
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert!(gui.pop_event().is_none());

    gui.set_clickable(label, true).unwrap();
    while gui.pop_event().is_some() {}
    gui.handle_input(InputEvent::Pointer {
        x: 42,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert_eq!(gui.focus(), Some(second));
    assert_eq!(gui.pop_event(), Some(UiEvent::Pressed(label)));
    assert_eq!(gui.pop_event(), Some(UiEvent::PointerPressed(label)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Clicked(label)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Activate(label)));
}

#[test]
fn focused_menu_changes_selection() {
    static ITEMS: [&str; 3] = ["PLAY", "OPTS", "QUIT"];
    let mut gui = GuiContext::<4, 8, 8>::new(Rect::new(0, 0, 96, 48));
    let menu = gui
        .add_menu(Rect::new(0, 0, 60, 30), &ITEMS, 0, Style::panel())
        .unwrap();

    assert_eq!(gui.focus(), Some(menu));
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.menu_selected(menu), Some(1));
    gui.handle_input(InputEvent::Up).unwrap();
    assert_eq!(gui.menu_selected(menu), Some(0));
}

#[test]
fn test_haptics_sequencer_and_widget_triggers() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 100, 50));
    assert_eq!(gui.haptic_intensity(), 0);

    gui.play_haptic(HapticPattern::DoubleClick);
    gui.tick_input(10).unwrap();
    assert!(gui.haptic_intensity() > 0);

    gui.tick_input(30).unwrap();
    assert_eq!(gui.haptic_intensity(), 0);

    let _btn = gui
        .add_themed_button(Rect::new(0, 0, 20, 10), "Btn")
        .unwrap();

    gui.handle_input(InputEvent::Pointer {
        x: 5,
        y: 5,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 5,
        y: 5,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();

    assert!(gui.haptic_intensity() > 0);
}
