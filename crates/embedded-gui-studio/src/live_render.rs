//! Host-side rendering of a [`ScreenDef`] into an RGB565 framebuffer using the
//! real `embedded-gui` [`GuiContext`], so what streams to the board matches
//! silicon pixels rather than the egui preview approximation.

use core::f32::consts::TAU;

use eframe::egui::Color32;
use embedded_graphics_core::Pixel;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Point, Size};
use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_gui::prelude::*;
use embedded_gui::{
    BusyWheel, ContentIndicatorDirection, ContentIndicatorWidget, CrumbsIndicatorWidget,
    NumberPickerWidget, PathVerb, PixelRead, ScaleMode, StatusBarWidget, StrokeStyle,
    TimePickerWidget, VectorPath,
};
use embedded_gui_codegen::{PathVerbDef, ScreenDef, WidgetDef};

use crate::layout::compute_track_sizes;
use crate::theme::ThemePalette;
use crate::types::DisplayTheme;

/// Fixed capacities for the throwaway render context. Screens in Studio are far
/// smaller than these bounds.
const NODES: usize = 256;
const EVENTS: usize = 64;
const DIRTY: usize = 256;
const PLOT_SAMPLES: usize = 64;
const MAX_PATH_VERBS: usize = 128;

/// A rendered screen as a flat row-major RGB565 buffer.
pub struct RenderedFrame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<Rgb565>,
}

impl RenderedFrame {
    /// Reads a pixel, returning black outside bounds.
    #[inline]
    fn at(&self, x: u32, y: u32) -> Rgb565 {
        if x < self.width as u32 && y < self.height as u32 {
            self.pixels[(y * self.width as u32 + x) as usize]
        } else {
            Rgb565::BLACK
        }
    }
}

impl RenderedFrame {
    /// Centers this frame inside a `width` x `height` panel, cropping whatever
    /// overflows. The board can only address pixels it physically has, so a
    /// screen larger than the panel must be fitted here rather than sent with
    /// out-of-range rectangles the agent would silently drop.
    pub fn fit_to(&self, width: u16, height: u16, background: Rgb565) -> RenderedFrame {
        if self.width == width && self.height == height {
            return RenderedFrame {
                width,
                height,
                pixels: self.pixels.clone(),
            };
        }

        let mut pixels = vec![background; width as usize * height as usize];
        let copy_w = self.width.min(width) as i32;
        let copy_h = self.height.min(height) as i32;
        let src_x0 = (self.width as i32 - copy_w) / 2;
        let src_y0 = (self.height as i32 - copy_h) / 2;
        let dst_x0 = (width as i32 - copy_w) / 2;
        let dst_y0 = (height as i32 - copy_h) / 2;

        for row in 0..copy_h {
            let src = ((src_y0 + row) * self.width as i32 + src_x0) as usize;
            let dst = ((dst_y0 + row) * width as i32 + dst_x0) as usize;
            let len = copy_w as usize;
            pixels[dst..dst + len].copy_from_slice(&self.pixels[src..src + len]);
        }

        RenderedFrame {
            width,
            height,
            pixels,
        }
    }

    /// Background color used when letterboxing a screen onto a larger panel.
    pub fn background_for(theme: DisplayTheme) -> Rgb565 {
        Palette565::for_theme(theme).display_bg
    }
}

/// A minimal `DrawTarget` writing into a borrowed RGB565 slice.
struct BufferTarget<'p> {
    buf: &'p mut [Rgb565],
    w: u32,
    h: u32,
}

impl OriginDimensions for BufferTarget<'_> {
    fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }
}

impl DrawTarget for BufferTarget<'_> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(p, color) in pixels {
            if p.x >= 0 && p.y >= 0 && (p.x as u32) < self.w && (p.y as u32) < self.h {
                self.buf[(p.y as u32 * self.w + p.x as u32) as usize] = color;
            }
        }
        Ok(())
    }
}

impl PixelRead for BufferTarget<'_> {
    fn get_pixel(&self, point: Point) -> Self::Color {
        if point.x >= 0 && point.y >= 0 && (point.x as u32) < self.w && (point.y as u32) < self.h {
            self.buf[(point.y as u32 * self.w + point.x as u32) as usize]
        } else {
            Rgb565::BLACK
        }
    }
}

/// RGB565 mirror of the Studio canvas palette, so the pixels streamed to the
/// board carry the same semantic colors the preview shows.
struct Palette565 {
    display_bg: Rgb565,
    card_bg: Rgb565,
    border: Rgb565,
    text_primary: Rgb565,
    text_dim: Rgb565,
    accent: Rgb565,
    success: Rgb565,
    danger: Rgb565,
}

/// Truncates 8-bit channels down to the 5/6/5 bit depth of the panel.
fn to_rgb565(c: Color32) -> Rgb565 {
    Rgb565::new(c.r() >> 3, c.g() >> 2, c.b() >> 3)
}

