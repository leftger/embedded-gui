use core::fmt::Write;

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use heapless::String;

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    block::Block,
    geometry::{EdgeInsets, Rect},
    image::{ImageFit, ImageRef, ReelPlayer},
    render::{Compositor, RenderCtx, StrokeStyle, TextAlign, TextStyle, TextWrap, VerticalAlign},
    style::{Border, Style, VisualState, WidgetStyle},
    widget::{FocusGroupId, StyleClassId, WidgetFlags, WidgetId},
};

pub const TEXTAREA_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceState {
    Ready,
    Loading,
    Empty,
    Error,
    Offline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum WidgetKind<'a> {
    Panel,
    Label(&'a str),
    Button(&'a str),
    ProgressBar {
        value: f32,
    },
    #[cfg(feature = "rich-widgets")]
    Toggle {
        label: &'a str,
        on: bool,
    },
    #[cfg(feature = "rich-widgets")]
    Checkbox {
        label: &'a str,
        checked: bool,
    },
    #[cfg(feature = "rich-widgets")]
    Slider {
        value: f32,
        min: f32,
        max: f32,
    },
    #[cfg(feature = "rich-widgets")]
    ValueLabel {
        label: &'a str,
        value: i32,
    },
    #[cfg(feature = "rich-widgets")]
    IconButton {
        icon: char,
        label: &'a str,
    },
    #[cfg(feature = "rich-widgets")]
    List {
        items: &'a [&'a str],
        selected: usize,
        offset: usize,
        visible_rows: usize,
    },
    #[cfg(feature = "rich-widgets")]
    ScrollView {
        offset_y: i32,
        content_h: u32,
    },
    #[cfg(feature = "rich-widgets")]
    Tabs {
        labels: &'a [&'a str],
        selected: usize,
    },
    #[cfg(feature = "rich-widgets")]
    Dialog {
        title: &'a str,
        body: &'a str,
    },
    #[cfg(feature = "rich-widgets")]
    Toast {
        text: &'a str,
        ttl_ms: u32,
    },
    #[cfg(feature = "rich-widgets")]
    Meter {
        value: f32,
        min: f32,
        max: f32,
    },
    #[cfg(feature = "rich-widgets")]
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
    #[cfg(feature = "rich-widgets")]
    Gauge {
        value: f32,
        min: f32,
        max: f32,
        major_ticks: u8,
        minor_ticks: u8,
        show_value: bool,
    },
    #[cfg(feature = "rich-widgets")]
    GaugeNeedle {
        value: f32,
        min: f32,
        max: f32,
        start_deg: i32,
        end_deg: i32,
    },
    /// Countdown/progress sweep: a filled pie-sector that grows clockwise
    /// with `progress` (0.0..=1.0) over a solid background, with a
    /// rounded-rect "window" punched in the middle for a caller-drawn value
    /// (e.g. a large countdown numeral in a font the crate doesn't own).
    /// Numeric-only, so it needs no lifetime and is drivable by
    /// [`crate::WidgetAnimator`] through `set_progress`.
    SweepingArc {
        progress: f32,
        arc_radius: u32,
        frame_inset: u16,
        corner_radius: u8,
        bg_color: Rgb565,
        arc_color: Rgb565,
        frame_color: Rgb565,
    },
    #[cfg(feature = "rich-widgets")]
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
    Plotter {
        values: &'a [f32],
        head: usize,
        min: f32,
        max: f32,
        thickness: u8,
        show_grid: bool,
        show_axes: bool,
    },
    CircularList {
        items: &'a [&'a str],
        selected: usize,
        offset: usize,
        visible_rows: usize,
    },
    Spinner {
        phase: f32,
    },
    #[cfg(feature = "rich-widgets")]
    Dropdown {
        items: &'a [&'a str],
        selected: usize,
        open: bool,
    },
    #[cfg(feature = "rich-widgets")]
    Roller {
        items: &'a [&'a str],
        selected: usize,
    },
    #[cfg(feature = "rich-widgets")]
    Table {
        rows: &'a [&'a [&'a str]],
        separators: bool,
        cell_padding: u8,
        align: TextAlign,
    },
    #[cfg(feature = "rich-widgets")]
    TextArea {
        text_buf: [u8; TEXTAREA_CAPACITY],
        text_len: u8,
        cursor: usize,
        placeholder: &'a str,
        selection: Option<(usize, usize)>,
        cursor_visible: bool,
        read_only: bool,
        single_line: bool,
        accept_newline: bool,
    },
    #[cfg(feature = "rich-widgets")]
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
    #[default]
    Spacer,
    #[cfg(feature = "rich-widgets")]
    Menu {
        items: &'a [&'a str],
        selected: usize,
    },
    #[cfg(feature = "rich-widgets")]
    PeekReveal {
        icon: ImageRef<'a>,
        title: &'a str,
        subtitle: &'a str,
        progress: f32,
    },
    #[cfg(feature = "rich-widgets")]
    GlanceTile {
        icon: char,
        title: &'a str,
        subtitle: &'a str,
        highlighted: bool,
    },
    #[cfg(feature = "rich-widgets")]
    CardDeck {
        titles: &'a [&'a str],
        selected: usize,
    },
    #[cfg(feature = "rich-widgets")]
    Reel {
        player: ReelPlayer<'a>,
        fit: ImageFit,
    },
    #[cfg(feature = "rich-widgets")]
    StateSurface {
        state: SurfaceState,
        title: &'a str,
        message: &'a str,
        action: Option<&'a str>,
        busy_phase: f32,
    },
    #[cfg(feature = "rich-widgets")]
    HeadsUpBanner {
        level: NotificationLevel,
        text: &'a str,
        ttl_ms: u32,
    },
    #[cfg(feature = "rich-widgets")]
    NotificationActionSheet {
        level: NotificationLevel,
        title: &'a str,
        body: &'a str,
        actions: &'a [&'a str],
        selected: usize,
        open: bool,
    },
    #[cfg(feature = "rich-widgets")]
    FeedTimeline {
        items: &'a [&'a str],
        selected: usize,
        offset: usize,
        visible_rows: usize,
        expanded: bool,
    },
    Dial {
        value: f32,
        min: f32,
        max: f32,
    },
    RlePlayer {
        rle_data: &'static [u8],
        frame_width: u16,
        frame_height: u16,
        total_frames: usize,
        current_frame: usize,
        elapsed_ms: u32,
        frame_duration_ms: u32,
    },
    AutoComplete {
        text_buf: [u8; 32],
        text_len: u8,
        suggestions: &'a [&'a str],
        filtered: [Option<&'a str>; 8],
        filter_count: u8,
        selected: Option<usize>,
        expanded: bool,
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
        #[cfg(feature = "rich-widgets")]
        if matches!(
            self,
            Self::Toggle { .. }
                | Self::Checkbox { .. }
                | Self::Slider { .. }
                | Self::IconButton { .. }
                | Self::List { .. }
                | Self::CircularList { .. }
                | Self::ScrollView { .. }
                | Self::Tabs { .. }
                | Self::Dropdown { .. }
                | Self::Roller { .. }
                | Self::TextArea { .. }
                | Self::Keyboard { .. }
                | Self::Menu { .. }
                | Self::FeedTimeline { .. }
                | Self::Dial { .. }
                | Self::AutoComplete { .. }
        ) {
            return true;
        }
        matches!(self, Self::Button { .. } | Self::RlePlayer { .. })
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
    pub fn new<S>(id: WidgetId, rect: impl Into<Rect>, kind: WidgetKind<'a>, style: S) -> Self
    where
        S: Into<WidgetStyle>,
    {
        Self {
            id,
            parent: None,
            style_class: None,
            focus_group: FocusGroupId::ROOT,
            rect: rect.into(),
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

    pub fn render<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        state: VisualState,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        self.render_at(ctx, self.rect, state)
    }

    pub fn render_at<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        rect: Rect,
        state: VisualState,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
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
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Toggle { label, on } => {
                render_toggle(ctx, rect, label, on, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Checkbox { label, checked } => {
                render_checkbox(ctx, rect, label, checked, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Slider { value, min, max } => {
                render_slider(ctx, rect, value, min, max, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::ValueLabel { label, value } => {
                render_value_label(ctx, rect, label, value, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::IconButton { icon, label } => {
                render_icon_button(ctx, rect, icon, label, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
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
            WidgetKind::CircularList {
                items,
                selected,
                offset,
                visible_rows,
            } => render_circular_list(
                ctx,
                rect,
                items,
                selected,
                offset,
                visible_rows,
                self.style,
                state,
            ),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::ScrollView {
                offset_y,
                content_h,
            } => render_scroll_view(ctx, rect, offset_y, content_h, self.style, state),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Tabs { labels, selected } => {
                render_tabs(ctx, rect, labels, selected, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Dialog { title, body } => {
                render_dialog(ctx, rect, title, body, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Toast { text, ttl_ms } => {
                render_toast(ctx, rect, text, ttl_ms, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Meter { value, min, max } => {
                render_meter(ctx, rect, value, min, max, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
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
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Gauge {
                value,
                min,
                max,
                major_ticks,
                minor_ticks,
                show_value,
            } => render_gauge(
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
            ),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::GaugeNeedle {
                value,
                min,
                max,
                start_deg,
                end_deg,
            } => render_gauge_needle(
                ctx, rect, value, min, max, start_deg, end_deg, self.style, state,
            ),
            WidgetKind::SweepingArc {
                progress,
                arc_radius,
                frame_inset,
                corner_radius,
                bg_color,
                arc_color,
                frame_color,
            } => render_sweeping_arc(
                ctx,
                rect,
                progress,
                arc_radius,
                frame_inset,
                corner_radius,
                bg_color,
                arc_color,
                frame_color,
            ),
            #[cfg(feature = "rich-widgets")]
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
            } => render_chart(
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
            ),
            WidgetKind::Plotter {
                values,
                head,
                min,
                max,
                thickness,
                show_grid,
                show_axes,
            } => render_plotter(
                ctx, rect, values, head, min, max, thickness, show_grid, show_axes, self.style,
                state,
            ),
            WidgetKind::Spinner { phase } => render_spinner(ctx, rect, phase, self.style, state),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Dropdown {
                items,
                selected,
                open,
            } => render_dropdown(ctx, rect, items, selected, open, self.style, state),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Roller { items, selected } => {
                render_roller(ctx, rect, items, selected, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
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
            #[cfg(feature = "rich-widgets")]
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                placeholder,
                selection,
                cursor_visible,
                ..
            } => render_textarea(
                ctx,
                rect,
                textarea_text(&text_buf, text_len),
                cursor,
                placeholder,
                selection,
                cursor_visible,
                self.style,
                state,
            ),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Keyboard {
                keys,
                selected,
                cols,
                alt_keys,
                layout,
                ..
            } => render_keyboard(
                ctx, rect, keys, selected, cols, alt_keys, layout, self.style, state,
            ),
            WidgetKind::Image { image, fit } => {
                render_image(ctx, rect, image, fit, self.style, state)
            }
            WidgetKind::Border => ctx.stroke_rect(rect, self.style.resolve(state).border),
            WidgetKind::Spacer => Ok(()),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Menu { items, selected } => {
                render_menu(ctx, rect, items, selected, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::PeekReveal {
                icon,
                title,
                subtitle,
                progress,
            } => render_peek_reveal(
                ctx, rect, icon, title, subtitle, progress, self.style, state,
            ),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::GlanceTile {
                icon,
                title,
                subtitle,
                highlighted,
            } => render_glance_tile(
                ctx,
                rect,
                icon,
                title,
                subtitle,
                highlighted,
                self.style,
                state,
            ),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::CardDeck { titles, selected } => {
                render_card_deck(ctx, rect, titles, selected, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::Reel { player, fit } => {
                render_reel(ctx, rect, player, fit, self.style, state)
            }
            #[cfg(feature = "rich-widgets")]
            WidgetKind::StateSurface {
                state: surface_state,
                title,
                message,
                action,
                busy_phase,
            } => render_state_surface(
                ctx,
                rect,
                surface_state,
                title,
                message,
                action,
                busy_phase,
                self.style,
                state,
            ),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::HeadsUpBanner {
                level,
                text,
                ttl_ms,
            } => render_heads_up_banner(ctx, rect, level, text, ttl_ms, self.style, state),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::NotificationActionSheet {
                level,
                title,
                body,
                actions,
                selected,
                open,
            } => render_notification_action_sheet(
                ctx, rect, level, title, body, actions, selected, open, self.style, state,
            ),
            #[cfg(feature = "rich-widgets")]
            WidgetKind::FeedTimeline {
                items,
                selected,
                offset,
                visible_rows,
                expanded,
            } => render_feed_timeline(
                ctx,
                rect,
                items,
                selected,
                offset,
                visible_rows,
                expanded,
                self.style,
                state,
            ),
            WidgetKind::Dial { value, min, max } => {
                render_dial(ctx, rect, value, min, max, self.style, state)
            }
            WidgetKind::RlePlayer {
                rle_data,
                frame_width,
                frame_height,
                current_frame,
                ..
            } => render_rle_player(
                ctx,
                rect,
                rle_data,
                current_frame,
                frame_width,
                frame_height,
                self.style,
                state,
            ),
            WidgetKind::AutoComplete {
                text_buf,
                text_len,
                filtered,
                filter_count,
                selected,
                expanded,
                ..
            } => render_autocomplete(
                ctx,
                rect,
                &text_buf,
                text_len,
                &filtered,
                filter_count,
                selected,
                expanded,
                self.style,
                state,
            ),
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
    #[cfg(feature = "rich-widgets")]
    if matches!(kind, WidgetKind::ScrollView { .. }) {
        flags = WidgetFlags::from_bits(flags.bits() | WidgetFlags::SCROLLABLE.bits());
    }
    flags
}

fn render_panel<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    Block::styled(style).render(rect, ctx)
}

fn render_label<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    text: &str,
    style: WidgetStyle,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

fn render_button<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    text: &str,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

fn render_progress<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    value: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn render_toggle<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    label: &str,
    on: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn render_checkbox<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    label: &str,
    checked: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn render_slider<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    value: f32,
    min: f32,
    max: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

fn render_dial<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    value: f32,
    min: f32,
    max: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);

    let cx = inner.x + inner.w as i32 / 2;
    let cy = inner.y + inner.h as i32 / 2;
    let radius = (inner.w.min(inner.h) as i32 / 2).saturating_sub(2);

    if radius > 0 {
        ctx.stroke_circle(cx, cy, radius as u32, style.text)?;

        let range = (max - min).max(f32::EPSILON);
        let t = ((value - min) / range).clamp(0.0, 1.0);

        #[cfg(not(feature = "std"))]
        use crate::math::F32Ext as _;

        let angle = t * 2.0 * core::f32::consts::PI - (core::f32::consts::PI / 2.0);
        let cos_val = angle.cos();
        let sin_val = angle.sin();

        let px = cx + (radius as f32 * cos_val).round() as i32;
        let py = cy + (radius as f32 * sin_val).round() as i32;

        ctx.draw_line(cx, cy, px, py, style.accent)?;
        ctx.fill_circle(cx, cy, 2, style.accent)?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::needless_range_loop)]
fn render_rle_player<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    rle_data: &[u8],
    current_frame: usize,
    frame_w: u16,
    frame_h: u16,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);

    if rle_data.len() < 3 {
        return Ok(());
    }
    let total_frames = u16::from_le_bytes([rle_data[0], rle_data[1]]) as usize;
    if current_frame >= total_frames {
        return Ok(());
    }
    let offset_start = 2 + current_frame * 4;
    if offset_start + 4 > rle_data.len() {
        return Ok(());
    }
    let frame_offset = u32::from_le_bytes([
        rle_data[offset_start],
        rle_data[offset_start + 1],
        rle_data[offset_start + 2],
        rle_data[offset_start + 3],
    ]) as usize;

    if frame_offset >= rle_data.len() {
        return Ok(());
    }
    let pal_size = rle_data[frame_offset] as usize;
    let mut pal_colors = [Rgb565::BLACK; 256];
    let pal_colors_start = frame_offset + 1;
    for i in 0..pal_size {
        let idx = pal_colors_start + i * 2;
        if idx + 2 <= rle_data.len() {
            let color_u16 = u16::from_le_bytes([rle_data[idx], rle_data[idx + 1]]);
            let r = ((color_u16 >> 11) & 0x1F) as u8;
            let g = ((color_u16 >> 5) & 0x3F) as u8;
            let b = (color_u16 & 0x1F) as u8;
            pal_colors[i] = Rgb565::new(r, g, b);
        }
    }

    let runs_start = pal_colors_start + pal_size * 2;
    let mut cur_x = 0i32;
    let mut cur_y = 0i32;
    let mut idx = runs_start;

    while idx + 2 <= rle_data.len() {
        let run_len = rle_data[idx] as i32;
        let pal_idx = rle_data[idx + 1] as usize;
        idx += 2;

        if run_len == 0 {
            break;
        }

        let color = if pal_idx < pal_size {
            pal_colors[pal_idx]
        } else {
            Rgb565::BLACK
        };

        for _ in 0..run_len {
            if cur_y >= frame_h as i32 {
                break;
            }
            let px = inner.x + cur_x;
            let py = inner.y + cur_y;
            if inner.contains(px, py) {
                ctx.fill_rect(Rect::new(px, py, 1, 1), color)?;
            }

            cur_x += 1;
            if cur_x >= frame_w as i32 {
                cur_x = 0;
                cur_y += 1;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::needless_range_loop)]
fn render_autocomplete<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    text_buf: &[u8; 32],
    text_len: u8,
    filtered: &[Option<&str>; 8],
    filter_count: u8,
    selected: Option<usize>,
    expanded: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);

    let row_h = style.font.line_height();
    let input_h = row_h.saturating_add(4);
    let input_rect = Rect::new(rect.x, rect.y, rect.w, input_h);

    block.render(input_rect, ctx)?;
    let inner = block.inner(input_rect);

    let current_text = core::str::from_utf8(&text_buf[..text_len as usize]).unwrap_or("");
    if text_len == 0 {
        ctx.draw_text_in(
            inner,
            "Search...",
            TextStyle::new(Rgb565::new(16, 32, 16)).with_font(style.font),
        )?;
    } else {
        ctx.draw_text_in(
            inner,
            current_text,
            TextStyle::new(style.text).with_font(style.font),
        )?;

        if state == VisualState::Focused {
            let cursor_x =
                inner.x + current_text.chars().count() as i32 * style.font.advance() as i32;
            if cursor_x < inner.right() {
                ctx.fill_rect(Rect::new(cursor_x, inner.y, 1, inner.h), style.accent)?;
            }
        }
    }

    ctx.draw_text_in(
        Rect::new(inner.right() - 7, inner.y, 7, inner.h),
        if expanded { "^" } else { "v" },
        TextStyle::new(style.accent)
            .with_font(style.font)
            .centered(),
    )?;

    if expanded && filter_count > 0 {
        let popup_h = (row_h.saturating_add(2))
            .saturating_mul(filter_count as u32)
            .min(100);
        let popup = Rect::new(rect.x, input_rect.bottom() + 1, rect.w, popup_h);
        ctx.fill_rect(popup, style.background.unwrap_or(Rgb565::new(4, 6, 8)))?;
        ctx.stroke_rect(popup, Border::one(style.border.color))?;

        for i in 0..filter_count as usize {
            if let Some(s) = filtered[i] {
                let row_y = popup.y + i as i32 * (row_h as i32 + 2);
                let row_rect = Rect::new(popup.x + 1, row_y + 1, popup.w.saturating_sub(2), row_h);

                if selected == Some(i) {
                    ctx.fill_rect(row_rect, style.accent)?;
                    ctx.draw_text_in(
                        Rect::new(row_rect.x + 2, row_rect.y, row_rect.w - 2, row_rect.h),
                        s,
                        TextStyle::new(style.foreground).with_font(style.font),
                    )?;
                } else {
                    ctx.draw_text_in(
                        Rect::new(row_rect.x + 2, row_rect.y, row_rect.w - 2, row_rect.h),
                        s,
                        TextStyle::new(style.text).with_font(style.font),
                    )?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "rich-widgets")]
fn render_value_label<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    label: &str,
    value: i32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn render_icon_button<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    icon: char,
    label: &str,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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
#[cfg(feature = "rich-widgets")]
fn render_list<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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

#[allow(clippy::too_many_arguments)]
fn render_circular_list<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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

    let center_y = inner.y + (inner.h as i32) / 2;
    let half_h = (inner.h as f32 / 2.0).max(1.0);
    let max_shift = (inner.w as f32 * 0.25).max(8.0);

    for row_idx in 0..rows {
        let item_idx = offset.saturating_add(row_idx);
        if item_idx >= items.len() {
            break;
        }

        let item_center_y = inner.y + (row_idx as u32 * row_h + row_h / 2) as i32;
        let dy = (item_center_y - center_y) as f32;

        let normalized_dist = dy / half_h;
        let x_shift = (normalized_dist * normalized_dist * max_shift) as i32;

        let row = Rect::new(
            inner.x + x_shift,
            inner.y + (row_idx as u32 * row_h) as i32,
            inner.w.saturating_sub(x_shift as u32),
            row_h,
        );

        if item_idx == selected {
            ctx.fill_rect(row, style.accent)?;
        }

        ctx.draw_text_in(
            row.inset(crate::geometry::EdgeInsets::symmetric(2, 4)),
            items[item_idx],
            TextStyle {
                color: if item_idx == selected {
                    style.background.unwrap_or(style.text)
                } else {
                    style.text
                },
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

#[cfg(feature = "rich-widgets")]
fn render_scroll_view<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    offset_y: i32,
    content_h: u32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn render_tabs<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    labels: &[&str],
    selected: usize,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn render_dialog<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    title: &str,
    body: &str,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn render_toast<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    text: &str,
    ttl_ms: u32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn render_meter<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    value: f32,
    min: f32,
    max: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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
#[cfg(feature = "rich-widgets")]
fn render_arc_gauge<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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

#[allow(clippy::too_many_arguments)]
fn render_sweeping_arc<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    progress: f32,
    arc_radius: u32,
    frame_inset: u16,
    corner_radius: u8,
    bg_color: Rgb565,
    arc_color: Rgb565,
    frame_color: Rgb565,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    // Solid background behind the sweep.
    ctx.fill_rect(rect, bg_color)?;
    // Sweeping pie-sector, growing clockwise from 12 o'clock.
    let cx = rect.x + rect.w as i32 / 2;
    let cy = rect.y + rect.h as i32 / 2;
    let sweep = progress.clamp(0.0, 1.0) * 360.0;
    ctx.fill_sector_sweep(cx, cy, arc_radius, -90.0, sweep, arc_color)?;
    // Rounded-rect "window" punched in the middle for the caller's value.
    let inset = frame_inset as i32;
    let fw = (rect.w as i32 - 2 * inset).max(0) as u32;
    let fh = (rect.h as i32 - 2 * inset).max(0) as u32;
    let frame = Rect::new(rect.x + inset, rect.y + inset, fw, fh);
    ctx.fill_rounded_rect(frame, corner_radius, frame_color)?;
    ctx.stroke_rounded_rect(frame, corner_radius, Border::one(frame_color))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_gauge<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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
#[cfg(feature = "rich-widgets")]
fn render_gauge_needle<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_chart<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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
            Rect::new(
                inner.x + 1,
                inner.y,
                inner.w.saturating_sub(2),
                style.font.line_height(),
            ),
            max_label.as_str(),
            TextStyle::new(style.text).with_font(style.font),
        )?;
        ctx.draw_text_in(
            Rect::new(
                inner.x + 1,
                inner
                    .bottom()
                    .saturating_sub(style.font.line_height() as i32),
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

#[allow(clippy::too_many_arguments)]
fn render_plotter<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    values: &[f32],
    head: usize,
    min: f32,
    max: f32,
    thickness: u8,
    show_grid: bool,
    show_axes: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

    let range = (max - min).max(f32::EPSILON);
    let dx = (inner.w.saturating_sub(1) as f32) / (values.len().saturating_sub(1) as f32);

    for i in 1..values.len() {
        let idx0 = (head + i - 1) % values.len();
        let idx1 = (head + i) % values.len();

        let v0 = ((values[idx0] - min) / range).clamp(0.0, 1.0);
        let v1 = ((values[idx1] - min) / range).clamp(0.0, 1.0);

        let x0 = inner.x + ((i - 1) as f32 * dx) as i32;
        let x1 = inner.x + (i as f32 * dx) as i32;

        let y0 = inner.bottom() - 1 - (v0 * (inner.h.saturating_sub(1)) as f32) as i32;
        let y1 = inner.bottom() - 1 - (v1 * (inner.h.saturating_sub(1)) as f32) as i32;

        ctx.draw_line_styled(
            x0,
            y0,
            x1,
            y1,
            StrokeStyle::new(style.accent)
                .with_width(thickness.max(1))
                .with_antialias(true),
        )?;
    }

    Ok(())
}

fn render_spinner<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    phase: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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
        StrokeStyle::new(style.accent)
            .with_width(2)
            .with_antialias(true),
    )
}

#[cfg(feature = "rich-widgets")]
fn render_dropdown<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    items: &[&str],
    selected: usize,
    open: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let text = items.get(selected).copied().unwrap_or("-");
    ctx.draw_text_in(
        Rect::new(inner.x, inner.y, inner.w.saturating_sub(8), inner.h),
        text,
        TextStyle::new(style.text).with_font(style.font),
    )?;
    ctx.draw_text_in(
        Rect::new(inner.right() - 7, inner.y, 7, inner.h),
        if open { "^" } else { "v" },
        TextStyle::new(style.accent)
            .with_font(style.font)
            .centered(),
    )?;
    if open {
        let row_h = style.font.line_height().max(6);
        let popup_h = (row_h.saturating_mul(items.len() as u32))
            .min(40)
            .max(row_h);
        let popup = Rect::new(inner.x, inner.bottom() + 1, inner.w, popup_h);
        ctx.fill_rect(popup, style.background.unwrap_or(Rgb565::new(8, 12, 16)))?;
        ctx.stroke_rect(popup, Border::one(style.border.color))?;
        let visible = (popup_h / row_h).max(1) as usize;
        let start = selected
            .saturating_sub(visible / 2)
            .min(items.len().saturating_sub(visible));
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

#[cfg(feature = "rich-widgets")]
fn render_roller<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    items: &[&str],
    selected: usize,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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
        let row = Rect::new(
            inner.x,
            inner.y + (idx as u32 * row_h) as i32,
            inner.w,
            row_h,
        );
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

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_table<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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
#[cfg(feature = "rich-widgets")]
fn draw_arc_ticks<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
fn draw_gauge_value_label<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    inner: Rect,
    value: f32,
    min: f32,
    max: f32,
    style: Style,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_textarea<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    text: &str,
    cursor: usize,
    placeholder: &str,
    selection: Option<(usize, usize)>,
    cursor_visible: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect).inset(EdgeInsets::all(1));
    let max_chars = (inner.w / style.font.advance()).max(1) as usize;
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
    if !text.is_empty() {
        if let Some((start, end)) = selection {
            let start = start.min(end).min(text.chars().count());
            let end = end.max(start).min(text.chars().count());
            for idx in start..end {
                let (col, row) = textarea_grid_position(text, idx, max_chars);
                let sel_rect = Rect::new(
                    inner.x + (col as u32 * style.font.advance()) as i32,
                    inner.y + (row as u32 * style.font.line_height()) as i32,
                    style.font.advance(),
                    style.font.line_height().min(inner.h),
                );
                ctx.fill_rect(sel_rect, style.accent)?;
            }
        }
    }
    ctx.draw_text_in(
        inner,
        shown,
        TextStyle::new(color)
            .with_font(style.font)
            .with_wrap(TextWrap::Character),
    )?;
    let chars = text.chars().count();
    let cursor = cursor.min(chars);
    if state == VisualState::Focused && cursor_visible {
        let (col, row) = textarea_grid_position(text, cursor, max_chars);
        let x = inner.x + (col as u32 * style.font.advance()) as i32;
        let y = inner.y + (row as u32 * style.font.line_height()) as i32;
        let caret = Rect::new(x, y, 1, style.font.line_height().min(inner.h));
        ctx.fill_rect(caret, style.accent)?;
    }
    Ok(())
}

#[cfg(feature = "rich-widgets")]
fn textarea_grid_position(text: &str, cursor: usize, max_chars: usize) -> (usize, usize) {
    let mut row = 0usize;
    let mut col = 0usize;
    for ch in text.chars().take(cursor) {
        if ch == '\n' {
            row += 1;
            col = 0;
            continue;
        }
        col += 1;
        if col >= max_chars {
            row += 1;
            col = 0;
        }
    }
    (col, row)
}

#[cfg(feature = "rich-widgets")]
fn textarea_text(buf: &[u8; TEXTAREA_CAPACITY], len: u8) -> &str {
    let used = (len as usize).min(TEXTAREA_CAPACITY);
    core::str::from_utf8(&buf[..used]).unwrap_or("")
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_keyboard<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
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
    C: Compositor<D>,
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

#[cfg(feature = "rich-widgets")]
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

#[cfg(feature = "rich-widgets")]
fn render_menu<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    items: &[&str],
    selected: usize,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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

fn render_image<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    image: ImageRef<'_>,
    fit: ImageFit,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    ctx.draw_image(block.inner(rect), image, fit)
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_peek_reveal<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    icon: ImageRef<'_>,
    title: &str,
    subtitle: &str,
    progress: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    let t = progress.clamp(0.0, 1.0);
    let icon_size = ((inner.h.min(inner.w / 3) as f32) * (0.2 + 0.8 * t))
        .max(2.0)
        .round() as u32;
    let icon_rect = Rect::new(inner.x + 1, inner.y + 1, icon_size, icon_size);
    ctx.draw_image(icon_rect, icon, ImageFit::Stretch)?;
    if t > 0.25 {
        ctx.draw_text_in(
            Rect::new(
                inner.x + icon_size as i32 + 2,
                inner.y,
                inner.w.saturating_sub(icon_size + 2),
                inner.h / 2,
            ),
            title,
            TextStyle::new(style.text).with_font(style.font),
        )?;
    }
    if t > 0.5 {
        ctx.draw_text_in(
            Rect::new(
                inner.x + icon_size as i32 + 2,
                inner.y + (inner.h / 2) as i32,
                inner.w.saturating_sub(icon_size + 2),
                inner.h / 2,
            ),
            subtitle,
            TextStyle::new(style.accent).with_font(style.font),
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_glance_tile<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    icon: char,
    title: &str,
    subtitle: &str,
    highlighted: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    if highlighted {
        ctx.fill_rect(Rect::new(inner.x, inner.y, inner.w, 2), style.accent)?;
    }
    let mut icon_buf = [0u8; 4];
    let icon_str = icon.encode_utf8(&mut icon_buf);
    ctx.draw_text_in(
        Rect::new(inner.x, inner.y, 10, inner.h),
        icon_str,
        TextStyle::new(style.accent)
            .with_font(style.font)
            .centered(),
    )?;
    ctx.draw_text_in(
        Rect::new(
            inner.x + 12,
            inner.y,
            inner.w.saturating_sub(12),
            inner.h / 2,
        ),
        title,
        TextStyle::new(style.text).with_font(style.font),
    )?;
    ctx.draw_text_in(
        Rect::new(
            inner.x + 12,
            inner.y + (inner.h / 2) as i32,
            inner.w.saturating_sub(12),
            inner.h / 2,
        ),
        subtitle,
        TextStyle::new(style.accent).with_font(style.font),
    )?;
    Ok(())
}

#[cfg(feature = "rich-widgets")]
fn render_card_deck<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    titles: &[&str],
    selected: usize,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    let inner = block.inner(rect);
    if titles.is_empty() {
        return Ok(());
    }
    let active = titles[selected.min(titles.len() - 1)];
    ctx.draw_text_in(
        inner,
        active,
        TextStyle::new(style.text).with_font(style.font).centered(),
    )?;
    Ok(())
}

#[cfg(feature = "rich-widgets")]
fn render_reel<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    player: ReelPlayer<'_>,
    fit: ImageFit,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    if let Some(src) = player.current_sprite_rect() {
        let inner = block.inner(rect);
        let frame_index = (src.x / player.sheet.sprite_w.max(1) as i32) as u8
            + ((src.y / player.sheet.sprite_h.max(1) as i32) as u8) * 2;
        let accent = match frame_index & 0x03 {
            0 => Rgb565::new(0, 40, 31),
            1 => Rgb565::new(31, 20, 0),
            2 => Rgb565::new(20, 0, 31),
            _ => Rgb565::new(31, 40, 0),
        };
        ctx.stroke_rect(inner, Border::one(accent))?;
        let w = inner.w.saturating_sub(4);
        let h = inner.h.saturating_sub(4);
        let bar_w = (w / 4).max(1);
        for i in 0..4u32 {
            let x = inner.x + 2 + (i * bar_w) as i32;
            let bar = Rect::new(x, inner.y + 2, bar_w.saturating_sub(1), h);
            let active = i as u8 <= (frame_index & 0x03);
            ctx.fill_rect(bar, if active { accent } else { Rgb565::new(4, 6, 6) })?;
        }
        if matches!(fit, ImageFit::Stretch | ImageFit::Center) {
            // Keep fit consumed so API remains stable while reel internals stay lightweight.
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_state_surface<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    surface: SurfaceState,
    title: &str,
    message: &str,
    action: Option<&str>,
    busy_phase: f32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    let style = style.resolve(state);
    let block = Block::styled(style)
        .title(title)
        .title_align(TextAlign::Center);
    block.render(rect, ctx)?;
    let inner = block.content_area(rect);

    let badge = match surface {
        SurfaceState::Ready => "READY",
        SurfaceState::Loading => "LOADING",
        SurfaceState::Empty => "EMPTY",
        SurfaceState::Error => "ERROR",
        SurfaceState::Offline => "OFFLINE",
    };
    ctx.draw_text_in(
        Rect::new(inner.x, inner.y, inner.w, style.font.line_height()),
        badge,
        TextStyle::new(style.accent)
            .with_font(style.font)
            .centered(),
    )?;

    if matches!(surface, SurfaceState::Loading) {
        let y = inner.y + style.font.line_height() as i32 + 3;
        let w = inner.w.saturating_sub(10);
        let x = inner.x + 5;
        ctx.stroke_rect(Rect::new(x, y, w, 5), Border::one(style.border.color))?;
        let t = busy_phase.fract().abs();
        let pulse = ((w as f32 * 0.2) as u32).max(2);
        let offset = ((w.saturating_sub(pulse) as f32) * t) as i32;
        ctx.fill_rect(Rect::new(x + offset, y + 1, pulse, 3), style.accent)?;
    }

    ctx.draw_text_in(
        Rect::new(
            inner.x + 2,
            inner.y + style.font.line_height() as i32 + 10,
            inner.w.saturating_sub(4),
            inner.h.saturating_sub(style.font.line_height() + 20),
        ),
        message,
        TextStyle::new(style.text)
            .with_font(style.font)
            .with_align(TextAlign::Center)
            .with_wrap(TextWrap::Character),
    )?;

    if let Some(action_label) = action {
        let action_h = style.font.line_height() + 3;
        let action_rect = Rect::new(
            inner.x + 4,
            inner.bottom() - action_h as i32 - 2,
            inner.w.saturating_sub(8),
            action_h,
        );
        ctx.stroke_rect(action_rect, Border::one(style.accent))?;
        ctx.draw_text_in(
            action_rect,
            action_label,
            TextStyle::new(style.accent)
                .with_font(style.font)
                .with_align(TextAlign::Center),
        )?;
    }

    Ok(())
}

#[cfg(feature = "rich-widgets")]
fn render_heads_up_banner<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    level: NotificationLevel,
    text: &str,
    ttl_ms: u32,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    if ttl_ms == 0 {
        return Ok(());
    }
    let mut style = style.resolve(state);
    style.accent = match level {
        NotificationLevel::Info => Rgb565::new(0, 32, 31),
        NotificationLevel::Success => Rgb565::new(0, 50, 0),
        NotificationLevel::Warning => Rgb565::new(31, 40, 0),
        NotificationLevel::Error => Rgb565::new(31, 0, 0),
    };
    let block = Block::styled(style);
    block.render(rect, ctx)?;
    ctx.draw_text_in(
        block.inner(rect),
        text,
        TextStyle::new(style.text)
            .with_font(style.font)
            .with_align(TextAlign::Center),
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_notification_action_sheet<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    level: NotificationLevel,
    title: &str,
    body: &str,
    actions: &[&str],
    selected: usize,
    open: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    if !open {
        return Ok(());
    }
    let mut style = style.resolve(state);
    style.accent = match level {
        NotificationLevel::Info => Rgb565::new(0, 32, 31),
        NotificationLevel::Success => Rgb565::new(0, 50, 0),
        NotificationLevel::Warning => Rgb565::new(31, 40, 0),
        NotificationLevel::Error => Rgb565::new(31, 0, 0),
    };
    let block = Block::styled(style)
        .title(title)
        .title_align(TextAlign::Center);
    block.render(rect, ctx)?;
    let inner = block.content_area(rect);
    let body_h = inner.h.saturating_sub(style.font.line_height() + 12);
    ctx.draw_text_in(
        Rect::new(inner.x + 2, inner.y + 2, inner.w.saturating_sub(4), body_h),
        body,
        TextStyle::new(style.text)
            .with_font(style.font)
            .with_wrap(TextWrap::Character),
    )?;
    if actions.is_empty() {
        return Ok(());
    }
    let action_h = style.font.line_height() + 2;
    let y = inner.bottom() - action_h as i32 - 2;
    let action_w = (inner.w / actions.len() as u32).max(1);
    for (i, action) in actions.iter().enumerate() {
        let cell = Rect::new(
            inner.x + (i as u32 * action_w) as i32,
            y,
            action_w,
            action_h,
        );
        if i == selected.min(actions.len() - 1) {
            ctx.fill_rect(cell, style.accent)?;
        } else {
            ctx.stroke_rect(cell, Border::one(style.border.color))?;
        }
        ctx.draw_text_in(
            cell,
            action,
            TextStyle::new(style.text)
                .with_font(style.font)
                .with_align(TextAlign::Center),
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "rich-widgets")]
fn render_feed_timeline<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    items: &[&str],
    selected: usize,
    offset: usize,
    visible_rows: usize,
    expanded: bool,
    style: WidgetStyle,
    state: VisualState,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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
        let is_selected = item_idx == selected;
        if is_selected {
            ctx.fill_rect(row, style.accent)?;
        }
        ctx.draw_text_in(
            row.inset(EdgeInsets::symmetric(2, 1)),
            items[item_idx],
            TextStyle::new(style.text)
                .with_font(style.font)
                .with_wrap(TextWrap::Character),
        )?;
        if expanded && is_selected && row_h > style.font.line_height() + 4 {
            let detail = Rect::new(
                row.x + 2,
                row.y + style.font.line_height() as i32,
                row.w.saturating_sub(4),
                row.h.saturating_sub(style.font.line_height()),
            );
            ctx.draw_text_in(
                detail,
                "details...",
                TextStyle::new(style.text).with_font(style.font),
            )?;
        }
    }
    Ok(())
}

#[cfg(feature = "rich-widgets")]
fn draw_i32_right<D, C>(
    ctx: &mut RenderCtx<'_, D, C>,
    rect: Rect,
    value: i32,
    color: Rgb565,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
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
