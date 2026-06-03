//! Widget-level animation bindings built on top of [`AnimationManager`].
//! Keeps fixed-capacity, no-allocation behavior suitable for embedded targets.

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use heapless::Vec;

use crate::{
    animation::{Animation, AnimationError, AnimationId, AnimationManager, Easing, PathPoint},
    context::{GuiContext, GuiError},
    math::F32Ext as _,
    widget::WidgetId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetAnimationError {
    AnimationsFull,
    BindingsFull,
    ConflictIgnored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimatedProperty {
    Progress,
    Meter,
    SliderValue,
    ScrollOffsetY,
    TabSelected,
    DropdownSelected,
    RollerSelected,
    GaugeValue,
    SpinnerPhase,
    CornerRadius,
    AccentR,
    AccentG,
    AccentB,
    WidgetX,
    WidgetY,
    WidgetWidth,
    WidgetHeight,
    Opacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetKeyframeState {
    pub x: i32,
    pub y: i32,
    pub opacity: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetPropertyKeyframe {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub opacity: Option<u8>,
    pub duration_ms: u32,
    pub easing: Easing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnimationConflictPolicy {
    #[default]
    Replace,
    Ignore,
    Queue,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WidgetAnimationCallbacks {
    pub on_start: Option<fn(AnimationId, WidgetId, AnimatedProperty)>,
    pub on_repeat: Option<fn(AnimationId, WidgetId, AnimatedProperty)>,
    pub on_complete: Option<fn(AnimationId, WidgetId, AnimatedProperty)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Binding {
    animation_id: AnimationId,
    widget_id: WidgetId,
    property: AnimatedProperty,
    last_iteration: u16,
    queued: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingSnapshot {
    pub animation_id: AnimationId,
    pub widget_id: WidgetId,
    pub property: AnimatedProperty,
    pub queued: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct WidgetAnimator<const TRACKS: usize, const BINDINGS: usize> {
    animations: AnimationManager<TRACKS>,
    bindings: [Option<Binding>; BINDINGS],
    callbacks: WidgetAnimationCallbacks,
}

impl<const TRACKS: usize, const BINDINGS: usize> Default for WidgetAnimator<TRACKS, BINDINGS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const TRACKS: usize, const BINDINGS: usize> WidgetAnimator<TRACKS, BINDINGS> {
    pub const fn new() -> Self {
        Self {
            animations: AnimationManager::new(),
            bindings: [None; BINDINGS],
            callbacks: WidgetAnimationCallbacks {
                on_start: None,
                on_repeat: None,
                on_complete: None,
            },
        }
    }

    pub fn set_callbacks(&mut self, callbacks: WidgetAnimationCallbacks) {
        self.callbacks = callbacks;
    }

    pub fn animate_progress(
        &mut self,
        widget_id: WidgetId,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property(
            widget_id,
            AnimatedProperty::Progress,
            Animation::new(from, to, duration_ms, easing),
        )
    }

    pub fn animate_meter(
        &mut self,
        widget_id: WidgetId,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property(
            widget_id,
            AnimatedProperty::Meter,
            Animation::new(from, to, duration_ms, easing),
        )
    }

    pub fn animate_slider_value(
        &mut self,
        widget_id: WidgetId,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.animate_slider_value_with_policy(
            widget_id,
            from,
            to,
            duration_ms,
            easing,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn animate_slider_value_with_policy(
        &mut self,
        widget_id: WidgetId,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: Easing,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::SliderValue,
            Animation::new(from, to, duration_ms, easing),
            policy,
        )
    }

    pub fn animate_scroll_offset_y(
        &mut self,
        widget_id: WidgetId,
        from: i32,
        to: i32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.animate_scroll_offset_y_with_policy(
            widget_id,
            from,
            to,
            duration_ms,
            easing,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn animate_scroll_offset_y_with_policy(
        &mut self,
        widget_id: WidgetId,
        from: i32,
        to: i32,
        duration_ms: u32,
        easing: Easing,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::ScrollOffsetY,
            Animation::new(from as f32, to as f32, duration_ms, easing),
            policy,
        )
    }

    pub fn animate_tab_selected(
        &mut self,
        widget_id: WidgetId,
        from: usize,
        to: usize,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property(
            widget_id,
            AnimatedProperty::TabSelected,
            Animation::new(from as f32, to as f32, duration_ms, easing),
        )
    }

    pub fn animate_dropdown_selected(
        &mut self,
        widget_id: WidgetId,
        from: usize,
        to: usize,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property(
            widget_id,
            AnimatedProperty::DropdownSelected,
            Animation::new(from as f32, to as f32, duration_ms, easing),
        )
    }

    pub fn animate_roller_selected(
        &mut self,
        widget_id: WidgetId,
        from: usize,
        to: usize,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property(
            widget_id,
            AnimatedProperty::RollerSelected,
            Animation::new(from as f32, to as f32, duration_ms, easing),
        )
    }

    pub fn animate_gauge_value(
        &mut self,
        widget_id: WidgetId,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property(
            widget_id,
            AnimatedProperty::GaugeValue,
            Animation::new(from, to, duration_ms, easing),
        )
    }

    pub fn animate_spinner_phase(
        &mut self,
        widget_id: WidgetId,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property(
            widget_id,
            AnimatedProperty::SpinnerPhase,
            Animation::new(from, to, duration_ms, easing),
        )
    }

    pub fn animate_widget_x(
        &mut self,
        widget_id: WidgetId,
        from: i32,
        to: i32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.animate_widget_x_with_policy(
            widget_id,
            from,
            to,
            duration_ms,
            easing,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn animate_widget_x_with_policy(
        &mut self,
        widget_id: WidgetId,
        from: i32,
        to: i32,
        duration_ms: u32,
        easing: Easing,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetX,
            Animation::new(from as f32, to as f32, duration_ms, easing),
            policy,
        )
    }

    pub fn animate_widget_x_with_custom_interpolator(
        &mut self,
        widget_id: WidgetId,
        from: i32,
        to: i32,
        duration_ms: u32,
        easing: Easing,
        interpolator: fn(f32, f32, f32) -> f32,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        let animation = Animation::new(from as f32, to as f32, duration_ms, easing)
            .with_custom_interpolator(interpolator);
        self.bind_property_with_policy(widget_id, AnimatedProperty::WidgetX, animation, policy)
    }

    pub fn animate_widget_y(
        &mut self,
        widget_id: WidgetId,
        from: i32,
        to: i32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.animate_widget_y_with_policy(
            widget_id,
            from,
            to,
            duration_ms,
            easing,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn animate_widget_y_with_policy(
        &mut self,
        widget_id: WidgetId,
        from: i32,
        to: i32,
        duration_ms: u32,
        easing: Easing,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetY,
            Animation::new(from as f32, to as f32, duration_ms, easing),
            policy,
        )
    }

    pub fn animate_widget_y_with_custom_curve(
        &mut self,
        widget_id: WidgetId,
        from: i32,
        to: i32,
        duration_ms: u32,
        easing: Easing,
        curve: fn(f32) -> f32,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        let animation =
            Animation::new(from as f32, to as f32, duration_ms, easing).with_custom_curve(curve);
        self.bind_property_with_policy(widget_id, AnimatedProperty::WidgetY, animation, policy)
    }

    pub fn animate_widget_width(
        &mut self,
        widget_id: WidgetId,
        from: u32,
        to: u32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.animate_widget_width_with_policy(
            widget_id,
            from,
            to,
            duration_ms,
            easing,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn animate_widget_width_with_policy(
        &mut self,
        widget_id: WidgetId,
        from: u32,
        to: u32,
        duration_ms: u32,
        easing: Easing,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetWidth,
            Animation::new(from as f32, to as f32, duration_ms, easing),
            policy,
        )
    }

    pub fn animate_widget_height(
        &mut self,
        widget_id: WidgetId,
        from: u32,
        to: u32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.animate_widget_height_with_policy(
            widget_id,
            from,
            to,
            duration_ms,
            easing,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn animate_widget_height_with_policy(
        &mut self,
        widget_id: WidgetId,
        from: u32,
        to: u32,
        duration_ms: u32,
        easing: Easing,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetHeight,
            Animation::new(from as f32, to as f32, duration_ms, easing),
            policy,
        )
    }

    pub fn animate_opacity(
        &mut self,
        widget_id: WidgetId,
        from: u8,
        to: u8,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.animate_opacity_with_policy(
            widget_id,
            from,
            to,
            duration_ms,
            easing,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn animate_opacity_with_policy(
        &mut self,
        widget_id: WidgetId,
        from: u8,
        to: u8,
        duration_ms: u32,
        easing: Easing,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::Opacity,
            Animation::new(from as f32, to as f32, duration_ms, easing),
            policy,
        )
    }

    pub fn animate_opacity_with_custom_interpolator(
        &mut self,
        widget_id: WidgetId,
        from: u8,
        to: u8,
        duration_ms: u32,
        easing: Easing,
        interpolator: fn(f32, f32, f32) -> f32,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        let animation = Animation::new(from as f32, to as f32, duration_ms, easing)
            .with_custom_interpolator(interpolator);
        self.bind_property_with_policy(widget_id, AnimatedProperty::Opacity, animation, policy)
    }

    pub fn animate_widget_keyframes(
        &mut self,
        widget_id: WidgetId,
        initial: WidgetKeyframeState,
        keyframes: &[WidgetPropertyKeyframe],
        policy: AnimationConflictPolicy,
    ) -> Result<usize, WidgetAnimationError> {
        let mut created = 0usize;
        let mut delay_ms = 0u32;
        let mut state = initial;
        for (idx, keyframe) in keyframes.iter().copied().enumerate() {
            let step_policy = if idx == 0 {
                policy
            } else {
                AnimationConflictPolicy::Queue
            };
            if let Some(next_x) = keyframe.x {
                let anim = Animation::new(
                    state.x as f32,
                    next_x as f32,
                    keyframe.duration_ms,
                    keyframe.easing,
                )
                .with_delay(delay_ms);
                self.bind_property_with_policy(
                    widget_id,
                    AnimatedProperty::WidgetX,
                    anim,
                    step_policy,
                )?;
                created += 1;
                state.x = next_x;
            }
            if let Some(next_y) = keyframe.y {
                let anim = Animation::new(
                    state.y as f32,
                    next_y as f32,
                    keyframe.duration_ms,
                    keyframe.easing,
                )
                .with_delay(delay_ms);
                self.bind_property_with_policy(
                    widget_id,
                    AnimatedProperty::WidgetY,
                    anim,
                    step_policy,
                )?;
                created += 1;
                state.y = next_y;
            }
            if let Some(next_opacity) = keyframe.opacity {
                let anim = Animation::new(
                    state.opacity as f32,
                    next_opacity as f32,
                    keyframe.duration_ms,
                    keyframe.easing,
                )
                .with_delay(delay_ms);
                self.bind_property_with_policy(
                    widget_id,
                    AnimatedProperty::Opacity,
                    anim,
                    step_policy,
                )?;
                created += 1;
                state.opacity = next_opacity;
            }
            delay_ms = delay_ms.saturating_add(keyframe.duration_ms);
        }
        Ok(created)
    }

    pub fn pulse_opacity(
        &mut self,
        widget_id: WidgetId,
        low: u8,
        high: u8,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        let animation = Animation::new(low as f32, high as f32, duration_ms, easing)
            .with_repeat_mode(crate::animation::RepeatMode::PingPong)
            .with_repeat_count(None);
        self.bind_property(widget_id, AnimatedProperty::Opacity, animation)
    }

    pub fn ping_pong_progress(
        &mut self,
        widget_id: WidgetId,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        let animation = Animation::new(from, to, duration_ms, easing)
            .with_repeat_mode(crate::animation::RepeatMode::PingPong)
            .with_repeat_count(None);
        self.bind_property(widget_id, AnimatedProperty::Progress, animation)
    }

    pub fn animate_corner_radius(
        &mut self,
        widget_id: WidgetId,
        from: u8,
        to: u8,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property(
            widget_id,
            AnimatedProperty::CornerRadius,
            Animation::new(from as f32, to as f32, duration_ms, easing),
        )
    }

    pub fn animate_accent_color(
        &mut self,
        widget_id: WidgetId,
        from: Rgb565,
        to: Rgb565,
        duration_ms: u32,
        easing: Easing,
    ) -> Result<[AnimationId; 3], WidgetAnimationError> {
        let mut ids = [AnimationId::new(0); 3];
        let mut started = Vec::<AnimationId, 3>::new();
        let r = self.bind_property(
            widget_id,
            AnimatedProperty::AccentR,
            Animation::new(from.r() as f32, to.r() as f32, duration_ms, easing),
        )?;
        ids[0] = r;
        let _ = started.push(r);
        let g = match self.bind_property(
            widget_id,
            AnimatedProperty::AccentG,
            Animation::new(from.g() as f32, to.g() as f32, duration_ms, easing),
        ) {
            Ok(v) => v,
            Err(err) => {
                for id in started {
                    let _ = self.stop(id);
                }
                return Err(err);
            }
        };
        ids[1] = g;
        let _ = started.push(g);
        let b = match self.bind_property(
            widget_id,
            AnimatedProperty::AccentB,
            Animation::new(from.b() as f32, to.b() as f32, duration_ms, easing),
        ) {
            Ok(v) => v,
            Err(err) => {
                for id in started {
                    let _ = self.stop(id);
                }
                return Err(err);
            }
        };
        ids[2] = b;
        Ok(ids)
    }

    pub fn animate_widget_path(
        &mut self,
        widget_id: WidgetId,
        points: &[PathPoint],
        duration_ms: u32,
        easing: Easing,
    ) -> Result<(AnimationId, AnimationId), WidgetAnimationError> {
        self.animate_widget_path_with_policy(
            widget_id,
            points,
            duration_ms,
            easing,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn animate_widget_path_with_policy(
        &mut self,
        widget_id: WidgetId,
        points: &[PathPoint],
        duration_ms: u32,
        easing: Easing,
        policy: AnimationConflictPolicy,
    ) -> Result<(AnimationId, AnimationId), WidgetAnimationError> {
        if points.len() < 2 {
            return Err(WidgetAnimationError::ConflictIgnored);
        }
        let segs = (points.len() - 1) as u32;
        let seg_duration = (duration_ms / segs).max(1);
        let mut ids = Vec::<AnimationId, BINDINGS>::new();
        let mut first_x = AnimationId::new(0);
        let mut first_y = AnimationId::new(0);

        for i in 0..(points.len() - 1) {
            let from = points[i];
            let to = points[i + 1];
            let delay = seg_duration.saturating_mul(i as u32);
            let x_anim = Animation::new(from.x, to.x, seg_duration, easing).with_delay(delay);
            let y_anim = Animation::new(from.y, to.y, seg_duration, easing).with_delay(delay);
            let step_policy = if i == 0 {
                policy
            } else {
                AnimationConflictPolicy::Queue
            };
            let x_id = match self.bind_property_with_policy(
                widget_id,
                AnimatedProperty::WidgetX,
                x_anim,
                step_policy,
            ) {
                Ok(id) => id,
                Err(err) => {
                    for id in ids {
                        let _ = self.stop(id);
                    }
                    return Err(err);
                }
            };
            let _ = ids.push(x_id);
            let y_id = match self.bind_property_with_policy(
                widget_id,
                AnimatedProperty::WidgetY,
                y_anim,
                step_policy,
            ) {
                Ok(id) => id,
                Err(err) => {
                    for id in ids {
                        let _ = self.stop(id);
                    }
                    return Err(err);
                }
            };
            let _ = ids.push(y_id);
            if i == 0 {
                first_x = x_id;
                first_y = y_id;
            }
        }
        Ok((first_x, first_y))
    }

    pub fn stagger_widget_x(
        &mut self,
        widget_ids: &[WidgetId],
        from: i32,
        to: i32,
        duration_ms: u32,
        stagger_ms: u32,
        easing: Easing,
    ) -> Result<usize, WidgetAnimationError> {
        let mut created = 0usize;
        let mut started = Vec::<AnimationId, BINDINGS>::new();
        for (idx, id) in widget_ids.iter().copied().enumerate() {
            let delay = stagger_ms.saturating_mul(idx as u32);
            let animation =
                Animation::new(from as f32, to as f32, duration_ms, easing).with_delay(delay);
            match self.bind_property_with_policy(
                id,
                AnimatedProperty::WidgetX,
                animation,
                AnimationConflictPolicy::Replace,
            ) {
                Ok(track) => {
                    let _ = started.push(track);
                    created += 1;
                }
                Err(err) => {
                    for track in started {
                        let _ = self.stop(track);
                    }
                    return Err(err);
                }
            }
        }
        Ok(created)
    }

    pub fn preset_fade_in_up(
        &mut self,
        widget_id: WidgetId,
        from_y: i32,
        to_y: i32,
        duration_ms: u32,
    ) -> Result<(AnimationId, AnimationId), WidgetAnimationError> {
        let y = self.animate_widget_y(widget_id, from_y, to_y, duration_ms, Easing::OutCubic)?;
        let alpha = self.animate_opacity(widget_id, 0, 255, duration_ms, Easing::OutSine)?;
        Ok((y, alpha))
    }

    pub fn preset_attention_shake(
        &mut self,
        widget_id: WidgetId,
        base_x: i32,
        amplitude: i32,
        duration_ms: u32,
    ) -> Result<(AnimationId, AnimationId), WidgetAnimationError> {
        let step = (duration_ms / 3).max(1);
        let a = self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetX,
            Animation::new(
                base_x as f32,
                (base_x + amplitude) as f32,
                step,
                Easing::InOutSine,
            ),
            AnimationConflictPolicy::Replace,
        )?;
        let b = self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetX,
            Animation::new(
                (base_x + amplitude) as f32,
                (base_x - amplitude) as f32,
                step,
                Easing::InOutSine,
            )
            .with_delay(step),
            AnimationConflictPolicy::Queue,
        )?;
        self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetX,
            Animation::new(
                (base_x - amplitude) as f32,
                base_x as f32,
                step,
                Easing::InOutSine,
            )
            .with_delay(step.saturating_mul(2)),
            AnimationConflictPolicy::Queue,
        )?;
        Ok((a, b))
    }

    pub fn preset_selection_bump_settle(
        &mut self,
        widget_id: WidgetId,
        base_y: i32,
        bump_px: i32,
        duration_ms: u32,
    ) -> Result<(AnimationId, AnimationId), WidgetAnimationError> {
        let up_ms = (duration_ms / 3).max(1);
        let settle_ms = duration_ms.saturating_sub(up_ms).max(1);
        let bump_y = base_y - bump_px.abs();
        let up = self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetY,
            Animation::new(base_y as f32, bump_y as f32, up_ms, Easing::OutCubic),
            AnimationConflictPolicy::Replace,
        )?;
        let settle = self.bind_property_with_policy(
            widget_id,
            AnimatedProperty::WidgetY,
            Animation::new(bump_y as f32, base_y as f32, settle_ms, Easing::OutBounce)
                .with_delay(up_ms),
            AnimationConflictPolicy::Queue,
        )?;
        Ok((up, settle))
    }

    pub fn bind_property_with_policy(
        &mut self,
        widget_id: WidgetId,
        property: AnimatedProperty,
        animation: Animation,
        policy: AnimationConflictPolicy,
    ) -> Result<AnimationId, WidgetAnimationError> {
        match policy {
            AnimationConflictPolicy::Ignore
                if self
                    .bindings
                    .iter()
                    .flatten()
                    .any(|b| b.widget_id == widget_id && b.property == property) =>
            {
                return Err(WidgetAnimationError::ConflictIgnored);
            }
            AnimationConflictPolicy::Replace => {
                let ids_to_stop: Vec<AnimationId, BINDINGS> = self
                    .bindings
                    .iter()
                    .flatten()
                    .filter(|b| b.widget_id == widget_id && b.property == property)
                    .map(|b| b.animation_id)
                    .collect();
                for id in ids_to_stop {
                    let _ = self.stop(id);
                }
            }
            AnimationConflictPolicy::Queue | AnimationConflictPolicy::Ignore => {}
        }

        let animation_id = self
            .animations
            .start(animation)
            .map_err(|_| WidgetAnimationError::AnimationsFull)?;

        let has_existing = self
            .bindings
            .iter()
            .flatten()
            .any(|b| b.widget_id == widget_id && b.property == property);

        if let Some(slot) = self.bindings.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(Binding {
                animation_id,
                widget_id,
                property,
                last_iteration: 0,
                queued: has_existing && policy == AnimationConflictPolicy::Queue,
            });
            if let Some(cb) = self.callbacks.on_start {
                cb(animation_id, widget_id, property);
            }
            Ok(animation_id)
        } else {
            let _ = self.animations.stop(animation_id);
            Err(WidgetAnimationError::BindingsFull)
        }
    }

    pub fn bind_property(
        &mut self,
        widget_id: WidgetId,
        property: AnimatedProperty,
        animation: Animation,
    ) -> Result<AnimationId, WidgetAnimationError> {
        self.bind_property_with_policy(
            widget_id,
            property,
            animation,
            AnimationConflictPolicy::Replace,
        )
    }

    pub fn stop(&mut self, animation_id: AnimationId) -> bool {
        let stopped = self.animations.stop(animation_id);
        for slot in &mut self.bindings {
            if slot
                .as_ref()
                .is_some_and(|binding| binding.animation_id == animation_id)
            {
                *slot = None;
            }
        }
        stopped
    }

    pub fn stop_widget(&mut self, widget_id: WidgetId) -> usize {
        let ids: Vec<AnimationId, BINDINGS> = self
            .bindings
            .iter()
            .flatten()
            .filter(|b| b.widget_id == widget_id)
            .map(|b| b.animation_id)
            .collect();
        let count = ids.len();
        for id in ids {
            let _ = self.stop(id);
        }
        count
    }

    pub fn stop_widget_property(
        &mut self,
        widget_id: WidgetId,
        property: AnimatedProperty,
    ) -> usize {
        let ids: Vec<AnimationId, BINDINGS> = self
            .bindings
            .iter()
            .flatten()
            .filter(|b| b.widget_id == widget_id && b.property == property)
            .map(|b| b.animation_id)
            .collect();
        let count = ids.len();
        for id in ids {
            let _ = self.stop(id);
        }
        count
    }

    pub fn is_animating_widget(&self, widget_id: WidgetId) -> bool {
        self.bindings
            .iter()
            .flatten()
            .any(|b| b.widget_id == widget_id)
    }

    pub fn is_animating_widget_property(
        &self,
        widget_id: WidgetId,
        property: AnimatedProperty,
    ) -> bool {
        self.bindings
            .iter()
            .flatten()
            .any(|b| b.widget_id == widget_id && b.property == property)
    }

    pub fn handles_for_widget<const M: usize>(
        &self,
        widget_id: WidgetId,
        out: &mut Vec<AnimationId, M>,
    ) -> usize {
        out.clear();
        for binding in self
            .bindings
            .iter()
            .flatten()
            .filter(|b| b.widget_id == widget_id)
        {
            let _ = out.push(binding.animation_id);
        }
        out.len()
    }

    pub fn active_bindings<const M: usize>(&self, out: &mut Vec<BindingSnapshot, M>) -> usize {
        out.clear();
        for binding in self.bindings.iter().flatten() {
            let _ = out.push(BindingSnapshot {
                animation_id: binding.animation_id,
                widget_id: binding.widget_id,
                property: binding.property,
                queued: binding.queued,
            });
        }
        out.len()
    }

    pub fn tick<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>(
        &mut self,
        dt_ms: u32,
        gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    ) -> Result<(), GuiError> {
        for idx in 0..self.bindings.len() {
            let Some(binding) = self.bindings[idx] else {
                continue;
            };
            if binding.queued
                && self.bindings.iter().enumerate().any(|(other_idx, other)| {
                    other_idx != idx
                        && other.as_ref().is_some_and(|other| {
                            other.widget_id == binding.widget_id
                                && other.property == binding.property
                                && other.animation_id != binding.animation_id
                        })
                })
            {
                continue;
            }

            let Some((value, iteration, done)) = ({
                if let Some(anim) = self.animations.animation_mut(binding.animation_id) {
                    anim.tick(dt_ms);
                    Some((anim.value(), anim.iteration(), anim.is_done()))
                } else {
                    None
                }
            }) else {
                if let Some(cb) = self.callbacks.on_complete {
                    cb(binding.animation_id, binding.widget_id, binding.property);
                }
                self.bindings[idx] = None;
                continue;
            };

            if iteration > binding.last_iteration {
                if let Some(cb) = self.callbacks.on_repeat {
                    cb(binding.animation_id, binding.widget_id, binding.property);
                }
                if let Some(slot_binding) = self.bindings[idx].as_mut() {
                    slot_binding.last_iteration = iteration;
                }
            }

            match binding.property {
                AnimatedProperty::Progress => gui.set_progress(binding.widget_id, value)?,
                AnimatedProperty::Meter => gui.set_meter_value(binding.widget_id, value)?,
                AnimatedProperty::SliderValue => gui.set_slider_value(binding.widget_id, value)?,
                AnimatedProperty::ScrollOffsetY => {
                    gui.set_scroll_offset(binding.widget_id, value.round() as i32)?
                }
                AnimatedProperty::TabSelected => {
                    gui.set_tab_selected(binding.widget_id, value.max(0.0).round() as usize)?
                }
                AnimatedProperty::DropdownSelected => {
                    gui.set_dropdown_selected(binding.widget_id, value.max(0.0).round() as usize)?
                }
                AnimatedProperty::RollerSelected => {
                    gui.set_roller_selected(binding.widget_id, value.max(0.0).round() as usize)?
                }
                AnimatedProperty::GaugeValue => gui.set_gauge_value(binding.widget_id, value)?,
                AnimatedProperty::SpinnerPhase => {
                    gui.set_spinner_phase(binding.widget_id, value)?
                }
                AnimatedProperty::CornerRadius => gui.set_widget_corner_radius(
                    binding.widget_id,
                    value.clamp(0.0, 255.0).round() as u8,
                )?,
                AnimatedProperty::AccentR
                | AnimatedProperty::AccentG
                | AnimatedProperty::AccentB => {
                    let node = gui
                        .widgets()
                        .iter()
                        .find(|node| node.id == binding.widget_id)
                        .ok_or(GuiError::NotFound)?;
                    let mut accent = node.style.normal.accent;
                    match binding.property {
                        AnimatedProperty::AccentR => {
                            accent = Rgb565::new(
                                value.clamp(0.0, 31.0).round() as u8,
                                accent.g(),
                                accent.b(),
                            );
                        }
                        AnimatedProperty::AccentG => {
                            accent = Rgb565::new(
                                accent.r(),
                                value.clamp(0.0, 63.0).round() as u8,
                                accent.b(),
                            );
                        }
                        AnimatedProperty::AccentB => {
                            accent = Rgb565::new(
                                accent.r(),
                                accent.g(),
                                value.clamp(0.0, 31.0).round() as u8,
                            );
                        }
                        _ => {}
                    }
                    gui.set_widget_accent(binding.widget_id, accent)?;
                }
                AnimatedProperty::WidgetX => {
                    gui.set_widget_x(binding.widget_id, value.round() as i32)?
                }
                AnimatedProperty::WidgetY => {
                    gui.set_widget_y(binding.widget_id, value.round() as i32)?
                }
                AnimatedProperty::WidgetWidth => {
                    gui.set_widget_width(binding.widget_id, value.max(1.0).round() as u32)?
                }
                AnimatedProperty::WidgetHeight => {
                    gui.set_widget_height(binding.widget_id, value.max(1.0).round() as u32)?
                }
                AnimatedProperty::Opacity => {
                    gui.set_widget_opacity(binding.widget_id, value.clamp(0.0, 255.0) as u8)?
                }
            }
            if done {
                let _ = self.animations.stop(binding.animation_id);
                if let Some(cb) = self.callbacks.on_complete {
                    cb(binding.animation_id, binding.widget_id, binding.property);
                }
                self.bindings[idx] = None;
            }
        }
        Ok(())
    }

    pub fn active_count(&self) -> usize {
        self.bindings.iter().flatten().count()
    }
}

impl From<AnimationError> for WidgetAnimationError {
    fn from(_: AnimationError) -> Self {
        Self::AnimationsFull
    }
}

pub mod presets {
    use embedded_graphics_core::pixelcolor::Rgb565;

    use super::{
        AnimationConflictPolicy, Easing, PathPoint, WidgetAnimationError, WidgetAnimator, WidgetId,
    };
    use crate::cinematic::{
        GlanceTileSpec, PeekRevealSpec, animate_glance_focus, animate_peek_reveal,
    };

    pub fn entrance_fade_in_up<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        widget_id: WidgetId,
        from_y: i32,
        to_y: i32,
        duration_ms: u32,
    ) -> Result<(), WidgetAnimationError> {
        animator.preset_fade_in_up(widget_id, from_y, to_y, duration_ms)?;
        Ok(())
    }

    pub fn attention_shake<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        widget_id: WidgetId,
        base_x: i32,
        amplitude: i32,
        duration_ms: u32,
    ) -> Result<(), WidgetAnimationError> {
        animator.preset_attention_shake(widget_id, base_x, amplitude, duration_ms)?;
        Ok(())
    }

    pub fn selection_bump_settle<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        widget_id: WidgetId,
        base_y: i32,
        bump_px: i32,
        duration_ms: u32,
    ) -> Result<(), WidgetAnimationError> {
        animator.preset_selection_bump_settle(widget_id, base_y, bump_px, duration_ms)?;
        Ok(())
    }

    pub fn style_breathe<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        widget_id: WidgetId,
        low_opacity: u8,
        high_opacity: u8,
        low_radius: u8,
        high_radius: u8,
        duration_ms: u32,
    ) -> Result<(), WidgetAnimationError> {
        animator.pulse_opacity(
            widget_id,
            low_opacity,
            high_opacity,
            duration_ms,
            Easing::InOutSine,
        )?;
        animator.animate_corner_radius(
            widget_id,
            low_radius,
            high_radius,
            duration_ms,
            Easing::InOutSine,
        )?;
        Ok(())
    }

    pub fn style_accent_cycle<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        widget_id: WidgetId,
        from: Rgb565,
        to: Rgb565,
        duration_ms: u32,
    ) -> Result<(), WidgetAnimationError> {
        animator.animate_accent_color(widget_id, from, to, duration_ms, Easing::InOutSine)?;
        Ok(())
    }

    pub fn path_float_loop<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        widget_id: WidgetId,
        center_x: i32,
        center_y: i32,
        radius: i32,
        duration_ms: u32,
    ) -> Result<(), WidgetAnimationError> {
        let points = [
            PathPoint::new(center_x as f32, (center_y - radius) as f32),
            PathPoint::new((center_x + radius) as f32, center_y as f32),
            PathPoint::new(center_x as f32, (center_y + radius) as f32),
            PathPoint::new((center_x - radius) as f32, center_y as f32),
            PathPoint::new(center_x as f32, (center_y - radius) as f32),
        ];
        animator.animate_widget_path(widget_id, &points, duration_ms, Easing::InOutSine)?;
        Ok(())
    }

    pub fn orchestrate_stagger_x<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        widget_ids: &[WidgetId],
        from: i32,
        to: i32,
        duration_ms: u32,
        stagger_ms: u32,
    ) -> Result<usize, WidgetAnimationError> {
        animator.stagger_widget_x(
            widget_ids,
            from,
            to,
            duration_ms,
            stagger_ms,
            Easing::OutSine,
        )
    }

    pub fn menu_focus_choreography<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        focused: WidgetId,
        base_x: i32,
        base_y: i32,
    ) -> Result<(), WidgetAnimationError> {
        animator.preset_selection_bump_settle(focused, base_y, 3, 120)?;
        animator.animate_widget_x_with_custom_interpolator(
            focused,
            base_x,
            base_x + 6,
            120,
            Easing::InOutSine,
            |from, to, t| {
                if t < 0.5 {
                    from + (to - from) * (t * 1.6)
                } else {
                    to - (to - from) * ((t - 0.5) * 1.2)
                }
            },
            AnimationConflictPolicy::Replace,
        )?;
        animator.animate_opacity(focused, 180, 255, 120, Easing::OutSine)?;
        Ok(())
    }

    pub fn dialog_pop_choreography<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        dialog: WidgetId,
        base_y: i32,
    ) -> Result<(), WidgetAnimationError> {
        animator.preset_fade_in_up(dialog, base_y + 8, base_y, 180)?;
        animator.animate_corner_radius(dialog, 1, 4, 180, Easing::OutBack)?;
        animator.animate_opacity(dialog, 120, 255, 180, Easing::OutSine)?;
        Ok(())
    }

    pub fn list_focus_with_neighbors<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        focused: WidgetId,
        neighbors: &[WidgetId],
        base_x: i32,
        base_y: i32,
    ) -> Result<(), WidgetAnimationError> {
        menu_focus_choreography(animator, focused, base_x, base_y)?;
        for neighbor in neighbors.iter().copied() {
            animator.animate_widget_x(
                neighbor,
                base_x,
                base_x.saturating_sub(2),
                120,
                Easing::OutSine,
            )?;
            animator.animate_opacity(neighbor, 255, 170, 120, Easing::OutSine)?;
        }
        Ok(())
    }

    pub fn peek_reveal<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        icon_widget: WidgetId,
        title_widget: Option<WidgetId>,
        subtitle_widget: Option<WidgetId>,
        base_x: i32,
        base_y: i32,
    ) -> Result<(), WidgetAnimationError> {
        animate_peek_reveal(
            animator,
            icon_widget,
            title_widget,
            subtitle_widget,
            base_x,
            base_y,
            PeekRevealSpec::default(),
        )
    }

    pub fn glance_focus<const TRACKS: usize, const BINDINGS: usize>(
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        focused: WidgetId,
        neighbors: &[WidgetId],
        base_x: i32,
        base_y: i32,
    ) -> Result<(), WidgetAnimationError> {
        animate_glance_focus(
            animator,
            focused,
            neighbors,
            base_x,
            base_y,
            GlanceTileSpec::default(),
        )
    }
}
