use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

use crate::{
    geometry::{EdgeInsets, Rect},
    render::{RenderCtx, TextAlign, TextStyle},
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

    pub fn inner(self, rect: Rect) -> Rect {
        let border = self.border.width as i16;
        rect.inset(EdgeInsets {
            left: self.padding.left.saturating_add(border),
            right: self.padding.right.saturating_add(border),
            top: self.padding.top.saturating_add(border),
            bottom: self.padding.bottom.saturating_add(border),
        })
    }

    pub fn title_area(self, rect: Rect) -> Option<Rect> {
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

    pub fn content_area(self, rect: Rect) -> Rect {
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

    pub fn render<D>(self, rect: Rect, ctx: &mut RenderCtx<'_, D>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        if let Some(shadow) = self.style.shadow {
            let spread = ctx.shadow_spread_for(shadow.spread);
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
                ctx.fill_rounded_rect_alpha(shadow_rect, radius, shadow.color, opacity)?;
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
