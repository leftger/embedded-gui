//! Render widgets after state changes to exercise widget render branches that
//! are not reached by default-constructed widget catalogs.

use embedded_gui::MenuCell;
use embedded_gui::prelude::*;

mod common;
use common::MockTarget;

#[test]
fn renders_common_widgets_after_state_mutations() {
    static ITEMS: [&str; 3] = ["A", "B", "C"];
    static CELLS: [MenuCell; 3] = [
        MenuCell::new("One"),
        MenuCell::new("Two"),
        MenuCell::new("Three"),
    ];
    static VALUES: [f32; 4] = [0.0, 0.25, 0.5, 1.0];

    let mut gui = GuiContext::<48, 32, 32>::new(Rect::new(0, 0, 320, 640));
    let mut y = 0u32;
    let mut rect = |height: u32| {
        let r = Rect::new(0, y as i32, 320, height);
        y += height + 2;
        r
    };

    let progress = gui
        .add_progress_bar(rect(16), 0.0, Style::progress())
        .unwrap();
    let toggle_on = gui
        .add_toggle(rect(20), "On", false, Style::button())
        .unwrap();
    let toggle_off = gui
        .add_toggle(rect(20), "Off", true, Style::button())
        .unwrap();
    let checkbox = gui
        .add_checkbox(rect(20), "Check", false, Style::button())
        .unwrap();
    let slider = gui
        .add_slider(rect(20), 0.0, 0.0, 1.0, Style::panel())
        .unwrap();
    let list = gui
        .add_list(rect(60), &ITEMS, 0, 3, Style::panel())
        .unwrap();
    let menu = gui.add_menu(rect(60), &ITEMS, 0, Style::panel()).unwrap();
    let rich = gui
        .add_rich_menu(rect(60), &CELLS, 0, 3, Style::panel())
        .unwrap();
    let tabs = gui.add_tabs(rect(20), &ITEMS, 0, Style::panel()).unwrap();
    let dropdown = gui
        .add_dropdown(rect(24), &ITEMS, 0, Style::panel())
        .unwrap();
    let roller = gui.add_roller(rect(40), &ITEMS, 0, Style::panel()).unwrap();
    let scroll = gui
        .add_scroll_view(rect(60), 0, 200, Style::panel())
        .unwrap();
    let meter = gui
        .add_meter(rect(20), 0.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let spinner = gui.add_spinner(rect(20), 0.0, Style::panel()).unwrap();
    let gauge = gui
        .add_gauge(rect(60), 0.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let dial = gui
        .add_dial(rect(60), 0.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let plotter = gui
        .add_plotter(rect(40), &VALUES, 0, 0.0, 1.0, Style::panel())
        .unwrap();
    let textarea = gui.add_textarea(rect(40), "", "", Style::panel()).unwrap();
    let toast = gui
        .add_toast(rect(20), "Toast", 100, Style::panel())
        .unwrap();

    gui.set_progress(progress, 0.5).unwrap();
    gui.set_toggle(toggle_on, true).unwrap();
    gui.set_toggle(toggle_off, false).unwrap();
    gui.set_checked(checkbox, true).unwrap();
    gui.set_slider_value(slider, 0.75).unwrap();
    gui.set_list_selected(list, 2).unwrap();
    gui.set_menu_selected(menu, 1).unwrap();
    gui.set_rich_menu_selected(rich, 2).unwrap();
    gui.set_tab_selected(tabs, 2).unwrap();
    gui.set_dropdown_selected(dropdown, 1).unwrap();
    gui.set_dropdown_open(dropdown, true).unwrap();
    gui.set_roller_selected(roller, 2).unwrap();
    gui.set_scroll_offset(scroll, 10).unwrap();
    gui.set_meter_value(meter, 6.0).unwrap();
    gui.set_spinner_phase(spinner, 0.5).unwrap();
    gui.set_gauge_value(gauge, 8.0).unwrap();
    gui.set_dial_value(dial, 7.0).unwrap();
    gui.set_plotter_head(plotter, 2).unwrap();
    gui.set_toast_ttl(toast, 50).unwrap();
    gui.tick_toast(toast, 10).unwrap();
    gui.set_focus(Some(textarea)).unwrap();
    gui.handle_input(InputEvent::Home).unwrap();
    gui.handle_input(InputEvent::End).unwrap();
    gui.textarea_insert_char(textarea, 'x').unwrap();

    let mut target = MockTarget::new(320, 640);
    gui.render(&mut target).unwrap();
    assert!(!target.pixels.is_empty());
}

#[test]
fn renders_disabled_hidden_and_focused_widgets() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let a = gui
        .add_button(Rect::new(0, 0, 30, 10), "A", Style::button())
        .unwrap();
    let b = gui
        .add_button(Rect::new(0, 14, 30, 10), "B", Style::button())
        .unwrap();

    gui.set_focus(Some(a)).unwrap();
    gui.set_disabled(b, true).unwrap();
    gui.set_hidden(b, false).unwrap();
    gui.set_widget_opacity(a, 128).unwrap();

    let mut target = MockTarget::new(64, 32);
    gui.render(&mut target).unwrap();
    assert!(!target.pixels.is_empty());
}
