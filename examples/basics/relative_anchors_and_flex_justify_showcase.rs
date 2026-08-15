//! Showcase: Interactive Relative Anchor Positioning & Flex Justification
//!
//! Demonstrates:
//! 1. Relative alignment (`Rect::align_to`) and compound 2D presets (`Rect::anchor_to`).
//! 2. Positioning badges, tooltips, and dropdowns relative to reference elements without manual coordinate arithmetic.
//! 3. Primary axis space distribution in `LinearLayout` (`JustifyContent::SpaceBetween`, `SpaceAround`, `SpaceEvenly`).
//!
//! ### Controls:
//! - **Space**: Cycle through JustifyContent distribution modes
//! - **Up / Down / Left / Right**: Move reference card around the screen
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
    geometry::{Anchor, HorizontalAlign, Rect, VerticalAlign},
    layout::{JustifyContent, LayoutItem, LinearLayout},
    render::RenderCtx,
    style::Border,
};

const W: u32 = 320;
const H: u32 = 240;

fn main() {
    println!("=== embedded-gui: Interactive Relative Anchors & Flex Justify Showcase ===");
    println!("Controls:");
    println!("  [Space]     - Cycle Flex Justify mode (SpaceBetween / SpaceAround / SpaceEvenly)");
    println!("  [Arrows]    - Move Reference Card");
    println!("  [Esc / Q]   - Exit");

    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Relative Anchors & Flex Justification (320x240)", &settings);

    let mut card_offset_x = 0i32;
    let mut card_offset_y = 0i32;
    let mut justify_mode = 0usize; // 0: SpaceBetween, 1: SpaceAround, 2: SpaceEvenly

    'running: loop {
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

        display.clear(Rgb565::new(1, 2, 3)).unwrap();

        let screen = Rect::new(0, 0, W, H);
        let mut ctx = RenderCtx::new(&mut display, screen);

        // Header Title
        ctx.fill_rect(Rect::new(0, 0, W, 20), Rgb565::new(3, 6, 9))
            .unwrap();
        ctx.draw_text(10, 5, "RELATIVE ANCHORS & FLEX ENGINE", Rgb565::CSS_CYAN)
            .unwrap();

        // 1. Reference Card
        let base_card =
            Rect::new(0, 0, 160, 80).anchor_to(Rect::new(0, 20, W, 120), Anchor::Center);
        let card = Rect::new(
            base_card.x + card_offset_x,
            base_card.y + card_offset_y,
            base_card.w,
            base_card.h,
        );

        ctx.fill_rounded_rect(card, 6, Rgb565::new(6, 12, 18))
            .unwrap();
        ctx.stroke_rounded_rect(card, 6, Border::one(Rgb565::CSS_DARK_CYAN))
            .unwrap();
        ctx.draw_text(
            card.x + 12,
            card.y + 12,
            "REFERENCE CARD",
            Rgb565::CSS_WHITE,
        )
        .unwrap();
        ctx.draw_text(
            card.x + 12,
            card.y + 30,
            "Relative anchors attach",
            Rgb565::new(15, 30, 20),
        )
        .unwrap();
        ctx.draw_text(
            card.x + 12,
            card.y + 44,
            "without coordinate math",
            Rgb565::new(15, 30, 20),
        )
        .unwrap();

        // Top-Right Badge attached to Card
        let badge_size = Rect::new(0, 0, 32, 16);
        let badge = badge_size.anchor_to(card, Anchor::TopRight);
        ctx.fill_rounded_rect(badge, 3, Rgb565::new(31, 0, 0))
            .unwrap();
        ctx.draw_text(badge.x + 4, badge.y + 3, "LIVE", Rgb565::CSS_WHITE)
            .unwrap();

        // Dropdown attached Outside-Bottom of Card
        let dropdown_size = Rect::new(0, 0, 160, 24);
        let dropdown =
            dropdown_size.align_to(card, HorizontalAlign::Left, VerticalAlign::TopToBottom);
        ctx.fill_rounded_rect(dropdown, 4, Rgb565::new(3, 6, 12))
            .unwrap();
        ctx.stroke_rounded_rect(dropdown, 4, Border::one(Rgb565::new(10, 20, 30)))
            .unwrap();
        ctx.draw_text(
            dropdown.x + 8,
            dropdown.y + 6,
            "Attached Dropdown v",
            Rgb565::CSS_GOLD,
        )
        .unwrap();

        // Side Button attached Outside-Right of Card
        let side_btn_size = Rect::new(0, 0, 28, 36);
        let side_btn =
            side_btn_size.align_to(card, HorizontalAlign::LeftToRight, VerticalAlign::Center);
        ctx.fill_rounded_rect(side_btn, 4, Rgb565::new(0, 35, 20))
            .unwrap();
        ctx.draw_text(side_btn.x + 8, side_btn.y + 12, "+", Rgb565::CSS_WHITE)
            .unwrap();

        // 2. LinearLayout Flex Spacing Distribution
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
        ctx.fill_rounded_rect(flex_container, 4, Rgb565::new(4, 8, 12))
            .unwrap();
        ctx.draw_text(
            flex_container.x + 8,
            flex_container.y + 6,
            mode_name,
            Rgb565::CSS_ORANGE,
        )
        .unwrap();

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
            ctx.fill_rounded_rect(*slot, 3, colors[i % 4]).unwrap();
            ctx.draw_text(
                slot.x + 12,
                slot.y + 10,
                &format!("Box {}", i),
                Rgb565::CSS_WHITE,
            )
            .unwrap();
        }

        window.update(&display);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
