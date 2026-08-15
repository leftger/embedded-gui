//! Showcase: Interactive Draw Task Graph & Partial Band-Buffer Rendering
//!
//! Demonstrates:
//! 1. Queuing retained-mode drawing commands into a fixed-capacity `DrawTaskQueue`.
//! 2. Dispatching drawing tasks through pluggable `DrawUnit` hardware/software units.
//! 3. Band-buffered rendering using `PartialBandBuffer` (narrow horizontal slices) directly
//!    to an interactive SDL2 desktop window.
//!
//! ### Controls:
//! - **Space**: Add / cycle draw tasks in the queue
//! - **Esc / Q**: Exit

use core::convert::Infallible;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, WebColors},
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::{
    geometry::Rect,
    render::{DrawTask, DrawTaskQueue, PartialBandBuffer, TextStyle, WindowedDrawTarget},
    style::Border,
};

const W: u32 = 320;
const H: u32 = 240;

/// Frame buffer wrapping SimulatorDisplay that supports WindowedDrawTarget.
struct WindowedSimulator {
    display: SimulatorDisplay<Rgb565>,
    window_count: usize,
    active_window: Option<Rectangle>,
}

impl WindowedSimulator {
    fn new() -> Self {
        Self {
            display: SimulatorDisplay::<Rgb565>::new(Size::new(W, H)),
            window_count: 0,
            active_window: None,
        }
    }
}

impl OriginDimensions for WindowedSimulator {
    fn size(&self) -> Size {
        Size::new(W, H)
    }
}

impl DrawTarget for WindowedSimulator {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.display.draw_iter(pixels)
    }
}

impl WindowedDrawTarget for WindowedSimulator {
    fn set_window(&mut self, rect: &Rectangle) -> Result<(), Self::Error> {
        self.active_window = Some(*rect);
        self.window_count += 1;
        Ok(())
    }
}

fn main() {
    println!("=== embedded-gui: Interactive Draw Task Graph & Band Buffer Showcase ===");
    println!("Controls:");
    println!("  [Space]  - Animate / cycle draw task progress");
    println!("  [Esc/Q]  - Exit");

    let mut sim = WindowedSimulator::new();
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Draw Task Graph & Band Buffer (320x240)", &settings);

    let mut progress: u32 = 120;
    let mut dir: i32 = 2;

    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Escape | Keycode::Q => break 'running,
                    Keycode::Space => {
                        progress = (progress + 20) % 250;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Animate progress smoothly
        if progress >= 240 {
            dir = -2;
        } else if progress <= 20 {
            dir = 2;
        }
        progress = (progress as i32 + dir).max(0) as u32;

        // 1. Build a queue of draw tasks
        let mut queue = DrawTaskQueue::<16>::new();

        // Background fill task
        queue
            .push(DrawTask::Fill {
                rect: Rect::new(0, 0, W, H),
                color: Rgb565::new(2, 4, 6),
                radius: 0,
                opacity: 255,
            })
            .expect("Task queued");

        // Top Status Header
        queue
            .push(DrawTask::Fill {
                rect: Rect::new(0, 0, W, 22),
                color: Rgb565::new(4, 8, 14),
                radius: 0,
                opacity: 255,
            })
            .expect("Task queued");

        queue
            .push(DrawTask::Label {
                rect: Rect::new(12, 5, 200, 14),
                text: "BAND BUFFER RENDER ENGINE",
                style: TextStyle::new(Rgb565::CSS_CYAN),
            })
            .expect("Task queued");

        // Main Panel Card
        queue
            .push(DrawTask::Fill {
                rect: Rect::new(20, 36, 280, 90),
                color: Rgb565::new(6, 12, 18),
                radius: 6,
                opacity: 255,
            })
            .expect("Task queued");

        queue
            .push(DrawTask::Border {
                rect: Rect::new(20, 36, 280, 90),
                border: Border::one(Rgb565::CSS_DARK_CYAN),
                radius: 6,
            })
            .expect("Task queued");

        // Card Label
        queue
            .push(DrawTask::Label {
                rect: Rect::new(35, 48, 250, 16),
                text: "SRAM CONSUMPTION: 12.8 KB",
                style: TextStyle::new(Rgb565::CSS_WHITE),
            })
            .expect("Task queued");

        queue
            .push(DrawTask::Label {
                rect: Rect::new(35, 66, 250, 14),
                text: "Transfers 12 bands of 20 lines each",
                style: TextStyle::new(Rgb565::new(15, 30, 20)),
            })
            .expect("Task queued");

        // Progress Bar Background & Active Fill
        queue
            .push(DrawTask::Fill {
                rect: Rect::new(35, 94, 250, 14),
                color: Rgb565::new(3, 6, 9),
                radius: 3,
                opacity: 255,
            })
            .expect("Task queued");

        queue
            .push(DrawTask::Fill {
                rect: Rect::new(35, 94, progress, 14),
                color: Rgb565::CSS_GREEN,
                radius: 3,
                opacity: 255,
            })
            .expect("Task queued");

        // Secondary Info Card
        queue
            .push(DrawTask::Fill {
                rect: Rect::new(20, 138, 280, 80),
                color: Rgb565::new(6, 12, 18),
                radius: 6,
                opacity: 255,
            })
            .expect("Task queued");

        queue
            .push(DrawTask::Border {
                rect: Rect::new(20, 138, 280, 80),
                border: Border::one(Rgb565::new(12, 24, 18)),
                radius: 6,
            })
            .expect("Task queued");

        queue
            .push(DrawTask::Label {
                rect: Rect::new(35, 152, 250, 16),
                text: "HARDWARE DMA STREAMING",
                style: TextStyle::new(Rgb565::CSS_GOLD),
            })
            .expect("Task queued");

        queue
            .push(DrawTask::Label {
                rect: Rect::new(35, 172, 250, 14),
                text: "Windowed SPI/8080 transfers direct to LCD",
                style: TextStyle::new(Rgb565::new(20, 40, 30)),
            })
            .expect("Task queued");

        // 2. Render using PartialBandBuffer (20 lines per band = 12.8 KB RAM)
        let screen = Rect::new(0, 0, W, H);
        let mut band_buffer = PartialBandBuffer::<{ 320 * 20 }>::new(320, 20);

        band_buffer
            .render_tasks_banded(screen, screen, &queue, &mut sim, Some(Rgb565::new(2, 4, 6)))
            .expect("Render succeeded");

        window.update(&sim.display);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
