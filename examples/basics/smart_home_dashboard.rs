//! Showcase: Smart Home & Industrial IoT Telemetry Dashboard (320x240)
//!
//! Demonstrates:
//! 1. **Multi-Card Glassmorphic Dashboard Layout**
//! 2. **HVAC Radial Temperature & Fan Speed Gauge**
//! 3. **Smooth Color Temperature & Brightness Sliders**
//! 4. **Live Translucent Telemetry Sparkline & Grid**
//! 5. **Tactile Scene Quick-Action Buttons & Status Indicators**

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor, WebColors},
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::{
    EdgeInsets, Framebuffer, Rect, RenderCtx, ScaleWidget, StrokeStyle, Style, TextStyle,
};

const W: u32 = 320;
const H: u32 = 240;
const FB_SIZE: usize = (W * H) as usize;

fn render_dashboard<D: DrawTarget<Color = Rgb565> + embedded_gui::PixelRead>(
    target: &mut D,
    frame: u32,
) -> Result<(), D::Error> {
    // 1. Dark ambient background
    let bg_rect = Rectangle::new(Point::zero(), Size::new(W, H));
    target.fill_solid(&bg_rect, Rgb565::new(1, 2, 4))?;

    let viewport = Rect::new(0, 0, W, H);
    let mut ctx = RenderCtx::compositing(target, viewport);

    // 2. Top Header Bar
    let header_rect = Rect::new(8, 6, W - 16, 22);
    ctx.fill_rounded_rect(header_rect, 4, Rgb565::new(3, 7, 14))?;
    ctx.stroke_rounded_rect(
        header_rect,
        4,
        embedded_gui::Border::one(Rgb565::new(0, 25, 35)),
    )?;
    ctx.draw_text_in(
        header_rect.inset(EdgeInsets::symmetric(8, 4)),
        "SMART HOME HUB // LIVING ROOM",
        TextStyle::new(Rgb565::WHITE),
    )?;
    ctx.draw_text(
        header_rect.right() - 75,
        header_rect.y + 4,
        "22.5 C  WiFi",
        Rgb565::CSS_CYAN,
    )?;

    // 3. Card 1: Climate & HVAC (Left: 96px width)
    let card1 = Rect::new(8, 32, 96, 156);
    ctx.fill_rounded_rect(card1, 6, Rgb565::new(2, 4, 10))?;
    ctx.stroke_rounded_rect(card1, 6, embedded_gui::Border::one(Rgb565::new(0, 20, 30)))?;

    ctx.draw_text(card1.x + 8, card1.y + 6, "CLIMATE", Rgb565::CSS_ORANGE)?;

    let temp_val = 21.5 + ((frame as f32 * 0.05).sin() * 4.0);
    let temp_gauge = ScaleWidget::new(16.0, 28.0, temp_val)
        .with_ticks(4, 2)
        .with_angles(180, 0)
        .with_needle(true, Rgb565::CSS_ORANGE);
    temp_gauge.render(
        &mut ctx,
        Rect::new(card1.x + 6, card1.y + 22, card1.w - 12, 48),
        Style::panel().into(),
        embedded_gui::VisualState::Normal,
    )?;

    ctx.draw_text(card1.x + 18, card1.y + 76, "Target: 22 C", Rgb565::WHITE)?;
    ctx.draw_text(
        card1.x + 12,
        card1.y + 92,
        "AC Fan: Auto",
        Rgb565::CSS_LIGHT_GRAY,
    )?;

    // Mode Toggle Button
    let mode_btn = Rect::new(card1.x + 8, card1.y + 116, card1.w - 16, 26);
    ctx.fill_rounded_rect(mode_btn, 4, Rgb565::new(0, 20, 25))?;
    ctx.stroke_rounded_rect(
        mode_btn,
        4,
        embedded_gui::Border::one(Rgb565::new(0, 45, 30)),
    )?;
    ctx.draw_text_in(
        mode_btn.inset(EdgeInsets::all(4)),
        "ECO ACTIVE",
        TextStyle::new(Rgb565::CSS_SPRING_GREEN).with_align(embedded_gui::TextAlign::Center),
    )?;

    // 4. Card 2: Lighting & Ambience (Center: 96px width)
    let card2 = Rect::new(112, 32, 96, 156);
    ctx.fill_rounded_rect(card2, 6, Rgb565::new(2, 4, 10))?;
    ctx.stroke_rounded_rect(card2, 6, embedded_gui::Border::one(Rgb565::new(0, 20, 30)))?;

    ctx.draw_text(card2.x + 8, card2.y + 6, "LIGHTING", Rgb565::CSS_YELLOW)?;

    // Brightness Slider Bar
    ctx.draw_text(card2.x + 8, card2.y + 26, "Brightness", Rgb565::CSS_GRAY)?;
    let b_slider = Rect::new(card2.x + 8, card2.y + 42, card2.w - 16, 14);
    ctx.fill_rounded_rect(b_slider, 4, Rgb565::new(5, 10, 15))?;
    let b_pct = 0.5 + ((frame as f32 * 0.04).sin() * 0.3);
    let fill_w = ((b_slider.w as f32) * b_pct) as u32;
    if fill_w > 0 {
        ctx.fill_rounded_rect(
            Rect::new(b_slider.x, b_slider.y, fill_w, b_slider.h),
            4,
            Rgb565::CSS_YELLOW,
        )?;
    }

    // Color Temperature Slider
    ctx.draw_text(card2.x + 8, card2.y + 68, "Color Temp", Rgb565::CSS_GRAY)?;
    let c_slider = Rect::new(card2.x + 8, card2.y + 84, card2.w - 16, 14);
    ctx.fill_rounded_rect(c_slider, 4, Rgb565::new(10, 15, 25))?;
    let c_pct = 0.6 + ((frame as f32 * 0.03).cos() * 0.25);
    let c_fill = ((c_slider.w as f32) * c_pct) as u32;
    if c_fill > 0 {
        ctx.fill_rounded_rect(
            Rect::new(c_slider.x, c_slider.y, c_fill, c_slider.h),
            4,
            Rgb565::CSS_CYAN,
        )?;
    }

    let light_btn = Rect::new(card2.x + 8, card2.y + 116, card2.w - 16, 26);
    ctx.fill_rounded_rect(light_btn, 4, Rgb565::new(15, 20, 5))?;
    ctx.stroke_rounded_rect(
        light_btn,
        4,
        embedded_gui::Border::one(Rgb565::new(30, 40, 0)),
    )?;
    ctx.draw_text_in(
        light_btn.inset(EdgeInsets::all(4)),
        "ALL ON",
        TextStyle::new(Rgb565::CSS_YELLOW).with_align(embedded_gui::TextAlign::Center),
    )?;

    // 5. Card 3: Energy Telemetry Sparkline (Right: 96px width)
    let card3 = Rect::new(216, 32, 96, 156);
    ctx.fill_rounded_rect(card3, 6, Rgb565::new(2, 4, 10))?;
    ctx.stroke_rounded_rect(card3, 6, embedded_gui::Border::one(Rgb565::new(0, 20, 30)))?;

    ctx.draw_text(
        card3.x + 8,
        card3.y + 6,
        "POWER USAGE",
        Rgb565::CSS_SPRING_GREEN,
    )?;
    let pwr_kw = 1.4 + ((frame as f32 * 0.08).sin() * 0.5);
    ctx.draw_text(
        card3.x + 8,
        card3.y + 24,
        &format!("{:.1} kW", pwr_kw),
        Rgb565::WHITE,
    )?;

    // Real-time sparkline graph
    let graph_rect = Rect::new(card3.x + 8, card3.y + 44, card3.w - 16, 60);
    ctx.fill_rect(graph_rect, Rgb565::new(1, 3, 6))?;
    ctx.stroke_rect(
        graph_rect,
        embedded_gui::Border::one(Rgb565::new(0, 15, 20)),
    )?;

    // Draw sparkline wave
    let mut prev_pt = Point::new(graph_rect.x, graph_rect.bottom() - 10);
    let step_x = (graph_rect.w as f32) / 8.0;
    for i in 1..=8 {
        let x = graph_rect.x + (i as f32 * step_x) as i32;
        let wave_y = ((frame as f32 * 0.1 + i as f32 * 0.9).sin() * 18.0) as i32;
        let y = (graph_rect.y + 30 + wave_y).clamp(graph_rect.y + 4, graph_rect.bottom() - 4);
        let cur_pt = Point::new(x, y);
        ctx.draw_line_styled(
            prev_pt.x,
            prev_pt.y,
            cur_pt.x,
            cur_pt.y,
            StrokeStyle::new(Rgb565::CSS_SPRING_GREEN).with_width(2),
        )?;
        prev_pt = cur_pt;
    }

    let pwr_btn = Rect::new(card3.x + 8, card3.y + 116, card3.w - 16, 26);
    ctx.fill_rounded_rect(pwr_btn, 4, Rgb565::new(4, 12, 8))?;
    ctx.stroke_rounded_rect(
        pwr_btn,
        4,
        embedded_gui::Border::one(Rgb565::new(0, 30, 20)),
    )?;
    ctx.draw_text_in(
        pwr_btn.inset(EdgeInsets::all(4)),
        "GRID OPTIMAL",
        TextStyle::new(Rgb565::CSS_LIME).with_align(embedded_gui::TextAlign::Center),
    )?;

    // 6. Bottom Scene Bar: Quick Presets
    let bot_bar = Rect::new(8, 194, W - 16, 38);
    ctx.fill_rounded_rect(bot_bar, 6, Rgb565::new(2, 4, 9))?;
    ctx.stroke_rounded_rect(
        bot_bar,
        6,
        embedded_gui::Border::one(Rgb565::new(0, 20, 30)),
    )?;

    let scenes = ["MORNING", "FOCUS", "CINEMA", "NIGHT"];
    let sel_scene = ((frame / 30) % 4) as usize;
    let btn_w = (bot_bar.w.saturating_sub(20)) / 4;

    for (i, &name) in scenes.iter().enumerate() {
        let btn_x = bot_bar.x + 6 + (i as u32 * (btn_w + 2)) as i32;
        let scene_rect = Rect::new(btn_x, bot_bar.y + 6, btn_w, bot_bar.h - 12);
        let is_sel = i == sel_scene;

        if is_sel {
            ctx.fill_rounded_rect(scene_rect, 4, Rgb565::new(0, 30, 25))?;
            ctx.stroke_rounded_rect(scene_rect, 4, embedded_gui::Border::one(Rgb565::CSS_CYAN))?;
        } else {
            ctx.fill_rounded_rect(scene_rect, 4, Rgb565::new(3, 7, 12))?;
            ctx.stroke_rounded_rect(
                scene_rect,
                4,
                embedded_gui::Border::one(Rgb565::new(0, 15, 20)),
            )?;
        }

        ctx.draw_text_in(
            scene_rect.inset(EdgeInsets::all(2)),
            name,
            TextStyle::new(if is_sel {
                Rgb565::WHITE
            } else {
                Rgb565::CSS_GRAY
            })
            .with_align(embedded_gui::TextAlign::Center),
        )?;
    }

    Ok(())
}

