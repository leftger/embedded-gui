//! Host-side rendering of a [`ScreenDef`] into an RGB565 framebuffer using the
//! real `embedded-gui` [`GuiContext`], so what streams to the board matches
//! silicon pixels rather than the egui preview approximation.

use core::f32::consts::TAU;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use eframe::egui::Color32;
use embedded_graphics_core::Pixel;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Point, Size};
use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_gui::interop::three_d::{Geometry, MeshPanel, MeshShading, render_mesh_panel};
use embedded_gui::prelude::*;
use embedded_gui::{
    BusyWheel, ContentIndicatorDirection, ContentIndicatorWidget, CrumbsIndicatorWidget,
    NumberPickerWidget, PathVerb, PixelRead, ScaleMode, StatusBarWidget, StrokeStyle,
    TimePickerWidget, VectorPath,
};
use embedded_gui_codegen::assets::{BitmapFontData, MeshData, MonoBitmapData};
use embedded_gui_codegen::{PathVerbDef, ScreenDef, WidgetAnimationDef, WidgetDef};

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

struct RawProjectImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

struct LoadedProjectImage {
    width: u32,
    height: u32,
    pixels: Vec<u16>,
}

/// Fonts imported by a screen, keyed by the name KDL refers to them by.
type ScreenFonts = HashMap<String, &'static BitmapFont>;

/// Caches parsed BDF fonts for the lifetime of the process.
///
/// `FontId::Bitmap` borrows for `'static`, but Studio discovers fonts at
/// runtime, so each unique (file, character set) is parsed once and leaked.
/// The set of fonts a designer opens is small and bounded, unlike the render
/// loop that would otherwise leak on every frame.
fn intern_bitmap_font(key: (PathBuf, String), data: BitmapFontData) -> &'static BitmapFont {
    static CACHE: OnceLock<Mutex<HashMap<(PathBuf, String), &'static BitmapFont>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().unwrap_or_else(|err| err.into_inner());
    if let Some(font) = cache.get(&key) {
        return font;
    }
    let glyphs: &'static [u8] = Box::leak(data.glyphs.into_boxed_slice());
    let font: &'static BitmapFont = Box::leak(Box::new(BitmapFont {
        width: data.width,
        height: data.height,
        advance: data.advance,
        line_height: data.line_height,
        first_char: data.first_char,
        bytes_per_row: data.bytes_per_row,
        glyphs,
    }));
    cache.insert(key, font);
    font
}

/// Resolves a project-relative asset path, rejecting anything that escapes the
/// project root.
fn project_asset_path(project_root: Option<&Path>, source: &str) -> Option<PathBuf> {
    let root = project_root?;
    let rel = Path::new(source);
    if rel.is_absolute()
        || rel
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::RootDir))
    {
        return None;
    }
    Some(root.join(rel))
}

fn load_screen_fonts(screen: &ScreenDef, project_root: Option<&Path>) -> ScreenFonts {
    let mut fonts = ScreenFonts::new();
    for font in &screen.fonts {
        let Some(path) = project_asset_path(project_root, &font.source) else {
            continue;
        };
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let chars = (!font.chars.is_empty()).then_some(font.chars.as_str());
        let Ok(data) = embedded_gui_codegen::assets::parse_bdf(&source, chars) else {
            continue;
        };
        let interned = intern_bitmap_font((path, font.chars.clone()), data);
        fonts.insert(font.name.clone(), interned);
    }
    fonts
}

/// Decodes the 1bpp parts of every composite icon on the screen, in widget order.
fn load_icon_bitmaps(
    screen: &ScreenDef,
    project_root: Option<&Path>,
) -> Vec<Vec<(MonoBitmapData, usize)>> {
    screen
        .grid
        .children
        .iter()
        .filter_map(|(_, widget)| {
            let WidgetDef::CompositeIcon {
                parts,
                threshold,
                invert,
                ..
            } = widget
            else {
                return None;
            };
            let decoded = parts
                .iter()
                .enumerate()
                .filter_map(|(part_idx, part)| {
                    let path = project_asset_path(project_root, &part.source)?;
                    let image = image::open(path).ok()?.into_rgba8();
                    let (width, height) = image.dimensions();
                    Some((
                        embedded_gui_codegen::assets::mono_from_rgba(
                            width,
                            height,
                            &image.into_raw(),
                            *threshold,
                            *invert,
                        ),
                        part_idx,
                    ))
                })
                .collect();
            Some(decoded)
        })
        .collect()
}

