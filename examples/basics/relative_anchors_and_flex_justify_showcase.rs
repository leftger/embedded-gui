//! Showcase: Relative Anchor Positioning & Flex Justification
//!
//! Demonstrates:
//! 1. Relative alignment (`Rect::align_to`) and compound 2D presets (`Rect::anchor_to`).
//! 2. Positioning badges, tooltips, and dropdowns relative to reference elements without manual coordinate arithmetic.
//! 3. Primary axis space distribution in `LinearLayout` (`JustifyContent::SpaceBetween`, `SpaceAround`, `SpaceEvenly`).
//!
//! ### Interactive Controls (when desktop window is available):
//! - **Space**: Cycle through JustifyContent distribution modes
//! - **Arrows**: Move reference card around the screen
//! - **Esc / Q**: Exit

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
    geometry::{Anchor, HorizontalAlign, Rect, VerticalAlign},
    layout::{JustifyContent, LayoutItem, LinearLayout},
    render::RenderCtx,
    style::Border,
};

const W: u32 = 320;
const H: u32 = 240;

fn main() {
    println!("=== embedded-gui: Relative Anchors & Flex Justify Showcase ===");

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
    let mut window = Window::new("Relative Anchors & Flex Justification (320x240)", &settings);

    let mut card_offset_x = 0i32;
    let mut card_offset_y = 0i32;
    let mut justify_mode = 0usize;

    'running: loop {
        display.clear(Rgb565::new(1, 2, 3)).unwrap();

        let screen = Rect::new(0, 0, W, H);
        let mut ctx = RenderCtx::new(&mut display, screen);

        render_anchors_and_flex(&mut ctx, card_offset_x, card_offset_y, justify_mode);

        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape | Keycode::Q => break 'running,
                    Keycode::Space => {
                        justify_mode = (justify_mode + 1) % 3;
                    }
                    Keycode::Left => card_offset_x -= 8,
                    Keycode::Right => card_offset_x += 8,
                    Keycode::Up => card_offset_y -= 8,
                    Keycode::Down => card_offset_y += 8,
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
    let mut fb = Framebuffer::<{ 320 * 240 }>::new(W, H);
    let mut ctx = RenderCtx::new(&mut fb, screen);

    render_anchors_and_flex(&mut ctx, 0, 0, 0);

    let card = Rect::new(0, 0, 160, 80).anchor_to(Rect::new(0, 20, W, 120), Anchor::Center);
    let badge = Rect::new(0, 0, 32, 16).anchor_to(card, Anchor::TopRight);
    let dropdown =
        Rect::new(0, 0, 160, 24).align_to(card, HorizontalAlign::Left, VerticalAlign::TopToBottom);
    let side_btn =
        Rect::new(0, 0, 28, 36).align_to(card, HorizontalAlign::LeftToRight, VerticalAlign::Center);

    println!("Card centered on screen:      {:?}", card);
    println!("Badge anchored at TopRight:   {:?}", badge);
    println!("Dropdown aligned at Bottom:   {:?}", dropdown);
    println!("Side button aligned at Right: {:?}", side_btn);

    println!("\nRelative anchors and flex justification rendering completed successfully!");
}

