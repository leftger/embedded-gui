//! Showcase: Framebuffer-Free Rendering
//!
//! Inspired by <https://bensimms.moe/reverse-engineering-scooter/>, which
//! reverse-engineers a scooter dashboard's ST7796 display driver: the panel
//! has its own on-chip GRAM, so the firmware never needs a full host-side
//! RGB565 framebuffer — it just issues "set column/row address window, then
//! stream pixel data" (CASET/RASET/RAMWR) commands directly to the controller.
//!
//! `embedded-gui` already renders this way via [`GuiContext::render_dirty_buffered`]:
//! each dirty rect is rendered into a small caller-sized scratch buffer, then
//! flushed to the display with a single `fill_contiguous` call — the
//! `embedded-graphics` equivalent of a windowed hardware burst write. This
//! example makes that concrete: [`DirectPanel`] below is a write-only
//! `DrawTarget` that holds *no* full-screen pixel buffer of its own (only a
//! window/stat-tracking state), standing in for a real MIPI-DBI-style
//! controller. The only pixel storage this program owns is a 512-pixel
//! (1 KiB) scratch buffer — regardless of how large the display is.
//!
//! ### Interactive Controls (when desktop window is available):
//! - **Esc / Q**: Exit

use core::convert::Infallible;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::prelude::*;

const W: u32 = 320;
const H: u32 = 240;

/// Bytes a naive "render into a full RGB565 framebuffer, then blit" design
/// would need for this screen — for comparison against the fixed 1 KiB
/// scratch buffer this program actually allocates.
const FULL_FRAME_BYTES: u32 = W * H * 2;

/// A write-only "direct panel" bus: no full-screen pixel array lives here.
/// `gram` (the simulator window) stands in for the controller's own on-chip
/// GRAM — physically part of the display, never MCU RAM — purely so the
/// example is visible on screen; a real driver would instead bit-bang a
/// parallel/SPI bus, as in the linked article's `ParallelInterface`.
struct DirectPanel {
    gram: SimulatorDisplay<Rgb565>,
    pixels_streamed_last_frame: u32,
    transfers_last_frame: u32,
    pixels_streamed_this_frame: u32,
    transfers_this_frame: u32,
}

impl DirectPanel {
    fn new(size: Size) -> Self {
        Self {
            gram: SimulatorDisplay::new(size),
            pixels_streamed_last_frame: 0,
            transfers_last_frame: 0,
            pixels_streamed_this_frame: 0,
            transfers_this_frame: 0,
        }
    }

    /// Rolls this frame's stream stats into "last frame" and resets the
    /// counters, so the on-screen stat labels show a settled value instead
    /// of a mid-render partial count.
    fn end_frame(&mut self) {
        self.pixels_streamed_last_frame = self.pixels_streamed_this_frame;
        self.transfers_last_frame = self.transfers_this_frame;
        self.pixels_streamed_this_frame = 0;
        self.transfers_this_frame = 0;
    }
}

impl OriginDimensions for DirectPanel {
    fn size(&self) -> Size {
        self.gram.size()
    }
}

impl DrawTarget for DirectPanel {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        // Not exercised by `render_dirty_buffered` (it always flushes via
        // `fill_contiguous`), but still a real bus transfer on real
        // hardware: each pixel is its own 1x1 address-window + RAMWR.
        for pixel in pixels {
            self.pixels_streamed_this_frame += 1;
            self.transfers_this_frame += 1;
            self.gram.draw_iter([pixel])?;
        }
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        // The moment-by-moment equivalent of CASET + RASET + a burst of
        // RAMWR writes for `area` on a real MIPI-DBI controller.
        self.transfers_this_frame += 1;
        self.pixels_streamed_this_frame += area.size.width * area.size.height;
        self.gram.fill_contiguous(area, colors)
    }
}

fn main() {
    let mut panel = DirectPanel::new(Size::new(W, H));
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("embedded-gui: framebuffer-free rendering", &settings);

    let mut gui = GuiContext::<16, 24, 16>::new(Rect::new(0, 0, W, H));
    gui.add_panel(Rect::new(0, 0, W, H), Style::panel())
        .unwrap();
    gui.add_label(
        Rect::new(12, 8, 280, 14),
        "FRAMEBUFFER-FREE RENDERING",
        Style::label(),
    )
    .unwrap();
    gui.add_label(
        Rect::new(12, 24, 280, 12),
        "1 KIB SCRATCH BUFFER, NO HOST FRAMEBUFFER",
        Style::label(),
    )
    .unwrap();

    let progress = gui
        .add_progress_bar(Rect::new(20, 50, 280, 14), 0.0, Style::progress())
        .unwrap();
    let percent = gui
        .add_value_label(Rect::new(20, 70, 130, 14), "PCT", 0, Style::panel())
        .unwrap();
    let dirty_count = gui
        .add_value_label(
            Rect::new(170, 70, 130, 14),
            "DIRTY RECTS",
            0,
            Style::panel(),
        )
        .unwrap();

    let streamed = gui
        .add_value_label(
            Rect::new(20, 100, 130, 14),
            "PIXELS/FRAME",
            0,
            Style::panel(),
        )
        .unwrap();
    let transfers = gui
        .add_value_label(
            Rect::new(170, 100, 130, 14),
            "XFERS/FRAME",
            0,
            Style::panel(),
        )
        .unwrap();

    gui.add_label(
        Rect::new(20, 130, 280, 12),
        "FULL FRAME WOULD BE 153600 BYTES",
        Style::label(),
    )
    .unwrap();
    gui.add_label(
        Rect::new(20, 146, 280, 12),
        "THIS PROGRAM OWNS 1024 BYTES OF PIXEL RAM",
        Style::label(),
    )
    .unwrap();
    debug_assert_eq!(FULL_FRAME_BYTES, 153_600);

    // The *only* pixel storage this program owns, no matter how large the
    // display is: 512 Rgb565 pixels = 1 KiB. `render_dirty_buffered` renders
    // each dirty rect through this buffer in row/band-sized chunks and
    // streams every chunk straight to `panel` via `fill_contiguous`.
    let mut scratch = [Rgb565::BLACK; 512];

    gui.render_dirty_buffered(&mut panel, &mut scratch).unwrap();
    gui.clear_dirty();
    panel.end_frame();
    window.update(&panel.gram);

    let mut tween = Animation::new(0.0, 1.0, 1400, Easing::InOutSine)
        .with_repeat_mode(RepeatMode::PingPong)
        .with_repeat_count(None);

    'running: loop {
        tween.tick(16);

        let value = tween.value();
        gui.set_progress(progress, value).unwrap();
        gui.set_value_label(percent, (value * 100.0) as i32)
            .unwrap();
        gui.set_value_label(dirty_count, gui.dirty_regions().len() as i32)
            .unwrap();
        gui.set_value_label(streamed, panel.pixels_streamed_last_frame as i32)
            .unwrap();
        gui.set_value_label(transfers, panel.transfers_last_frame as i32)
            .unwrap();

        gui.render_dirty_buffered(&mut panel, &mut scratch).unwrap();
        gui.clear_dirty();
        panel.end_frame();

        window.update(&panel.gram);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Escape | Keycode::Q,
                    ..
                } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
