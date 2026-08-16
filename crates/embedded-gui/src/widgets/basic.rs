use crate::{
    geometry::Rect,
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// Basic Panel widget container.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelWidget;

impl Widget for PanelWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}
}

/// Text label widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelWidget<'a> {
    pub text: &'a str,
}

impl<'a> LabelWidget<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Widget for LabelWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Text => Some(PropertyValue::Str(self.text)),
            _ => None,
        }
    }

    fn set_property<'b>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'b>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Text, PropertyValue::Str(_s)) => Ok(()),
            _ => Err(PropertyError::NotFound),
        }
    }
}

/// Interactive button widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonWidget<'a> {
    pub text: &'a str,
}

impl<'a> ButtonWidget<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Widget for ButtonWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Text => Some(PropertyValue::Str(self.text)),
            _ => None,
        }
    }
}

/// Spacer widget for layout alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpacerWidget;

impl Widget for SpacerWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}
}
