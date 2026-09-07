//! Coverage for small widget wrappers, display palettes, and simple presenters.

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor, WebColors};
use embedded_gui::palette::{DisplayMode, DisplayPalette, InkRole, RoleColors};
use embedded_gui::widget::{PropertyKey, PropertyValue, Widget};
use embedded_gui::widgets::{
    ButtonWidget, CheckboxWidget, GlanceTileWidget, InverterWidget, LabelWidget, ListWidget,
    PanelWidget, SliderWidget, SpacerWidget, ToggleWidget,
};

#[test]
fn control_widget_property_get_set() {
    let mut slider = SliderWidget::new(0.5, 0.0, 1.0);
    assert_eq!(
        slider.get_property(PropertyKey::Value),
        Some(PropertyValue::Float(0.5))
    );
    assert_eq!(
        slider.get_property(PropertyKey::Min),
        Some(PropertyValue::Float(0.0))
    );
    assert_eq!(
        slider.get_property(PropertyKey::Max),
        Some(PropertyValue::Float(1.0))
    );
    assert!(
        slider
            .set_property(PropertyKey::Value, PropertyValue::Float(0.75))
            .is_ok()
    );
    assert!(
        slider
            .set_property(PropertyKey::Min, PropertyValue::Float(0.0))
            .is_err()
    );

    let mut toggle = ToggleWidget::new("T", false);
    assert_eq!(
        toggle.get_property(PropertyKey::State),
        Some(PropertyValue::Bool(false))
    );
    assert_eq!(
        toggle.get_property(PropertyKey::Text),
        Some(PropertyValue::Str("T"))
    );
    assert!(
        toggle
            .set_property(PropertyKey::State, PropertyValue::Bool(true))
            .is_ok()
    );
    assert!(
        toggle
            .set_property(PropertyKey::State, PropertyValue::Float(1.0))
            .is_err()
    );

    let mut checkbox = CheckboxWidget {
        label: "C",
        checked: false,
    };
    assert_eq!(
        checkbox.get_property(PropertyKey::State),
        Some(PropertyValue::Bool(false))
    );
    assert!(
        checkbox
            .set_property(PropertyKey::State, PropertyValue::Bool(true))
            .is_err()
    );
}

#[test]
fn data_widget_property_get_set() {
    static ITEMS: [&str; 3] = ["A", "B", "C"];
    let mut list = ListWidget {
        items: &ITEMS,
        selected: 0,
        offset: 0,
        visible_rows: 3,
    };
    assert_eq!(
        list.get_property(PropertyKey::Selected),
        Some(PropertyValue::Usize(0))
    );
    assert_eq!(
        list.get_property(PropertyKey::Offset),
        Some(PropertyValue::Usize(0))
    );
    assert!(
        list.set_property(PropertyKey::Selected, PropertyValue::Usize(2))
            .is_ok()
    );
    assert!(
        list.set_property(PropertyKey::Offset, PropertyValue::Usize(1))
            .is_ok()
    );
    assert_eq!(list.get_property(PropertyKey::Text), None);
    assert!(
        list.set_property(PropertyKey::Text, PropertyValue::Str("x"))
            .is_err()
    );
}

#[test]
fn display_palette_modes_and_roles() {
    let normal = RoleColors::new(
        Rgb565::CSS_BLACK,
        Rgb565::CSS_WHITE,
        Rgb565::CSS_CYAN,
        Rgb565::CSS_RED,
        Rgb565::CSS_GRAY,
    );
    let stealth = RoleColors::new(
        Rgb565::CSS_BLACK,
        Rgb565::new(0, 10, 0),
        Rgb565::new(0, 20, 0),
        Rgb565::new(0, 15, 0),
        Rgb565::new(0, 5, 0),
    );
    let mut palette = DisplayPalette::new(normal, stealth);
    assert_eq!(palette.mode, DisplayMode::Normal);
    assert!(palette.resolve(InkRole::Primary).r() > 0);
    palette.set_mode(DisplayMode::Stealth);
    assert!(palette.resolve(InkRole::Primary).r() == 0);
    assert_eq!(normal.resolve(InkRole::Accent), Rgb565::CSS_RED);
}

#[test]
fn inverter_widget_properties() {
    let mut inverter = InverterWidget::new();
    assert_eq!(
        inverter.get_property(PropertyKey::State),
        Some(PropertyValue::Bool(true))
    );
    assert!(
        inverter
            .set_property(PropertyKey::State, PropertyValue::Bool(false))
            .is_ok()
    );
    assert_eq!(
        inverter.get_property(PropertyKey::State),
        Some(PropertyValue::Bool(false))
    );
    assert!(
        inverter
            .set_property(PropertyKey::State, PropertyValue::Float(1.0))
            .is_err()
    );
}

#[test]
fn basic_and_cinematic_widget_properties() {
    let label = LabelWidget::new("Hi");
    assert_eq!(
        label.get_property(PropertyKey::Text),
        Some(PropertyValue::Str("Hi"))
    );

    let mut button = ButtonWidget::new("B");
    assert_eq!(
        button.get_property(PropertyKey::Text),
        Some(PropertyValue::Str("B"))
    );
    assert!(
        button
            .set_property(PropertyKey::Text, PropertyValue::Str("x"))
            .is_err()
    );

    let panel = PanelWidget;
    let spacer = SpacerWidget;
    assert_eq!(panel.get_property(PropertyKey::Text), None);
    assert_eq!(spacer.get_property(PropertyKey::Text), None);

    let glance = GlanceTileWidget {
        icon: 'g',
        title: "Title",
        subtitle: "Sub",
        highlighted: false,
    };
    assert_eq!(
        glance.get_property(PropertyKey::State),
        Some(PropertyValue::Bool(false))
    );
    assert_eq!(
        glance.get_property(PropertyKey::Text),
        Some(PropertyValue::Str("Title"))
    );
    assert_eq!(
        glance.get_property(PropertyKey::Custom(0)),
        Some(PropertyValue::Char('g'))
    );
    assert_eq!(glance.get_property(PropertyKey::Custom(1)), None);
}

#[cfg(feature = "embedded-3dgfx")]
#[test]
fn embedded_graphics_interop_rect_conversions() {
    use embedded_gui::geometry::Rect;
    use embedded_gui::interop::graphics::{rect_to_rectangle, rectangle_to_rect};

    let rect = Rect::new(1, 2, 30, 40);
    assert_eq!(rectangle_to_rect(rect_to_rectangle(rect)), rect);
}
