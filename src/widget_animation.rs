//! Widget-level animation bindings built on top of [`AnimationManager`].
//! Keeps fixed-capacity, no-allocation behavior suitable for embedded targets.

use heapless::Vec;

use crate::{
    animation::{Animation, AnimationError, AnimationId, AnimationManager, Easing},
    context::{GuiContext, GuiError},
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
    WidgetX,
    WidgetY,
    WidgetWidth,
    WidgetHeight,
    Opacity,
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

    pub fn stop_widget_property(&mut self, widget_id: WidgetId, property: AnimatedProperty) -> usize {
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

    pub fn is_animating_widget_property(&self, widget_id: WidgetId, property: AnimatedProperty) -> bool {
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
        for binding in self.bindings.iter().flatten().filter(|b| b.widget_id == widget_id) {
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
                AnimatedProperty::WidgetX => gui.set_widget_x(binding.widget_id, value.round() as i32)?,
                AnimatedProperty::WidgetY => gui.set_widget_y(binding.widget_id, value.round() as i32)?,
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
