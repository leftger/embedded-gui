use embedded_gui::prelude::*;
use embedded_gui::{
    EventPhaseMask, FocusGroupId, NavDirection, UiEventFilter, WidgetDispatchPolicy,
    WidgetEventFilter,
};

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

#[test]
fn event_filters_and_dispatch_policies() {
    let mut gui = GuiContext::<4, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let btn = gui
        .add_button(Rect::new(0, 0, 30, 10), "B", Style::button())
        .unwrap();

    gui.set_event_filter(btn, UiEventFilter::ACTIVATE | UiEventFilter::VALUE)
        .unwrap();
    assert!(
        gui.event_filter(btn)
            .unwrap()
            .contains(UiEventFilter::ACTIVATE)
    );
    gui.clear_event_filter(btn).unwrap();
    assert!(gui.event_filter(btn).unwrap().contains(UiEventFilter::ALL));

    gui.set_dispatch_policy(
        btn,
        WidgetDispatchPolicy::stop(WidgetEventFilter::ALL, EventPhaseMask::ALL),
    )
    .unwrap();
    assert!(gui.dispatch_policy(btn).unwrap().is_some());
    gui.clear_dispatch_policy(btn).unwrap();
    assert!(gui.dispatch_policy(btn).unwrap().is_none());
}

#[test]
fn spatial_focus_groups_and_keyboard_navigation() {
    static ITEMS: [&str; 3] = ["A", "B", "C"];
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 128, 96));

    let left = gui
        .add_button(Rect::new(0, 0, 20, 10), "L", Style::button())
        .unwrap();
    let right = gui
        .add_button(Rect::new(40, 0, 20, 10), "R", Style::button())
        .unwrap();
    let menu = gui
        .add_menu(Rect::new(0, 30, 60, 30), &ITEMS, 0, Style::panel())
        .unwrap();

    gui.set_focus_group(left, FocusGroupId::new(1)).unwrap();
    gui.set_focus_group(right, FocusGroupId::new(1)).unwrap();
    gui.set_active_focus_group(Some(FocusGroupId::new(1)));

    assert!(gui.move_focus_direction(NavDirection::Right).unwrap());
    assert_eq!(gui.focus(), Some(right));
    assert!(gui.move_focus_direction(NavDirection::Left).unwrap());
    assert_eq!(gui.focus(), Some(left));
    gui.set_active_focus_group(None);

    gui.set_focus(Some(menu)).unwrap();
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.menu_selected(menu), Some(1));
    gui.handle_input(InputEvent::Up).unwrap();
    assert_eq!(gui.menu_selected(menu), Some(0));
    gui.handle_input(InputEvent::Select).unwrap();
    while gui.pop_event().is_some() {}
    gui.handle_input(InputEvent::Back).unwrap();
    gui.tick_input(10).unwrap();
    gui.pop_event();
}

#[test]
fn textarea_key_events_and_long_press_repeat() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 128, 64));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 80, 30), "hello world", "", Style::panel())
        .unwrap();

    gui.set_focus(Some(textarea)).unwrap();
    gui.handle_input(InputEvent::End).unwrap();
    gui.handle_input(InputEvent::Home).unwrap();
    gui.handle_input(InputEvent::WordRight).unwrap();
    gui.handle_input(InputEvent::WordLeft).unwrap();
    gui.handle_input(InputEvent::SelectEnd).unwrap();
    gui.handle_input(InputEvent::SelectHome).unwrap();
    gui.handle_input(InputEvent::SelectWordLeft).unwrap();
    gui.handle_input(InputEvent::SelectWordRight).unwrap();
    gui.handle_input(InputEvent::Encoder { delta: 2 }).unwrap();
    gui.handle_input(InputEvent::Left).unwrap();
    gui.handle_input(InputEvent::Right).unwrap();
    gui.handle_input(InputEvent::SelectLeft).unwrap();
    gui.handle_input(InputEvent::SelectRight).unwrap();
    gui.tick_input(200).unwrap();
    while gui.pop_event().is_some() {}
}

#[test]
fn key_activation_changes_focusable_widget_state() {
    static ITEMS: [&str; 3] = ["A", "B", "C"];
    static KEYS: [char; 4] = ['A', 'B', 'C', 'D'];
    let mut gui = GuiContext::<16, 128, 8>::new(Rect::new(0, 0, 128, 96));
    let toggle = gui
        .add_toggle(Rect::new(0, 0, 20, 10), "T", false, Style::button())
        .unwrap();
    let checkbox = gui
        .add_checkbox(Rect::new(0, 12, 20, 10), "C", false, Style::button())
        .unwrap();
    let slider = gui
        .add_slider(Rect::new(0, 24, 30, 10), 0.5, 0.0, 1.0, Style::panel())
        .unwrap();
    let tabs = gui
        .add_tabs(Rect::new(0, 36, 40, 10), &ITEMS, 0, Style::panel())
        .unwrap();
    let textarea = gui
        .add_textarea(Rect::new(0, 48, 40, 10), "abc", "", Style::panel())
        .unwrap();
    let dial = gui
        .add_dial(Rect::new(0, 60, 20, 20), 5.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let keyboard = gui
        .add_keyboard(
            Rect::new(0, 80, 30, 16),
            &KEYS,
            2,
            Some(textarea),
            Style::panel(),
        )
        .unwrap();

    gui.set_focus(Some(toggle)).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.toggle_value(toggle), Some(true));

    gui.set_focus(Some(checkbox)).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.checked_value(checkbox), Some(true));

    gui.set_focus(Some(slider)).unwrap();
    gui.handle_input(InputEvent::Right).unwrap();
    assert!(gui.slider_value(slider).unwrap() > 0.5);
    gui.handle_input(InputEvent::Left).unwrap();
    assert_eq!(gui.slider_value(slider), Some(0.5));

    gui.set_focus(Some(tabs)).unwrap();
    gui.handle_input(InputEvent::Right).unwrap();

    gui.set_focus(Some(textarea)).unwrap();
    gui.handle_input(InputEvent::End).unwrap();
    gui.handle_input(InputEvent::Left).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(2));

    gui.set_focus(Some(dial)).unwrap();
    gui.handle_input(InputEvent::Right).unwrap();
    assert!(gui.dial_value(dial).unwrap() > 5.0);

    gui.set_focus(Some(keyboard)).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    let mut saw_text = false;
    while let Some(event) = gui.pop_event() {
        if matches!(event, UiEvent::TextInput { .. }) {
            saw_text = true;
        }
    }
    assert!(saw_text);
}

#[test]
fn select_toggles_dropdown_and_roller() {
    static ITEMS: [&str; 3] = ["A", "B", "C"];
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 100, 100));
    let dropdown = gui
        .add_dropdown(Rect::new(0, 0, 40, 20), &ITEMS, 0, Style::panel())
        .unwrap();
    let roller = gui
        .add_roller(Rect::new(0, 30, 40, 20), &ITEMS, 0, Style::panel())
        .unwrap();

    gui.set_focus(Some(dropdown)).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.dropdown_open(dropdown), Some(true));
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.dropdown_selected(dropdown), Some(1));
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.dropdown_open(dropdown), Some(false));

    gui.set_focus(Some(roller)).unwrap();
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.roller_selected(roller), Some(1));
    gui.handle_input(InputEvent::Up).unwrap();
    assert_eq!(gui.roller_selected(roller), Some(0));
}
