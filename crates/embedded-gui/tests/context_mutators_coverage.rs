//! Coverage tests for the context mutator/accessor API surface.
//!
//! The context mutators are the bridge between application code and widget
//! state. These tests call the common setter/getter families directly so their
//! `WidgetKind` dispatch arms are measured.

use embedded_graphics_core::pixelcolor::{Rgb565, WebColors};
use embedded_gui::MenuCell;
use embedded_gui::prelude::*;
use embedded_gui::widget::{PropertyKey, PropertyValue, WidgetFlags};
use embedded_gui::widgets::KeyboardLayout;

fn rect_at(y: &mut u32, height: u32) -> Rect {
    let r = Rect::new(0, *y as i32, 640, height);
    *y += height + 2;
    r
}

#[test]
fn mutators_for_common_widgets() {
    static ITEMS: [&str; 3] = ["A", "B", "C"];
    static VALUES: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
    static KEYS: [char; 8] = ['a', 'b', 'c', 'd', '1', '2', '3', '4'];
    static CELLS: [MenuCell; 3] = [
        MenuCell::new("One"),
        MenuCell::new("Two"),
        MenuCell::new("Three"),
    ];
    static SUGGESTIONS: [&str; 3] = ["alpha", "beta", "gamma"];

    let mut gui = GuiContext::<64, 32, 32>::new(Rect::new(0, 0, 640, 1200));
    let mut y = 0u32;

    let progress = gui
        .add_progress_bar(rect_at(&mut y, 16), 0.0, Style::progress())
        .unwrap();
    let toggle = gui
        .add_toggle(rect_at(&mut y, 20), "Toggle", false, Style::button())
        .unwrap();
    let checkbox = gui
        .add_checkbox(rect_at(&mut y, 20), "Check", false, Style::button())
        .unwrap();
    let slider = gui
        .add_slider(rect_at(&mut y, 20), 0.0, 0.0, 1.0, Style::panel())
        .unwrap();
    let value_label = gui
        .add_value_label(rect_at(&mut y, 20), "Value", 1, Style::panel())
        .unwrap();
    let list = gui
        .add_list(rect_at(&mut y, 40), &ITEMS, 0, 3, Style::panel())
        .unwrap();
    let scroll = gui
        .add_scroll_view(rect_at(&mut y, 40), 0, 200, Style::panel())
        .unwrap();
    let tabs = gui
        .add_tabs(rect_at(&mut y, 20), &ITEMS, 0, Style::panel())
        .unwrap();
    let menu = gui
        .add_menu(rect_at(&mut y, 40), &ITEMS, 0, Style::panel())
        .unwrap();
    let rich_menu = gui
        .add_rich_menu(rect_at(&mut y, 40), &CELLS, 0, 3, Style::panel())
        .unwrap();
    let meter = gui
        .add_meter(rect_at(&mut y, 20), 0.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let spinner = gui
        .add_spinner(rect_at(&mut y, 20), 0.0, Style::panel())
        .unwrap();
    let dropdown = gui
        .add_dropdown(rect_at(&mut y, 24), &ITEMS, 0, Style::panel())
        .unwrap();
    let roller = gui
        .add_roller(rect_at(&mut y, 30), &ITEMS, 0, Style::panel())
        .unwrap();
    let plotter = gui
        .add_plotter(rect_at(&mut y, 40), &VALUES, 0, 0.0, 10.0, Style::panel())
        .unwrap();
    let toast = gui
        .add_toast(rect_at(&mut y, 20), "Hello", 1000, Style::panel())
        .unwrap();
    let textarea = gui
        .add_textarea(rect_at(&mut y, 40), "hello", "placeholder", Style::panel())
        .unwrap();
    let keyboard = gui
        .add_keyboard(rect_at(&mut y, 40), &KEYS, 4, None, Style::panel())
        .unwrap();
    let gauge = gui
        .add_gauge(rect_at(&mut y, 60), 0.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let dial = gui
        .add_dial(rect_at(&mut y, 60), 0.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let auto = gui
        .add_autocomplete_widget(rect_at(&mut y, 40), &SUGGESTIONS, Style::panel())
        .unwrap();
    let panel = gui.add_panel(rect_at(&mut y, 20), Style::panel()).unwrap();
    let child = gui
        .add_label(rect_at(&mut y, 16), "child", Style::label())
        .unwrap();

    // Progress / toggle / checkbox / slider.
    gui.set_progress(progress, 0.75).unwrap();
    assert!(matches!(
        gui.get_widget_property(progress, PropertyKey::Progress),
        Some(PropertyValue::Float(v)) if (v - 0.75).abs() < 1e-5
    ));
    gui.set_toggle(toggle, true).unwrap();
    assert_eq!(gui.toggle_value(toggle), Some(true));
    gui.set_checked(checkbox, true).unwrap();
    assert_eq!(gui.checked_value(checkbox), Some(true));
    gui.set_slider_value(slider, 0.4).unwrap();
    assert_eq!(gui.slider_value(slider), Some(0.4));
    gui.set_value_label(value_label, 99).unwrap();

    // Selections / scroll.
    gui.set_list_selected(list, 2).unwrap();
    assert_eq!(gui.list_selected(list), Some(2));
    gui.set_scroll_offset(scroll, 12).unwrap();
    assert_eq!(gui.scroll_offset(scroll), Some(12));
    gui.set_tab_selected(tabs, 1).unwrap();
    assert_eq!(gui.tab_selected(tabs), Some(1));
    gui.set_menu_selected(menu, 1).unwrap();
    assert_eq!(gui.menu_selected(menu), Some(1));
    gui.set_rich_menu_selected(rich_menu, 1).unwrap();
    assert_eq!(gui.rich_menu_selected(rich_menu), Some(1));

    // Meter / spinner / dropdown / roller.
    gui.set_meter_value(meter, 5.0).unwrap();
    gui.set_spinner_phase(spinner, 0.5).unwrap();
    gui.tick_spinner(spinner, 100, 1.0).unwrap();
    gui.set_dropdown_selected(dropdown, 2).unwrap();
    assert_eq!(gui.dropdown_selected(dropdown), Some(2));
    gui.set_dropdown_open(dropdown, true).unwrap();
    assert_eq!(gui.dropdown_open(dropdown), Some(true));
    gui.set_roller_selected(roller, 1).unwrap();
    assert_eq!(gui.roller_selected(roller), Some(1));

    // Plotter / toast.
    gui.set_plotter_head(plotter, 2).unwrap();
    gui.set_plotter_values(plotter, &VALUES).unwrap();
    gui.set_toast_ttl(toast, 500).unwrap();
    gui.tick_toast(toast, 100).unwrap();

    // Textarea cursor/text editing families.
    gui.set_textarea_text(textarea, "hello world").unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("hello world"));
    gui.set_textarea_cursor(textarea, 5).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(5));
    gui.move_textarea_cursor(textarea, 1).unwrap();
    gui.move_textarea_cursor_select(textarea, -1).unwrap();
    gui.move_textarea_cursor_word(textarea, 1).unwrap();
    gui.move_textarea_cursor_word_select(textarea, -1).unwrap();
    gui.set_textarea_cursor_home(textarea).unwrap();
    gui.set_textarea_cursor_end(textarea).unwrap();
    gui.set_textarea_cursor_line_home(textarea).unwrap();
    gui.set_textarea_cursor_line_home_select(textarea).unwrap();
    gui.set_textarea_cursor_line_end(textarea).unwrap();
    gui.set_textarea_cursor_line_end_select(textarea).unwrap();
    gui.set_textarea_selection(textarea, 1, 3).unwrap();
    assert!(gui.textarea_selection(textarea).is_some());
    gui.clear_textarea_selection(textarea).unwrap();
    gui.set_textarea_capabilities(textarea, false, true, true)
        .unwrap();
    gui.textarea_insert_char(textarea, '!').unwrap();
    gui.textarea_backspace(textarea).unwrap();
    gui.textarea_delete_forward(textarea).unwrap();
    assert!(gui.textarea_cursor_visible(textarea).is_some());

    // Keyboard / gauge / dial / autocomplete.
    gui.set_keyboard_layout(keyboard, KeyboardLayout::Shift)
        .unwrap();
    assert_eq!(gui.keyboard_layout(keyboard), Some(KeyboardLayout::Shift));
    gui.set_keyboard_target(keyboard, Some(textarea)).unwrap();
    let _ = gui.keyboard_selected_key(keyboard);
    gui.set_gauge_value(gauge, 7.0).unwrap();
    gui.set_gauge_ticks(gauge, 5, 2, true).unwrap();
    gui.set_dial_value(dial, 6.0).unwrap();
    assert_eq!(gui.dial_value(dial), Some(6.0));
    gui.set_autocomplete_text(auto, "al").unwrap();
    assert_eq!(gui.autocomplete_text(auto), Some("al"));
    gui.insert_autocomplete_char(auto, 'p').unwrap();
    gui.delete_autocomplete_char(auto).unwrap();
    gui.autocomplete_confirm_selection(auto).unwrap();

    // Widget geometry/style/flags/parent.
    gui.set_widget_x(panel, 1).unwrap();
    gui.set_widget_y(panel, 2).unwrap();
    gui.set_widget_width(panel, 100).unwrap();
    gui.set_widget_height(panel, 50).unwrap();
    gui.set_widget_rect(panel, Rect::new(1, 2, 100, 50))
        .unwrap();
    assert_eq!(gui.absolute_rect(panel), Some(Rect::new(1, 2, 100, 50)));
    gui.set_widget_opacity(panel, 200).unwrap();
    gui.set_widget_corner_radius(panel, 4).unwrap();
    gui.set_widget_accent(panel, Rgb565::CSS_RED).unwrap();
    gui.set_widget_text_color(panel, Rgb565::CSS_WHITE).unwrap();
    gui.set_widget_foreground(panel, Rgb565::CSS_CYAN).unwrap();
    gui.set_widget_background(panel, Some(Rgb565::CSS_BLACK))
        .unwrap();
    gui.add_child(panel, child).unwrap();
    assert!(gui.children_of(panel).count() >= 1);
    gui.set_widget_parent(child, None).unwrap();

    gui.set_flag(panel, WidgetFlags::HIDDEN, true).unwrap();
    assert!(gui.has_flag(panel, WidgetFlags::HIDDEN).unwrap());
    gui.insert_flag(panel, WidgetFlags::CLICKABLE).unwrap();
    gui.remove_flag(panel, WidgetFlags::CLICKABLE).unwrap();
    gui.set_hidden(panel, false).unwrap();
    gui.set_disabled(panel, false).unwrap();
    gui.set_clickable(panel, true).unwrap();
    gui.set_scrollable(panel, false).unwrap();
    gui.set_visible(panel, true).unwrap();
    gui.set_enabled(panel, true).unwrap();
    gui.mark_subtree_dirty(panel).unwrap();
}
