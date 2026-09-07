//! Inverter layer widget for zero-allocation high-contrast focus/selection highlights.
//!
//! Inspired by PebbleOS `inverter_layer.c`.

use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

use crate::{
    geometry::Rect,
    render::{Compositor, PixelRead, RenderCtx},
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// PebbleOS-style Inverter widget that inverts the colors of all pixels within its bounds.
///
/// Provides zero-allocation, high-contrast focus and selection highlights by inverting RGB channels
/// (`31 - r`, `63 - g`, `31 - b`). Requires a readback-capable draw target (`PixelRead`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InverterWidget {
    pub enabled: bool,
}

impl Default for InverterWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl InverterWidget {
    /// Create a new active inverter widget.
    pub const fn new() -> Self {
        Self { enabled: true }
    }

    /// Render the inverter effect across `bounds`.
    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, bounds: Rect) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565> + PixelRead,
        C: Compositor<D>,
    {
        if !self.enabled || bounds.w == 0 || bounds.h == 0 {
            return Ok(());
        }
        ctx.reverse_colour_rect(bounds)
    }
}

impl Widget for InverterWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::State => Some(PropertyValue::Bool(self.enabled)),
            _ => None,
        }
    }

    fn set_property<'a>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::State, PropertyValue::Bool(b)) => {
                self.enabled = b;
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;
    use embedded_graphics_core::pixelcolor::RgbColor;

    #[test]
    fn test_inverter_widget_render() {
        let mut fb = Framebuffer::<400>::new(20, 20);
        fb.clear_color(Rgb565::RED);

        let inverter = InverterWidget::new();
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        assert!(inverter.render(&mut ctx, Rect::new(5, 5, 10, 10)).is_ok());

        // Center pixel should be inverted: 31 - 31 = 0, 63 - 0 = 63, 31 - 0 = 31 => Cyan
        let center = fb.pixels()[10 * 20 + 10];
        assert_eq!(center, Rgb565::new(0, 63, 31));

        // Outside pixel should remain RED
        let outside = fb.pixels()[0];
        assert_eq!(outside, Rgb565::RED);
    }
}
