//! Showcase: Relative Anchor Positioning & Flex Justification
//!
//! Demonstrates:
//! 1. Relative alignment (`Rect::align_to`) and compound 2D presets (`Rect::anchor_to`).
//! 2. Positioning badges, tooltips, and dropdowns relative to reference elements without manual coordinate arithmetic.
//! 3. Primary axis space distribution in `LinearLayout` (`JustifyContent::SpaceBetween`, `SpaceAround`, `SpaceEvenly`).

use embedded_gui::{
    geometry::{Anchor, HorizontalAlign, Rect, VerticalAlign},
    layout::{JustifyContent, LayoutItem, LinearLayout},
};

fn main() {
    println!("=== embedded-gui: Relative Anchors & Flex Justify Showcase ===");

    // 1. Relative Positioning & Compound Anchors
    let screen = Rect::new(0, 0, 320, 240);
    let card = Rect::new(0, 0, 200, 120).anchor_to(screen, Anchor::Center);
    println!("Card centered on 320x240 screen: {:?}", card);

    // Position a status badge at the top-right corner of the card
    let badge_size = Rect::new(0, 0, 24, 16);
    let badge = badge_size.anchor_to(card, Anchor::TopRight);
    println!("Badge at Card TopRight: {:?}", badge);

    // Position a dropdown menu directly below the card (outside bottom)
    let dropdown_size = Rect::new(0, 0, 200, 60);
    let dropdown = dropdown_size.align_to(card, HorizontalAlign::Left, VerticalAlign::TopToBottom);
    println!("Dropdown menu outside bottom: {:?}", dropdown);

    // Position an adjacent side action button to the right of the card
    let side_btn_size = Rect::new(0, 0, 32, 40);
    let side_btn =
        side_btn_size.align_to(card, HorizontalAlign::LeftToRight, VerticalAlign::Center);
    println!("Side button to the right of card: {:?}", side_btn);

    // 2. LinearLayout with JustifyContent
    let items = [
        LayoutItem::fixed(40),
        LayoutItem::fixed(40),
        LayoutItem::fixed(40),
    ];
    let container = Rect::new(0, 0, 300, 50);
    let mut slots = [Rect::empty(); 3];

    println!("\nArranging 3x 40px items in 300px container:");

    // SpaceBetween
    let layout_between = LinearLayout::row()
        .with_gap(0)
        .with_justify(JustifyContent::SpaceBetween);
    layout_between.arrange_items(container, &items, &mut slots);
    println!("  SpaceBetween slots:");
    for (i, slot) in slots.iter().enumerate() {
        println!("    Item {}: x={}, w={}", i, slot.x, slot.w);
    }

    // SpaceAround
    let layout_around = LinearLayout::row()
        .with_gap(0)
        .with_justify(JustifyContent::SpaceAround);
    layout_around.arrange_items(container, &items, &mut slots);
    println!("  SpaceAround slots:");
    for (i, slot) in slots.iter().enumerate() {
        println!("    Item {}: x={}, w={}", i, slot.x, slot.w);
    }

    // SpaceEvenly
    let layout_evenly = LinearLayout::row()
        .with_gap(0)
        .with_justify(JustifyContent::SpaceEvenly);
    layout_evenly.arrange_items(container, &items, &mut slots);
    println!("  SpaceEvenly slots:");
    for (i, slot) in slots.iter().enumerate() {
        println!("    Item {}: x={}, w={}", i, slot.x, slot.w);
    }

    println!("\nRelative anchors and flex justification showcase completed successfully!");
}
