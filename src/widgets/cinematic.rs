use crate::{
    geometry::Rect,
    style::Style,
    widget::{PropertyKey, PropertyValue, Widget},
};

/// Glance tile HUD component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlanceTileWidget<'a> {
    pub icon: char,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub highlighted: bool,
}

impl<'a> Widget for GlanceTileWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::State => Some(PropertyValue::Bool(self.highlighted)),
            PropertyKey::Text => Some(PropertyValue::Str(self.title)),
            PropertyKey::Custom(0) => Some(PropertyValue::Char(self.icon)),
            _ => None,
        }
    }
}
