//! Showcase: Wearable OS Subsystems & UI Patterns
//!
//! Demonstrates:
//! 1. **Timeline Relationship Bars (`TimelineNodeWidget`)**: Chronological connective bars linking past, active, and future event pins.
//! 2. **Reactive Peek Banners (`PeekBannerWidget`)**: Heads-up reminder ribbon modifying unobstructed screen canvas.
//! 3. **Modal Notification Sheets (`NotificationSheetWidget`)**: Priority alert popups with action choices and auto-dismiss countdowns.
//! 4. **Cascading Action Menus (`ActionMenuWidget`)**: Hierarchical nested action sheets with highlight cursor.
//! 5. **Rich Multi-Span Text Nodes (`RichTextNodeWidget`)**: Formatted inline text tags and badge spans.

use embedded_graphics_core::pixelcolor::{Rgb565, WebColors};
use embedded_gui::{
    framebuffer::Framebuffer,
    geometry::Rect,
    render::RenderCtx,
    round::UnobstructedArea,
    widgets::{
        ActionMenuWidget, NotificationPriority, NotificationSheetWidget, PeekBannerWidget,
        RichTextNodeWidget, TextSpan, TimelineNodeState, TimelineNodeWidget,
    },
};

fn main() {
    println!("=== embedded-gui: Wearable OS Subsystems Showcase ===");

    let screen = Rect::new(0, 0, 240, 240);
    let mut fb = Framebuffer::<{ 240 * 240 }>::new(240, 240);
    let mut ctx = RenderCtx::new(&mut fb, screen);

    // 1. Timeline Connective RelBar Demonstration
    println!("\n1. Timeline Relationship Connector Bars:");
    let past_node = TimelineNodeWidget::new(TimelineNodeState::Past);
    let mut active_node = TimelineNodeWidget::new(TimelineNodeState::ActiveNow);
    active_node.active_color = Rgb565::CSS_ORANGE;
    let future_node = TimelineNodeWidget::new(TimelineNodeState::Future);

    let past_slot = Rect::new(12, 10, 16, 40);
    let active_slot = Rect::new(12, 50, 16, 40);
    let future_slot = Rect::new(12, 90, 16, 40);

    past_node.render(&mut ctx, past_slot).unwrap();
    active_node.render(&mut ctx, active_slot).unwrap();
    future_node.render(&mut ctx, future_slot).unwrap();

    println!("  Past pin connector:   {:?}", past_slot);
    println!("  Active NOW pin node:  {:?}", active_slot);
    println!("  Upcoming pin node:    {:?}", future_slot);

    // 2. Reactive Peek Banner adapting UnobstructedArea
    println!("\n2. Reactive Canvas Peek Banner:");
    let mut unobstructed = UnobstructedArea::new(screen);
    let peek = PeekBannerWidget::new("TEAM SYNC (10m)");
    peek.apply_to_unobstructed_area(&mut unobstructed);

    let banner_rect = Rect::new(screen.x, screen.y, screen.w, peek.height as u32);
    peek.render(&mut ctx, banner_rect).unwrap();
    println!("  Peek banner rendered at: {:?}", banner_rect);
    println!(
        "  Canvas area adjusted to: {:?}",
        unobstructed.visible_rect()
    );

    // 3. Multi-Span Rich Text Flow Node
    println!("\n3. Multi-Span Rich Text Node with Badges:");
    let mut text_node = RichTextNodeWidget::<4>::new();
    text_node
        .push_span(TextSpan::badge(
            "CRITICAL",
            Rgb565::CSS_WHITE,
            Rgb565::new(28, 4, 4),
        ))
        .unwrap();
    text_node
        .push_span(TextSpan::plain("CPU core temp 48C", Rgb565::CSS_WHITE))
        .unwrap();

    let text_rect = Rect::new(40, 60, 190, 24);
    text_node.render(&mut ctx, text_rect).unwrap();
    println!(
        "  Rendered {} styled text spans in {:?}",
        text_node.spans.len(),
        text_rect
    );

    // 4. Modal Notification Sheet with Action Buttons & Progress Bar
    println!("\n4. Modal Notification Sheet:");
    let mut notif = NotificationSheetWidget::<3>::new(
        "CALENDAR REMINDER",
        "Architecture Design Review @ 10:00",
        NotificationPriority::Important,
    );
    notif.add_action("DISMISS", 101).unwrap();
    notif.add_action("SNOOZE", 102).unwrap();
    notif.auto_dismiss_progress = 0.75; // 75% timer remaining

    let notif_rect = Rect::new(20, 140, 200, 85);
    notif.render(&mut ctx, notif_rect).unwrap();
    println!(
        "  Rendered modal notification card with {} actions & 75% countdown timer.",
        notif.actions.len()
    );

    // 5. Cascading Hierarchical Action Menu
    println!("\n5. Cascading Action Menu:");
    let mut menu = ActionMenuWidget::<4>::new(Some("SYSTEM ACTIONS"));
    menu.add_item("Wireless Sync", 1, true).unwrap();
    menu.add_item("Do Not Disturb", 2, false).unwrap();
    menu.add_item("Power Options", 3, true).unwrap();
    menu.selected_index = 0;

    let menu_rect = Rect::new(30, 40, 180, 75);
    menu.render(&mut ctx, menu_rect).unwrap();
    println!(
        "  Rendered action menu with {} items, highlighted index {}.",
        menu.items.len(),
        menu.selected_index
    );

    println!("\nWearable OS subsystems showcase executed and rendered successfully!");
}
