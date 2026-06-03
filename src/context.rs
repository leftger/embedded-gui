use embedded_graphics_core::pixelcolor::Rgb565;
use heapless::Vec;

use crate::{
    geometry::{DirtyError, DirtyTracker, Rect},
    image::{ImageFit, ImageRef, ReelPlayer},
    input::{
        InputEvent, PointerState, UiEvent, UiEventFilter, WidgetDispatchPolicy, WidgetEvent,
        WidgetEventKind,
    },
    layout::{Axis, LayoutItem, LinearLayout},
    math::F32Ext as _,
    present::PresentRegion,
    render::{RenderCtx, RenderQuality, TextAlign},
    state::{ListState, ScrollState, SliderState, TabsState},
    style::{Style, Theme, VisualState, WidgetStyle, lerp_style},
    widget::{
        EventContext, EventPhase, EventPolicy, FocusGroupId, StyleClassId, WidgetFlags, WidgetId,
    },
    widgets::{ChartMode, KeyboardLayout, TEXTAREA_CAPACITY, WidgetKind, WidgetNode},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiError {
    WidgetsFull,
    EventsFull,
    DirtyFull,
    NotFound,
}

impl From<DirtyError> for GuiError {
    fn from(_: DirtyError) -> Self {
        Self::DirtyFull
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PressTracker {
    id: WidgetId,
    start_x: i32,
    start_y: i32,
    last_x: i32,
    last_y: i32,
    elapsed_ms: u32,
    long_emitted: bool,
    gesture_emitted: bool,
    repeat_elapsed_ms: u32,
    scroll_velocity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct InertiaScroll {
    id: WidgetId,
    velocity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollPhysics {
    pub velocity_threshold: f32,
    pub velocity_decay: f32,
    pub drag_velocity_blend: f32,
}

impl Default for ScrollPhysics {
    fn default() -> Self {
        Self {
            velocity_threshold: 0.05,
            velocity_decay: 0.86,
            drag_velocity_blend: 0.4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PressTiming {
    pub long_press_ms: u32,
    pub repeat_delay_ms: u32,
    pub repeat_interval_ms: u32,
}

impl PressTiming {
    pub const fn new(long_press_ms: u32, repeat_delay_ms: u32, repeat_interval_ms: u32) -> Self {
        Self {
            long_press_ms,
            repeat_delay_ms,
            repeat_interval_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetKeyInputPolicy {
    pub raw_select: bool,
    pub raw_back: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyBindingAction {
    Default,
    Ignore,
    Activate,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetKeyBindings {
    pub select: KeyBindingAction,
    pub back: KeyBindingAction,
}

impl Default for WidgetKeyBindings {
    fn default() -> Self {
        Self {
            select: KeyBindingAction::Default,
            back: KeyBindingAction::Default,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TextareaSnapshot {
    text_buf: [u8; TEXTAREA_CAPACITY],
    text_len: u8,
    cursor: usize,
    selection: Option<(usize, usize)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TextareaHistoryEntry {
    id: WidgetId,
    snapshot: TextareaSnapshot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StateTransition {
    id: WidgetId,
    from: VisualState,
    to: VisualState,
    elapsed_ms: u32,
}

pub struct GuiContext<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize> {
    viewport: Rect,
    widgets: Vec<WidgetNode<'a>, NODES>,
    subscriptions: Vec<(WidgetId, UiEventFilter), NODES>,
    dispatch_policies: Vec<(WidgetId, WidgetDispatchPolicy), NODES>,
    class_styles: Vec<(StyleClassId, WidgetStyle), NODES>,
    events: Vec<UiEvent, EVENTS>,
    dirty: DirtyTracker<DIRTY>,
    theme: Theme,
    focus: Option<WidgetId>,
    active_focus_group: Option<FocusGroupId>,
    render_quality: RenderQuality,
    long_press_ms: u32,
    textarea_cursor_blink_ms: u32,
    textarea_cursor_blink_elapsed_ms: u32,
    press_repeat_delay_ms: u32,
    press_repeat_interval_ms: u32,
    select_double_window_ms: u32,
    select_elapsed_ms: u32,
    last_select_id: Option<WidgetId>,
    pointer_double_window_ms: u32,
    pointer_elapsed_ms: u32,
    last_pointer_id: Option<WidgetId>,
    pressed: Option<PressTracker>,
    inertia_scroll: Option<InertiaScroll>,
    scroll_physics: ScrollPhysics,
    state_transition_ms: u32,
    state_transitions: Vec<StateTransition, NODES>,
    widget_press_timings: Vec<(WidgetId, PressTiming), NODES>,
    widget_key_policies: Vec<(WidgetId, WidgetKeyInputPolicy), NODES>,
    widget_key_bindings: Vec<(WidgetId, WidgetKeyBindings), NODES>,
    textarea_undo: Vec<TextareaHistoryEntry, NODES>,
    textarea_redo: Vec<TextareaHistoryEntry, NODES>,
    next_id: u16,
}

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

    pub fn add_panel<S>(&mut self, rect: Rect, style: S) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Panel, style)
    }

    pub fn add_themed_panel(&mut self, rect: Rect) -> Result<WidgetId, GuiError> {
        self.add_panel(rect, self.theme.panel)
    }

    pub fn add_label<S>(
        &mut self,
        rect: Rect,
        text: &'a str,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Label(text), style)
    }

    pub fn add_themed_label(&mut self, rect: Rect, text: &'a str) -> Result<WidgetId, GuiError> {
        self.add_label(rect, text, self.theme.label)
    }

    pub fn add_button<S>(
        &mut self,
        rect: Rect,
        text: &'a str,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = self.add_widget(rect, WidgetKind::Button(text), style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_button(&mut self, rect: Rect, text: &'a str) -> Result<WidgetId, GuiError> {
        self.add_button(rect, text, self.theme.button)
    }

    pub fn add_progress_bar<S>(
        &mut self,
        rect: Rect,
        value: f32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::ProgressBar {
                value: value.clamp(0.0, 1.0),
            },
            style,
        )
    }

    pub fn add_themed_progress_bar(
        &mut self,
        rect: Rect,
        value: f32,
    ) -> Result<WidgetId, GuiError> {
        self.add_progress_bar(rect, value, self.theme.progress)
    }

    pub fn add_toggle<S>(
        &mut self,
        rect: Rect,
        label: &'a str,
        on: bool,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = self.add_widget(rect, WidgetKind::Toggle { label, on }, style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_toggle(
        &mut self,
        rect: Rect,
        label: &'a str,
        on: bool,
    ) -> Result<WidgetId, GuiError> {
        self.add_toggle(rect, label, on, self.theme.toggle)
    }

    pub fn add_checkbox<S>(
        &mut self,
        rect: Rect,
        label: &'a str,
        checked: bool,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = self.add_widget(rect, WidgetKind::Checkbox { label, checked }, style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_checkbox(
        &mut self,
        rect: Rect,
        label: &'a str,
        checked: bool,
    ) -> Result<WidgetId, GuiError> {
        self.add_checkbox(rect, label, checked, self.theme.checkbox)
    }

    pub fn add_slider<S>(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let value = value.clamp(min.min(max), min.max(max));
        let id = self.add_widget(rect, WidgetKind::Slider { value, min, max }, style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_slider(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
    ) -> Result<WidgetId, GuiError> {
        self.add_slider(rect, value, min, max, self.theme.slider)
    }

    pub fn add_value_label<S>(
        &mut self,
        rect: Rect,
        label: &'a str,
        value: i32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::ValueLabel { label, value }, style)
    }

    pub fn add_themed_value_label(
        &mut self,
        rect: Rect,
        label: &'a str,
        value: i32,
    ) -> Result<WidgetId, GuiError> {
        self.add_value_label(rect, label, value, self.theme.value_label)
    }

    pub fn add_icon_button<S>(
        &mut self,
        rect: Rect,
        icon: char,
        label: &'a str,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = self.add_widget(rect, WidgetKind::IconButton { icon, label }, style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_icon_button(
        &mut self,
        rect: Rect,
        icon: char,
        label: &'a str,
    ) -> Result<WidgetId, GuiError> {
        self.add_icon_button(rect, icon, label, self.theme.icon_button)
    }

    pub fn add_list<S>(
        &mut self,
        rect: Rect,
        items: &'a [&'a str],
        selected: usize,
        visible_rows: usize,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let selected = selected.min(items.len().saturating_sub(1));
        let id = self.add_widget(
            rect,
            WidgetKind::List {
                items,
                selected,
                offset: selected,
                visible_rows: visible_rows.max(1),
            },
            style,
        )?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_list(
        &mut self,
        rect: Rect,
        items: &'a [&'a str],
        selected: usize,
        visible_rows: usize,
    ) -> Result<WidgetId, GuiError> {
        self.add_list(rect, items, selected, visible_rows, self.theme.list)
    }

    pub fn add_scroll_view<S>(
        &mut self,
        rect: Rect,
        offset_y: i32,
        content_h: u32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = self.add_widget(
            rect,
            WidgetKind::ScrollView {
                offset_y,
                content_h,
            },
            style,
        )?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_scroll_view(
        &mut self,
        rect: Rect,
        offset_y: i32,
        content_h: u32,
    ) -> Result<WidgetId, GuiError> {
        self.add_scroll_view(rect, offset_y, content_h, self.theme.list)
    }

    pub fn add_tabs<S>(
        &mut self,
        rect: Rect,
        labels: &'a [&'a str],
        selected: usize,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let selected = selected.min(labels.len().saturating_sub(1));
        let id = self.add_widget(rect, WidgetKind::Tabs { labels, selected }, style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_tabs(
        &mut self,
        rect: Rect,
        labels: &'a [&'a str],
        selected: usize,
    ) -> Result<WidgetId, GuiError> {
        self.add_tabs(rect, labels, selected, self.theme.tabs)
    }

    pub fn add_dialog<S>(
        &mut self,
        rect: Rect,
        title: &'a str,
        body: &'a str,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Dialog { title, body }, style)
    }

    pub fn add_themed_dialog(
        &mut self,
        rect: Rect,
        title: &'a str,
        body: &'a str,
    ) -> Result<WidgetId, GuiError> {
        self.add_dialog(rect, title, body, self.theme.dialog)
    }

    pub fn add_toast<S>(
        &mut self,
        rect: Rect,
        text: &'a str,
        ttl_ms: u32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Toast { text, ttl_ms }, style)
    }

    pub fn add_themed_toast(
        &mut self,
        rect: Rect,
        text: &'a str,
        ttl_ms: u32,
    ) -> Result<WidgetId, GuiError> {
        self.add_toast(rect, text, ttl_ms, self.theme.toast)
    }

    pub fn add_meter<S>(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Meter { value, min, max }, style)
    }

    pub fn add_themed_meter(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
    ) -> Result<WidgetId, GuiError> {
        self.add_meter(rect, value, min, max, self.theme.meter)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_arc_gauge<S>(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
        start_deg: i32,
        end_deg: i32,
        thickness: u8,
        antialias: bool,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::ArcGauge {
                value,
                min,
                max,
                start_deg,
                end_deg,
                thickness: thickness.max(1),
                antialias,
                major_ticks: 6,
                minor_ticks: 2,
                show_value: false,
            },
            style,
        )
    }

    pub fn add_gauge<S>(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::Gauge {
                value,
                min,
                max,
                major_ticks: 6,
                minor_ticks: 2,
                show_value: false,
            },
            style,
        )
    }

    pub fn add_gauge_needle<S>(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
        start_deg: i32,
        end_deg: i32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::GaugeNeedle {
                value,
                min,
                max,
                start_deg,
                end_deg,
            },
            style,
        )
    }

    pub fn add_chart<S>(
        &mut self,
        rect: Rect,
        values: &'a [f32],
        min: f32,
        max: f32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::Chart {
                values,
                min,
                max,
                thickness: 1,
                fill_under: false,
                markers: false,
                mode: ChartMode::Line,
                show_grid: false,
                show_axes: false,
                show_labels: false,
            },
            style,
        )
    }

    pub fn set_chart_style(
        &mut self,
        id: WidgetId,
        thickness: u8,
        fill_under: bool,
        markers: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Chart {
                thickness: ref mut t,
                fill_under: ref mut fill,
                markers: ref mut mark,
                ..
            } => {
                *t = thickness.max(1);
                *fill = fill_under;
                *mark = markers;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_chart_decoration(
        &mut self,
        id: WidgetId,
        mode: ChartMode,
        show_grid: bool,
        show_axes: bool,
        show_labels: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Chart {
                mode: ref mut chart_mode,
                show_grid: ref mut grid,
                show_axes: ref mut axes,
                show_labels: ref mut labels,
                ..
            } => {
                *chart_mode = mode;
                *grid = show_grid;
                *axes = show_axes;
                *labels = show_labels;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn add_spinner<S>(&mut self, rect: Rect, phase: f32, style: S) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Spinner { phase }, style)
    }

    pub fn add_dropdown<S>(
        &mut self,
        rect: Rect,
        items: &'a [&'a str],
        selected: usize,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let selected = selected.min(items.len().saturating_sub(1));
        let id = self.add_widget(
            rect,
            WidgetKind::Dropdown {
                items,
                selected,
                open: false,
            },
            style,
        )?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_roller<S>(
        &mut self,
        rect: Rect,
        items: &'a [&'a str],
        selected: usize,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let selected = selected.min(items.len().saturating_sub(1));
        let id = self.add_widget(rect, WidgetKind::Roller { items, selected }, style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_table<S>(
        &mut self,
        rect: Rect,
        rows: &'a [&'a [&'a str]],
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::Table {
                rows,
                separators: true,
                cell_padding: 1,
                align: TextAlign::Left,
            },
            style,
        )
    }

    pub fn set_table_style(
        &mut self,
        id: WidgetId,
        separators: bool,
        cell_padding: u8,
        align: TextAlign,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Table {
                separators: ref mut cell_sep,
                cell_padding: ref mut pad,
                align: ref mut table_align,
                ..
            } => {
                *cell_sep = separators;
                *pad = cell_padding.min(6);
                *table_align = align;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn add_textarea<S>(
        &mut self,
        rect: Rect,
        text: &'a str,
        placeholder: &'a str,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let cursor = text.chars().count();
        let (text_buf, text_len) = textarea_storage_from_str(text);
        let id = self.add_widget(
            rect,
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                placeholder,
                selection: None,
                cursor_visible: true,
                read_only: false,
                single_line: false,
                accept_newline: true,
            },
            style,
        )?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_keyboard<S>(
        &mut self,
        rect: Rect,
        keys: &'a [char],
        cols: u8,
        target: Option<WidgetId>,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_keyboard_with_alt(rect, keys, None, cols, target, style)
    }

    pub fn add_keyboard_with_alt<S>(
        &mut self,
        rect: Rect,
        keys: &'a [char],
        alt_keys: Option<&'a [char]>,
        cols: u8,
        target: Option<WidgetId>,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = self.add_widget(
            rect,
            WidgetKind::Keyboard {
                keys,
                selected: 0,
                cols: cols.max(1),
                alt_keys,
                layout: KeyboardLayout::Normal,
                target,
            },
            style,
        )?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_image<S>(
        &mut self,
        rect: Rect,
        image: ImageRef<'a>,
        fit: ImageFit,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Image { image, fit }, style)
    }

    pub fn add_peek_reveal<S>(
        &mut self,
        rect: Rect,
        icon: ImageRef<'a>,
        title: &'a str,
        subtitle: &'a str,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::PeekReveal {
                icon,
                title,
                subtitle,
                progress: 0.0,
            },
            style,
        )
    }

    pub fn add_glance_tile<S>(
        &mut self,
        rect: Rect,
        icon: char,
        title: &'a str,
        subtitle: &'a str,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = self.add_widget(
            rect,
            WidgetKind::GlanceTile {
                icon,
                title,
                subtitle,
                highlighted: false,
            },
            style,
        )?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_card_deck<S>(
        &mut self,
        rect: Rect,
        titles: &'a [&'a str],
        selected: usize,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::CardDeck {
                titles,
                selected: selected.min(titles.len().saturating_sub(1)),
            },
            style,
        )
    }

    pub fn add_reel<S>(
        &mut self,
        rect: Rect,
        player: ReelPlayer<'a>,
        fit: ImageFit,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Reel { player, fit }, style)
    }

    pub fn add_border<S>(&mut self, rect: Rect, style: S) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Border, style)
    }

    pub fn add_spacer(&mut self, rect: Rect) -> Result<WidgetId, GuiError> {
        self.add_widget(rect, WidgetKind::Spacer, Style::default())
    }

    pub fn add_menu<S>(
        &mut self,
        rect: Rect,
        items: &'a [&'a str],
        selected: usize,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let selected = selected.min(items.len().saturating_sub(1));
        let id = self.add_widget(rect, WidgetKind::Menu { items, selected }, style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn set_progress(&mut self, id: WidgetId, value: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::ProgressBar { value: ref mut v } => {
                *v = value.clamp(0.0, 1.0);
                self.dirty.add(rect)?;
                Ok(())
            }
            WidgetKind::PeekReveal {
                progress: ref mut v, ..
            } => {
                *v = value.clamp(0.0, 1.0);
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_glance_highlighted(&mut self, id: WidgetId, highlighted: bool) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::GlanceTile {
                highlighted: ref mut h,
                ..
            } => {
                *h = highlighted;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_card_deck_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::CardDeck {
                titles,
                selected: ref mut current,
            } => {
                *current = selected.min(titles.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tick_reel(&mut self, id: WidgetId, dt_ms: u32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Reel {
                player: ref mut reel, ..
            } => {
                reel.tick(dt_ms);
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_menu_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Menu {
                items,
                selected: ref mut current,
            } => {
                *current = selected.min(items.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn menu_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::Menu { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn list_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::List { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_list_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::List {
                items,
                selected: ref mut current,
                ref mut offset,
                visible_rows,
            } => {
                let mut state = ListState::new(*current, *offset, visible_rows);
                state.set_selected(selected, items.len());
                *current = state.selected;
                *offset = state.offset;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_toggle(&mut self, id: WidgetId, on: bool) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Toggle { on: ref mut v, .. } => {
                *v = on;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn toggle_value(&self, id: WidgetId) -> Option<bool> {
        match self.node(id)?.kind {
            WidgetKind::Toggle { on, .. } => Some(on),
            _ => None,
        }
    }

    pub fn set_checked(&mut self, id: WidgetId, checked: bool) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Checkbox {
                checked: ref mut v, ..
            } => {
                *v = checked;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn checked_value(&self, id: WidgetId) -> Option<bool> {
        match self.node(id)?.kind {
            WidgetKind::Checkbox { checked, .. } => Some(checked),
            _ => None,
        }
    }

    pub fn set_slider_value(&mut self, id: WidgetId, value: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Slider {
                value: ref mut v,
                min,
                max,
            } => {
                let mut state = SliderState::new(*v, min, max);
                state.set_value(value);
                *v = state.value;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn slider_value(&self, id: WidgetId) -> Option<f32> {
        match self.node(id)?.kind {
            WidgetKind::Slider { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn set_value_label(&mut self, id: WidgetId, value: i32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::ValueLabel {
                value: ref mut v, ..
            } => {
                *v = value;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_scroll_offset(&mut self, id: WidgetId, offset_y: i32) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::ScrollView {
                offset_y: ref mut v,
                content_h,
            } => {
                let mut state = ScrollState::new(*v, content_h);
                state.set_offset(offset_y);
                *v = state.offset_y;
                self.mark_subtree_dirty(id)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn scroll_offset(&self, id: WidgetId) -> Option<i32> {
        match self.node(id)?.kind {
            WidgetKind::ScrollView { offset_y, .. } => Some(offset_y),
            _ => None,
        }
    }

    pub fn set_tab_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Tabs {
                labels,
                selected: ref mut v,
            } => {
                let mut state = TabsState::new(*v);
                state.set_selected(selected, labels.len());
                *v = state.selected;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tab_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::Tabs { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_toast_ttl(&mut self, id: WidgetId, ttl_ms: u32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Toast {
                ttl_ms: ref mut v, ..
            } => {
                *v = ttl_ms;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tick_toast(&mut self, id: WidgetId, dt_ms: u32) -> Result<(), GuiError> {
        let ttl = match self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::Toast { ttl_ms, .. } => ttl_ms.saturating_sub(dt_ms),
            _ => return Err(GuiError::NotFound),
        };
        self.set_toast_ttl(id, ttl)
    }

    pub fn set_meter_value(&mut self, id: WidgetId, value: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Meter {
                value: ref mut v,
                min,
                max,
            } => {
                *v = value.clamp(min.min(max), min.max(max));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_spinner_phase(&mut self, id: WidgetId, phase: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Spinner { phase: ref mut v } => {
                *v = phase;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tick_spinner(
        &mut self,
        id: WidgetId,
        dt_ms: u32,
        cycles_per_sec: f32,
    ) -> Result<(), GuiError> {
        let phase = match self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::Spinner { phase } => phase + (dt_ms as f32 / 1000.0) * cycles_per_sec,
            _ => return Err(GuiError::NotFound),
        };
        self.set_spinner_phase(id, phase)
    }

    pub fn set_dropdown_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Dropdown {
                items,
                selected: ref mut current,
                ..
            } => {
                *current = selected.min(items.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn dropdown_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::Dropdown { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_dropdown_open(&mut self, id: WidgetId, open: bool) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Dropdown {
                open: ref mut is_open,
                ..
            } => {
                if *is_open != open {
                    *is_open = open;
                    self.dirty.add(rect)?;
                    self.push_event(if open {
                        UiEvent::Opened(id)
                    } else {
                        UiEvent::Closed(id)
                    })?;
                }
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn dropdown_open(&self, id: WidgetId) -> Option<bool> {
        match self.node(id)?.kind {
            WidgetKind::Dropdown { open, .. } => Some(open),
            _ => None,
        }
    }

    pub fn set_roller_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Roller {
                items,
                selected: ref mut current,
            } => {
                *current = selected.min(items.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn roller_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::Roller { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_textarea_text(&mut self, id: WidgetId, text: &'a str) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf: ref mut buf,
                text_len: ref mut len,
                cursor: ref mut c,
                ..
            } => {
                let (next_buf, next_len) = textarea_storage_from_str(text);
                *buf = next_buf;
                *len = next_len;
                *c = (*c).min(textarea_text(buf, *len).chars().count());
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn textarea_text(&self, id: WidgetId) -> Option<&str> {
        match &self.node(id)?.kind {
            WidgetKind::TextArea {
                text_buf, text_len, ..
            } => Some(textarea_text(text_buf, *text_len)),
            _ => None,
        }
    }

    pub fn set_textarea_cursor(&mut self, id: WidgetId, cursor: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor: ref mut current,
                ..
            } => {
                let text = textarea_text(&text_buf, text_len);
                *current = cursor.min(text.chars().count());
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn move_textarea_cursor(&mut self, id: WidgetId, delta: i8) -> Result<(), GuiError> {
        let next = self.textarea_cursor(id).ok_or(GuiError::NotFound)? as i32 + delta as i32;
        self.set_textarea_cursor_with_extend(id, next.max(0) as usize, false)
    }

    pub fn move_textarea_cursor_select(&mut self, id: WidgetId, delta: i8) -> Result<(), GuiError> {
        let next = self.textarea_cursor(id).ok_or(GuiError::NotFound)? as i32 + delta as i32;
        self.set_textarea_cursor_with_extend(id, next.max(0) as usize, true)
    }

    pub fn move_textarea_cursor_word(&mut self, id: WidgetId, delta: i8) -> Result<(), GuiError> {
        let (text, cursor) = match &self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                ..
            } => (textarea_text(text_buf, *text_len), *cursor),
            _ => return Err(GuiError::NotFound),
        };
        let next = if delta >= 0 {
            next_word_boundary(text, cursor)
        } else {
            prev_word_boundary(text, cursor)
        };
        self.set_textarea_cursor_with_extend(id, next, false)
    }

    pub fn move_textarea_cursor_word_select(
        &mut self,
        id: WidgetId,
        delta: i8,
    ) -> Result<(), GuiError> {
        let (text, cursor) = match &self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                ..
            } => (textarea_text(text_buf, *text_len), *cursor),
            _ => return Err(GuiError::NotFound),
        };
        let next = if delta >= 0 {
            next_word_boundary(text, cursor)
        } else {
            prev_word_boundary(text, cursor)
        };
        self.set_textarea_cursor_with_extend(id, next, true)
    }

    pub fn set_textarea_cursor_home(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.set_textarea_cursor(id, 0)
    }

    pub fn set_textarea_cursor_end(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let len = self
            .textarea_text(id)
            .map(|text| text.chars().count())
            .ok_or(GuiError::NotFound)?;
        self.set_textarea_cursor(id, len)
    }

    pub fn set_textarea_cursor_line_home(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let (text, cursor, wrap_cols) = self.textarea_line_context(id)?;
        let (row, _) = textarea_row_col_at_cursor(text, cursor, wrap_cols);
        let next = textarea_cursor_from_row_col(text, row, 0, wrap_cols);
        self.set_textarea_cursor_with_extend(id, next, false)
    }

    pub fn set_textarea_cursor_line_home_select(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let (text, cursor, wrap_cols) = self.textarea_line_context(id)?;
        let (row, _) = textarea_row_col_at_cursor(text, cursor, wrap_cols);
        let next = textarea_cursor_from_row_col(text, row, 0, wrap_cols);
        self.set_textarea_cursor_with_extend(id, next, true)
    }

    pub fn set_textarea_cursor_line_end(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let (text, cursor, wrap_cols) = self.textarea_line_context(id)?;
        let (row, _) = textarea_row_col_at_cursor(text, cursor, wrap_cols);
        let row_end = textarea_row_end_col(text, row, wrap_cols);
        let next = textarea_cursor_from_row_col(text, row, row_end, wrap_cols);
        self.set_textarea_cursor_with_extend(id, next, false)
    }

    pub fn set_textarea_cursor_line_end_select(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let (text, cursor, wrap_cols) = self.textarea_line_context(id)?;
        let (row, _) = textarea_row_col_at_cursor(text, cursor, wrap_cols);
        let row_end = textarea_row_end_col(text, row, wrap_cols);
        let next = textarea_cursor_from_row_col(text, row, row_end, wrap_cols);
        self.set_textarea_cursor_with_extend(id, next, true)
    }

    pub fn textarea_cursor(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::TextArea { cursor, .. } => Some(cursor),
            _ => None,
        }
    }

    pub fn set_textarea_selection(
        &mut self,
        id: WidgetId,
        start: usize,
        end: usize,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                selection: ref mut current,
                ..
            } => {
                let text = textarea_text(&text_buf, text_len);
                let len = text.chars().count();
                let start = start.min(len);
                let end = end.min(len);
                *current = Some((start.min(end), start.max(end)));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn clear_textarea_selection(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                selection: ref mut current,
                ..
            } => {
                *current = None;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn textarea_selection(&self, id: WidgetId) -> Option<(usize, usize)> {
        match self.node(id)?.kind {
            WidgetKind::TextArea { selection, .. } => selection,
            _ => None,
        }
    }

    pub fn textarea_cursor_visible(&self, id: WidgetId) -> Option<bool> {
        match self.node(id)?.kind {
            WidgetKind::TextArea { cursor_visible, .. } => Some(cursor_visible),
            _ => None,
        }
    }

    pub fn set_textarea_capabilities(
        &mut self,
        id: WidgetId,
        read_only: bool,
        single_line: bool,
        accept_newline: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                read_only: ref mut ro,
                single_line: ref mut sl,
                accept_newline: ref mut an,
                ..
            } => {
                *ro = read_only;
                *sl = single_line;
                *an = accept_newline && !single_line;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn textarea_insert_char(&mut self, id: WidgetId, ch: char) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let before = self.capture_textarea_snapshot(id)?;
        let mut emit = false;
        if let Some(node) = self.node_mut(id) {
            if let WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                selection,
                read_only,
                single_line,
                accept_newline,
                ..
            } = &mut node.kind
            {
                if *read_only {
                    return Ok(());
                }
                if ch == '\n' && (*single_line || !*accept_newline) {
                    return Ok(());
                }
                let mut chars: heapless::Vec<char, TEXTAREA_CAPACITY> = heapless::Vec::new();
                for c in textarea_text(text_buf, *text_len).chars() {
                    let _ = chars.push(c);
                }
                let original_len = chars.len();
                let original_cursor = *cursor;

                if ch == '\u{8}' {
                    let removed_selection = delete_selection_if_any(&mut chars, cursor, selection);
                    if !removed_selection && *cursor > 0 && *cursor <= chars.len() {
                        chars.remove(*cursor - 1);
                        *cursor -= 1;
                    }
                    if removed_selection
                        || *cursor != original_cursor
                        || chars.len() != original_len
                    {
                        *selection = None;
                        let (next_buf, next_len) = textarea_storage_from_chars(&chars);
                        *text_buf = next_buf;
                        *text_len = next_len;
                        emit = true;
                    }
                } else if ch == '\u{7f}' {
                    let removed_selection = delete_selection_if_any(&mut chars, cursor, selection);
                    if !removed_selection && *cursor < chars.len() {
                        chars.remove(*cursor);
                    }
                    if removed_selection || chars.len() != original_len {
                        *selection = None;
                        let (next_buf, next_len) = textarea_storage_from_chars(&chars);
                        *text_buf = next_buf;
                        *text_len = next_len;
                        emit = true;
                    }
                } else if ch != '\n' || *cursor < TEXTAREA_CAPACITY {
                    if delete_selection_if_any(&mut chars, cursor, selection) {
                        *selection = None;
                    }
                    if chars.len() < TEXTAREA_CAPACITY && *cursor <= chars.len() {
                        let _ = chars.insert(*cursor, ch);
                        *cursor += 1;
                        *selection = None;
                        let (next_buf, next_len) = textarea_storage_from_chars(&chars);
                        *text_buf = next_buf;
                        *text_len = next_len;
                        emit = true;
                    }
                }
            } else {
                return Err(GuiError::NotFound);
            }
        }
        if emit {
            self.push_textarea_undo(id, before);
            self.clear_textarea_redo_for(id);
            self.dirty.add(rect)?;
            self.push_event(UiEvent::TextInput { id, ch })?;
            self.push_event(UiEvent::ValueChanged(id))?;
        }
        Ok(())
    }

    fn textarea_line_context(&self, id: WidgetId) -> Result<(&str, usize, usize), GuiError> {
        let node = self.node(id).ok_or(GuiError::NotFound)?;
        match &node.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                ..
            } => {
                let font = node.style.normal.font;
                let inner_w = node.rect.w.saturating_sub(2);
                let cols = (inner_w / font.advance()).max(1) as usize;
                Ok((textarea_text(text_buf, *text_len), *cursor, cols))
            }
            _ => Err(GuiError::NotFound),
        }
    }

    fn set_textarea_cursor_with_extend(
        &mut self,
        id: WidgetId,
        cursor: usize,
        extend_selection: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor: ref mut current_cursor,
                ref mut selection,
                ..
            } => {
                let len = textarea_text(&text_buf, text_len).chars().count();
                let next = cursor.min(len);
                if extend_selection {
                    let anchor = match *selection {
                        Some((start, end)) => {
                            if *current_cursor == start {
                                end
                            } else {
                                start
                            }
                        }
                        None => *current_cursor,
                    };
                    if anchor == next {
                        *selection = None;
                    } else {
                        *selection = Some((anchor.min(next), anchor.max(next)));
                    }
                } else {
                    *selection = None;
                }
                *current_cursor = next;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    fn capture_textarea_snapshot(&self, id: WidgetId) -> Result<TextareaSnapshot, GuiError> {
        match self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                selection,
                ..
            } => Ok(TextareaSnapshot {
                text_buf,
                text_len,
                cursor,
                selection,
            }),
            _ => Err(GuiError::NotFound),
        }
    }

    fn apply_textarea_snapshot(
        &mut self,
        id: WidgetId,
        snap: TextareaSnapshot,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf: ref mut buf,
                text_len: ref mut len,
                cursor: ref mut c,
                selection: ref mut sel,
                ..
            } => {
                *buf = snap.text_buf;
                *len = snap.text_len;
                *c = snap.cursor;
                *sel = snap.selection;
                self.dirty.add(rect)?;
                self.push_event(UiEvent::ValueChanged(id))
            }
            _ => Err(GuiError::NotFound),
        }
    }

    fn push_textarea_undo(&mut self, id: WidgetId, snapshot: TextareaSnapshot) {
        if self.textarea_undo.len() == self.textarea_undo.capacity() {
            self.textarea_undo.remove(0);
        }
        let _ = self
            .textarea_undo
            .push(TextareaHistoryEntry { id, snapshot });
    }

    fn push_textarea_redo(&mut self, id: WidgetId, snapshot: TextareaSnapshot) {
        if self.textarea_redo.len() == self.textarea_redo.capacity() {
            self.textarea_redo.remove(0);
        }
        let _ = self
            .textarea_redo
            .push(TextareaHistoryEntry { id, snapshot });
    }

    fn clear_textarea_redo_for(&mut self, id: WidgetId) {
        let mut i = 0usize;
        while i < self.textarea_redo.len() {
            if self.textarea_redo[i].id == id {
                self.textarea_redo.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn textarea_undo(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let Some(pos) = self.textarea_undo.iter().rposition(|entry| entry.id == id) else {
            return Ok(());
        };
        let current = self.capture_textarea_snapshot(id)?;
        let prior = self.textarea_undo.remove(pos).snapshot;
        self.push_textarea_redo(id, current);
        self.apply_textarea_snapshot(id, prior)
    }

    fn textarea_redo(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let Some(pos) = self.textarea_redo.iter().rposition(|entry| entry.id == id) else {
            return Ok(());
        };
        let current = self.capture_textarea_snapshot(id)?;
        let next = self.textarea_redo.remove(pos).snapshot;
        self.push_textarea_undo(id, current);
        self.apply_textarea_snapshot(id, next)
    }

    pub fn textarea_backspace(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.textarea_insert_char(id, '\u{8}')
    }

    pub fn textarea_delete_forward(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.textarea_insert_char(id, '\u{7f}')
    }

    pub fn keyboard_selected_key(&self, id: WidgetId) -> Option<char> {
        match self.node(id)?.kind {
            WidgetKind::Keyboard {
                keys,
                alt_keys,
                selected,
                layout,
                ..
            } => keyboard_char_for_layout(keys, alt_keys, selected, layout),
            _ => None,
        }
    }

    pub fn keyboard_layout(&self, id: WidgetId) -> Option<KeyboardLayout> {
        match self.node(id)?.kind {
            WidgetKind::Keyboard { layout, .. } => Some(layout),
            _ => None,
        }
    }

    pub fn set_keyboard_layout(
        &mut self,
        id: WidgetId,
        layout: KeyboardLayout,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Keyboard {
                layout: ref mut current,
                ..
            } => {
                *current = layout;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_keyboard_target(
        &mut self,
        id: WidgetId,
        target: Option<WidgetId>,
    ) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Keyboard {
                target: ref mut current,
                ..
            } => {
                *current = target;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_gauge_value(&mut self, id: WidgetId, value: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Gauge {
                value: ref mut v,
                min,
                max,
                ..
            }
            | WidgetKind::ArcGauge {
                value: ref mut v,
                min,
                max,
                ..
            }
            | WidgetKind::GaugeNeedle {
                value: ref mut v,
                min,
                max,
                ..
            } => {
                *v = value.clamp(min.min(max), min.max(max));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_gauge_ticks(
        &mut self,
        id: WidgetId,
        major_ticks: u8,
        minor_ticks: u8,
        show_value: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Gauge {
                major_ticks: ref mut major,
                minor_ticks: ref mut minor,
                show_value: ref mut show,
                ..
            }
            | WidgetKind::ArcGauge {
                major_ticks: ref mut major,
                minor_ticks: ref mut minor,
                show_value: ref mut show,
                ..
            } => {
                *major = major_ticks.max(1);
                *minor = minor_ticks.max(1);
                *show = show_value;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_widget_rect(&mut self, id: WidgetId, rect: Rect) -> Result<(), GuiError> {
        let old = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.rect = rect;
        self.dirty.add(old)?;
        self.mark_subtree_dirty(id)?;
        Ok(())
    }

    pub fn set_widget_x(&mut self, id: WidgetId, x: i32) -> Result<(), GuiError> {
        let mut rect = self.node(id).ok_or(GuiError::NotFound)?.rect;
        rect.x = x;
        self.set_widget_rect(id, rect)
    }

    pub fn set_widget_y(&mut self, id: WidgetId, y: i32) -> Result<(), GuiError> {
        let mut rect = self.node(id).ok_or(GuiError::NotFound)?.rect;
        rect.y = y;
        self.set_widget_rect(id, rect)
    }

    pub fn set_widget_width(&mut self, id: WidgetId, w: u32) -> Result<(), GuiError> {
        let mut rect = self.node(id).ok_or(GuiError::NotFound)?.rect;
        rect.w = w.max(1);
        self.set_widget_rect(id, rect)
    }

    pub fn set_widget_height(&mut self, id: WidgetId, h: u32) -> Result<(), GuiError> {
        let mut rect = self.node(id).ok_or(GuiError::NotFound)?.rect;
        rect.h = h.max(1);
        self.set_widget_rect(id, rect)
    }

    pub fn set_widget_opacity(&mut self, id: WidgetId, opacity: u8) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.style.normal.opacity = opacity;
        node.style.focused.opacity = opacity;
        node.style.pressed.opacity = opacity;
        node.style.disabled.opacity = opacity;
        self.mark_subtree_dirty(id)
    }

    pub fn set_widget_corner_radius(&mut self, id: WidgetId, radius: u8) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.style.normal.corner_radius = radius;
        node.style.focused.corner_radius = radius;
        node.style.pressed.corner_radius = radius;
        node.style.disabled.corner_radius = radius;
        self.mark_subtree_dirty(id)
    }

    pub fn set_widget_accent(&mut self, id: WidgetId, accent: Rgb565) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.style.normal.accent = accent;
        node.style.focused.accent = accent;
        node.style.pressed.accent = accent;
        node.style.disabled.accent = accent;
        self.mark_subtree_dirty(id)
    }

    pub fn set_widget_parent(
        &mut self,
        id: WidgetId,
        parent: Option<WidgetId>,
    ) -> Result<(), GuiError> {
        if let Some(parent) = parent {
            self.node(parent).ok_or(GuiError::NotFound)?;
        }
        self.node_mut(id).ok_or(GuiError::NotFound)?.parent = parent;
        self.mark_subtree_dirty(id)?;
        Ok(())
    }

    pub fn add_child(&mut self, parent: WidgetId, child: WidgetId) -> Result<(), GuiError> {
        self.set_widget_parent(child, Some(parent))
    }

    pub fn children_of(&self, parent: WidgetId) -> impl Iterator<Item = &WidgetNode<'a>> + '_ {
        self.widgets
            .iter()
            .filter(move |node| node.parent == Some(parent))
    }

    pub fn absolute_rect(&self, id: WidgetId) -> Option<Rect> {
        let node = self.node(id)?;
        let mut rect = node.rect;
        let mut parent = node.parent;
        let mut depth = 0;
        while let Some(parent_id) = parent {
            if depth >= NODES {
                return None;
            }
            let parent_node = self.node(parent_id)?;
            rect.x += parent_node.rect.x;
            rect.y += parent_node.rect.y;
            parent = parent_node.parent;
            depth += 1;
        }
        Some(rect)
    }

    pub fn set_flag(
        &mut self,
        id: WidgetId,
        flag: WidgetFlags,
        enabled: bool,
    ) -> Result<(), GuiError> {
        let was_set = self.has_flag(id, flag)?;
        let before_state = self.current_visual_state(id);
        self.mark_subtree_dirty(id)?;
        self.node_mut(id)
            .ok_or(GuiError::NotFound)?
            .flags
            .set(flag, enabled);
        if flag == WidgetFlags::DISABLED && enabled {
            if self.pressed.is_some_and(|pressed| pressed.id == id) {
                self.pressed = None;
            }
        }
        self.mark_subtree_dirty(id)?;
        if self
            .focus
            .is_some_and(|focus| !self.effective_focusable(focus))
        {
            self.focus = None;
            self.ensure_focus();
        }
        if flag == WidgetFlags::DISABLED && was_set != enabled {
            let after_state = self.current_visual_state(id);
            self.start_state_transition(id, before_state, after_state);
        }
        Ok(())
    }

    pub fn has_flag(&self, id: WidgetId, flag: WidgetFlags) -> Result<bool, GuiError> {
        Ok(self
            .node(id)
            .ok_or(GuiError::NotFound)?
            .flags
            .contains(flag))
    }

    pub fn insert_flag(&mut self, id: WidgetId, flag: WidgetFlags) -> Result<(), GuiError> {
        self.set_flag(id, flag, true)
    }

    pub fn remove_flag(&mut self, id: WidgetId, flag: WidgetFlags) -> Result<(), GuiError> {
        self.set_flag(id, flag, false)
    }

    pub fn set_hidden(&mut self, id: WidgetId, hidden: bool) -> Result<(), GuiError> {
        self.set_flag(id, WidgetFlags::HIDDEN, hidden)
    }

    pub fn set_disabled(&mut self, id: WidgetId, disabled: bool) -> Result<(), GuiError> {
        self.set_flag(id, WidgetFlags::DISABLED, disabled)
    }

    pub fn set_clickable(&mut self, id: WidgetId, clickable: bool) -> Result<(), GuiError> {
        self.set_flag(id, WidgetFlags::CLICKABLE, clickable)
    }

    pub fn set_scrollable(&mut self, id: WidgetId, scrollable: bool) -> Result<(), GuiError> {
        self.set_flag(id, WidgetFlags::SCROLLABLE, scrollable)
    }

    pub fn set_visible(&mut self, id: WidgetId, visible: bool) -> Result<(), GuiError> {
        self.set_hidden(id, !visible)
    }

    pub fn set_enabled(&mut self, id: WidgetId, enabled: bool) -> Result<(), GuiError> {
        self.set_disabled(id, !enabled)
    }

    pub fn event_path<const M: usize>(
        &self,
        target: WidgetId,
        out: &mut heapless::Vec<EventContext, M>,
    ) -> Result<usize, GuiError> {
        self.node(target).ok_or(GuiError::NotFound)?;
        out.clear();

        let mut chain = heapless::Vec::<WidgetId, NODES>::new();
        let mut current = Some(target);
        while let Some(id) = current {
            chain.push(id).map_err(|_| GuiError::WidgetsFull)?;
            current = self.node(id).ok_or(GuiError::NotFound)?.parent;
        }

        for id in chain.iter().rev().copied().filter(|&id| id != target) {
            out.push(EventContext {
                target,
                current: id,
                phase: EventPhase::Capture,
            })
            .map_err(|_| GuiError::EventsFull)?;
        }

        out.push(EventContext {
            target,
            current: target,
            phase: EventPhase::Target,
        })
        .map_err(|_| GuiError::EventsFull)?;

        for id in chain.iter().copied().skip(1) {
            out.push(EventContext {
                target,
                current: id,
                phase: EventPhase::Bubble,
            })
            .map_err(|_| GuiError::EventsFull)?;
        }

        Ok(out.len())
    }

    pub fn widget_event_path<const M: usize>(
        &self,
        target: WidgetId,
        kind: WidgetEventKind,
        out: &mut heapless::Vec<WidgetEvent, M>,
    ) -> Result<usize, GuiError> {
        self.node(target).ok_or(GuiError::NotFound)?;
        out.clear();

        let mut chain = heapless::Vec::<WidgetId, NODES>::new();
        let mut current = Some(target);
        while let Some(id) = current {
            chain.push(id).map_err(|_| GuiError::WidgetsFull)?;
            current = self.node(id).ok_or(GuiError::NotFound)?.parent;
        }

        for id in chain.iter().rev().copied().filter(|&id| id != target) {
            out.push(WidgetEvent {
                target,
                current: id,
                phase: EventPhase::Capture,
                kind,
            })
            .map_err(|_| GuiError::EventsFull)?;
        }

        out.push(WidgetEvent {
            target,
            current: target,
            phase: EventPhase::Target,
            kind,
        })
        .map_err(|_| GuiError::EventsFull)?;

        if self.has_flag(target, WidgetFlags::EVENT_BUBBLE)? {
            for id in chain.iter().copied().skip(1) {
                out.push(WidgetEvent {
                    target,
                    current: id,
                    phase: EventPhase::Bubble,
                    kind,
                })
                .map_err(|_| GuiError::EventsFull)?;
            }
        }

        Ok(out.len())
    }

    pub fn dispatch_widget_event<const M: usize, F>(
        &self,
        target: WidgetId,
        kind: WidgetEventKind,
        scratch: &mut heapless::Vec<WidgetEvent, M>,
        mut handler: F,
    ) -> Result<(), GuiError>
    where
        F: FnMut(WidgetEvent) -> EventPolicy,
    {
        self.widget_event_path(target, kind, scratch)?;
        for event in scratch.iter().copied() {
            let handler_policy = handler(event);
            if matches!(handler_policy, EventPolicy::Stop)
                || self.stop_due_to_builtin_widget_behavior(event)
                || self.stop_due_to_registered_policy(event)
            {
                break;
            }
        }
        Ok(())
    }

    pub fn mark_subtree_dirty(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        self.dirty.add(rect)?;
        let child_ids: heapless::Vec<WidgetId, NODES> = self
            .widgets
            .iter()
            .filter(|node| node.parent == Some(id))
            .map(|node| node.id)
            .collect();
        for child in child_ids {
            self.mark_subtree_dirty(child)?;
        }
        Ok(())
    }

    pub fn set_focus_group(&mut self, id: WidgetId, group: FocusGroupId) -> Result<(), GuiError> {
        self.node_mut(id).ok_or(GuiError::NotFound)?.focus_group = group;
        Ok(())
    }

    pub fn set_active_focus_group(&mut self, group: Option<FocusGroupId>) {
        self.active_focus_group = group;
        if let Some(focus) = self.focus {
            let still_valid = self.node(focus).is_some_and(|node| {
                group.is_none_or(|active| node.focus_group == active)
                    && self.effective_focusable(focus)
            });
            if !still_valid {
                self.focus = None;
                self.ensure_focus();
            }
        }
    }

    pub fn apply_layout(
        &mut self,
        layout: LinearLayout,
        area: Rect,
        ids: &[WidgetId],
    ) -> Result<usize, GuiError> {
        let mut rects = [Rect::empty(); 16];
        let count = layout.arrange(area, ids.len().min(rects.len()), &mut rects);
        for (id, rect) in ids.iter().copied().zip(rects.into_iter()).take(count) {
            self.set_widget_rect(id, rect)?;
        }
        Ok(count)
    }

    pub fn apply_layout_flex(
        &mut self,
        layout: LinearLayout,
        area: Rect,
        ids: &[WidgetId],
        items: &[LayoutItem],
        enable_grow: bool,
        enable_shrink: bool,
    ) -> Result<usize, GuiError> {
        let mut rects = [Rect::empty(); 16];
        let count = ids.len().min(items.len()).min(rects.len());
        let laid_out = layout.arrange_items_flex(
            area,
            &items[..count],
            &mut rects,
            enable_grow,
            enable_shrink,
        );
        for (id, rect) in ids.iter().copied().zip(rects.into_iter()).take(laid_out) {
            self.set_widget_rect(id, rect)?;
        }
        Ok(laid_out)
    }

    pub fn apply_layout_intrinsic(
        &mut self,
        layout: LinearLayout,
        area: Rect,
        ids: &[WidgetId],
    ) -> Result<usize, GuiError> {
        self.apply_layout_intrinsic_with_cross(layout, area, ids, false)
    }

    pub fn apply_layout_intrinsic_with_cross(
        &mut self,
        layout: LinearLayout,
        area: Rect,
        ids: &[WidgetId],
        preserve_cross: bool,
    ) -> Result<usize, GuiError> {
        let mut specs = [LayoutItem::fill(); 16];
        let mut rects = [Rect::empty(); 16];
        let count = ids.len().min(specs.len()).min(rects.len());

        for (idx, id) in ids.iter().copied().take(count).enumerate() {
            let (w, h) = self.intrinsic_size(id).ok_or(GuiError::NotFound)?;
            specs[idx] = match layout.axis {
                Axis::Horizontal => LayoutItem::length(w).with_cross(if preserve_cross {
                    crate::layout::Constraint::Length(h)
                } else {
                    crate::layout::Constraint::Fill(1)
                }),
                Axis::Vertical => LayoutItem::length(h).with_cross(if preserve_cross {
                    crate::layout::Constraint::Length(w)
                } else {
                    crate::layout::Constraint::Fill(1)
                }),
            };
        }

        let laid_out = layout.arrange_items(area, &specs[..count], &mut rects);
        for (id, rect) in ids.iter().copied().zip(rects.into_iter()).take(laid_out) {
            self.set_widget_rect(id, rect)?;
        }
        Ok(laid_out)
    }

    pub fn render<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        let mut ctx = RenderCtx::new(target, self.viewport);
        ctx.set_quality(self.render_quality);
        self.render_into(&mut ctx, 0, 0, 255)
    }

    pub fn render_dirty<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        if self.dirty.is_empty() {
            return Ok(());
        }

        for dirty in self.dirty.as_slice() {
            let mut ctx = RenderCtx::with_dirty(target, self.viewport, *dirty);
            ctx.set_quality(self.render_quality);
            self.render_into(&mut ctx, 0, 0, 255)?;
        }
        Ok(())
    }

    pub fn render_with_offset<D>(
        &self,
        target: &mut D,
        offset_x: i32,
        offset_y: i32,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        self.render_with_offset_and_opacity(target, offset_x, offset_y, 255)
    }

    pub fn render_with_offset_and_opacity<D>(
        &self,
        target: &mut D,
        offset_x: i32,
        offset_y: i32,
        opacity: u8,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        let mut ctx = RenderCtx::new(target, self.viewport);
        ctx.set_quality(self.render_quality);
        self.render_into(&mut ctx, offset_x, offset_y, opacity)
    }

    pub fn render_with_offset_opacity_and_clip<D>(
        &self,
        target: &mut D,
        offset_x: i32,
        offset_y: i32,
        opacity: u8,
        clip: Rect,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        let mut ctx = RenderCtx::new(target, self.viewport);
        ctx.set_quality(self.render_quality);
        let old_clip = ctx.clip();
        ctx.set_clip(old_clip.intersection(clip));
        self.render_into(&mut ctx, offset_x, offset_y, opacity)
    }

    pub fn handle_input(&mut self, event: InputEvent) -> Result<(), GuiError> {
        match event {
            InputEvent::Home => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.set_textarea_cursor_line_home(id)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::End => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.set_textarea_cursor_line_end(id)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::WordLeft => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_word(id, -1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::WordRight => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_word(id, 1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::Undo => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.textarea_undo(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::Redo => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.textarea_redo(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::SelectLeft => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_select(id, -1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectRight => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_select(id, 1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectHome => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.set_textarea_cursor_line_home_select(id)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectEnd => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.set_textarea_cursor_line_end_select(id)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectWordLeft => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_word_select(id, -1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectWordRight => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_word_select(id, 1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::Up => {
                if !self.adjust_focused_selection(-1)? {
                    self.focus_prev()?;
                }
                Ok(())
            }
            InputEvent::Down => {
                if !self.adjust_focused_selection(1)? {
                    self.focus_next()?;
                }
                Ok(())
            }
            InputEvent::Left => {
                if !self.adjust_focused_scalar(-1.0)? {
                    self.focus_prev()?;
                }
                Ok(())
            }
            InputEvent::Right => {
                if !self.adjust_focused_scalar(1.0)? {
                    self.focus_next()?;
                }
                Ok(())
            }
            InputEvent::Encoder { delta } if delta > 0 => {
                if !self.adjust_focused_selection(1)? {
                    self.focus_next()?;
                }
                Ok(())
            }
            InputEvent::Encoder { delta } if delta < 0 => {
                if !self.adjust_focused_selection(-1)? {
                    self.focus_prev()?;
                }
                Ok(())
            }
            InputEvent::Select => {
                if let Some(id) = self.focus {
                    match self.key_bindings_for(id).select {
                        KeyBindingAction::Default | KeyBindingAction::Activate => {
                            self.handle_select_activation(id)?
                        }
                        KeyBindingAction::Back => self.handle_back_action()?,
                        KeyBindingAction::Ignore => {}
                    }
                }
                Ok(())
            }
            InputEvent::SelectPressed => {
                if let Some(id) = self.focus {
                    if self.key_input_policy_for(id).raw_select {
                        self.dispatch_key_pressed(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::SelectReleased => {
                if let Some(id) = self.focus {
                    if self.key_input_policy_for(id).raw_select {
                        self.dispatch_key_released(id)?;
                        self.handle_select_activation(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::Back => {
                if let Some(id) = self.focus {
                    match self.key_bindings_for(id).back {
                        KeyBindingAction::Default | KeyBindingAction::Back => {
                            self.handle_back_action()
                        }
                        KeyBindingAction::Activate => self.handle_select_activation(id),
                        KeyBindingAction::Ignore => Ok(()),
                    }
                } else {
                    self.handle_back_action()
                }
            }
            InputEvent::BackPressed => {
                if let Some(id) = self.focus {
                    if self.key_input_policy_for(id).raw_back {
                        self.dispatch_key_pressed(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::BackReleased => {
                if let Some(id) = self.focus {
                    if self.key_input_policy_for(id).raw_back {
                        self.dispatch_key_released(id)?;
                        return self.handle_back_action();
                    }
                }
                Ok(())
            }
            InputEvent::Pointer {
                x,
                y,
                state: PointerState::Pressed,
                ..
            } => self.handle_pointer_pressed(x, y),
            InputEvent::Pointer {
                x,
                y,
                state: PointerState::Released,
                ..
            } => self.handle_pointer_released(x, y),
            InputEvent::Pointer {
                x,
                y,
                state: PointerState::Moved,
                ..
            } => self.handle_pointer_moved(x, y),
            _ => Ok(()),
        }
    }

    pub fn tick_input(&mut self, dt_ms: u32) -> Result<(), GuiError> {
        if self.last_select_id.is_some() {
            self.select_elapsed_ms = self.select_elapsed_ms.saturating_add(dt_ms);
            if self.select_elapsed_ms > self.select_double_window_ms {
                self.last_select_id = None;
                self.select_elapsed_ms = 0;
            }
        }
        if self.last_pointer_id.is_some() {
            self.pointer_elapsed_ms = self.pointer_elapsed_ms.saturating_add(dt_ms);
            if self.pointer_elapsed_ms > self.pointer_double_window_ms {
                self.last_pointer_id = None;
                self.pointer_elapsed_ms = 0;
            }
        }
        self.tick_state_transitions(dt_ms)?;
        if let Some(mut inertia) = self.inertia_scroll {
            if inertia.velocity.abs() < self.scroll_physics.velocity_threshold {
                self.inertia_scroll = None;
            } else {
                let current = self.scroll_offset(inertia.id).unwrap_or(0);
                let delta = (inertia.velocity * (dt_ms as f32 / 16.0)).round() as i32;
                if delta != 0 {
                    let next = current.saturating_sub(delta);
                    if next != current {
                        self.set_scroll_offset(inertia.id, next)?;
                        self.push_event(UiEvent::Scroll {
                            id: inertia.id,
                            delta: next - current,
                        })?;
                    }
                }
                inertia.velocity *= self
                    .scroll_physics
                    .velocity_decay
                    .powf((dt_ms as f32 / 16.0).max(1.0));
                self.inertia_scroll = Some(inertia);
            }
        }
        self.tick_textarea_cursor_blink(dt_ms)?;
        let Some(mut pressed) = self.pressed else {
            return Ok(());
        };
        if !self.effective_visible(pressed.id) || !self.effective_enabled(pressed.id) {
            self.pressed = None;
            return Ok(());
        }
        let timing = self.press_timing_for(pressed.id);
        pressed.elapsed_ms = pressed.elapsed_ms.saturating_add(dt_ms);
        pressed.repeat_elapsed_ms = pressed.repeat_elapsed_ms.saturating_add(dt_ms);
        if !pressed.long_emitted && pressed.elapsed_ms >= timing.long_press_ms {
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(
                pressed.id,
                WidgetEventKind::LongPressed,
                &mut events,
                |_| EventPolicy::Continue,
            )?;
            self.push_event(UiEvent::LongPressed(pressed.id))?;
            pressed.long_emitted = true;
        }
        if pressed.repeat_elapsed_ms >= timing.repeat_delay_ms
            && self.repeatable_widget(pressed.id)
            && pressed.long_emitted
        {
            let intervals =
                (pressed.repeat_elapsed_ms - timing.repeat_delay_ms) / timing.repeat_interval_ms;
            if intervals > 0 {
                self.dispatch_repeat_activation(pressed.id)?;
                pressed.repeat_elapsed_ms = timing.repeat_delay_ms;
            }
        }
        self.pressed = Some(pressed);
        Ok(())
    }

    pub fn pop_event(&mut self) -> Option<UiEvent> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0))
        }
    }

    pub fn set_event_filter(
        &mut self,
        id: WidgetId,
        filter: UiEventFilter,
    ) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some((_, current)) = self
            .subscriptions
            .iter_mut()
            .find(|(sub_id, _)| *sub_id == id)
        {
            *current = filter;
            return Ok(());
        }
        self.subscriptions
            .push((id, filter))
            .map_err(|_| GuiError::WidgetsFull)
    }

    pub fn event_filter(&self, id: WidgetId) -> Result<UiEventFilter, GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        Ok(self
            .subscriptions
            .iter()
            .find(|(sub_id, _)| *sub_id == id)
            .map(|(_, filter)| *filter)
            .unwrap_or(UiEventFilter::ALL))
    }

    pub fn clear_event_filter(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some(pos) = self
            .subscriptions
            .iter()
            .position(|(sub_id, _)| *sub_id == id)
        {
            self.subscriptions.remove(pos);
        }
        Ok(())
    }

    pub fn set_dispatch_policy(
        &mut self,
        id: WidgetId,
        policy: WidgetDispatchPolicy,
    ) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some((_, current)) = self
            .dispatch_policies
            .iter_mut()
            .find(|(policy_id, _)| *policy_id == id)
        {
            *current = policy;
            return Ok(());
        }
        self.dispatch_policies
            .push((id, policy))
            .map_err(|_| GuiError::WidgetsFull)
    }

    pub fn dispatch_policy(&self, id: WidgetId) -> Result<Option<WidgetDispatchPolicy>, GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        Ok(self
            .dispatch_policies
            .iter()
            .find(|(policy_id, _)| *policy_id == id)
            .map(|(_, policy)| *policy))
    }

    pub fn clear_dispatch_policy(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some(pos) = self
            .dispatch_policies
            .iter()
            .position(|(policy_id, _)| *policy_id == id)
        {
            self.dispatch_policies.remove(pos);
        }
        Ok(())
    }

    fn add_widget<S>(
        &mut self,
        rect: Rect,
        kind: WidgetKind<'a>,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = WidgetId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1).max(1);
        let node = WidgetNode::new(id, rect, kind, style);
        self.widgets.push(node).map_err(|_| GuiError::WidgetsFull)?;
        self.dirty.add(rect)?;
        Ok(id)
    }

    fn render_into<D>(
        &self,
        ctx: &mut RenderCtx<'_, D>,
        offset_x: i32,
        offset_y: i32,
        opacity: u8,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        for node in &self.widgets {
            if !self.effective_visible(node.id) {
                continue;
            }
            let Some(base_rect) = self.absolute_rect(node.id) else {
                continue;
            };
            let rect = Rect::new(
                base_rect.x + offset_x,
                base_rect.y + offset_y,
                base_rect.w,
                base_rect.h,
            );
            let base_clip = self.inherited_clip(node.id).unwrap_or(self.viewport);
            let clip = Rect::new(
                base_clip.x + offset_x,
                base_clip.y + offset_y,
                base_clip.w,
                base_clip.h,
            );
            if rect.intersection(clip).is_empty() {
                continue;
            }
            let old_clip = ctx.clip();
            ctx.set_clip(old_clip.intersection(clip));
            let state = if self.pressed.is_some_and(|pressed| pressed.id == node.id) {
                VisualState::Pressed
            } else if Some(node.id) == self.focus {
                VisualState::Focused
            } else if !self.effective_enabled(node.id) {
                VisualState::Disabled
            } else {
                VisualState::Normal
            };
            let mut render_node = *node;
            let class_style = node.style_class.and_then(|class| {
                self.class_styles
                    .iter()
                    .find(|(id, _)| *id == class)
                    .map(|(_, style)| *style)
            });
            let resolve_state_style = |vs: VisualState| {
                class_style
                    .map(|style| style.resolve(vs))
                    .unwrap_or_else(|| render_node.style.resolve(vs))
            };
            let active_style = if let Some((from, to, t)) = self.state_transition_progress(node.id)
            {
                lerp_style(resolve_state_style(from), resolve_state_style(to), t)
            } else {
                resolve_state_style(state)
            };
            render_node.style = render_node.style.with_state_override(state, active_style);
            if opacity < 255 {
                let apply = |v: u8| -> u8 { ((v as u16 * opacity as u16) / 255) as u8 };
                render_node.style.normal.opacity = apply(render_node.style.normal.opacity);
                render_node.style.focused.opacity = apply(render_node.style.focused.opacity);
                render_node.style.pressed.opacity = apply(render_node.style.pressed.opacity);
                render_node.style.disabled.opacity = apply(render_node.style.disabled.opacity);
            }
            render_node.render_at(ctx, rect, state)?;
            ctx.set_clip(old_clip);
        }
        Ok(())
    }

    fn node(&self, id: WidgetId) -> Option<&WidgetNode<'a>> {
        self.widgets.iter().find(|node| node.id == id)
    }

    fn intrinsic_size(&self, id: WidgetId) -> Option<(u32, u32)> {
        let node = self.node(id)?;
        let style = node.style.resolve(VisualState::Normal);
        let pad_x = style.padding.left.max(0) as u32 + style.padding.right.max(0) as u32;
        let pad_y = style.padding.top.max(0) as u32 + style.padding.bottom.max(0) as u32;
        let border = style.border.width as u32 * 2;
        let text_width = |text: &str| text.chars().count() as u32 * style.font.advance();
        let text_height = style.font.line_height();

        let content = match node.kind {
            WidgetKind::Label(text) => (text_width(text), text_height),
            WidgetKind::Button(text) => (text_width(text).saturating_add(6), text_height),
            WidgetKind::Toggle { label, .. } => (text_width(label).saturating_add(12), text_height),
            WidgetKind::Checkbox { label, .. } => {
                (text_width(label).saturating_add(10), text_height)
            }
            WidgetKind::ValueLabel { label, .. } => {
                (text_width(label).saturating_add(16), text_height)
            }
            WidgetKind::IconButton { label, .. } => {
                (text_width(label).saturating_add(10), text_height)
            }
            WidgetKind::Tabs { labels, .. } => {
                let max = labels.iter().map(|s| text_width(s)).max().unwrap_or(0);
                (
                    max.saturating_mul(labels.len() as u32).saturating_add(4),
                    text_height,
                )
            }
            WidgetKind::Dialog { title, body } => {
                let w = text_width(title).max(text_width(body)).saturating_add(8);
                (w, text_height.saturating_mul(3))
            }
            WidgetKind::Toast { text, .. } => (
                text_width(text).saturating_add(8),
                text_height.saturating_add(2),
            ),
            WidgetKind::Dropdown {
                items, selected, ..
            } => (
                text_width(items.get(selected).copied().unwrap_or("-")).saturating_add(10),
                text_height.saturating_add(2),
            ),
            WidgetKind::TextArea {
                text_buf,
                text_len,
                placeholder,
                ..
            } => (
                text_width(if text_len == 0 {
                    placeholder
                } else {
                    textarea_text(&text_buf, text_len)
                })
                .saturating_add(10),
                text_height.saturating_add(4),
            ),
            WidgetKind::Keyboard { keys, cols, .. } => {
                let cols = cols.max(1) as u32;
                let rows = (keys.len() as u32).div_ceil(cols).max(1);
                (
                    cols.saturating_mul(style.font.advance().saturating_add(4)),
                    rows.saturating_mul(style.font.line_height().saturating_add(4)),
                )
            }
            WidgetKind::List {
                items,
                visible_rows,
                ..
            } => {
                let max = items.iter().map(|s| text_width(s)).max().unwrap_or(0);
                (
                    max.saturating_add(6),
                    (text_height.saturating_add(2))
                        .saturating_mul(visible_rows as u32)
                        .max(text_height),
                )
            }
            WidgetKind::Menu { items, .. } => {
                let max = items.iter().map(|s| text_width(s)).max().unwrap_or(0);
                (
                    max.saturating_add(6),
                    (text_height.saturating_add(2))
                        .saturating_mul(items.len() as u32)
                        .max(text_height),
                )
            }
            _ => (node.rect.w.max(1), node.rect.h.max(1)),
        };

        Some((
            content
                .0
                .saturating_add(pad_x)
                .saturating_add(border)
                .max(1),
            content
                .1
                .saturating_add(pad_y)
                .saturating_add(border)
                .max(1),
        ))
    }

    fn node_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode<'a>> {
        self.widgets.iter_mut().find(|node| node.id == id)
    }

    fn effective_visible(&self, id: WidgetId) -> bool {
        let mut current = Some(id);
        let mut depth = 0;
        while let Some(widget_id) = current {
            if depth >= NODES {
                return false;
            }
            let Some(node) = self.node(widget_id) else {
                return false;
            };
            if node.hidden() {
                return false;
            }
            current = node.parent;
            depth += 1;
        }
        true
    }

    fn inherited_clip(&self, id: WidgetId) -> Option<Rect> {
        let mut clip = self.viewport;
        let mut chain = heapless::Vec::<WidgetId, NODES>::new();
        let mut current = Some(id);
        while let Some(widget_id) = current {
            chain.push(widget_id).ok()?;
            current = self.node(widget_id)?.parent;
        }
        for widget_id in chain.iter().rev().copied() {
            let node = self.node(widget_id)?;
            if widget_id == id || node.clips_children() {
                clip = clip.intersection(self.absolute_rect(widget_id)?);
            }
            if clip.is_empty() {
                return None;
            }
        }
        Some(clip)
    }

    fn effective_enabled(&self, id: WidgetId) -> bool {
        let mut current = Some(id);
        let mut depth = 0;
        while let Some(widget_id) = current {
            if depth >= NODES {
                return false;
            }
            let Some(node) = self.node(widget_id) else {
                return false;
            };
            if node.disabled() {
                return false;
            }
            current = node.parent;
            depth += 1;
        }
        true
    }

    fn effective_focusable(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| {
            self.node_in_active_group(node)
                && node.focusable()
                && self.effective_visible(id)
                && self.effective_enabled(id)
        })
    }

    fn ensure_focus(&mut self) {
        if self.focus.is_none() {
            self.focus = self
                .widgets
                .iter()
                .find(|node| self.effective_focusable(node.id))
                .map(|n| n.id);
        }
    }

    fn focus_next(&mut self) -> Result<(), GuiError> {
        self.move_focus(1)
    }

    fn focus_prev(&mut self) -> Result<(), GuiError> {
        self.move_focus(-1)
    }

    fn move_focus(&mut self, delta: i8) -> Result<(), GuiError> {
        let focusable = self
            .widgets
            .iter()
            .filter(|node| self.effective_focusable(node.id))
            .count();
        if focusable == 0 {
            return Ok(());
        }

        let current_pos = self
            .widgets
            .iter()
            .filter(|node| self.effective_focusable(node.id))
            .position(|node| Some(node.id) == self.focus)
            .unwrap_or(0);

        let next_pos = if delta >= 0 {
            (current_pos + 1) % focusable
        } else if current_pos == 0 {
            focusable - 1
        } else {
            current_pos - 1
        };

        let next = self
            .widgets
            .iter()
            .filter(|node| self.effective_focusable(node.id))
            .nth(next_pos)
            .map(|node| node.id);
        self.set_focus(next)
    }

    fn adjust_focused_selection(&mut self, delta: i8) -> Result<bool, GuiError> {
        let Some(id) = self.focus else {
            return Ok(false);
        };

        let mut changed_rect = None;
        let mut changed = false;

        if let Some(node) = self.node_mut(id) {
            match node.kind {
                WidgetKind::Menu {
                    items,
                    selected: ref mut current,
                } => {
                    if items.is_empty() {
                        return Ok(true);
                    }
                    changed = bump_index(current, items.len(), delta);
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Dropdown {
                    items,
                    selected: ref mut current,
                    open,
                } => {
                    if !open {
                        return Ok(false);
                    }
                    if items.is_empty() {
                        return Ok(true);
                    }
                    changed = bump_index(current, items.len(), delta);
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Roller {
                    items,
                    selected: ref mut current,
                } => {
                    if items.is_empty() {
                        return Ok(true);
                    }
                    changed = bump_index(current, items.len(), delta);
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Keyboard {
                    keys,
                    selected: ref mut current,
                    ..
                } => {
                    if keys.is_empty() {
                        return Ok(true);
                    }
                    changed = bump_index(current, keys.len(), delta);
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::List {
                    items,
                    selected: ref mut current,
                    ref mut offset,
                    visible_rows,
                } => {
                    if items.is_empty() {
                        return Ok(true);
                    }
                    let mut state = ListState::new(*current, *offset, visible_rows);
                    changed = state.bump(items.len(), delta);
                    *current = state.selected;
                    *offset = state.offset;
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::ScrollView {
                    offset_y: ref mut offset,
                    content_h,
                } => {
                    let mut state = ScrollState::new(*offset, content_h);
                    changed = state.scroll_by(delta as i32 * 8);
                    *offset = state.offset_y;
                    changed_rect = changed.then_some(node.rect);
                }
                _ => return Ok(false),
            }
        }

        if let Some(rect) = changed_rect {
            self.dirty.add(rect)?;
        }
        if changed {
            self.push_event(UiEvent::ValueChanged(id))?;
        }
        Ok(true)
    }

    fn adjust_focused_scalar(&mut self, direction: f32) -> Result<bool, GuiError> {
        let Some(id) = self.focus else {
            return Ok(false);
        };

        let mut changed_rect = None;
        let mut changed = false;

        if let Some(node) = self.node_mut(id) {
            match node.kind {
                WidgetKind::Slider {
                    value: ref mut current,
                    min,
                    max,
                } => {
                    let mut state = SliderState::new(*current, min, max);
                    changed = state.step_by(direction);
                    *current = state.value;
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Tabs {
                    labels,
                    selected: ref mut current,
                } => {
                    if labels.is_empty() {
                        return Ok(true);
                    }
                    let mut state = TabsState::new(*current);
                    changed = state.bump(labels.len(), if direction >= 0.0 { 1 } else { -1 });
                    *current = state.selected;
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::TextArea {
                    text_buf,
                    text_len,
                    cursor: ref mut current,
                    ..
                } => {
                    let text = textarea_text(&text_buf, text_len);
                    let len = text.chars().count();
                    if direction >= 0.0 {
                        let next = (*current + 1).min(len);
                        changed = next != *current;
                        *current = next;
                    } else {
                        let next = current.saturating_sub(1);
                        changed = next != *current;
                        *current = next;
                    }
                    changed_rect = changed.then_some(node.rect);
                }
                _ => return Ok(false),
            }
        }

        if let Some(rect) = changed_rect {
            self.dirty.add(rect)?;
        }
        if changed {
            self.push_event(UiEvent::ValueChanged(id))?;
        }
        Ok(true)
    }

    fn activate_focused(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut changed_rect = None;
        let mut changed = false;
        let mut dropdown_state_event = None;

        if let Some(node) = self.node_mut(id) {
            match node.kind {
                WidgetKind::Toggle { on: ref mut v, .. } => {
                    *v = !*v;
                    changed = true;
                    changed_rect = Some(node.rect);
                }
                WidgetKind::Checkbox {
                    checked: ref mut v, ..
                } => {
                    *v = !*v;
                    changed = true;
                    changed_rect = Some(node.rect);
                }
                WidgetKind::Keyboard {
                    keys,
                    alt_keys,
                    selected,
                    layout,
                    target,
                    ..
                } => {
                    if let Some(ch) = keyboard_char_for_layout(keys, alt_keys, selected, layout) {
                        changed = true;
                        changed_rect = Some(node.rect);
                        if let Some(target) = target {
                            let _ = self.push_event(UiEvent::TextInput { id: target, ch });
                            let _ = self.push_event(UiEvent::ValueChanged(target));
                        }
                    }
                }
                WidgetKind::Dropdown {
                    open: ref mut is_open,
                    ..
                } => {
                    *is_open = !*is_open;
                    changed = true;
                    changed_rect = Some(node.rect);
                    dropdown_state_event = Some(*is_open);
                }
                _ => {}
            }
        }

        if let Some(open) = dropdown_state_event {
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(
                id,
                if open {
                    WidgetEventKind::Opened
                } else {
                    WidgetEventKind::Closed
                },
                &mut events,
                |_| EventPolicy::Continue,
            )?;
            self.push_event(if open {
                UiEvent::Opened(id)
            } else {
                UiEvent::Closed(id)
            })?;
        }

        if let Some(rect) = changed_rect {
            self.dirty.add(rect)?;
        }
        if changed {
            self.push_event(UiEvent::ValueChanged(id))?;
        }
        Ok(())
    }

    fn node_in_active_group(&self, node: &WidgetNode<'_>) -> bool {
        self.active_focus_group
            .is_none_or(|group| node.focus_group == group)
    }

    fn handle_pointer_pressed(&mut self, x: i32, y: i32) -> Result<(), GuiError> {
        let hit = self.pointer_hit(x, y, true);

        if let Some(id) = hit {
            self.dispatch_activation(id, true)?;
            self.pressed = Some(PressTracker {
                id,
                start_x: x,
                start_y: y,
                last_x: x,
                last_y: y,
                elapsed_ms: 0,
                long_emitted: false,
                gesture_emitted: false,
                repeat_elapsed_ms: 0,
                scroll_velocity: 0.0,
            });
            self.inertia_scroll = None;
        }
        Ok(())
    }

    fn handle_pointer_released(&mut self, _x: i32, _y: i32) -> Result<(), GuiError> {
        let mut released_id = None;
        if let Some(pressed) = self.pressed {
            if let Some(scroll_id) = self.scrollable_ancestor(pressed.id) {
                if pressed.scroll_velocity.abs() > self.scroll_physics.velocity_threshold {
                    self.inertia_scroll = Some(InertiaScroll {
                        id: scroll_id,
                        velocity: pressed.scroll_velocity,
                    });
                }
            }
            released_id = Some(pressed.id);
        }
        self.pressed = None;
        if let Some(id) = released_id {
            let to = if !self.effective_enabled(id) {
                VisualState::Disabled
            } else if Some(id) == self.focus {
                VisualState::Focused
            } else {
                VisualState::Normal
            };
            self.start_state_transition(id, VisualState::Pressed, to);
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(id, WidgetEventKind::Released, &mut events, |_| {
                EventPolicy::Continue
            })?;
            self.push_event(UiEvent::Released(id))?;
            self.push_event(UiEvent::PointerReleased(id))?;
            let double_pointer = self.last_pointer_id == Some(id)
                && self.pointer_elapsed_ms <= self.pointer_double_window_ms;
            if double_pointer {
                self.dispatch_double_clicked(id)?;
                self.last_pointer_id = None;
                self.pointer_elapsed_ms = 0;
            } else {
                self.last_pointer_id = Some(id);
                self.pointer_elapsed_ms = 0;
            }
        }
        Ok(())
    }

    fn handle_pointer_moved(&mut self, x: i32, y: i32) -> Result<(), GuiError> {
        let Some(mut pressed) = self.pressed else {
            return Ok(());
        };
        let dy = y - pressed.last_y;
        pressed.last_x = x;
        pressed.last_y = y;

        let moved_from_start =
            (x - pressed.start_x).unsigned_abs() + (y - pressed.start_y).unsigned_abs();
        if !pressed.gesture_emitted && moved_from_start >= 6 {
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(pressed.id, WidgetEventKind::Gesture, &mut events, |_| {
                EventPolicy::Continue
            })?;
            self.push_event(UiEvent::Gesture(pressed.id))?;
            pressed.gesture_emitted = true;
        }

        if let Some(scroll_id) = self.scrollable_ancestor(pressed.id) {
            let current = self.scroll_offset(scroll_id).unwrap_or(0);
            let next = current.saturating_sub(dy);
            if next != current {
                self.set_scroll_offset(scroll_id, next)?;
                self.push_event(UiEvent::Scroll {
                    id: scroll_id,
                    delta: next - current,
                })?;
                let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
                self.dispatch_widget_event(
                    scroll_id,
                    WidgetEventKind::Scroll {
                        delta: next - current,
                    },
                    &mut events,
                    |_| EventPolicy::Continue,
                )?;
            }
            let blend = self.scroll_physics.drag_velocity_blend;
            pressed.scroll_velocity = pressed.scroll_velocity * (1.0 - blend) + (dy as f32) * blend;
        }
        self.pressed = Some(pressed);
        Ok(())
    }

    fn dispatch_activation(&mut self, id: WidgetId, is_pointer: bool) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::Pressed, &mut events, |_| {
            EventPolicy::Continue
        })?;
        if self.effective_focusable(id) {
            self.set_focus(Some(id))?;
        }
        self.push_event(UiEvent::Pressed(id))?;
        if is_pointer {
            self.push_event(UiEvent::PointerPressed(id))?;
        }
        let from = if Some(id) == self.focus {
            VisualState::Focused
        } else {
            VisualState::Normal
        };
        self.start_state_transition(id, from, VisualState::Pressed);

        self.activate_focused(id)?;
        self.dispatch_widget_event(id, WidgetEventKind::Clicked, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Clicked(id))?;
        self.push_event(UiEvent::Activate(id))?;
        Ok(())
    }

    fn dispatch_repeat_activation(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::Clicked, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Clicked(id))?;
        self.push_event(UiEvent::Activate(id))
    }

    fn dispatch_double_clicked(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::DoubleClicked, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::DoubleClicked(id))
    }

    fn dispatch_key_pressed(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::Pressed, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Pressed(id))
    }

    fn dispatch_key_released(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::Released, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Released(id))
    }

    fn repeatable_widget(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| {
            matches!(
                node.kind,
                WidgetKind::Button(_) | WidgetKind::IconButton { .. }
            )
        })
    }

    fn pointer_hit(&self, x: i32, y: i32, clickable_only: bool) -> Option<WidgetId> {
        self.widgets
            .iter()
            .rev()
            .find(|node| {
                (!clickable_only || node.clickable())
                    && self.effective_visible(node.id)
                    && self.effective_enabled(node.id)
                    && self
                        .absolute_rect(node.id)
                        .is_some_and(|rect| rect.contains(x, y))
            })
            .map(|node| node.id)
    }

    fn scrollable_ancestor(&self, id: WidgetId) -> Option<WidgetId> {
        let mut current = Some(id);
        let mut depth = 0usize;
        while let Some(widget_id) = current {
            if depth >= NODES {
                return None;
            }
            let node = self.node(widget_id)?;
            if node.scrollable() {
                return Some(widget_id);
            }
            current = node.parent;
            depth += 1;
        }
        None
    }

    fn mark_focus_pair(
        &mut self,
        old: Option<WidgetId>,
        new: Option<WidgetId>,
    ) -> Result<(), GuiError> {
        if let Some(id) = old {
            if let Some(rect) = self.absolute_rect(id) {
                self.dirty.add(rect)?;
            }
        }
        if let Some(id) = new {
            if let Some(rect) = self.absolute_rect(id) {
                self.dirty.add(rect)?;
            }
        }
        Ok(())
    }

    fn start_focus_transitions(&mut self, old: Option<WidgetId>, new: Option<WidgetId>) {
        if self.state_transition_ms == 0 {
            return;
        }
        if let Some(id) = old {
            self.start_state_transition(id, VisualState::Focused, VisualState::Normal);
        }
        if let Some(id) = new {
            self.start_state_transition(id, VisualState::Normal, VisualState::Focused);
        }
    }

    fn start_state_transition(&mut self, id: WidgetId, from: VisualState, to: VisualState) {
        if self.state_transition_ms == 0 || from == to {
            return;
        }
        if let Some(entry) = self
            .state_transitions
            .iter_mut()
            .find(|entry| entry.id == id)
        {
            *entry = StateTransition {
                id,
                from,
                to,
                elapsed_ms: 0,
            };
            return;
        }
        if self.state_transitions.len() == self.state_transitions.capacity() {
            self.state_transitions.remove(0);
        }
        let _ = self.state_transitions.push(StateTransition {
            id,
            from,
            to,
            elapsed_ms: 0,
        });
    }

    fn tick_state_transitions(&mut self, dt_ms: u32) -> Result<(), GuiError> {
        if self.state_transitions.is_empty() || self.state_transition_ms == 0 {
            return Ok(());
        }
        let mut i = 0usize;
        let mut completed_pressed = heapless::Vec::<WidgetId, NODES>::new();
        while i < self.state_transitions.len() {
            let mut remove = false;
            let id;
            let to;
            {
                let entry = &mut self.state_transitions[i];
                entry.elapsed_ms = entry.elapsed_ms.saturating_add(dt_ms);
                if entry.elapsed_ms >= self.state_transition_ms {
                    remove = true;
                }
                id = entry.id;
                to = entry.to;
            }
            if let Some(rect) = self.absolute_rect(id) {
                self.dirty.add(rect)?;
            }
            if remove {
                if to == VisualState::Pressed {
                    let _ = completed_pressed.push(id);
                }
                self.state_transitions.remove(i);
            } else {
                i += 1;
            }
        }
        for id in completed_pressed {
            // Pointer-held presses keep visual pressed state until release.
            if self.pressed.is_some_and(|pressed| pressed.id == id) {
                continue;
            }
            let to = self.resting_visual_state(id);
            self.start_state_transition(id, VisualState::Pressed, to);
        }
        Ok(())
    }

    fn state_transition_progress(&self, id: WidgetId) -> Option<(VisualState, VisualState, f32)> {
        let duration = self.state_transition_ms.max(1);
        self.state_transitions
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| {
                let t = (entry.elapsed_ms as f32 / duration as f32).clamp(0.0, 1.0);
                (entry.from, entry.to, t)
            })
    }

    fn set_textarea_cursor_visible(&mut self, id: Option<WidgetId>, visible: bool) {
        let Some(id) = id else {
            return;
        };
        let Some(rect) = self.absolute_rect(id) else {
            return;
        };
        let Some(node) = self.node_mut(id) else {
            return;
        };
        if let WidgetKind::TextArea {
            cursor_visible: ref mut current,
            ..
        } = node.kind
        {
            *current = visible;
            let _ = self.dirty.add(rect);
        }
    }

    fn tick_textarea_cursor_blink(&mut self, dt_ms: u32) -> Result<(), GuiError> {
        let Some(id) = self.focus else {
            return Ok(());
        };
        let is_textarea = matches!(
            self.node(id).map(|n| n.kind),
            Some(WidgetKind::TextArea { .. })
        );
        if !is_textarea {
            return Ok(());
        }
        self.textarea_cursor_blink_elapsed_ms =
            self.textarea_cursor_blink_elapsed_ms.saturating_add(dt_ms);
        if self.textarea_cursor_blink_elapsed_ms < self.textarea_cursor_blink_ms {
            return Ok(());
        }
        self.textarea_cursor_blink_elapsed_ms = 0;
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        if let WidgetKind::TextArea {
            cursor_visible: ref mut visible,
            ..
        } = node.kind
        {
            *visible = !*visible;
            self.dirty.add(rect)?;
        }
        Ok(())
    }

    fn push_event(&mut self, event: UiEvent) -> Result<(), GuiError> {
        if self.should_emit_event(event)? {
            self.events.push(event).map_err(|_| GuiError::EventsFull)?;
        }
        Ok(())
    }

    fn should_emit_event(&self, event: UiEvent) -> Result<bool, GuiError> {
        let Some(target) = event.target() else {
            return Ok(true);
        };
        let filter = self.event_filter(target)?;
        Ok(filter.contains(event.filter()))
    }

    fn stop_due_to_builtin_widget_behavior(&self, event: WidgetEvent) -> bool {
        if event.phase != EventPhase::Capture || event.current == event.target {
            return false;
        }
        let is_pointer_kind = matches!(
            event.kind,
            WidgetEventKind::Pressed | WidgetEventKind::Released | WidgetEventKind::Clicked
        );
        is_pointer_kind
            && self
                .node(event.current)
                .is_some_and(|node| matches!(node.kind, WidgetKind::ScrollView { .. }))
    }

    fn stop_due_to_registered_policy(&self, event: WidgetEvent) -> bool {
        self.dispatch_policies
            .iter()
            .find(|(id, _)| *id == event.current)
            .is_some_and(|(_, policy)| policy.stop && policy.allows(event.kind, event.phase))
    }

    fn resting_visual_state(&self, id: WidgetId) -> VisualState {
        if !self.effective_enabled(id) {
            VisualState::Disabled
        } else if Some(id) == self.focus {
            VisualState::Focused
        } else {
            VisualState::Normal
        }
    }

    fn current_visual_state(&self, id: WidgetId) -> VisualState {
        if self.pressed.is_some_and(|pressed| pressed.id == id) {
            VisualState::Pressed
        } else {
            self.resting_visual_state(id)
        }
    }

    fn press_timing_for(&self, id: WidgetId) -> PressTiming {
        self.widget_press_timings
            .iter()
            .find(|(timing_id, _)| *timing_id == id)
            .map(|(_, timing)| *timing)
            .unwrap_or(PressTiming {
                long_press_ms: self.long_press_ms,
                repeat_delay_ms: self.press_repeat_delay_ms,
                repeat_interval_ms: self.press_repeat_interval_ms,
            })
    }

    fn key_input_policy_for(&self, id: WidgetId) -> WidgetKeyInputPolicy {
        self.widget_key_policies
            .iter()
            .find(|(policy_id, _)| *policy_id == id)
            .map(|(_, policy)| *policy)
            .unwrap_or_default()
    }

    fn key_bindings_for(&self, id: WidgetId) -> WidgetKeyBindings {
        self.widget_key_bindings
            .iter()
            .find(|(binding_id, _)| *binding_id == id)
            .map(|(_, bindings)| *bindings)
            .unwrap_or_default()
    }

    fn handle_select_activation(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let double_select = self.last_select_id == Some(id)
            && self.select_elapsed_ms <= self.select_double_window_ms;
        self.dispatch_activation(id, false)?;
        if double_select {
            self.dispatch_double_clicked(id)?;
            self.last_select_id = None;
            self.select_elapsed_ms = 0;
        } else {
            self.last_select_id = Some(id);
            self.select_elapsed_ms = 0;
        }
        Ok(())
    }

    fn handle_back_action(&mut self) -> Result<(), GuiError> {
        if let Some(id) = self.focus {
            if matches!(
                self.node(id).map(|n| n.kind),
                Some(WidgetKind::TextArea { .. })
            ) {
                self.textarea_backspace(id)?;
                return Ok(());
            }
            if matches!(
                self.node(id).map(|n| n.kind),
                Some(WidgetKind::Dropdown { open: true, .. })
            ) {
                self.set_dropdown_open(id, false)?;
                return Ok(());
            }
        }
        self.push_event(UiEvent::Back)
    }
}

fn bump_index(current: &mut usize, len: usize, delta: i8) -> bool {
    if len == 0 {
        return false;
    }
    let next = if delta >= 0 {
        (*current + 1) % len
    } else if *current == 0 {
        len - 1
    } else {
        *current - 1
    };
    if next != *current {
        *current = next;
        true
    } else {
        false
    }
}

fn keyboard_char_for_layout(
    keys: &[char],
    alt_keys: Option<&[char]>,
    selected: usize,
    layout: KeyboardLayout,
) -> Option<char> {
    let base = keys.get(selected).copied()?;
    Some(match layout {
        KeyboardLayout::Normal => base,
        KeyboardLayout::Shift => {
            if base.is_ascii_alphabetic() {
                base.to_ascii_uppercase()
            } else {
                base
            }
        }
        KeyboardLayout::Symbols => alt_keys
            .and_then(|keys| keys.get(selected).copied())
            .unwrap_or('#'),
    })
}

fn textarea_text(buf: &[u8; TEXTAREA_CAPACITY], len: u8) -> &str {
    let used = (len as usize).min(TEXTAREA_CAPACITY);
    core::str::from_utf8(&buf[..used]).unwrap_or("")
}

fn textarea_storage_from_str(text: &str) -> ([u8; TEXTAREA_CAPACITY], u8) {
    let mut out = [0u8; TEXTAREA_CAPACITY];
    let mut len = 0usize;
    for ch in text.chars() {
        let mut tmp = [0u8; 4];
        let enc = ch.encode_utf8(&mut tmp).as_bytes();
        if len + enc.len() > TEXTAREA_CAPACITY {
            break;
        }
        out[len..len + enc.len()].copy_from_slice(enc);
        len += enc.len();
    }
    (out, len as u8)
}

fn textarea_storage_from_chars(
    chars: &heapless::Vec<char, TEXTAREA_CAPACITY>,
) -> ([u8; TEXTAREA_CAPACITY], u8) {
    let mut out = [0u8; TEXTAREA_CAPACITY];
    let mut len = 0usize;
    for ch in chars {
        let mut tmp = [0u8; 4];
        let enc = ch.encode_utf8(&mut tmp).as_bytes();
        if len + enc.len() > TEXTAREA_CAPACITY {
            break;
        }
        out[len..len + enc.len()].copy_from_slice(enc);
        len += enc.len();
    }
    (out, len as u8)
}

fn char_at(text: &str, idx: usize) -> Option<char> {
    text.chars().nth(idx)
}

fn prev_word_boundary(text: &str, cursor: usize) -> usize {
    let mut pos = cursor.min(text.chars().count());
    while pos > 0 && char_at(text, pos - 1).is_some_and(|ch| ch.is_whitespace()) {
        pos -= 1;
    }
    while pos > 0 && char_at(text, pos - 1).is_some_and(|ch| !ch.is_whitespace()) {
        pos -= 1;
    }
    pos
}

fn next_word_boundary(text: &str, cursor: usize) -> usize {
    let len = text.chars().count();
    let mut pos = cursor.min(len);
    while pos < len && char_at(text, pos).is_some_and(|ch| !ch.is_whitespace()) {
        pos += 1;
    }
    while pos < len && char_at(text, pos).is_some_and(|ch| ch.is_whitespace()) {
        pos += 1;
    }
    pos
}

fn delete_selection_if_any(
    chars: &mut heapless::Vec<char, TEXTAREA_CAPACITY>,
    cursor: &mut usize,
    selection: &mut Option<(usize, usize)>,
) -> bool {
    let Some((start, end)) = *selection else {
        return false;
    };
    let start = start.min(end).min(chars.len());
    let end = end.max(start).min(chars.len());
    if end <= start {
        *selection = None;
        *cursor = start;
        return false;
    }
    for _ in start..end {
        chars.remove(start);
    }
    *cursor = start;
    *selection = None;
    true
}

fn textarea_row_col_at_cursor(text: &str, cursor: usize, wrap_cols: usize) -> (usize, usize) {
    let mut row = 0usize;
    let mut col = 0usize;
    for ch in text.chars().take(cursor) {
        if ch == '\n' {
            row += 1;
            col = 0;
            continue;
        }
        col += 1;
        if col >= wrap_cols {
            row += 1;
            col = 0;
        }
    }
    (row, col)
}

fn textarea_cursor_from_row_col(
    text: &str,
    target_row: usize,
    target_col: usize,
    wrap_cols: usize,
) -> usize {
    let mut row = 0usize;
    let mut col = 0usize;
    let mut idx = 0usize;
    for ch in text.chars() {
        if row == target_row && col >= target_col {
            break;
        }
        if ch == '\n' {
            if row == target_row {
                break;
            }
            row += 1;
            col = 0;
            idx += 1;
            continue;
        }
        idx += 1;
        col += 1;
        if col >= wrap_cols {
            if row == target_row {
                break;
            }
            row += 1;
            col = 0;
        }
    }
    idx
}

fn textarea_row_end_col(text: &str, target_row: usize, wrap_cols: usize) -> usize {
    let mut row = 0usize;
    let mut col = 0usize;
    for ch in text.chars() {
        if row == target_row {
            if ch == '\n' {
                break;
            }
            col += 1;
            if col >= wrap_cols {
                break;
            }
        } else if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
            if col >= wrap_cols {
                row += 1;
                col = 0;
            }
        }
    }
    col
}
