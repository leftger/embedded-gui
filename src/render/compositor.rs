use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
}

pub trait PixelRead: DrawTarget {
    fn get_pixel(&self, point: Point) -> Self::Color;
}

pub trait WindowedDrawTarget: DrawTarget {
    fn set_window(
        &mut self,
        area: &embedded_graphics_core::primitives::Rectangle,
    ) -> Result<(), Self::Error>;
}

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dither;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Blend;

#[inline(always)]
fn should_draw_at_opacity(x: i32, y: i32, opacity: u8) -> bool {
    if opacity == 255 {
        return true;
    }
    if opacity == 0 {
        return false;
    }
    #[rustfmt::skip]
    const BAYER: [u8; 16] = [
         0,  8,  2, 10,
        12,  4, 14,  6,
         3, 11,  1,  9,
        15,  7, 13,  5,
    ];
    let xi = (x & 3) as usize;
    let yi = (y & 3) as usize;
    let threshold = BAYER[yi * 4 + xi] * 16 + 8;
    opacity > threshold
}

#[inline(always)]
pub fn lerp_rgb565(c1: Rgb565, c2: Rgb565, alpha: u8) -> Rgb565 {
    if alpha == 0 {
        return c1;
    }
    if alpha == 255 {
        return c2;
    }
    let a = alpha as u32;
    let r1 = c1.r() as u32;
    let g1 = c1.g() as u32;
    let b1 = c1.b() as u32;
    let r2 = c2.r() as u32;
    let g2 = c2.g() as u32;
    let b2 = c2.b() as u32;

    let r = ((r1 * (255 - a) + r2 * a) / 255) as u8;
    let g = ((g1 * (255 - a) + g2 * a) / 255) as u8;
    let b = ((b1 * (255 - a) + b2 * a) / 255) as u8;

    Rgb565::new(r, g, b)
}

#[inline(always)]
fn apply_blend_mode(fg: Rgb565, mode: BlendMode, bg: Rgb565) -> Rgb565 {
    match mode {
        BlendMode::Normal => fg,
        BlendMode::Add => {
            let r = (fg.r() as u16 + bg.r() as u16).min(31) as u8;
            let g = (fg.g() as u16 + bg.g() as u16).min(63) as u8;
            let b = (fg.b() as u16 + bg.b() as u16).min(31) as u8;
            Rgb565::new(r, g, b)
        }
        BlendMode::Multiply => {
            let r = ((fg.r() as u16 * bg.r() as u16) / 31) as u8;
            let g = ((fg.g() as u16 * bg.g() as u16) / 63) as u8;
            let b = ((fg.b() as u16 * bg.b() as u16) / 31) as u8;
            Rgb565::new(r, g, b)
        }
        BlendMode::Screen => {
            let r = (31 - ((31 - fg.r() as u16) * (31 - bg.r() as u16)) / 31) as u8;
            let g = (63 - ((63 - fg.g() as u16) * (63 - bg.g() as u16)) / 63) as u8;
            let b = (31 - ((31 - fg.b() as u16) * (31 - bg.b() as u16)) / 31) as u8;
            Rgb565::new(r, g, b)
        }
    }
}

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