fn load_meshes(screen: &ScreenDef, project_root: Option<&Path>) -> Vec<Option<MeshData>> {
    screen
        .grid
        .children
        .iter()
        .map(|(_, widget)| {
            let WidgetDef::Mesh3d { source, .. } = widget else {
                return None;
            };
            let path = project_asset_path(project_root, source)?;
            let bytes = std::fs::read(path).ok()?;
            let mut mesh = embedded_gui_codegen::assets::parse_mesh(source, &bytes).ok()?;
            mesh.normalize();
            Some(mesh)
        })
        .collect()
}

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
            Some("accent") | Some("body-accent") | Some("hint-accent") => self.accent,
            Some("success") | Some("body-success") => self.success,
            Some("danger") | Some("body-danger") => self.danger,
            Some("dim") | Some("body-dim") | Some("hint") | Some("hint-dim") => self.text_dim,
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
                // Labels are transparent in embedded-gui. Painting every KDL
                // label as a rounded card made compact legacy UIs look nothing
                // like their firmware output.
                s.background = if other == Some("card") {
                    Some(self.card_bg)
                } else {
                    None
                };
                s.text = c;
                s.foreground = c;
                if matches!(
                    other,
                    Some("body")
                        | Some("body-accent")
                        | Some("body-success")
                        | Some("body-danger")
                        | Some("body-dim")
                        | Some("menu")
                ) {
                    s.font = FontId::Scaled6x10;
                } else if matches!(
                    other,
                    Some("hint") | Some("hint-dim") | Some("hint-accent") | Some("bold")
                ) {
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
    }) || screen
        .grid
        .children
        .iter()
        .any(|(placement, _)| placement.animation.is_some())
}

fn animation_easing(name: &str, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match name {
        "in_sine" => 1.0 - (t * core::f32::consts::FRAC_PI_2).cos(),
        "out_sine" => (t * core::f32::consts::FRAC_PI_2).sin(),
        "in_out_sine" => -((core::f32::consts::PI * t).cos() - 1.0) / 2.0,
        "out_cubic" => 1.0 - (1.0 - t).powi(3),
        "out_back" => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        "out_bounce" => {
            let n1 = 7.5625;
            let d1 = 2.75;
            if t < 1.0 / d1 {
                n1 * t * t
            } else if t < 2.0 / d1 {
                let x = t - 1.5 / d1;
                n1 * x * x + 0.75
            } else if t < 2.5 / d1 {
                let x = t - 2.25 / d1;
                n1 * x * x + 0.9375
            } else {
                let x = t - 2.625 / d1;
                n1 * x * x + 0.984375
            }
        }
        // Close host-side approximation of the runtime's compact spatial curve.
        "moook" => t * t * (3.0 - 2.0 * t),
        _ => t,
    }
}

