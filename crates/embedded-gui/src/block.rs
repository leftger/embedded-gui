use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

use crate::{
    geometry::{EdgeInsets, Rect},
    render::{Compositor, RenderCtx, TextAlign, TextStyle},
    style::{Border, Style},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block<'a> {
    pub title: Option<&'a str>,
    pub title_align: TextAlign,
    pub border: Border,
    pub style: Style,
    pub padding: EdgeInsets,
}

impl<'a> Block<'a> {
    pub const fn new() -> Self {
        Self {
            title: None,
            title_align: TextAlign::Left,
            border: Border::none(),
            style: Style::new(),
            padding: EdgeInsets::all(0),
        }
    }

    pub const fn styled(style: Style) -> Self {
        Self {
            title: None,
            title_align: TextAlign::Left,
            border: style.border,
            padding: style.padding,
            style,
        }
    }

    pub const fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub const fn title_align(mut self, align: TextAlign) -> Self {
        self.title_align = align;
        self
    }

    pub const fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    pub const fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn inner(self, rect: impl Into<Rect>) -> Rect {
        let rect = rect.into();
        let border = self.border.width as i16;
        rect.inset(EdgeInsets {
            left: self.padding.left.saturating_add(border),
            right: self.padding.right.saturating_add(border),
            top: self.padding.top.saturating_add(border),
            bottom: self.padding.bottom.saturating_add(border),
        })
    }

    pub fn title_area(self, rect: impl Into<Rect>) -> Option<Rect> {
        let rect = rect.into();
        self.title.map(|_| {
            Rect::new(
                rect.x + self.border.width as i32 + self.padding.left.max(0) as i32,
                rect.y,
                rect.w
                    .saturating_sub(self.border.width as u32 * 2)
                    .saturating_sub(self.padding.left.max(0) as u32)
                    .saturating_sub(self.padding.right.max(0) as u32),
                self.style.font.line_height() + 1,
            )
        })
    }

    pub fn content_area(self, rect: impl Into<Rect>) -> Rect {
        let rect = rect.into();
        let inner = self.inner(rect);
        if self.title.is_none() {
            return inner;
        }

        let title_h = self.style.font.line_height() + 3;
        Rect::new(
            inner.x,
            inner.y + title_h as i32,
            inner.w,
            inner.h.saturating_sub(title_h),
        )
    }

    pub fn render<D, C>(
        self,
        rect: impl Into<Rect>,
        ctx: &mut RenderCtx<'_, D, C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        let rect = rect.into();
        if let Some(shadow) = self.style.shadow {
            let spread = ctx.shadow_spread_for(shadow.spread);
            let is_opaque_fill = self.style.opacity == 255
                && (self.style.background.is_some() || self.style.gradient.is_some());
            let r = self.style.corner_radius as u32;
            let occluded_core = if is_opaque_fill && rect.w > r * 2 && rect.h > r * 2 {
                Some(Rect::new(
                    rect.x + r as i32,
                    rect.y + r as i32,
                    rect.w - r * 2,
                    rect.h - r * 2,
                ))
            } else {
                None
            };

            let mut i = 0u8;
            while i < spread {
                let grow = i as i32;
                let shadow_rect = Rect::new(
                    rect.x + shadow.offset_x as i32 - grow,
                    rect.y + shadow.offset_y as i32 - grow,
                    rect.w.saturating_add((i as u32) * 2),
                    rect.h.saturating_add((i as u32) * 2),
                );
                let fade = spread as u16;
                let opacity = ((shadow.opacity as u16) * (fade - i as u16) / fade) as u8;
                let radius = self.style.corner_radius.saturating_add(i);

                if let Some(core) = occluded_core {
                    let top_h = (core.y - shadow_rect.y).max(0) as u32;
                    let top_strip = Rect::new(shadow_rect.x, shadow_rect.y, shadow_rect.w, top_h);

                    let bottom_y = core.bottom();
                    let bottom_h = (shadow_rect.bottom() - bottom_y).max(0) as u32;
                    let bottom_strip = Rect::new(shadow_rect.x, bottom_y, shadow_rect.w, bottom_h);

                    let left_w = (core.x - shadow_rect.x).max(0) as u32;
                    let left_strip = Rect::new(shadow_rect.x, core.y, left_w, core.h);

                    let right_x = core.right();
                    let right_w = (shadow_rect.right() - right_x).max(0) as u32;
                    let right_strip = Rect::new(right_x, core.y, right_w, core.h);

                    let old_clip = ctx.clip();
                    for strip in [top_strip, bottom_strip, left_strip, right_strip] {
                        let clip_strip = old_clip.intersection(strip);
                        if !clip_strip.is_empty() {
                            ctx.set_clip(clip_strip);
                            ctx.fill_rounded_rect_alpha(
                                shadow_rect,
                                radius,
                                shadow.color,
                                opacity,
                            )?;
                        }
                    }
                    ctx.set_clip(old_clip);
                } else {
                    ctx.fill_rounded_rect_alpha(shadow_rect, radius, shadow.color, opacity)?;
                }

                i += 1;
            }
        }

        if let Some(gradient) = self.style.gradient {
            ctx.fill_rounded_rect_gradient_alpha(
                rect,
                self.style.corner_radius,
                gradient,
                self.style.opacity,
            )?;
        } else if let Some(bg) = self.style.background {
            ctx.fill_rounded_rect_alpha(rect, self.style.corner_radius, bg, self.style.opacity)?;
        }
        ctx.stroke_rounded_rect_alpha(
            rect,
            self.style.corner_radius,
            self.border,
            self.style.opacity,
        )?;

        if let Some(title) = self.title {
            let title_rect = self.title_area(rect).unwrap_or(Rect::empty());
            ctx.draw_text_in(
                title_rect,
                title,
                TextStyle::new(self.style.accent)
                    .with_font(self.style.font)
                    .with_align(self.title_align)
                    .with_opacity(self.style.opacity),
            )?;
        }

        Ok(())
    }
}

