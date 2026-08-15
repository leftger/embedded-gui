//! Showcase: Draw Task Graph & Partial Band-Buffer Rendering
//!
//! Demonstrates:
//! 1. Queuing retained-mode drawing commands into a fixed-capacity `DrawTaskQueue`.
//! 2. Dispatching drawing tasks through pluggable `DrawUnit` hardware/software units.
//! 3. Band-buffered rendering using `PartialBandBuffer` on microcontrollers with constrained RAM.

use core::convert::Infallible;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, WebColors},
    primitives::Rectangle,
};
use embedded_gui::{
    geometry::Rect,
    render::{DrawTask, DrawTaskQueue, PartialBandBuffer, TextStyle, WindowedDrawTarget},
    style::Border,
};

/// Mock LCD controller simulating a hardware display with set_window address windowing.
struct MockWindowedLcd {
    width: u32,
    height: u32,
    pixels: [Rgb565; 320 * 240],
    window_count: usize,
    active_window: Option<Rectangle>,
}

impl MockWindowedLcd {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: [Rgb565::CSS_BLACK; 320 * 240],
            window_count: 0,
            active_window: None,
        }
    }
}

impl OriginDimensions for MockWindowedLcd {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl DrawTarget for MockWindowedLcd {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(pt, color) in pixels {
            if pt.x >= 0 && pt.y >= 0 && (pt.x as u32) < self.width && (pt.y as u32) < self.height {
                let idx = pt.y as usize * self.width as usize + pt.x as usize;
                self.pixels[idx] = color;
            }
        }
        Ok(())
    }
}

impl WindowedDrawTarget for MockWindowedLcd {
    fn set_window(&mut self, rect: &Rectangle) -> Result<(), Self::Error> {
        self.active_window = Some(*rect);
        self.window_count += 1;
        Ok(())
    }
}

fn main() {
    println!("=== embedded-gui: Draw Task Graph & Band Buffer Showcase ===");

    // 1. Build a queue of draw tasks
    let mut queue = DrawTaskQueue::<16>::new();

    // Background fill task
    queue
        .push(DrawTask::Fill {
            rect: Rect::new(0, 0, 320, 240),
            color: Rgb565::new(3, 6, 8),
            radius: 0,
            opacity: 255,
        })
        .expect("Task queued");

    // Card panel task
    queue
        .push(DrawTask::Fill {
            rect: Rect::new(20, 20, 280, 80),
            color: Rgb565::new(8, 16, 24),
            radius: 8,
            opacity: 255,
        })
        .expect("Task queued");

    // Border task
    queue
        .push(DrawTask::Border {
            rect: Rect::new(20, 20, 280, 80),
            border: Border::one(Rgb565::CSS_CYAN),
            radius: 8,
        })
        .expect("Task queued");

    // Label task
    queue
        .push(DrawTask::Label {
            rect: Rect::new(35, 35, 250, 20),
            text: "BANDED MEMORY ENGINE",
            style: TextStyle::new(Rgb565::CSS_WHITE),
        })
        .expect("Task queued");

    // Progress bar task
    queue
        .push(DrawTask::Fill {
            rect: Rect::new(35, 65, 250, 12),
            color: Rgb565::new(5, 10, 15),
            radius: 3,
            opacity: 255,
        })
        .expect("Task queued");

    queue
        .push(DrawTask::Fill {
            rect: Rect::new(35, 65, 175, 12),
            color: Rgb565::CSS_GREEN,
            radius: 3,
            opacity: 255,
        })
        .expect("Task queued");

    println!("Queued {} draw tasks in task graph.", queue.len());

    // 2. Render using PartialBandBuffer (20 lines per band = only 12.8 KB RAM for a 320x240 screen)
    let screen = Rect::new(0, 0, 320, 240);
    let mut lcd = MockWindowedLcd::new(320, 240);
    let mut band_buffer = PartialBandBuffer::<{ 320 * 20 }>::new(320, 20);

    band_buffer
        .render_tasks_banded(screen, screen, &queue, &mut lcd, Some(Rgb565::new(3, 6, 8)))
        .expect("Render succeeded");

    println!(
        "Rendered 320x240 screen in bands using {} window transfers.",
        lcd.window_count
    );
    println!("Band buffer rendering demonstration completed successfully!");
}
