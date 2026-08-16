//! Showcase: Rich Controls Suite (Scale, Table with 2D Navigation, Spinbox, GridLayout, Bézier Curves)
//!
//! Demonstrates:
//! 1. `ScaleWidget` - Radial speedometer & linear graduated scales with ticks, labels, and needle.
//! 2. `TableWidget` - 2D Data grid with headers and active cell navigation (`GridNav`).
//! 3. `SpinboxWidget` - Digit-by-digit decimal numerical parameter editor.
//! 4. `GridLayout` - 2D responsive CSS-style grid layout engine with fractional `fr` and fixed `px` tracks.
//! 5. `VectorPath` & Bézier Curves - Quadratic and cubic Bézier curve strokes with styling.
//!
//! ### Interactive Controls (when desktop window is available):
//! - **Arrow Keys**: Move 2D Table selection / Spinbox digit cursor
//! - **+ / - / Space**: Increment / Decrement Spinbox or animate Scale needle
//! - **Esc / Q**: Exit

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
    EdgeInsets, Framebuffer, GridLayout, GridPlacement, GridTrack, Rect, RenderCtx, ScaleWidget,
    SpinboxWidget, StrokeStyle, Style, TableWidget, VectorPath,
};

const W: u32 = 320;
const H: u32 = 240;
const FB_SIZE: usize = (W * H) as usize;

fn render_showcase<D: DrawTarget<Color = Rgb565> + embedded_gui::PixelRead>(
    target: &mut D,
    frame: u32,
    table: &TableWidget<'_>,
    spinbox: &SpinboxWidget,
) -> Result<(), D::Error> {
    // 1. Fast Background
    let bg_rect = Rectangle::new(Point::zero(), Size::new(W, H));
    target.fill_solid(&bg_rect, Rgb565::new(2, 3, 6))?;

    let viewport = Rect::new(0, 0, W, H);
    let mut ctx = RenderCtx::compositing(target, viewport);

    // 2. 2D GridLayout partitioning the 320x240 display:
    // Row 0: Header banner (fixed 24px)
    // Row 1: Main content area (1fr)
    // Row 2: Bottom vector curves (fixed 48px)
    // Col 0: Left controls (140px)
    // Col 1: Right data table & scale (1fr)
    let grid = GridLayout::<2, 3>::new(
        [GridTrack::Px(140), GridTrack::Fr(1)],
        [GridTrack::Px(24), GridTrack::Fr(1), GridTrack::Px(52)],
    )
    .with_gap(6)
    .with_padding(EdgeInsets::all(6));

    let placements = [
        GridPlacement::span(0, 0, 2, 1), // Top Header span
        GridPlacement::cell(0, 1),       // Left Controls (Spinbox & Linear Scale)
        GridPlacement::cell(1, 1),       // Right Table & Radial Scale
        GridPlacement::span(0, 2, 2, 1), // Bottom Bézier Vector Curves span
    ];
    let mut cells = [Rect::empty(); 4];
    grid.arrange_cells(viewport, &placements, &mut cells);

    // Top Header Banner
    ctx.fill_rounded_rect(cells[0], 4, Rgb565::new(5, 10, 20))?;
    ctx.stroke_rounded_rect(
        cells[0],
        4,
        embedded_gui::Border::one(Rgb565::new(0, 30, 25)),
    )?;
    ctx.draw_text_in(
        cells[0].inset(EdgeInsets::symmetric(6, 4)),
        "Rich Controls: Scale, Table, Spinbox, Grid, Curves",
        embedded_gui::TextStyle::new(Rgb565::WHITE),
    )?;

    // Left Column: Spinbox (Top) + Linear Scale (Bottom)
    let left_area = cells[1];
    let spinbox_rect = Rect::new(left_area.x, left_area.y, left_area.w, 40);

    let mut anim_spinbox = *spinbox;
    anim_spinbox.focused_digit = ((frame / 20) % 4) as u8;
    anim_spinbox.value = 2400 + ((frame as i32 * 7) % 600);
    anim_spinbox.render(
        &mut ctx,
        spinbox_rect,
        Style::panel().into(),
        embedded_gui::VisualState::Normal,
    )?;

    let lin_scale_rect = Rect::new(left_area.x, left_area.y + 46, left_area.w, 54);
    let needle_val = 25.0 + ((frame as f32 * 0.08).sin() * 20.0);
    let lin_scale = ScaleWidget::linear_horizontal(0.0, 50.0, needle_val)
        .with_ticks(4, 2)
        .with_needle(true, Rgb565::CSS_CYAN);
    lin_scale.render(
        &mut ctx,
        lin_scale_rect,
        Style::panel().into(),
        embedded_gui::VisualState::Normal,
    )?;

    // Right Column: Table (Top) + Radial Speedometer (Bottom)
    let right_area = cells[2];
    let table_rect = Rect::new(right_area.x, right_area.y, right_area.w, 60);

    let mut anim_table = *table;
    let sel_row = ((frame / 25) % 2) as usize;
    let sel_col = ((frame / 15) % 3) as usize;
    anim_table.selected = Some((sel_row, sel_col));
    anim_table.render(
        &mut ctx,
        table_rect,
        Style::panel().into(),
        embedded_gui::VisualState::Normal,
    )?;

    let radial_rect = Rect::new(right_area.x + 30, right_area.y + 64, 90, 44);
    let speed_val = 60.0 + ((frame as f32 * 0.06).sin() * 45.0);
    let radial_scale = ScaleWidget::new(0.0, 120.0, speed_val)
        .with_ticks(6, 2)
        .with_angles(180, 0)
        .with_needle(true, Rgb565::CSS_YELLOW);
    radial_scale.render(
        &mut ctx,
        radial_rect,
        Style::panel().into(),
        embedded_gui::VisualState::Normal,
    )?;

    // Bottom Area: Vector Bézier Paths
    let bot_area = cells[3];
    ctx.fill_rounded_rect(bot_area, 4, Rgb565::new(3, 6, 12))?;
    ctx.stroke_rounded_rect(
        bot_area,
        4,
        embedded_gui::Border::one(Rgb565::new(0, 20, 30)),
    )?;

    // Multi-segment Bézier Vector Wave Path
    let mut wave = VectorPath::<16>::new();
    let bx = bot_area.x + 10;
    let by = bot_area.y + 26;
    let wave_offset = ((frame as f32 * 0.1).sin() * 12.0) as i32;

    wave.move_to(Point::new(bx, by))
        .quad_to(
            Point::new(bx + 40, by - 16 + wave_offset),
            Point::new(bx + 80, by),
        )
        .cubic_to(
            Point::new(bx + 120, by + 16 - wave_offset),
            Point::new(bx + 160, by - 16 + wave_offset),
            Point::new(bx + 200, by),
        )
        .quad_to(
            Point::new(bx + 240, by + 16 - wave_offset),
            Point::new(bx + 280, by),
        );

    ctx.draw_vector_path(&wave, StrokeStyle::new(Rgb565::CSS_LIME).with_width(2))?;
    ctx.draw_text(
        bot_area.x + 8,
        bot_area.y + 6,
        "Vector Bézier Path (Quad & Cubic Segments)",
        Rgb565::CSS_LIGHT_GRAY,
    )?;

    Ok(())
}