fn render_anchors_and_flex<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    card_offset_x: i32,
    card_offset_y: i32,
    justify_mode: usize,
) where
    D: DrawTarget<Color = Rgb565>,
    C: embedded_gui::render::Compositor<D>,
{
    // Header Title
    let _ = ctx.fill_rect(Rect::new(0, 0, W, 20), Rgb565::new(3, 6, 9));
    let _ = ctx.draw_text(10, 5, "RELATIVE ANCHORS & FLEX ENGINE", Rgb565::CSS_CYAN);

    // Reference Card
    let base_card = Rect::new(0, 0, 160, 80).anchor_to(Rect::new(0, 20, W, 120), Anchor::Center);
    let card = Rect::new(
        base_card.x + card_offset_x,
        base_card.y + card_offset_y,
        base_card.w,
        base_card.h,
    );

    let _ = ctx.fill_rounded_rect(card, 6, Rgb565::new(6, 12, 18));
    let _ = ctx.stroke_rounded_rect(card, 6, Border::one(Rgb565::CSS_DARK_CYAN));
    let _ = ctx.draw_text(
        card.x + 12,
        card.y + 12,
        "REFERENCE CARD",
        Rgb565::CSS_WHITE,
    );
    let _ = ctx.draw_text(
        card.x + 12,
        card.y + 30,
        "Relative anchors attach",
        Rgb565::new(15, 30, 20),
    );
    let _ = ctx.draw_text(
        card.x + 12,
        card.y + 44,
        "without coordinate math",
        Rgb565::new(15, 30, 20),
    );

    // Top-Right Badge attached to Card
    let badge_size = Rect::new(0, 0, 32, 16);
    let badge = badge_size.anchor_to(card, Anchor::TopRight);
    let _ = ctx.fill_rounded_rect(badge, 3, Rgb565::new(31, 0, 0));
    let _ = ctx.draw_text(badge.x + 4, badge.y + 3, "LIVE", Rgb565::CSS_WHITE);

    // Dropdown attached Outside-Bottom of Card
    let dropdown_size = Rect::new(0, 0, 160, 24);
    let dropdown = dropdown_size.align_to(card, HorizontalAlign::Left, VerticalAlign::TopToBottom);
    let _ = ctx.fill_rounded_rect(dropdown, 4, Rgb565::new(3, 6, 12));
    let _ = ctx.stroke_rounded_rect(dropdown, 4, Border::one(Rgb565::new(10, 20, 30)));
    let _ = ctx.draw_text(
        dropdown.x + 8,
        dropdown.y + 6,
        "Attached Dropdown v",
        Rgb565::CSS_GOLD,
    );

    // Side Button attached Outside-Right of Card
    let side_btn_size = Rect::new(0, 0, 28, 36);
    let side_btn =
        side_btn_size.align_to(card, HorizontalAlign::LeftToRight, VerticalAlign::Center);
    let _ = ctx.fill_rounded_rect(side_btn, 4, Rgb565::new(0, 35, 20));
    let _ = ctx.draw_text(side_btn.x + 8, side_btn.y + 12, "+", Rgb565::CSS_WHITE);

    // LinearLayout Flex Spacing Distribution
    let justify = match justify_mode {
        0 => JustifyContent::SpaceBetween,
        1 => JustifyContent::SpaceAround,
        _ => JustifyContent::SpaceEvenly,
    };
    let mode_name = match justify_mode {
        0 => "JustifyContent::SpaceBetween",
        1 => "JustifyContent::SpaceAround",
        _ => "JustifyContent::SpaceEvenly",
    };

    let flex_container = Rect::new(10, 165, 300, 65);
    let _ = ctx.fill_rounded_rect(flex_container, 4, Rgb565::new(4, 8, 12));
    let _ = ctx.draw_text(
        flex_container.x + 8,
        flex_container.y + 6,
        mode_name,
        Rgb565::CSS_ORANGE,
    );

    let items = [
        LayoutItem::fixed(50),
        LayoutItem::fixed(50),
        LayoutItem::fixed(50),
        LayoutItem::fixed(50),
    ];
    let mut slots = [Rect::empty(); 4];

    let layout = LinearLayout::row().with_gap(0).with_justify(justify);
    let inner_flex = Rect::new(
        flex_container.x + 6,
        flex_container.y + 24,
        flex_container.w - 12,
        32,
    );
    layout.arrange_items(inner_flex, &items, &mut slots);

    for (i, slot) in slots.iter().enumerate() {
        let colors = [
            Rgb565::new(20, 5, 5),
            Rgb565::new(5, 20, 5),
            Rgb565::new(5, 5, 20),
            Rgb565::new(15, 15, 5),
        ];
        let _ = ctx.fill_rounded_rect(*slot, 3, colors[i % 4]);
        let _ = ctx.draw_text(
            slot.x + 12,
            slot.y + 10,
            &format!("Box {}", i),
            Rgb565::CSS_WHITE,
        );
    }
}
