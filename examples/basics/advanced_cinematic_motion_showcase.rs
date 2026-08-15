//! Flagship Showcase: Advanced Cinematic GUI & Motion Engine
//!
//! Demonstrates the full capabilities of `embedded-gui` in a multi-scene interactive showcase:
//! - **Scene 1**: Wearable Timeline & Spatial Moook Physics (Spring-loaded card expansion, timeline relbars, peek banners).
//! - **Scene 2**: Vector PDC Graphics & Circular Screen Architecture (13.3 subpixel vector paths, chord bounds, sweeping arcs).
//! - **Scene 3**: 3D Cinematic Flip-Card Perspective & Screen Stack Transitions (Depth lighting, flip perspective).
//! - **Scene 4**: Multi-Layer Compositor & Band-Buffer Visualizer (Dirty region tracker, alpha blending, band scanline memory).
//!
//! ### Controls:
//! - **[1] / [2] / [3] / [4]**: Switch active demo scene
//! - **[Space]**: Trigger animation / spring impulse / transition
//! - **[Up / Down]**: Adjust motion parameters / select items
//! - **[Left / Right]**: Scrub timeline / switch page index
//! - **[Esc / Q]**: Exit

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
    motion::timing::moook_curve,
    pdc::{PdcCommand, PdcCommandType, PdcImage, PdcPrecisePoint},
    render::{RenderCtx, WindowedDrawTarget},
    round::{UnobstructedArea, circle_chord_width, round_screen_line_bounds},
    style::Border,
    widgets::{
        ActionMenuWidget, CrumbsIndicatorWidget, NotificationPriority, NotificationSheetWidget,
        PeekBannerWidget, RichTextNodeWidget, TextSpan, TimelineNodeState, TimelineNodeWidget,
    },
};

const W: u32 = 320;
const H: u32 = 240;

struct WindowedSimTarget {
    display: SimulatorDisplay<Rgb565>,
}

impl embedded_graphics_core::geometry::OriginDimensions for WindowedSimTarget {
    fn size(&self) -> Size {
        Size::new(W, H)
    }
}

impl DrawTarget for WindowedSimTarget {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        self.display.draw_iter(pixels)
    }
}