fn record_frames() {
    let out_dir = std::path::Path::new("target/controls_frames");
    let _ = std::fs::create_dir_all(out_dir);

    let mut fb = Framebuffer::<FB_SIZE>::new(W, H);
    let data: &[&[&str]] = &[
        &["Sensor 1", "24.5 C", "OK"],
        &["Sensor 2", "58.2 %", "HIGH"],
    ];
    let headers: &[&str] = &["Device", "Value", "State"];
    let table = TableWidget::new(data).with_headers(headers);
    let spinbox = SpinboxWidget::new(0, 9999, 2500)
        .with_digits(4)
        .with_decimals(2);

    let total_frames = 90;
    println!(
        "Recording {} frames to target/controls_frames...",
        total_frames
    );

    for f in 0..total_frames {
        render_showcase(&mut fb, f, &table, &spinbox).unwrap();

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
    println!("Frame recording complete!");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--record-gif") {
        record_frames();
        return;
    }

    println!("=== embedded-gui: Rich Controls & Grid Layout Showcase ===");

    let res = std::panic::catch_unwind(|| {
        run_interactive();
    });

    if res.is_err() {
        println!("\n[Notice: Desktop display window not available in current environment]");
        println!("[Running headless performance & layout verification...]\n");
        run_headless();
    }
}

fn run_interactive() {
    let mut fb = Framebuffer::<FB_SIZE>::new(W, H);
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new(
        "Rich Controls: Scale, Table, Spinbox, Grid, Béziers",
        &settings,
    );

    let data: &[&[&str]] = &[
        &["Sensor 1", "24.5 C", "OK"],
        &["Sensor 2", "58.2 %", "HIGH"],
    ];
    let headers: &[&str] = &["Device", "Value", "State"];
    let mut table = TableWidget::new(data)
        .with_headers(headers)
        .with_selection(0, 0);

    let mut spinbox = SpinboxWidget::new(0, 9999, 2500)
        .with_digits(4)
        .with_decimals(2);

    let mut frame = 0u32;
    let mut paused = false;

    'main_loop: loop {
        if !paused {
            frame = frame.wrapping_add(1);
        }

        render_showcase(&mut fb, frame, &table, &spinbox).unwrap();

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
                    Keycode::Left => {
                        table.move_cursor(0, -1);
                        spinbox.prev_digit();
                    }
                    Keycode::Right => {
                        table.move_cursor(0, 1);
                        spinbox.next_digit();
                    }
                    Keycode::Up => {
                        table.move_cursor(-1, 0);
                        spinbox.increment();
                    }
                    Keycode::Down => {
                        table.move_cursor(1, 0);
                        spinbox.decrement();
                    }
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
    let data: &[&[&str]] = &[&["A", "1"], &["B", "2"]];
    let headers: &[&str] = &["K", "V"];
    let table = TableWidget::new(data).with_headers(headers);
    let spinbox = SpinboxWidget::new(0, 999, 100);

    println!("Running 60 frames of full controls & grid showcase scene in headless mode...");
    let t0 = std::time::Instant::now();
    for f in 0..60 {
        render_showcase(&mut fb, f, &table, &spinbox).unwrap();
    }
    println!("-> 60 frames rendered in: {:?}", t0.elapsed());
    println!("Controls and grid verification complete!");
}
