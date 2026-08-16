use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_gui::prelude::*;
use embedded_gui::widgets::ButtonWidget;
use embedded_gui::{PropertyKey, PropertyValue};

mod common;
use common::MockTarget;

#[test]
fn renders_label_and_progress_bar() {
    let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 64, 32));
    gui.add_label(Rect::new(2, 2, 40, 8), "OK", Style::label())
        .unwrap();
    gui.add_progress_bar(Rect::new(2, 14, 20, 6), 0.5, Style::progress())
        .unwrap();

    let mut target = MockTarget::new(64, 32);
    gui.render(&mut target).unwrap();

    assert!(!target.pixels.is_empty());
    assert!(
        target
            .pixels
            .iter()
            .any(|&(_, _, color)| color == Rgb565::new(0, 50, 18))
    );
}

#[test]
fn test_plotter_and_circular_list_widgets() {
    static VALUES: [f32; 4] = [10.0, 20.0, 15.0, 30.0];
    static ITEMS: [&str; 4] = ["A", "B", "C", "D"];

    let mut gui = GuiContext::<16, 16, 16>::new(Rect::new(0, 0, 128, 64));

    let plotter_id = gui
        .add_plotter(
            Rect::new(0, 0, 60, 30),
            &VALUES,
            0,
            0.0,
            40.0,
            Style::panel(),
        )
        .unwrap();

    let circ_id = gui
        .add_circular_list(Rect::new(64, 0, 60, 30), &ITEMS, 0, 3, Style::panel())
        .unwrap();

    let mut target = MockTarget::new(128, 64);
    gui.render(&mut target).unwrap();

    assert_eq!(gui.widgets().len(), 2);
    assert_eq!(gui.focus(), Some(circ_id));
    assert!(gui.set_enabled(plotter_id, true).is_ok());
}

#[test]
fn test_generic_property_binding_engine() {
    let mut gui = GuiContext::<16, 16, 16>::new(Rect::new(0, 0, 128, 64));
    let progress_id = gui
        .add_themed_progress_bar(Rect::new(0, 0, 50, 10), 0.2)
        .unwrap();

    assert_eq!(
        gui.get_widget_property(progress_id, PropertyKey::Value),
        Some(PropertyValue::Float(0.2))
    );

    gui.set_widget_property(progress_id, PropertyKey::Value, PropertyValue::Float(0.85))
        .unwrap();

    assert_eq!(
        gui.get_widget_property(progress_id, PropertyKey::Value),
        Some(PropertyValue::Float(0.85))
    );
}

#[test]
fn test_fluent_widget_builder_spawn() {
    let mut gui = GuiContext::<16, 16, 16>::new(Rect::new(0, 0, 128, 64));

    let btn_id = gui
        .spawn(Rect::new(10, 10, 50, 20), ButtonWidget::new("Fluent"))
        .with_style_class(StyleClassId::NONE)
        .build()
        .unwrap();

    assert_eq!(gui.widgets().len(), 1);
    assert_eq!(btn_id.raw(), 1);
}
