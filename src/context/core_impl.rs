use heapless::Vec;

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    geometry::{DirtyTracker, Rect},
    input::UiEvent,
    present::PresentRegion,
    render::RenderQuality,
    style::{Style, Theme, VisualState, WidgetStyle, lerp_style},
    widget::{MenuContract, StyleClassId, WidgetId},
    widgets::WidgetNode,
};

use super::*;

impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    GuiContext<'a, NODES, EVENTS, DIRTY>
{
    pub fn new(viewport: Rect) -> Self {
        let mut dirty = DirtyTracker::new();
        let _ = dirty.mark_all(viewport);
        Self {
            viewport,
            widgets: Vec::new(),
            subscriptions: Vec::new(),
            dispatch_policies: Vec::new(),
            class_styles: Vec::new(),
            events: Vec::new(),
            dirty,
            theme: Theme::default(),
            focus: None,
            active_focus_group: None,
            render_quality: RenderQuality::High,
            long_press_ms: 500,
            textarea_cursor_blink_ms: 500,
            textarea_cursor_blink_elapsed_ms: 0,
            press_repeat_delay_ms: 650,
            press_repeat_interval_ms: 140,
            select_double_window_ms: 300,
            select_elapsed_ms: 0,
            last_select_id: None,
            pointer_double_window_ms: 300,
            pointer_elapsed_ms: 0,
            last_pointer_id: None,
            pressed: None,
            inertia_scroll: None,
            scroll_physics: ScrollPhysics::default(),
            state_transition_ms: 0,
            state_transitions: Vec::new(),
            widget_press_timings: Vec::new(),
            widget_key_policies: Vec::new(),
            widget_key_bindings: Vec::new(),
            menu_contract: MenuContract::default(),
            textarea_undo: Vec::new(),
            textarea_redo: Vec::new(),
            next_id: 1,
        }
    }

    pub const fn viewport(&self) -> Rect {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: Rect) -> Result<(), GuiError> {
        self.viewport = viewport;
        self.dirty.mark_all(viewport)?;
        Ok(())
    }

    pub fn clear_widgets(&mut self) -> Result<(), GuiError> {
        self.widgets.clear();
        self.subscriptions.clear();
        self.dispatch_policies.clear();
        self.class_styles.clear();
        self.focus = None;
        self.pressed = None;
        self.inertia_scroll = None;
        self.last_select_id = None;
        self.select_elapsed_ms = 0;
        self.last_pointer_id = None;
        self.pointer_elapsed_ms = 0;
        self.state_transitions.clear();
        self.widget_press_timings.clear();
        self.widget_key_policies.clear();
        self.widget_key_bindings.clear();
        self.textarea_undo.clear();
        self.textarea_redo.clear();
        self.dirty.mark_all(self.viewport)?;
        Ok(())
    }

    pub const fn long_press_threshold_ms(&self) -> u32 {
        self.long_press_ms
    }

    pub fn set_long_press_threshold_ms(&mut self, threshold_ms: u32) {
        self.long_press_ms = threshold_ms.max(1);
    }

    pub fn set_press_repeat_timing(&mut self, delay_ms: u32, interval_ms: u32) {
        self.press_repeat_delay_ms = delay_ms.max(1);
        self.press_repeat_interval_ms = interval_ms.max(1);
    }

    pub fn set_double_select_window_ms(&mut self, window_ms: u32) {
        self.select_double_window_ms = window_ms.max(1);
    }

    pub fn set_double_pointer_window_ms(&mut self, window_ms: u32) {
        self.pointer_double_window_ms = window_ms.max(1);
    }

    pub fn menu_contract(&self) -> MenuContract {
        self.menu_contract
    }

    pub fn set_menu_contract(&mut self, contract: MenuContract) {
        self.menu_contract = contract;
    }

    pub fn set_widget_press_timing(
        &mut self,
        id: WidgetId,
        timing: PressTiming,
    ) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        let timing = PressTiming {
            long_press_ms: timing.long_press_ms.max(1),
            repeat_delay_ms: timing.repeat_delay_ms.max(1),
            repeat_interval_ms: timing.repeat_interval_ms.max(1),
        };
        if let Some((_, current)) = self
            .widget_press_timings
            .iter_mut()
            .find(|(timing_id, _)| *timing_id == id)
        {
            *current = timing;
            return Ok(());
        }
        self.widget_press_timings
            .push((id, timing))
            .map_err(|_| GuiError::WidgetsFull)
    }

    pub fn clear_widget_press_timing(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some(pos) = self
            .widget_press_timings
            .iter()
            .position(|(timing_id, _)| *timing_id == id)
        {
            self.widget_press_timings.remove(pos);
        }
        Ok(())
    }

    pub fn widget_press_timing(&self, id: WidgetId) -> Result<Option<PressTiming>, GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        Ok(self
            .widget_press_timings
            .iter()
            .find(|(timing_id, _)| *timing_id == id)
            .map(|(_, timing)| *timing))
    }

    pub fn set_widget_key_input_policy(
        &mut self,
        id: WidgetId,
        policy: WidgetKeyInputPolicy,
    ) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some((_, current)) = self
            .widget_key_policies
            .iter_mut()
            .find(|(policy_id, _)| *policy_id == id)
        {
            *current = policy;
            return Ok(());
        }
        self.widget_key_policies
            .push((id, policy))
            .map_err(|_| GuiError::WidgetsFull)
    }

    pub fn clear_widget_key_input_policy(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some(pos) = self
            .widget_key_policies
            .iter()
            .position(|(policy_id, _)| *policy_id == id)
        {
            self.widget_key_policies.remove(pos);
        }
        Ok(())
    }

    pub fn widget_key_input_policy(
        &self,
        id: WidgetId,
    ) -> Result<Option<WidgetKeyInputPolicy>, GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        Ok(self
            .widget_key_policies
            .iter()
            .find(|(policy_id, _)| *policy_id == id)
            .map(|(_, policy)| *policy))
    }

    pub fn set_widget_key_bindings(
        &mut self,
        id: WidgetId,
        bindings: WidgetKeyBindings,
    ) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some((_, current)) = self
            .widget_key_bindings
            .iter_mut()
            .find(|(binding_id, _)| *binding_id == id)
        {
            *current = bindings;
            return Ok(());
        }
        self.widget_key_bindings
            .push((id, bindings))
            .map_err(|_| GuiError::WidgetsFull)
    }

    pub fn clear_widget_key_bindings(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some(pos) = self
            .widget_key_bindings
            .iter()
            .position(|(binding_id, _)| *binding_id == id)
        {
            self.widget_key_bindings.remove(pos);
        }
        Ok(())
    }

    pub fn widget_key_bindings(&self, id: WidgetId) -> Result<Option<WidgetKeyBindings>, GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        Ok(self
            .widget_key_bindings
            .iter()
            .find(|(binding_id, _)| *binding_id == id)
            .map(|(_, bindings)| *bindings))
    }

    pub fn set_scroll_physics(
        &mut self,
        velocity_threshold: f32,
        velocity_decay: f32,
        drag_velocity_blend: f32,
    ) {
        self.scroll_physics.velocity_threshold = velocity_threshold.max(0.001);
        self.scroll_physics.velocity_decay = velocity_decay.clamp(0.01, 0.999);
        self.scroll_physics.drag_velocity_blend = drag_velocity_blend.clamp(0.01, 1.0);
    }

    pub fn set_state_transition_duration_ms(&mut self, duration_ms: u32) {
        self.state_transition_ms = duration_ms;
        if duration_ms == 0 {
            self.state_transitions.clear();
        }
    }

    pub fn active_state_transitions(&self) -> usize {
        self.state_transitions.len()
    }

    pub fn set_textarea_cursor_blink_timing(&mut self, period_ms: u32) {
        self.textarea_cursor_blink_ms = period_ms.max(1);
    }

    pub fn widgets(&self) -> &[WidgetNode<'a>] {
        self.widgets.as_slice()
    }

    pub fn dirty_regions(&self) -> &[Rect] {
        self.dirty.as_slice()
    }

    pub fn present_regions(&self) -> impl Iterator<Item = PresentRegion> + '_ {
        self.dirty
            .as_slice()
            .iter()
            .copied()
            .map(PresentRegion::from)
    }

    pub fn bounding_present_region(&self) -> Option<PresentRegion> {
        self.dirty.bounding_rect().map(PresentRegion::from)
    }

    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    pub const fn theme(&self) -> Theme {
        self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) -> Result<(), GuiError> {
        self.theme = theme;
        self.dirty.mark_all(self.viewport)?;
        Ok(())
    }

    pub fn set_style_class<S>(&mut self, class: StyleClassId, style: S) -> Result<(), GuiError>
    where
        S: Into<WidgetStyle>,
    {
        if class == StyleClassId::NONE {
            return Ok(());
        }
        if let Some((_, slot)) = self.class_styles.iter_mut().find(|(id, _)| *id == class) {
            *slot = style.into();
        } else {
            self.class_styles
                .push((class, style.into()))
                .map_err(|_| GuiError::WidgetsFull)?;
        }
        self.dirty.mark_all(self.viewport)?;
        Ok(())
    }

    pub fn clear_style_class(&mut self, class: StyleClassId) -> Result<(), GuiError> {
        if let Some(pos) = self.class_styles.iter().position(|(id, _)| *id == class) {
            self.class_styles.remove(pos);
            self.dirty.mark_all(self.viewport)?;
        }
        Ok(())
    }

    pub fn set_style_class_state(
        &mut self,
        class: StyleClassId,
        state: VisualState,
        style: Style,
    ) -> Result<(), GuiError> {
        if class == StyleClassId::NONE {
            return Ok(());
        }
        if let Some((_, slot)) = self.class_styles.iter_mut().find(|(id, _)| *id == class) {
            *slot = slot.with_state_override(state, style);
        } else {
            let base = WidgetStyle::new(Style::new()).with_state_override(state, style);
            self.class_styles
                .push((class, base))
                .map_err(|_| GuiError::WidgetsFull)?;
        }
        self.dirty.mark_all(self.viewport)?;
        Ok(())
    }

    pub fn set_widget_style_class(
        &mut self,
        id: WidgetId,
        class: Option<StyleClassId>,
    ) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.style_class = class.filter(|c| *c != StyleClassId::NONE);
        self.mark_subtree_dirty(id)
    }

    pub fn apply_widget_style_transition(
        &mut self,
        id: WidgetId,
        from: VisualState,
        to: VisualState,
        t: f32,
    ) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        let a = node.style.resolve(from);
        let b = node.style.resolve(to);
        let blended = lerp_style(a, b, t);
        node.style = node.style.with_state_override(VisualState::Normal, blended);
        self.mark_subtree_dirty(id)
    }

    pub const fn render_quality(&self) -> RenderQuality {
        self.render_quality
    }

    pub fn set_render_quality(&mut self, quality: RenderQuality) -> Result<(), GuiError> {
        if self.render_quality != quality {
            self.render_quality = quality;
            self.dirty.mark_all(self.viewport)?;
        }
        Ok(())
    }

    pub const fn focus(&self) -> Option<WidgetId> {
        self.focus
    }

    pub fn set_focus(&mut self, focus: Option<WidgetId>) -> Result<(), GuiError> {
        if let Some(id) = focus {
            self.node(id).ok_or(GuiError::NotFound)?;
            if !self.effective_focusable(id) {
                return Err(GuiError::NotFound);
            }
        }

        let old = self.focus;
        self.focus = focus;
        self.textarea_cursor_blink_elapsed_ms = 0;
        self.set_textarea_cursor_visible(old, true);
        self.set_textarea_cursor_visible(focus, true);
        self.start_focus_transitions(old, focus);
        self.mark_focus_pair(old, focus)?;
        if let Some(id) = old {
            self.push_event(UiEvent::Defocused(id))?;
        }
        if let Some(id) = focus {
            self.push_event(UiEvent::Focused(id))?;
        }
        self.push_event(UiEvent::FocusChanged { old, new: focus })?;
        Ok(())
    }
}
