pub mod compositor;
pub mod stroke;
pub mod text_style;

use core::marker::PhantomData;

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
};

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    font::{FontId, glyph_rows},
    geometry::Rect,
    image::{ImageFit, ImageRef, TileMode, TileRef},
    palette::{DisplayPalette, InkRole},
    style::{AlphaLinearGradient, AlphaRadialGradient, Border, GradientDirection, LinearGradient},
    text,
};

pub const CHAR_WIDTH: u32 = 4;
pub const CHAR_HEIGHT: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrap {
    None,
    Character,
    Word,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOverflow {
    Clip,
    Ellipsis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EllipsisMode {
    ThreeDots,
    SingleGlyph,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOverflowPolicy {
    Global(TextOverflow),
    WrapThenEllipsis { max_lines: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextStyle {
    pub color: Rgb565,
    pub font: FontId,
    pub opacity: u8,
    pub align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub overflow_policy: TextOverflowPolicy,
    pub kerning: bool,
    pub max_lines: Option<u8>,
    pub ellipsis: EllipsisMode,
    pub line_spacing: u8,
}

impl TextStyle {
    pub const fn new(color: Rgb565) -> Self {
        Self {
            color,
            font: FontId::Tiny3x5,
            opacity: 255,
            align: TextAlign::Left,
            vertical_align: VerticalAlign::Top,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            overflow_policy: TextOverflowPolicy::Global(TextOverflow::Clip),
            kerning: false,
            max_lines: None,
            ellipsis: EllipsisMode::ThreeDots,
            line_spacing: 1,
        }
    }

    pub const fn centered(mut self) -> Self {
        self.align = TextAlign::Center;
        self.vertical_align = VerticalAlign::Middle;
        self
    }

    pub const fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub const fn with_vertical_align(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = align;
        self
    }

    pub const fn with_wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub const fn with_line_spacing(mut self, spacing: u8) -> Self {
        self.line_spacing = spacing;
        self
    }

    pub const fn with_overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self.overflow_policy = TextOverflowPolicy::Global(overflow);
        self
    }

    pub const fn with_kerning(mut self, kerning: bool) -> Self {
        self.kerning = kerning;
        self
    }

    pub const fn with_max_lines(mut self, max_lines: Option<u8>) -> Self {
        self.max_lines = max_lines;
        self
    }

    pub const fn with_ellipsis_mode(mut self, ellipsis: EllipsisMode) -> Self {
        self.ellipsis = ellipsis;
        self
    }

    pub const fn with_overflow_policy(mut self, policy: TextOverflowPolicy) -> Self {
        self.overflow_policy = policy;
        self
    }

    pub const fn with_opacity(mut self, opacity: u8) -> Self {
        self.opacity = opacity;
        self
    }

    pub const fn with_font_id(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    pub fn with_font(mut self, font: impl Into<FontId>) -> Self {
        self.font = font.into();
        self
    }
}

#[cfg(feature = "embedded-graphics")]
impl From<&embedded_graphics::mono_font::MonoTextStyle<'static, Rgb565>> for TextStyle {
    fn from(mono_style: &embedded_graphics::mono_font::MonoTextStyle<'static, Rgb565>) -> Self {
        let mut style = TextStyle::new(mono_style.text_color.unwrap_or(Rgb565::WHITE));
        style.font = FontId::MonoFont(mono_style.font);
        style
    }
}

#[cfg(feature = "embedded-graphics")]
impl From<embedded_graphics::mono_font::MonoTextStyle<'static, Rgb565>> for TextStyle {
    fn from(mono_style: embedded_graphics::mono_font::MonoTextStyle<'static, Rgb565>) -> Self {
        Self::from(&mono_style)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextMetrics {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AntiAliasMode {
    None,
    Coverage,
    Subpixel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrokeStyle {
    pub color: Rgb565,
    pub width: u8,
    pub antialias: bool,
    pub antialias_mode: AntiAliasMode,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeCap {
    Butt,
    Round,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeJoin {
    Miter,
    Round,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub const fn translation(x: f32, y: f32) -> Self {
        Self {
            tx: x,
            ty: y,
            ..Self::IDENTITY
        }
    }

    pub const fn scale(x: f32, y: f32) -> Self {
        Self {
            m11: x,
            m22: y,
            ..Self::IDENTITY
        }
    }

    pub fn rotation(deg: f32) -> Self {
        let r = deg.to_radians();
        Self {
            m11: r.cos(),
            m12: -r.sin(),
            m21: r.sin(),
            m22: r.cos(),
            ..Self::IDENTITY
        }
    }

    pub fn skew(x_deg: f32, y_deg: f32) -> Self {
        Self {
            m12: x_deg.to_radians().tan(),
            m21: y_deg.to_radians().tan(),
            ..Self::IDENTITY
        }
    }

    pub fn then(self, rhs: Self) -> Self {
        Self {
            m11: self.m11 * rhs.m11 + self.m12 * rhs.m21,
            m12: self.m11 * rhs.m12 + self.m12 * rhs.m22,
            m21: self.m21 * rhs.m11 + self.m22 * rhs.m21,
            m22: self.m21 * rhs.m12 + self.m22 * rhs.m22,
            tx: self.m11 * rhs.tx + self.m12 * rhs.ty + self.tx,
            ty: self.m21 * rhs.tx + self.m22 * rhs.ty + self.ty,
        }
    }

    #[inline(always)]
    pub fn is_identity(self) -> bool {
        self.m11 == 1.0
            && self.m12 == 0.0
            && self.m21 == 0.0
            && self.m22 == 1.0
            && self.tx == 0.0
            && self.ty == 0.0
    }

    #[inline(always)]
    pub fn apply(self, x: i32, y: i32) -> (i32, i32) {
        if self.is_identity() {
            (x, y)
        } else {
            let xf = x as f32;
            let yf = y as f32;
            (
                (self.m11 * xf + self.m12 * yf + self.tx).round() as i32,
                (self.m21 * xf + self.m22 * yf + self.ty).round() as i32,
            )
        }
    }

    #[inline(always)]
    pub fn apply_f32(self, x: f32, y: f32) -> (f32, f32) {
        if self.is_identity() {
            (x, y)
        } else {
            (
                self.m11 * x + self.m12 * y + self.tx,
                self.m21 * x + self.m22 * y + self.ty,
            )
        }
    }

    pub fn inverse(self) -> Option<Self> {
        let det = self.m11 * self.m22 - self.m12 * self.m21;
        if det.abs() < 1e-7 {
            return None;
        }
        let inv_det = 1.0 / det;
        let m11 = self.m22 * inv_det;
        let m12 = -self.m12 * inv_det;
        let m21 = -self.m21 * inv_det;
        let m22 = self.m11 * inv_det;
        let tx = (self.m12 * self.ty - self.m22 * self.tx) * inv_det;
        let ty = (self.m21 * self.tx - self.m11 * self.ty) * inv_det;
        Some(Self {
            m11,
            m12,
            m21,
            m22,
            tx,
            ty,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
}

/// Capability trait for draw targets that can read back a pixel they've
/// already written. Optional: the default opacity/blend APIs on
/// [`RenderCtx`] only require [`DrawTarget`] and approximate translucency
/// with ordered dithering, so they keep working on write-only displays.
/// Implementing `PixelRead` additionally unlocks the `*_true_alpha` methods,
/// which composite against the destination's actual current contents
/// instead of dithering.
///
/// Re-exported from [`embedded_draw_target`] so that a buffer implementing it
/// once is accepted by every crate in the ecosystem that needs readback,
/// rather than by this one alone.
pub use embedded_draw_target::PixelRead;

/// Capability trait for hardware display controllers (e.g. ST7789, ILI9341, SSD1306)
/// supporting direct column/row address window setting (`set_address_window`).
/// Allows rendering dirty regions by transmitting SPI/DMA transfers exclusively to
/// the target sub-window instead of the full screen.
///
/// Re-exported from [`embedded_draw_target`].
pub use embedded_draw_target::WindowedDrawTarget;

/// Pixel-plotting policy for a [`RenderCtx`]. Selected by the ctx's `C` type
/// parameter so the *same* drawing calls composite differently depending on
/// the target's capabilities — with no runtime branch and no specialization.
///
/// - [`Dither`] (the default) approximates translucency with ordered dithering
///   and works on any write-only [`DrawTarget`].
/// - [`Blend`] performs true per-pixel alpha compositing and requires a
///   readback-capable target ([`PixelRead`]), e.g. a
///   [`Framebuffer`](crate::Framebuffer).
///
/// `plot` receives screen-space coords already transformed and clipped, the
/// layer-combined `opacity`, and the active layer blend mode + backdrop.
pub trait Compositor<D: DrawTarget<Color = Rgb565>> {
    fn plot(
        target: &mut D,
        x: i32,
        y: i32,
        color: Rgb565,
        opacity: u8,
        blend: BlendMode,
        backdrop: Rgb565,
    ) -> Result<(), D::Error>;
}

/// Ordered-dither compositor (default). No readback required.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dither;

/// True alpha-blending compositor. Requires a [`PixelRead`] target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Blend;

impl<D: DrawTarget<Color = Rgb565>> Compositor<D> for Dither {
    fn plot(
        target: &mut D,
        x: i32,
        y: i32,
        color: Rgb565,
        opacity: u8,
        blend: BlendMode,
        backdrop: Rgb565,
    ) -> Result<(), D::Error> {
        if !should_draw_at_opacity(x, y, opacity) {
            return Ok(());
        }
        let color = apply_blend_mode(color, blend, backdrop);
        target.draw_iter([Pixel(Point::new(x, y), color)])
    }
}

impl<D: DrawTarget<Color = Rgb565> + PixelRead> Compositor<D> for Blend {
    fn plot(
        target: &mut D,
        x: i32,
        y: i32,
        color: Rgb565,
        opacity: u8,
        blend: BlendMode,
        backdrop: Rgb565,
    ) -> Result<(), D::Error> {
        if opacity == 0 {
            return Ok(());
        }
        let bg = target.get_pixel(Point::new(x, y));
        let blended = lerp_rgb565(bg, color, opacity);
        let blended = apply_blend_mode(blended, blend, backdrop);
        target.draw_iter([Pixel(Point::new(x, y), blended)])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorFormat {
    Rgb565,
    Rgb888,
    Argb8888,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderBackendCaps {
    pub color_format: ColorFormat,
    pub supports_layers: bool,
    pub supports_subpixel: bool,
}

impl RenderBackendCaps {
    pub const fn software_rgb565() -> Self {
        Self {
            color_format: ColorFormat::Rgb565,
            supports_layers: true,
            supports_subpixel: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerState {
    pub opacity: u8,
    pub blend: BlendMode,
    pub backdrop: Rgb565,
}

impl LayerState {
    pub const fn normal() -> Self {
        Self {
            opacity: 255,
            blend: BlendMode::Normal,
            backdrop: Rgb565::BLACK,
        }
    }
}

impl StrokeStyle {
    pub const fn new(color: Rgb565) -> Self {
        Self {
            color,
            width: 1,
            antialias: false,
            antialias_mode: AntiAliasMode::None,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
        }
    }

    pub const fn with_width(mut self, width: u8) -> Self {
        self.width = if width == 0 { 1 } else { width };
        self
    }

    pub const fn with_antialias(mut self, antialias: bool) -> Self {
        self.antialias = antialias;
        if antialias {
            if let AntiAliasMode::None = self.antialias_mode {
                self.antialias_mode = AntiAliasMode::Coverage;
            }
        }
        if !antialias {
            self.antialias_mode = AntiAliasMode::None;
        }
        self
    }

    pub const fn with_antialias_mode(mut self, mode: AntiAliasMode) -> Self {
        self.antialias_mode = mode;
        self.antialias = !matches!(mode, AntiAliasMode::None);
        self
    }

    pub const fn with_cap(mut self, cap: StrokeCap) -> Self {
        self.cap = cap;
        self
    }

    pub const fn with_join(mut self, join: StrokeJoin) -> Self {
        self.join = join;
        self
    }
}

pub struct RenderCtx<'a, D, C = Dither>
where
    D: DrawTarget<Color = Rgb565>,
{
    target: &'a mut D,
    clip: Rect,
    dirty: Option<Rect>,
    quality: RenderQuality,
    backend_caps: RenderBackendCaps,
    transform_stack: [Transform2D; 8],
    transform_len: usize,
    layer_stack: [LayerState; 8],
    layer_len: usize,
    palette: Option<DisplayPalette>,
    _compositor: PhantomData<C>,
}

impl<'a, D> RenderCtx<'a, D, Dither>
where
    D: DrawTarget<Color = Rgb565>,
{
    pub fn new(target: &'a mut D, viewport: Rect) -> Self {
        Self {
            target,
            clip: viewport,
            dirty: None,
            quality: RenderQuality::High,
            backend_caps: RenderBackendCaps::software_rgb565(),
            transform_stack: [Transform2D::IDENTITY; 8],
            transform_len: 1,
            layer_stack: [LayerState::normal(); 8],
            layer_len: 1,
            palette: None,
            _compositor: PhantomData,
        }
    }

    pub fn with_palette(mut self, palette: DisplayPalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Resolve a semantic ink role through the active palette, if set.
    pub fn ink(&self, role: InkRole) -> Rgb565 {
        self.palette
            .map(|palette| palette.resolve(role))
            .unwrap_or(Rgb565::WHITE)
    }

    pub fn with_dirty(target: &'a mut D, viewport: Rect, dirty: Rect) -> Self {
        Self {
            target,
            clip: viewport,
            dirty: Some(dirty),
            quality: RenderQuality::High,
            backend_caps: RenderBackendCaps::software_rgb565(),
            transform_stack: [Transform2D::IDENTITY; 8],
            transform_len: 1,
            layer_stack: [LayerState::normal(); 8],
            layer_len: 1,
            palette: None,
            _compositor: PhantomData,
        }
    }
}

impl<'a, D> RenderCtx<'a, D, Blend>
where
    D: DrawTarget<Color = Rgb565> + PixelRead,
{
    /// Like [`RenderCtx::new`], but every drawing call alpha-composites against
    /// the target's current contents (true blending) instead of dithering.
    /// Requires a readback-capable target ([`PixelRead`]), e.g. a
    /// [`Framebuffer`](crate::Framebuffer).
    pub fn compositing(target: &'a mut D, viewport: Rect) -> Self {
        Self {
            target,
            clip: viewport,
            dirty: None,
            quality: RenderQuality::High,
            backend_caps: RenderBackendCaps::software_rgb565(),
            transform_stack: [Transform2D::IDENTITY; 8],
            transform_len: 1,
            layer_stack: [LayerState::normal(); 8],
            layer_len: 1,
            palette: None,
            _compositor: PhantomData,
        }
    }

    /// Apply Fast IIR Blur to a sub-region `rect` on the destination target.
    pub fn blur_rect(&mut self, rect: Rect, blur_degree: u8) -> Result<(), D::Error> {
        let draw = self.visible_rect(rect);
        if draw.is_empty() || blur_degree == 0 {
            return Ok(());
        }
        let x0 = draw.x;
        let y0 = draw.y;
        let x1 = draw.right();
        let y1 = draw.bottom();
        let alpha = 256 - (blur_degree as i32);

        let mut row_buf = [Rgb565::BLACK; 1024];
        let w_buf = ((x1 - x0) as usize).min(1024);

        // Horizontal forward & reverse passes (row by row)
        for y in y0..y1 {
            let r_len = ((x1 - x0) as usize).min(w_buf);
            if r_len == 0 {
                continue;
            }
            for (i, x) in (x0..x1).take(r_len).enumerate() {
                row_buf[i] = self.target.get_pixel(Point::new(x, y));
            }

            // Forward H pass
            let p0 = row_buf[0];
            let (r5, g6, b5) = (p0.r(), p0.g(), p0.b());
            let mut acc_r = (((r5 << 3) | (r5 >> 2)) as i32) << 8;
            let mut acc_g = (((g6 << 2) | (g6 >> 4)) as i32) << 8;
            let mut acc_b = (((b5 << 3) | (b5 >> 2)) as i32) << 8;

            for p in row_buf[..r_len].iter_mut() {
                let (r5, g6, b5) = (p.r(), p.g(), p.b());
                let r8 = ((r5 << 3) | (r5 >> 2)) as i32;
                let g8 = ((g6 << 2) | (g6 >> 4)) as i32;
                let b8 = ((b5 << 3) | (b5 >> 2)) as i32;
                acc_r += (((r8 << 8) - acc_r) * alpha) >> 8;
                acc_g += (((g8 << 8) - acc_g) * alpha) >> 8;
                acc_b += (((b8 << 8) - acc_b) * alpha) >> 8;
                *p = Rgb565::new(
                    ((acc_r >> 8).clamp(0, 255) as u8) >> 3,
                    ((acc_g >> 8).clamp(0, 255) as u8) >> 2,
                    ((acc_b >> 8).clamp(0, 255) as u8) >> 3,
                );
            }

            // Reverse H pass
            let p_last = row_buf[r_len - 1];
            let (r5, g6, b5) = (p_last.r(), p_last.g(), p_last.b());
            let mut acc_r = (((r5 << 3) | (r5 >> 2)) as i32) << 8;
            let mut acc_g = (((g6 << 2) | (g6 >> 4)) as i32) << 8;
            let mut acc_b = (((b5 << 3) | (b5 >> 2)) as i32) << 8;

            for p in row_buf[..r_len].iter_mut().rev() {
                let (r5, g6, b5) = (p.r(), p.g(), p.b());
                let r8 = ((r5 << 3) | (r5 >> 2)) as i32;
                let g8 = ((g6 << 2) | (g6 >> 4)) as i32;
                let b8 = ((b5 << 3) | (b5 >> 2)) as i32;
                acc_r += (((r8 << 8) - acc_r) * alpha) >> 8;
                acc_g += (((g8 << 8) - acc_g) * alpha) >> 8;
                acc_b += (((b8 << 8) - acc_b) * alpha) >> 8;
                *p = Rgb565::new(
                    ((acc_r >> 8).clamp(0, 255) as u8) >> 3,
                    ((acc_g >> 8).clamp(0, 255) as u8) >> 2,
                    ((acc_b >> 8).clamp(0, 255) as u8) >> 3,
                );
            }

            for (i, x) in (x0..x1).take(r_len).enumerate() {
                self.target
                    .draw_iter([Pixel(Point::new(x, y), row_buf[i])])?;
            }
        }

        // Vertical forward & reverse passes (column by column)
        let mut col_buf = [Rgb565::BLACK; 1024];
        let h_buf = ((y1 - y0) as usize).min(1024);

        for x in x0..x1 {
            let c_len = ((y1 - y0) as usize).min(h_buf);
            if c_len == 0 {
                continue;
            }
            for (i, y) in (y0..y1).take(c_len).enumerate() {
                col_buf[i] = self.target.get_pixel(Point::new(x, y));
            }

            // Forward V pass
            let p0 = col_buf[0];
            let (r5, g6, b5) = (p0.r(), p0.g(), p0.b());
            let mut acc_r = (((r5 << 3) | (r5 >> 2)) as i32) << 8;
            let mut acc_g = (((g6 << 2) | (g6 >> 4)) as i32) << 8;
            let mut acc_b = (((b5 << 3) | (b5 >> 2)) as i32) << 8;

            for p in col_buf[..c_len].iter_mut() {
                let (r5, g6, b5) = (p.r(), p.g(), p.b());
                let r8 = ((r5 << 3) | (r5 >> 2)) as i32;
                let g8 = ((g6 << 2) | (g6 >> 4)) as i32;
                let b8 = ((b5 << 3) | (b5 >> 2)) as i32;
                acc_r += (((r8 << 8) - acc_r) * alpha) >> 8;
                acc_g += (((g8 << 8) - acc_g) * alpha) >> 8;
                acc_b += (((b8 << 8) - acc_b) * alpha) >> 8;
                *p = Rgb565::new(
                    ((acc_r >> 8).clamp(0, 255) as u8) >> 3,
                    ((acc_g >> 8).clamp(0, 255) as u8) >> 2,
                    ((acc_b >> 8).clamp(0, 255) as u8) >> 3,
                );
            }

            // Reverse V pass
            let p_last = col_buf[c_len - 1];
            let (r5, g6, b5) = (p_last.r(), p_last.g(), p_last.b());
            let mut acc_r = (((r5 << 3) | (r5 >> 2)) as i32) << 8;
            let mut acc_g = (((g6 << 2) | (g6 >> 4)) as i32) << 8;
            let mut acc_b = (((b5 << 3) | (b5 >> 2)) as i32) << 8;

            for p in col_buf[..c_len].iter_mut().rev() {
                let (r5, g6, b5) = (p.r(), p.g(), p.b());
                let r8 = ((r5 << 3) | (r5 >> 2)) as i32;
                let g8 = ((g6 << 2) | (g6 >> 4)) as i32;
                let b8 = ((b5 << 3) | (b5 >> 2)) as i32;
                acc_r += (((r8 << 8) - acc_r) * alpha) >> 8;
                acc_g += (((g8 << 8) - acc_g) * alpha) >> 8;
                acc_b += (((b8 << 8) - acc_b) * alpha) >> 8;
                *p = Rgb565::new(
                    ((acc_r >> 8).clamp(0, 255) as u8) >> 3,
                    ((acc_g >> 8).clamp(0, 255) as u8) >> 2,
                    ((acc_b >> 8).clamp(0, 255) as u8) >> 3,
                );
            }

            for (i, y) in (y0..y1).take(c_len).enumerate() {
                self.target
                    .draw_iter([Pixel(Point::new(x, y), col_buf[i])])?;
            }
        }

        Ok(())
    }

    /// Apply reverse colour (color inversion) filter on `rect` (PixelRead target).
    pub fn reverse_colour_rect(&mut self, rect: Rect) -> Result<(), D::Error> {
        let bounds = self.clip.intersection(rect);
        if bounds.is_empty() {
            return Ok(());
        }
        let x0 = bounds.x;
        let y0 = bounds.y;
        let x1 = bounds.right();
        let y1 = bounds.bottom();

        for y in y0..y1 {
            for x in x0..x1 {
                let pt = Point::new(x, y);
                let c = self.target.get_pixel(pt);
                let inv = Rgb565::new(31 - c.r(), 63 - c.g(), 31 - c.b());
                self.target.draw_iter([Pixel(pt, inv)])?;
            }
        }
        Ok(())
    }

    /// Fill `rect` using a horizontal 1D line mask array.
    pub fn fill_rect_horizontal_line_mask(
        &mut self,
        rect: Rect,
        mask: &[u8],
        color: Rgb565,
        opacity: u8,
    ) -> Result<(), D::Error> {
        if mask.is_empty() || opacity == 0 {
            return Ok(());
        }
        let bounds = self.clip.intersection(rect);
        if bounds.is_empty() {
            return Ok(());
        }

        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                let mask_x = ((x - rect.x) as usize) % mask.len();
                let alpha = ((mask[mask_x] as u32 * opacity as u32) >> 8) as u8;
                if alpha == 0 {
                    continue;
                }
                let pt = Point::new(x, y);
                let bg = self.target.get_pixel(pt);
                let blended = lerp_rgb565(bg, color, alpha);
                self.target.draw_iter([Pixel(pt, blended)])?;
            }
        }
        Ok(())
    }

    /// Fill `rect` using a vertical 1D line mask array.
    pub fn fill_rect_vertical_line_mask(
        &mut self,
        rect: Rect,
        mask: &[u8],
        color: Rgb565,
        opacity: u8,
    ) -> Result<(), D::Error> {
        if mask.is_empty() || opacity == 0 {
            return Ok(());
        }
        let bounds = self.clip.intersection(rect);
        if bounds.is_empty() {
            return Ok(());
        }

        for y in bounds.y..bounds.bottom() {
            let mask_y = ((y - rect.y) as usize) % mask.len();
            let alpha = ((mask[mask_y] as u32 * opacity as u32) >> 8) as u8;
            if alpha == 0 {
                continue;
            }
            for x in bounds.x..bounds.right() {
                let pt = Point::new(x, y);
                let bg = self.target.get_pixel(pt);
                let blended = lerp_rgb565(bg, color, alpha);
                self.target.draw_iter([Pixel(pt, blended)])?;
            }
        }
        Ok(())
    }
}

impl<'a, D, C> RenderCtx<'a, D, C>
where
    D: DrawTarget<Color = Rgb565>,
    C: Compositor<D>,
{
    pub const fn clip(&self) -> Rect {
        self.clip
    }

    pub fn set_clip(&mut self, clip: Rect) {
        self.clip = clip;
    }

    /// Draws any `embedded_graphics::Drawable` (e.g. an `embedded_text::TextBox`
    /// built via [`crate::interop::text::text_box`], or an arranged
    /// `embedded_layout` view group) onto this context's target, clipped to
    /// the current [`clip`](Self::clip) rect.
    #[cfg(any(
        feature = "embedded-text",
        feature = "embedded-layout",
        feature = "embedded-graphics"
    ))]
    pub fn draw_embedded_graphics<T>(&mut self, drawable: &T) -> Result<T::Output, D::Error>
    where
        T: embedded_graphics::Drawable<Color = Rgb565>,
    {
        use embedded_graphics::draw_target::DrawTargetExt;
        use embedded_graphics::geometry::{Point, Size};
        use embedded_graphics::primitives::Rectangle;

        let clip_rect = Rectangle::new(
            Point::new(self.clip.x, self.clip.y),
            Size::new(self.clip.w, self.clip.h),
        );
        let mut clipped = self.target.clipped(&clip_rect);
        drawable.draw(&mut clipped)
    }

    pub const fn quality(&self) -> RenderQuality {
        self.quality
    }

    pub fn set_quality(&mut self, quality: RenderQuality) {
        self.quality = quality;
    }

    pub const fn backend_caps(&self) -> RenderBackendCaps {
        self.backend_caps
    }

    pub fn set_backend_caps(&mut self, caps: RenderBackendCaps) {
        self.backend_caps = caps;
    }

    pub fn push_transform(&mut self, transform: Transform2D) {
        if self.transform_len >= self.transform_stack.len() {
            return;
        }
        let current = self.current_transform();
        self.transform_stack[self.transform_len] = current.then(transform);
        self.transform_len += 1;
    }

    pub fn pop_transform(&mut self) {
        if self.transform_len > 1 {
            self.transform_len -= 1;
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.push_transform(Transform2D::translation(x, y));
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.push_transform(Transform2D::scale(x, y));
    }

    pub fn rotate(&mut self, deg: f32) {
        self.push_transform(Transform2D::rotation(deg));
    }

    pub fn skew(&mut self, x_deg: f32, y_deg: f32) {
        self.push_transform(Transform2D::skew(x_deg, y_deg));
    }

    pub fn push_layer(&mut self, layer: LayerState) {
        if self.layer_len >= self.layer_stack.len() {
            return;
        }
        let current = self.current_layer();
        self.layer_stack[self.layer_len] = LayerState {
            opacity: ((current.opacity as u16 * layer.opacity as u16) / 255) as u8,
            blend: layer.blend,
            backdrop: layer.backdrop,
        };
        self.layer_len += 1;
    }

    pub fn pop_layer(&mut self) {
        if self.layer_len > 1 {
            self.layer_len -= 1;
        }
    }

    pub const fn shadow_spread_for(&self, spread: u8) -> u8 {
        match self.quality {
            RenderQuality::Low => 0,
            RenderQuality::Medium => {
                if spread > 1 {
                    1
                } else {
                    spread
                }
            }
            RenderQuality::High => spread,
        }
    }

    pub fn fill_rect(&mut self, rect: impl Into<Rect>, color: Rgb565) -> Result<(), D::Error> {
        self.fill_rect_alpha(rect, color, 255)
    }

    pub fn fill_rect_alpha(
        &mut self,
        rect: impl Into<Rect>,
        color: Rgb565,
        opacity: u8,
    ) -> Result<(), D::Error> {
        self.fill_rounded_rect_alpha(rect, 0, color, opacity)
    }

    pub fn fill_rounded_rect(
        &mut self,
        rect: impl Into<Rect>,
        radius: u8,
        color: Rgb565,
    ) -> Result<(), D::Error> {
        self.fill_rounded_rect_alpha(rect, radius, color, 255)
    }

    pub fn fill_rounded_rect_alpha(
        &mut self,
        rect: impl Into<Rect>,
        radius: u8,
        color: Rgb565,
        opacity: u8,
    ) -> Result<(), D::Error> {
        let rect = rect.into();
        let draw = self.visible_rect(rect);
        if draw.is_empty() || opacity == 0 {
            return Ok(());
        }
        let radius = radius.min((rect.w.min(rect.h) / 2) as u8);

        let layer = self.current_layer();
        let combined_opacity = ((opacity as u16 * layer.opacity as u16) / 255) as u8;

        // Fast path for solid un-transformed rectangular fills:
        // Leverages hardware fill_solid on the display target instead of per-pixel loops
        if radius == 0
            && combined_opacity == 255
            && self.current_transform().is_identity()
            && layer.blend == BlendMode::Normal
        {
            let eg_rect = embedded_graphics_core::primitives::Rectangle::new(
                embedded_graphics_core::geometry::Point::new(draw.x, draw.y),
                embedded_graphics_core::geometry::Size::new(draw.w, draw.h),
            );
            return self.target.fill_solid(&eg_rect, color);
        }

        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                if !in_rounded_rect(x, y, rect, radius) {
                    continue;
                }
                self.pixel(x, y, color, opacity)?;
            }
        }
        Ok(())
    }

    pub fn fill_rounded_rect_gradient_alpha(
        &mut self,
        rect: impl Into<Rect>,
        radius: u8,
        gradient: LinearGradient,
        opacity: u8,
    ) -> Result<(), D::Error> {
        let rect = rect.into();
        let draw = self.visible_rect(rect);
        if draw.is_empty() || opacity == 0 {
            return Ok(());
        }
        let radius = radius.min((rect.w.min(rect.h) / 2) as u8);
        let denom = match gradient.direction {
            GradientDirection::Horizontal => rect.w.saturating_sub(1).max(1),
            GradientDirection::Vertical => rect.h.saturating_sub(1).max(1),
        };

        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                if !in_rounded_rect(x, y, rect, radius) {
                    continue;
                }
                let numer = match gradient.direction {
                    GradientDirection::Horizontal => (x - rect.x).max(0) as u32,
                    GradientDirection::Vertical => (y - rect.y).max(0) as u32,
                }
                .min(denom);
                let mut t = ((numer * 255) / denom) as u8;
                t = match self.quality {
                    RenderQuality::Low => 128,
                    RenderQuality::Medium => (t / 64) * 64,
                    RenderQuality::High => t,
                };
                let color = lerp_rgb565(gradient.start, gradient.end, t);
                self.pixel(x, y, color, opacity)?;
            }
        }
        Ok(())
    }

    pub fn stroke_rect(&mut self, rect: impl Into<Rect>, border: Border) -> Result<(), D::Error> {
        self.stroke_rect_alpha(rect, border, 255)
    }

    pub fn stroke_rect_alpha(
        &mut self,
        rect: impl Into<Rect>,
        border: Border,
        opacity: u8,
    ) -> Result<(), D::Error> {
        let rect = rect.into();
        if border.width == 0 || rect.is_empty() {
            return Ok(());
        }

        for i in 0..border.width as i32 {
            let w = rect.w.saturating_sub((i as u32).saturating_mul(2));
            let h = rect.h.saturating_sub((i as u32).saturating_mul(2));
            if w == 0 || h == 0 {
                break;
            }
            let r = Rect::new(rect.x + i, rect.y + i, w, h);
            self.fill_rect_alpha(Rect::new(r.x, r.y, r.w, 1), border.color, opacity)?;
            if r.h > 1 {
                self.fill_rect_alpha(
                    Rect::new(r.x, r.bottom() - 1, r.w, 1),
                    border.color,
                    opacity,
                )?;
            }
            if r.h > 2 {
                self.fill_rect_alpha(Rect::new(r.x, r.y + 1, 1, r.h - 2), border.color, opacity)?;
                if r.w > 1 {
                    self.fill_rect_alpha(
                        Rect::new(r.right() - 1, r.y + 1, 1, r.h - 2),
                        border.color,
                        opacity,
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn stroke_rounded_rect(
        &mut self,
        rect: impl Into<Rect>,
        radius: u8,
        border: Border,
    ) -> Result<(), D::Error> {
        self.stroke_rounded_rect_alpha(rect, radius, border, 255)
    }

    pub fn stroke_rounded_rect_alpha(
        &mut self,
        rect: impl Into<Rect>,
        radius: u8,
        border: Border,
        opacity: u8,
    ) -> Result<(), D::Error> {
        let rect = rect.into();
        if border.width == 0 || rect.is_empty() || opacity == 0 {
            return Ok(());
        }

        let draw = self.visible_rect(rect);
        if draw.is_empty() {
            return Ok(());
        }

        let radius = radius.min((rect.w.min(rect.h) / 2) as u8);
        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                if !in_rounded_rect(x, y, rect, radius) {
                    continue;
                }

                let mut inner_hit = false;
                let mut i = 1u8;
                while i < border.width {
                    let inset = i as i32;
                    let inner = Rect::new(
                        rect.x + inset,
                        rect.y + inset,
                        rect.w.saturating_sub((i as u32) * 2),
                        rect.h.saturating_sub((i as u32) * 2),
                    );
                    let inner_radius = radius.saturating_sub(i);
                    if !inner.is_empty() && in_rounded_rect(x, y, inner, inner_radius) {
                        inner_hit = true;
                        break;
                    }
                    i += 1;
                }

                if !inner_hit {
                    self.pixel(x, y, border.color, opacity)?;
                }
            }
        }
        Ok(())
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Rgb565) -> Result<(), D::Error> {
        self.draw_text_with_font(x, y, text, color, FontId::Tiny3x5)
    }

    pub fn draw_text_with_font(
        &mut self,
        x: i32,
        y: i32,
        text: &str,
        color: Rgb565,
        font: impl Into<FontId>,
    ) -> Result<(), D::Error> {
        let font = font.into();
        let advance = font.advance() as i32;
        let line_h = font.line_height() as i32;
        let mut cursor_x = x;
        let mut cursor_y = y;
        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = x;
                cursor_y += line_h;
                continue;
            }
            self.draw_char_with_font(cursor_x, cursor_y, ch, color, 255, font)?;
            cursor_x += advance;
        }
        Ok(())
    }

    pub fn draw_text_in(
        &mut self,
        rect: impl Into<Rect>,
        text: &str,
        style: TextStyle,
    ) -> Result<(), D::Error> {
        self.draw_text_in_with_font(rect, text, style, style.font)
    }

    pub fn draw_text_shaped_in<S, const N: usize>(
        &mut self,
        rect: Rect,
        text: &str,
        style: TextStyle,
        shaper: &S,
        config: crate::text::ShapingConfig,
    ) -> Result<(), D::Error>
    where
        S: crate::text::TextShaper,
    {
        if rect.is_empty() {
            return Ok(());
        }
        let mut shaped = heapless::Vec::<crate::text::ShapedGlyph, N>::new();
        shaper.shape(text, config, &mut shaped);
        if shaped.is_empty() {
            return Ok(());
        }
        let mut x = rect.x;
        let y = rect.y + rect.h.saturating_sub(style.font.line_height()) as i32 / 2;
        for glyph in shaped {
            self.draw_char_with_font(x, y, glyph.ch, style.color, style.opacity, style.font)?;
            x += (glyph.x_advance as i32).max(1) * style.font.advance() as i32;
            if x >= rect.right() {
                break;
            }
        }
        Ok(())
    }

    pub fn draw_text_in_with_font(
        &mut self,
        rect: impl Into<Rect>,
        text: &str,
        style: TextStyle,
        font: impl Into<FontId>,
    ) -> Result<(), D::Error> {
        let rect = rect.into();
        let font = font.into();
        if rect.is_empty() {
            return Ok(());
        }

        let advance = font.advance();
        let line_h = font.line_height();
        let max_chars = (rect.w / advance).max(1) as usize;
        let char_count = text.chars().count();
        let line_count = count_lines(text, max_chars, style.wrap).max(1);
        let line_step = line_h + style.line_spacing as u32;
        let total_h = line_count as u32 * line_h
            + line_count.saturating_sub(1) as u32 * style.line_spacing as u32;
        let mut y = match style.vertical_align {
            VerticalAlign::Top => rect.y,
            VerticalAlign::Middle => rect.y + rect.h.saturating_sub(total_h) as i32 / 2,
            VerticalAlign::Bottom => rect.y + rect.h.saturating_sub(total_h) as i32,
        };

        let mut start = 0;
        let mut rendered_lines = 0u8;
        let max_lines = match style.overflow_policy {
            TextOverflowPolicy::WrapThenEllipsis { max_lines } => max_lines.max(1),
            TextOverflowPolicy::Global(_) => style.max_lines.unwrap_or(u8::MAX),
        };
        while start < char_count {
            if rendered_lines >= max_lines {
                break;
            }
            let (len, consumed_newline) = line_len_at(text, start, max_chars, style.wrap);
            let mut draw_len = len;
            let is_last_allowed_line = rendered_lines.saturating_add(1) >= max_lines;
            let use_ellipsis = match style.overflow_policy {
                TextOverflowPolicy::WrapThenEllipsis { .. } => is_last_allowed_line,
                TextOverflowPolicy::Global(mode) => mode == TextOverflow::Ellipsis,
            };
            if use_ellipsis
                && ((!consumed_newline && start + len < char_count) || is_last_allowed_line)
            {
                let ellipsis_width = match style.ellipsis {
                    EllipsisMode::ThreeDots => 3usize,
                    EllipsisMode::SingleGlyph => 1usize,
                };
                if len > ellipsis_width {
                    draw_len = len - ellipsis_width;
                }
            }
            let line_w = self.substring_width(text, start, draw_len, font, style.kerning);
            let x = match style.align {
                TextAlign::Left => rect.x,
                TextAlign::Center => rect.x + rect.w.saturating_sub(line_w) as i32 / 2,
                TextAlign::Right => rect.x + rect.w.saturating_sub(line_w) as i32,
            };
            self.draw_chars_with_font(
                x,
                y,
                text,
                start,
                draw_len,
                style.color,
                style.opacity,
                font,
                style.kerning,
            )?;
            if draw_len < len && use_ellipsis {
                let token = match style.ellipsis {
                    EllipsisMode::ThreeDots => "...",
                    EllipsisMode::SingleGlyph => ".",
                };
                self.draw_text_with_font(x + line_w as i32, y, token, style.color, font)?;
            }
            y += line_step as i32;
            rendered_lines = rendered_lines.saturating_add(1);
            start += len + usize::from(consumed_newline);
            if style.wrap == TextWrap::Word && start < char_count {
                while text.chars().nth(start).is_some_and(|ch| ch == ' ') {
                    start += 1;
                }
            }
            if len == 0 && !consumed_newline {
                break;
            }
        }

        Ok(())
    }

    pub fn draw_line_in(&mut self, rect: Rect, line: text::Line<'_>) -> Result<(), D::Error> {
        if rect.is_empty() {
            return Ok(());
        }

        self.draw_line_segment_in(rect, line, 0, line.width_chars())
    }

    pub fn draw_line(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: Rgb565,
    ) -> Result<(), D::Error> {
        self.draw_line_styled(x0, y0, x1, y1, StrokeStyle::new(color))
    }

    pub fn draw_line_styled(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        style: StrokeStyle,
    ) -> Result<(), D::Error> {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let half = (style.width as i32 / 2).max(0);
        let opacity = self.stroke_opacity(style);

        loop {
            for oy in -half..=half {
                for ox in -half..=half {
                    self.pixel(x + ox, y + oy, style.color, opacity)?;
                }
            }
            if style.cap == StrokeCap::Round {
                self.fill_circle(x0, y0, half.max(1) as u32, style.color)?;
                self.fill_circle(x1, y1, half.max(1) as u32, style.color)?;
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
        Ok(())
    }

    pub fn fill_circle(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: u32,
        color: Rgb565,
    ) -> Result<(), D::Error> {
        let radius = radius as i32;
        if radius <= 0 {
            return Ok(());
        }
        let r_sq = radius * radius;
        for dy in -radius..=radius {
            let dx = ((r_sq - dy * dy) as f32).sqrt() as i32;
            if dx >= 0 {
                let w = (dx * 2 + 1) as u32;
                self.fill_rect(Rect::new(center_x - dx, center_y + dy, w, 1), color)?;
            }
        }
        Ok(())
    }

    pub fn stroke_circle(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: u32,
        color: Rgb565,
    ) -> Result<(), D::Error> {
        let radius = radius as i32;
        if radius <= 0 {
            return Ok(());
        }
        let mut x = radius;
        let mut y = 0;
        let mut err = 1 - x;
        while x >= y {
            self.pixel(center_x + x, center_y + y, color, 255)?;
            self.pixel(center_x + y, center_y + x, color, 255)?;
            self.pixel(center_x - y, center_y + x, color, 255)?;
            self.pixel(center_x - x, center_y + y, color, 255)?;
            self.pixel(center_x - x, center_y - y, color, 255)?;
            self.pixel(center_x - y, center_y - x, color, 255)?;
            self.pixel(center_x + y, center_y - x, color, 255)?;
            self.pixel(center_x + x, center_y - y, color, 255)?;
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
        Ok(())
    }

    pub fn stroke_arc(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: u32,
        start_deg: i32,
        end_deg: i32,
        color: Rgb565,
    ) -> Result<(), D::Error> {
        self.stroke_arc_styled(
            center_x,
            center_y,
            radius,
            start_deg,
            end_deg,
            StrokeStyle::new(color),
        )
    }

    pub fn stroke_arc_styled(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: u32,
        start_deg: i32,
        end_deg: i32,
        style: StrokeStyle,
    ) -> Result<(), D::Error> {
        let mut start = start_deg;
        let mut end = end_deg;
        if end < start {
            core::mem::swap(&mut start, &mut end);
        }
        let mut deg = start;
        let step = match self.quality {
            RenderQuality::Low => 8,
            RenderQuality::Medium => 4,
            RenderQuality::High => 2,
        };
        while deg <= end {
            let rad = (deg as f32).to_radians();
            let x = center_x + (radius as f32 * rad.cos()) as i32;
            let y = center_y + (radius as f32 * rad.sin()) as i32;
            let half = (style.width as i32 / 2).max(0);
            let opacity = self.stroke_opacity(style);
            for oy in -half..=half {
                for ox in -half..=half {
                    self.pixel(x + ox, y + oy, style.color, opacity)?;
                }
            }
            if style.join == StrokeJoin::Round {
                self.fill_circle(x, y, half.max(1) as u32, style.color)?;
            }
            deg += step;
        }
        Ok(())
    }

    /// Fill a sector ("pie slice") using a start angle and sweep angle in degrees.
    ///
    /// Positive sweep draws counterclockwise, negative sweep clockwise.
    pub fn fill_sector_sweep(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: u32,
        start_deg: f32,
        sweep_deg: f32,
        color: Rgb565,
    ) -> Result<(), D::Error> {
        if radius == 0 {
            return Ok(());
        }

        let draw = self.visible_rect(Rect::new(
            center_x - radius as i32,
            center_y - radius as i32,
            radius.saturating_mul(2).saturating_add(1),
            radius.saturating_mul(2).saturating_add(1),
        ));
        if draw.is_empty() {
            return Ok(());
        }

        let max_sweep = sweep_deg.abs().min(360.0);
        if max_sweep <= 0.0 {
            return Ok(());
        }

        let rr = (radius as i32) * (radius as i32);
        let start = normalize_angle_deg(start_deg);
        let ccw = sweep_deg >= 0.0;

        // The sector is the arc of length `max_sweep`, in degrees, that
        // starts at `lo_deg` and ends at `hi_deg` (both expressed in the
        // same increasing-angle direction `atan2` would report). Reduce
        // this to two boundary direction vectors so the per-pixel test is
        // a couple of multiply-subtracts instead of an `atan2` + degrees
        // conversion for every pixel in the circle -- `atan2` is a
        // software-emulated call on MCUs without a hardware FPU trig unit,
        // and this loop used to run it for every pixel inside the radius,
        // every frame.
        let (lo_deg, hi_deg) = if ccw {
            (start, start + max_sweep)
        } else {
            (start - max_sweep, start)
        };
        let (lo_c, lo_s) = cardinal_unit(lo_deg)
            .unwrap_or_else(|| (lo_deg.to_radians().cos(), lo_deg.to_radians().sin()));
        let (hi_c, hi_s) = cardinal_unit(hi_deg)
            .unwrap_or_else(|| (hi_deg.to_radians().cos(), hi_deg.to_radians().sin()));
        // A sweep over half a circle or less is a convex wedge, testable
        // directly with two half-plane (cross-product) checks. A sweep
        // past 180 degrees is non-convex, but its complement (the
        // untouched slice) is convex and always < 180 degrees, so test
        // for exclusion from that instead.
        let full_circle = max_sweep >= 360.0;
        let reflex = max_sweep > 180.0;

        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                let dx = x - center_x;
                let dy = y - center_y;
                let d2 = dx * dx + dy * dy;
                if d2 > rr {
                    continue;
                }

                let in_sweep = if full_circle {
                    true
                } else {
                    let (fx, fy) = (dx as f32, dy as f32);
                    if !reflex {
                        cross(lo_c, lo_s, fx, fy) >= 0.0 && cross(fx, fy, hi_c, hi_s) >= 0.0
                    } else {
                        !(cross(hi_c, hi_s, fx, fy) >= 0.0 && cross(fx, fy, lo_c, lo_s) >= 0.0)
                    }
                };
                if in_sweep {
                    self.pixel(x, y, color, 255)?;
                }
            }
        }
        Ok(())
    }

    pub fn fill_polygon(&mut self, points: &[Point], color: Rgb565) -> Result<(), D::Error> {
        if points.len() < 3 {
            return Ok(());
        }
        let min_y = points.iter().map(|p| p.y).min().unwrap_or(0);
        let max_y = points.iter().map(|p| p.y).max().unwrap_or(-1);
        for y in min_y..=max_y {
            let mut intersections = [i32::MIN; 16];
            let mut count = 0usize;
            for i in 0..points.len() {
                let p1 = points[i];
                let p2 = points[(i + 1) % points.len()];
                let (y1, y2) = if p1.y <= p2.y {
                    (p1.y, p2.y)
                } else {
                    (p2.y, p1.y)
                };
                if y < y1 || y >= y2 || y1 == y2 {
                    continue;
                }
                if count >= intersections.len() {
                    break;
                }
                let x = p1.x + ((y - p1.y) * (p2.x - p1.x)) / (p2.y - p1.y);
                intersections[count] = x;
                count += 1;
            }
            intersections[..count].sort_unstable();
            let mut i = 0;
            while i + 1 < count {
                let x0 = intersections[i];
                let x1 = intersections[i + 1];
                for x in x0..=x1 {
                    self.pixel(x, y, color, 255)?;
                }
                i += 2;
            }
        }
        Ok(())
    }

    pub fn draw_image(
        &mut self,
        rect: Rect,
        image: ImageRef<'_>,
        fit: ImageFit,
    ) -> Result<(), D::Error> {
        self.draw_image_region(rect, image, fit, Rect::new(0, 0, image.width, image.height))
    }

    pub fn draw_image_region(
        &mut self,
        rect: Rect,
        image: ImageRef<'_>,
        fit: ImageFit,
        src_rect: Rect,
    ) -> Result<(), D::Error> {
        let bounds = image.bounds_at(rect, fit);
        if bounds.is_empty() || image.width == 0 || image.height == 0 {
            return Ok(());
        }
        let src_w = image.width as usize;
        for y in 0..bounds.h {
            let src_y = match fit {
                ImageFit::Stretch => {
                    src_rect.y.max(0) as usize
                        + ((y as u64 * src_rect.h as u64) / bounds.h as u64) as usize
                }
                ImageFit::Center => src_rect.y.max(0) as usize + y as usize,
            };
            for x in 0..bounds.w {
                let src_x = match fit {
                    ImageFit::Stretch => {
                        src_rect.x.max(0) as usize
                            + ((x as u64 * src_rect.w as u64) / bounds.w as u64) as usize
                    }
                    ImageFit::Center => src_rect.x.max(0) as usize + x as usize,
                };
                let idx = src_y.saturating_mul(src_w).saturating_add(src_x);
                if let Some(raw) = image.pixels.get(idx) {
                    let color = Rgb565::new(
                        ((raw >> 11) & 0x1F) as u8,
                        ((raw >> 5) & 0x3F) as u8,
                        (raw & 0x1F) as u8,
                    );
                    self.pixel(bounds.x + x as i32, bounds.y + y as i32, color, 255)?;
                }
            }
        }
        Ok(())
    }

    pub fn draw_image_transformed(
        &mut self,
        rect: Rect,
        image: ImageRef<'_>,
        scale: f32,
        rotation_deg: f32,
    ) -> Result<(), D::Error> {
        if rect.is_empty() || image.width == 0 || image.height == 0 || scale <= 0.0 {
            return Ok(());
        }
        let cx = rect.x + rect.w as i32 / 2;
        let cy = rect.y + rect.h as i32 / 2;
        let rad = rotation_deg.to_radians();
        let cos_r = rad.cos();
        let sin_r = rad.sin();
        let src_w = image.width as usize;
        let src_cx = image.width as f32 / 2.0;
        let src_cy = image.height as f32 / 2.0;
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                let dx = (x - cx) as f32 / scale;
                let dy = (y - cy) as f32 / scale;
                let sx = cos_r * dx + sin_r * dy + src_cx;
                let sy = -sin_r * dx + cos_r * dy + src_cy;
                if sx < 0.0 || sy < 0.0 || sx >= image.width as f32 || sy >= image.height as f32 {
                    continue;
                }
                let idx = (sy as usize)
                    .saturating_mul(src_w)
                    .saturating_add(sx as usize);
                if let Some(raw) = image.pixels.get(idx) {
                    let color = Rgb565::new(
                        ((raw >> 11) & 0x1F) as u8,
                        ((raw >> 5) & 0x3F) as u8,
                        (raw & 0x1F) as u8,
                    );
                    self.pixel(x, y, color, 255)?;
                }
            }
        }
        Ok(())
    }

    pub fn fill_rect_masked(
        &mut self,
        rect: Rect,
        color: Rgb565,
        mask: fn(i32, i32) -> bool,
    ) -> Result<(), D::Error> {
        let draw = self.visible_rect(rect);
        if draw.is_empty() {
            return Ok(());
        }
        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                if mask(x, y) {
                    self.pixel(x, y, color, 255)?;
                }
            }
        }
        Ok(())
    }

    pub fn draw_text_model_in(&mut self, rect: Rect, text: text::Text<'_>) -> Result<(), D::Error> {
        if rect.is_empty() || text.lines.is_empty() {
            return Ok(());
        }

        let metrics = text.metrics(rect.w);
        let max_line_height = text
            .lines
            .iter()
            .map(|line| line.max_line_height())
            .max()
            .unwrap_or(CHAR_HEIGHT);
        let line_step = max_line_height + text.line_spacing as u32;
        let mut y = match text.vertical_align {
            VerticalAlign::Top => rect.y,
            VerticalAlign::Middle => rect.y + rect.h.saturating_sub(metrics.height) as i32 / 2,
            VerticalAlign::Bottom => rect.y + rect.h.saturating_sub(metrics.height) as i32,
        };
        for line in text.lines {
            let align = if line.align == TextAlign::Left {
                text.align
            } else {
                line.align
            };
            let line = text::Line { align, ..*line };

            let mut start = 0;
            let char_count = line.char_count();
            if char_count == 0 {
                y += line_step as i32;
                continue;
            }
            while start < char_count {
                if y >= rect.bottom() {
                    return Ok(());
                }
                let (len, consumed_newline) = line.segment_len_at(start, rect.w, text.wrap);
                self.draw_line_segment_in(
                    Rect::new(rect.x, y, rect.w, max_line_height),
                    line,
                    start,
                    len,
                )?;
                y += line_step as i32;
                start += len + usize::from(consumed_newline);
                if len == 0 && !consumed_newline {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn text_metrics(text: &str) -> TextMetrics {
        Self::text_metrics_with_font(text, FontId::Tiny3x5)
    }

    pub fn text_metrics_with_font(text: &str, font: impl Into<FontId>) -> TextMetrics {
        let font = font.into();
        TextMetrics {
            width: text.chars().count() as u32 * font.advance(),
            height: font.line_height(),
        }
    }

    pub fn text_metrics_wrapped(text: &str, max_width: u32, wrap: TextWrap) -> TextMetrics {
        Self::text_metrics_wrapped_with_font(text, max_width, wrap, FontId::Tiny3x5)
    }

    pub fn text_metrics_wrapped_with_font(
        text: &str,
        max_width: u32,
        wrap: TextWrap,
        font: impl Into<FontId>,
    ) -> TextMetrics {
        let font = font.into();
        let max_chars = (max_width / font.advance()).max(1) as usize;
        let lines = count_lines(text, max_chars, wrap).max(1);
        let widest = widest_line(text, max_chars, wrap) as u32 * font.advance();
        TextMetrics {
            width: widest.min(max_width),
            height: lines as u32 * font.line_height() + lines.saturating_sub(1) as u32,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_chars_with_font(
        &mut self,
        x: i32,
        y: i32,
        text: &str,
        start: usize,
        len: usize,
        color: Rgb565,
        opacity: u8,
        font: FontId,
        kerning: bool,
    ) -> Result<(), D::Error> {
        let advance = font.advance() as i32;
        let mut cursor_x = x;
        let mut prev: Option<char> = None;
        for ch in text.chars().skip(start).take(len) {
            self.draw_char_with_font(cursor_x, y, ch, color, opacity, font)?;
            cursor_x += advance + kerning_adjust(prev, ch, kerning);
            prev = Some(ch);
        }
        Ok(())
    }

    fn substring_width(
        &self,
        text: &str,
        start: usize,
        len: usize,
        font: FontId,
        kerning: bool,
    ) -> u32 {
        let mut width = 0u32;
        let mut prev = None;
        for ch in text.chars().skip(start).take(len) {
            width = width.saturating_add(font.advance());
            let adjust = kerning_adjust(prev, ch, kerning);
            if adjust < 0 {
                width = width.saturating_sub((-adjust) as u32);
            } else {
                width = width.saturating_add(adjust as u32);
            }
            prev = Some(ch);
        }
        width
    }

    fn draw_line_segment_in(
        &mut self,
        rect: Rect,
        line: text::Line<'_>,
        start: usize,
        len: usize,
    ) -> Result<(), D::Error> {
        if rect.is_empty() || len == 0 {
            return Ok(());
        }

        let line_w = self.line_segment_width(line, start, len);
        let x = match line.align {
            TextAlign::Left => rect.x,
            TextAlign::Center => rect.x + rect.w.saturating_sub(line_w) as i32 / 2,
            TextAlign::Right => rect.x + rect.w.saturating_sub(line_w) as i32,
        };

        let old_clip = self.clip;
        self.clip = self.clip.intersection(rect);
        let result = self.draw_span_chars(x, rect.y, line, start, len);
        self.clip = old_clip;
        result
    }

    fn draw_span_chars(
        &mut self,
        x: i32,
        y: i32,
        line: text::Line<'_>,
        start: usize,
        len: usize,
    ) -> Result<(), D::Error> {
        let mut cursor_x = x;
        for (idx, (ch, style)) in line
            .spans
            .iter()
            .flat_map(|span| span.content.chars().map(move |ch| (ch, span.style)))
            .enumerate()
        {
            if idx < start {
                continue;
            }
            if idx >= start + len {
                break;
            }
            if ch != '\n' {
                self.draw_char_with_font(cursor_x, y, ch, style.color, 255, style.font)?;
                cursor_x += style.font.advance() as i32;
            }
        }
        Ok(())
    }

    fn line_segment_width(&self, line: text::Line<'_>, start: usize, len: usize) -> u32 {
        line.spans
            .iter()
            .flat_map(|span| span.content.chars().map(move |ch| (ch, span.style.font)))
            .enumerate()
            .filter_map(|(idx, (ch, font))| {
                if idx < start || idx >= start + len || ch == '\n' {
                    None
                } else {
                    Some(font.advance())
                }
            })
            .sum()
    }

    fn draw_char_with_font(
        &mut self,
        x: i32,
        y: i32,
        ch: char,
        color: Rgb565,
        opacity: u8,
        font: FontId,
    ) -> Result<(), D::Error> {
        let glyph = glyph_rows(font, ch);
        let layer = self.current_layer();
        let fast_spans = opacity == 255
            && layer.opacity == 255
            && self.current_transform().is_identity()
            && layer.blend == BlendMode::Normal;

        match font {
            FontId::Tiny3x5 | FontId::Medium4x7 | FontId::Custom(_) => {
                for (row, bits) in glyph.iter().enumerate() {
                    let ry = y + row as i32;
                    if fast_spans && *bits == 0b111 {
                        self.fill_rect(Rect::new(x, ry, 3, 1), color)?;
                    } else if fast_spans && *bits == 0b110 {
                        self.fill_rect(Rect::new(x, ry, 2, 1), color)?;
                    } else if fast_spans && *bits == 0b011 {
                        self.fill_rect(Rect::new(x + 1, ry, 2, 1), color)?;
                    } else {
                        for col in 0..3 {
                            if bits & (1 << (2 - col)) != 0 {
                                self.pixel(x + col, ry, color, opacity)?;
                            }
                        }
                    }
                }
            }
            FontId::Scaled6x10 => {
                for (row, bits) in glyph.iter().enumerate() {
                    for col in 0..3 {
                        if bits & (1 << (2 - col)) != 0 {
                            let px = x + (col * 2);
                            let py = y + (row as i32 * 2);
                            self.pixel(px, py, color, opacity)?;
                            self.pixel(px + 1, py, color, opacity)?;
                            self.pixel(px, py + 1, color, opacity)?;
                            self.pixel(px + 1, py + 1, color, opacity)?;
                        }
                    }
                }
            }
            FontId::Vector(scale) => {
                let glyph = crate::font::get_vector_glyph(ch);
                let mut last_point: Option<(i32, i32)> = None;
                let scale_f = scale as f32;
                for &(px, py) in glyph {
                    if px == 0xFF && py == 0xFF {
                        last_point = None;
                        continue;
                    }
                    let draw_x = x + (px as f32 * scale_f) as i32;
                    let draw_y = y + (py as f32 * scale_f) as i32;
                    if let Some((lx, ly)) = last_point {
                        self.draw_line_styled(
                            lx,
                            ly,
                            draw_x,
                            draw_y,
                            StrokeStyle::new(color).with_width(1).with_antialias(true),
                        )?;
                    }
                    last_point = Some((draw_x, draw_y));
                }
            }
            #[cfg(feature = "embedded-graphics")]
            FontId::MonoFont(font) => {
                use embedded_graphics::Drawable;
                use embedded_graphics::draw_target::DrawTarget;
                use embedded_graphics::geometry::{OriginDimensions, Point, Size};
                use embedded_graphics::mono_font::MonoTextStyle;
                use embedded_graphics::pixelcolor::BinaryColor;
                use embedded_graphics::text::Text;

                struct GlyphPixelCollector<'a, F> {
                    x: i32,
                    y: i32,
                    f: &'a mut F,
                }

                impl<F> OriginDimensions for GlyphPixelCollector<'_, F> {
                    fn size(&self) -> Size {
                        Size::new(u32::MAX, u32::MAX)
                    }
                }

                impl<F: FnMut(i32, i32)> DrawTarget for GlyphPixelCollector<'_, F> {
                    type Color = BinaryColor;
                    type Error = core::convert::Infallible;

                    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
                    where
                        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
                    {
                        for embedded_graphics::Pixel(pos, color) in pixels {
                            if color.is_on() {
                                (self.f)(self.x + pos.x, self.y + pos.y);
                            }
                        }
                        Ok(())
                    }
                }

                let mut collector_err = Ok(());
                let mut pixel_cb = |px: i32, py: i32| {
                    if collector_err.is_ok() {
                        if let Err(e) = self.pixel(px, py, color, opacity) {
                            collector_err = Err(e);
                        }
                    }
                };

                let mut collector = GlyphPixelCollector {
                    x,
                    y,
                    f: &mut pixel_cb,
                };

                let text_style = MonoTextStyle::new(font, BinaryColor::On);
                let mut buf = [0u8; 4];
                let ch_str = ch.encode_utf8(&mut buf);
                let _ = Text::new(ch_str, Point::zero(), text_style).draw(&mut collector);
                collector_err?;
            }
        }
        Ok(())
    }

    fn pixel(&mut self, x: i32, y: i32, color: Rgb565, opacity: u8) -> Result<(), D::Error> {
        let (x, y) = self.current_transform().apply(x, y);
        if !self.clip.contains(x, y) {
            return Ok(());
        }
        if let Some(dirty) = self.dirty {
            if !dirty.contains(x, y) {
                return Ok(());
            }
        }
        let layer = self.current_layer();
        let combined_opacity = ((opacity as u16 * layer.opacity as u16) / 255) as u8;
        // The compositor policy (`Dither` vs `Blend`) decides how the pixel
        // lands: ordered dither for write-only targets, true alpha blend for
        // readback-capable ones. Zero-cost — resolved by `C` at monomorphization.
        C::plot(
            self.target,
            x,
            y,
            color,
            combined_opacity,
            layer.blend,
            layer.backdrop,
        )
    }

    fn visible_rect(&self, rect: Rect) -> Rect {
        let mut draw = rect.intersection(self.clip);
        if let Some(dirty) = self.dirty {
            draw = draw.intersection(dirty);
        }
        draw
    }

    fn current_transform(&self) -> Transform2D {
        self.transform_stack[self.transform_len - 1]
    }

    fn current_layer(&self) -> LayerState {
        self.layer_stack[self.layer_len - 1]
    }

    fn stroke_opacity(&self, style: StrokeStyle) -> u8 {
        if !style.antialias || matches!(style.antialias_mode, AntiAliasMode::None) {
            return 255;
        }
        match style.antialias_mode {
            AntiAliasMode::None => 255,
            AntiAliasMode::Coverage => match self.quality {
                RenderQuality::Low => 96,
                RenderQuality::Medium => 160,
                RenderQuality::High => 220,
            },
            AntiAliasMode::Subpixel => {
                if self.backend_caps.supports_subpixel {
                    match self.quality {
                        RenderQuality::Low => 128,
                        RenderQuality::Medium => 192,
                        RenderQuality::High => 240,
                    }
                } else {
                    match self.quality {
                        RenderQuality::Low => 96,
                        RenderQuality::Medium => 160,
                        RenderQuality::High => 220,
                    }
                }
            }
        }
    }
}

impl<'a, D, C> RenderCtx<'a, D, C>
where
    D: DrawTarget<Color = Rgb565> + PixelRead,
    C: Compositor<D>,
{
    /// Alpha-composite `color` over whatever is already at `(x, y)` in the
    /// destination, using true per-pixel blending (`lerp_rgb565`) rather
    /// than the dithered approximation `pixel()` uses.
    fn pixel_blended(&mut self, x: i32, y: i32, color: Rgb565, alpha: u8) -> Result<(), D::Error> {
        let (x, y) = self.current_transform().apply(x, y);
        if !self.clip.contains(x, y) {
            return Ok(());
        }
        if let Some(dirty) = self.dirty {
            if !dirty.contains(x, y) {
                return Ok(());
            }
        }
        let layer = self.current_layer();
        let combined_alpha = ((alpha as u16 * layer.opacity as u16) / 255) as u8;
        if combined_alpha == 0 {
            return Ok(());
        }
        let backdrop = self.target.get_pixel(Point::new(x, y));
        let blended = lerp_rgb565(backdrop, color, combined_alpha);
        let blended = apply_blend_mode(blended, layer.blend, layer.backdrop);
        self.target.draw_iter([Pixel(Point::new(x, y), blended)])
    }

    /// Like [`RenderCtx::fill_rect_alpha`], but alpha-composites against the
    /// destination's real current pixels instead of dithering.
    pub fn fill_rect_true_alpha(
        &mut self,
        rect: Rect,
        color: Rgb565,
        alpha: u8,
    ) -> Result<(), D::Error> {
        self.fill_rounded_rect_true_alpha(rect, 0, color, alpha)
    }

    /// Like [`RenderCtx::fill_rounded_rect_alpha`], but alpha-composites
    /// against the destination's real current pixels instead of dithering.
    pub fn fill_rounded_rect_true_alpha(
        &mut self,
        rect: Rect,
        radius: u8,
        color: Rgb565,
        alpha: u8,
    ) -> Result<(), D::Error> {
        let draw = self.visible_rect(rect);
        if draw.is_empty() || alpha == 0 {
            return Ok(());
        }
        let radius = radius.min((rect.w.min(rect.h) / 2) as u8);

        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                if !in_rounded_rect(x, y, rect, radius) {
                    continue;
                }
                self.pixel_blended(x, y, color, alpha)?;
            }
        }
        Ok(())
    }

    /// Fill a rectangle with an 8-bit alpha mask and solid color.
    pub fn fill_rect_alpha_mask(
        &mut self,
        rect: Rect,
        mask: &[u8],
        mask_stride: usize,
        color: Rgb565,
        opacity: u8,
    ) -> Result<(), D::Error> {
        let draw = self.visible_rect(rect);
        if draw.is_empty() || opacity == 0 || mask_stride == 0 {
            return Ok(());
        }
        for y in draw.y..draw.bottom() {
            let my = (y - rect.y) as usize;
            for x in draw.x..draw.right() {
                let mx = (x - rect.x) as usize;
                let idx = my * mask_stride + mx;
                if let Some(&m_val) = mask.get(idx) {
                    if m_val > 0 {
                        let pix_opacity = ((m_val as u16 * opacity as u16) / 255) as u8;
                        self.pixel(x, y, color, pix_opacity)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Fill a rounded rectangle with an [`AlphaLinearGradient`].
    pub fn fill_rounded_rect_alpha_gradient(
        &mut self,
        rect: Rect,
        radius: u8,
        gradient: &AlphaLinearGradient,
        opacity: u8,
    ) -> Result<(), D::Error> {
        let draw = self.visible_rect(rect);
        if draw.is_empty() || opacity == 0 {
            return Ok(());
        }
        let radius = radius.min((rect.w.min(rect.h) / 2) as u8);
        let denom = match gradient.direction {
            GradientDirection::Horizontal => rect.w.saturating_sub(1).max(1),
            GradientDirection::Vertical => rect.h.saturating_sub(1).max(1),
        };

        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                if !in_rounded_rect(x, y, rect, radius) {
                    continue;
                }
                let numer = match gradient.direction {
                    GradientDirection::Horizontal => (x - rect.x).max(0) as u32,
                    GradientDirection::Vertical => (y - rect.y).max(0) as u32,
                }
                .min(denom);
                let t = ((numer * 255) / denom) as u8;
                let (color, grad_alpha) = gradient.sample(t);
                let combined_alpha = ((grad_alpha as u16 * opacity as u16) / 255) as u8;
                self.pixel(x, y, color, combined_alpha)?;
            }
        }
        Ok(())
    }

    /// Fill a rounded rectangle with an [`AlphaRadialGradient`].
    pub fn fill_rounded_rect_radial_gradient(
        &mut self,
        rect: Rect,
        radius: u8,
        gradient: &AlphaRadialGradient,
        opacity: u8,
    ) -> Result<(), D::Error> {
        let draw = self.visible_rect(rect);
        if draw.is_empty() || opacity == 0 {
            return Ok(());
        }
        let radius = radius.min((rect.w.min(rect.h) / 2) as u8);
        let cx = rect.x as f32 + rect.w as f32 * gradient.center_x;
        let cy = rect.y as f32 + rect.h as f32 * gradient.center_y;

        for y in draw.y..draw.bottom() {
            let dy = y as f32 - cy;
            for x in draw.x..draw.right() {
                if !in_rounded_rect(x, y, rect, radius) {
                    continue;
                }
                let dx = x as f32 - cx;
                let dist = (dx * dx + dy * dy).sqrt();
                let (color, grad_alpha) = gradient.sample_at_dist(dist);
                let combined_alpha = ((grad_alpha as u16 * opacity as u16) / 255) as u8;
                self.pixel(x, y, color, combined_alpha)?;
            }
        }
        Ok(())
    }

    /// Render a soft drop shadow around a rounded rectangle.
    pub fn draw_drop_shadow(
        &mut self,
        rect: Rect,
        _radius: u8,
        shadow_color: Rgb565,
        shadow_opacity: u8,
        shadow_spread: u8,
        blur_radius: u8,
    ) -> Result<(), D::Error> {
        if shadow_opacity == 0 {
            return Ok(());
        }
        let margin = (shadow_spread as i32) + (blur_radius as i32);
        let shadow_rect = Rect::new(
            rect.x - margin,
            rect.y - margin,
            rect.w + (margin as u32 * 2),
            rect.h + (margin as u32 * 2),
        );
        let draw = self.visible_rect(shadow_rect);
        if draw.is_empty() {
            return Ok(());
        }

        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                let dx = if x < rect.x {
                    rect.x - x
                } else if x >= rect.right() {
                    x - rect.right() + 1
                } else {
                    0
                };
                let dy = if y < rect.y {
                    rect.y - y
                } else if y >= rect.bottom() {
                    y - rect.bottom() + 1
                } else {
                    0
                };

                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > margin as f32 {
                    continue;
                }

                let factor = (1.0 - (dist / (margin as f32 + 1.0))).clamp(0.0, 1.0);
                let alpha = (shadow_opacity as f32 * factor) as u8;
                if alpha > 0 {
                    self.pixel(x, y, shadow_color, alpha)?;
                }
            }
        }
        Ok(())
    }

    /// Draw a UI card fill with an alpha gradient background and soft drop shadow.
    pub fn draw_card_fill(
        &mut self,
        rect: Rect,
        radius: u8,
        bg_gradient: &AlphaLinearGradient,
        shadow_color: Rgb565,
        shadow_opacity: u8,
        blur_radius: u8,
    ) -> Result<(), D::Error> {
        if shadow_opacity > 0 && blur_radius > 0 {
            self.draw_drop_shadow(rect, radius, shadow_color, shadow_opacity, 2, blur_radius)?;
        }
        self.fill_rounded_rect_alpha_gradient(rect, radius, bg_gradient, 255)
    }

    /// Render a tile with optional wrapping mode.
    pub fn draw_tile(
        &mut self,
        rect: Rect,
        tile: TileRef<'_>,
        opacity: u8,
    ) -> Result<(), D::Error> {
        self.draw_tile_transformed_ssaa(rect, tile, Transform2D::IDENTITY, opacity, false)
    }

    /// Render a transformed tile with optional wrapping mode.
    pub fn draw_tile_transformed(
        &mut self,
        rect: Rect,
        tile: TileRef<'_>,
        transform: Transform2D,
        opacity: u8,
    ) -> Result<(), D::Error> {
        self.draw_tile_transformed_ssaa(rect, tile, transform, opacity, false)
    }

    /// Render a transformed tile with 2xSSAA (2x Super-Sampling Anti-Aliasing).
    pub fn draw_tile_transformed_ssaa(
        &mut self,
        rect: Rect,
        tile: TileRef<'_>,
        transform: Transform2D,
        opacity: u8,
        enable_ssaa: bool,
    ) -> Result<(), D::Error> {
        let draw = self.visible_rect(rect);
        if draw.is_empty() || opacity == 0 || tile.width == 0 || tile.height == 0 {
            return Ok(());
        }

        let inv_transform = match transform.inverse() {
            Some(inv) => inv,
            None => return Ok(()),
        };

        let cx = rect.x as f32 + rect.w as f32 * 0.5;
        let cy = rect.y as f32 + rect.h as f32 * 0.5;

        let offsets = [
            (0.25f32, 0.25f32),
            (0.75f32, 0.25f32),
            (0.25f32, 0.75f32),
            (0.75f32, 0.75f32),
        ];

        for y in draw.y..draw.bottom() {
            for x in draw.x..draw.right() {
                if !enable_ssaa {
                    let px = (x as f32 + 0.5) - cx;
                    let py = (y as f32 + 0.5) - cy;
                    let (tx, ty) = inv_transform.apply_f32(px, py);
                    let u = (tx + tile.width as f32 * 0.5).floor() as i32;
                    let v = (ty + tile.height as f32 * 0.5).floor() as i32;
                    if let Some(col) = tile.get_pixel(u, v) {
                        self.pixel(x, y, col, opacity)?;
                    }
                } else {
                    let mut r_sum = 0u32;
                    let mut g_sum = 0u32;
                    let mut b_sum = 0u32;
                    let mut weight = 0u32;

                    for &(ox, oy) in &offsets {
                        let px = (x as f32 + ox) - cx;
                        let py = (y as f32 + oy) - cy;
                        let (tx, ty) = inv_transform.apply_f32(px, py);
                        let u = (tx + tile.width as f32 * 0.5).floor() as i32;
                        let v = (ty + tile.height as f32 * 0.5).floor() as i32;
                        if let Some(col) = tile.get_pixel(u, v) {
                            r_sum += col.r() as u32;
                            g_sum += col.g() as u32;
                            b_sum += col.b() as u32;
                            weight += 1;
                        }
                    }

                    if let Some(w) = core::num::NonZeroU32::new(weight) {
                        let weight_val = w.get();
                        let r_avg = (r_sum / weight_val) as u8;
                        let g_avg = (g_sum / weight_val) as u8;
                        let b_avg = (b_sum / weight_val) as u8;
                        let color = Rgb565::new(r_avg, g_avg, b_avg);
                        let pix_opacity = ((weight * opacity as u32 + 2) / 4) as u8;
                        self.pixel(x, y, color, pix_opacity)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Render a transformed image/tile with optional 2xSSAA.
    pub fn draw_image_transformed_ssaa(
        &mut self,
        rect: Rect,
        image: ImageRef<'_>,
        scale: f32,
        rotation_deg: f32,
        opacity: u8,
        enable_ssaa: bool,
    ) -> Result<(), D::Error> {
        let transform = Transform2D::rotation(rotation_deg).then(Transform2D::scale(scale, scale));
        let tile = TileRef::from_image(image, TileMode::None);
        self.draw_tile_transformed_ssaa(rect, tile, transform, opacity, enable_ssaa)
    }
}

fn should_draw_at_opacity(x: i32, y: i32, opacity: u8) -> bool {
    if opacity == 255 {
        return true;
    }
    if opacity == 0 {
        return false;
    }
    let bayer4 = [
        [0u8, 8, 2, 10],
        [12, 4, 14, 6],
        [3, 11, 1, 9],
        [15, 7, 13, 5],
    ];
    let threshold = ((opacity as u16 * 16) / 255) as u8;
    let sample = bayer4[(y as usize) & 3][(x as usize) & 3];
    sample < threshold.max(1)
}

fn lerp_rgb565(a: Rgb565, b: Rgb565, t: u8) -> Rgb565 {
    let t = t as u16;
    let inv = 255u16.saturating_sub(t);
    let r = ((a.r() as u16 * inv) + (b.r() as u16 * t)) / 255;
    let g = ((a.g() as u16 * inv) + (b.g() as u16 * t)) / 255;
    let bb = ((a.b() as u16 * inv) + (b.b() as u16 * t)) / 255;
    Rgb565::new(r as u8, g as u8, bb as u8)
}

#[inline]
fn normalize_angle_deg(mut deg: f32) -> f32 {
    while deg < 0.0 {
        deg += 360.0;
    }
    while deg >= 360.0 {
        deg -= 360.0;
    }
    deg
}

/// Exact (cos, sin) for a boundary angle that lands on a cardinal direction,
/// or `None` to fall back to a real trig call. Widgets built around a fixed
/// "12 o'clock" (or 3/6/9 o'clock) start angle -- the common case, e.g. a
/// sweeping-arc or gauge starting at -90 degrees -- hit this on every call
/// for that boundary, since only the other (animated) boundary ever lands on
/// a non-cardinal angle. Skips the `sin`/`cos` pair entirely for that
/// boundary instead of computing (and rounding) values that are always
/// exactly 0, 1, or -1.
#[inline]
fn cardinal_unit(deg: f32) -> Option<(f32, f32)> {
    const EPS: f32 = 1e-4;
    let normalized = normalize_angle_deg(deg);
    if (normalized - 0.0).abs() < EPS {
        Some((1.0, 0.0))
    } else if (normalized - 90.0).abs() < EPS {
        Some((0.0, 1.0))
    } else if (normalized - 180.0).abs() < EPS {
        Some((-1.0, 0.0))
    } else if (normalized - 270.0).abs() < EPS {
        Some((0.0, -1.0))
    } else {
        None
    }
}

#[inline]
fn cross(ux: f32, uy: f32, vx: f32, vy: f32) -> f32 {
    ux * vy - uy * vx
}

fn apply_blend_mode(src: Rgb565, mode: BlendMode, backdrop: Rgb565) -> Rgb565 {
    match mode {
        BlendMode::Normal => src,
        BlendMode::Add => Rgb565::new(
            src.r().saturating_add(backdrop.r()),
            src.g().saturating_add(backdrop.g()),
            src.b().saturating_add(backdrop.b()),
        ),
        BlendMode::Multiply => Rgb565::new(
            ((src.r() as u16 * backdrop.r() as u16) / 31) as u8,
            ((src.g() as u16 * backdrop.g() as u16) / 63) as u8,
            ((src.b() as u16 * backdrop.b() as u16) / 31) as u8,
        ),
        BlendMode::Screen => Rgb565::new(
            (31 - ((31 - src.r() as u16) * (31 - backdrop.r() as u16) / 31)) as u8,
            (63 - ((63 - src.g() as u16) * (63 - backdrop.g() as u16) / 63)) as u8,
            (31 - ((31 - src.b() as u16) * (31 - backdrop.b() as u16) / 31)) as u8,
        ),
    }
}

fn in_rounded_rect(x: i32, y: i32, rect: Rect, radius: u8) -> bool {
    if rect.is_empty() {
        return false;
    }
    let radius = radius as i32;
    if radius <= 0 {
        return rect.contains(x, y);
    }

    let left = rect.x;
    let top = rect.y;
    let right = rect.right() - 1;
    let bottom = rect.bottom() - 1;
    let inner_left = left + radius;
    let inner_right = right - radius;
    let inner_top = top + radius;
    let inner_bottom = bottom - radius;

    if (x >= inner_left && x <= inner_right) || (y >= inner_top && y <= inner_bottom) {
        return rect.contains(x, y);
    }

    let (cx, cy) = if x < inner_left && y < inner_top {
        (inner_left, inner_top)
    } else if x > inner_right && y < inner_top {
        (inner_right, inner_top)
    } else if x < inner_left && y > inner_bottom {
        (inner_left, inner_bottom)
    } else if x > inner_right && y > inner_bottom {
        (inner_right, inner_bottom)
    } else {
        return rect.contains(x, y);
    };

    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= radius * radius
}

fn line_len_at(text: &str, start: usize, max_chars: usize, wrap: TextWrap) -> (usize, bool) {
    let mut len = 0;
    let limit = match wrap {
        TextWrap::None => usize::MAX,
        TextWrap::Character => max_chars.max(1),
        TextWrap::Word => max_chars.max(1),
    };
    let mut last_ws_break = None;

    for ch in text.chars().skip(start) {
        if ch == '\n' {
            return (len, true);
        }
        if matches!(wrap, TextWrap::Word) && ch.is_whitespace() {
            last_ws_break = Some(len + 1);
        }
        if len >= limit {
            if matches!(wrap, TextWrap::Word) {
                if let Some(idx) = last_ws_break {
                    return (idx, false);
                }
            }
            return (len, false);
        }
        len += 1;
    }

    (len, false)
}

fn count_lines(text: &str, max_chars: usize, wrap: TextWrap) -> usize {
    if text.is_empty() {
        return 1;
    }
    let char_count = text.chars().count();
    let mut lines = 0;
    let mut start = 0;
    while start < char_count {
        let (len, consumed_newline) = line_len_at(text, start, max_chars, wrap);
        lines += 1;
        start += len + usize::from(consumed_newline);
        if len == 0 && !consumed_newline {
            break;
        }
    }
    lines
}

fn widest_line(text: &str, max_chars: usize, wrap: TextWrap) -> usize {
    let char_count = text.chars().count();
    let mut widest = 0;
    let mut start = 0;
    while start < char_count {
        let (len, consumed_newline) = line_len_at(text, start, max_chars, wrap);
        widest = widest.max(len);
        start += len + usize::from(consumed_newline);
        if len == 0 && !consumed_newline {
            break;
        }
    }
    widest
}

fn kerning_adjust(prev: Option<char>, next: char, enabled: bool) -> i32 {
    if !enabled {
        return 0;
    }
    match (prev, next) {
        (Some('A'), 'V') | (Some('A'), 'W') | (Some('T'), 'o') | (Some('L'), 'T') => -1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform2d_is_identity() {
        let id = Transform2D::IDENTITY;
        assert!(id.is_identity());
        assert_eq!(id.apply(10, 20), (10, 20));

        let tr = Transform2D::translation(5.0, 10.0);
        assert!(!tr.is_identity());
        assert_eq!(tr.apply(10, 20), (15, 30));
    }

    #[test]
    fn test_fill_circle_scanline_spans_correctness() {
        let mut buf = crate::test_buffer::TestBuffer::new(50, 50);
        let mut ctx = RenderCtx::new(&mut buf, Rect::new(0, 0, 50, 50));

        // Draw a circle of radius 10 at center (25, 25)
        ctx.fill_circle(25, 25, 10, Rgb565::RED).unwrap();

        // Center pixel must be red
        assert_eq!(buf.pixel_at(25, 25), Some(Rgb565::RED));

        // Points inside radius 10 must be red (e.g. 25 + 7, 25 + 7 => dist^2 = 98 <= 100)
        assert_eq!(buf.pixel_at(32, 32), Some(Rgb565::RED));

        // Points outside radius 10 must remain black (e.g. 25 + 11, 25)
        assert_eq!(buf.pixel_at(37, 25), Some(Rgb565::BLACK));
        assert_eq!(buf.pixel_at(25, 37), Some(Rgb565::BLACK));
    }

    #[test]
    fn test_cardinal_unit_exact_values_and_fallback() {
        assert_eq!(cardinal_unit(0.0), Some((1.0, 0.0)));
        assert_eq!(cardinal_unit(90.0), Some((0.0, 1.0)));
        assert_eq!(cardinal_unit(180.0), Some((-1.0, 0.0)));
        assert_eq!(cardinal_unit(270.0), Some((0.0, -1.0)));
        // -90 degrees normalizes to 270 -- the common "12 o'clock start"
        // sweeping-arc/gauge convention.
        assert_eq!(cardinal_unit(-90.0), Some((0.0, -1.0)));
        // A non-cardinal angle (or one more than EPS off a cardinal one)
        // must fall through to a real trig call.
        assert_eq!(cardinal_unit(45.0), None);
        assert_eq!(cardinal_unit(89.99), None);
    }

    #[test]
    fn test_cardinal_unit_agrees_with_real_trig_at_cardinal_angles() {
        // The fast path's exact 0/1/-1 constants must be numerically
        // consistent with what a real sin/cos call would produce for the
        // same angle (up to float rounding) -- this is the actual property
        // that makes skipping the trig call safe, independent of any
        // downstream rasterization sensitivity near sector boundaries.
        for deg in [0.0_f32, 90.0, 180.0, 270.0, -90.0, 450.0] {
            let (fast_c, fast_s) = cardinal_unit(deg).expect("cardinal angle");
            let (real_c, real_s) = (deg.to_radians().cos(), deg.to_radians().sin());
            assert!(
                (fast_c - real_c).abs() < 1e-6,
                "cos mismatch at {deg}: fast={fast_c} real={real_c}"
            );
            assert!(
                (fast_s - real_s).abs() < 1e-6,
                "sin mismatch at {deg}: fast={fast_s} real={real_s}"
            );
        }
    }

    #[test]
    fn test_fill_sector_sweep_cardinal_fast_path_renders() {
        // Smoke-test the fast path end-to-end: a start angle that hits
        // cardinal_unit must still paint a plausible, growing sector (the
        // per-pixel geometry test is unchanged either way -- only how the
        // boundary direction vectors are obtained differs).
        let mut buf = crate::test_buffer::TestBuffer::new(50, 50);
        let mut ctx = RenderCtx::new(&mut buf, Rect::new(0, 0, 50, 50));
        ctx.fill_sector_sweep(25, 25, 20, -90.0, 90.0, Rgb565::RED)
            .unwrap();
        assert!(buf.count_color(Rgb565::RED) > 0);
        // A quarter sweep from 12 o'clock (clockwise, since sweep is
        // positive/ccw in this atan2-angle convention going toward 3
        // o'clock) should light up the pixel directly right of center but
        // not the one directly below it.
        assert_eq!(buf.pixel_at(40, 25), Some(Rgb565::RED));
        assert_eq!(buf.pixel_at(25, 40), Some(Rgb565::BLACK));
    }
}
