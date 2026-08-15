//! Showcase: Interactive Wearable OS Subsystems Simulator
//!
//! Visualizes all 5 Wearable OS UI patterns on a 240x240 display:
//! 1. **Timeline Relationship Bars (`TimelineNodeWidget`)**: Chronological connective bars linking past, active, and future event pins.
//! 2. **Reactive Canvas Peek Banners (`PeekBannerWidget`)**: Heads-up reminder ribbon modifying unobstructed screen canvas.
//! 3. **Modal Notification Sheets (`NotificationSheetWidget`)**: Priority alert popups with action choices and auto-dismiss countdowns.
//! 4. **Cascading Action Menus (`ActionMenuWidget`)**: Hierarchical nested action sheets with highlight cursor.
//! 5. **Rich Multi-Span Text Nodes (`RichTextNodeWidget`)**: Formatted inline text tags and badge spans.
//!
//! ### Controls:
//! - **Up / Down Arrow**: Navigate Action Menu items
//! - **Left / Right Arrow**: Switch Notification action button
//! - **Space**: Toggle Peek Banner expansion
//! - **Esc / Q**: Exit simulator

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, WebColors},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::{
    geometry::Rect,
    render::RenderCtx,
    round::UnobstructedArea,
    widgets::{
        ActionMenuWidget, NotificationPriority, NotificationSheetWidget, PeekBannerWidget,
        RichTextNodeWidget, TextSpan, TimelineNodeState, TimelineNodeWidget,
    },
};

const W: u32 = 240;
const H: u32 = 240;

fn main() {
    println!("=== embedded-gui: Interactive Wearable OS Subsystems Showcase ===");
    println!("Controls:");
    println!("  [Up / Down]    - Navigate Action Menu");
    println!("  [Left / Right] - Switch Notification Action");
    println!("  [Space]        - Toggle Peek Banner expansion");
    println!("  [Esc / Q]      - Exit");

    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Wearable OS Subsystems Showcase (240x240)", &settings);

    // Subsystem States
    let mut peek_expanded = false;
    let mut selected_menu_idx = 0usize;
    let mut selected_notif_action = 0usize;
    let mut notif_progress = 1.0f32;

    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape | Keycode::Q => break 'running,
                    Keycode::Up => {
                        selected_menu_idx = selected_menu_idx.saturating_sub(1);
                    }
                    Keycode::Down => {
                        selected_menu_idx = (selected_menu_idx + 1).min(2);
                    }
                    Keycode::Left => {
                        selected_notif_action = selected_notif_action.saturating_sub(1);
                    }
                    Keycode::Right => {
                        selected_notif_action = (selected_notif_action + 1).min(1);
                    }
                    Keycode::Space => {
                        peek_expanded = !peek_expanded;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Animate notification auto-dismiss countdown
        notif_progress -= 0.003;
        if notif_progress <= 0.0 {
            notif_progress = 1.0;
        }

        // Clear display with dark slate background
        display.clear(Rgb565::new(1, 2, 3)).unwrap();

        let screen = Rect::new(0, 0, W, H);
        let mut ctx = RenderCtx::new(&mut display, screen);

        // 1. Reactive Peek Banner at top
        let mut peek = PeekBannerWidget::new("TEAM SYNC (10m)");
        peek.subtitle = Some("Room 4B • Audio Bridge active");
        peek.is_expanded = peek_expanded;
        peek.height = if peek_expanded { 38 } else { 22 };

        let mut unobstructed = UnobstructedArea::new(screen);
        peek.apply_to_unobstructed_area(&mut unobstructed);

        let banner_rect = Rect::new(screen.x, screen.y, screen.w, peek.height as u32);
        peek.render(&mut ctx, banner_rect).unwrap();

        // 2. Timeline Relationship Connectors (RelBar) on left column
        let past_node = TimelineNodeWidget::new(TimelineNodeState::Past);
        let mut active_node = TimelineNodeWidget::new(TimelineNodeState::ActiveNow);
        active_node.active_color = Rgb565::CSS_ORANGE;
        let future_node = TimelineNodeWidget::new(TimelineNodeState::Upcoming);

        let base_y = banner_rect.bottom() + 6;
        let past_slot = Rect::new(8, base_y, 14, 28);
        let active_slot = Rect::new(8, past_slot.bottom(), 14, 32);
        let future_slot = Rect::new(8, active_slot.bottom(), 14, 28);

        past_node.render(&mut ctx, past_slot).unwrap();
        active_node.render(&mut ctx, active_slot).unwrap();
        future_node.render(&mut ctx, future_slot).unwrap();

        // 3. Multi-Span Rich Text Badges next to timeline
        let mut text_node = RichTextNodeWidget::<4>::new();
        text_node
            .push_span(TextSpan::badge(
                "CRITICAL",
                Rgb565::CSS_WHITE,
                Rgb565::new(28, 4, 4),
            ))
            .unwrap();
        text_node
            .push_span(TextSpan::plain("Core 48C", Rgb565::CSS_WHITE))
            .unwrap();

        let text_rect = Rect::new(26, base_y + 4, 96, 16);
        text_node.render(&mut ctx, text_rect).unwrap();

        // 4. Cascading Action Menu on upper-right
        let mut menu = ActionMenuWidget::<4>::new(Some("ACTIONS"));
        menu.add_item("Wireless Sync", 1, true).unwrap();
        menu.add_item("Do Not Disturb", 2, false).unwrap();
        menu.add_item("Power Save", 3, true).unwrap();
        menu.selected_index = selected_menu_idx;

        let menu_rect = Rect::new(126, base_y, 106, 68);
        menu.render(&mut ctx, menu_rect).unwrap();

        // 5. Modal Notification Sheet at bottom
        let mut notif = NotificationSheetWidget::<2>::new(
            "CALENDAR ALERT",
            "Design Review in 10 mins",
            NotificationPriority::Important,
        );
        notif.add_action("DISMISS", 101).unwrap();
        notif.add_action("SNOOZE", 102).unwrap();
        notif.selected_action = selected_notif_action;
        notif.auto_dismiss_progress = notif_progress;

        let notif_rect = Rect::new(8, 150, 224, 82);
        notif.render(&mut ctx, notif_rect).unwrap();

        window.update(&display);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
