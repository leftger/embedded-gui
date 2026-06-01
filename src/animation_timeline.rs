use crate::animation::{Animation, AnimationError, AnimationId, AnimationManager};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineError {
    Full,
    Empty,
    Animation(AnimationError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionMode {
    Sequence,
    Spawn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositionControls {
    pub start_delay_ms: u32,
    pub repeat_count: Option<u16>,
    pub reverse: bool,
}

impl Default for CompositionControls {
    fn default() -> Self {
        Self {
            start_delay_ms: 0,
            repeat_count: Some(1),
            reverse: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimelineStep {
    Delay { duration_ms: u32 },
    Animate { animation: Animation },
    Label { id: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe {
    pub value: f32,
    pub duration_ms: u32,
    pub easing: crate::Easing,
}

#[derive(Clone, Copy, Debug)]
pub struct KeyframeTrack<const N: usize> {
    keyframes: [Option<Keyframe>; N],
    len: usize,
    index: usize,
    current: Option<Animation>,
    current_from: f32,
    done: bool,
    callbacks: KeyframeTrackCallbacks,
}

impl<const N: usize> KeyframeTrack<N> {
    pub const fn new() -> Self {
        Self {
            keyframes: [None; N],
            len: 0,
            index: 0,
            current: None,
            current_from: 0.0,
            done: false,
            callbacks: KeyframeTrackCallbacks {
                on_segment_start: None,
                on_segment_complete: None,
            },
        }
    }

    pub fn push(&mut self, keyframe: Keyframe) -> Result<(), TimelineError> {
        if self.len >= N {
            return Err(TimelineError::Full);
        }
        self.keyframes[self.len] = Some(keyframe);
        self.len += 1;
        Ok(())
    }

    pub fn reset(&mut self, start: f32) {
        self.index = 0;
        self.current = None;
        self.current_from = start;
        self.done = false;
    }

    pub fn set_callbacks(&mut self, callbacks: KeyframeTrackCallbacks) {
        self.callbacks = callbacks;
    }

    pub fn tick(&mut self, dt_ms: u32) -> Result<(), TimelineError> {
        if self.done {
            return Ok(());
        }
        if self.len == 0 {
            self.done = true;
            return Err(TimelineError::Empty);
        }

        if self.current.is_none() {
            let Some(kf) = self.keyframes[self.index] else {
                self.done = true;
                return Ok(());
            };
            if let Some(cb) = self.callbacks.on_segment_start {
                cb(self.index, self.current_from, kf.value);
            }
            self.current = Some(Animation::new(
                self.current_from,
                kf.value,
                kf.duration_ms,
                kf.easing,
            ));
        }

        let anim = self.current.as_mut().expect("animation exists");
        anim.tick(dt_ms);
        if anim.is_done() {
            self.current_from = anim.value();
            if let Some(cb) = self.callbacks.on_segment_complete {
                cb(self.index, self.current_from);
            }
            self.current = None;
            self.index += 1;
            if self.index >= self.len {
                self.done = true;
            }
        }
        Ok(())
    }

    pub fn value(&self) -> Option<f32> {
        if let Some(anim) = self.current {
            Some(anim.value())
        } else if self.done {
            Some(self.current_from)
        } else {
            None
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }
}

impl<const N: usize> Default for KeyframeTrack<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeyframeTrackCallbacks {
    pub on_segment_start: Option<fn(usize, f32, f32)>,
    pub on_segment_complete: Option<fn(usize, f32)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SequencePlayerStatus {
    pub step_idx: usize,
    pub active: bool,
    pub done: bool,
}

impl<const TRACKS: usize, const STEPS: usize> SequencePlayer<TRACKS, STEPS> {
    pub fn status(&self) -> SequencePlayerStatus {
        SequencePlayerStatus {
            step_idx: self.step_idx,
            active: self.active.is_some(),
            done: self.done,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationSequence<const STEPS: usize> {
    steps: [Option<TimelineStep>; STEPS],
    len: usize,
}

impl<const STEPS: usize> AnimationSequence<STEPS> {
    pub const fn new() -> Self {
        Self {
            steps: [None; STEPS],
            len: 0,
        }
    }

    pub fn push_delay(&mut self, duration_ms: u32) -> Result<(), TimelineError> {
        self.push_step(TimelineStep::Delay { duration_ms })
    }

    pub fn push_animation(&mut self, animation: Animation) -> Result<(), TimelineError> {
        self.push_step(TimelineStep::Animate { animation })
    }

    pub fn push_step(&mut self, step: TimelineStep) -> Result<(), TimelineError> {
        if self.len >= STEPS {
            return Err(TimelineError::Full);
        }
        self.steps[self.len] = Some(step);
        self.len += 1;
        Ok(())
    }

    pub fn push_label(&mut self, id: u8) -> Result<(), TimelineError> {
        self.push_step(TimelineStep::Label { id })
    }

    pub fn find_label(&self, id: u8) -> Option<usize> {
        self.steps
            .iter()
            .take(self.len)
            .position(|step| matches!(step, Some(TimelineStep::Label { id: x }) if *x == id))
    }
}

impl<const STEPS: usize> Default for AnimationSequence<STEPS> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SequencePlayer<const TRACKS: usize, const STEPS: usize> {
    manager: AnimationManager<TRACKS>,
    sequence: AnimationSequence<STEPS>,
    step_idx: usize,
    delay_elapsed_ms: u32,
    active: Option<AnimationId>,
    repeat_mode: SequenceRepeatMode,
    done: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SequenceRepeatMode {
    #[default]
    Once,
    Loop,
}

impl<const TRACKS: usize, const STEPS: usize> SequencePlayer<TRACKS, STEPS> {
    pub const fn new(sequence: AnimationSequence<STEPS>) -> Self {
        Self {
            manager: AnimationManager::new(),
            sequence,
            step_idx: 0,
            delay_elapsed_ms: 0,
            active: None,
            repeat_mode: SequenceRepeatMode::Once,
            done: false,
        }
    }

    pub fn set_repeat_mode(&mut self, mode: SequenceRepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn tick(&mut self, dt_ms: u32) -> Result<(), TimelineError> {
        if self.done {
            return Ok(());
        }
        if self.sequence.len == 0 {
            self.done = true;
            return Err(TimelineError::Empty);
        }

        if let Some(id) = self.active {
            self.manager.tick(dt_ms);
            if self.manager.animation(id).is_none() {
                self.active = None;
                self.step_idx += 1;
            }
            if self.step_idx >= self.sequence.len {
                if self.repeat_mode == SequenceRepeatMode::Loop {
                    self.step_idx = 0;
                    self.done = false;
                } else {
                    self.done = true;
                }
            }
            return Ok(());
        }

        let Some(step) = self.sequence.steps[self.step_idx] else {
            self.done = true;
            return Ok(());
        };
        match step {
            TimelineStep::Label { .. } => {
                self.step_idx += 1;
            }
            TimelineStep::Delay { duration_ms } => {
                self.delay_elapsed_ms = self.delay_elapsed_ms.saturating_add(dt_ms);
                if self.delay_elapsed_ms >= duration_ms {
                    self.delay_elapsed_ms = 0;
                    self.step_idx += 1;
                }
            }
            TimelineStep::Animate { animation } => {
                let id = self
                    .manager
                    .start(animation)
                    .map_err(TimelineError::Animation)?;
                self.active = Some(id);
            }
        }
        if self.step_idx >= self.sequence.len {
            if self.repeat_mode == SequenceRepeatMode::Loop {
                self.step_idx = 0;
            } else {
                self.done = true;
            }
        }
        Ok(())
    }

    pub fn active_value(&self) -> Option<f32> {
        self.active.and_then(|id| self.manager.value(id))
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn seek_to_label(&mut self, id: u8) -> Result<(), TimelineError> {
        let Some(idx) = self.sequence.find_label(id) else {
            return Err(TimelineError::Empty);
        };
        self.step_idx = idx.saturating_add(1);
        self.delay_elapsed_ms = 0;
        self.active = None;
        self.done = false;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationGroup<const N: usize> {
    tracks: [Option<Animation>; N],
    len: usize,
}

impl<const N: usize> AnimationGroup<N> {
    pub const fn new() -> Self {
        Self {
            tracks: [None; N],
            len: 0,
        }
    }

    pub fn push(&mut self, animation: Animation) -> Result<(), TimelineError> {
        if self.len >= N {
            return Err(TimelineError::Full);
        }
        self.tracks[self.len] = Some(animation);
        self.len += 1;
        Ok(())
    }

    pub fn start<const TRACKS: usize>(
        self,
        manager: &mut AnimationManager<TRACKS>,
    ) -> Result<[Option<AnimationId>; N], TimelineError> {
        let mut ids = [None; N];
        for (idx, track) in self.tracks.iter().enumerate().take(self.len) {
            if let Some(anim) = track {
                ids[idx] = Some(manager.start(*anim).map_err(TimelineError::Animation)?);
            }
        }
        Ok(ids)
    }
}

impl<const N: usize> Default for AnimationGroup<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComposedAnimation<const N: usize> {
    mode: CompositionMode,
    tracks: [Option<Animation>; N],
    len: usize,
    controls: CompositionControls,
}

impl<const N: usize> ComposedAnimation<N> {
    pub const fn new(mode: CompositionMode) -> Self {
        Self {
            mode,
            tracks: [None; N],
            len: 0,
            controls: CompositionControls {
                start_delay_ms: 0,
                repeat_count: Some(1),
                reverse: false,
            },
        }
    }

    pub fn push(&mut self, animation: Animation) -> Result<(), TimelineError> {
        if self.len >= N {
            return Err(TimelineError::Full);
        }
        self.tracks[self.len] = Some(animation);
        self.len += 1;
        Ok(())
    }

    pub fn with_controls(mut self, controls: CompositionControls) -> Self {
        self.controls = controls;
        self
    }
}

impl<const N: usize> Default for ComposedAnimation<N> {
    fn default() -> Self {
        Self::new(CompositionMode::Sequence)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComposedAnimationStatus {
    pub cycle: u16,
    pub active: bool,
    pub done: bool,
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ComposedAnimationCallbacks {
    pub on_cycle_start: Option<fn(u16)>,
    pub on_cycle_complete: Option<fn(u16)>,
    pub on_done: Option<fn()>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComposedAnimationPlayer<const TRACKS: usize, const N: usize> {
    manager: AnimationManager<TRACKS>,
    composition: ComposedAnimation<N>,
    ids: [Option<AnimationId>; N],
    delay_elapsed_ms: u32,
    started: bool,
    cycles_completed: u16,
    done: bool,
    paused: bool,
    callbacks: ComposedAnimationCallbacks,
}

impl<const TRACKS: usize, const N: usize> ComposedAnimationPlayer<TRACKS, N> {
    pub const fn new(composition: ComposedAnimation<N>) -> Self {
        Self {
            manager: AnimationManager::new(),
            composition,
            ids: [None; N],
            delay_elapsed_ms: 0,
            started: false,
            cycles_completed: 0,
            done: false,
            paused: false,
            callbacks: ComposedAnimationCallbacks {
                on_cycle_start: None,
                on_cycle_complete: None,
                on_done: None,
            },
        }
    }

    pub fn set_callbacks(&mut self, callbacks: ComposedAnimationCallbacks) {
        self.callbacks = callbacks;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn restart(&mut self) {
        self.ids = [None; N];
        self.delay_elapsed_ms = 0;
        self.started = false;
        self.cycles_completed = 0;
        self.done = false;
    }

    pub fn stop(&mut self) {
        for id in self.ids.iter().flatten().copied() {
            let _ = self.manager.stop(id);
        }
        self.ids = [None; N];
        self.done = true;
    }

    pub fn seek_active_stepped(&mut self, elapsed_ms: u32, step_ms: u32) -> bool {
        let mut any = false;
        for id in self.ids.iter().flatten().copied() {
            any |= self.manager.seek_stepped(id, elapsed_ms, step_ms);
        }
        any
    }

    pub fn tick(&mut self, dt_ms: u32) -> Result<(), TimelineError> {
        if self.done {
            return Ok(());
        }
        if self.paused {
            return Ok(());
        }
        if self.composition.len == 0 {
            self.done = true;
            return Err(TimelineError::Empty);
        }

        if !self.started {
            self.delay_elapsed_ms = self.delay_elapsed_ms.saturating_add(dt_ms);
            if self.delay_elapsed_ms < self.composition.controls.start_delay_ms {
                return Ok(());
            }
            self.delay_elapsed_ms = 0;
            self.schedule_cycle()?;
            self.started = true;
            if let Some(cb) = self.callbacks.on_cycle_start {
                cb(self.cycles_completed);
            }
            return Ok(());
        }

        self.manager.tick(dt_ms);
        let any_active = self
            .ids
            .iter()
            .flatten()
            .any(|id| self.manager.animation(*id).is_some());
        if any_active {
            return Ok(());
        }

        self.cycles_completed = self.cycles_completed.saturating_add(1);
        if let Some(cb) = self.callbacks.on_cycle_complete {
            cb(self.cycles_completed);
        }
        if self
            .composition
            .controls
            .repeat_count
            .is_some_and(|count| self.cycles_completed >= count)
        {
            self.done = true;
            if let Some(cb) = self.callbacks.on_done {
                cb();
            }
            return Ok(());
        }

        self.ids = [None; N];
        self.started = false;
        Ok(())
    }

    pub fn status(&self) -> ComposedAnimationStatus {
        ComposedAnimationStatus {
            cycle: self.cycles_completed,
            active: self
                .ids
                .iter()
                .flatten()
                .any(|id| self.manager.animation(*id).is_some()),
            done: self.done,
        }
    }

    fn schedule_cycle(&mut self) -> Result<(), TimelineError> {
        let mut cumulative_delay = 0u32;
        for idx in 0..self.composition.len {
            let src_idx = if self.composition.controls.reverse {
                self.composition.len - 1 - idx
            } else {
                idx
            };
            let Some(mut anim) = self.composition.tracks[src_idx] else {
                continue;
            };
            if self.composition.controls.reverse {
                anim.set_reversed(true);
            }
            let delay = match self.composition.mode {
                CompositionMode::Spawn => 0,
                CompositionMode::Sequence => cumulative_delay,
            };
            anim = anim.with_delay(anim.delay_ms.saturating_add(delay));
            let id = self
                .manager
                .start(anim)
                .map_err(TimelineError::Animation)?;
            self.ids[idx] = Some(id);
            if self.composition.mode == CompositionMode::Sequence {
                let segment = anim.total_duration_ms(true, false).unwrap_or(anim.duration_ms);
                cumulative_delay = cumulative_delay.saturating_add(segment);
            }
        }
        Ok(())
    }
}
