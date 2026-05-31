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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeInsets {
    pub left: i16,
    pub right: i16,
    pub top: i16,
    pub bottom: i16,
}

impl EdgeInsets {
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
