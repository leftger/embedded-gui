use crate::{
    geometry::Rect,
    render::RenderCtx,
    style::{VisualState, lerp_style},
};
use embedded_graphics_core::pixelcolor::Rgb565;

use super::*;

impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    GuiContext<'a, NODES, EVENTS, DIRTY>
{
    pub fn render<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        let mut ctx = RenderCtx::new(target, self.viewport);
        ctx.set_quality(self.render_quality);
        self.render_into(&mut ctx, 0, 0, 255)
    }

    /// Like [`GuiContext::render`], but every widget's translucency is composited
    /// with true per-pixel alpha blending instead of ordered dithering. Requires
    /// a readback-capable target ([`PixelRead`](crate::PixelRead)), e.g. a
    /// [`Framebuffer`](crate::Framebuffer) used as a software back buffer.
    pub fn render_composited<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>
            + crate::render::PixelRead,
    {
        let mut ctx = RenderCtx::compositing(target, self.viewport);
        ctx.set_quality(self.render_quality);
        self.render_into::<D, crate::render::Blend>(&mut ctx, 0, 0, 255)
    }

    pub fn render_dirty<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        let slice = self.dirty.as_slice();
        if slice.is_empty() {
            return Ok(());
        }

        if slice.len() == 1 {
            let mut ctx = RenderCtx::with_dirty(target, self.viewport, slice[0]);
            ctx.set_quality(self.render_quality);
            return self.render_into(&mut ctx, 0, 0, 255);
        }

        if let Some(bound) = self.dirty.bounding_rect() {
            let area_bound = bound.w as u64 * bound.h as u64;
            let sum_area: u64 = slice.iter().map(|r| r.w as u64 * r.h as u64).sum();
            // If bounding rect area is close to sum of individual areas (overlap or cluster),
            // render bounding rect in 1 pass to avoid multiple tree sweeps
            if area_bound <= sum_area + (sum_area / 2) {
                let mut ctx = RenderCtx::with_dirty(target, self.viewport, bound);
                ctx.set_quality(self.render_quality);
                return self.render_into(&mut ctx, 0, 0, 255);
            }
        }

        for dirty in slice {
            let mut ctx = RenderCtx::with_dirty(target, self.viewport, *dirty);
            ctx.set_quality(self.render_quality);
            self.render_into(&mut ctx, 0, 0, 255)?;
        }
        Ok(())
    }

    /// Renders dirty regions to a hardware display controller implementing [`crate::render::WindowedDrawTarget`].
    /// Sets the physical controller's column/row window bounds (`set_window`) before rendering,
    /// enabling hardware SPI/DMA transfers exclusively to the dirty sub-window.
    pub fn render_dirty_windowed<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>
            + crate::render::WindowedDrawTarget,
    {
        let slice = self.dirty.as_slice();
        if slice.is_empty() {
            return Ok(());
        }

        for dirty in slice {
            let eg_rect = embedded_graphics_core::primitives::Rectangle::new(
                embedded_graphics_core::geometry::Point::new(dirty.x, dirty.y),
                embedded_graphics_core::geometry::Size::new(dirty.w, dirty.h),
            );
            target.set_window(&eg_rect)?;
            let mut ctx = RenderCtx::with_dirty(target, self.viewport, *dirty);
            ctx.set_quality(self.render_quality);
            self.render_into(&mut ctx, 0, 0, 255)?;
        }
        Ok(())
    }

    /// Renders dirty regions into a small caller-provided scratch pixel buffer, then
    /// contiguously flushes each region to the display target.
    ///
    /// This eliminates visual flicker and dramatically speeds up partial updates on
    /// SPI/I2C displays without requiring a full RAM framebuffer.
    pub fn render_dirty_buffered<D>(
        &self,
        target: &mut D,
        scratch_buffer: &mut [Rgb565],
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        let slice = self.dirty.as_slice();
        if slice.is_empty() || scratch_buffer.is_empty() {
            return Ok(());
        }

        for &dirty in slice {
            if dirty.is_empty() {
                continue;
            }
            let total_pixels = (dirty.w as usize).saturating_mul(dirty.h as usize);
            if total_pixels <= scratch_buffer.len() {
                // Entire dirty rect fits in the scratch buffer
                let mut fb =
                    crate::framebuffer::FramebufferSlice::new(scratch_buffer, dirty.w, dirty.h);
                let bg = self.theme.panel.background.unwrap_or(Rgb565::new(0, 0, 0));
                fb.clear_color(bg);
                let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, dirty.w, dirty.h));
                ctx.set_quality(self.render_quality);
                let _ = self.render_into(&mut ctx, -dirty.x, -dirty.y, 255);

                let area = embedded_graphics_core::primitives::Rectangle::new(
                    embedded_graphics_core::geometry::Point::new(dirty.x, dirty.y),
                    embedded_graphics_core::geometry::Size::new(dirty.w, dirty.h),
                );
                target.fill_contiguous(&area, scratch_buffer[..total_pixels].iter().copied())?;
            } else if scratch_buffer.len() >= dirty.w as usize {
                // Buffer can fit at least one row: render in vertical bands
                let band_height = (scratch_buffer.len() / dirty.w as usize) as u32;
                let mut y = dirty.y;
                let bottom = dirty.bottom();
                while y < bottom {
                    let h = band_height.min((bottom - y) as u32);
                    let band_pixels = (dirty.w * h) as usize;
                    let mut fb =
                        crate::framebuffer::FramebufferSlice::new(scratch_buffer, dirty.w, h);
                    let bg = self.theme.panel.background.unwrap_or(Rgb565::new(0, 0, 0));
                    fb.clear_color(bg);
                    let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, dirty.w, h));
                    ctx.set_quality(self.render_quality);
                    let _ = self.render_into(&mut ctx, -dirty.x, -y, 255);

                    let area = embedded_graphics_core::primitives::Rectangle::new(
                        embedded_graphics_core::geometry::Point::new(dirty.x, y),
                        embedded_graphics_core::geometry::Size::new(dirty.w, h),
                    );
                    target.fill_contiguous(&area, scratch_buffer[..band_pixels].iter().copied())?;
                    y += h as i32;
                }
            } else {
                // Micro-scratch buffer (smaller than 1 scanline): chunk sub-row rectangles
                let chunk_w = scratch_buffer.len() as u32;
                let mut y = dirty.y;
                let bottom = dirty.bottom();
                while y < bottom {
                    let mut x = dirty.x;
                    let right = dirty.right();
                    while x < right {
                        let w = chunk_w.min((right - x) as u32);
                        let chunk_pixels = w as usize;
                        let mut fb =
                            crate::framebuffer::FramebufferSlice::new(scratch_buffer, w, 1);
                        let bg = self.theme.panel.background.unwrap_or(Rgb565::new(0, 0, 0));
                        fb.clear_color(bg);
                        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, w, 1));
                        ctx.set_quality(self.render_quality);
                        let _ = self.render_into(&mut ctx, -x, -y, 255);

                        let area = embedded_graphics_core::primitives::Rectangle::new(
                            embedded_graphics_core::geometry::Point::new(x, y),
                            embedded_graphics_core::geometry::Size::new(w, 1),
                        );
                        target.fill_contiguous(
                            &area,
                            scratch_buffer[..chunk_pixels].iter().copied(),
                        )?;
                        x += w as i32;
                    }
                    y += 1;
                }
            }
        }
        Ok(())
    }

    pub fn render_with_offset<D>(
        &self,
        target: &mut D,
        offset_x: i32,
        offset_y: i32,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        self.render_with_offset_and_opacity(target, offset_x, offset_y, 255)
    }

    pub fn render_with_offset_and_opacity<D>(
        &self,
        target: &mut D,
        offset_x: i32,
        offset_y: i32,
        opacity: u8,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        let mut ctx = RenderCtx::new(target, self.viewport);
        ctx.set_quality(self.render_quality);
        self.render_into(&mut ctx, offset_x, offset_y, opacity)
    }

    pub fn render_with_offset_opacity_and_clip<D>(
        &self,
        target: &mut D,
        offset_x: i32,
        offset_y: i32,
        opacity: u8,
        clip: Rect,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
    {
        let mut ctx = RenderCtx::new(target, self.viewport);
        ctx.set_quality(self.render_quality);
        let old_clip = ctx.clip();
        ctx.set_clip(old_clip.intersection(clip));
        self.render_into(&mut ctx, offset_x, offset_y, opacity)
    }

    pub(crate) fn render_into<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        offset_x: i32,
        offset_y: i32,
        opacity: u8,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        for node in &self.widgets {
            if !self.effective_visible(node.id) {
                continue;
            }
            let Some(base_rect) = self.absolute_rect(node.id) else {
                continue;
            };
            let rect = Rect::new(
                base_rect.x + offset_x,
                base_rect.y + offset_y,
                base_rect.w,
                base_rect.h,
            );
            let base_clip = self.inherited_clip(node.id).unwrap_or(self.viewport);
            let clip = Rect::new(
                base_clip.x + offset_x,
                base_clip.y + offset_y,
                base_clip.w,
                base_clip.h,
            );
            if rect.intersection(clip).is_empty() {
                continue;
            }
            let old_clip = ctx.clip();
            ctx.set_clip(old_clip.intersection(clip));
            let state = if self.pressed.is_some_and(|pressed| pressed.id == node.id) {
                VisualState::Pressed
            } else if Some(node.id) == self.focus {
                VisualState::Focused
            } else if !self.effective_enabled(node.id) {
                VisualState::Disabled
            } else {
                VisualState::Normal
            };
            let mut render_node = *node;
            let class_style = node.style_class.and_then(|class| {
                self.class_styles
                    .iter()
                    .find(|(id, _)| *id == class)
                    .map(|(_, style)| *style)
            });
            let resolve_state_style = |vs: VisualState| {
                class_style
                    .map(|style| style.resolve(vs))
                    .unwrap_or_else(|| render_node.style.resolve(vs))
            };
            let active_style = if let Some((from, to, t)) = self.state_transition_progress(node.id)
            {
                lerp_style(resolve_state_style(from), resolve_state_style(to), t)
            } else {
                resolve_state_style(state)
            };
            render_node.style = render_node.style.with_state_override(state, active_style);
            if opacity < 255 {
                let apply = |v: u8| -> u8 { ((v as u16 * opacity as u16) / 255) as u8 };
                render_node.style.normal.opacity = apply(render_node.style.normal.opacity);
                render_node.style.focused.opacity = apply(render_node.style.focused.opacity);
                render_node.style.pressed.opacity = apply(render_node.style.pressed.opacity);
                render_node.style.disabled.opacity = apply(render_node.style.disabled.opacity);
            }
            render_node.render_at(ctx, rect, state)?;
            ctx.set_clip(old_clip);
        }
        Ok(())
    }
}
