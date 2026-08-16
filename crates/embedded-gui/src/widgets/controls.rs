use crate::{
    geometry::Rect,
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// Interactive slider widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderWidget {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl SliderWidget {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value: value.clamp(min, max),
            min,
            max,
        }
    }
}

impl Widget for SliderWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Value => Some(PropertyValue::Float(self.value)),
            PropertyKey::Min => Some(PropertyValue::Float(self.min)),
            PropertyKey::Max => Some(PropertyValue::Float(self.max)),
            _ => None,
        }
    }

    fn set_property<'a>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Value, PropertyValue::Float(v)) => {
                self.value = v.clamp(self.min, self.max);
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}

/// Toggle switch widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleWidget<'a> {
    pub label: &'a str,
    pub on: bool,
}

impl<'a> ToggleWidget<'a> {
    pub fn new(label: &'a str, on: bool) -> Self {
        Self { label, on }
    }
}

impl<'a> Widget for ToggleWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::State => Some(PropertyValue::Bool(self.on)),
            PropertyKey::Text => Some(PropertyValue::Str(self.label)),
            _ => None,
        }
    }

    fn set_property<'b>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'b>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::State, PropertyValue::Bool(b)) => {
                self.on = b;
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}

/// Checkbox widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxWidget<'a> {
    pub label: &'a str,
    pub checked: bool,
}

impl<'a> Widget for CheckboxWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::State => Some(PropertyValue::Bool(self.checked)),
            PropertyKey::Text => Some(PropertyValue::Str(self.label)),
            _ => None,
        }
    }
}
