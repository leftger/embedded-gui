use crate::{
    geometry::Rect,
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// List view widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListWidget<'a> {
    pub items: &'a [&'a str],
    pub selected: usize,
    pub offset: usize,
    pub visible_rows: usize,
}

impl<'a> Widget for ListWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Selected => Some(PropertyValue::Usize(self.selected)),
            PropertyKey::Offset => Some(PropertyValue::Usize(self.offset)),
            _ => None,
        }
    }

    fn set_property<'b>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'b>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Selected, PropertyValue::Usize(s)) => {
                self.selected = s;
                Ok(())
            }
            (PropertyKey::Offset, PropertyValue::Usize(o)) => {
                self.offset = o;
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}
