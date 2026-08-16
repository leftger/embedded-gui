//! Circular / Round Screen Geometry and Unobstructed Area adaptation.
//!
//! Provides line chord width calculations for circular displays (e.g. 180×180 /
//! 240×240 GC9A01 LCDs) and reactive unobstructed area layout management.

use crate::geometry::Rect;
#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;

/// Calculates the horizontal chord width across a circle at a given vertical distance from center.
///
/// Formula: $w = 2 \times \sqrt{R^2 - y^2}$
#[inline]
pub fn circle_chord_width(radius: u32, y_offset_from_center: i32) -> u32 {
    let r = radius as i32;
    let y = y_offset_from_center.abs();
    if y >= r {
        return 0;
    }
    let r2 = r * r;
    let y2 = y * y;
    let half_w = ((r2 - y2) as f32).sqrt() as u32;
    half_w * 2
}

/// Calculates the safe bounding rectangle for text or widget lines on a circular display.
///
/// Ensures elements remain fully within the circular screen perimeter at the given vertical position.
pub fn round_screen_line_bounds(diameter: u32, line_y: i32, line_height: u32) -> Rect {
    let radius = (diameter / 2) as i32;
    let center_y = radius;

    // Check top and bottom edges of the line
    let y_top = line_y - center_y;
    let y_bottom = (line_y + line_height as i32) - center_y;

    let w_top = circle_chord_width(radius as u32, y_top);
    let w_bottom = circle_chord_width(radius as u32, y_bottom);
    let min_width = w_top.min(w_bottom);

    let offset_x = (radius - (min_width as i32 / 2)).max(0);
    Rect::new(offset_x, line_y, min_width, line_height)
}

/// Manages dynamic unobstructed screen bounds when system overlays or banners
/// (e.g., status bars, timeline peek, heads-up notifications) cover portions of the display.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnobstructedArea {
    pub screen_size: Rect,
    pub inset_top: u16,
    pub inset_bottom: u16,
    pub inset_left: u16,
    pub inset_right: u16,
}

impl UnobstructedArea {
    pub const fn new(screen_size: Rect) -> Self {
        Self {
            screen_size,
            inset_top: 0,
            inset_bottom: 0,
            inset_left: 0,
            inset_right: 0,
        }
    }

    pub fn set_insets(&mut self, top: u16, bottom: u16, left: u16, right: u16) {
        self.inset_top = top;
        self.inset_bottom = bottom;
        self.inset_left = left;
        self.inset_right = right;
    }

    /// Returns the currently visible and unobstructed rectangle.
    pub fn visible_rect(&self) -> Rect {
        let x = self.screen_size.x + self.inset_left as i32;
        let y = self.screen_size.y + self.inset_top as i32;
        let w = self
            .screen_size
            .w
            .saturating_sub((self.inset_left + self.inset_right) as u32);
        let h = self
            .screen_size
            .h
            .saturating_sub((self.inset_top + self.inset_bottom) as u32);
        Rect::new(x, y, w, h)
    }

    /// Returns whether the display is partially obstructed.
    pub const fn is_obstructed(&self) -> bool {
        self.inset_top > 0 || self.inset_bottom > 0 || self.inset_left > 0 || self.inset_right > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_chord_calculations() {
        let radius = 90; // 180px diameter circle
        // At center (y = 0), chord width equals full diameter
        assert_eq!(circle_chord_width(radius, 0), 180);

        // Near top/bottom (y = 80)
        let w = circle_chord_width(radius, 80);
        assert!(w > 0 && w < 180);

        // Out of bounds
        assert_eq!(circle_chord_width(radius, 95), 0);
    }

    #[test]
    fn test_round_screen_line_bounds() {
        let diameter = 180;
        let center_line = round_screen_line_bounds(diameter, 80, 20);
        assert!(center_line.w > 160);
        assert_eq!(center_line.y, 80);
    }

    #[test]
    fn test_unobstructed_area_rect() {
        let mut area = UnobstructedArea::new(Rect::new(0, 0, 144, 168));
        assert_eq!(area.visible_rect(), Rect::new(0, 0, 144, 168));
        assert!(!area.is_obstructed());

        area.set_insets(16, 24, 0, 0);
        assert!(area.is_obstructed());
        assert_eq!(area.visible_rect(), Rect::new(0, 16, 144, 128));
    }
}
