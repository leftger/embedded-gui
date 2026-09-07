//! Broad rendering coverage for the built-in widget catalog.
//!
//! Exercises the public builder API for each widget kind so the core render
//! functions in `widgets/mod.rs` are measured by coverage.

use embedded_graphics_core::pixelcolor::{Rgb565, WebColors};
use embedded_gui::prelude::*;

mod common;
use common::MockTarget;

#[test]
fn renders_builtin_widget_catalog_without_panic() {
    static ITEMS: [&str; 3] = ["A", "B", "C"];
    static ROWS: [&[&str]; 2] = [&["1", "2"], &["3", "4"]];
    static KEYS: [char; 12] = ['1', '2', '3', '4', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
    static VALUES: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
    static CELLS: [embedded_gui::MenuCell; 3] = [
        embedded_gui::MenuCell::new("One").with_subtitle("first"),
        embedded_gui::MenuCell::new("Two"),
        embedded_gui::MenuCell::new("Three").with_enabled(false),
    ];

    let mut gui = GuiContext::<96, 64, 64>::new(Rect::new(0, 0, 320, 640));
    let mut y = 0u32;
    let mut next = |height: u32| {
        let r = Rect::new(0, y as i32, 320, height);
        y += height + 2;
        r
    };

    let r = next(16);
    gui.add_panel(r, Style::panel()).unwrap();
    let r = next(16);
    gui.add_label(r, "Label", Style::label()).unwrap();
    let r = next(20);
    gui.add_button(r, "Button", Style::button()).unwrap();
    let r = next(16);
    gui.add_progress_bar(r, 0.5, Style::progress()).unwrap();
    let r = next(20);
    gui.add_toggle(r, "Toggle", true, Style::button()).unwrap();
    let r = next(20);
    gui.add_checkbox(r, "Check", true, Style::button()).unwrap();
    let r = next(20);
    gui.add_slider(r, 0.5, 0.0, 1.0, Style::panel()).unwrap();
    let r = next(20);
    gui.add_value_label(r, "Value", 42, Style::panel()).unwrap();
    let r = next(20);
    gui.add_icon_button(r, 'i', "Icon", Style::button())
        .unwrap();
    let r = next(40);
    gui.add_list(r, &ITEMS, 0, 3, Style::panel()).unwrap();
    let r = next(40);
    gui.add_circular_list(r, &ITEMS, 0, 3, Style::panel())
        .unwrap();
    let r = next(40);
    gui.add_scroll_view(r, 0, 200, Style::panel()).unwrap();
    let r = next(20);
    gui.add_tabs(r, &ITEMS, 0, Style::panel()).unwrap();
    let r = next(40);
    gui.add_dialog(r, "Title", "Body", Style::panel()).unwrap();
    let r = next(20);
    gui.add_toast(r, "Toast", 1000, Style::panel()).unwrap();
    let r = next(20);
    gui.add_meter(r, 5.0, 0.0, 10.0, Style::panel()).unwrap();
    let r = next(60);
    gui.add_arc_gauge(r, 5.0, 0.0, 10.0, 180, 270, 4, false, Style::panel())
        .unwrap();
    let r = next(60);
    gui.add_gauge(r, 5.0, 0.0, 10.0, Style::panel()).unwrap();
    let r = next(40);
    gui.add_sweeping_arc(
        r,
        0.5,
        true,
        16,
        2,
        2,
        Rgb565::CSS_BLACK,
        Rgb565::CSS_CYAN,
        Rgb565::CSS_WHITE,
        Style::panel(),
    )
    .unwrap();
    let r = next(60);
    gui.add_gauge_needle(r, 5.0, 0.0, 10.0, 180, 270, Style::panel())
        .unwrap();
    let r = next(40);
    gui.add_chart(r, &VALUES, 0.0, 10.0, Style::panel())
        .unwrap();
    let r = next(40);
    gui.add_plotter(r, &VALUES, 0, 0.0, 10.0, Style::panel())
        .unwrap();
    let r = next(20);
    gui.add_spinner(r, 0.5, Style::panel()).unwrap();
    let r = next(24);
    gui.add_dropdown(r, &ITEMS, 0, Style::panel()).unwrap();
    let r = next(30);
    gui.add_roller(r, &ITEMS, 0, Style::panel()).unwrap();
    let r = next(30);
    gui.add_table(r, &ROWS, Style::panel()).unwrap();
    let r = next(60);
    gui.add_radial_scale(r, 0.0, 10.0, 5.0, Style::panel())
        .unwrap();
    let r = next(20);
    gui.add_linear_scale(r, 0.0, 10.0, 5.0, Style::panel())
        .unwrap();
    let r = next(24);
    gui.add_spinbox(r, 0, 10, 5, Style::panel()).unwrap();
    let r = next(40);
    gui.add_textarea(r, "Hello", "placeholder", Style::panel())
        .unwrap();
    let r = next(40);
    gui.add_keyboard(r, &KEYS, 6, None, Style::panel()).unwrap();
    let r = next(40);
    gui.add_menu(r, &ITEMS, 0, Style::panel()).unwrap();
    let r = next(40);
    gui.add_rich_menu(r, &CELLS, 0, 3, Style::panel()).unwrap();
    let r = next(60);
    gui.add_dial(r, 5.0, 0.0, 10.0, Style::panel()).unwrap();
    let r = next(20);
    gui.add_border(r, Style::panel()).unwrap();
    let r = next(10);
    gui.add_spacer(r).unwrap();

    let mut target = MockTarget::new(320, 640);
    gui.render(&mut target).unwrap();
    assert!(!target.pixels.is_empty());
}
