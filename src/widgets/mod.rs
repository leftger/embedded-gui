use core::fmt::Write;

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use heapless::String;

use crate::{
    block::Block,
    geometry::{EdgeInsets, Rect},
    image::{ImageFit, ImageRef},
    render::{RenderCtx, StrokeStyle, TextAlign, TextStyle, TextWrap, VerticalAlign},
    style::{Border, Style, VisualState, WidgetStyle},
    widget::{FocusGroupId, StyleClassId, WidgetFlags, WidgetId},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WidgetKind<'a> {
    Panel,
    Label(&'a str),
    Button(&'a str),
    ProgressBar {
        value: f32,
    },
    Toggle {
        label: &'a str,
        on: bool,
    },
    Checkbox {
        label: &'a str,
        checked: bool,
    },
    Slider {
        value: f32,
        min: f32,
        max: f32,
    },
    ValueLabel {
        label: &'a str,
        value: i32,
    },
    IconButton {
        icon: char,
        label: &'a str,
    },
    List {
        items: &'a [&'a str],
        selected: usize,
        offset: usize,
        visible_rows: usize,
    },
    ScrollView {
        offset_y: i32,
        content_h: u32,
    },
    Tabs {
        labels: &'a [&'a str],
        selected: usize,
    },
    Dialog {
        title: &'a str,
        body: &'a str,
    },
    Toast {
        text: &'a str,
        ttl_ms: u32,
    },
    Meter {
        value: f32,
        min: f32,
        max: f32,
    },
    ArcGauge {
        value: f32,
        min: f32,
        max: f32,
        start_deg: i32,
        end_deg: i32,
        thickness: u8,
        antialias: bool,
        major_ticks: u8,
        minor_ticks: u8,
        show_value: bool,
    },
    Gauge {
        value: f32,
        min: f32,
        max: f32,
        major_ticks: u8,
        minor_ticks: u8,
        show_value: bool,
    },
    GaugeNeedle {
        value: f32,
        min: f32,
        max: f32,
        start_deg: i32,
        end_deg: i32,
    },
    Chart {
        values: &'a [f32],
        min: f32,
        max: f32,
        thickness: u8,
        fill_under: bool,
        markers: bool,
        mode: ChartMode,
        show_grid: bool,
        show_axes: bool,
        show_labels: bool,
    },
    Spinner {
        phase: f32,
    },
    Dropdown {
        items: &'a [&'a str],
        selected: usize,
        open: bool,
    },
    Roller {
        items: &'a [&'a str],
        selected: usize,
    },
    Table {
        rows: &'a [&'a [&'a str]],
        separators: bool,
        cell_padding: u8,
        align: TextAlign,
    },
    TextArea {
        text: &'a str,
        cursor: usize,
        placeholder: &'a str,
    },
    Keyboard {
        keys: &'a [char],
        selected: usize,
        cols: u8,
        alt_keys: Option<&'a [char]>,
        layout: KeyboardLayout,
        target: Option<WidgetId>,
    },
    Image {
        image: ImageRef<'a>,
        fit: ImageFit,
    },
    Border,
    Spacer,
    Menu {
        items: &'a [&'a str],
        selected: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartMode {
    Line,
    Bars,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardLayout {
    Normal,
    Shift,
    Symbols,
}

impl WidgetKind<'_> {
    pub const fn focusable(self) -> bool {
        matches!(
            self,
            Self::Button(_)
                | Self::Toggle { .. }
                | Self::Checkbox { .. }
                | Self::Slider { .. }
                | Self::IconButton { .. }
                | Self::List { .. }
                | Self::ScrollView { .. }
                | Self::Tabs { .. }
                | Self::Dropdown { .. }
                | Self::Roller { .. }
                | Self::TextArea { .. }
                | Self::Keyboard { .. }
                | Self::Menu { .. }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetNode<'a> {
    pub id: WidgetId,
    pub parent: Option<WidgetId>,
    pub style_class: Option<StyleClassId>,
    pub focus_group: FocusGroupId,
    pub rect: Rect,
    pub style: WidgetStyle,
    pub kind: WidgetKind<'a>,
    pub flags: WidgetFlags,
}

impl<'a> WidgetNode<'a> {
    pub fn new<S>(id: WidgetId, rect: Rect, kind: WidgetKind<'a>, style: S) -> Self
    where
        S: Into<WidgetStyle>,
    {
        Self {
            id,
            parent: None,
            style_class: None,
            focus_group: FocusGroupId::ROOT,
            rect,
            style: style.into(),
            kind,
            flags: default_flags(kind),
        }
    }

    pub const fn hidden(&self) -> bool {
        self.flags.contains(WidgetFlags::HIDDEN)
    }

    pub const fn disabled(&self) -> bool {
        self.flags.contains(WidgetFlags::DISABLED)
    }

    pub const fn clickable(&self) -> bool {
        self.flags.contains(WidgetFlags::CLICKABLE)
    }

    pub const fn scrollable(&self) -> bool {
        self.flags.contains(WidgetFlags::SCROLLABLE)
    }

    pub const fn clips_children(&self) -> bool {
        self.flags.contains(WidgetFlags::CLIP_CHILDREN)
    }

    pub const fn focusable(&self) -> bool {
        !self.hidden() && !self.disabled() && self.flags.contains(WidgetFlags::FOCUSABLE)
    }

    pub fn render<D>(&self, ctx: &mut RenderCtx<'_, D>, state: VisualState) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        self.render_at(ctx, self.rect, state)
    }

    pub fn render_at<D>(
        &self,
        ctx: &mut RenderCtx<'_, D>,
        rect: Rect,
        state: VisualState,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        if self.hidden() {
            return Ok(());
        }

        match self.kind {
            WidgetKind::Panel => render_panel(ctx, rect, self.style, state),
            WidgetKind::Label(text) => render_label(ctx, rect, text, self.style),
            WidgetKind::Button(text) => render_button(ctx, rect, text, self.style, state),
            WidgetKind::ProgressBar { value } => {
                render_progress(ctx, rect, value, self.style, state)
            }
            WidgetKind::Toggle { label, on } => {
                render_toggle(ctx, rect, label, on, self.style, state)
            }
            WidgetKind::Checkbox { label, checked } => {
                render_checkbox(ctx, rect, label, checked, self.style, state)
            }
            WidgetKind::Slider { value, min, max } => {
                render_slider(ctx, rect, value, min, max, self.style, state)
            }
            WidgetKind::ValueLabel { label, value } => {
                render_value_label(ctx, rect, label, value, self.style, state)
            }
            WidgetKind::IconButton { icon, label } => {
                render_icon_button(ctx, rect, icon, label, self.style, state)
            }
            WidgetKind::List {
                items,
                selected,
                offset,
                visible_rows,
            } => render_list(
                ctx,
                rect,
                items,
                selected,
                offset,
                visible_rows,
                self.style,
                state,
            ),
            WidgetKind::ScrollView {
                offset_y,
                content_h,
            } => render_scroll_view(ctx, rect, offset_y, content_h, self.style, state),
            WidgetKind::Tabs { labels, selected } => {
                render_tabs(ctx, rect, labels, selected, self.style, state)
            }
            WidgetKind::Dialog { title, body } => {
                render_dialog(ctx, rect, title, body, self.style, state)
            }
            WidgetKind::Toast { text, ttl_ms } => {
                render_toast(ctx, rect, text, ttl_ms, self.style, state)
            }
            WidgetKind::Meter { value, min, max } => {
                render_meter(ctx, rect, value, min, max, self.style, state)
            }
            WidgetKind::ArcGauge {
                value,
                min,
                max,
                start_deg,
                end_deg,
                thickness,
                antialias,
                major_ticks,
                minor_ticks,
                show_value,
            } => render_arc_gauge(
                ctx,
                rect,
                value,
                min,
                max,
                start_deg,
                end_deg,
                thickness,
                antialias,
                major_ticks,
                minor_ticks,
                show_value,
                self.style,
                state,
            ),
            WidgetKind::Gauge {
                value,
                min,
                max,
                major_ticks,
                minor_ticks,
                show_value,
            } => {
                render_gauge(
                    ctx,
                    rect,
                    value,
                    min,
                    max,
                    major_ticks,
                    minor_ticks,
                    show_value,
                    self.style,
                    state,
                )
            }
            WidgetKind::GaugeNeedle {
                value,
                min,
                max,
                start_deg,
                end_deg,
            } => render_gauge_needle(ctx, rect, value, min, max, start_deg, end_deg, self.style, state),
            WidgetKind::Chart {
                values,
                min,
                max,
                thickness,
                fill_under,
                markers,
                mode,
                show_grid,
                show_axes,
                show_labels,
            } => {
                render_chart(
                    ctx,
                    rect,
                    values,
                    min,
                    max,
                    thickness,
                    fill_under,
                    markers,
                    mode,
                    show_grid,
                    show_axes,
                    show_labels,
                    self.style,
                    state,
                )
            }
            WidgetKind::Spinner { phase } => render_spinner(ctx, rect, phase, self.style, state),
            WidgetKind::Dropdown {
                items,
                selected,
                open,
            } => {
                render_dropdown(ctx, rect, items, selected, open, self.style, state)
            }
            WidgetKind::Roller { items, selected } => {
                render_roller(ctx, rect, items, selected, self.style, state)
            }
            WidgetKind::Table {
                rows,
                separators,
                cell_padding,
                align,
            } => render_table(
                ctx,
                rect,
                rows,
                separators,
                cell_padding,
                align,
                self.style,
                state,
            ),
            WidgetKind::TextArea {
                text,
                cursor,
                placeholder,
            } => render_textarea(ctx, rect, text, cursor, placeholder, self.style, state),
            WidgetKind::Keyboard {
                keys,
                selected,
                cols,
                alt_keys,
                layout,
                ..
            } => render_keyboard(ctx, rect, keys, selected, cols, alt_keys, layout, self.style, state),
            WidgetKind::Image { image, fit } => render_image(ctx, rect, image, fit, self.style, state),
            WidgetKind::Border => ctx.stroke_rect(rect, self.style.resolve(state).border),
            WidgetKind::Spacer => Ok(()),
            WidgetKind::Menu { items, selected } => {
                render_menu(ctx, rect, items, selected, self.style, state)
            }
        }
    }
}

const fn default_flags(kind: WidgetKind<'_>) -> WidgetFlags {
    let mut flags = WidgetFlags::from_bits(
        WidgetFlags::CLIP_CHILDREN.bits() | WidgetFlags::EVENT_BUBBLE.bits(),
    );
    if kind.focusable() {
        flags = WidgetFlags::from_bits(
            flags.bits() | WidgetFlags::FOCUSABLE.bits() | WidgetFlags::CLICKABLE.bits(),
        );
    }
    if matches!(kind, WidgetKind::ScrollView { .. }) {
        flags = WidgetFlags::from_bits(flags.bits() | WidgetFlags::SCROLLABLE.bits());
    }
    flags
}

fn render_panel<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    Block::styled(style).render(rect, ctx)
}

fn render_label<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    text: &str,
    style: WidgetStyle,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(VisualState::Normal);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    ctx.draw_text_in(
        inner,
        text,
        TextStyle::new(style.text).with_font(style.font),
    )
}

fn render_button<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    text: &str,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let active_style = style.resolve(state);
    let block = Block::styled(active_style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    ctx.draw_text_in(
        inner,
        text,
        TextStyle::new(active_style.text)
            .with_font(active_style.font)
            .centered(),
    )
}

fn render_progress<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    value: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let fill_w = ((inner.w as f32 * value.clamp(0.0, 1.0)) as u32).min(inner.w);
    if fill_w > 0 {
        let color = if matches!(state, VisualState::Focused) {
            style.accent
        } else {
            style.foreground
        };
        ctx.fill_rect(Rect::new(inner.x, inner.y, fill_w, inner.h), color)?;
    }
    Ok(())
}

fn render_toggle<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    label: &str,
    on: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let knob_w = (inner.w / 4).max(8).min(inner.w);
    let track = Rect::new(
        inner.right() - knob_w as i32 - 2,
        inner.y + 1,
        knob_w,
        inner.h.saturating_sub(2),
    );
    ctx.fill_rect(
        track,
        if on {
            style.accent
        } else {
            Rgb565::new(7, 10, 10)
        },
    )?;
    ctx.draw_text_in(
        Rect::new(
            inner.x,
            inner.y,
            inner.w.saturating_sub(knob_w + 4),
            inner.h,
        ),
        label,
        TextStyle::new(style.text).with_font(style.font),
    )
}

