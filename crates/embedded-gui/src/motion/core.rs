//! Core animation primitives for `embedded-gui`.
//! Designed for deterministic, fixed-capacity operation.
#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Smoothstep,
    Smootherstep,
    Steps(u8),
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
    /// Spatial curve for stack push/pop (`interpolate_moook`).
    Moook,
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
        Easing::Smootherstep => t * t * t * (t * (t * 6.0 - 15.0) + 10.0),
        Easing::Steps(count) => {
            let count = count.max(1) as f32;
            if t >= 1.0 {
                1.0
            } else {
                ((t * count) as u32 as f32) / count
            }
        }
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
                    (2.0_f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin() * 0.5 + 1.0
                }
            }
        }
        Easing::Moook => crate::animation_timing::moook_curve(t),
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationHandlers {
    pub on_started: Option<fn()>,
    pub on_stopped: Option<fn(bool)>,
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
    pub handlers: AnimationHandlers,
    pub delay_ms: u32,
    pub repeat_mode: RepeatMode,
    pub repeat_count: Option<u16>,
    elapsed_ms: u32,
    iteration: u16,
    reversed: bool,
    started: bool,
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
            handlers: AnimationHandlers {
                on_started: None,
                on_stopped: None,
            },
            delay_ms: 0,
            repeat_mode: RepeatMode::Once,
            repeat_count: None,
            elapsed_ms: 0,
            iteration: 0,
            reversed: false,
            started: false,
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

    pub fn set_handlers(&mut self, handlers: AnimationHandlers) {
        self.handlers = handlers;
    }

    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
        self.iteration = 0;
        self.started = false;
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
        self.emit_started_if_ready();
        self.finished = self.resolve_finished();
        if self.finished {
            if let Some(cb) = self.handlers.on_stopped {
                cb(true);
            }
        }
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

    pub(crate) fn notify_stopped(&mut self, finished: bool) {
        if let Some(cb) = self.handlers.on_stopped {
            cb(finished);
        }
    }

    fn emit_started_if_ready(&mut self) {
        if self.started {
            return;
        }
        if self.delay_ms > 0 && self.elapsed_ms < self.delay_ms {
            return;
        }
        self.started = true;
        if let Some(cb) = self.handlers.on_started {
            cb();
        }
    }

    pub fn duration_from_speed(delta: f32, units_per_second: f32) -> u32 {
        if delta <= 0.0 || units_per_second <= 0.0 {
            return 0;
        }
        ((delta / units_per_second) * 1000.0).ceil() as u32
    }

    pub fn total_duration_ms(
        &self,
        include_delay: bool,
        include_repeat_count: bool,
    ) -> Option<u32> {
        let base = if include_delay {
            self.duration_ms.saturating_add(self.delay_ms)
        } else {
            self.duration_ms
        };
        if !include_repeat_count || self.repeat_mode == RepeatMode::Once {
            return Some(base);
        }
        self.repeat_count
            .map(|count| base.saturating_mul(count as u32))
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
                if let Some(track) = slot.as_mut() {
                    track.animation.notify_stopped(false);
                }
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

    pub fn seek_stepped(&mut self, id: AnimationId, elapsed_ms: u32, step_ms: u32) -> bool {
        let Some(track) = self
            .tracks
            .iter_mut()
            .flatten()
            .find(|track| track.id == id)
        else {
            return false;
        };
        let step = step_ms.max(1);
        let current = track.animation.elapsed_ms();
        if elapsed_ms <= current {
            track.animation.set_elapsed(elapsed_ms);
            track.last_iteration = track.animation.iteration();
            return true;
        }
        let mut cursor = current;
        while cursor < elapsed_ms {
            cursor = core::cmp::min(cursor.saturating_add(step), elapsed_ms);
            track.animation.set_elapsed(cursor);
        }
        track.last_iteration = track.animation.iteration();
        true
    }

    pub fn replay_stepped<F>(
        &mut self,
        id: AnimationId,
        elapsed_ms: u32,
        step_ms: u32,
        mut on_sample: F,
    ) -> bool
    where
        F: FnMut(f32),
    {
        let Some(track) = self
            .tracks
            .iter_mut()
            .flatten()
            .find(|track| track.id == id)
        else {
            return false;
        };
        let step = step_ms.max(1);
        let current = track.animation.elapsed_ms();
        if elapsed_ms <= current {
            track.animation.set_elapsed(elapsed_ms);
            on_sample(track.animation.value());
            track.last_iteration = track.animation.iteration();
            return true;
        }
        let mut cursor = current;
        while cursor < elapsed_ms {
            cursor = core::cmp::min(cursor.saturating_add(step), elapsed_ms);
            track.animation.set_elapsed(cursor);
            on_sample(track.animation.value());
        }
        track.last_iteration = track.animation.iteration();
        true
    }

    pub fn set_next_id_for_test(&mut self, id: u16) {
        self.next_id = id.max(1);
    }
}

/// Default position/velocity settling thresholds for [`SpringAnimator::is_done`],
/// matching Flutter's `Tolerance.defaultTolerance` (±0.001 on both axes).
pub const SPRING_TOLERANCE_DISTANCE: f32 = 1e-3;
pub const SPRING_TOLERANCE_VELOCITY: f32 = 1e-3;

/// `e^x`, expressed via the crate's shared `powf` (backed by `std`, `libm`, or
/// `micromath` depending on target) so the spring solver needs no extra math
/// backend surface.
#[inline]
fn exp(x: f32) -> f32 {
    core::f32::consts::E.powf(x)
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

    /// Builds a spring from a perceptual `duration` and a `bounce` amount,
    /// mirroring Flutter's `SpringDescription.withDurationAndBounce`. `bounce`
    /// of `0.0` is critically damped (no oscillation), `(0.0, 1.0]` oscillates
    /// (`1.0` never settles), and negative values are overdamped (sluggish).
    /// This is a friendlier authoring surface than raw `stiffness`/`damping`
    /// for KDL markup or hand-tuned motion presets.
    pub fn with_duration_and_bounce(
        value: f32,
        target: f32,
        duration_ms: u32,
        bounce: f32,
    ) -> Self {
        let duration_s = (duration_ms.max(1) as f32) / 1000.0;
        const TAU_SQUARED: f32 = 4.0 * core::f32::consts::PI * core::f32::consts::PI;
        let stiffness = TAU_SQUARED / (duration_s * duration_s);
        let damping_ratio = if bounce > 0.0 {
            1.0 - bounce
        } else {
            1.0 / (bounce + 1.0)
        };
        let damping = damping_ratio * 2.0 * stiffness.sqrt();
        Self {
            value,
            velocity: 0.0,
            target,
            stiffness,
            damping,
        }
    }

    /// Advances the spring by `dt_ms` using the closed-form solution of the
    /// damped harmonic oscillator (critically/over/under-damped branches,
    /// after Flutter's `SpringSimulation`), rather than integrating the
    /// force numerically step by step. Since `value(t)`/`velocity(t)` are
    /// evaluated analytically from the state at the start of this tick, the
    /// result has no integration drift and stays stable for any `dt_ms`
    /// (including large or irregular frame gaps).
    pub fn tick(&mut self, dt_ms: u32) -> f32 {
        let dt = dt_ms as f32 / 1000.0;
        if dt <= 0.0 {
            return self.value;
        }

        let d0 = self.value - self.target;
        let v0 = self.velocity;
        let k = self.stiffness;
        let c = self.damping;
        let disc = c * c - 4.0 * k;

        let (d1, v1) = if disc > 0.0 {
            // Overdamped: two real exponential roots.
            let sq = disc.sqrt();
            let r1 = (-c - sq) * 0.5;
            let r2 = (-c + sq) * 0.5;
            let c2 = (v0 - r1 * d0) / (r2 - r1);
            let c1 = d0 - c2;
            let e1 = exp(r1 * dt);
            let e2 = exp(r2 * dt);
            (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        } else if disc < 0.0 {
            // Underdamped: decaying oscillation.
            let w = (-disc).sqrt() * 0.5;
            let r = -c * 0.5;
            let c1 = d0;
            let c2 = (v0 - r * d0) / w;
            let e = exp(r * dt);
            let (sin_wt, cos_wt) = ((w * dt).sin(), (w * dt).cos());
            let x = e * (c1 * cos_wt + c2 * sin_wt);
            let dx = e * ((r * c1 + w * c2) * cos_wt + (r * c2 - w * c1) * sin_wt);
            (x, dx)
        } else {
            // Critically damped: fastest non-oscillating return.
            let r = -c * 0.5;
            let c1 = d0;
            let c2 = v0 - r * d0;
            let e = exp(r * dt);
            ((c1 + c2 * dt) * e, r * (c1 + c2 * dt) * e + c2 * e)
        };

        self.value = self.target + d1;
        self.velocity = v1;
        self.value
    }

    /// Whether the spring has settled within the default tolerance
    /// ([`SPRING_TOLERANCE_DISTANCE`]/[`SPRING_TOLERANCE_VELOCITY`]) — both the
    /// remaining distance to `target` and the current `velocity` must be
    /// negligible, so a fast-moving spring that happens to cross `target`
    /// isn't mistaken for "done".
    pub fn is_done(&self) -> bool {
        self.is_done_with_tolerance(SPRING_TOLERANCE_DISTANCE, SPRING_TOLERANCE_VELOCITY)
    }

    /// Like [`Self::is_done`], with caller-supplied tolerances (e.g. looser
    /// bounds for a coarse-pixel display where sub-pixel settling is moot).
    pub fn is_done_with_tolerance(&self, distance_tolerance: f32, velocity_tolerance: f32) -> bool {
        (self.value - self.target).abs() <= distance_tolerance
            && self.velocity.abs() <= velocity_tolerance
    }

    /// The spring iOS/Flutter's `BouncingScrollPhysics` uses to snap an
    /// overscrolled, stationary scroll view back to its boundary: an
    /// intentionally overdamped spring (roots `r1 = -lambda`,
    /// `r2 = -100000 * lambda`, `lambda = ln(2) / 0.07`) tuned so the
    /// far-negative root decays away almost instantly, collapsing the
    /// visible motion to a clean exponential with a ~0.07s half-life
    /// instead of a bouncy spring feel. See
    /// `crate::state::ScrollState::scroll_by_rubber_band`.
    pub fn rubber_band_snap_back(value: f32, target: f32) -> Self {
        const HALF_LIFE_S: f32 = 0.07;
        let lambda = core::f32::consts::LN_2 / HALF_LIFE_S;
        Self {
            value,
            velocity: 0.0,
            target,
            stiffness: 1.0e5 * lambda * lambda,
            damping: (1.0e5 + 1.0) * lambda,
        }
    }
}

/// Quadratic rubber-band friction factor (iOS/Flutter `BouncingScrollPhysics`,
/// `ScrollDecelerationRate.normal`): as `overscroll_fraction` (in `0.0..=1.0`,
/// how far past the edge relative to the viewport) grows, this shrinks
/// toward `0.0`, so further drag delta is scaled down quadratically instead
/// of being hard-clamped — the first pixels of overscroll move almost with
/// the finger, and it gets progressively harder to push further.
pub fn rubber_band_friction_factor(overscroll_fraction: f32) -> f32 {
    let f = overscroll_fraction.clamp(0.0, 1.0);
    (1.0 - f) * (1.0 - f) * 0.52
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_easing_variants_hit_endpoints() {
        let variants = [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::Smoothstep,
            Easing::Smootherstep,
            Easing::Steps(4),
            Easing::InCubic,
            Easing::OutCubic,
            Easing::InOutCubic,
            Easing::InQuart,
            Easing::OutQuart,
            Easing::InOutQuart,
            Easing::InQuint,
            Easing::OutQuint,
            Easing::InOutQuint,
            Easing::InSine,
            Easing::OutSine,
            Easing::InOutSine,
            Easing::InExpo,
            Easing::OutExpo,
            Easing::InOutExpo,
            Easing::InCirc,
            Easing::OutCirc,
            Easing::InOutCirc,
            Easing::InBack,
            Easing::OutBack,
            Easing::InOutBack,
            Easing::InBounce,
            Easing::OutBounce,
            Easing::InOutBounce,
            Easing::InElastic,
            Easing::OutElastic,
            Easing::InOutElastic,
            Easing::Moook,
        ];

        for easing in variants {
            let t0 = apply_easing(0.0, easing);
            let t1 = apply_easing(1.0, easing);
            assert!(
                (t0 - 0.0).abs() < 1e-5,
                "{easing:?} should start at 0, got {t0}"
            );
            assert!(
                (t1 - 1.0).abs() < 1e-5,
                "{easing:?} should end at 1, got {t1}"
            );
            let mid = apply_easing(0.5, easing);
            assert!(mid.is_finite(), "{easing:?} should stay finite");
        }
    }

    #[test]
    fn timer_once_repeating_and_tween_value() {
        let mut timer = Timer::new(100);
        assert!(!timer.tick(50));
        assert!(!timer.tick(49));
        assert!(timer.tick(1));
        assert_eq!(timer.progress(), 1.0);
        timer.reset();
        assert_eq!(timer.progress(), 0.0);

        let mut repeat = Timer::repeating(10);
        assert!(repeat.tick(10));
        assert_eq!(repeat.elapsed_ms, 0);
        assert!(repeat.tick(25));
        assert_eq!(repeat.elapsed_ms, 5);

        let mut tween = Tween::new(0.0, 100.0, 100, Easing::Linear);
        assert!(!tween.tick(50));
        assert_eq!(tween.value(), 50.0);
        assert!(tween.tick(50));
        assert_eq!(tween.value(), 100.0);
        assert!(tween.is_done());
    }

    fn noop_curve(t: f32) -> f32 {
        t
    }

    fn noop_interpolator(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    #[test]
    fn animation_custom_builders_and_repeat_flow() {
        let mut anim = Animation::new(0.0, 10.0, 100, Easing::Linear)
            .with_delay(50)
            .with_repeat_mode(RepeatMode::Loop)
            .with_repeat_count(Some(3))
            .with_custom_curve(noop_curve)
            .with_custom_interpolator(noop_interpolator);
        anim.set_reversed(true);
        anim.reset();
        anim.clear_custom_curve();
        anim.clear_custom_interpolator();
        anim.set_elapsed(25);
        assert_eq!(anim.value(), 10.0);
        anim.set_elapsed(400);
        assert!(anim.is_done());
        anim.reset();
        assert!(!anim.is_done());
        anim.notify_stopped(true);
        assert_eq!(Animation::duration_from_speed(10.0, 100.0), 100);
        assert_eq!(Animation::duration_from_speed(-1.0, 100.0), 0);
        let single = Animation::new(0.0, 1.0, 100, Easing::Linear);
        assert_eq!(single.total_duration_ms(true, true), Some(100));
        let looping = Animation::new(0.0, 1.0, 100, Easing::Linear)
            .with_repeat_mode(RepeatMode::Loop)
            .with_repeat_count(Some(4));
        assert_eq!(looping.total_duration_ms(false, true), Some(400));
        assert_eq!(looping.total_duration_ms(false, false), Some(100));
    }

    #[test]
    fn animation_manager_lifecycle_and_callbacks() {
        static STARTED: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        static REPEATS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        static DONE: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

        fn on_start(_: AnimationId) {
            STARTED.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        }
        fn on_repeat(_: AnimationId, _: u16) {
            REPEATS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        }
        fn on_complete(_: AnimationId, _: bool) {
            DONE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        }

        let mut manager = AnimationManager::<2>::new();
        manager.set_callbacks(AnimationManagerCallbacks {
            on_start: Some(on_start),
            on_repeat: Some(on_repeat),
            on_complete: Some(on_complete),
        });
        let id = manager
            .start(Animation::new(0.0, 1.0, 100, Easing::Linear))
            .unwrap();
        let _ = manager
            .start(Animation::new(0.0, 1.0, 100, Easing::Linear))
            .unwrap();
        assert_eq!(manager.active_count(), 2);
        assert!(manager.value(id).is_some());
        assert!(manager.animation(id).is_some());
        assert!(manager.animation_mut(id).is_some());

        assert!(matches!(
            manager.start(Animation::new(0.0, 1.0, 100, Easing::Linear)),
            Err(AnimationError::Full)
        ));

        manager.set_paused(true);
        manager.tick(10);
        assert!(manager.is_paused());
        manager.set_paused(false);

        assert!(manager.seek(id, 50));
        assert!(!manager.seek(AnimationId::new(999), 50));
        assert!(manager.seek_stepped(id, 120, 30));
        assert!(!manager.seek_stepped(AnimationId::new(999), 1, 30));
        assert!(manager.replay_stepped(id, 140, 40, |_| {}));
        assert!(!manager.replay_stepped(AnimationId::new(999), 1, 1, |_| {}));

        manager.tick(1000);
        assert_eq!(manager.active_count(), 0);
        assert_eq!(STARTED.load(core::sync::atomic::Ordering::Relaxed), 2);
        assert_eq!(DONE.load(core::sync::atomic::Ordering::Relaxed), 2);

        let id2 = manager
            .start(Animation::new(0.0, 1.0, 100, Easing::Linear))
            .unwrap();
        assert!(manager.stop(id2));
        assert!(!manager.stop(id2));
        assert_eq!(DONE.load(core::sync::atomic::Ordering::Relaxed), 3);

        manager.set_next_id_for_test(7);
        let id3 = manager
            .start(Animation::new(0.0, 1.0, 100, Easing::Linear))
            .unwrap();
        assert_eq!(id3.raw(), 7);
    }

    #[test]
    fn spring_inertia_and_path_animators() {
        let mut spring = SpringAnimator::new(0.0, 10.0);
        let _ = spring.tick(16);
        assert!(spring.value > 0.0);

        let mut inertia = InertiaAnimator::new(0.0, 100.0);
        let _ = inertia.tick(16);
        assert!(inertia.value > 0.0);

        let mut path = PathAnimator::<3>::new(100, Easing::Linear);
        assert!(path.value().is_none());
        path.push_point(PathPoint::new(0.0, 0.0)).unwrap();
        path.push_point(PathPoint::new(10.0, 10.0)).unwrap();
        path.push_point(PathPoint::new(20.0, 0.0)).unwrap();
        assert!(path.push_point(PathPoint::new(30.0, 0.0)).is_err());
        assert!(path.value().is_some());
        assert!(!path.tick(50));
        assert!(path.tick(50));
        assert!(path.value().is_some());
        path.reset();
        assert_eq!(path.timer.elapsed_ms, 0);
    }

    #[test]
    fn spring_settles_and_reports_is_done_underdamped() {
        // Default stiffness/damping (120/16) is underdamped: disc < 0.
        let mut spring = SpringAnimator::new(0.0, 10.0);
        assert!(!spring.is_done());
        for _ in 0..600 {
            spring.tick(16);
            if spring.is_done() {
                break;
            }
        }
        assert!(spring.is_done());
        assert!((spring.value - 10.0).abs() <= SPRING_TOLERANCE_DISTANCE);
        assert!(spring.velocity.abs() <= SPRING_TOLERANCE_VELOCITY);
    }

    #[test]
    fn spring_with_duration_and_bounce_covers_all_branches() {
        // bounce = 0 -> critically damped (disc ~= 0).
        let mut critical = SpringAnimator::with_duration_and_bounce(0.0, 1.0, 300, 0.0);
        // bounce > 0 -> underdamped (oscillates before settling).
        let mut bouncy = SpringAnimator::with_duration_and_bounce(0.0, 1.0, 300, 0.5);
        // bounce < 0 -> overdamped (sluggish, no oscillation).
        let mut sluggish = SpringAnimator::with_duration_and_bounce(0.0, 1.0, 300, -0.5);

        for spring in [&mut critical, &mut bouncy, &mut sluggish] {
            for _ in 0..200 {
                let v = spring.tick(16);
                assert!(v.is_finite());
            }
            assert!(spring.is_done_with_tolerance(1e-2, 1e-2));
        }
    }

    #[test]
    fn rubber_band_friction_factor_shrinks_toward_zero_as_overscroll_grows() {
        let none = rubber_band_friction_factor(0.0);
        let some = rubber_band_friction_factor(0.5);
        let full = rubber_band_friction_factor(1.0);
        assert!((none - 0.52).abs() < 1e-6);
        assert!(some > 0.0 && some < none);
        assert!(full.abs() < 1e-6);
        // Out-of-range fractions are clamped, not negative/blown-up.
        assert_eq!(rubber_band_friction_factor(-1.0), none);
        assert_eq!(rubber_band_friction_factor(2.0), full);
    }

    #[test]
    fn rubber_band_snap_back_is_overdamped_and_settles_without_overshoot() {
        let mut spring = SpringAnimator::rubber_band_snap_back(-20.0, 0.0);
        // Deliberately overdamped: damping^2 must dwarf 4*stiffness.
        assert!(spring.damping * spring.damping > 4.0 * spring.stiffness);

        let mut last = spring.value;
        for _ in 0..200 {
            let v = spring.tick(16);
            assert!(v.is_finite());
            // Monotonic approach to the target: an overdamped/critical
            // spring never overshoots or oscillates past it.
            assert!(
                v >= last && v <= 0.0,
                "overshot or reversed: {v} after {last}"
            );
            last = v;
            if spring.is_done() {
                break;
            }
        }
        assert!(spring.is_done(), "must settle at the boundary");
        assert!((spring.value - 0.0).abs() <= SPRING_TOLERANCE_DISTANCE);
    }

    #[test]
    fn spring_tick_is_stable_for_large_dt() {
        let mut spring = SpringAnimator::new(0.0, 10.0);
        // A single large step (e.g. after a stall) must not diverge, unlike
        // semi-implicit Euler integration at the same stiffness/damping.
        let v = spring.tick(2000);
        assert!(v.is_finite());
        assert!(spring.velocity.is_finite());
    }
}
