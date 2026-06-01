pub trait F32Ext {
    fn atan2(self, other: f32) -> f32;
    fn sin(self) -> f32;
    fn cos(self) -> f32;
    fn tan(self) -> f32;
    fn sqrt(self) -> f32;
    fn floor(self) -> f32;
    fn ceil(self) -> f32;
    fn round(self) -> f32;
    fn fract(self) -> f32;
    fn powf(self, n: f32) -> f32;
    fn powi(self, n: i32) -> f32;
    fn hypot(self, other: f32) -> f32;
}

#[cfg(all(not(feature = "std"), feature = "libm", feature = "micromath"))]
compile_error!("Select at most one math backend: `libm` or `micromath`.");

#[cfg(all(
    not(feature = "std"),
    not(feature = "libm"),
    not(feature = "micromath")
))]
compile_error!("no_std requires a math backend feature: enable `libm` or `micromath`.");

#[cfg(feature = "std")]
impl F32Ext for f32 {
    #[inline]
    fn atan2(self, other: f32) -> f32 {
        f32::atan2(self, other)
    }

    #[inline]
    fn sin(self) -> f32 {
        f32::sin(self)
    }

    #[inline]
    fn cos(self) -> f32 {
        f32::cos(self)
    }

    #[inline]
    fn tan(self) -> f32 {
        f32::tan(self)
    }

    #[inline]
    fn sqrt(self) -> f32 {
        f32::sqrt(self)
    }

    #[inline]
    fn floor(self) -> f32 {
        f32::floor(self)
    }

    #[inline]
    fn ceil(self) -> f32 {
        f32::ceil(self)
    }

    #[inline]
    fn round(self) -> f32 {
        f32::round(self)
    }

    #[inline]
    fn fract(self) -> f32 {
        f32::fract(self)
    }

    #[inline]
    fn powf(self, n: f32) -> f32 {
        f32::powf(self, n)
    }

    #[inline]
    fn powi(self, n: i32) -> f32 {
        f32::powi(self, n)
    }

    #[inline]
    fn hypot(self, other: f32) -> f32 {
        f32::hypot(self, other)
    }
}

#[cfg(not(feature = "std"))]
#[cfg(feature = "libm")]
impl F32Ext for f32 {
    #[inline]
    fn atan2(self, other: f32) -> f32 {
        libm::atan2f(self, other)
    }

    #[inline]
    fn sin(self) -> f32 {
        libm::sinf(self)
    }

    #[inline]
    fn cos(self) -> f32 {
        libm::cosf(self)
    }

    #[inline]
    fn tan(self) -> f32 {
        libm::tanf(self)
    }

    #[inline]
    fn sqrt(self) -> f32 {
        libm::sqrtf(self)
    }

    #[inline]
    fn floor(self) -> f32 {
        libm::floorf(self)
    }

    #[inline]
    fn ceil(self) -> f32 {
        libm::ceilf(self)
    }

    #[inline]
    fn round(self) -> f32 {
        libm::roundf(self)
    }

    #[inline]
    fn fract(self) -> f32 {
        self - libm::floorf(self)
    }

    #[inline]
    fn powf(self, n: f32) -> f32 {
        libm::powf(self, n)
    }

    #[inline]
    fn powi(self, n: i32) -> f32 {
        libm::powf(self, n as f32)
    }

    #[inline]
    fn hypot(self, other: f32) -> f32 {
        libm::hypotf(self, other)
    }
}

#[cfg(not(feature = "std"))]
#[cfg(feature = "micromath")]
impl F32Ext for f32 {
    #[inline]
    fn atan2(self, other: f32) -> f32 {
        micromath::F32Ext::atan2(self, other)
    }

    #[inline]
    fn sin(self) -> f32 {
        micromath::F32Ext::sin(self)
    }

    #[inline]
    fn cos(self) -> f32 {
        micromath::F32Ext::cos(self)
    }

    #[inline]
    fn tan(self) -> f32 {
        micromath::F32Ext::tan(self)
    }

    #[inline]
    fn sqrt(self) -> f32 {
        micromath::F32Ext::sqrt(self)
    }

    #[inline]
    fn floor(self) -> f32 {
        micromath::F32Ext::floor(self)
    }

    #[inline]
    fn ceil(self) -> f32 {
        micromath::F32Ext::ceil(self)
    }

    #[inline]
    fn round(self) -> f32 {
        micromath::F32Ext::round(self)
    }

    #[inline]
    fn fract(self) -> f32 {
        micromath::F32Ext::fract(self)
    }

    #[inline]
    fn powf(self, n: f32) -> f32 {
        micromath::F32Ext::powf(self, n)
    }

    #[inline]
    fn powi(self, n: i32) -> f32 {
        micromath::F32Ext::powi(self, n)
    }

    #[inline]
    fn hypot(self, other: f32) -> f32 {
        micromath::F32Ext::hypot(self, other)
    }
}