fn render_checkbox<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    label: &str,
    checked: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let box_size = inner.h.min(8);
    let box_rect = Rect::new(
        inner.x,
        inner.y + (inner.h.saturating_sub(box_size) as i32 / 2),
        box_size,
        box_size,
    );
    ctx.stroke_rect(box_rect, Border::one(style.text))?;
    if checked && box_size > 4 {
        ctx.fill_rect(
            box_rect.inset(crate::geometry::EdgeInsets::all(2)),
            style.accent,
        )?;
    }
    ctx.draw_text_in(
        Rect::new(
            inner.x + box_size as i32 + 3,
            inner.y,
            inner.w.saturating_sub(box_size + 3),
            inner.h,
        ),
        label,
        TextStyle::new(style.text).with_font(style.font),
    )
}

fn render_slider<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    value: f32,
    min: f32,
    max: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let range = (max - min).max(f32::EPSILON);
    let t = ((value - min) / range).clamp(0.0, 1.0);
    let track_y = inner.y + inner.h as i32 / 2;
    ctx.fill_rect(Rect::new(inner.x, track_y, inner.w, 1), style.text)?;
    let knob_x = inner.x + ((inner.w.saturating_sub(3) as f32 * t) as i32);
    ctx.fill_rect(Rect::new(knob_x, track_y - 2, 3, 5), style.accent)
}

