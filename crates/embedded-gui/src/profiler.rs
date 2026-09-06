//! UI performance profiling, frame pacing, and CPU load estimation for `#![no_std]` targets.
//!
//! Provides deterministic measurement of frame render times, running FPS estimation,
//! and dropped frame detection without requiring standard library timers or dynamic memory.

use embedded_graphics_core::pixelcolor::Rgb565;

use crate::geometry::Rect;

/// Tracks frame render times, computes running average FPS, and detects dropped frame budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameBudgetTracker {
    target_frame_time_ms: u32,
    samples: [u32; 16],
    sample_head: usize,
    sample_count: usize,
    dropped_frames: u32,
    total_frames: u64,
}

impl FrameBudgetTracker {
    /// Creates a new frame budget tracker with the specified target frame duration in ms
    /// (e.g. `16` for 60 FPS, `33` for 30 FPS).
    pub const fn new(target_frame_time_ms: u32) -> Self {
        Self {
            target_frame_time_ms: if target_frame_time_ms == 0 {
                16
            } else {
                target_frame_time_ms
            },
            samples: [0; 16],
            sample_head: 0,
            sample_count: 0,
            dropped_frames: 0,
            total_frames: 0,
        }
    }

    /// Records the execution time of a completed frame in milliseconds.
    pub fn record_frame(&mut self, render_time_ms: u32) {
        self.samples[self.sample_head] = render_time_ms;
        self.sample_head = (self.sample_head + 1) % self.samples.len();
        if self.sample_count < self.samples.len() {
            self.sample_count += 1;
        }

        self.total_frames = self.total_frames.saturating_add(1);

        if render_time_ms > self.target_frame_time_ms {
            self.dropped_frames = self.dropped_frames.saturating_add(1);
        }
    }

    /// Target frame budget in milliseconds.
    pub const fn target_frame_time_ms(&self) -> u32 {
        self.target_frame_time_ms
    }

    /// Running average frame render time in milliseconds over recent samples.
    pub fn average_render_time_ms(&self) -> u32 {
        if self.sample_count == 0 {
            return 0;
        }
        let sum: u32 = self.samples[..self.sample_count].iter().sum();
        sum / self.sample_count as u32
    }

    /// Estimated achievable frames per second based on recent render times.
    pub fn estimated_fps(&self) -> u32 {
        let avg = self.average_render_time_ms();
        1000u32
            .checked_div(avg)
            .unwrap_or(1000 / self.target_frame_time_ms)
            .min(1000)
    }

    /// Estimated CPU load percentage `0..=100%` dedicated to GUI rendering.
    pub fn cpu_load_percent(&self) -> u8 {
        let avg = self.average_render_time_ms();
        let load = (avg * 100) / self.target_frame_time_ms;
        load.min(100) as u8
    }

    /// Total count of frames that exceeded the target frame time budget.
    pub const fn dropped_frames(&self) -> u32 {
        self.dropped_frames
    }

    /// Total count of recorded frames since initialization or last reset.
    pub const fn total_frames(&self) -> u64 {
        self.total_frames
    }

    /// Resets all accumulated frame samples and statistics.
    pub fn reset(&mut self) {
        self.samples = [0; 16];
        self.sample_head = 0;
        self.sample_count = 0;
        self.dropped_frames = 0;
        self.total_frames = 0;
    }
}

/// Debug visualizer configuration for highlighting dirty repaint rectangles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRectVisualizer {
    pub enabled: bool,
    pub outline_color: Rgb565,
}

impl Default for DirtyRectVisualizer {
    fn default() -> Self {
        Self {
            enabled: false,
            outline_color: Rgb565::new(31, 0, 31), // Magenta
        }
    }
}

impl DirtyRectVisualizer {
    pub const fn new(outline_color: Rgb565) -> Self {
        Self {
            enabled: true,
            outline_color,
        }
    }

    /// Returns the 4 boundary line rectangles representing a 1px outline for `rect`.
    pub fn outline_rects(rect: Rect) -> [Rect; 4] {
        if rect.is_empty() {
            return [Rect::empty(); 4];
        }
        [
            Rect::new(rect.x, rect.y, rect.w, 1),
            Rect::new(rect.x, rect.bottom().saturating_sub(1), rect.w, 1),
            Rect::new(rect.x, rect.y, 1, rect.h),
            Rect::new(rect.right().saturating_sub(1), rect.y, 1, rect.h),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_budget_tracking() {
        let mut tracker = FrameBudgetTracker::new(16); // 60 FPS target (~16ms)
        assert_eq!(tracker.dropped_frames(), 0);
        assert_eq!(tracker.total_frames(), 0);

        // Record 4 frames of 8ms (within budget, ~50% CPU load)
        for _ in 0..4 {
            tracker.record_frame(8);
        }

        assert_eq!(tracker.total_frames(), 4);
        assert_eq!(tracker.dropped_frames(), 0);
        assert_eq!(tracker.average_render_time_ms(), 8);
        assert_eq!(tracker.cpu_load_percent(), 50);

        // Record a dropped frame of 24ms
        tracker.record_frame(24);
        assert_eq!(tracker.dropped_frames(), 1);
        assert_eq!(tracker.total_frames(), 5);
    }

    #[test]
    fn test_dirty_rect_visualizer_geometry() {
        let rect = Rect::new(10, 20, 100, 50);
        let outlines = DirtyRectVisualizer::outline_rects(rect);
        assert_eq!(outlines[0], Rect::new(10, 20, 100, 1));
        assert_eq!(outlines[1], Rect::new(10, 69, 100, 1));
        assert_eq!(outlines[2], Rect::new(10, 20, 1, 50));
        assert_eq!(outlines[3], Rect::new(109, 20, 1, 50));
    }
}