fn sample_widget_animation(animation: &WidgetAnimationDef, phase: f32, cell: &Cell) -> (Cell, u8) {
    let duration = animation.duration_ms.max(1) as f32;
    let delay = animation.delay_ms as f32;
    let cycle = duration + delay;
    let plays = if animation.repeat == 0 {
        1.0
    } else {
        animation.repeat as f32
    };
    let elapsed = phase.rem_euclid(1.0) * cycle * plays;
    let in_cycle = elapsed.rem_euclid(cycle.max(1.0));
    let raw = ((in_cycle - delay) / duration).clamp(0.0, 1.0);
    let mut t = animation_easing(&animation.easing, raw);
    if animation.trigger == "screen_exit" {
        t = 1.0 - t;
    }
    let mut out = Cell {
        x: cell.x,
        y: cell.y,
        w: cell.w,
        h: cell.h,
    };
    let mut opacity = 255u8;
    let travel_x = cell.w.max(16) as f32;
    let travel_y = cell.h.max(16) as f32;

    match animation.preset.as_str() {
        "fade_in" => opacity = (255.0 * t.clamp(0.0, 1.0)) as u8,
        "fade_in_up" => {
            out.y += (24.0 * (1.0 - t)) as i32;
            opacity = (255.0 * t.clamp(0.0, 1.0)) as u8;
        }
        "slide_in_left" => out.x -= (travel_x * (1.0 - t)) as i32,
        "slide_in_right" => out.x += (travel_x * (1.0 - t)) as i32,
        "slide_in_up" => out.y += (travel_y * (1.0 - t)) as i32,
        "slide_in_down" => out.y -= (travel_y * (1.0 - t)) as i32,
        "zoom_in" => {
            let scale = (0.35 + 0.65 * t).clamp(0.05, 1.2);
            let w = (cell.w as f32 * scale).max(1.0) as u32;
            let h = (cell.h as f32 * scale).max(1.0) as u32;
            out.x += (cell.w.saturating_sub(w) / 2) as i32;
            out.y += (cell.h.saturating_sub(h) / 2) as i32;
            out.w = w;
            out.h = h;
            opacity = (255.0 * t.clamp(0.0, 1.0)) as u8;
        }
        "pulse" => {
            let scale = 1.0 + 0.08 * (core::f32::consts::PI * raw).sin();
            let w = (cell.w as f32 * scale) as u32;
            let h = (cell.h as f32 * scale) as u32;
            out.x -= (w.saturating_sub(cell.w) / 2) as i32;
            out.y -= (h.saturating_sub(cell.h) / 2) as i32;
            out.w = w;
            out.h = h;
        }
        "breathe" => {
            opacity = (150.0 + 105.0 * (core::f32::consts::PI * raw).sin()) as u8;
        }
        "shake" => {
            out.x += (5.0 * (raw * core::f32::consts::PI * 6.0).sin() * (1.0 - raw)) as i32;
        }
        _ => {}
    }
    (out, opacity)
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
#[cfg(test)]
pub fn render_screen_at(
    screen: &ScreenDef,
    theme: DisplayTheme,
    animation_phase: f32,
    highlight: Option<usize>,
) -> RenderedFrame {
    render_screen_at_with_assets(screen, theme, animation_phase, highlight, None, None)
}

/// Renders a screen and resolves `image src="..."` nodes relative to the
/// project root. Invalid, absolute, or escaping paths are ignored.
///
/// `widget_override` lets a single widget (by grid child index) be sampled at
/// its own independent phase, overriding the shared `animation_phase` for that
/// widget only. Used by the inspector's one-click animation preview so
/// replaying one widget doesn't restart every other animated widget sharing
/// the screen's timeline.
pub fn render_screen_at_with_assets(
    screen: &ScreenDef,
    theme: DisplayTheme,
    animation_phase: f32,
    highlight: Option<usize>,
    project_root: Option<&Path>,
    widget_override: Option<(usize, f32)>,
) -> RenderedFrame {
    let mut option_lists: Vec<Vec<&str>> = Vec::new();
    let mut table_storage: Vec<Vec<Vec<&str>>> = Vec::new();
    let mut plot_samples: Vec<Vec<f32>> = Vec::new();
    let raw_images = load_project_images(screen, project_root);

    for (_, widget) in &screen.grid.children {
        match widget {
            WidgetDef::Dropdown { options, .. } | WidgetDef::Roller { options, .. } => {
                option_lists.push(options.iter().map(String::as_str).collect());
            }
            WidgetDef::Carousel { items, .. } => {
                option_lists.push(items.iter().map(String::as_str).collect());
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

    let fonts = load_screen_fonts(screen, project_root);
    let icon_bitmaps = load_icon_bitmaps(screen, project_root);
    let meshes = load_meshes(screen, project_root);

    render_inner(
        screen,
        theme,
        animation_phase.rem_euclid(1.0),
        &option_lists,
        &table_rows,
        &plot_samples,
        &raw_images,
        &fonts,
        &icon_bitmaps,
        &meshes,
        highlight,
        widget_override,
    )
}

fn load_project_images(
    screen: &ScreenDef,
    project_root: Option<&Path>,
) -> Vec<Option<RawProjectImage>> {
    screen
        .grid
        .children
        .iter()
        .map(|(_, widget)| {
            let WidgetDef::Image { source, .. } = widget else {
                return None;
            };
            let root = project_root?;
            let rel = Path::new(source);
            if rel.is_absolute()
                || rel
                    .components()
                    .any(|part| matches!(part, Component::ParentDir | Component::RootDir))
            {
                return None;
            }
            let decoded = image::open(root.join(rel)).ok()?.into_rgba8();
            let (width, height) = decoded.dimensions();
            Some(RawProjectImage {
                width,
                height,
                rgba: decoded.into_raw(),
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn render_inner<'a>(
    screen: &'a ScreenDef,
    theme: DisplayTheme,
    animation_phase: f32,
    option_lists: &'a [Vec<&'a str>],
    table_rows: &'a [Vec<&'a [&'a str]>],
    plot_samples: &'a [Vec<f32>],
    raw_images: &'a [Option<RawProjectImage>],
    fonts: &'a ScreenFonts,
    icon_bitmaps: &'a [Vec<(MonoBitmapData, usize)>],
    meshes: &'a [Option<MeshData>],
    highlight: Option<usize>,
    widget_override: Option<(usize, f32)>,
) -> RenderedFrame {
    let width = screen.width.max(1) as u16;
    let height = screen.height.max(1) as u16;
    let palette = Palette565::for_theme(theme);
    let loaded_images: Vec<Option<LoadedProjectImage>> = screen
        .grid
        .children
        .iter()
        .zip(raw_images)
        .map(|((_, widget), raw)| {
            let WidgetDef::Image { mode, tint, .. } = widget else {
                return None;
            };
            raw.as_ref()
                .map(|raw| convert_project_image(raw, mode, tint.as_deref(), &palette))
        })
        .collect();
    let mut pixels = vec![palette.display_bg; width as usize * height as usize];
    let mut cells = compute_cells(screen);
    let mut widget_opacities = vec![255u8; cells.len()];
    for (idx, ((placement, _), cell)) in screen
        .grid
        .children
        .iter()
        .zip(cells.iter_mut())
        .enumerate()
    {
        if let Some(animation) = &placement.animation {
            let phase = match widget_override {
                Some((override_idx, override_phase)) if override_idx == idx => override_phase,
                _ => animation_phase,
            };
            let (animated, opacity) = sample_widget_animation(animation, phase, cell);
            *cell = animated;
            widget_opacities[idx] = opacity;
        }
    }

    // Icon parts borrow their bitmaps, so the `IconPart` slices have to outlive
    // the context they are handed to: declare them before it.
    let icon_parts: Vec<Vec<IconPart<'_>>> =
        screen
            .grid
            .children
            .iter()
            .filter_map(|(_, widget)| match widget {
                WidgetDef::CompositeIcon { parts, tint, .. } => Some((parts, tint)),
                _ => None,
            })
            .zip(icon_bitmaps)
            .map(|((parts, tint), decoded)| {
                decoded
                    .iter()
                    .map(|(bitmap, part_idx)| {
                        let part = &parts[*part_idx];
                        IconPart {
                            bitmap: MonoBitmap::new(bitmap.width, bitmap.height, &bitmap.bits),
                            dx: part.dx,
                            dy: part.dy,
                            visible: part.visible,
                            tint: part.tint.as_deref().or(tint.as_deref()).map(|token| {
                                parse_hex_color(token, palette.token_color(Some(token)))
                            }),
                        }
                    })
                    .collect()
            })
            .collect();

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
    let mut icon_idx = 0usize;

    for (idx, (_, widget)) in screen.grid.children.iter().enumerate() {
        let Some(cell) = cells.get(idx) else { continue };
        let rect = Rect::new(cell.x, cell.y, cell.w, cell.h);
        if let Some(widget_id) = add_widget(
            &mut gui,
            rect,
            widget,
            &palette,
            animation_phase,
            option_lists,
            table_rows,
            plot_samples,
            loaded_images.get(idx).and_then(Option::as_ref),
            fonts,
            &icon_parts,
            &mut option_idx,
            &mut table_idx,
            &mut plot_idx,
            &mut icon_idx,
            &mut overlays,
        ) {
            if let Some(opacity) = widget_opacities
                .get(idx)
                .copied()
                .filter(|value| *value < 255)
            {
                let _ = gui.set_widget_opacity(widget_id, opacity);
            }
        }
    }

    {
        let mut target = BufferTarget {
            buf: &mut pixels,
            w: width as u32,
            h: height as u32,
        };
        let _ = gui.render(&mut target);

        // Meshes rasterize straight into the framebuffer: the 3D pipeline owns
        // a Z-buffer and cannot go through the widget tree.
        let mut zbuffer = Vec::new();
        for (idx, (_, widget)) in screen.grid.children.iter().enumerate() {
            let WidgetDef::Mesh3d { .. } = widget else {
                continue;
            };
            let (Some(cell), Some(Some(mesh))) = (cells.get(idx), meshes.get(idx)) else {
                continue;
            };
            let rect = Rect::new(cell.x, cell.y, cell.w, cell.h);
            zbuffer.resize((rect.w * rect.h) as usize, 0);
            let panel = mesh_panel_for(widget, mesh, &palette);
            let _ = render_mesh_panel(&mut target, rect, &panel, &mut zbuffer);
        }

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

/// Builds the 3D panel for a `mesh` node, resolving its color token against the
/// preview palette.
fn mesh_panel_for<'m>(
    widget: &WidgetDef,
    mesh: &'m MeshData,
    palette: &Palette565,
) -> MeshPanel<'m> {
    let WidgetDef::Mesh3d {
        shading,
        color,
        scale,
        roll,
        pitch,
        yaw,
        camera_distance,
        fov,
        ..
    } = widget
    else {
        unreachable!("mesh_panel_for is only called for mesh nodes");
    };

    let mut panel = MeshPanel::new(
        Geometry {
            vertices: &mesh.vertices,
            faces: &mesh.faces,
            normals: &mesh.normals,
            ..Geometry::default()
        },
        color
            .as_deref()
            .map(|token| parse_hex_color(token, palette.token_color(Some(token))))
            .unwrap_or(palette.text_primary),
    );
    panel.shading = match shading.as_str() {
        "points" => MeshShading::Points,
        "lines" | "wireframe" => MeshShading::Lines,
        "lit" => MeshShading::Lit,
        _ => MeshShading::Solid,
    };
    panel.scale = *scale;
    panel.attitude = (*roll, *pitch, *yaw);
    panel.camera_distance = *camera_distance;
    panel.fov = *fov;
    panel
}

fn convert_project_image(
    raw: &RawProjectImage,
    mode: &str,
    tint: Option<&str>,
    palette: &Palette565,
) -> LoadedProjectImage {
    let tint_color = tint
        .map(|value| parse_hex_color(value, palette.token_color(Some(value))))
        .unwrap_or(palette.text_primary);
    let mut pixels = Vec::with_capacity((raw.width * raw.height) as usize);
    for rgba in raw.rgba.chunks_exact(4) {
        let alpha = u16::from(rgba[3]);
        let color = if mode == "mask" || mode == "mono" {
            let luminance =
                (u16::from(rgba[0]) * 77 + u16::from(rgba[1]) * 150 + u16::from(rgba[2]) * 29) >> 8;
            if alpha > 0 && luminance < 128 {
                tint_color
            } else {
                palette.display_bg
            }
        } else {
            let bg_r = u16::from(palette.display_bg.r()) * 255 / 31;
            let bg_g = u16::from(palette.display_bg.g()) * 255 / 63;
            let bg_b = u16::from(palette.display_bg.b()) * 255 / 31;
            let r = (u16::from(rgba[0]) * alpha + bg_r * (255 - alpha)) / 255;
            let g = (u16::from(rgba[1]) * alpha + bg_g * (255 - alpha)) / 255;
            let b = (u16::from(rgba[2]) * alpha + bg_b * (255 - alpha)) / 255;
            Rgb565::new(
                (r * 31 / 255) as u8,
                (g * 63 / 255) as u8,
                (b * 31 / 255) as u8,
            )
        };
        pixels.push(
            (u16::from(color.r()) << 11) | (u16::from(color.g()) << 5) | u16::from(color.b()),
        );
    }
    LoadedProjectImage {
        width: raw.width,
        height: raw.height,
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
    image: Option<&'a LoadedProjectImage>,
    fonts: &'a ScreenFonts,
    icon_parts: &'a [Vec<IconPart<'a>>],
    option_idx: &mut usize,
    table_idx: &mut usize,
    plot_idx: &mut usize,
    icon_idx: &mut usize,
    overlays: &mut Vec<Overlay<'a>>,
) -> Option<WidgetId> {
    let style = p.panel(Some("card"));
    match widget {
        WidgetDef::Label {
            text,
            style: token,
            font,
            ..
        } => {
            let mut label_style = p.label(token.as_deref());
            if let Some(custom) = font.as_deref().and_then(|name| fonts.get(name)) {
                label_style.font = FontId::Bitmap(custom);
            }
            gui.add_label(rect, text.as_str(), label_style)
        }
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
        WidgetDef::Carousel {
            selected,
            item_step,
            visible,
            shift,
            mask_top,
            mask_bottom,
            fade,
            indicator,
            pulse,
            style: token,
            font,
            ..
        } => {
            let items = option_lists
                .get(*option_idx)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            *option_idx += 1;
            let mut carousel_style = p.label(token.as_deref());
            carousel_style.background = Some(p.display_bg);
            if let Some(custom) = font.as_deref().and_then(|name| fonts.get(name)) {
                carousel_style.font = FontId::Bitmap(custom);
            }
            gui.add_carousel(
                rect,
                items,
                *selected,
                CarouselSpec {
                    item_step: *item_step,
                    visible_slots: *visible,
                    shift: *shift,
                    mask_top: *mask_top,
                    mask_bottom: *mask_bottom,
                    fade_edges: *fade,
                    indicator: *indicator,
                    indicator_pulse: *pulse,
                    ..CarouselSpec::default()
                },
                carousel_style,
            )
        }
        WidgetDef::CompositeIcon { scale, align, .. } => {
            let parts = icon_parts
                .get(*icon_idx)
                .map(Vec::as_slice)
                .unwrap_or(&[] as &[IconPart<'a>]);
            *icon_idx += 1;
            let mut icon_style = p.label(None);
            icon_style.background = None;
            gui.add_composite_icon(
                rect,
                parts,
                CompositeIconSpec {
                    scale: *scale,
                    align: if align == "top_left" {
                        IconAlign::TopLeft
                    } else {
                        IconAlign::Center
                    },
                    paper: None,
                },
                icon_style,
            )
        }
        // Meshes are rasterized after the widget pass, straight into the
        // framebuffer, so they only reserve their rect here.
        WidgetDef::Mesh3d { .. } => gui.add_spacer(rect),
        WidgetDef::Panel { style: token, .. } => gui.add_panel(rect, p.panel(token.as_deref())),
        WidgetDef::Image { fit, .. } => {
            if let Some(image) = image {
                let fit = if fit == "center" {
                    ImageFit::Center
                } else {
                    ImageFit::Stretch
                };
                gui.add_image(
                    rect,
                    ImageRef::new(image.width, image.height, &image.pixels),
                    fit,
                    Style::default(),
                )
            } else {
                gui.add_spacer(rect)
            }
        }
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
    }
    .ok()
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
    fn declarative_widget_animation_changes_preview_pixels() {
        let kdl = r#"screen id="Motion" width=160 height=80 {
            grid cols="1fr" rows="1fr" gap=0 padding=8 {
                button id="go" text="GO" animation="slide_in_left" animation_duration=400 animation_easing="out_cubic" col=0 row=0
            }
        }"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        assert!(has_animated_content(&screen));

        let first = render_screen_at(&screen, DisplayTheme::DarkTft, 0.05, None);
        let last = render_screen_at(&screen, DisplayTheme::DarkTft, 0.95, None);
        assert!(!changed_tiles(&first, &last, 20, 20).is_empty());
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

    fn demo_project_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/ssd1357-demo")
    }

    fn render_demo_screen(name: &str) -> RenderedFrame {
        let root = demo_project_root();
        let source = std::fs::read_to_string(root.join("screens").join(name)).unwrap();
        let screen = parse_kdl_screen(&source).unwrap();
        render_screen_at_with_assets(&screen, DisplayTheme::DarkTft, 0.0, None, Some(&root), None)
    }

    #[test]
    fn carousel_dims_rows_away_from_the_selection() {
        let frame = render_demo_screen("menu.kdl");
        // The selected row is centered and full brightness; neighbours fall off.
        let brightest = |row: u32| {
            (0..frame.width as u32)
                .map(|x| {
                    let px = frame.at(x, row);
                    u16::from(px.r()) + u16::from(px.g()) + u16::from(px.b())
                })
                .max()
                .unwrap_or(0)
        };
        let center = (28..36).map(brightest).max().unwrap();
        let neighbour = (44..52).map(brightest).max().unwrap();
        assert!(
            center > neighbour,
            "center {center} <= neighbour {neighbour}"
        );
        assert!(neighbour > 0, "neighbouring row was not drawn at all");
    }

    #[test]
    fn carousel_masks_hide_overhang_behind_the_header() {
        let frame = render_demo_screen("menu.kdl");
        // Row 4 sits inside the 14px header band the carousel masks out, so
        // anything there is either backdrop or the header label drawn after.
        let backdrop = frame.at(0, 4);
        let ink = (0..frame.width as u32)
            .filter(|x| frame.at(*x, 4) != backdrop)
            .count();
        assert!(
            ink < frame.width as usize / 3,
            "carousel rows bled through the masked band: {ink} px"
        );
    }

    #[test]
    fn counter_screen_draws_font_icon_and_mesh() {
        let frame = render_demo_screen("counter.kdl");
        assert_eq!((frame.width, frame.height), (96, 64));

        // Big BDF digits occupy the middle band on the left.
        let digits = (0..70)
            .flat_map(|x| (16..46).map(move |y| (x, y)))
            .filter(|(x, y)| frame.at(*x, *y) != Rgb565::BLACK)
            .count();
        assert!(digits > 100, "seven-segment digits missing: {digits} px");

        // The battery icon occupies the right-hand column.
        let lit = (70..96)
            .flat_map(|x| (16..46).map(move |y| (x, y)))
            .filter(|(x, y)| frame.at(*x, *y) != Rgb565::BLACK)
            .count();
        assert!(lit > 20, "composite icon missing: {lit} px");

        // The mesh rasterizes into the bottom band.
        let mesh = (0..96)
            .flat_map(|x| (50..64).map(move |y| (x, y)))
            .filter(|(x, y)| frame.at(*x, *y) != Rgb565::BLACK)
            .count();
        assert!(mesh > 20, "mesh did not rasterize: {mesh} px");
    }

    #[test]
    fn renders_a_mesh_exported_as_binary_stl() {
        // CAD tools export STL, so a project should be able to point a mesh
        // node straight at the file the artist already has.
        let root = std::env::temp_dir().join(format!("egs-stl-{}", std::process::id()));
        std::fs::create_dir_all(root.join("assets/meshes")).unwrap();

        let corners: [[[f32; 3]; 3]; 2] = [
            [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0]],
            [[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]],
        ];
        let mut stl = vec![0u8; 80];
        stl.extend_from_slice(&(corners.len() as u32).to_le_bytes());
        for triangle in corners {
            stl.extend_from_slice(&[0u8; 12]);
            for corner in triangle {
                for axis in corner {
                    stl.extend_from_slice(&axis.to_le_bytes());
                }
            }
            stl.extend_from_slice(&[0u8; 2]);
        }
        std::fs::write(root.join("assets/meshes/quad.stl"), &stl).unwrap();

        let kdl = r##"screen id="Stl" width=96 height=64 {
            grid cols="1fr" rows="1fr" gap=0 padding=0 {
                mesh id="quad" src="assets/meshes/quad.stl" shading="solid" color="#00FF00" scale=1.0 camera_distance=3.0 fov=1.2 col=0 row=0
            }
        }"##;
        let screen = parse_kdl_screen(kdl).unwrap();
        let frame = render_screen_at_with_assets(
            &screen,
            DisplayTheme::DarkTft,
            0.0,
            None,
            Some(&root),
            None,
        );

        let lit = frame
            .pixels
            .iter()
            .filter(|px| **px != Rgb565::BLACK)
            .count();
        std::fs::remove_dir_all(&root).ok();
        assert!(lit > 100, "STL mesh did not rasterize: {lit} px");
    }

    #[test]
    fn hidden_icon_parts_are_not_drawn() {
        let root = demo_project_root();
        let source = std::fs::read_to_string(root.join("screens/counter.kdl")).unwrap();
        let mut screen = parse_kdl_screen(&source).unwrap();
        let with_bolt_hidden = render_screen_at_with_assets(
            &screen,
            DisplayTheme::DarkTft,
            0.0,
            None,
            Some(&root),
            None,
        );

        for (_, widget) in &mut screen.grid.children {
            if let WidgetDef::CompositeIcon { parts, .. } = widget {
                for part in parts {
                    part.visible = true;
                }
            }
        }
        let with_bolt_shown = render_screen_at_with_assets(
            &screen,
            DisplayTheme::DarkTft,
            0.0,
            None,
            Some(&root),
            None,
        );

        let changed = with_bolt_hidden
            .pixels
            .iter()
            .zip(&with_bolt_shown.pixels)
            .filter(|(hidden, shown)| hidden != shown)
            .count();
        assert!(changed > 10, "toggling a part changed {changed} pixels");
    }
}
