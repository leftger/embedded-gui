//! Coverage tests for list repeater selection and seven-segment geometry.

use embedded_graphics_core::pixelcolor::{Rgb565, WebColors};
use embedded_gui::prelude::*;
use embedded_gui::widgets::{RepeaterWidget, SegmentStyle, SevenSegmentDisplay};

#[test]
fn repeater_widget_selection_scroll_and_item_bounds() {
    let mut repeater = RepeaterWidget::<3>::new(20).with_spacing(2);
    repeater.total_count = 5;
    repeater.set_selected(4);
    assert!(repeater.set_selected(2));
    assert!(repeater.bump_selection(1));
    assert!(repeater.bump_selection(-1));
    assert!(repeater.bump_selection(1));

    repeater.total_count = 0;
    assert!(!repeater.bump_selection(1));

    let bounds = repeater.item_bounds(Rect::new(0, 10, 50, 20), 1);
    assert!(bounds.y > 10);
}

#[test]
fn seven_segment_style_and_digit_geometry() {
    let style = SegmentStyle::new(Rgb565::CSS_RED).with_ghosting(Rgb565::CSS_DARK_GRAY);
    let display = SevenSegmentDisplay::new("12", style);
    let segments = SevenSegmentDisplay::digit_segments(Rect::new(0, 0, 20, 30), 2);
    assert_eq!(segments.len(), 7);
    assert_eq!(display.text, "12");
}