fn render_value_label<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    label: &str,
    value: i32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    ctx.draw_text_in(
        Rect::new(inner.x, inner.y, inner.w / 2, inner.h),
        label,
        TextStyle::new(style.text).with_font(style.font),
    )?;
    draw_i32_right(
        ctx,
        Rect::new(
            inner.x + (inner.w / 2) as i32,
            inner.y,
            inner.w - inner.w / 2,
            inner.h,
        ),
        value,
        style.accent,
    )
}

fn render_icon_button<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    icon: char,
    label: &str,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let mut icon_buf = [0u8; 4];
    let icon_str = icon.encode_utf8(&mut icon_buf);
    ctx.draw_text_in(
        Rect::new(inner.x, inner.y, 8, inner.h),
        icon_str,
        TextStyle::new(style.accent)
            .with_font(style.font)
            .centered(),
    )?;
    ctx.draw_text_in(
        Rect::new(inner.x + 10, inner.y, inner.w.saturating_sub(10), inner.h),
        label,
        TextStyle::new(style.text).with_font(style.font),
    )
}

#[allow(clippy::too_many_arguments)]
fn render_list<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    items: &[&str],
    selected: usize,
    offset: usize,
    visible_rows: usize,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    if items.is_empty() {
        return Ok(());
    }
    let inner = block.inner(rect);
    let rows = visible_rows.max(1).min(items.len());
    let row_h = (inner.h / rows as u32).max(1);
    for row_idx in 0..rows {
        let item_idx = offset.saturating_add(row_idx);
        if item_idx >= items.len() {
            break;
        }
        let row = Rect::new(
            inner.x,
            inner.y + (row_idx as u32 * row_h) as i32,
            inner.w,
            row_h,
        );
        if item_idx == selected {
            ctx.fill_rect(row, style.accent)?;
        }
        ctx.draw_text_in(
            row.inset(crate::geometry::EdgeInsets::symmetric(2, 1)),
            items[item_idx],
            TextStyle {
                color: style.text,
                font: style.font,
                opacity: style.opacity,
                align: TextAlign::Left,
                vertical_align: VerticalAlign::Middle,
                wrap: TextWrap::None,
                overflow: crate::render::TextOverflow::Clip,
                overflow_policy: crate::render::TextOverflowPolicy::Global(
                    crate::render::TextOverflow::Clip,
                ),
                kerning: false,
                max_lines: None,
                ellipsis: crate::render::EllipsisMode::ThreeDots,
                line_spacing: 0,
            },
        )?;
    }
    Ok(())
}