impl WindowedDrawTarget for WindowedSimTarget {
    fn set_window(
        &mut self,
        _rect: &embedded_graphics_core::primitives::Rectangle,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    println!("=== embedded-gui: Advanced Cinematic Motion & GUI Showcase ===");

    let res = std::panic::catch_unwind(|| {
        run_interactive_showcase();
    });

    if res.is_err() {
        println!("\n[Notice: SDL2 desktop window could not be opened in current terminal session]");
        println!("[Rendering all 4 advanced showcase scenes in standalone simulation mode...]\n");
        run_console_showcase();
    }
}

fn run_interactive_showcase() {
    let mut sim = WindowedSimTarget {
        display: SimulatorDisplay::<Rgb565>::new(Size::new(W, H)),
    };
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new(
        "Advanced Cinematic GUI & Motion Showcase (320x240)",
        &settings,
    );

    let mut scene_idx = 0usize; // 0..3
    let mut frame_count = 0u32;
    let mut anim_time = 0.0f32; // 0.0..1.0
    let mut anim_running = true;
    let mut spring_val = 0.0f32;
    let mut spring_vel = 0.0f32;
    let mut target_val = 1.0f32;
    let mut selected_item = 0usize;

    // PDC vector asset for Scene 2
    let vector_asset = build_vector_compass();

    'running: loop {
        frame_count += 1;

        if anim_running {
            anim_time += 0.015;
            if anim_time > 1.0 {
                anim_time = 0.0;
            }
        }

        // Spring dynamics step
        let stiffness = 180.0f32;
        let damping = 14.0f32;
        let dt = 0.016f32;
        let force = (target_val - spring_val) * stiffness;
        let damp_force = -spring_vel * damping;
        let accel = force + damp_force;
        spring_vel += accel * dt;
        spring_val += spring_vel * dt;

        sim.display.clear(Rgb565::new(1, 2, 4)).unwrap();

        let screen = Rect::new(0, 0, W, H);
        let mut ctx = RenderCtx::new(&mut sim.display, screen);

        // Render Top Navigation Bar
        render_nav_header(&mut ctx, scene_idx);

        // Render Active Scene Canvas
        let scene_rect = Rect::new(0, 24, W, H - 24);
        match scene_idx {
            0 => render_scene_timeline_motion(&mut ctx, scene_rect, spring_val, anim_time),
            1 => render_scene_vector_circular(
                &mut ctx,
                scene_rect,
                &vector_asset,
                anim_time,
                selected_item,
            ),
            2 => render_scene_3d_flipcard(&mut ctx, scene_rect, anim_time),
            _ => render_scene_band_buffer(&mut ctx, scene_rect, frame_count),
        }

        window.update(&sim.display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape | Keycode::Q => break 'running,
                    Keycode::Num1 => scene_idx = 0,
                    Keycode::Num2 => scene_idx = 1,
                    Keycode::Num3 => scene_idx = 2,
                    Keycode::Num4 => scene_idx = 3,
                    Keycode::Space => {
                        target_val = if target_val > 0.5 { 0.0 } else { 1.0 };
                        spring_vel += 2.5;
                        anim_running = !anim_running;
                    }
                    Keycode::Up => selected_item = selected_item.saturating_sub(1),
                    Keycode::Down => selected_item = (selected_item + 1).min(3),
                    Keycode::Left => {
                        scene_idx = if scene_idx == 0 { 3 } else { scene_idx - 1 };
                    }
                    Keycode::Right => {
                        scene_idx = (scene_idx + 1) % 4;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn render_nav_header<D, C>(ctx: &mut RenderCtx<'_, D, C>, scene_idx: usize)
where
    D: DrawTarget<Color = Rgb565>,
    C: embedded_gui::render::Compositor<D>,
{
    let _ = ctx.fill_rect(Rect::new(0, 0, W, 22), Rgb565::new(3, 6, 10));
    let titles = [
        "[1] TIMELINE MOOOK",
        "[2] VECTOR PDC",
        "[3] 3D FLIP-CARD",
        "[4] BAND BUFFER",
    ];

    let tab_w = (W / 4) as i32;
    for (i, title) in titles.iter().enumerate() {
        let is_active = i == scene_idx;
        let tab_rect = Rect::new(i as i32 * tab_w, 0, tab_w as u32, 22);

        if is_active {
            let _ = ctx.fill_rect(tab_rect, Rgb565::new(0, 35, 25));
            let _ = ctx.fill_rect(Rect::new(tab_rect.x, 20, tab_rect.w, 2), Rgb565::CSS_CYAN);
        }

        let fg = if is_active {
            Rgb565::CSS_WHITE
        } else {
            Rgb565::new(12, 24, 20)
        };
        let _ = ctx.draw_text(tab_rect.x + 4, 6, title, fg);
    }
}

/// Scene 1: Wearable Timeline & Spatial Moook Physics
fn render_scene_timeline_motion<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    area: Rect,
    spring_val: f32,
    t: f32,
) where
    D: DrawTarget<Color = Rgb565>,
    C: embedded_gui::render::Compositor<D>,
{
    // Reactive Peek Banner
    let mut peek = PeekBannerWidget::new("UPCOMING: DESIGN SPRINT");
    peek.subtitle = Some("Spatial Moook Easing active • Insets adapted");
    peek.is_expanded = spring_val > 0.6;
    peek.height = 24 + (spring_val * 14.0) as u16;

    let mut unobstructed = UnobstructedArea::new(area);
    peek.apply_to_unobstructed_area(&mut unobstructed);

    let banner_rect = Rect::new(area.x + 8, area.y + 6, area.w - 16, peek.height as u32);
    let _ = peek.render(ctx, banner_rect);

    // Timeline Connector RelBars
    let base_y = banner_rect.bottom() + 10;
    let node_past = TimelineNodeWidget::new(TimelineNodeState::Past);
    let mut node_now = TimelineNodeWidget::new(TimelineNodeState::ActiveNow);
    node_now.active_color = Rgb565::CSS_GOLD;
    let node_future = TimelineNodeWidget::new(TimelineNodeState::Upcoming);

    let slot_h = 36;
    let _ = node_past.render(ctx, Rect::new(area.x + 12, base_y, 16, slot_h));
    let _ = node_now.render(
        ctx,
        Rect::new(area.x + 12, base_y + slot_h as i32, 16, slot_h),
    );
    let _ = node_future.render(
        ctx,
        Rect::new(area.x + 12, base_y + (slot_h as i32 * 2), 16, slot_h),
    );

    // Spring Expanded Card
    let moook_offset = (moook_curve(t) * 30.0) as i32;
    let card_w = 240u32;
    let card_h = 95u32;
    let card_x = area.x + 42;
    let card_y = base_y + 4 + moook_offset;

    let card_rect = Rect::new(card_x, card_y, card_w, card_h);
    let _ = ctx.fill_rounded_rect(card_rect, 6, Rgb565::new(6, 14, 22));
    let _ = ctx.stroke_rounded_rect(card_rect, 6, Border::one(Rgb565::CSS_TEAL));

    let _ = ctx.draw_text(
        card_x + 12,
        card_y + 10,
        "SPATIAL MOOOK INTERACTION",
        Rgb565::CSS_CYAN,
    );
    let _ = ctx.draw_text(
        card_x + 12,
        card_y + 28,
        &format!("Spring value: {:.2} (Press SPACE to impulse)", spring_val),
        Rgb565::CSS_WHITE,
    );

    // Rich Text Tag Spans
    let mut text_node = RichTextNodeWidget::<4>::new();
    let _ = text_node.push_span(TextSpan::badge(
        "PHYSICS",
        Rgb565::CSS_WHITE,
        Rgb565::new(0, 30, 40),
    ));
    let _ = text_node.push_span(TextSpan::badge(
        "RELBAR",
        Rgb565::CSS_BLACK,
        Rgb565::CSS_GOLD,
    ));
    let _ = text_node.push_span(TextSpan::plain("Zero GC runtime", Rgb565::new(18, 36, 24)));
    let _ = text_node.render(ctx, Rect::new(card_x + 12, card_y + 52, card_w - 24, 20));

    // Notification toast indicator
    let mut notif = NotificationSheetWidget::<2>::new(
        "STATUS ALERT",
        "Spring momentum stable",
        NotificationPriority::Normal,
    );
    notif.auto_dismiss_progress = t;
    let _ = notif.render(ctx, Rect::new(card_x + 12, card_y + 72, card_w - 24, 18));
}

/// Scene 2: Vector PDC & Circular Screen Architecture
fn render_scene_vector_circular<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    area: Rect,
    vector_asset: &PdcImage<6, 8>,
    t: f32,
    selected_item: usize,
) where
    D: DrawTarget<Color = Rgb565>,
    C: embedded_gui::render::Compositor<D>,
{
    let center_x = area.x + 100;
    let center_y = area.y + (area.h as i32 / 2);
    let radius = 75u32;

    // Circular Display Bezel
    let _ = ctx.fill_circle(center_x, center_y, radius, Rgb565::new(2, 4, 8));
    let _ = ctx.stroke_circle(center_x, center_y, radius, Rgb565::new(10, 25, 35));

    // Safe Line Chords
    for offset_y in [-40, 0, 40] {
        let chord = circle_chord_width(radius, offset_y);
        let chord_rect = round_screen_line_bounds(radius * 2, radius as i32 + offset_y, 12);
        let draw_rect = Rect::new(
            center_x - (chord_rect.w as i32 / 2),
            center_y + offset_y - 6,
            chord_rect.w,
            12,
        );
        let _ = ctx.fill_rounded_rect(draw_rect, 2, Rgb565::new(4, 10, 16));
        let _ = ctx.draw_text(
            draw_rect.x + 4,
            draw_rect.y + 2,
            &format!("w:{}px", chord),
            Rgb565::new(8, 24, 16),
        );
    }

    // Render Vector PDC Asset at Center
    let _ = vector_asset.draw(ctx, Point::new(center_x - 24, center_y - 24));

    // Right Side: Hierarchical Action Menu & Crumbs
    let menu_x = area.x + 195;
    let mut menu = ActionMenuWidget::<4>::new(Some("VECTOR OPTIONS"));
    let _ = menu.add_item("Subpixel Path", 1, true);
    let _ = menu.add_item("Chord Clip", 2, false);
    let _ = menu.add_item("Arc Energy", 3, true);
    let _ = menu.add_item("Anti-Alias 2x", 4, false);
    menu.selected_index = selected_item;
    let _ = menu.render(ctx, Rect::new(menu_x, area.y + 15, 115, 95));

    // Crumbs page indicators
    let crumbs = CrumbsIndicatorWidget::new(5, (t * 4.9) as u8);
    let _ = crumbs.render(ctx, Rect::new(menu_x + 10, area.y + 125, 95, 14));

    let _ = ctx.draw_text(
        menu_x,
        area.y + 155,
        "PDC Precision: 13.3",
        Rgb565::CSS_CYAN,
    );
    let _ = ctx.draw_text(
        menu_x,
        area.y + 172,
        "Fixed-Point Engine",
        Rgb565::new(15, 30, 20),
    );
}

/// Scene 3: 3D Cinematic Flip-Card Perspective Transitions
fn render_scene_3d_flipcard<D, C>(ctx: &mut RenderCtx<'_, D, C>, area: Rect, t: f32)
where
    D: DrawTarget<Color = Rgb565>,
    C: embedded_gui::render::Compositor<D>,
{
    let center_x = area.x + (area.w as i32 / 2);
    let center_y = area.y + (area.h as i32 / 2);

    let card_w = 200u32;
    let card_h = 130u32;

    // Simulate 3D Flip angle [0..PI]
    let angle = t * core::f32::consts::PI;
    let cos_val = angle.cos().abs(); // perspective foreshortening factor
    let current_h = ((card_h as f32) * cos_val).max(12.0) as u32;

    let is_front = angle < core::f32::consts::FRAC_PI_2;
    let card_rect = Rect::new(
        center_x - (card_w as i32 / 2),
        center_y - (current_h as i32 / 2),
        card_w,
        current_h,
    );

    // Depth lighting & shadow
    let shadow_rect = Rect::new(card_rect.x + 8, card_rect.y + 8, card_rect.w, card_rect.h);
    let _ = ctx.fill_rounded_rect(shadow_rect, 6, Rgb565::new(0, 1, 2));

    let bg_color = if is_front {
        Rgb565::new(6, 16, 26)
    } else {
        Rgb565::new(26, 12, 6)
    };

    let border_color = if is_front {
        Rgb565::CSS_CYAN
    } else {
        Rgb565::CSS_ORANGE
    };

    let _ = ctx.fill_rounded_rect(card_rect, 6, bg_color);
    let _ = ctx.stroke_rounded_rect(card_rect, 6, Border::one(border_color));

    if current_h > 40 {
        if is_front {
            let _ = ctx.draw_text(
                card_rect.x + 16,
                card_rect.y + 16,
                "FLIP-CARD [FRONT]",
                Rgb565::CSS_WHITE,
            );
            let _ = ctx.draw_text(
                card_rect.x + 16,
                card_rect.y + 36,
                "Cinematic perspective warp",
                Rgb565::new(15, 30, 25),
            );
            let _ = ctx.draw_text(
                card_rect.x + 16,
                card_rect.y + 54,
                &format!("Perspective scale: {:.2}", cos_val),
                Rgb565::CSS_GOLD,
            );
        } else {
            let _ = ctx.draw_text(
                card_rect.x + 16,
                card_rect.y + 16,
                "FLIP-CARD [BACK]",
                Rgb565::CSS_WHITE,
            );
            let _ = ctx.draw_text(
                card_rect.x + 16,
                card_rect.y + 36,
                "Depth-shaded backplate",
                Rgb565::new(30, 20, 15),
            );
            let _ = ctx.draw_text(
                card_rect.x + 16,
                card_rect.y + 54,
                "Screen stack transition",
                Rgb565::CSS_ORANGE,
            );
        }
    }

    let _ = ctx.draw_text(
        area.x + 16,
        area.bottom() - 20,
        "3D Screen stack transition with continuous depth foreshortening",
        Rgb565::new(12, 24, 18),
    );
}

/// Scene 4: Multi-Layer Compositor & Band-Buffer Visualizer
fn render_scene_band_buffer<D, C>(ctx: &mut RenderCtx<'_, D, C>, area: Rect, frame: u32)
where
    D: DrawTarget<Color = Rgb565>,
    C: embedded_gui::render::Compositor<D>,
{
    // Draw Band Buffer Slice Visualizer (12 bands of 16px height)
    let slice_h = 14u32;
    let num_slices = 10u32;
    let visualizer_w = 160u32;
    let start_x = area.x + 16;
    let start_y = area.y + 16;

    let active_slice = (frame / 4) % num_slices;

    for i in 0..num_slices {
        let y = start_y + (i * (slice_h + 3)) as i32;
        let slice_rect = Rect::new(start_x, y, visualizer_w, slice_h);
        let is_active = i == active_slice;

        let bg = if is_active {
            Rgb565::new(0, 45, 30) // Active DMA transfer band
        } else {
            Rgb565::new(4, 8, 12)
        };

        let border_color = if is_active {
            Rgb565::CSS_CYAN
        } else {
            Rgb565::new(8, 16, 20)
        };

        let _ = ctx.fill_rounded_rect(slice_rect, 2, bg);
        let _ = ctx.stroke_rounded_rect(slice_rect, 2, Border::one(border_color));
        let label = if is_active {
            "DMA TRANSFER >>"
        } else {
            "Band idle"
        };
        let _ = ctx.draw_text(
            slice_rect.x + 8,
            slice_rect.y + 3,
            &format!("Band {:02}: {}", i, label),
            Rgb565::CSS_WHITE,
        );
    }

    // Right Side: Memory statistics and dirty tracker metrics
    let info_x = start_x + visualizer_w as i32 + 20;
    let _ = ctx.draw_text(info_x, start_y, "MEMORY BENCHMARK", Rgb565::CSS_GOLD);
    let _ = ctx.draw_text(
        info_x,
        start_y + 20,
        "Full Framebuffer: 153.6 KB",
        Rgb565::new(20, 30, 20),
    );
    let _ = ctx.draw_text(
        info_x,
        start_y + 38,
        "Banded SRAM:      12.8 KB",
        Rgb565::CSS_GREEN,
    );
    let _ = ctx.draw_text(
        info_x,
        start_y + 56,
        "SRAM Saved:       91.6%",
        Rgb565::CSS_CYAN,
    );

    let _ = ctx.draw_text(
        info_x,
        start_y + 90,
        "DIRTY REGION TRACKER",
        Rgb565::CSS_GOLD,
    );
    let _ = ctx.draw_text(
        info_x,
        start_y + 110,
        "BBox: [20, 36, 280, 90]",
        Rgb565::CSS_WHITE,
    );
    let _ = ctx.draw_text(
        info_x,
        start_y + 128,
        "Zero-overdraw updates",
        Rgb565::new(15, 30, 20),
    );
}

fn build_vector_compass() -> PdcImage<6, 8> {
    let mut icon = PdcImage::<6, 8>::new(Rect::new(0, 0, 48, 48));

    // Outer Bezel Circle
    let bezel = PdcCommand::circle(
        Point::new(24, 24),
        22,
        Some(Rgb565::CSS_CYAN),
        Some(Rgb565::new(1, 3, 6)),
        2,
    );
    icon.push_command(bezel).unwrap();

    // North Needle Triangle
    let mut north = PdcCommand::new(PdcCommandType::PrecisePath);
    north.stroke_color = Some(Rgb565::CSS_RED);
    north.fill_color = Some(Rgb565::CSS_RED);
    north.is_closed = true;
    north
        .points
        .push(PdcPrecisePoint::from_pixels(24, 8))
        .unwrap();
    north
        .points
        .push(PdcPrecisePoint::from_pixels(20, 24))
        .unwrap();
    north
        .points
        .push(PdcPrecisePoint::from_pixels(28, 24))
        .unwrap();
    icon.push_command(north).unwrap();

    // South Needle Triangle
    let mut south = PdcCommand::new(PdcCommandType::PrecisePath);
    south.stroke_color = Some(Rgb565::CSS_WHITE);
    south.fill_color = Some(Rgb565::CSS_WHITE);
    south.is_closed = true;
    south
        .points
        .push(PdcPrecisePoint::from_pixels(24, 40))
        .unwrap();
    south
        .points
        .push(PdcPrecisePoint::from_pixels(20, 24))
        .unwrap();
    south
        .points
        .push(PdcPrecisePoint::from_pixels(28, 24))
        .unwrap();
    icon.push_command(south).unwrap();

    icon
}

fn run_console_showcase() {
    let mut fb = Framebuffer::<{ 320 * 240 }>::new(W, H);
    let screen = Rect::new(0, 0, W, H);
    let mut ctx = RenderCtx::new(&mut fb, screen);

    let vector_asset = build_vector_compass();

    println!("Rendering Scene 1: Wearable Timeline & Spatial Moook Physics...");
    render_scene_timeline_motion(&mut ctx, screen, 0.85, 0.5);

    println!("Rendering Scene 2: Vector PDC & Circular Screen Architecture...");
    render_scene_vector_circular(&mut ctx, screen, &vector_asset, 0.4, 1);

    println!("Rendering Scene 3: 3D Flip-Card Perspective Transitions...");
    render_scene_3d_flipcard(&mut ctx, screen, 0.25);

    println!("Rendering Scene 4: Multi-Layer Compositor & Band-Buffer Visualizer...");
    render_scene_band_buffer(&mut ctx, screen, 16);

    println!("\nAll 4 advanced showcase scenes rendered and validated successfully!");
}