fn parse_hex_color(hex: &str, fallback: Rgb565) -> Rgb565 {
    let h = hex.trim().trim_start_matches('#');
    if h.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&h[0..2], 16),
            u8::from_str_radix(&h[2..4], 16),
            u8::from_str_radix(&h[4..6], 16),
        ) {
            return Rgb565::new(r >> 3, g >> 2, b >> 3);
        }
    }
    fallback
}

impl Palette565 {
    fn for_theme(theme: DisplayTheme) -> Self {
        let p = ThemePalette::for_theme(theme);
        Self {
            display_bg: to_rgb565(p.display_bg),
            card_bg: to_rgb565(p.card_bg),
            border: to_rgb565(p.border),
            text_primary: to_rgb565(p.text_primary),
            text_dim: to_rgb565(p.text_dim),
            accent: to_rgb565(p.accent),
            success: to_rgb565(p.success),
            danger: to_rgb565(p.danger),
        }
    }

    fn token_color(&self, token: Option<&str>) -> Rgb565 {
        match token {
            Some("accent") => self.accent,
            Some("success") => self.success,
            Some("danger") => self.danger,
            Some("dim") => self.text_dim,
            _ => self.text_primary,
        }
    }

    fn label(&self, token: Option<&str>) -> Style {
        let mut s = Style::label();
        s.corner_radius = 3;
        s.accent = self.accent;
        match token {
            Some("inverted") | Some("xor") => {
                s.background = Some(self.text_primary);
                s.text = self.display_bg;
                s.foreground = self.display_bg;
                s.corner_radius = 2;
            }
            other => {
                let c = self.token_color(other);
                s.background = Some(self.card_bg);
                s.text = c;
                s.foreground = c;
                if other == Some("bold") {
                    s.font = FontId::Medium4x7;
                }
            }
        }
        s
    }

    fn button(&self, token: Option<&str>) -> Style {
        let (bg, fg) = match token {
            Some("accent") => (self.accent, self.text_primary),
            Some("danger") => (self.danger, self.text_primary),
            Some("inverted") => (self.text_primary, self.display_bg),
            _ => (self.card_bg, self.text_primary),
        };
        let mut s = Style::button();
        s.gradient = None;
        s.background = Some(bg);
        s.text = fg;
        s.foreground = fg;
        s.accent = self.accent;
        s.border = Border::one(self.border);
        s
    }

    fn panel(&self, token: Option<&str>) -> Style {
        let bg = if token == Some("card") {
            self.card_bg
        } else {
            self.display_bg
        };
        let mut s = Style::panel();
        s.gradient = None;
        s.background = Some(bg);
        s.text = self.text_primary;
        s.foreground = self.text_primary;
        s.accent = self.accent;
        s.border = Border::one(self.border);
        s
    }

    fn toggle(&self, checked: bool) -> Style {
        let on = if checked { self.success } else { self.text_dim };
        let mut s = self.button(None);
        s.foreground = on;
        s.accent = on;
        s
    }

    fn bar(&self, fill: Rgb565) -> Style {
        let mut s = Style::progress();
        s.gradient = None;
        s.background = Some(self.card_bg);
        s.foreground = fill;
        s.accent = fill;
        s.text = self.text_primary;
        s.border = Border::one(self.border);
        s
    }
}

