use embedded_graphics_core::pixelcolor::Rgb565;
use heapless::Vec;

use crate::{
    geometry::{DirtyError, DirtyTracker, Rect},
    image::{ImageFit, ImageRef},
    input::{
        InputEvent, PointerState, UiEvent, UiEventFilter, WidgetDispatchPolicy, WidgetEvent,
        WidgetEventKind,
    },
    layout::{Axis, LayoutItem, LinearLayout},
    present::PresentRegion,
    render::{RenderCtx, RenderQuality, TextAlign},
    state::{ListState, ScrollState, SliderState, TabsState},
    style::{Style, Theme, VisualState, WidgetStyle, lerp_style},
    widget::{
        EventContext, EventPhase, EventPolicy, FocusGroupId, StyleClassId, WidgetFlags, WidgetId,
    },
    widgets::{ChartMode, KeyboardLayout, WidgetKind, WidgetNode},
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
    press_repeat_delay_ms: u32,
    press_repeat_interval_ms: u32,
    pressed: Option<PressTracker>,
    inertia_scroll: Option<InertiaScroll>,
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
            press_repeat_delay_ms: 650,
            press_repeat_interval_ms: 140,
            pressed: None,
            inertia_scroll: None,
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

    pub fn add_spinner<S>(
        &mut self,
        rect: Rect,
        phase: f32,
        style: S,
    ) -> Result<WidgetId, GuiError>
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
        let id = self.add_widget(
            rect,
            WidgetKind::TextArea {
                text,
                cursor,
                placeholder,
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

    pub fn tick_spinner(&mut self, id: WidgetId, dt_ms: u32, cycles_per_sec: f32) -> Result<(), GuiError> {
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
                text: ref mut current,
                cursor: ref mut c,
                ..
            } => {
                *current = text;
                *c = (*c).min(text.chars().count());
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn textarea_text(&self, id: WidgetId) -> Option<&'a str> {
        match self.node(id)?.kind {
            WidgetKind::TextArea { text, .. } => Some(text),
            _ => None,
        }
    }

    pub fn set_textarea_cursor(&mut self, id: WidgetId, cursor: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text,
                cursor: ref mut current,
                ..
            } => {
                *current = cursor.min(text.chars().count());
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn move_textarea_cursor(&mut self, id: WidgetId, delta: i8) -> Result<(), GuiError> {
        let next = self.textarea_cursor(id).ok_or(GuiError::NotFound)? as i32 + delta as i32;
        self.set_textarea_cursor(id, next.max(0) as usize)
    }

    pub fn textarea_cursor(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::TextArea { cursor, .. } => Some(cursor),
            _ => None,
        }
    }

    pub fn textarea_insert_char(&mut self, id: WidgetId, ch: char) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        self.push_event(UiEvent::TextInput { id, ch })?;
        self.push_event(UiEvent::ValueChanged(id))
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

    pub fn set_keyboard_layout(&mut self, id: WidgetId, layout: KeyboardLayout) -> Result<(), GuiError> {
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
            InputEvent::Back => {
                if let Some(id) = self.focus {
                    if matches!(self.node(id).map(|n| n.kind), Some(WidgetKind::TextArea { .. })) {
                        self.textarea_backspace(id)?;
                        return Ok(());
                    }
                }
                self.push_event(UiEvent::Back)
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
        if let Some(mut inertia) = self.inertia_scroll {
            if inertia.velocity.abs() < 0.05 {
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
                inertia.velocity *= 0.86f32.powf((dt_ms as f32 / 16.0).max(1.0));
                self.inertia_scroll = Some(inertia);
            }
        }
        let Some(mut pressed) = self.pressed else {
            return Ok(());
        };
        if !self.effective_visible(pressed.id) || !self.effective_enabled(pressed.id) {
            self.pressed = None;
            return Ok(());
        }
        pressed.elapsed_ms = pressed.elapsed_ms.saturating_add(dt_ms);
        pressed.repeat_elapsed_ms = pressed.repeat_elapsed_ms.saturating_add(dt_ms);
        if !pressed.long_emitted && pressed.elapsed_ms >= self.long_press_ms {
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(pressed.id, WidgetEventKind::LongPressed, &mut events, |_| {
                EventPolicy::Continue
            })?;
            self.push_event(UiEvent::LongPressed(pressed.id))?;
            pressed.long_emitted = true;
        }
        if pressed.repeat_elapsed_ms >= self.press_repeat_delay_ms
            && self.repeatable_widget(pressed.id)
            && pressed.long_emitted
        {
            let intervals = (pressed.repeat_elapsed_ms - self.press_repeat_delay_ms)
                / self.press_repeat_interval_ms;
            if intervals > 0 {
                self.dispatch_repeat_activation(pressed.id)?;
                pressed.repeat_elapsed_ms = self.press_repeat_delay_ms;
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
                text, placeholder, ..
            } => (
                text_width(if text.is_empty() { placeholder } else { text }).saturating_add(10),
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
                    text,
                    cursor: ref mut current,
                    ..
                } => {
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

    fn handle_pointer_released(&mut self, x: i32, y: i32) -> Result<(), GuiError> {
        if let Some(pressed) = self.pressed {
            if let Some(scroll_id) = self.scrollable_ancestor(pressed.id) {
                if pressed.scroll_velocity.abs() > 0.2 {
                    self.inertia_scroll = Some(InertiaScroll {
                        id: scroll_id,
                        velocity: pressed.scroll_velocity,
                    });
                }
            }
        }
        self.pressed = None;
        let hit = self.pointer_hit(x, y, true);
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
            pressed.scroll_velocity = pressed.scroll_velocity * 0.6 + (dy as f32) * 0.4;
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

    fn repeatable_widget(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| {
            matches!(node.kind, WidgetKind::Button(_) | WidgetKind::IconButton { .. })
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
