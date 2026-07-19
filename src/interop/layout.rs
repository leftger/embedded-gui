//! Adapter for the `embedded-layout` crate's `View`-based arrangement algorithms.
//!
//! embedded-gui's own [`crate::layout::LinearLayout`] arranges plain
//! [`crate::geometry::Rect`]s in place, with a richer constraint model
//! (`Percent`/`Ratio`/weighted `Fill`, grow/shrink) than embedded-layout
//! currently offers. [`RectView`] exists for the opposite direction: when you
//! want embedded-layout's own `LinearLayout`/alignment/spacing algorithms
//! (e.g. `DistributeFill`, `align_to`) to arrange embedded-gui widget slots,
//! wrap each slot's [`Rect`] in a `RectView`, let embedded-layout position
//! them, then read the resulting rects back out.
//!
//! ```
//! use embedded_gui::geometry::Rect;
//! use embedded_gui::interop::layout::RectView;
//! use embedded_layout::{
//!     layout::linear::{FixedMargin, LinearLayout},
//!     prelude::*,
//! };
//!
//! let mut slots = [
//!     RectView::from(Rect::new(0, 0, 40, 10)),
//!     RectView::from(Rect::new(0, 0, 40, 16)),
//! ];
//! let parent = RectView::from(Rect::new(0, 0, 40, 64));
//!
//! let arranged = LinearLayout::vertical(Views::new(&mut slots))
//!     .with_spacing(FixedMargin(2))
//!     .arrange()
//!     .align_to(&parent, horizontal::Left, vertical::Top);
//!
//! let rects: [Rect; 2] = [arranged.inner()[0].into(), arranged.inner()[1].into()];
//! assert_eq!(rects[0], Rect::new(0, 0, 40, 10));
//! assert_eq!(rects[1], Rect::new(0, 12, 40, 16));
//! ```

use embedded_graphics::{prelude::Point, primitives::Rectangle};
use embedded_layout::View;

use crate::geometry::Rect;

/// Wraps a [`Rect`] so it can be arranged by embedded-layout's `View`-based
/// layout algorithms (`LinearLayout`, alignment, spacing).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RectView(pub Rect);

impl From<Rect> for RectView {
    fn from(rect: Rect) -> Self {
        Self(rect)
    }
}

impl From<RectView> for Rect {
    fn from(view: RectView) -> Self {
        view.0
    }
}

impl View for RectView {
    fn translate_impl(&mut self, by: Point) {
        self.0.x += by.x;
        self.0.y += by.y;
    }

    fn bounds(&self) -> Rectangle {
        self.0.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_layout::{
        layout::linear::{FixedMargin, LinearLayout},
        prelude::*,
    };

    #[test]
    fn arranges_rects_vertically_with_spacing() {
        let mut slots = [
            RectView::from(Rect::new(0, 0, 40, 10)),
            RectView::from(Rect::new(0, 0, 40, 16)),
        ];
        let parent = RectView::from(Rect::new(0, 0, 40, 64));

        let arranged = LinearLayout::vertical(Views::new(&mut slots))
            .with_spacing(FixedMargin(2))
            .arrange()
            .align_to(&parent, horizontal::Left, vertical::Top);

        let rects: [Rect; 2] = [arranged.inner()[0].into(), arranged.inner()[1].into()];
        assert_eq!(rects[0], Rect::new(0, 0, 40, 10));
        assert_eq!(rects[1], Rect::new(0, 12, 40, 16));
    }
}