impl Default for Block<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;
    use crate::render::PixelRead;
    use crate::style::Shadow;
    use embedded_graphics_core::geometry::Point;
    use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};

    #[test]
    fn test_shadow_ring_clipping_pixel_exact_equivalence() {
        const W: usize = 64;
        const H: usize = 64;
        let mut fb_optimized = Framebuffer::<{ W * H }>::new(W as u32, H as u32);
        let mut fb_baseline = Framebuffer::<{ W * H }>::new(W as u32, H as u32);

        let style = Style {
            background: Some(Rgb565::RED),
            corner_radius: 6,
            shadow: Some(Shadow {
                offset_x: 2,
                offset_y: 3,
                spread: 4,
                opacity: 180,
                color: Rgb565::BLUE,
            }),
            opacity: 255,
            ..Style::new()
        };

        let block = Block::styled(style);
        let widget_rect = Rect::new(8, 8, 48, 48);

        // Render optimized
        {
            let mut ctx = RenderCtx::new(&mut fb_optimized, Rect::new(0, 0, W as u32, H as u32));
            block.render(widget_rect, &mut ctx).unwrap();
        }

        // Render baseline (unclipped full shadow manually)
        {
            let mut ctx = RenderCtx::new(&mut fb_baseline, Rect::new(0, 0, W as u32, H as u32));
            let shadow = style.shadow.unwrap();
            let spread = ctx.shadow_spread_for(shadow.spread);
            let mut i = 0u8;
            while i < spread {
                let grow = i as i32;
                let shadow_rect = Rect::new(
                    widget_rect.x + shadow.offset_x as i32 - grow,
                    widget_rect.y + shadow.offset_y as i32 - grow,
                    widget_rect.w.saturating_add((i as u32) * 2),
                    widget_rect.h.saturating_add((i as u32) * 2),
                );
                let fade = spread as u16;
                let opacity = ((shadow.opacity as u16) * (fade - i as u16) / fade) as u8;
                let radius = style.corner_radius.saturating_add(i);
                ctx.fill_rounded_rect_alpha(shadow_rect, radius, shadow.color, opacity)
                    .unwrap();
                i += 1;
            }
            if let Some(bg) = style.background {
                ctx.fill_rounded_rect_alpha(widget_rect, style.corner_radius, bg, style.opacity)
                    .unwrap();
            }
            ctx.stroke_rounded_rect_alpha(
                widget_rect,
                style.corner_radius,
                style.border,
                style.opacity,
            )
            .unwrap();
        }

        assert_eq!(fb_optimized.pixels(), fb_baseline.pixels());
    }

    #[test]
    fn test_shadow_interior_drawn_when_no_fill() {
        const W: usize = 32;
        const H: usize = 32;
        let mut fb = Framebuffer::<{ W * H }>::new(W as u32, H as u32);

        let style = Style {
            background: None,
            gradient: None,
            corner_radius: 4,
            shadow: Some(Shadow {
                offset_x: 0,
                offset_y: 0,
                spread: 2,
                opacity: 200,
                color: Rgb565::GREEN,
            }),
            opacity: 255,
            ..Style::new()
        };

        let block = Block::styled(style);
        let widget_rect = Rect::new(4, 4, 24, 24);

        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, W as u32, H as u32));
        block.render(widget_rect, &mut ctx).unwrap();

        // Center pixel inside the block should have shadow color rendered
        let center_color = fb.get_pixel(Point::new(16, 16));
        assert_ne!(
            center_color,
            Rgb565::BLACK,
            "Center pixel should have shadow drawn when no fill is present"
        );
    }

    #[test]
    fn test_shadow_interior_drawn_when_translucent() {
        const W: usize = 32;
        const H: usize = 32;
        let mut fb_translucent = Framebuffer::<{ W * H }>::new(W as u32, H as u32);
        let mut fb_opaque = Framebuffer::<{ W * H }>::new(W as u32, H as u32);

        let base_style = Style {
            background: Some(Rgb565::WHITE),
            corner_radius: 4,
            shadow: Some(Shadow {
                offset_x: 0,
                offset_y: 0,
                spread: 2,
                opacity: 255,
                color: Rgb565::BLUE,
            }),
            ..Style::new()
        };

        let widget_rect = Rect::new(4, 4, 24, 24);

        // Translucent (opacity 128)
        let block_translucent = Block::styled(Style {
            opacity: 128,
            ..base_style
        });
        let mut ctx = RenderCtx::new(&mut fb_translucent, Rect::new(0, 0, W as u32, H as u32));
        block_translucent.render(widget_rect, &mut ctx).unwrap();

        // Opaque (opacity 255)
        let block_opaque = Block::styled(Style {
            opacity: 255,
            ..base_style
        });
        let mut ctx = RenderCtx::new(&mut fb_opaque, Rect::new(0, 0, W as u32, H as u32));
        block_opaque.render(widget_rect, &mut ctx).unwrap();

        // In the opaque block, every pixel in the core is solid white.
        for y in 8..24 {
            for x in 8..24 {
                assert_eq!(fb_opaque.get_pixel(Point::new(x, y)), Rgb565::WHITE);
            }
        }

        // In the translucent block, the shadow underneath is preserved and visible through the dither.
        let has_shadow_in_core = (8..24)
            .any(|y| (8..24).any(|x| fb_translucent.get_pixel(Point::new(x, y)) == Rgb565::BLUE));
        assert!(
            has_shadow_in_core,
            "Translucent fill should allow shadow to show through in the core"
        );
    }
}
