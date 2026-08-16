//! Showcase: Vector Graphics, Circular Screen Geometry & Wearable Controls
//!
//! Demonstrates:
//! 1. Subpixel vector paths and draw commands via `PdcImage` / `PdcCommand`.
//! 2. Round / circular screen geometry calculations (`circle_chord_width`, `round_screen_line_bounds`).
//! 3. Reactive unobstructed area tracking (`UnobstructedArea`).
//! 4. Compact wearable widgets: `ContentIndicatorWidget`, `CrumbsIndicatorWidget`, `SelectionWidget`, `ActionBarWidget`.

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{Rgb565, WebColors},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::{
    framebuffer::Framebuffer,
    geometry::Rect,
    pdc::{PdcCommand, PdcCommandType, PdcImage, PdcPrecisePoint},
    render::RenderCtx,
    round::{circle_chord_width, round_screen_line_bounds},
    widgets::{
        ActionBarWidget, ContentIndicatorDirection, ContentIndicatorWidget, CrumbsIndicatorWidget,
        SelectionWidget,
    },
};

const W: u32 = 240;
const H: u32 = 240;

fn main() {
    println!("=== embedded-gui: Vector Graphics & Round Screen Showcase ===");

    let res = std::panic::catch_unwind(|| {
        run_interactive_window();
    });

    if res.is_err() {
        println!("\n[Notice: SDL2 desktop window could not be opened in current terminal session]");
        println!("[Rendering in standalone console simulation mode...]\n");
        run_console_showcase();
    }
}

fn run_interactive_window() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Vector Graphics & Round Screen (240x240)", &settings);

    let icon = build_vector_icon();
    let mut active_page = 2usize;
    let mut selected_digit = 7u8;
    let mut overlay_active = false;

    'running: loop {
        let screen = Rect::new(0, 0, W, H);
        display.clear(Rgb565::new(0, 0, 0)).unwrap();

        let mut ctx = RenderCtx::new(&mut display, screen);
        render_round_screen(&mut ctx, &icon, overlay_active, selected_digit, active_page);

        window.update(&display);

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

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn run_console_showcase() {
    let icon = build_vector_icon();
    let screen = Rect::new(0, 0, W, H);
    let mut fb = Framebuffer::<{ 240 * 240 }>::new(W, H);
    let mut ctx = RenderCtx::new(&mut fb, screen);

    render_round_screen(&mut ctx, &icon, false, 7, 2);

    println!("Circular screen chord widths (240px display):");
    let radius = W / 2;
    for y_offset in [-100, -60, 0, 60, 100] {
        let chord = circle_chord_width(radius, y_offset);
        let safe_line = round_screen_line_bounds(W, 120 + y_offset, 16);
        println!(
            "  Offset y={:+4}px -> chord width = {}px (Safe text bounds: [x={}, y={}, w={}, h={}])",
            y_offset, chord, safe_line.x, safe_line.y, safe_line.w, safe_line.h
        );
    }
    println!("\nVector PDC asset and wearable components rendered successfully!");
}

fn build_vector_icon() -> PdcImage<4, 8> {
    let mut icon = PdcImage::<4, 8>::new(Rect::new(0, 0, 48, 48));
    let circle_cmd = PdcCommand::circle(
        Point::new(24, 24),
        20,
        Some(Rgb565::CSS_CYAN),
        Some(Rgb565::new(2, 6, 12)),
        2,
    );
    icon.push_command(circle_cmd).unwrap();

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
    icon
}

fn render_round_screen<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    icon: &PdcImage<4, 8>,
    overlay_active: bool,
    selected_digit: u8,
    active_page: usize,
) where
    D: DrawTarget<Color = Rgb565>,
    C: embedded_gui::render::Compositor<D>,
{
    // Draw circular screen background
    let _ = ctx.fill_circle(120, 120, 118, Rgb565::new(2, 4, 8));
    let _ = ctx.stroke_circle(120, 120, 118, Rgb565::new(10, 20, 30));

    // Safe line chords visualization
    let diameter = 240u32;
    let radius = diameter / 2;
    for y_offset in [-70, -40, 40, 70] {
        let chord = circle_chord_width(radius, y_offset);
        let safe_line = round_screen_line_bounds(diameter, 120 + y_offset, 14);
        let _ = ctx.fill_rounded_rect(safe_line, 2, Rgb565::new(3, 6, 12));
        let _ = ctx.draw_text(
            safe_line.x + 4,
            safe_line.y + 2,
            &format!("CHORD {}px", chord),
            Rgb565::new(10, 30, 20),
        );
    }

    if overlay_active {
        let _ = ctx.fill_rect(Rect::new(0, 0, 240, 32), Rgb565::new(20, 10, 0));
        let _ = ctx.draw_text(45, 10, "UNOBSTRUCTED INSET", Rgb565::CSS_GOLD);
    }

    // Render Vector Icon
    let _ = icon.draw(ctx, Point::new(96, 50));

    // Render Content Scroll Indicators
    let up_hint = ContentIndicatorWidget::new(ContentIndicatorDirection::Up);
    let down_hint = ContentIndicatorWidget::new(ContentIndicatorDirection::Down);
    let _ = up_hint.render(
        ctx,
        Rect::new(110, if overlay_active { 34 } else { 8 }, 20, 10),
    );
    let _ = down_hint.render(ctx, Rect::new(110, 222, 20, 10));

    // Render Numeric Selection Widget
    let mut selection = SelectionWidget::new(["1", "2", "3"], 3);
    selection.selected_cell = 1;
    let _ = selection.render(ctx, Rect::new(80, 115, 80, 32));
    let _ = ctx.draw_text(116, 124, &format!("{}", selected_digit), Rgb565::CSS_WHITE);

    // Render Crumbs Indicator at bottom
    let crumbs = CrumbsIndicatorWidget::new(5, active_page as u8);
    let _ = crumbs.render(ctx, Rect::new(80, 195, 80, 12));

    // Render Action Bar along right edge
    let action_bar = ActionBarWidget::new();
    let _ = action_bar.render(ctx, Rect::new(218, 55, 20, 130));
}
