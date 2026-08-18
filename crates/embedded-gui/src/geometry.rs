use heapless::Vec;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub const fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub const fn right(self) -> i32 {
        self.x + self.w as i32
    }

    pub const fn bottom(self) -> i32 {
        self.y + self.h as i32
    }

    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }

    pub fn intersects(self, other: Self) -> bool {
        !self.intersection(other).is_empty()
    }

    pub fn intersection(self, other: Self) -> Self {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = self.right().min(other.right());
        let y1 = self.bottom().min(other.bottom());

        if x1 <= x0 || y1 <= y0 {
            Self::empty()
        } else {
            Self::new(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32)
        }
    }

    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }

        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = self.right().max(other.right());
        let y1 = self.bottom().max(other.bottom());
        Self::new(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32)
    }

    pub fn inset(self, edges: EdgeInsets) -> Self {
        let left = edges.left.max(0) as u32;
        let right = edges.right.max(0) as u32;
        let top = edges.top.max(0) as u32;
        let bottom = edges.bottom.max(0) as u32;
        let shrink_w = left.saturating_add(right).min(self.w);
        let shrink_h = top.saturating_add(bottom).min(self.h);

        Self::new(
            self.x + left as i32,
            self.y + top as i32,
            self.w - shrink_w,
            self.h - shrink_h,
        )
    }

    /// Positions `self` relative to `reference` along horizontal and vertical alignment axes.
    pub fn align_to(self, reference: Rect, h: HorizontalAlign, v: VerticalAlign) -> Rect {
        let x = match h {
            HorizontalAlign::Left => reference.x,
            HorizontalAlign::Center => reference.x + (reference.w as i32 - self.w as i32) / 2,
            HorizontalAlign::Right => reference.right() - self.w as i32,
            HorizontalAlign::LeftToRight => reference.right(),
            HorizontalAlign::RightToLeft => reference.x - self.w as i32,
        };
        let y = match v {
            VerticalAlign::Top => reference.y,
            VerticalAlign::Center => reference.y + (reference.h as i32 - self.h as i32) / 2,
            VerticalAlign::Bottom => reference.bottom() - self.h as i32,
            VerticalAlign::TopToBottom => reference.bottom(),
            VerticalAlign::BottomToTop => reference.y - self.h as i32,
        };
        Rect::new(x, y, self.w, self.h)
    }

    /// Positions `self` relative to `reference` using a compound 2D anchor preset.
    pub fn anchor_to(self, reference: Rect, anchor: Anchor) -> Rect {
        match anchor {
            Anchor::TopLeft => self.align_to(reference, HorizontalAlign::Left, VerticalAlign::Top),
            Anchor::TopCenter => {
                self.align_to(reference, HorizontalAlign::Center, VerticalAlign::Top)
            }
            Anchor::TopRight => {
                self.align_to(reference, HorizontalAlign::Right, VerticalAlign::Top)
            }
            Anchor::CenterLeft => {
                self.align_to(reference, HorizontalAlign::Left, VerticalAlign::Center)
            }
            Anchor::Center => {
                self.align_to(reference, HorizontalAlign::Center, VerticalAlign::Center)
            }
            Anchor::CenterRight => {
                self.align_to(reference, HorizontalAlign::Right, VerticalAlign::Center)
            }
            Anchor::BottomLeft => {
                self.align_to(reference, HorizontalAlign::Left, VerticalAlign::Bottom)
            }
            Anchor::BottomCenter => {
                self.align_to(reference, HorizontalAlign::Center, VerticalAlign::Bottom)
            }
            Anchor::BottomRight => {
                self.align_to(reference, HorizontalAlign::Right, VerticalAlign::Bottom)
            }
            Anchor::OutsideTop => self.align_to(
                reference,
                HorizontalAlign::Center,
                VerticalAlign::BottomToTop,
            ),
            Anchor::OutsideBottom => self.align_to(
                reference,
                HorizontalAlign::Center,
                VerticalAlign::TopToBottom,
            ),
            Anchor::OutsideLeft => self.align_to(
                reference,
                HorizontalAlign::RightToLeft,
                VerticalAlign::Center,
            ),
            Anchor::OutsideRight => self.align_to(
                reference,
                HorizontalAlign::LeftToRight,
                VerticalAlign::Center,
            ),
        }
    }
}

/// Horizontal alignment policy for relative positioning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
    /// Place directly adjacent to the right outer edge of reference.
    LeftToRight,
    /// Place directly adjacent to the left outer edge of reference.
    RightToLeft,
}

/// Vertical alignment policy for relative positioning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
    /// Place directly below the bottom outer edge of reference.
    TopToBottom,
    /// Place directly above the top outer edge of reference.
    BottomToTop,
}

/// Compound 2D anchor presets for positioning UI elements relative to parents or siblings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    OutsideTop,
    OutsideBottom,
    OutsideLeft,
    OutsideRight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeInsets {
    pub left: i16,
    pub right: i16,
    pub top: i16,
    pub bottom: i16,
}