fn render_scroll_view<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    offset_y: i32,
    content_h: u32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    if content_h > rect.h {
        let inner = block.inner(rect);
        let thumb_h = ((inner.h as u64 * inner.h as u64) / content_h.max(1) as u64)
            .max(4)
            .min(inner.h as u64) as u32;
        let max_offset = content_h.saturating_sub(inner.h).max(1) as i32;
        let y = inner.y
            + ((inner.h.saturating_sub(thumb_h) as i32 * offset_y.clamp(0, max_offset))
                / max_offset);
        ctx.fill_rect(Rect::new(inner.right() - 3, y, 2, thumb_h), style.accent)?;
    }
    Ok(())
}

fn render_tabs<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    labels: &[&str],
    selected: usize,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    if labels.is_empty() {
        return Ok(());
    }
    let inner = block.inner(rect);
    let tab_w = (inner.w / labels.len() as u32).max(1);
    for (idx, label) in labels.iter().enumerate() {
        let tab = Rect::new(
            inner.x + (idx as u32 * tab_w) as i32,
            inner.y,
            tab_w,
            inner.h,
        );
        if idx == selected {
            ctx.fill_rect(tab, style.accent)?;
        }
        ctx.draw_text_in(
            tab.inset(EdgeInsets::all(1)),
            label,
            TextStyle::new(style.text).with_font(style.font).centered(),
        )?;
    }
    Ok(())
}

fn render_dialog<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    title: &str,
    body: &str,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style)
        .title(title)
        .title_align(TextAlign::Center);
    block.render(rect, ctx)?;
    let inner = block.content_area(rect);
    ctx.draw_text_in(
        inner,
        body,
        TextStyle {
            color: style.text,
            font: style.font,
            opacity: style.opacity,
            align: TextAlign::Center,
            vertical_align: VerticalAlign::Middle,
            wrap: TextWrap::Character,
            overflow: crate::render::TextOverflow::Clip,
            overflow_policy: crate::render::TextOverflowPolicy::Global(
                crate::render::TextOverflow::Clip,
            ),
            kerning: false,
            max_lines: None,
            ellipsis: crate::render::EllipsisMode::ThreeDots,
            line_spacing: 1,
        },
    )
}

fn render_toast<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    text: &str,
    ttl_ms: u32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    if ttl_ms == 0 {
        return Ok(());
    }
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    ctx.draw_text_in(
        block.inner(rect),
        text,
        TextStyle {
            color: style.text,
            font: style.font,
            opacity: style.opacity,
            align: TextAlign::Center,
            vertical_align: VerticalAlign::Middle,
            wrap: TextWrap::Character,
            overflow: crate::render::TextOverflow::Clip,
            overflow_policy: crate::render::TextOverflowPolicy::Global(
                crate::render::TextOverflow::Clip,
            ),
            kerning: false,
            max_lines: None,
            ellipsis: crate::render::EllipsisMode::ThreeDots,
            line_spacing: 0,
        },
    )
}