struct Cell {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

/// Grid track sizes and start offsets, in device pixels.
///
/// The canvas overlays derive their positions from this so the editor's
/// dividers and handles land on the same boundaries the rendered pixels use.
/// Track sizing is not uniformly scalable — `px` and `auto` tracks are absolute
/// — so overlays must scale this result rather than recompute it at zoom.
pub struct GridGeometry {
    pub col_sizes: Vec<f32>,
    pub row_sizes: Vec<f32>,
    pub col_starts: Vec<f32>,
    pub row_starts: Vec<f32>,
}

pub fn grid_geometry(screen: &ScreenDef) -> GridGeometry {
    let gap = screen.grid.gap as f32;
    let pad = screen.grid.padding as f32;

    let avail_w = (screen.width as f32 - 2.0 * pad).max(0.0);
    let avail_h = (screen.height as f32 - 2.0 * pad).max(0.0);

    let col_sizes = compute_track_sizes(&screen.grid.cols, avail_w, gap);
    let row_sizes = compute_track_sizes(&screen.grid.rows, avail_h, gap);

    let col_starts = track_starts(&col_sizes, pad, gap);
    let row_starts = track_starts(&row_sizes, pad, gap);

    GridGeometry {
        col_sizes,
        row_sizes,
        col_starts,
        row_starts,
    }
}

fn compute_cells(screen: &ScreenDef) -> Vec<Cell> {
    let gap = screen.grid.gap as f32;
    let pad = screen.grid.padding as f32;
    let GridGeometry {
        col_sizes,
        row_sizes,
        col_starts,
        row_starts,
    } = grid_geometry(screen);

    let mut cells = Vec::with_capacity(screen.grid.children.len());
    for (placement, _) in &screen.grid.children {
        let col = placement.col.min(col_sizes.len().saturating_sub(1));
        let row = placement.row.min(row_sizes.len().saturating_sub(1));
        let col_end = (placement.col + placement.col_span).min(col_sizes.len());
        let row_end = (placement.row + placement.row_span).min(row_sizes.len());

        let x = col_starts.get(col).copied().unwrap_or(pad);
        let y = row_starts.get(row).copied().unwrap_or(pad);

        let span_cols = placement.col_span.max(1).saturating_sub(1) as f32;
        let span_rows = placement.row_span.max(1).saturating_sub(1) as f32;
        let w: f32 = col_sizes[col..col_end.max(col + 1).min(col_sizes.len())]
            .iter()
            .sum::<f32>()
            + gap * span_cols;
        let h: f32 = row_sizes[row..row_end.max(row + 1).min(row_sizes.len())]
            .iter()
            .sum::<f32>()
            + gap * span_rows;

        cells.push(Cell {
            x: x.round() as i32,
            y: y.round() as i32,
            w: w.round().max(1.0) as u32,
            h: h.round().max(1.0) as u32,
        });
    }
    cells
}

fn track_starts(sizes: &[f32], pad: f32, gap: f32) -> Vec<f32> {
    let mut starts = Vec::with_capacity(sizes.len());
    let mut cursor = pad;
    for (i, s) in sizes.iter().enumerate() {
        if i > 0 {
            cursor += gap;
        }
        starts.push(cursor);
        cursor += *s;
    }
    starts
}

fn parse_scale_mode(mode: &str) -> ScaleMode {
    if mode.eq_ignore_ascii_case("radial") {
        ScaleMode::Radial
    } else if mode.eq_ignore_ascii_case("linear_vertical") {
        ScaleMode::LinearVertical
    } else {
        ScaleMode::LinearHorizontal
    }
}

fn demo_plot_samples(mode: &str) -> Vec<f32> {
    let mut samples = Vec::with_capacity(PLOT_SAMPLES);
    for i in 0..PLOT_SAMPLES {
        let t = i as f32 / (PLOT_SAMPLES as f32 - 1.0).max(1.0);
        let v = if mode.eq_ignore_ascii_case("bar") {
            ((t * 4.0).sin().abs() * 0.7 + 0.15).clamp(0.0, 1.0)
        } else {
            (0.5 + 0.45 * (t * TAU * 2.0).sin()).clamp(0.0, 1.0)
        };
        samples.push(v);
    }
    samples
}

fn sweeping_progress(start_angle: i16, end_angle: i16) -> f32 {
    let span = (i32::from(end_angle) - i32::from(start_angle)).rem_euclid(360) as f32;
    (span / 360.0).clamp(0.05, 1.0)
}

/// Widgets painted through [`RenderCtx`] after the GuiContext pass.
enum Overlay<'a> {
    BusyWheel {
        rect: Rect,
        active: bool,
        phase: f32,
    },
    StatusBar {
        rect: Rect,
        time: &'a str,
    },
    TimePicker {
        rect: Rect,
        hour: u8,
        minute: u8,
        is_12h: bool,
        is_pm: bool,
    },
    NumberPicker {
        rect: Rect,
        min: i32,
        max: i32,
        value: i32,
        unit: &'a str,
    },
    ContentIndicator {
        rect: Rect,
    },
    CrumbsIndicator {
        rect: Rect,
        count: u8,
        active: u8,
    },
    VectorPath {
        rect: Rect,
        stroke_width: u8,
        verbs: &'a [PathVerbDef],
    },
    RectShape {
        rect: Rect,
        radius: u8,
        stroke_width: u8,
        fill_color: Option<&'a str>,
        stroke_color: Option<&'a str>,
    },
    LineShape {
        rect: Rect,
        stroke_width: u8,
        color: Option<&'a str>,
    },
    CircleShape {
        rect: Rect,
        radius: u16,
        stroke_width: u8,
        fill_color: Option<&'a str>,
        stroke_color: Option<&'a str>,
    },
}

/// Returns true when rendering this screen at a new timeline phase can change
/// pixels without an edit to the KDL document.
pub fn has_animated_content(screen: &ScreenDef) -> bool {
    screen.grid.children.iter().any(|(_, widget)| {
        matches!(
            widget,
            WidgetDef::BusyWheel { active: true, .. } | WidgetDef::Plotter { .. }
        )
    })
}

/// Renders a static snapshot. Kept for callers and tests that don't have a
/// playback clock.
#[cfg(test)]
pub fn render_screen(screen: &ScreenDef, theme: DisplayTheme) -> RenderedFrame {
    render_screen_at(screen, theme, 0.0, None)
}

