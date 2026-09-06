//! UI animation stopwatch and playback controller adapted from `bevy_time`.
//!
//! Provides elapsed time tracking, playback speed scaling, pausing, reversing,
//! and looping modes (`Once`, `Loop`, `PingPong`) for fluid, deterministic UI animations.

/// Playback repeat mode for animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackMode {
    /// Play once from start to finish and stop at 1.0.
    #[default]
    Once,
    /// Loop continuously from start to finish.
    Loop,
    /// Bounce back and forth between start and finish.
    PingPong,
}

/// A precision stopwatch and playback controller for UI animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stopwatch {
    elapsed_ms: u32,
    duration_ms: u32,
    speed_factor: f32,
    mode: PlaybackMode,
    paused: bool,
    forward: bool,
    finished: bool,
    times_finished_this_tick: u32,
}

impl Stopwatch {
    /// Creates a new stopwatch with the specified target duration in milliseconds.
    pub const fn new(duration_ms: u32) -> Self {
        Self {
            elapsed_ms: 0,
            duration_ms: if duration_ms == 0 { 1 } else { duration_ms },
            speed_factor: 1.0,
            mode: PlaybackMode::Once,
            paused: false,
            forward: true,
            finished: false,
            times_finished_this_tick: 0,
        }
    }

    /// Sets the playback mode (`Once`, `Loop`, or `PingPong`).
    pub fn with_mode(mut self, mode: PlaybackMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets playback speed multiplier (e.g. `0.5` for half-speed, `2.0` for double-speed).
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed_factor = speed.max(0.0);
        self
    }

    /// Advances the stopwatch by `delta_ms`.
    /// Returns `true` if the stopwatch completed or looped during this tick.
    pub fn tick(&mut self, delta_ms: u32) -> bool {
        self.times_finished_this_tick = 0;
        if self.paused || self.finished {
            return false;
        }

        let effective_delta = (delta_ms as f32 * self.speed_factor + 0.5) as u32;

        match self.mode {
            PlaybackMode::Once => {
                let next = self.elapsed_ms.saturating_add(effective_delta);
                if next >= self.duration_ms {
                    self.elapsed_ms = self.duration_ms;
                    self.finished = true;
                    self.times_finished_this_tick = 1;
                    true
                } else {
                    self.elapsed_ms = next;
                    false
                }
            }
            PlaybackMode::Loop => {
                let next = self.elapsed_ms.saturating_add(effective_delta);
                if next >= self.duration_ms {
                    self.times_finished_this_tick = (next / self.duration_ms).max(1);
                    self.elapsed_ms = next % self.duration_ms;
                    true
                } else {
                    self.elapsed_ms = next;
                    false
                }
            }
            PlaybackMode::PingPong => {
                if self.forward {
                    let next = self.elapsed_ms.saturating_add(effective_delta);
                    if next >= self.duration_ms {
                        let overshoot = next - self.duration_ms;
                        self.elapsed_ms = self.duration_ms.saturating_sub(overshoot);
                        self.forward = false;
                        self.times_finished_this_tick = 1;
                        true
                    } else {
                        self.elapsed_ms = next;
                        false
                    }
                } else if effective_delta >= self.elapsed_ms {
                    let overshoot = effective_delta - self.elapsed_ms;
                    self.elapsed_ms = overshoot.min(self.duration_ms);
                    self.forward = true;
                    self.times_finished_this_tick = 1;
                    true
                } else {
                    self.elapsed_ms -= effective_delta;
                    false
                }
            }
        }
    }

    /// Normalized progress from `0.0` to `1.0`.
    pub fn progress(&self) -> f32 {
        (self.elapsed_ms as f32 / self.duration_ms as f32).clamp(0.0, 1.0)
    }

    /// Progress mapped into 8-bit integer space `0..=255`.
    pub fn progress_u8(&self) -> u8 {
        ((self.elapsed_ms as u64 * 255) / self.duration_ms as u64).min(255) as u8
    }

    /// Current elapsed time in milliseconds.
    pub const fn elapsed_ms(&self) -> u32 {
        self.elapsed_ms
    }

    /// Total target duration in milliseconds.
    pub const fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    /// Pauses playback.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes playback.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Whether playback is paused.
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    /// Whether playback has finished (only applicable for `PlaybackMode::Once`).
    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    /// Number of times the animation reached an end or loop boundary during the last tick.
    pub const fn times_finished_this_tick(&self) -> u32 {
        self.times_finished_this_tick
    }

    /// Resets the stopwatch to 0 ms.
    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
        self.finished = false;
        self.forward = true;
        self.times_finished_this_tick = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stopwatch_once_mode() {
        let mut sw = Stopwatch::new(100);
        assert_eq!(sw.progress(), 0.0);

        assert!(!sw.tick(50));
        assert_eq!(sw.elapsed_ms(), 50);
        assert_eq!(sw.progress(), 0.5);

        assert!(sw.tick(50));
        assert_eq!(sw.elapsed_ms(), 100);
        assert_eq!(sw.progress(), 1.0);
        assert!(sw.is_finished());

        // Subsequent ticks do nothing
        assert!(!sw.tick(50));
        assert_eq!(sw.elapsed_ms(), 100);
    }

    #[test]
    fn test_stopwatch_loop_mode() {
        let mut sw = Stopwatch::new(100).with_mode(PlaybackMode::Loop);

        assert!(!sw.tick(60));
        assert_eq!(sw.elapsed_ms(), 60);

        assert!(sw.tick(60)); // Total 120 -> wraps to 20
        assert_eq!(sw.elapsed_ms(), 20);
        assert_eq!(sw.times_finished_this_tick(), 1);
    }

    #[test]
    fn test_stopwatch_ping_pong_mode() {
        let mut sw = Stopwatch::new(100).with_mode(PlaybackMode::PingPong);

        assert!(!sw.tick(80));
        assert_eq!(sw.elapsed_ms(), 80);

        assert!(sw.tick(30)); // Reaches 100 and bounces back 10ms -> 90ms
        assert_eq!(sw.elapsed_ms(), 90);

        assert!(!sw.tick(40)); // Backward 40ms -> 50ms
        assert_eq!(sw.elapsed_ms(), 50);

        assert!(sw.tick(60)); // Backward 60ms past 0 -> bounces forward to 10ms
        assert_eq!(sw.elapsed_ms(), 10);
    }
}