fn render_meter<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    value: f32,
    min: f32,
    max: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let range = (max - min).max(f32::EPSILON);
    let t = ((value - min) / range).clamp(0.0, 1.0);
    let bars = 10usize;
    let gap = 1u32;
    let bar_w = inner
        .w
        .saturating_sub(gap * (bars as u32 - 1))
        .max(bars as u32)
        / bars as u32;
    for i in 0..bars {
        let x = inner.x + (i as u32 * (bar_w + gap)) as i32;
        let active = (i as f32) < t * bars as f32;
        let h = ((inner.h as f32 * (i + 1) as f32 / bars as f32) as u32).max(1);
        let y = inner.bottom() - h as i32;
        ctx.fill_rect(
            Rect::new(x, y, bar_w, h),
            if active {
                style.accent
            } else {
                Rgb565::new(5, 8, 8)
            },
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_arc_gauge<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    value: f32,
    min: f32,
    max: f32,
    start_deg: i32,
    end_deg: i32,
    thickness: u8,
    antialias: bool,
    major_ticks: u8,
    minor_ticks: u8,
    show_value: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let cx = inner.x + inner.w as i32 / 2;
    let cy = inner.y + inner.h as i32 / 2;
    let radius = (inner.w.min(inner.h) / 2).saturating_sub(1);
    let track = Rgb565::new(5, 8, 8);
    draw_arc_ticks(
        ctx,
        cx,
        cy,
        radius.saturating_sub((thickness.max(1) / 2) as u32),
        start_deg,
        end_deg,
        major_ticks,
        minor_ticks,
        track,
    )?;
    ctx.stroke_arc_styled(
        cx,
        cy,
        radius,
        start_deg,
        end_deg,
        StrokeStyle::new(track)
            .with_width(thickness)
            .with_antialias(antialias),
    )?;
    let range = (max - min).max(f32::EPSILON);
    let t = ((value - min) / range).clamp(0.0, 1.0);
    let active_end = start_deg + (((end_deg - start_deg) as f32) * t) as i32;
    ctx.stroke_arc_styled(
        cx,
        cy,
        radius,
        start_deg,
        active_end,
        StrokeStyle::new(style.accent)
            .with_width(thickness)
            .with_antialias(antialias),
    )?;
    if show_value {
        draw_gauge_value_label(ctx, inner, value, min, max, style)?;
    }
    Ok(())
}

fn render_gauge<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    value: f32,
    min: f32,
    max: f32,
    major_ticks: u8,
    minor_ticks: u8,
    show_value: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    render_arc_gauge(
        ctx,
        rect,
        value,
        min,
        max,
        135,
        405,
        2,
        true,
        major_ticks,
        minor_ticks,
        show_value,
        style,
        state,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_gauge_needle<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    value: f32,
    min: f32,
    max: f32,
    start_deg: i32,
    end_deg: i32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let cx = inner.x + inner.w as i32 / 2;
    let cy = inner.y + inner.h as i32 / 2;
    let radius = (inner.w.min(inner.h) / 2).saturating_sub(2);
    ctx.stroke_arc_styled(
        cx,
        cy,
        radius,
        start_deg,
        end_deg,
        StrokeStyle::new(Rgb565::new(8, 10, 10)).with_width(1),
    )?;
    let range = (max - min).max(f32::EPSILON);
    let t = ((value - min) / range).clamp(0.0, 1.0);
    let angle = (start_deg as f32 + (end_deg - start_deg) as f32 * t).to_radians();
    let nx = cx + (radius as f32 * angle.cos()) as i32;
    let ny = cy + (radius as f32 * angle.sin()) as i32;
    ctx.draw_line_styled(
        cx,
        cy,
        nx,
        ny,
        StrokeStyle::new(style.accent)
            .with_width(2)
            .with_antialias(true)
            .with_cap(crate::render::StrokeCap::Round),
    )?;
    ctx.fill_circle(cx, cy, 2, style.accent)
}

fn render_chart<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    values: &[f32],
    min: f32,
    max: f32,
    thickness: u8,
    fill_under: bool,
    markers: bool,
    mode: ChartMode,
    show_grid: bool,
    show_axes: bool,
    show_labels: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    if values.len() < 2 {
        return Ok(());
    }
    let inner = block.inner(rect);
    if show_grid {
        for row in [1u32, 2, 3] {
            let y = inner.y + ((inner.h.saturating_sub(1) * row) / 4) as i32;
            ctx.draw_line_styled(
                inner.x,
                y,
                inner.right().saturating_sub(1),
                y,
                StrokeStyle::new(Rgb565::new(6, 10, 10)).with_width(1),
            )?;
        }
    }
    if show_axes {
        let axis = Rgb565::new(12, 18, 18);
        ctx.draw_line_styled(
            inner.x,
            inner.y,
            inner.x,
            inner.bottom().saturating_sub(1),
            StrokeStyle::new(axis).with_width(1),
        )?;
        ctx.draw_line_styled(
            inner.x,
            inner.bottom().saturating_sub(1),
            inner.right().saturating_sub(1),
            inner.bottom().saturating_sub(1),
            StrokeStyle::new(axis).with_width(1),
        )?;
    }
    if show_labels {
        let mut max_label: String<12> = String::new();
        let _ = write!(&mut max_label, "{:.1}", max);
        let mut min_label: String<12> = String::new();
        let _ = write!(&mut min_label, "{:.1}", min);
        ctx.draw_text_in(
            Rect::new(inner.x + 1, inner.y, inner.w.saturating_sub(2), style.font.line_height()),
            max_label.as_str(),
            TextStyle::new(style.text).with_font(style.font),
        )?;
        ctx.draw_text_in(
            Rect::new(
                inner.x + 1,
                inner.bottom().saturating_sub(style.font.line_height() as i32),
                inner.w.saturating_sub(2),
                style.font.line_height(),
            ),
            min_label.as_str(),
            TextStyle::new(style.text).with_font(style.font),
        )?;
    }
    let range = (max - min).max(f32::EPSILON);
    match mode {
        ChartMode::Line => {
            let dx = (inner.w.saturating_sub(1) as f32) / (values.len().saturating_sub(1) as f32);
            for i in 1..values.len() {
                let v0 = ((values[i - 1] - min) / range).clamp(0.0, 1.0);
                let v1 = ((values[i] - min) / range).clamp(0.0, 1.0);
                let x0 = inner.x + ((i - 1) as f32 * dx) as i32;
                let x1 = inner.x + (i as f32 * dx) as i32;
                let y0 = inner.bottom() - 1 - (v0 * (inner.h.saturating_sub(1)) as f32) as i32;
                let y1 = inner.bottom() - 1 - (v1 * (inner.h.saturating_sub(1)) as f32) as i32;
                if fill_under {
                    let base = inner.bottom() - 1;
                    ctx.fill_polygon(
                        &[
                            embedded_graphics_core::geometry::Point::new(x0, base),
                            embedded_graphics_core::geometry::Point::new(x0, y0),
                            embedded_graphics_core::geometry::Point::new(x1, y1),
                            embedded_graphics_core::geometry::Point::new(x1, base),
                        ],
                        Rgb565::new(2, 8, 2),
                    )?;
                }
                ctx.draw_line_styled(
                    x0,
                    y0,
                    x1,
                    y1,
                    StrokeStyle::new(style.accent)
                        .with_width(thickness.max(1))
                        .with_antialias(true),
                )?;
                if markers {
                    ctx.fill_circle(x0, y0, 1, style.accent)?;
                    ctx.fill_circle(x1, y1, 1, style.accent)?;
                }
            }
        }
        ChartMode::Bars => {
            let count = values.len() as u32;
            let gap = 1u32;
            let bar_w = inner
                .w
                .saturating_sub(gap.saturating_mul(count.saturating_sub(1)))
                .max(count)
                / count;
            for (i, value) in values.iter().copied().enumerate() {
                let t = ((value - min) / range).clamp(0.0, 1.0);
                let h = (t * inner.h.saturating_sub(1) as f32) as u32;
                let x = inner.x + (i as u32 * (bar_w + gap)) as i32;
                let y = inner.bottom().saturating_sub(h as i32 + 1);
                let bar = Rect::new(x, y, bar_w.max(1), h.max(1));
                ctx.fill_rect(bar, style.accent)?;
                if markers {
                    ctx.fill_circle(x + (bar_w / 2) as i32, y, 1, style.text)?;
                }
            }
        }
    }
    Ok(())
}

