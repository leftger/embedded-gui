//! Alpha-blending showcase: the *same* translucent content rendered two ways,
//! side by side, so the difference is obvious — **left = ordered dither,
//! right = true alpha blend**.
//!
//! Watch closely: the left half is a visible checkerboard *stipple* (all a
//! write-only display can do), the right half is smooth. The effect is
//! strongest near 50% opacity, so the fill holds around there. Rendered at high
//! zoom on purpose — at small pixel sizes a 50% stipple averages to the same
//! colour your eye reads as the blend, which is exactly why the difference is
//! easy to miss on a real panel.
//!
//! Press SPACE to toggle *how* it's drawn (the picture is meant to look the
//! same both ways — that's the point):
//! - **Shapes** — direct drawing via `RenderCtx::new` (`Dither`) vs
//!   `RenderCtx::compositing` (`Blend`).
//! - **Widgets** — a `GuiContext` of translucent panels via `GuiContext::render`
//!   (`Dither`) vs `render_composited` (`Blend`).
//!
//! Run: `cargo run --example alpha_blending_showcase`

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const HALF_W: u32 = 72;
const W: u32 = HALF_W * 2;
const H: u32 = 88;
const FB_N: usize = (W * H) as usize;
const HALF_N: usize = (HALF_W * H) as usize;
const SCALE: u32 = 8; // big pixels so the dither checkerboard is resolvable
const FRAME_MS: u32 = 33;

const BACKDROP: Rgb565 = Rgb565::new(6, 12, 8); // dark so translucency shows
const RED: Rgb565 = Rgb565::new(31, 0, 0);
const GREEN: Rgb565 = Rgb565::new(0, 63, 0);
const BLUE: Rgb565 = Rgb565::new(0, 0, 31);

#[derive(Clone, Copy, PartialEq)]
enum Scene {
    Shapes,
    Widgets,
}

/// Direct-drawing test scene into one half. Generic over the compositor `C`, so
/// the identical calls dither or blend depending on the ctx it's given.
fn draw_shapes<D, C>(ctx: &mut RenderCtx<'_, D, C>, ox: i32, alpha: u8)
where
    D: DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let _ = ctx.fill_rect(Rect::new(ox, 0, HALF_W, H), BACKDROP);
    let _ = ctx.fill_rounded_rect_alpha(Rect::new(ox + 8, 18, 40, 40), 7, RED, alpha);
    let _ = ctx.fill_rounded_rect_alpha(Rect::new(ox + 24, 18, 40, 40), 7, GREEN, alpha);
    let _ = ctx.fill_rounded_rect_alpha(Rect::new(ox + 16, 34, 40, 40), 7, BLUE, alpha);
}

/// A `GuiContext` of three overlapping translucent panels (one half wide),
/// laid out to match `draw_shapes`.
fn build_widget_gui() -> (GuiContext<'static, 8, 8, 16>, [WidgetId; 3]) {
    let mut gui = GuiContext::new(Rect::new(0, 0, HALF_W, H));
    let panel = |bg: Rgb565| Style {
        background: Some(bg),
        corner_radius: 7,
        ..Style::new()
    };
    let a = gui.add_panel(Rect::new(8, 18, 40, 40), panel(RED)).unwrap();
    let b = gui
        .add_panel(Rect::new(24, 18, 40, 40), panel(GREEN))
        .unwrap();
    let c = gui
        .add_panel(Rect::new(16, 34, 40, 40), panel(BLUE))
        .unwrap();
    (gui, [a, b, c])
}

/// Copy a framebuffer into the SDL surface at horizontal offset `dst_x`.
fn blit<const N: usize>(fb: &Framebuffer<N>, sim: &mut SimulatorDisplay<Rgb565>, dst_x: i32) {
    for y in 0..fb.height() as i32 {
        for x in 0..fb.width() as i32 {
            let color = fb.get_pixel(Point::new(x, y));
            let _ = sim.draw_iter([Pixel(Point::new(dst_x + x, y), color)]);
        }
    }
}

fn main() {
    let mut sim = SimulatorDisplay::<Rgb565>::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(SCALE).build();
    let mut window = Window::new("Alpha blending — DITHER (left) vs BLEND (right)", &settings);

    let mut fb = Framebuffer::<FB_N>::new(W, H);
    let mut fb_left = Framebuffer::<HALF_N>::new(HALF_W, H);
    let mut fb_right = Framebuffer::<HALF_N>::new(HALF_W, H);
    let (mut gui, panel_ids) = build_widget_gui();

    let mut scene = Scene::Shapes;
    let mut t: f32 = 0.0;

    'running: loop {
        // Gently breathe around 50% opacity (~112..144), where the dither
        // checkerboard is densest and the contrast with blend is greatest.
        t += 0.04;
        let alpha = (128.0 + t.sin() * 16.0) as u8;

        let _ = sim.clear(Rgb565::BLACK);

        match scene {
            Scene::Shapes => {
                fb.clear_color(Rgb565::BLACK);
                {
                    let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, W, H));
                    ctx.set_clip(Rect::new(0, 0, HALF_W, H));
                    draw_shapes(&mut ctx, 0, alpha);
                }
                {
                    let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, W, H));
                    ctx.set_clip(Rect::new(HALF_W as i32, 0, HALF_W, H));
                    draw_shapes(&mut ctx, HALF_W as i32, alpha);
                }
                blit(&fb, &mut sim, 0);
            }
            Scene::Widgets => {
                for id in panel_ids {
                    let _ = gui.set_widget_opacity(id, alpha);
                }
                fb_left.clear_color(BACKDROP);
                let _ = gui.render(&mut fb_left);
                fb_right.clear_color(BACKDROP);
                let _ = gui.render_composited(&mut fb_right);
                blit(&fb_left, &mut sim, 0);
                blit(&fb_right, &mut sim, HALF_W as i32);
            }
        }

        // Divider, labels, and a scene hint, drawn opaque onto the SDL surface.
        {
            let mut ctx = RenderCtx::new(&mut sim, Rect::new(0, 0, W, H));
            let _ = ctx.fill_rect(
                Rect::new(HALF_W as i32 - 1, 0, 2, H),
                Rgb565::new(12, 24, 12),
            );
            let _ = ctx.draw_text_with_font(4, 3, "DITHER", Rgb565::WHITE, FontId::Scaled6x10);
            let _ = ctx.draw_text_with_font(
                HALF_W as i32 + 4,
                3,
                "BLEND",
                Rgb565::WHITE,
                FontId::Scaled6x10,
            );
            let hint = match scene {
                Scene::Shapes => "direct draw  (space: widgets)",
                Scene::Widgets => "widget tree  (space: shapes)",
            };
            let _ = ctx.draw_text_with_font(
                4,
                H as i32 - 9,
                hint,
                Rgb565::new(18, 36, 18),
                FontId::Medium4x7,
            );
        }

        window.update(&sim);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Space => {
                        scene = match scene {
                            Scene::Shapes => Scene::Widgets,
                            Scene::Widgets => Scene::Shapes,
                        };
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(FRAME_MS as u64));
    }
}