impl EdgeInsets {
    pub const fn zero() -> Self {
        Self {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        }
    }

    pub const fn all(v: i16) -> Self {
        Self {
            left: v,
            right: v,
            top: v,
            bottom: v,
        }
    }

    pub const fn symmetric(horizontal: i16, vertical: i16) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyError {
    Full,
}

pub struct DirtyTracker<const N: usize> {
    regions: Vec<Rect, N>,
}

impl<const N: usize> DirtyTracker<N> {
    pub const fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.regions.clear();
    }

    pub fn add(&mut self, rect: Rect) -> Result<(), DirtyError> {
        if rect.is_empty() {
            return Ok(());
        }

        if self.regions.iter().any(|r| r.intersects(rect)) {
            let mut merged = rect;
            let mut i = 0;
            while i < self.regions.len() {
                if self.regions[i].intersects(merged) {
                    merged = merged.union(self.regions.swap_remove(i));
                } else {
                    i += 1;
                }
            }
            return self.regions.push(merged).map_err(|_| DirtyError::Full);
        }

        self.regions.push(rect).map_err(|_| DirtyError::Full)
    }

    pub fn mark_all(&mut self, rect: Rect) -> Result<(), DirtyError> {
        self.regions.clear();
        self.add(rect)
    }

    pub fn as_slice(&self) -> &[Rect] {
        self.regions.as_slice()
    }

    pub fn bounding_rect(&self) -> Option<Rect> {
        let mut iter = self.regions.iter().copied();
        let first = iter.next()?;
        Some(iter.fold(first, Rect::union))
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }
}

impl<const N: usize> Default for DirtyTracker<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_empty_and_contains() {
        let r = Rect::new(10, 20, 30, 40);
        assert!(!r.is_empty());
        assert_eq!(r.right(), 40);
        assert_eq!(r.bottom(), 60);

        assert!(r.contains(10, 20));
        assert!(r.contains(39, 59));
        assert!(!r.contains(40, 60));
        assert!(!r.contains(9, 20));

        let empty = Rect::empty();
        assert!(empty.is_empty());
        assert!(!empty.contains(0, 0));
    }

    #[test]
    fn test_rect_intersection_and_union() {
        let r1 = Rect::new(0, 0, 20, 20);
        let r2 = Rect::new(10, 10, 20, 20);

        assert!(r1.intersects(r2));
        assert_eq!(r1.intersection(r2), Rect::new(10, 10, 10, 10));
        assert_eq!(r1.union(r2), Rect::new(0, 0, 30, 30));

        let r3 = Rect::new(50, 50, 10, 10);
        assert!(!r1.intersects(r3));
        assert!(r1.intersection(r3).is_empty());
    }

    #[test]
    fn test_rect_inset() {
        let r = Rect::new(10, 10, 40, 40);
        let inset = r.inset(EdgeInsets::all(5));
        assert_eq!(inset, Rect::new(15, 15, 30, 30));

        // Excess inset saturates width and height to 0
        let over_inset = r.inset(EdgeInsets::all(30));
        assert_eq!(over_inset.w, 0);
        assert_eq!(over_inset.h, 0);
    }

    #[test]
    fn test_dirty_tracker() {
        let mut dt: DirtyTracker<4> = DirtyTracker::new();
        assert!(dt.is_empty());

        dt.add(Rect::new(0, 0, 10, 10)).unwrap();
        assert_eq!(dt.as_slice().len(), 1);

        // Add non-overlapping rect
        dt.add(Rect::new(20, 20, 10, 10)).unwrap();
        assert_eq!(dt.as_slice().len(), 2);

        // Add overlapping rect to trigger merge
        dt.add(Rect::new(5, 5, 10, 10)).unwrap();
        // The overlapping rects merge into one larger region
        assert_eq!(dt.as_slice().len(), 2);

        assert_eq!(dt.bounding_rect(), Some(Rect::new(0, 0, 30, 30)));

        dt.clear();
        assert!(dt.is_empty());
        assert_eq!(dt.bounding_rect(), None);
    }

    #[test]
    fn test_rect_align_to_and_anchor_to() {
        let parent = Rect::new(0, 0, 100, 100);
        let child = Rect::new(0, 0, 20, 20);

        // Center alignment
        let centered = child.align_to(parent, HorizontalAlign::Center, VerticalAlign::Center);
        assert_eq!(centered, Rect::new(40, 40, 20, 20));

        // Top-right anchor
        let top_right = child.anchor_to(parent, Anchor::TopRight);
        assert_eq!(top_right, Rect::new(80, 0, 20, 20));

        // Outside-bottom anchor (dropdown/tooltip)
        let dropdown = child.anchor_to(parent, Anchor::OutsideBottom);
        assert_eq!(dropdown, Rect::new(40, 100, 20, 20));

        // Outside-right (badge/adjacent icon)
        let badge = child.align_to(parent, HorizontalAlign::LeftToRight, VerticalAlign::Top);
        assert_eq!(badge, Rect::new(100, 0, 20, 20));
    }
}