fn render_spinner<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    phase: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let cx = inner.x + inner.w as i32 / 2;
    let cy = inner.y + inner.h as i32 / 2;
    let radius = (inner.w.min(inner.h) / 2).saturating_sub(1);
    let base = ((phase.fract() * 360.0) as i32).rem_euclid(360);
    ctx.stroke_arc_styled(
        cx,
        cy,
        radius,
        base,
        base + 120,
        StrokeStyle::new(style.accent).with_width(2).with_antialias(true),
    )
}

fn render_dropdown<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    items: &[&str],
    selected: usize,
    open: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let text = if open {
        items.get(selected).copied().unwrap_or("-")
    } else {
        items.get(selected).copied().unwrap_or("-")
    };
    ctx.draw_text_in(
        Rect::new(inner.x, inner.y, inner.w.saturating_sub(8), inner.h),
        text,
        TextStyle::new(style.text).with_font(style.font),
    )?;
    ctx.draw_text_in(
        Rect::new(inner.right() - 7, inner.y, 7, inner.h),
        if open { "^" } else { "v" },
        TextStyle::new(style.accent).with_font(style.font).centered(),
    )?;
    if open {
        let row_h = style.font.line_height().max(6);
        let popup_h = (row_h.saturating_mul(items.len() as u32)).min(40).max(row_h);
        let popup = Rect::new(inner.x, inner.bottom() as i32 + 1, inner.w, popup_h);
        ctx.fill_rect(popup, style.background.unwrap_or(Rgb565::new(8, 12, 16)))?;
        ctx.stroke_rect(popup, Border::one(style.border.color))?;
        let visible = (popup_h / row_h).max(1) as usize;
        let start = selected.saturating_sub(visible / 2).min(items.len().saturating_sub(visible));
        for (i, item) in items.iter().enumerate().skip(start).take(visible) {
            let row = Rect::new(
                popup.x + 1,
                popup.y + ((i - start) as u32 * row_h) as i32,
                popup.w.saturating_sub(2),
                row_h,
            );
            if i == selected {
                ctx.fill_rect(row, style.accent)?;
            }
            ctx.draw_text_in(
                row.inset(EdgeInsets::all(1)),
                item,
                TextStyle::new(style.text).with_font(style.font),
            )?;
        }
    }
    Ok(())
}

