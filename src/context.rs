use embedded_graphics_core::pixelcolor::Rgb565;
use heapless::Vec;

use crate::{
    geometry::{DirtyError, DirtyTracker, Rect},
    input::{
        InputEvent, PointerState, UiEvent, UiEventFilter, WidgetDispatchPolicy, WidgetEvent,
        WidgetEventKind,
    },
    layout::{Axis, LayoutItem, LinearLayout},
    present::PresentRegion,
    render::{RenderCtx, RenderQuality},
    state::{ListState, ScrollState, SliderState, TabsState},
    style::{Style, Theme, VisualState, WidgetStyle},
    widget::{
        EventContext, EventPhase, EventPolicy, FocusGroupId, StyleClassId, WidgetFlags, WidgetId,
    },
    widgets::{WidgetKind, WidgetNode},
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
        self.dirty.mark_all(self.viewport)?;
        Ok(())
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
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::ScrollView {
                offset_y: ref mut v,
                content_h,
            } => {
                let mut state = ScrollState::new(*v, content_h);
                state.set_offset(offset_y);
                *v = state.offset_y;
                self.dirty.add(rect)?;
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

    pub fn set_widget_rect(&mut self, id: WidgetId, rect: Rect) -> Result<(), GuiError> {
        let old = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.rect = rect;
        self.dirty.add(old)?;
        self.mark_subtree_dirty(id)?;
        Ok(())
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
        self.mark_subtree_dirty(id)?;
        self.node_mut(id)
            .ok_or(GuiError::NotFound)?
            .flags
            .set(flag, enabled);
        self.mark_subtree_dirty(id)?;
        if self
            .focus
            .is_some_and(|focus| !self.effective_focusable(focus))
        {
            self.focus = None;
            self.ensure_focus();
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
        self.render_into(&mut ctx)
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
            self.render_into(&mut ctx)?;
        }
        Ok(())
    }

    pub fn handle_input(&mut self, event: InputEvent) -> Result<(), GuiError> {
        match event {
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
                    self.dispatch_activation(id, false)?;
                }
                Ok(())
            }
            InputEvent::Back => self.push_event(UiEvent::Back),
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
            _ => Ok(()),
        }
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

    fn render_into<D>(&self, ctx: &mut RenderCtx<'_, D>) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        for node in &self.widgets {
            if !self.effective_visible(node.id) {
                continue;
            }
            let Some(rect) = self.absolute_rect(node.id) else {
                continue;
            };
            let clip = self.inherited_clip(node.id).unwrap_or(self.viewport);
            if rect.intersection(clip).is_empty() {
                continue;
            }
            let old_clip = ctx.clip();
            ctx.set_clip(old_clip.intersection(clip));
            let state = if Some(node.id) == self.focus {
                VisualState::Focused
            } else if !self.effective_enabled(node.id) {
                VisualState::Disabled
            } else {
                VisualState::Normal
            };
            let mut render_node = *node;
            if let Some(class) = node.style_class {
                if let Some((_, style)) = self.class_styles.iter().find(|(id, _)| *id == class) {
                    let class_state = style.resolve(state);
                    render_node.style = node.style.with_state_override(state, class_state);
                }
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
            clip = clip.intersection(self.absolute_rect(widget_id)?);
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
                _ => {}
            }
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
        let hit = self
            .widgets
            .iter()
            .rev()
            .find(|node| {
                node.clickable()
                    && self.effective_visible(node.id)
                    && self.effective_enabled(node.id)
                    && self
                        .absolute_rect(node.id)
                        .is_some_and(|rect| rect.contains(x, y))
            })
            .map(|node| node.id);

        if let Some(id) = hit {
            self.dispatch_activation(id, true)?;
        }
        Ok(())
    }

    fn handle_pointer_released(&mut self, x: i32, y: i32) -> Result<(), GuiError> {
        let hit = self
            .widgets
            .iter()
            .rev()
            .find(|node| {
                node.clickable()
                    && self.effective_visible(node.id)
                    && self.effective_enabled(node.id)
                    && self
                        .absolute_rect(node.id)
                        .is_some_and(|rect| rect.contains(x, y))
            })
            .map(|node| node.id);
        if let Some(id) = hit {
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(id, WidgetEventKind::Released, &mut events, |_| {
                EventPolicy::Continue
            })?;
            self.push_event(UiEvent::Released(id))?;
            self.push_event(UiEvent::PointerReleased(id))?;
        }
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

        self.activate_focused(id)?;
        self.dispatch_widget_event(id, WidgetEventKind::Clicked, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Clicked(id))?;
        self.push_event(UiEvent::Activate(id))?;
        Ok(())
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