/// Renders `screen` at a normalized animation phase in `0.0..1.0`.
///
/// `highlight` is the index of a widget receiving transient press feedback in
/// Live Interactive; it draws an accent ring on that cell so a tap is visible
/// both on the canvas and on the streamed panel.
pub fn render_screen_at(
    screen: &ScreenDef,
    theme: DisplayTheme,
    animation_phase: f32,
    highlight: Option<usize>,
) -> RenderedFrame {
    let mut option_lists: Vec<Vec<&str>> = Vec::new();
    let mut table_storage: Vec<Vec<Vec<&str>>> = Vec::new();
    let mut plot_samples: Vec<Vec<f32>> = Vec::new();

    for (_, widget) in &screen.grid.children {
        match widget {
            WidgetDef::Dropdown { options, .. } | WidgetDef::Roller { options, .. } => {
                option_lists.push(options.iter().map(String::as_str).collect());
            }
            WidgetDef::Table { headers, rows, .. } => {
                let mut grid = Vec::new();
                if let Some(headers) = headers {
                    grid.push(headers.iter().map(String::as_str).collect());
                }
                for row in rows {
                    grid.push(row.iter().map(String::as_str).collect());
                }
                table_storage.push(grid);
            }
            WidgetDef::Plotter { mode, .. } => {
                plot_samples.push(demo_plot_samples(mode));
            }
            _ => {}
        }
    }

    let table_rows: Vec<Vec<&[&str]>> = table_storage
        .iter()
        .map(|table| table.iter().map(Vec::as_slice).collect())
        .collect();

    render_inner(
        screen,
        theme,
        animation_phase.rem_euclid(1.0),
        &option_lists,
        &table_rows,
        &plot_samples,
        highlight,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_inner<'a>(
    screen: &'a ScreenDef,
    theme: DisplayTheme,
    animation_phase: f32,
    option_lists: &'a [Vec<&'a str>],
    table_rows: &'a [Vec<&'a [&'a str]>],
    plot_samples: &'a [Vec<f32>],
    highlight: Option<usize>,
) -> RenderedFrame {
    let width = screen.width.max(1) as u16;
    let height = screen.height.max(1) as u16;
    let palette = Palette565::for_theme(theme);
    let mut pixels = vec![palette.display_bg; width as usize * height as usize];
    let cells = compute_cells(screen);

    let mut gui = Box::new(GuiContext::<NODES, EVENTS, DIRTY>::new(Rect::new(
        0,
        0,
        width as u32,
        height as u32,
    )));

    let mut overlays = Vec::new();
    let mut option_idx = 0usize;
    let mut table_idx = 0usize;
    let mut plot_idx = 0usize;

    for (idx, (_, widget)) in screen.grid.children.iter().enumerate() {
        let Some(cell) = cells.get(idx) else { continue };
        let rect = Rect::new(cell.x, cell.y, cell.w, cell.h);
        add_widget(
            &mut gui,
            rect,
            widget,
            &palette,
            animation_phase,
            option_lists,
            table_rows,
            plot_samples,
            &mut option_idx,
            &mut table_idx,
            &mut plot_idx,
            &mut overlays,
        );
    }

    {
        let mut target = BufferTarget {
            buf: &mut pixels,
            w: width as u32,
            h: height as u32,
        };
        let _ = gui.render(&mut target);

        let viewport = Rect::new(0, 0, width as u32, height as u32);
        let mut ctx = RenderCtx::new(&mut target, viewport);
        for overlay in &overlays {
            paint_overlay(&mut ctx, overlay, &palette);
        }

        // Transient press feedback: a two-pixel accent ring around the touched
        // cell. Drawn last so it sits above the widget it belongs to.
        if let Some(cell) = highlight.and_then(|idx| cells.get(idx)) {
            let outer = Rect::new(cell.x, cell.y, cell.w, cell.h);
            let _ = ctx.stroke_rounded_rect(outer, 3, Border::one(palette.accent));
            if cell.w > 2 && cell.h > 2 {
                let inner = Rect::new(cell.x + 1, cell.y + 1, cell.w - 2, cell.h - 2);
                let _ = ctx.stroke_rounded_rect(inner, 3, Border::one(palette.accent));
            }
        }
    }

    RenderedFrame {
        width,
        height,
        pixels,
    }
}

#[allow(clippy::too_many_arguments)]
fn add_widget<'a>(
    gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    rect: Rect,
    widget: &'a WidgetDef,
    p: &Palette565,
    animation_phase: f32,
    option_lists: &'a [Vec<&'a str>],
    table_rows: &'a [Vec<&'a [&'a str]>],
    plot_samples: &'a [Vec<f32>],
    option_idx: &mut usize,
    table_idx: &mut usize,
    plot_idx: &mut usize,
    overlays: &mut Vec<Overlay<'a>>,
) {
    let style = p.panel(Some("card"));
    let _ = match widget {
        WidgetDef::Label {
            text, style: token, ..
        } => gui.add_label(rect, text.as_str(), p.label(token.as_deref())),
        WidgetDef::Button {
            text, style: token, ..
        } => gui.add_button(rect, text.as_str(), p.button(token.as_deref())),
        WidgetDef::Toggle { label, checked, .. } => {
            gui.add_toggle(rect, label.as_str(), *checked, p.toggle(*checked))
        }
        WidgetDef::Checkbox { label, checked, .. } => {
            gui.add_checkbox(rect, label.as_str(), *checked, p.toggle(*checked))
        }
        WidgetDef::Slider {
            min, max, value, ..
        } => gui.add_slider(
            rect,
            *min as f32,
            *max as f32,
            *value as f32,
            p.bar(p.accent),
        ),
        WidgetDef::ProgressBar { value, .. } => {
            gui.add_progress_bar(rect, value.clamp(0.0, 1.0), p.bar(p.success))
        }
        WidgetDef::Spacer => gui.add_spacer(rect),
        WidgetDef::Panel { style: token, .. } => gui.add_panel(rect, p.panel(token.as_deref())),
        WidgetDef::Dropdown { selected, .. } => {
            let items = option_lists
                .get(*option_idx)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            *option_idx += 1;
            gui.add_dropdown(rect, items, *selected, style)
        }
        WidgetDef::Roller { selected, .. } => {
            let items = option_lists
                .get(*option_idx)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            *option_idx += 1;
            gui.add_roller(rect, items, *selected, style)
        }
        WidgetDef::Scale {
            mode,
            min,
            max,
            value,
            ..
        } => gui.add_scale(rect, parse_scale_mode(mode), *min, *max, *value, style),
        WidgetDef::Spinbox {
            min, max, value, ..
        } => gui.add_spinbox(rect, *min, *max, *value, style),
        WidgetDef::Table { .. } => {
            let rows = table_rows.get(*table_idx).map(Vec::as_slice).unwrap_or(&[]);
            *table_idx += 1;
            gui.add_table(rect, rows, style)
        }
        WidgetDef::SweepingArc {
            start_angle,
            end_angle,
            ..
        } => {
            let radius = rect.w.min(rect.h) / 2;
            gui.add_sweeping_arc(
                rect,
                sweeping_progress(*start_angle, *end_angle),
                true,
                radius,
                4,
                4,
                p.card_bg,
                p.accent,
                p.border,
                style,
            )
        }
        WidgetDef::Plotter { .. } => {
            let samples = plot_samples
                .get(*plot_idx)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            *plot_idx += 1;
            let head = ((samples.len().saturating_sub(1) as f32) * animation_phase) as usize;
            gui.add_plotter(rect, samples, head, 0.0, 1.0, style)
        }
        WidgetDef::Dialog { title, message, .. } => {
            gui.add_dialog(rect, title.as_str(), message.as_str(), style)
        }
        WidgetDef::BusyWheel { active, .. } => {
            overlays.push(Overlay::BusyWheel {
                rect,
                active: *active,
                phase: animation_phase,
            });
            gui.add_spacer(rect)
        }
        WidgetDef::StatusBar { time, .. } => {
            overlays.push(Overlay::StatusBar {
                rect,
                time: time.as_str(),
            });
            gui.add_spacer(rect)
        }
        WidgetDef::TimePicker {
            hour,
            minute,
            is_12h,
            is_pm,
            ..
        } => {
            overlays.push(Overlay::TimePicker {
                rect,
                hour: *hour,
                minute: *minute,
                is_12h: *is_12h,
                is_pm: *is_pm,
            });
            gui.add_spacer(rect)
        }
        WidgetDef::NumberPicker {
            min,
            max,
            value,
            unit,
            ..
        } => {
            overlays.push(Overlay::NumberPicker {
                rect,
                min: *min,
                max: *max,
                value: *value,
                unit: unit.as_str(),
            });
            gui.add_spacer(rect)
        }
        WidgetDef::ContentIndicator { .. } => {
            overlays.push(Overlay::ContentIndicator { rect });
            gui.add_spacer(rect)
        }
        WidgetDef::CrumbsIndicator { count, active, .. } => {
            overlays.push(Overlay::CrumbsIndicator {
                rect,
                count: *count,
                active: *active,
            });
            gui.add_spacer(rect)
        }
        WidgetDef::VectorPath {
            stroke_width,
            verbs,
            ..
        } => {
            overlays.push(Overlay::VectorPath {
                rect,
                stroke_width: *stroke_width,
                verbs,
            });
            gui.add_spacer(rect)
        }
        WidgetDef::RectShape {
            radius,
            stroke_width,
            fill_color,
            stroke_color,
            ..
        } => {
            overlays.push(Overlay::RectShape {
                rect,
                radius: *radius,
                stroke_width: *stroke_width,
                fill_color: fill_color.as_deref(),
                stroke_color: stroke_color.as_deref(),
            });
            gui.add_spacer(rect)
        }
        WidgetDef::LineShape {
            stroke_width,
            color,
            ..
        } => {
            overlays.push(Overlay::LineShape {
                rect,
                stroke_width: *stroke_width,
                color: color.as_deref(),
            });
            gui.add_spacer(rect)
        }
        WidgetDef::CircleShape {
            radius,
            stroke_width,
            fill_color,
            stroke_color,
            ..
        } => {
            overlays.push(Overlay::CircleShape {
                rect,
                radius: *radius,
                stroke_width: *stroke_width,
                fill_color: fill_color.as_deref(),
                stroke_color: stroke_color.as_deref(),
            });
            gui.add_spacer(rect)
        }
    };
}