fn render_roller<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    items: &[&str],
    selected: usize,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    if items.is_empty() {
        return Ok(());
    }
    let inner = block.inner(rect);
    let prev = items[(selected + items.len() - 1) % items.len()];
    let cur = items[selected];
    let next = items[(selected + 1) % items.len()];
    let row_h = (inner.h / 3).max(1);
    let rows = [prev, cur, next];
    for (idx, text) in rows.iter().enumerate() {
        let row = Rect::new(inner.x, inner.y + (idx as u32 * row_h) as i32, inner.w, row_h);
        if idx == 1 {
            ctx.fill_rect(row, style.accent)?;
        }
        ctx.draw_text_in(
            row,
            text,
            TextStyle::new(style.text).with_font(style.font).centered(),
        )?;
    }
    Ok(())
}

fn render_table<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    rows: &[&[&str]],
    separators: bool,
    cell_padding: u8,
    align: TextAlign,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    if rows.is_empty() {
        return Ok(());
    }
    let inner = block.inner(rect);
    let row_h = (inner.h / rows.len() as u32).max(1);
    let max_cols = rows.iter().map(|row| row.len()).max().unwrap_or(1).max(1);
    let col_w = (inner.w / max_cols as u32).max(1);
    for (r, cols) in rows.iter().enumerate() {
        for c in 0..max_cols {
            let text = cols.get(c).copied().unwrap_or("");
            let cell = Rect::new(
                inner.x + (c as u32 * col_w) as i32,
                inner.y + (r as u32 * row_h) as i32,
                col_w,
                row_h,
            );
            if separators {
                ctx.stroke_rect(cell, Border::one(style.border.color))?;
            }
            ctx.draw_text_in(
                cell.inset(EdgeInsets::all(cell_padding as i16)),
                text,
                TextStyle::new(style.text)
                    .with_font(style.font)
                    .with_align(align),
            )?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_arc_ticks<D>(
    ctx: &mut RenderCtx<'_, D>,
    cx: i32,
    cy: i32,
    radius: u32,
    start_deg: i32,
    end_deg: i32,
    major_ticks: u8,
    minor_ticks: u8,
    color: Rgb565,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let major_ticks = major_ticks.max(1);
    let minor_ticks = minor_ticks.max(1);
    let total_steps = (major_ticks as u32).saturating_mul(minor_ticks as u32);
    for step in 0..=total_steps {
        let t = if total_steps == 0 {
            0.0
        } else {
            step as f32 / total_steps as f32
        };
        let angle = (start_deg as f32 + (end_deg - start_deg) as f32 * t).to_radians();
        let is_major = step % minor_ticks as u32 == 0;
        let tick_len = if is_major { 4 } else { 2 };
        let outer_x = cx + (radius as f32 * angle.cos()) as i32;
        let outer_y = cy + (radius as f32 * angle.sin()) as i32;
        let inner_x = cx + ((radius.saturating_sub(tick_len)) as f32 * angle.cos()) as i32;
        let inner_y = cy + ((radius.saturating_sub(tick_len)) as f32 * angle.sin()) as i32;
        ctx.draw_line_styled(
            inner_x,
            inner_y,
            outer_x,
            outer_y,
            StrokeStyle::new(color).with_width(1),
        )?;
    }
    Ok(())
}

fn draw_gauge_value_label<D>(
    ctx: &mut RenderCtx<'_, D>,
    inner: Rect,
    value: f32,
    min: f32,
    max: f32,
    style: Style,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let range = (max - min).max(f32::EPSILON);
    let percent = (((value - min) / range).clamp(0.0, 1.0) * 100.0).round() as i32;
    let mut label: String<8> = String::new();
    let _ = write!(&mut label, "{}%", percent);
    ctx.draw_text_in(
        Rect::new(
            inner.x,
            inner.y + (inner.h as i32 / 2) - (style.font.line_height() as i32 / 2),
            inner.w,
            style.font.line_height(),
        ),
        label.as_str(),
        TextStyle::new(style.text)
            .with_font(style.font)
            .with_align(TextAlign::Center),
    )
}

fn render_textarea<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    text: &str,
    cursor: usize,
    placeholder: &str,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect).inset(EdgeInsets::all(1));
    let shown = if text.is_empty() { placeholder } else { text };
    let color = if text.is_empty() {
        Rgb565::new(
            style.text.r().saturating_sub(8),
            style.text.g().saturating_sub(10),
            style.text.b().saturating_sub(8),
        )
    } else {
        style.text
    };
    ctx.draw_text_in(inner, shown, TextStyle::new(color).with_font(style.font))?;
    let chars = text.chars().count();
    let cursor = cursor.min(chars);
    let x = inner.x + (cursor as u32 * style.font.advance()) as i32;
    let caret = Rect::new(x, inner.y, 1, style.font.line_height().min(inner.h));
    ctx.fill_rect(caret, style.accent)
}

