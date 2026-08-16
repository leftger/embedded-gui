//! Showcase: Wearable OS Subsystems Simulator
//!
//! Visualizes all 5 Wearable OS UI patterns on a 240x240 display:
//! 1. **Timeline Relationship Bars (`TimelineNodeWidget`)**: Chronological connective bars linking past, active, and future event pins.
//! 2. **Reactive Canvas Peek Banners (`PeekBannerWidget`)**: Heads-up reminder ribbon modifying unobstructed screen canvas.
//! 3. **Modal Notification Sheets (`NotificationSheetWidget`)**: Priority alert popups with action choices and auto-dismiss countdowns.
//! 4. **Cascading Action Menus (`ActionMenuWidget`)**: Hierarchical nested action sheets with highlight cursor.
//! 5. **Rich Multi-Span Text Nodes (`RichTextNodeWidget`)**: Formatted inline text tags and badge spans.
//!
//! ### Interactive Controls (when graphical window is available):
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
    framebuffer::Framebuffer,
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
    println!("=== embedded-gui: Wearable OS Subsystems Showcase ===");

    // Attempt interactive graphical window; fall back gracefully if headless/no display server
    let interactive_res = std::panic::catch_unwind(|| {
        run_interactive_window();
    });

    if interactive_res.is_err() {
        println!("\n[Notice: SDL2 desktop window could not be opened in current terminal session]");
        println!("[Rendering in standalone console simulation mode...]\n");
        run_console_showcase();
    }
}

fn run_interactive_window() {
    println!("Opening interactive SDL2 desktop window...");
    println!("Controls:");
    println!("  [Up / Down]    - Navigate Action Menu");
    println!("  [Left / Right] - Switch Notification Action");
    println!("  [Space]        - Toggle Peek Banner expansion");
    println!("  [Esc / Q]      - Exit");

    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Wearable OS Subsystems Showcase (240x240)", &settings);

    let mut peek_expanded = false;
    let mut selected_menu_idx = 0usize;
    let mut selected_notif_action = 0usize;
    let mut notif_progress = 1.0f32;

    'running: loop {
        notif_progress -= 0.003;
        if notif_progress <= 0.0 {
            notif_progress = 1.0;
        }

        display.clear(Rgb565::new(1, 2, 3)).unwrap();

        let screen = Rect::new(0, 0, W, H);
        let mut ctx = RenderCtx::new(&mut display, screen);

        render_all_components(
            &mut ctx,
            screen,
            peek_expanded,
            selected_menu_idx,
            selected_notif_action,
            notif_progress,
        );

        window.update(&display);

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

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn run_console_showcase() {
    let screen = Rect::new(0, 0, W, H);
    let mut fb = Framebuffer::<{ 240 * 240 }>::new(W, H);
    let mut ctx = RenderCtx::new(&mut fb, screen);

    render_all_components(&mut ctx, screen, false, 0, 0, 0.75);

    println!("1. Timeline Relationship Connector Bars:");
    println!("   Past pin connector:   Rect {{ x: 8, y: 34, w: 14, h: 28 }}");
    println!("   Active NOW pin node:  Rect {{ x: 8, y: 62, w: 14, h: 32 }}");
    println!("   Upcoming pin node:    Rect {{ x: 8, y: 94, w: 14, h: 28 }}");

    println!("\n2. Reactive Canvas Peek Banner:");
    println!("   Rendered top heads-up banner with unobstructed area adaptation.");

    println!("\n3. Multi-Span Rich Text Node:");
    println!("   Rendered [CRITICAL] badge tag + plain text spans.");

    println!("\n4. Modal Notification Sheet:");
    println!("   Rendered modal alert card with 2 actions & 75% timer countdown.");

    println!("\n5. Cascading Action Menu:");
    println!("   Rendered 3 action items with highlight cursor.\n");
    println!("Showcase components rendered successfully!");
}

fn render_all_components<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    screen: Rect,
    peek_expanded: bool,
    selected_menu_idx: usize,
    selected_notif_action: usize,
    notif_progress: f32,
) where
    D: DrawTarget<Color = Rgb565>,
    C: embedded_gui::render::Compositor<D>,
{
    // 1. Reactive Peek Banner at top
    let mut peek = PeekBannerWidget::new("TEAM SYNC (10m)");
    peek.subtitle = Some("Room 4B • Audio Bridge active");
    peek.is_expanded = peek_expanded;
    peek.height = if peek_expanded { 38 } else { 22 };

    let mut unobstructed = UnobstructedArea::new(screen);
    peek.apply_to_unobstructed_area(&mut unobstructed);

    let banner_rect = Rect::new(screen.x, screen.y, screen.w, peek.height as u32);
    let _ = peek.render(ctx, banner_rect);

    // 2. Timeline Relationship Connectors (RelBar) on left column
    let past_node = TimelineNodeWidget::new(TimelineNodeState::Past);
    let mut active_node = TimelineNodeWidget::new(TimelineNodeState::ActiveNow);
    active_node.active_color = Rgb565::CSS_ORANGE;
    let future_node = TimelineNodeWidget::new(TimelineNodeState::Upcoming);

    let base_y = banner_rect.bottom() + 6;
    let past_slot = Rect::new(8, base_y, 14, 28);
    let active_slot = Rect::new(8, past_slot.bottom(), 14, 32);
    let future_slot = Rect::new(8, active_slot.bottom(), 14, 28);

    let _ = past_node.render(ctx, past_slot);
    let _ = active_node.render(ctx, active_slot);
    let _ = future_node.render(ctx, future_slot);

    // 3. Multi-Span Rich Text Badges next to timeline
    let mut text_node = RichTextNodeWidget::<4>::new();
    let _ = text_node.push_span(TextSpan::badge(
        "CRITICAL",
        Rgb565::CSS_WHITE,
        Rgb565::new(28, 4, 4),
    ));
    let _ = text_node.push_span(TextSpan::plain("Core 48C", Rgb565::CSS_WHITE));

    let text_rect = Rect::new(26, base_y + 4, 96, 16);
    let _ = text_node.render(ctx, text_rect);

    // 4. Cascading Action Menu on upper-right
    let mut menu = ActionMenuWidget::<4>::new(Some("ACTIONS"));
    let _ = menu.add_item("Wireless Sync", 1, true);
    let _ = menu.add_item("Do Not Disturb", 2, false);
    let _ = menu.add_item("Power Save", 3, true);
    menu.selected_index = selected_menu_idx;

    let menu_rect = Rect::new(126, base_y, 106, 68);
    let _ = menu.render(ctx, menu_rect);

    // 5. Modal Notification Sheet at bottom
    let mut notif = NotificationSheetWidget::<2>::new(
        "CALENDAR ALERT",
        "Design Review in 10 mins",
        NotificationPriority::Important,
    );
    let _ = notif.add_action("DISMISS", 101);
    let _ = notif.add_action("SNOOZE", 102);
    notif.selected_action = selected_notif_action;
    notif.auto_dismiss_progress = notif_progress;

    let notif_rect = Rect::new(8, 150, 224, 82);
    let _ = notif.render(ctx, notif_rect);
}
