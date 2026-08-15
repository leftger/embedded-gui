use crate::{
    geometry::Rect,
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// Progress bar widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressBarWidget {
    pub value: f32,
}

impl ProgressBarWidget {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }
}

impl Widget for ProgressBarWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Value | PropertyKey::Progress => Some(PropertyValue::Float(self.value)),
            _ => None,
        }
    }

    fn set_property<'a>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Value | PropertyKey::Progress, PropertyValue::Float(v)) => {
                self.value = v.clamp(0.0, 1.0);
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}