fn render_keyboard<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    keys: &[char],
    selected: usize,
    cols: u8,
    alt_keys: Option<&[char]>,
    layout: KeyboardLayout,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    if keys.is_empty() {
        return Ok(());
    }
    let inner = block.inner(rect).inset(EdgeInsets::all(1));
    let cols = cols.max(1) as usize;
    let rows = keys.len().div_ceil(cols).max(1);
    let cell_w = (inner.w / cols as u32).max(1);
    let cell_h = (inner.h / rows as u32).max(1);
    for (idx, key) in keys.iter().copied().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let cell = Rect::new(
            inner.x + (col as u32 * cell_w) as i32,
            inner.y + (row as u32 * cell_h) as i32,
            cell_w,
            cell_h,
        );
        if idx == selected.min(keys.len() - 1) {
            ctx.fill_rect(cell, style.accent)?;
        }
        let rendered = keyboard_key_for_layout(key, idx, keys, alt_keys, layout);
        let mut label = [0u8; 4];
        let text = rendered.encode_utf8(&mut label);
        ctx.draw_text_in(
            cell.inset(EdgeInsets::all(1)),
            text,
            TextStyle::new(style.text).with_font(style.font).centered(),
        )?;
    }
    Ok(())
}

fn keyboard_key_for_layout(
    base: char,
    idx: usize,
    base_keys: &[char],
    alt_keys: Option<&[char]>,
    layout: KeyboardLayout,
) -> char {
    match layout {
        KeyboardLayout::Normal => base,
        KeyboardLayout::Shift => {
            if base.is_ascii_alphabetic() {
                base.to_ascii_uppercase()
            } else {
                base
            }
        }
        KeyboardLayout::Symbols => alt_keys
            .and_then(|keys| keys.get(idx).copied())
            .or_else(|| {
                const FALLBACK: [char; 10] = ['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'];
                FALLBACK.get(idx % FALLBACK.len()).copied()
            })
            .unwrap_or_else(|| base_keys.get(idx).copied().unwrap_or(base)),
    }
}

fn render_menu<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    items: &[&str],
    selected: usize,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;

    if items.is_empty() {
        return Ok(());
    }

    let inner = block.inner(rect);
    let row_h = (inner.h / items.len() as u32).max(1);
    for (i, item) in items.iter().enumerate() {
        let row = Rect::new(inner.x, inner.y + (i as u32 * row_h) as i32, inner.w, row_h);
        let is_selected = i == selected;
        if is_selected {
            ctx.fill_rect(row, style.accent)?;
        }
        ctx.draw_text_in(
            row.inset(crate::geometry::EdgeInsets::symmetric(2, 1)),
            item,
            TextStyle {
                color: style.text,
                font: style.font,
                opacity: style.opacity,
                align: TextAlign::Left,
                vertical_align: VerticalAlign::Middle,
                wrap: TextWrap::None,
                overflow: crate::render::TextOverflow::Clip,
                overflow_policy: crate::render::TextOverflowPolicy::Global(
                    crate::render::TextOverflow::Clip,
                ),
                kerning: false,
                max_lines: None,
                ellipsis: crate::render::EllipsisMode::ThreeDots,
                line_spacing: 0,
            },
        )?;
    }
    Ok(())
}

fn render_image<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    image: ImageRef<'_>,
    fit: ImageFit,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    ctx.draw_image(block.inner(rect), image, fit)
}

fn draw_i32_right<D>(
    ctx: &mut RenderCtx<'_, D>,
    rect: Rect,
    value: i32,
    color: Rgb565,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
{
    let mut buf = [0u8; 12];
    let mut n = value.unsigned_abs();
    let negative = value < 0;
    let mut pos = buf.len();
    if n == 0 {
        pos -= 1;
        buf[pos] = b'0';
    } else {
        while n > 0 && pos > usize::from(negative) {
            pos -= 1;
            buf[pos] = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }
    if negative && pos > 0 {
        pos -= 1;
        buf[pos] = b'-';
    }
    let text = core::str::from_utf8(&buf[pos..]).unwrap_or("?");
    ctx.draw_text_in(
        rect,
        text,
        TextStyle {
            color,
            font: crate::font::FontId::Tiny3x5,
            opacity: 255,
            align: TextAlign::Right,
            vertical_align: VerticalAlign::Middle,
            wrap: TextWrap::None,
            overflow: crate::render::TextOverflow::Clip,
            overflow_policy: crate::render::TextOverflowPolicy::Global(
                crate::render::TextOverflow::Clip,
            ),
            kerning: false,
            max_lines: None,
            ellipsis: crate::render::EllipsisMode::ThreeDots,
            line_spacing: 0,
        },
    )
}

impl Default for WidgetKind<'_> {
    fn default() -> Self {
        Self::Spacer
    }
}

impl Default for WidgetNode<'_> {
    fn default() -> Self {
        Self::new(
            WidgetId::new(0),
            Rect::empty(),
            WidgetKind::Spacer,
            WidgetStyle::new(Style {
                background: None,
                gradient: None,
                font: crate::font::FontId::Tiny3x5,
                foreground: Rgb565::WHITE,
                text: Rgb565::WHITE,
                accent: Rgb565::WHITE,
                opacity: 255,
                corner_radius: 0,
                shadow: None,
                border: Border::none(),
                padding: crate::geometry::EdgeInsets::all(0),
            }),
        )
    }
}
