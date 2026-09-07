//! Coverage tests for GuiContext-level configuration, style classes, haptics,
//! dirty tracking, and theme transitions in `context/core_impl.rs`.

use embedded_gui::prelude::*;
use embedded_gui::{
    HapticPattern, KeyBindingAction, MenuContract, PressTiming, RenderQuality, StyleClassId, Theme,
    VisualState, WidgetKeyBindings, WidgetKeyInputPolicy,
};

#[test]
fn core_context_configuration_and_lifecycle() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let btn = gui
        .add_button(Rect::new(0, 0, 30, 10), "B", Style::button())
        .unwrap();

    gui.set_viewport(Rect::new(0, 0, 100, 50)).unwrap();
    gui.set_long_press_threshold_ms(400);
    assert!(gui.long_press_threshold_ms() >= 400);
    gui.set_press_repeat_timing(500, 50);
    gui.set_double_select_window_ms(200);
    gui.set_double_pointer_window_ms(250);

    let contract = MenuContract {
        wrap_navigation: false,
        ..Default::default()
    };
    gui.set_menu_contract(contract);
    let _ = gui.menu_contract();

    gui.set_widget_press_timing(btn, PressTiming::new(300, 200, 40))
        .unwrap();
    assert!(gui.widget_press_timing(btn).unwrap().is_some());
    gui.clear_widget_press_timing(btn).unwrap();
    assert!(gui.widget_press_timing(btn).unwrap().is_none());

    gui.set_widget_key_input_policy(
        btn,
        WidgetKeyInputPolicy {
            raw_select: true,
            raw_back: false,
        },
    )
    .unwrap();
    assert!(gui.widget_key_input_policy(btn).unwrap().is_some());
    gui.clear_widget_key_input_policy(btn).unwrap();

    gui.set_widget_key_bindings(
        btn,
        WidgetKeyBindings {
            select: KeyBindingAction::Ignore,
            back: KeyBindingAction::Back,
        },
    )
    .unwrap();
    assert!(gui.widget_key_bindings(btn).unwrap().is_some());
    gui.clear_widget_key_bindings(btn).unwrap();

    gui.set_scroll_physics(0.2, 0.9, 0.5);
    gui.set_state_transition_duration_ms(50);
    let _ = gui.active_state_transitions();
    gui.set_state_transition_duration_ms(0);
    gui.set_textarea_cursor_blink_timing(500);

    assert!(!gui.widgets().is_empty());
    let _ = gui.present_regions().count();
    let _ = gui.bounding_present_region();
    gui.clear_dirty();

    gui.set_theme(Theme::default()).unwrap();
    gui.start_theme_transition(Theme::default(), 30).unwrap();
    gui.start_theme_transition(Theme::default(), 0).unwrap();

    gui.play_haptic(HapticPattern::DoubleClick);
    gui.tick_input(5).unwrap();
    let _ = gui.haptic_intensity();
    gui.stop_haptic();

    let class = StyleClassId::new(1);
    gui.set_style_class(class, Style::panel()).unwrap();
    gui.set_style_class_state(class, VisualState::Focused, Style::panel())
        .unwrap();
    gui.set_widget_style_class(btn, Some(class)).unwrap();
    gui.apply_widget_style_transition(btn, VisualState::Normal, VisualState::Focused, 0.5)
        .unwrap();
    gui.clear_style_class(class).unwrap();

    gui.set_render_quality(RenderQuality::High).unwrap();
    assert_eq!(gui.render_quality(), RenderQuality::High);
    gui.set_focus(None).unwrap();
    assert_eq!(gui.focus(), None);

    gui.clear_widgets().unwrap();
    assert_eq!(gui.widgets().len(), 0);
}
