//! Showcase: Interactive Vector Graphics, Circular Screen Geometry & Wearable Controls
//!
//! Demonstrates:
//! 1. Subpixel vector paths and draw commands via `PdcImage` / `PdcCommand`.
//! 2. Round / circular screen geometry calculations (`circle_chord_width`, `round_screen_line_bounds`).
//! 3. Reactive unobstructed area tracking (`UnobstructedArea`).
//! 4. Compact wearable widgets: `ContentIndicatorWidget`, `CrumbsIndicatorWidget`, `SelectionWidget`, `ActionBarWidget`.
//!
//! ### Controls:
//! - **Left / Right**: Change active crumb page index
//! - **Up / Down**: Select active digit in SelectionWidget
//! - **Space**: Toggle Unobstructed Area overlay
//! - **Esc / Q**: Exit

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{Rgb565, WebColors},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::{
    geometry::Rect,
    pdc::{PdcCommand, PdcCommandType, PdcImage, PdcPrecisePoint},
    render::RenderCtx,
    round::{UnobstructedArea, circle_chord_width, round_screen_line_bounds},
    widgets::{
        ActionBarWidget, ContentIndicatorDirection, ContentIndicatorWidget, CrumbsIndicatorWidget,
        SelectionWidget,
    },
};

const W: u32 = 240;
const H: u32 = 240;

fn main() {
    println!("=== embedded-gui: Interactive Vector Graphics & Round Screen Showcase ===");
    println!("Controls:");
    println!("  [Left / Right] - Change Crumb Page");
    println!("  [Up / Down]    - Cycle Digit Value in Selection Widget");
    println!("  [Space]        - Toggle Overlay Inset");
    println!("  [Esc / Q]      - Exit");

    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Vector Graphics & Round Screen (240x240)", &settings);

    // 1. Vector Command (PDC) Construction
    let mut icon = PdcImage::<4, 8>::new(Rect::new(0, 0, 48, 48));

    // Outer circle
    let circle_cmd = PdcCommand::circle(
        Point::new(24, 24),
        20,
        Some(Rgb565::CSS_CYAN),
        Some(Rgb565::new(2, 6, 12)),
        2,
    );
    icon.push_command(circle_cmd).unwrap();

    // Checkmark subpixel vector path (13.3 fixed-point coordinates)
    let mut check_cmd = PdcCommand::new(PdcCommandType::PrecisePath);
    check_cmd.stroke_color = Some(Rgb565::CSS_WHITE);
    check_cmd.stroke_width = 2;
    check_cmd
        .points
        .push(PdcPrecisePoint::from_pixels(14, 24))
        .unwrap();
    check_cmd
        .points
        .push(PdcPrecisePoint::from_pixels(22, 32))
        .unwrap();
    check_cmd
        .points
        .push(PdcPrecisePoint::from_pixels(34, 16))
        .unwrap();
    icon.push_command(check_cmd).unwrap();

    let mut active_page = 2usize;
    let mut selected_digit = 7u8;
    let mut overlay_active = false;

    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape | Keycode::Q => break 'running,
                    Keycode::Left => {
                        active_page = active_page.saturating_sub(1);
                    }
                    Keycode::Right => {
                        active_page = (active_page + 1).min(4);
                    }
                    Keycode::Up => {
                        selected_digit = (selected_digit + 1) % 10;
                    }
                    Keycode::Down => {
                        selected_digit = if selected_digit == 0 {
                            9
                        } else {
                            selected_digit - 1
                        };
                    }
                    Keycode::Space => {
                        overlay_active = !overlay_active;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let screen = Rect::new(0, 0, W, H);
        let mut unobstructed = UnobstructedArea::new(screen);
        if overlay_active {
            unobstructed.set_insets(32, 0, 0, 0);
        }

        // Clear display with dark circular bezel
        display.clear(Rgb565::new(0, 0, 0)).unwrap();

        let mut ctx = RenderCtx::new(&mut display, screen);

        // Draw circular screen background
        ctx.fill_circle(120, 120, 118, Rgb565::new(2, 4, 8))
            .unwrap();
        ctx.stroke_circle(120, 120, 118, Rgb565::new(10, 20, 30))
            .unwrap();

        // Draw safe line chords visualization
        let diameter = 240u32;
        let radius = diameter / 2;
        for y_offset in [-70, -40, 40, 70] {
            let chord = circle_chord_width(radius, y_offset);
            let safe_line = round_screen_line_bounds(diameter, 120 + y_offset, 14);
            ctx.fill_rounded_rect(safe_line, 2, Rgb565::new(3, 6, 12))
                .unwrap();
            ctx.draw_text(
                safe_line.x + 4,
                safe_line.y + 2,
                &format!("CHORD {}px", chord),
                Rgb565::new(10, 30, 20),
            )
            .unwrap();
        }

        // If overlay active, draw top banner
        if overlay_active {
            ctx.fill_rect(Rect::new(0, 0, 240, 32), Rgb565::new(20, 10, 0))
                .unwrap();
            ctx.draw_text(45, 10, "UNOBSTRUCTED INSET", Rgb565::CSS_GOLD)
                .unwrap();
        }

        // Render Vector Icon (Center)
        icon.draw(&mut ctx, Point::new(96, 50)).unwrap();

        // Render Content Scroll Indicators
        let up_hint = ContentIndicatorWidget::new(ContentIndicatorDirection::Up);
        let down_hint = ContentIndicatorWidget::new(ContentIndicatorDirection::Down);
        up_hint
            .render(
                &mut ctx,
                Rect::new(110, if overlay_active { 34 } else { 8 }, 20, 10),
            )
            .unwrap();
        down_hint
            .render(&mut ctx, Rect::new(110, 222, 20, 10))
            .unwrap();

        // Render Numeric Selection Widget
        let mut selection = SelectionWidget::new(["1", "2", "3"], 3);
        selection.selected_cell = 1;
        selection
            .render(&mut ctx, Rect::new(80, 115, 80, 32))
            .unwrap();
        ctx.draw_text(116, 124, &format!("{}", selected_digit), Rgb565::CSS_WHITE)
            .unwrap();

        // Render Crumbs Indicator at bottom
        let crumbs = CrumbsIndicatorWidget::new(5, active_page as u8);
        crumbs.render(&mut ctx, Rect::new(80, 195, 80, 12)).unwrap();

        // Render Action Bar along the right edge
        let action_bar = ActionBarWidget::new();
        action_bar
            .render(&mut ctx, Rect::new(218, 55, 20, 130))
            .unwrap();

        window.update(&display);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
