use embedded_graphics_core::pixelcolor::Rgb565;

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;

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
pub enum StrokeCap {
    Butt,
    Round,
    Square,
    Triangle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

/// Zero-allocation dashed stroke pattern configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrokeDash {
    pub dash_len: u8,
    pub gap_len: u8,
    pub offset: u8,
}

impl StrokeDash {
    pub const fn new(dash_len: u8, gap_len: u8) -> Self {
        Self {
            dash_len,
            gap_len,
            offset: 0,
        }
    }

    pub const fn with_offset(mut self, offset: u8) -> Self {
        self.offset = offset;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrokeStyle {
    pub color: Rgb565,
    pub width: u8,
    pub antialias: bool,
    pub antialias_mode: AntiAliasMode,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub dash: Option<StrokeDash>,
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
            dash: None,
        }
    }

    pub const fn with_width(mut self, width: u8) -> Self {
        self.width = if width == 0 { 1 } else { width };
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

    pub const fn with_dash(mut self, dash: StrokeDash) -> Self {
        self.dash = Some(dash);
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

/// Path command for 2D vector paths.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathVerb {
    MoveTo(embedded_graphics_core::geometry::Point),
    LineTo(embedded_graphics_core::geometry::Point),
    QuadTo {
        control: embedded_graphics_core::geometry::Point,
        to: embedded_graphics_core::geometry::Point,
    },
    CubicTo {
        control1: embedded_graphics_core::geometry::Point,
        control2: embedded_graphics_core::geometry::Point,
        to: embedded_graphics_core::geometry::Point,
    },
    Close,
}

/// Fixed-capacity vector path (`#![no_std]` zero-allocation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorPath<const N: usize> {
    pub verbs: [PathVerb; N],
    pub len: usize,
}

impl<const N: usize> Default for VectorPath<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> VectorPath<N> {
    pub const fn new() -> Self {
        Self {
            verbs: [PathVerb::Close; N],
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, verb: PathVerb) -> bool {
        if self.len < N {
            self.verbs[self.len] = verb;
            self.len += 1;
            true
        } else {
            false
        }
    }

    pub fn move_to(&mut self, pt: embedded_graphics_core::geometry::Point) -> &mut Self {
        self.push(PathVerb::MoveTo(pt));
        self
    }

    pub fn line_to(&mut self, pt: embedded_graphics_core::geometry::Point) -> &mut Self {
        self.push(PathVerb::LineTo(pt));
        self
    }

    pub fn quad_to(
        &mut self,
        control: embedded_graphics_core::geometry::Point,
        to: embedded_graphics_core::geometry::Point,
    ) -> &mut Self {
        self.push(PathVerb::QuadTo { control, to });
        self
    }

    pub fn cubic_to(
        &mut self,
        control1: embedded_graphics_core::geometry::Point,
        control2: embedded_graphics_core::geometry::Point,
        to: embedded_graphics_core::geometry::Point,
    ) -> &mut Self {
        self.push(PathVerb::CubicTo {
            control1,
            control2,
            to,
        });
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.push(PathVerb::Close);
        self
    }

    pub fn verbs(&self) -> &[PathVerb] {
        &self.verbs[..self.len]
    }
}
