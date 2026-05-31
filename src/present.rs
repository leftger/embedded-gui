use crate::geometry::Rect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PresentRegion {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl PresentRegion {
    pub const fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

impl From<Rect> for PresentRegion {
    fn from(rect: Rect) -> Self {
        if rect.is_empty() {
            return Self::default();
        }

        Self {
            x: rect.x.max(0) as usize,
            y: rect.y.max(0) as usize,
            width: rect.w as usize,
            height: rect.h as usize,
        }
    }
}