fn paint_overlay<D>(ctx: &mut RenderCtx<'_, D>, overlay: &Overlay<'_>, p: &Palette565)
where
    D: DrawTarget<Color = Rgb565> + PixelRead,
{
    match overlay {
        Overlay::BusyWheel {
            rect,
            active,
            phase,
        } => {
            let _ = ctx.fill_rounded_rect(*rect, 3, p.card_bg);
            if *active {
                let cx = rect.x + rect.w as i32 / 2;
                let cy = rect.y + rect.h as i32 / 2;
                let radius = (rect.w.min(rect.h) / 3).max(4);
                let wheel = BusyWheel {
                    phase: *phase,
                    color: p.accent,
                    ..BusyWheel::new(cx, cy, radius)
                };
                let _ = wheel.draw(ctx);
            }
        }
        Overlay::StatusBar { rect, time } => {
            let mut bar = StatusBarWidget::new(time);
            bar.background_color = p.card_bg;
            bar.foreground_color = p.text_primary;
            bar.accent_color = p.accent;
            bar.separator_color = Some(p.border);
            let _ = bar.render(ctx, *rect);
        }
        Overlay::TimePicker {
            rect,
            hour,
            minute,
            is_12h,
            is_pm,
        } => {
            let picker = if *is_12h {
                TimePickerWidget::new_12h(*hour, *minute, *is_pm)
            } else {
                TimePickerWidget::new_24h(*hour, *minute)
            };
            let _ = picker.render(ctx, *rect);
        }
        Overlay::NumberPicker {
            rect,
            min,
            max,
            value,
            unit,
        } => {
            let picker = NumberPickerWidget::new(*min, *max, *value, unit);
            let _ = picker.render(ctx, *rect);
        }
        Overlay::ContentIndicator { rect } => {
            let indicator = ContentIndicatorWidget::new(ContentIndicatorDirection::Down);
            let _ = indicator.render(ctx, *rect);
        }
        Overlay::CrumbsIndicator {
            rect,
            count,
            active,
        } => {
            let crumbs = CrumbsIndicatorWidget::new(*count, *active);
            let _ = crumbs.render(ctx, *rect);
        }
        Overlay::VectorPath {
            rect,
            stroke_width,
            verbs,
        } => {
            let mut path = VectorPath::<MAX_PATH_VERBS>::new();
            for verb in *verbs {
                let ok = match *verb {
                    PathVerbDef::MoveTo(x, y) => {
                        path.push(PathVerb::MoveTo(Point::new(rect.x + x, rect.y + y)))
                    }
                    PathVerbDef::LineTo(x, y) => {
                        path.push(PathVerb::LineTo(Point::new(rect.x + x, rect.y + y)))
                    }
                    PathVerbDef::QuadTo(cx, cy, x, y) => path.push(PathVerb::QuadTo {
                        control: Point::new(rect.x + cx, rect.y + cy),
                        to: Point::new(rect.x + x, rect.y + y),
                    }),
                    PathVerbDef::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                        path.push(PathVerb::CubicTo {
                            control1: Point::new(rect.x + c1x, rect.y + c1y),
                            control2: Point::new(rect.x + c2x, rect.y + c2y),
                            to: Point::new(rect.x + x, rect.y + y),
                        })
                    }
                    PathVerbDef::Close => path.push(PathVerb::Close),
                };
                if !ok {
                    break;
                }
            }
            let style = StrokeStyle::new(p.accent).with_width((*stroke_width).max(1));
            let _ = ctx.draw_vector_path(&path, style);
        }
        Overlay::RectShape {
            rect,
            radius,
            stroke_width,
            fill_color,
            stroke_color,
        } => {
            let fill = fill_color
                .map(|c| parse_hex_color(c, p.card_bg))
                .unwrap_or(p.card_bg);
            let stroke = stroke_color
                .map(|c| parse_hex_color(c, p.border))
                .unwrap_or(p.border);
            let _ = ctx.fill_rounded_rect(*rect, *radius, fill);
            if *stroke_width > 0 {
                let _ = ctx.stroke_rounded_rect(*rect, *radius, Border::one(stroke));
            }
        }
        Overlay::LineShape {
            rect,
            stroke_width,
            color,
        } => {
            let color = color
                .map(|c| parse_hex_color(c, p.accent))
                .unwrap_or(p.accent);
            let style = StrokeStyle::new(color).with_width((*stroke_width).max(1));
            let _ = ctx.draw_line_styled(
                rect.x,
                rect.y + rect.h as i32 / 2,
                rect.right() - 1,
                rect.y + rect.h as i32 / 2,
                style,
            );
        }
        Overlay::CircleShape {
            rect,
            radius,
            stroke_width,
            fill_color,
            stroke_color,
        } => {
            let cx = rect.x + rect.w as i32 / 2;
            let cy = rect.y + rect.h as i32 / 2;
            let r = (*radius as u32).min(rect.w.min(rect.h) / 2).max(1);
            let fill = fill_color
                .map(|c| parse_hex_color(c, p.card_bg))
                .unwrap_or(p.card_bg);
            let stroke = stroke_color
                .map(|c| parse_hex_color(c, p.border))
                .unwrap_or(p.border);
            let _ = ctx.fill_circle(cx, cy, r, fill);
            if *stroke_width > 0 {
                let ring = Rect::new(cx - r as i32, cy - r as i32, r * 2, r * 2);
                let _ = ctx.stroke_rounded_rect(ring, r.min(255) as u8, Border::one(stroke));
            }
        }
    }
}

