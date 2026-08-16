//! Optional bridges to the wider embedded-graphics ecosystem.
//!
//! embedded-gui's own [`crate::layout`] and [`crate::text`] stay the default,
//! dependency-light path (only `embedded-graphics-core` + `heapless`). This
//! module adds opt-in adapters to two independently published crates for
//! projects that want them:
//!
//! - `embedded-layout` feature: [`layout::RectView`], a [`crate::geometry::Rect`]
//!   wrapper implementing `embedded_layout::View`, so embedded_layout's
//!   `LinearLayout`/alignment algorithms can arrange embedded-gui widget rects.
//! - `embedded-text` feature: [`text::text_box`], a helper to build an
//!   `embedded_text::TextBox` over a [`crate::geometry::Rect`], plus
//!   [`crate::render::RenderCtx::draw_embedded_graphics`] to draw it (or any
//!   other `embedded_graphics::Drawable`) during a widget's render pass.
//!
//! Neither adapter changes how existing widgets render by default.

#[cfg(any(
    feature = "embedded-graphics",
    feature = "embedded-text",
    feature = "embedded-layout"
))]
pub mod geometry;

#[cfg(feature = "embedded-graphics")]
pub mod graphics;

#[cfg(feature = "embedded-layout")]
pub mod layout;

#[cfg(feature = "embedded-text")]
pub mod text;

#[cfg(feature = "embedded-3dgfx")]
pub mod three_d;
