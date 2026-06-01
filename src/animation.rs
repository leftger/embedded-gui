//! Core animation primitives for `embedded-gui`.
//! Designed for deterministic, fixed-capacity operation.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Smoothstep,
    InCubic,
    OutCubic,
    InOutCubic,
    InQuart,
    OutQuart,
    InOutQuart,
    InQuint,
    OutQuint,
    InOutQuint,
    InSine,
    OutSine,
    InOutSine,
    InExpo,
    OutExpo,
    InOutExpo,
    InCirc,
    OutCirc,
    InOutCirc,
    InBack,
    OutBack,
    InOutBack,
    InBounce,
    OutBounce,
    InOutBounce,
    InElastic,
    OutElastic,
    InOutElastic,
}

#[inline]
pub fn apply_easing(t: f32, easing: Easing) -> f32 {
    let t = t.clamp(0.0, 1.0);
    const PI: f32 = core::f32::consts::PI;
    #[inline]
    fn out_bounce(t: f32) -> f32 {
        const N1: f32 = 7.5625;
        const D1: f32 = 2.75;
        if t < 1.0 / D1 {
            N1 * t * t
        } else if t < 2.0 / D1 {
            let t = t - 1.5 / D1;
            N1 * t * t + 0.75
        } else if t < 2.5 / D1 {
            let t = t - 2.25 / D1;
            N1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / D1;
            N1 * t * t + 0.984375
        }
    }
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) * 0.5
            }
        }
        Easing::Smoothstep => t * t * (3.0 - 2.0 * t),
        Easing::InCubic => t.powi(3),
        Easing::OutCubic => 1.0 - (1.0 - t).powi(3),
        Easing::InOutCubic => {
            if t < 0.5 {
                4.0 * t.powi(3)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
            }
        }
        Easing::InQuart => t.powi(4),
        Easing::OutQuart => 1.0 - (1.0 - t).powi(4),
        Easing::InOutQuart => {
            if t < 0.5 {
                8.0 * t.powi(4)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(4) * 0.5
            }
        }
        Easing::InQuint => t.powi(5),
        Easing::OutQuint => 1.0 - (1.0 - t).powi(5),
        Easing::InOutQuint => {
            if t < 0.5 {
                16.0 * t.powi(5)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(5) * 0.5
            }
        }
        Easing::InSine => 1.0 - ((t * PI) / 2.0).cos(),
        Easing::OutSine => ((t * PI) / 2.0).sin(),
        Easing::InOutSine => -(PI * t).cos() * 0.5 + 0.5,
        Easing::InExpo => {
            if t <= 0.0 {
                0.0
            } else {
                (2.0_f32).powf(10.0 * t - 10.0)
            }
        }
        Easing::OutExpo => {
            if t >= 1.0 {
                1.0
            } else {
                1.0 - (2.0_f32).powf(-10.0 * t)
            }
        }
        Easing::InOutExpo => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else if t < 0.5 {
                (2.0_f32).powf(20.0 * t - 10.0) * 0.5
            } else {
                (2.0 - (2.0_f32).powf(-20.0 * t + 10.0)) * 0.5
            }
        }
        Easing::InCirc => 1.0 - (1.0 - t * t).sqrt(),
        Easing::OutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
        Easing::InOutCirc => {
            if t < 0.5 {
                (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) * 0.5
            } else {
                ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) * 0.5
            }
        }
        Easing::InBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            c3 * t.powi(3) - c1 * t.powi(2)
        }
        Easing::OutBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        Easing::InOutBack => {
            let c1 = 1.70158;
            let c2 = c1 * 1.525;
            if t < 0.5 {
                ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) * 0.5
            } else {
                ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) * 0.5
            }
        }
        Easing::InBounce => 1.0 - out_bounce(1.0 - t),
        Easing::OutBounce => out_bounce(t),
        Easing::InOutBounce => {
            if t < 0.5 {
                (1.0 - out_bounce(1.0 - 2.0 * t)) * 0.5
            } else {
                (1.0 + out_bounce(2.0 * t - 1.0)) * 0.5
            }
        }
        Easing::InElastic => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                let c4 = (2.0 * PI) / 3.0;
                -(2.0_f32).powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c4).sin()
            }
        }
        Easing::OutElastic => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                let c4 = (2.0 * PI) / 3.0;
                (2.0_f32).powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            }
        }
        Easing::InOutElastic => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                let c5 = (2.0 * PI) / 4.5;
                if t < 0.5 {
                    -(2.0_f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin() * 0.5
                } else {
                    (2.0_f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin() * 0.5
                        + 1.0
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationId(u16);

impl AnimationId {
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationError {
    Full,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RepeatMode {
    #[default]
    Once,
    Loop,
    PingPong,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnimationState {
    #[default]
    Running,
    Finished,
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Animation {
    pub from: f32,
    pub to: f32,
    pub duration_ms: u32,
    pub easing: Easing,
    pub custom_curve: Option<fn(f32) -> f32>,
    pub custom_interpolator: Option<fn(f32, f32, f32) -> f32>,
    pub delay_ms: u32,
    pub repeat_mode: RepeatMode,
    pub repeat_count: Option<u16>,
    elapsed_ms: u32,
    iteration: u16,
    reversed: bool,
    finished: bool,
}

impl Animation {
    pub const fn new(from: f32, to: f32, duration_ms: u32, easing: Easing) -> Self {
        Self {
            from,
            to,
            duration_ms,
            easing,
            custom_curve: None,
            custom_interpolator: None,
            delay_ms: 0,
            repeat_mode: RepeatMode::Once,
            repeat_count: None,
            elapsed_ms: 0,
            iteration: 0,
            reversed: false,
            finished: false,
        }
    }

    pub const fn with_delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub const fn with_repeat_mode(mut self, repeat_mode: RepeatMode) -> Self {
        self.repeat_mode = repeat_mode;
        self
    }

    pub const fn with_repeat_count(mut self, repeat_count: Option<u16>) -> Self {
        self.repeat_count = repeat_count;
        self
    }

    pub fn with_custom_curve(mut self, curve: fn(f32) -> f32) -> Self {
        self.custom_curve = Some(curve);
        self
    }

    pub fn clear_custom_curve(&mut self) {
        self.custom_curve = None;
    }

    pub fn with_custom_interpolator(mut self, interpolator: fn(f32, f32, f32) -> f32) -> Self {
        self.custom_interpolator = Some(interpolator);
        self
    }

    pub fn clear_custom_interpolator(&mut self) {
        self.custom_interpolator = None;
    }

    pub fn set_reversed(&mut self, reversed: bool) {
        self.reversed = reversed;
    }

    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
        self.iteration = 0;
        self.finished = false;
    }

    pub fn set_elapsed(&mut self, elapsed_ms: u32) {
        self.elapsed_ms = elapsed_ms;
        self.finished = self.resolve_finished();
    }

    pub fn tick(&mut self, dt_ms: u32) -> AnimationState {
        if self.finished {
            return AnimationState::Finished;
        }
        self.elapsed_ms = self.elapsed_ms.saturating_add(dt_ms);
        self.finished = self.resolve_finished();
        if self.finished {
            AnimationState::Finished
        } else {
            AnimationState::Running
        }
    }

    pub fn value(&self) -> f32 {
        if self.delay_ms > 0 && self.elapsed_ms < self.delay_ms {
            return if self.reversed { self.to } else { self.from };
        }

        let duration = self.duration_ms.max(1);
        let active_elapsed = self.elapsed_ms.saturating_sub(self.delay_ms);
        let local_time = active_elapsed % duration;
        let mut progress = local_time as f32 / duration as f32;

        if self.finished && self.repeat_mode == RepeatMode::Once {
            progress = 1.0;
        }

        let iteration = self.current_iteration();
        let ping_pong_reverse = self.repeat_mode == RepeatMode::PingPong && (iteration % 2 == 1);
        if ping_pong_reverse {
            progress = 1.0 - progress;
        }
        if self.reversed {
            progress = 1.0 - progress;
        }

        let t = if let Some(curve) = self.custom_curve {
            curve(progress)
        } else {
            apply_easing(progress, self.easing)
        };
        if let Some(interpolator) = self.custom_interpolator {
            interpolator(self.from, self.to, t)
        } else {
            self.from + (self.to - self.from) * t
        }
    }

    pub fn is_done(&self) -> bool {
        self.finished
    }

    pub fn elapsed_ms(&self) -> u32 {
        self.elapsed_ms
    }

    pub fn iteration(&self) -> u16 {
        self.current_iteration()
    }

    fn current_iteration(&self) -> u16 {
        if self.delay_ms > 0 && self.elapsed_ms < self.delay_ms {
            return 0;
        }
        let duration = self.duration_ms.max(1);
        let active_elapsed = self.elapsed_ms.saturating_sub(self.delay_ms);
        (active_elapsed / duration) as u16
    }

    fn resolve_finished(&mut self) -> bool {
        if self.repeat_mode != RepeatMode::Once {
            if let Some(limit) = self.repeat_count {
                let iteration = self.current_iteration();
                self.iteration = iteration;
                return iteration >= limit;
            }
            return false;
        }

        let total = self.delay_ms.saturating_add(self.duration_ms);
        self.elapsed_ms >= total
    }

    pub fn duration_from_speed(delta: f32, units_per_second: f32) -> u32 {
        if delta <= 0.0 || units_per_second <= 0.0 {
            return 0;
        }
        ((delta / units_per_second) * 1000.0).ceil() as u32
    }

    pub fn total_duration_ms(&self, include_delay: bool, include_repeat_count: bool) -> Option<u32> {
        let base = if include_delay {
            self.duration_ms.saturating_add(self.delay_ms)
        } else {
            self.duration_ms
        };
        if !include_repeat_count || self.repeat_mode == RepeatMode::Once {
            return Some(base);
        }
        self.repeat_count.map(|count| base.saturating_mul(count as u32))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AnimationTrack {
    id: AnimationId,
    animation: Animation,
    last_iteration: u16,
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationManagerCallbacks {
    pub on_start: Option<fn(AnimationId)>,
    pub on_repeat: Option<fn(AnimationId, u16)>,
    pub on_complete: Option<fn(AnimationId, bool)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationManager<const N: usize> {
    tracks: [Option<AnimationTrack>; N],
    next_id: u16,
    paused: bool,
    callbacks: AnimationManagerCallbacks,
}

impl<const N: usize> Default for AnimationManager<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> AnimationManager<N> {
    pub const fn new() -> Self {
        Self {
            tracks: [None; N],
            next_id: 1,
            paused: false,
            callbacks: AnimationManagerCallbacks {
                on_start: None,
                on_repeat: None,
                on_complete: None,
            },
        }
    }

    pub fn set_callbacks(&mut self, callbacks: AnimationManagerCallbacks) {
        self.callbacks = callbacks;
    }

    pub fn start(&mut self, animation: Animation) -> Result<AnimationId, AnimationError> {
        let id = AnimationId::new(self.next_id);
        self.next_id = self.next_id.wrapping_add(1).max(1);

        if let Some(slot) = self.tracks.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(AnimationTrack {
                id,
                animation,
                last_iteration: 0,
            });
            if let Some(cb) = self.callbacks.on_start {
                cb(id);
            }
            Ok(id)
        } else {
            Err(AnimationError::Full)
        }
    }

    pub fn stop(&mut self, id: AnimationId) -> bool {
        for slot in &mut self.tracks {
            if slot.as_ref().is_some_and(|track| track.id == id) {
                *slot = None;
                if let Some(cb) = self.callbacks.on_complete {
                    cb(id, false);
                }
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self, dt_ms: u32) {
        if self.paused {
            return;
        }
        for slot in &mut self.tracks {
            if let Some(track) = slot.as_mut() {
                track.animation.tick(dt_ms);
                let iteration = track.animation.iteration();
                if iteration > track.last_iteration {
                    track.last_iteration = iteration;
                    if let Some(cb) = self.callbacks.on_repeat {
                        cb(track.id, iteration);
                    }
                }
                if track.animation.is_done() {
                    if let Some(cb) = self.callbacks.on_complete {
                        cb(track.id, true);
                    }
                    *slot = None;
                }
            }
        }
    }

    pub fn value(&self, id: AnimationId) -> Option<f32> {
        self.tracks
            .iter()
            .flatten()
            .find(|track| track.id == id)
            .map(|track| track.animation.value())
    }

    pub fn animation(&self, id: AnimationId) -> Option<&Animation> {
        self.tracks
            .iter()
            .flatten()
            .find(|track| track.id == id)
            .map(|track| &track.animation)
    }

    pub fn animation_mut(&mut self, id: AnimationId) -> Option<&mut Animation> {
        self.tracks
            .iter_mut()
            .flatten()
            .find(|track| track.id == id)
            .map(|track| &mut track.animation)
    }

    pub fn active_count(&self) -> usize {
        self.tracks.iter().flatten().count()
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn seek(&mut self, id: AnimationId, elapsed_ms: u32) -> bool {
        if let Some(track) = self
            .tracks
            .iter_mut()
            .flatten()
            .find(|track| track.id == id)
        {
            track.animation.set_elapsed(elapsed_ms);
            track.last_iteration = track.animation.iteration();
            true
        } else {
            false
        }
    }

    pub fn set_next_id_for_test(&mut self, id: u16) {
        self.next_id = id.max(1);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringAnimator {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl SpringAnimator {
    pub const fn new(value: f32, target: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            target,
            stiffness: 120.0,
            damping: 16.0,
        }
    }

    pub fn tick(&mut self, dt_ms: u32) -> f32 {
        let dt = (dt_ms as f32 / 1000.0).max(0.001);
        let force = self.stiffness * (self.target - self.value) - self.damping * self.velocity;
        self.velocity += force * dt;
        self.value += self.velocity * dt;
        self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InertiaAnimator {
    pub value: f32,
    pub velocity: f32,
    pub friction_per_second: f32,
}

impl InertiaAnimator {
    pub const fn new(value: f32, velocity: f32) -> Self {
        Self {
            value,
            velocity,
            friction_per_second: 0.88,
        }
    }

    pub fn tick(&mut self, dt_ms: u32) -> f32 {
        let dt = (dt_ms as f32 / 1000.0).max(0.001);
        self.value += self.velocity * dt;
        self.velocity *= self.friction_per_second.powf(dt);
        self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathPoint {
    pub x: f32,
    pub y: f32,
}

impl PathPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathAnimator<const N: usize> {
    points: [Option<PathPoint>; N],
    len: usize,
    pub timer: Timer,
    pub easing: Easing,
}

impl<const N: usize> PathAnimator<N> {
    pub const fn new(duration_ms: u32, easing: Easing) -> Self {
        Self {
            points: [None; N],
            len: 0,
            timer: Timer::new(duration_ms),
            easing,
        }
    }

    pub fn push_point(&mut self, point: PathPoint) -> Result<(), AnimationError> {
        if self.len >= N {
            return Err(AnimationError::Full);
        }
        self.points[self.len] = Some(point);
        self.len += 1;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.timer.reset();
    }

    pub fn tick(&mut self, dt_ms: u32) -> bool {
        self.timer.tick(dt_ms)
    }

    pub fn value(&self) -> Option<PathPoint> {
        if self.len == 0 {
            return None;
        }
        if self.len == 1 {
            return self.points[0];
        }
        let t = apply_easing(self.timer.progress(), self.easing);
        let segs = (self.len - 1) as f32;
        let pos = (t * segs).clamp(0.0, segs);
        let idx = pos.floor() as usize;
        let local = pos - idx as f32;
        let a = self.points[idx]?;
        let b = self.points[(idx + 1).min(self.len - 1)]?;
        Some(PathPoint {
            x: a.x + (b.x - a.x) * local,
            y: a.y + (b.y - a.y) * local,
        })
    }
}