/// Computes changed tiles between two frames of identical dimensions.
pub fn changed_tiles(
    prev: &RenderedFrame,
    next: &RenderedFrame,
    tile_w: u32,
    tile_h: u32,
) -> Vec<(u16, u16, u16, u16)> {
    let mut rects = Vec::new();
    if prev.width != next.width || prev.height != next.height {
        return rects;
    }
    let w = next.width as u32;
    let h = next.height as u32;

    let mut ty = 0;
    while ty < h {
        let th = tile_h.min(h - ty);
        let mut tx = 0;
        while tx < w {
            let tw = tile_w.min(w - tx);
            if tile_differs(prev, next, tx, ty, tw, th) {
                rects.push((tx as u16, ty as u16, tw as u16, th as u16));
            }
            tx += tile_w;
        }
        ty += tile_h;
    }
    rects
}

fn tile_differs(
    prev: &RenderedFrame,
    next: &RenderedFrame,
    tx: u32,
    ty: u32,
    tw: u32,
    th: u32,
) -> bool {
    for y in ty..ty + th {
        for x in tx..tx + tw {
            if prev.at(x, y) != next.at(x, y) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_gui_codegen::parse_kdl_screen;

    #[test]
    fn renders_a_simple_screen() {
        let kdl = r#"screen id="Test" width=320 height=240 {
            grid cols="1fr" rows="1fr 1fr" gap=4 padding=8 {
                label text="Hello" col=0 row=0
                button text="Tap" col=0 row=1
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        let frame = render_screen(&screen, DisplayTheme::DarkTft);
        assert_eq!(frame.width, 320);
        assert_eq!(frame.height, 240);
        assert!(frame.pixels.iter().any(|p| *p != Rgb565::BLACK));
    }

    #[test]
    fn playback_phase_changes_animated_pixels() {
        let kdl = r#"screen id="Motion" width=160 height=80 {
            grid cols="1fr 1fr" rows="1fr" gap=4 padding=4 {
                busy_wheel active=true col=0 row=0
                plotter mode="sine" col=1 row=0
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        assert!(has_animated_content(&screen));

        let first = render_screen_at(&screen, DisplayTheme::DarkTft, 0.1, None);
        let second = render_screen_at(&screen, DisplayTheme::DarkTft, 0.7, None);
        assert!(
            !changed_tiles(&first, &second, 40, 40).is_empty(),
            "different timeline phases must produce dirty tiles"
        );
    }

    #[test]
    fn success_label_streams_green_not_white() {
        let kdl = r#"screen id="Test" width=160 height=80 {
            grid cols="1fr" rows="1fr" gap=0 padding=0 {
                label id="status" text="READY" style="success" col=0 row=0
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        let frame = render_screen(&screen, DisplayTheme::DarkTft);

        let expected = Palette565::for_theme(DisplayTheme::DarkTft).success;
        assert!(
            frame.pixels.contains(&expected),
            "no pixels matched the palette success green"
        );
        assert!(
            !frame.pixels.contains(&Rgb565::WHITE),
            "text still rendered pure white"
        );
    }

    #[test]
    fn diff_detects_no_change_for_identical_frames() {
        let kdl = r#"screen id="Test" width=64 height=64 {
            grid cols="1fr" rows="1fr" gap=0 padding=0 {
                button text="A" col=0 row=0
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        let a = render_screen(&screen, DisplayTheme::DarkTft);
        let b = render_screen(&screen, DisplayTheme::DarkTft);
        assert!(changed_tiles(&a, &b, 32, 32).is_empty());
    }

    /// A screen larger than the panel must be centered and cropped, never sent
    /// at its own size where the agent would drop the overflow.
    #[test]
    fn oversized_screen_is_centered_onto_the_panel() {
        let kdl = r#"screen id="Big" width=480 height=272 {
            grid cols="1fr" rows="1fr" gap=0 padding=0 {
                label text="X" col=0 row=0
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        let frame = render_screen(&screen, DisplayTheme::DarkTft);
        let bg = RenderedFrame::background_for(DisplayTheme::DarkTft);
        let fitted = frame.fit_to(320, 240, bg);

        assert_eq!(fitted.width, 320);
        assert_eq!(fitted.height, 240);
        assert_eq!(fitted.pixels.len(), 320 * 240);
        // The centered crop keeps the middle of the source frame.
        assert_eq!(fitted.at(160, 120), frame.at(240, 136));
    }

    /// A screen smaller than the panel is letterboxed with the theme
    /// background rather than stretched.
    #[test]
    fn undersized_screen_is_letterboxed() {
        let kdl = r#"screen id="Small" width=128 height=64 {
            grid cols="1fr" rows="1fr" gap=0 padding=0 {
                label text="X" col=0 row=0
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        let frame = render_screen(&screen, DisplayTheme::DarkTft);
        let bg = RenderedFrame::background_for(DisplayTheme::DarkTft);
        let fitted = frame.fit_to(320, 240, bg);

        assert_eq!(fitted.width, 320);
        assert_eq!(fitted.height, 240);
        assert_eq!(fitted.at(2, 2), bg, "corner should be letterbox background");
        assert_eq!(fitted.at(160, 120), frame.at(64, 32));
    }

    /// Overlays scale this geometry rather than recomputing it at zoom, because
    /// `px` tracks keep their absolute size while `fr` tracks absorb the rest.
    /// Recomputing against a zoomed viewport would shift every boundary.
    #[test]
    fn px_tracks_do_not_scale_with_the_viewport() {
        let kdl = r#"screen id="Mixed" width=320 height=240 {
            grid cols="100px 1fr" rows="1fr" gap=8 padding=10 {
                label text="A" col=0 row=0
                label text="B" col=1 row=0
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        let geometry = grid_geometry(&screen);

        assert_eq!(geometry.col_sizes[0], 100.0);
        assert_eq!(geometry.col_starts[0], 10.0);
        assert_eq!(geometry.col_starts[1], 118.0);

        // Recomputing against a 1.5x viewport keeps the px track at 100 while
        // the fr track grows, so the second boundary no longer scales.
        let zoom = 1.5;
        let zoomed = crate::layout::compute_track_sizes(
            &screen.grid.cols,
            (screen.width as f32 - 20.0) * zoom,
            8.0 * zoom,
        );
        assert_ne!(zoomed[0], geometry.col_sizes[0] * zoom);
    }

    #[test]
    fn status_bar_follows_display_theme() {
        let kdl = r#"screen id="Dock" width=160 height=40 {
            grid cols="1fr" rows="1fr" gap=0 padding=0 {
                status_bar time="12:00" col=0 row=0
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        let dark = render_screen(&screen, DisplayTheme::DarkTft);
        let light = render_screen(&screen, DisplayTheme::LightTft);
        let amber = render_screen(&screen, DisplayTheme::AmberPhosphor);

        // The dock fills the whole screen, so theme-colored pixels must differ.
        assert_ne!(
            dark.pixels, light.pixels,
            "status bar should leave Dark TFT when Light TFT is selected"
        );
        assert_ne!(
            dark.pixels, amber.pixels,
            "status bar should leave Dark TFT when Amber Phosphor is selected"
        );
    }

    #[test]
    fn renders_all_widget_variants_without_panic() {
        let kdl = r##"screen id="KitchenSink" width=320 height=480 {
            grid cols="1fr 1fr" rows="40px 40px 40px 40px 40px 40px 40px 40px 40px" gap=4 padding=4 {
                label text="L" col=0 row=0
                button text="B" col=1 row=0
                toggle label="T" checked=true col=0 row=1
                checkbox label="C" checked=false col=1 row=1
                slider min=0 max=10 value=5 col=0 row=2
                progress value=0.5 col=1 row=2
                dropdown selected=1 col=0 row=3 {
                    option "A"
                    option "B"
                    option "C"
                }
                roller selected=0 col=1 row=3 {
                    option "X"
                    option "Y"
                }
                scale mode="radial" min=0.0 max=100.0 value=40.0 col=0 row=4
                spinbox min=0 max=99 value=12 col=1 row=4
                table col=0 row=5 {
                    headers "A" "B"
                    row "1" "2"
                    row "3" "4"
                }
                sweeping_arc start_angle=0 end_angle=270 col=1 row=5
                busy_wheel active=true col=0 row=6
                status_bar time="12:34" col=1 row=6
                plotter mode="line" col=0 row=7
                crumbs count=4 active=1 col=1 row=7
                rect radius=4 stroke_width=1 fill="#203040" stroke="#80A0C0" col=0 row=8
                circle radius=12 stroke_width=1 fill="#405060" stroke="#FFFFFF" col=1 row=8
            }
        }"##;
        let screen = parse_kdl_screen(kdl).unwrap();
        let frame = render_screen(&screen, DisplayTheme::DarkTft);
        assert_eq!(frame.width, 320);
        assert_eq!(frame.height, 480);
        assert!(frame.pixels.iter().any(|p| *p != frame.pixels[0]));
    }
}
