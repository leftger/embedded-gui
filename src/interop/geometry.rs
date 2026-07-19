use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Rectangle,
};

use crate::geometry::Rect;

impl From<Rect> for Rectangle {
    fn from(rect: Rect) -> Self {
        Rectangle::new(Point::new(rect.x, rect.y), Size::new(rect.w, rect.h))
    }
}

impl From<Rectangle> for Rect {
    fn from(rectangle: Rectangle) -> Self {
        Rect::new(
            rectangle.top_left.x,
            rectangle.top_left.y,
            rectangle.size.width,
            rectangle.size.height,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_to_rectangle_round_trip() {
        let rect = Rect::new(3, -4, 12, 7);
        let rectangle: Rectangle = rect.into();
        assert_eq!(rectangle.top_left, Point::new(3, -4));
        assert_eq!(rectangle.size, Size::new(12, 7));
        assert_eq!(Rect::from(rectangle), rect);
    }
}
