#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    Smoothstep,
}

#[inline]
pub fn apply_easing(t: f32, easing: Easing) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        Easing::Smoothstep => t * t * (3.0 - 2.0 * t),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timer {
    pub duration_ms: u32,
    pub elapsed_ms: u32,
    pub repeating: bool,
}

impl Timer {
    pub const fn new(duration_ms: u32) -> Self {
        Self {
            duration_ms,
            elapsed_ms: 0,
            repeating: false,
        }
    }

    pub const fn repeating(duration_ms: u32) -> Self {
        Self {
            duration_ms,
            elapsed_ms: 0,
            repeating: true,
        }
    }

    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
    }

    pub fn tick(&mut self, dt_ms: u32) -> bool {
        self.elapsed_ms = self.elapsed_ms.saturating_add(dt_ms);
        if self.elapsed_ms >= self.duration_ms {
            if self.repeating && self.duration_ms > 0 {
                self.elapsed_ms %= self.duration_ms;
            }
            true
        } else {
            false
        }
    }

    pub fn progress(&self) -> f32 {
        if self.duration_ms == 0 {
            return 1.0;
        }
        (self.elapsed_ms as f32 / self.duration_ms as f32).min(1.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub timer: Timer,
    pub easing: Easing,
}

impl Tween {
    pub const fn new(from: f32, to: f32, duration_ms: u32, easing: Easing) -> Self {
        Self {
            from,
            to,
            timer: Timer::new(duration_ms),
            easing,
        }
    }

    pub fn reset(&mut self) {
        self.timer.reset();
    }

    pub fn tick(&mut self, dt_ms: u32) -> bool {
        self.timer.tick(dt_ms)
    }

    pub fn value(&self) -> f32 {
        let t = apply_easing(self.timer.progress(), self.easing);
        self.from + (self.to - self.from) * t
    }

    pub fn is_done(&self) -> bool {
        self.timer.progress() >= 1.0
    }
}
