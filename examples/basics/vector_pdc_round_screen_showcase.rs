//! Showcase: Vector Graphics, Circular Screen Geometry & Wearable Controls
//!
//! Demonstrates:
//! 1. Subpixel vector paths and draw commands via `PdcImage` / `PdcCommand`.
//! 2. Round / circular screen geometry calculations (`circle_chord_width`, `round_screen_line_bounds`).
//! 3. Reactive unobstructed area tracking (`UnobstructedArea`).
//! 4. Compact wearable widgets: `ContentIndicatorWidget`, `CrumbsIndicatorWidget`, `SelectionWidget`, `ActionBarWidget`.

use embedded_graphics_core::{
    geometry::Point,
    pixelcolor::{Rgb565, WebColors},
};
use embedded_gui::{
    framebuffer::Framebuffer,
    geometry::Rect,
    pdc::{PdcCommand, PdcCommandType, PdcImage, PdcPrecisePoint},
    render::RenderCtx,
    round::{UnobstructedArea, circle_chord_width, round_screen_line_bounds},
    widgets::{
        ActionBarWidget, ContentIndicatorDirection, ContentIndicatorWidget, CrumbsIndicatorWidget,
        SelectionWidget,
    },
};

fn main() {
    println!("=== embedded-gui: Vector Graphics & Round Screen Showcase ===");

    // 1. Vector Command (PDC) Construction
    let mut icon = PdcImage::<4, 8>::new(Rect::new(0, 0, 48, 48));

    // Outer circle
    let circle_cmd = PdcCommand::circle(
        Point::new(24, 24),
        20,
        Some(Rgb565::CSS_CYAN),
        Some(Rgb565::new(2, 4, 8)),
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

    println!(
        "Created vector asset with {} commands.",
        icon.commands.len()
    );

    // 2. Circular / Round Screen Geometry
    let diameter = 240u32;
    let radius = diameter / 2;
    println!("\nCircular screen chord widths (240px display):");
    for y_offset in [-100, -60, 0, 60, 100] {
        let chord = circle_chord_width(radius, y_offset);
        let safe_line = round_screen_line_bounds(diameter, 120 + y_offset, 16);
        println!(
            "  Offset y={:+4}px -> chord width = {}px (Safe text bounds: [x={}, y={}, w={}, h={}])",
            y_offset, chord, safe_line.x, safe_line.y, safe_line.w, safe_line.h
        );
    }

    // 3. Unobstructed Area Tracking
    let screen = Rect::new(0, 0, 240, 240);
    let mut unobstructed = UnobstructedArea::new(screen);
    println!(
        "\nUnobstructed screen bounds: {:?}",
        unobstructed.visible_rect()
    );

    // System banner peeks down from top
    unobstructed.set_insets(32, 0, 0, 0);
    println!(
        "After 32px top status bar overlay: {:?}",
        unobstructed.visible_rect()
    );

    // 4. Render into Framebuffer
    let mut fb = Framebuffer::<{ 240 * 240 }>::new(240, 240);
    let mut ctx = RenderCtx::new(&mut fb, screen);

    // Render Vector Icon
    icon.draw(&mut ctx, Point::new(96, 40)).unwrap();

    // Render Content Scroll Indicators
    let up_hint = ContentIndicatorWidget::new(ContentIndicatorDirection::Up);
    let down_hint = ContentIndicatorWidget::new(ContentIndicatorDirection::Down);
    up_hint
        .render(&mut ctx, Rect::new(110, 10, 20, 12))
        .unwrap();
    down_hint
        .render(&mut ctx, Rect::new(110, 218, 20, 12))
        .unwrap();

    // Render Crumbs Indicator
    let crumbs = CrumbsIndicatorWidget::new(5, 2);
    crumbs
        .render(&mut ctx, Rect::new(70, 105, 100, 16))
        .unwrap();

    // Render Segmented PIN / Number Selector
    let selection = SelectionWidget::<4>::new(["1", "2", "3", "4"], 4);
    selection
        .render(&mut ctx, Rect::new(60, 135, 120, 32))
        .unwrap();

    // Render Contextual Action Bar
    let mut action_bar = ActionBarWidget::new();
    action_bar.up_label = Some("+");
    action_bar.select_label = Some("OK");
    action_bar.down_label = Some("-");
    action_bar
        .render(&mut ctx, Rect::new(210, 70, 26, 100))
        .unwrap();

    println!("\nVector and wearable UI showcase executed and rendered successfully!");
}