fn record_frames() {
    let out_dir = std::path::Path::new("target/dashboard_frames");
    let _ = std::fs::create_dir_all(out_dir);

    let mut fb = Framebuffer::<FB_SIZE>::new(W, H);
    let total_frames = 90;
    println!(
        "Recording {} frames to target/dashboard_frames...",
        total_frames
    );

    for f in 0..total_frames {
        render_dashboard(&mut fb, f).unwrap();

        let mut rgb888 = Vec::with_capacity((W * H * 3) as usize);
        for p in fb.pixels() {
            let r = (p.r() << 3) | (p.r() >> 2);
            let g = (p.g() << 2) | (p.g() >> 4);
            let b = (p.b() << 3) | (p.b() >> 2);
            rgb888.push(r);
            rgb888.push(g);
            rgb888.push(b);
        }

        let filename = out_dir.join(format!("frame_{:03}.raw", f));
        std::fs::write(filename, &rgb888).unwrap();
    }
    println!("Dashboard frame recording complete!");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--record-gif") {
        record_frames();
        return;
    }

    println!("=== embedded-gui: Smart Home Dashboard Showcase ===");

    let res = std::panic::catch_unwind(|| {
        run_interactive();
    });

    if res.is_err() {
        println!("\n[Notice: Desktop display window not available in current environment]");
        println!("[Running headless performance & telemetry verification...]\n");
        run_headless();
    }
}

fn run_interactive() {
    let mut fb = Framebuffer::<FB_SIZE>::new(W, H);
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Smart Home & IoT Telemetry Dashboard", &settings);

    let mut frame = 0u32;
    let mut paused = false;

    'main_loop: loop {
        if !paused {
            frame = frame.wrapping_add(1);
        }

        render_dashboard(&mut fb, frame).unwrap();

        let full_area = Rectangle::new(Point::zero(), Size::new(W, H));
        display
            .fill_contiguous(&full_area, fb.pixels().iter().copied())
            .unwrap();
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'main_loop,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape | Keycode::Q => break 'main_loop,
                    Keycode::Space => paused = !paused,
                    _ => {}
                },
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn run_headless() {
    let mut fb = Framebuffer::<FB_SIZE>::new(W, H);
    println!("Running 60 frames of dashboard scene in headless mode...");
    let t0 = std::time::Instant::now();
    for f in 0..60 {
        render_dashboard(&mut fb, f).unwrap();
    }
    println!("-> 60 frames rendered in: {:?}", t0.elapsed());
    println!("Dashboard verification complete!");
}
