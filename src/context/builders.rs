use crate::{
    geometry::Rect,
    haptics::HapticPattern,
    image::{ImageFit, ImageRef, ReelPlayer},
    render::TextAlign,
    style::{Style, WidgetStyle},
    widget::{FocusGroupId, StyleClassId, Widget, WidgetId},
    widgets::{ChartMode, KeyboardLayout, NotificationLevel, SurfaceState, WidgetKind},
};
use embedded_graphics_core::pixelcolor::Rgb565;

use super::*;

pub struct WidgetBuilder<'a, 'ctx, W, const NODES: usize, const EVENTS: usize, const DIRTY: usize> {
    ctx: &'ctx mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    rect: Rect,
    _widget: W,
    parent: Option<WidgetId>,
    style_class: Option<StyleClassId>,
    focus_group: FocusGroupId,
    style: Option<WidgetStyle>,
}

impl<'a, 'ctx, W, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    WidgetBuilder<'a, 'ctx, W, NODES, EVENTS, DIRTY>
where
    W: Widget + 'a,
{
    pub fn new(
        ctx: &'ctx mut GuiContext<'a, NODES, EVENTS, DIRTY>,
        rect: impl Into<Rect>,
        widget: W,
    ) -> Self {
        Self {
            ctx,
            rect: rect.into(),
            _widget: widget,
            parent: None,
            style_class: None,
            focus_group: FocusGroupId::ROOT,
            style: None,
        }
    }

    pub fn with_parent(mut self, parent: WidgetId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_style_class(mut self, style_class: StyleClassId) -> Self {
        self.style_class = Some(style_class);
        self
    }

    pub fn with_focus_group(mut self, focus_group: FocusGroupId) -> Self {
        self.focus_group = focus_group;
        self
    }

    pub fn with_style(mut self, style: impl Into<WidgetStyle>) -> Self {
        self.style = Some(style.into());
        self
    }

    pub fn build(self) -> Result<WidgetId, GuiError> {
        let style = self
            .style
            .unwrap_or_else(|| WidgetStyle::from(Style::default()));
        let id = self.ctx.add_widget(self.rect, WidgetKind::Spacer, style)?;
        if let Some(parent) = self.parent {
            self.ctx.add_child(parent, id)?;
        }
        if let Some(class_id) = self.style_class {
            if let Some(node) = self.ctx.node_mut(id) {
                node.style_class = Some(class_id);
            }
        }
        if self.focus_group != FocusGroupId::ROOT {
            self.ctx.set_focus_group(id, self.focus_group)?;
        }
        Ok(id)
    }
}

impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    GuiContext<'a, NODES, EVENTS, DIRTY>
{
    pub fn spawn<'ctx, W>(
        &'ctx mut self,
        rect: impl Into<Rect>,
        widget: W,
    ) -> WidgetBuilder<'a, 'ctx, W, NODES, EVENTS, DIRTY>
    where
        W: Widget + 'a,
    {
        WidgetBuilder::new(self, rect, widget)
    }
    pub fn add_panel<S>(&mut self, rect: impl Into<Rect>, style: S) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Panel, style)
    }

    pub fn add_themed_panel(&mut self, rect: impl Into<Rect>) -> Result<WidgetId, GuiError> {
        self.add_panel(rect, self.theme.panel)
    }

    pub fn add_label<S>(
        &mut self,
        rect: impl Into<Rect>,
        text: &'a str,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Label(text), style)
    }

    pub fn add_themed_label(
        &mut self,
        rect: impl Into<Rect>,
        text: &'a str,
    ) -> Result<WidgetId, GuiError> {
        self.add_label(rect, text, self.theme.label)
    }

    pub fn add_button<S>(
        &mut self,
        rect: impl Into<Rect>,
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

    pub fn add_themed_button(
        &mut self,
        rect: impl Into<Rect>,
        text: &'a str,
    ) -> Result<WidgetId, GuiError> {
        self.add_button(rect, text, self.theme.button)
    }

    #[cfg(feature = "rich-widgets")]
    pub fn add_progress_bar<S>(
        &mut self,
        rect: impl Into<Rect>,
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_progress_bar(
        &mut self,
        rect: impl Into<Rect>,
        value: f32,
    ) -> Result<WidgetId, GuiError> {
        self.add_progress_bar(rect, value, self.theme.progress)
    }

    #[cfg(feature = "rich-widgets")]
    pub fn add_toggle<S>(
        &mut self,
        rect: impl Into<Rect>,
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_toggle(
        &mut self,
        rect: impl Into<Rect>,
        label: &'a str,
        on: bool,
    ) -> Result<WidgetId, GuiError> {
        self.add_toggle(rect, label, on, self.theme.toggle)
    }

    #[cfg(feature = "rich-widgets")]
    pub fn add_checkbox<S>(
        &mut self,
        rect: impl Into<Rect>,
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_checkbox(
        &mut self,
        rect: impl Into<Rect>,
        label: &'a str,
        checked: bool,
    ) -> Result<WidgetId, GuiError> {
        self.add_checkbox(rect, label, checked, self.theme.checkbox)
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_slider(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
    ) -> Result<WidgetId, GuiError> {
        self.add_slider(rect, value, min, max, self.theme.slider)
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_value_label(
        &mut self,
        rect: Rect,
        label: &'a str,
        value: i32,
    ) -> Result<WidgetId, GuiError> {
        self.add_value_label(rect, label, value, self.theme.value_label)
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_icon_button(
        &mut self,
        rect: Rect,
        icon: char,
        label: &'a str,
    ) -> Result<WidgetId, GuiError> {
        self.add_icon_button(rect, icon, label, self.theme.icon_button)
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_feed_timeline<S>(
        &mut self,
        rect: Rect,
        items: &'a [&'a str],
        selected: usize,
        visible_rows: usize,
        expanded: bool,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let selected = selected.min(items.len().saturating_sub(1));
        let id = self.add_widget(
            rect,
            WidgetKind::FeedTimeline {
                items,
                selected,
                offset: selected,
                visible_rows: visible_rows.max(1),
                expanded,
            },
            style,
        )?;
        self.ensure_focus();
        Ok(id)
    }

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_list(
        &mut self,
        rect: Rect,
        items: &'a [&'a str],
        selected: usize,
        visible_rows: usize,
    ) -> Result<WidgetId, GuiError> {
        self.add_list(rect, items, selected, visible_rows, self.theme.list)
    }

    pub fn add_circular_list<S>(
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
            WidgetKind::CircularList {
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

    pub fn add_themed_circular_list(
        &mut self,
        rect: Rect,
        items: &'a [&'a str],
        selected: usize,
        visible_rows: usize,
    ) -> Result<WidgetId, GuiError> {
        self.add_circular_list(rect, items, selected, visible_rows, self.theme.list)
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_scroll_view(
        &mut self,
        rect: Rect,
        offset_y: i32,
        content_h: u32,
    ) -> Result<WidgetId, GuiError> {
        self.add_scroll_view(rect, offset_y, content_h, self.theme.list)
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_tabs(
        &mut self,
        rect: Rect,
        labels: &'a [&'a str],
        selected: usize,
    ) -> Result<WidgetId, GuiError> {
        self.add_tabs(rect, labels, selected, self.theme.tabs)
    }

    #[cfg(feature = "rich-widgets")]
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
        let id = self.add_widget(rect, WidgetKind::Dialog { title, body }, style)?;
        self.play_haptic(HapticPattern::Alert);
        Ok(id)
    }

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_dialog(
        &mut self,
        rect: Rect,
        title: &'a str,
        body: &'a str,
    ) -> Result<WidgetId, GuiError> {
        self.add_dialog(rect, title, body, self.theme.dialog)
    }

    #[cfg(feature = "rich-widgets")]
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
        let id = self.add_widget(rect, WidgetKind::Toast { text, ttl_ms }, style)?;
        self.play_haptic(HapticPattern::Success);
        Ok(id)
    }

    #[cfg(feature = "rich-widgets")]
    pub fn add_themed_toast(
        &mut self,
        rect: Rect,
        text: &'a str,
        ttl_ms: u32,
    ) -> Result<WidgetId, GuiError> {
        self.add_toast(rect, text, ttl_ms, self.theme.toast)
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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
    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[allow(clippy::too_many_arguments)]
    pub fn add_sweeping_arc<S>(
        &mut self,
        rect: Rect,
        progress: f32,
        clockwise: bool,
        arc_radius: u32,
        frame_inset: u16,
        corner_radius: u8,
        bg_color: Rgb565,
        arc_color: Rgb565,
        frame_color: Rgb565,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::SweepingArc {
                progress: progress.clamp(0.0, 1.0),
                clockwise,
                arc_radius,
                frame_inset,
                corner_radius,
                bg_color,
                arc_color,
                frame_color,
            },
            style,
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    pub fn add_plotter<S>(
        &mut self,
        rect: Rect,
        values: &'a [f32],
        head: usize,
        min: f32,
        max: f32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::Plotter {
                values,
                head,
                min,
                max,
                thickness: 1,
                show_grid: false,
                show_axes: false,
            },
            style,
        )
    }

    pub fn add_themed_plotter(
        &mut self,
        rect: Rect,
        values: &'a [f32],
        head: usize,
        min: f32,
        max: f32,
    ) -> Result<WidgetId, GuiError> {
        self.add_plotter(rect, values, head, min, max, self.theme.panel)
    }

    #[cfg(feature = "rich-widgets")]
    pub fn set_plotter_style(&mut self, id: WidgetId, thickness: u8) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Plotter {
                thickness: ref mut t,
                ..
            } => {
                *t = thickness.max(1);
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    #[cfg(feature = "rich-widgets")]
    pub fn set_plotter_decoration(
        &mut self,
        id: WidgetId,
        show_grid: bool,
        show_axes: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Plotter {
                show_grid: ref mut grid,
                show_axes: ref mut axes,
                ..
            } => {
                *grid = show_grid;
                *axes = show_axes;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_spinner<S>(&mut self, rect: Rect, phase: f32, style: S) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(rect, WidgetKind::Spinner { phase }, style)
    }

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
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

    #[cfg(feature = "rich-widgets")]
    pub fn add_state_surface<S>(
        &mut self,
        rect: Rect,
        state: SurfaceState,
        title: &'a str,
        message: &'a str,
        action: Option<&'a str>,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::StateSurface {
                state,
                title,
                message,
                action,
                busy_phase: 0.0,
            },
            style,
        )
    }

    #[cfg(feature = "rich-widgets")]
    pub fn add_heads_up_banner<S>(
        &mut self,
        rect: Rect,
        level: NotificationLevel,
        text: &'a str,
        ttl_ms: u32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::HeadsUpBanner {
                level,
                text,
                ttl_ms,
            },
            style,
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "rich-widgets")]
    pub fn add_notification_action_sheet<S>(
        &mut self,
        rect: Rect,
        level: NotificationLevel,
        title: &'a str,
        body: &'a str,
        actions: &'a [&'a str],
        selected: usize,
        open: bool,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::NotificationActionSheet {
                level,
                title,
                body,
                actions,
                selected: selected.min(actions.len().saturating_sub(1)),
                open,
            },
            style,
        )
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

    #[cfg(feature = "rich-widgets")]
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

    pub fn add_dial<S>(
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
        let value = value.clamp(min, max);
        let id = self.add_widget(rect, WidgetKind::Dial { value, min, max }, style)?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_dial(
        &mut self,
        rect: Rect,
        value: f32,
        min: f32,
        max: f32,
    ) -> Result<WidgetId, GuiError> {
        self.add_dial(rect, value, min, max, self.theme.button)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_rle_player<S>(
        &mut self,
        rect: impl Into<Rect>,
        rle_data: &'static [u8],
        frame_width: u16,
        frame_height: u16,
        total_frames: usize,
        frame_duration_ms: u32,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        self.add_widget(
            rect,
            WidgetKind::RlePlayer {
                rle_data,
                frame_width,
                frame_height,
                total_frames,
                current_frame: 0,
                elapsed_ms: 0,
                frame_duration_ms,
            },
            style,
        )
    }

    pub fn add_themed_rle_player(
        &mut self,
        rect: Rect,
        rle_data: &'static [u8],
        frame_width: u16,
        frame_height: u16,
        total_frames: usize,
        frame_duration_ms: u32,
    ) -> Result<WidgetId, GuiError> {
        self.add_rle_player(
            rect,
            rle_data,
            frame_width,
            frame_height,
            total_frames,
            frame_duration_ms,
            self.theme.panel,
        )
    }

    pub fn add_autocomplete_widget<S>(
        &mut self,
        rect: Rect,
        suggestions: &'a [&'a str],
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = self.add_widget(
            rect,
            WidgetKind::AutoComplete {
                text_buf: [0; 32],
                text_len: 0,
                suggestions,
                filtered: [None; 8],
                filter_count: 0,
                selected: None,
                expanded: false,
            },
            style,
        )?;
        self.ensure_focus();
        Ok(id)
    }

    pub fn add_themed_autocomplete(
        &mut self,
        rect: Rect,
        suggestions: &'a [&'a str],
    ) -> Result<WidgetId, GuiError> {
        self.add_autocomplete_widget(rect, suggestions, self.theme.panel)
    }
}
