//! 9-Patch / 9-Slice scaling engine adapted from `bevy_sprite::texture_slice`.
//!
//! Enables UI assets (buttons, modal windows, panels) to scale to arbitrary
//! bounding boxes while preserving crisp, unscaled corners and properly
//! repeating or stretching border edges and centers.

use crate::geometry::Rect;

/// Border insets defining the slice lines for a 9-patch texture.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct BorderRect {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl BorderRect {
    pub const fn all(inset: u16) -> Self {
        Self {
            top: inset,
            right: inset,
            bottom: inset,
            left: inset,
        }
    }

    pub const fn symmetric(vertical: u16, horizontal: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

/// Scale mode for side and center segments of a 9-patch slice.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum SliceScaleMode {
    /// The slice will stretch to fill the destination area.
    #[default]
    Stretch,
    /// The slice will tile (repeat) to fill the destination area.
    Tile,
}

/// Configuration for partitioning a texture or bitmap into 9 scalable regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NineSlice {
    pub border: BorderRect,
    pub center_mode: SliceScaleMode,
    pub sides_mode: SliceScaleMode,
}

impl NineSlice {
    pub const fn new(border: BorderRect) -> Self {
        Self {
            border,
            center_mode: SliceScaleMode::Stretch,
            sides_mode: SliceScaleMode::Stretch,
        }
    }

    pub const fn with_modes(
        border: BorderRect,
        center_mode: SliceScaleMode,
        sides_mode: SliceScaleMode,
    ) -> Self {
        Self {
            border,
            center_mode,
            sides_mode,
        }
    }

    /// Computes the 9 source and destination sub-rectangles for scaling from
    /// `source_bounds` to `target_bounds`.
    pub fn compute_slices(&self, source_bounds: Rect, target_bounds: Rect) -> NineSliceLayout {
        // Source slices
        let s_left = self.border.left as u32;
        let s_right = self.border.right as u32;
        let s_top = self.border.top as u32;
        let s_bottom = self.border.bottom as u32;

        let s_mid_w = source_bounds.w.saturating_sub(s_left + s_right);
        let s_mid_h = source_bounds.h.saturating_sub(s_top + s_bottom);

        // Destination slices: if target is smaller than corners, clamp them
        let max_corner_w = target_bounds.w / 2;
        let max_corner_h = target_bounds.h / 2;
        let d_left = s_left.min(max_corner_w);
        let d_right = s_right.min(max_corner_w);
        let d_top = s_top.min(max_corner_h);
        let d_bottom = s_bottom.min(max_corner_h);

        let d_mid_w = target_bounds.w.saturating_sub(d_left + d_right);
        let d_mid_h = target_bounds.h.saturating_sub(d_top + d_bottom);

        let sx0 = source_bounds.x;
        let sx1 = sx0 + s_left as i32;
        let sx2 = sx1 + s_mid_w as i32;

        let sy0 = source_bounds.y;
        let sy1 = sy0 + s_top as i32;
        let sy2 = sy1 + s_mid_h as i32;

        let dx0 = target_bounds.x;
        let dx1 = dx0 + d_left as i32;
        let dx2 = dx1 + d_mid_w as i32;

        let dy0 = target_bounds.y;
        let dy1 = dy0 + d_top as i32;
        let dy2 = dy1 + d_mid_h as i32;

        NineSliceLayout {
            top_left: SlicePair {
                source: Rect::new(sx0, sy0, s_left, s_top),
                dest: Rect::new(dx0, dy0, d_left, d_top),
            },
            top_edge: SlicePair {
                source: Rect::new(sx1, sy0, s_mid_w, s_top),
                dest: Rect::new(dx1, dy0, d_mid_w, d_top),
            },
            top_right: SlicePair {
                source: Rect::new(sx2, sy0, s_right, s_top),
                dest: Rect::new(dx2, dy0, d_right, d_top),
            },
            left_edge: SlicePair {
                source: Rect::new(sx0, sy1, s_left, s_mid_h),
                dest: Rect::new(dx0, dy1, d_left, d_mid_h),
            },
            center: SlicePair {
                source: Rect::new(sx1, sy1, s_mid_w, s_mid_h),
                dest: Rect::new(dx1, dy1, d_mid_w, d_mid_h),
            },
            right_edge: SlicePair {
                source: Rect::new(sx2, sy1, s_right, s_mid_h),
                dest: Rect::new(dx2, dy1, d_right, d_mid_h),
            },
            bottom_left: SlicePair {
                source: Rect::new(sx0, sy2, s_left, s_bottom),
                dest: Rect::new(dx0, dy2, d_left, d_bottom),
            },
            bottom_edge: SlicePair {
                source: Rect::new(sx1, sy2, s_mid_w, s_bottom),
                dest: Rect::new(dx1, dy2, d_mid_w, d_bottom),
            },
            bottom_right: SlicePair {
                source: Rect::new(sx2, sy2, s_right, s_bottom),
                dest: Rect::new(dx2, dy2, d_right, d_bottom),
            },
        }
    }
}

/// A source rectangle matched with its destination target rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlicePair {
    pub source: Rect,
    pub dest: Rect,
}

/// The layout of all 9 sliced segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NineSliceLayout {
    pub top_left: SlicePair,
    pub top_edge: SlicePair,
    pub top_right: SlicePair,
    pub left_edge: SlicePair,
    pub center: SlicePair,
    pub right_edge: SlicePair,
    pub bottom_left: SlicePair,
    pub bottom_edge: SlicePair,
    pub bottom_right: SlicePair,
}

impl NineSliceLayout {
    /// Iterates through all 9 slice pairs in row-major order.
    pub fn iter(&self) -> impl Iterator<Item = SlicePair> {
        [
            self.top_left,
            self.top_edge,
            self.top_right,
            self.left_edge,
            self.center,
            self.right_edge,
            self.bottom_left,
            self.bottom_edge,
            self.bottom_right,
        ]
        .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nine_slice_partitioning() {
        let border = BorderRect::new(4, 4, 4, 4);
        let slicer = NineSlice::new(border);

        let source = Rect::new(0, 0, 16, 16);
        let target = Rect::new(10, 20, 100, 50);

        let layout = slicer.compute_slices(source, target);

        // Top-left corner: source 4x4, dest 4x4
        assert_eq!(layout.top_left.source, Rect::new(0, 0, 4, 4));
        assert_eq!(layout.top_left.dest, Rect::new(10, 20, 4, 4));

        // Top-edge: source mid 8x4, dest mid (100 - 8 = 92) x 4
        assert_eq!(layout.top_edge.source, Rect::new(4, 0, 8, 4));
        assert_eq!(layout.top_edge.dest, Rect::new(14, 20, 92, 4));

        // Center: source 8x8, dest 92x42 (50 - 8 = 42)
        assert_eq!(layout.center.source, Rect::new(4, 4, 8, 8));
        assert_eq!(layout.center.dest, Rect::new(14, 24, 92, 42));

        // Bottom-right corner:
        assert_eq!(layout.bottom_right.source, Rect::new(12, 12, 4, 4));
        assert_eq!(layout.bottom_right.dest, Rect::new(106, 66, 4, 4));
    }
}
