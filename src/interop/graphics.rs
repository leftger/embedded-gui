//! Adapter for `embedded-graphics` primitives, font styles, and drawables.

use embedded_graphics::{
    Drawable, mono_font::MonoTextStyle, pixelcolor::Rgb565, primitives::Rectangle,
};

use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx, TextStyle},
};

/// Converts an embedded-gui [`Rect`] to an `embedded_graphics::primitives::Rectangle`.
pub fn rect_to_rectangle(rect: Rect) -> Rectangle {
    rect.into()
}

/// Converts an `embedded_graphics::primitives::Rectangle` to an embedded-gui [`Rect`].
pub fn rectangle_to_rect(rectangle: Rectangle) -> Rect {
    rectangle.into()
}

/// Helper function to draw any `embedded_graphics::Drawable` inside an embedded-gui [`RenderCtx`].
pub fn draw_drawable<D, C, T>(
    ctx: &mut RenderCtx<'_, D, C>,
    drawable: &T,
) -> Result<T::Output, D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
    T: Drawable<Color = Rgb565>,
{
    ctx.draw_embedded_graphics(drawable)
}

/// Builds a [`TextStyle`] from an `embedded-graphics` [`MonoTextStyle`].
pub fn text_style_from_mono(mono_style: &MonoTextStyle<'static, Rgb565>) -> TextStyle {
    mono_style.into()
}
