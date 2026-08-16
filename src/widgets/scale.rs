use core::fmt::Write as _;
use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor, WebColors};
use heapless::String;

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    block::Block,
    geometry::Rect,
    render::{Compositor, RenderCtx, StrokeCap, StrokeStyle, TextAlign, TextStyle},
    style::{Style, VisualState, WidgetStyle},
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// Orientation and mode of the graduated scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleMode {
    LinearHorizontal,
    LinearVertical,
    Radial,
}

/// Graduated scale widget with customizable major/minor ticks, labels, and needle indicator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleWidget {
    pub mode: ScaleMode,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub major_ticks: u8,
    pub minor_ticks: u8,
    pub start_angle: i16,
    pub end_angle: i16,
    pub show_labels: bool,
    pub show_needle: bool,
    pub tick_color: Rgb565,
    pub needle_color: Rgb565,
}

impl ScaleWidget {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        Self {
            mode: ScaleMode::Radial,
            value: value.clamp(min, max),
            min,
            max,
            major_ticks: 5,
            minor_ticks: 3,
            start_angle: 135,
            end_angle: 45,
            show_labels: true,
            show_needle: true,
            tick_color: Rgb565::CSS_GRAY,
            needle_color: Rgb565::CSS_RED,
        }
    }

    pub fn linear_horizontal(min: f32, max: f32, value: f32) -> Self {
        Self {
            mode: ScaleMode::LinearHorizontal,
            ..Self::new(min, max, value)
        }
    }

    pub fn linear_vertical(min: f32, max: f32, value: f32) -> Self {
        Self {
            mode: ScaleMode::LinearVertical,
            ..Self::new(min, max, value)
        }
    }

    pub fn with_ticks(mut self, major: u8, minor: u8) -> Self {
        self.major_ticks = major.max(1);
        self.minor_ticks = minor.max(1);
        self
    }

    pub fn with_angles(mut self, start_deg: i16, end_deg: i16) -> Self {
        self.start_angle = start_deg;
        self.end_angle = end_deg;
        self
    }

    pub fn with_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    pub fn with_needle(mut self, show: bool, color: Rgb565) -> Self {
        self.show_needle = show;
        self.needle_color = color;
        self
    }

    pub fn render<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        rect: Rect,
        style: WidgetStyle,
        state: VisualState,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        let resolved = style.resolve(state);
        let block = Block::styled(resolved);
        block.render(rect, ctx)?;
        let inner = block.inner(rect);

        match self.mode {
            ScaleMode::LinearHorizontal => self.render_linear_horizontal(ctx, inner, resolved),
            ScaleMode::LinearVertical => self.render_linear_vertical(ctx, inner, resolved),
            ScaleMode::Radial => self.render_radial(ctx, inner, resolved),
        }
    }

    fn render_linear_horizontal<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        inner: Rect,
        style: Style,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        let total_steps = (self.major_ticks as u32).saturating_mul(self.minor_ticks as u32);
        let baseline_y = inner.y + (inner.h as i32 * 2 / 3);

        // Draw baseline
        ctx.draw_line_styled(
            inner.x,
            baseline_y,
            inner.right(),
            baseline_y,
            StrokeStyle::new(self.tick_color).with_width(1),
        )?;

        let range = (self.max - self.min).max(f32::EPSILON);
        for step in 0..=total_steps {
            let t = step as f32 / total_steps as f32;
            let x = inner.x + (t * (inner.w as f32)) as i32;
            let is_major = step % (self.minor_ticks as u32) == 0;
            let tick_len = if is_major { 8 } else { 4 };

            ctx.draw_line_styled(
                x,
                baseline_y,
                x,
                baseline_y - tick_len,
                StrokeStyle::new(self.tick_color).with_width(if is_major { 2 } else { 1 }),
            )?;

            if is_major && self.show_labels {
                let val = (self.min + t * range).round() as i32;
                let mut label: String<8> = String::new();
                let _ = write!(&mut label, "{}", val);
                ctx.draw_text_in(
                    Rect::new(x - 15, baseline_y - 20, 30, style.font.line_height()),
                    label.as_str(),
                    TextStyle::new(style.text)
                        .with_font(style.font)
                        .with_align(TextAlign::Center),
                )?;
            }
        }

        // Draw needle pointer
        if self.show_needle {
            let t = ((self.value - self.min) / range).clamp(0.0, 1.0);
            let nx = inner.x + (t * (inner.w as f32)) as i32;
            ctx.draw_line_styled(
                nx,
                baseline_y - 12,
                nx,
                baseline_y + 6,
                StrokeStyle::new(self.needle_color)
                    .with_width(2)
                    .with_cap(StrokeCap::Round),
            )?;
            ctx.fill_circle(nx, baseline_y, 3, self.needle_color)?;
        }

        Ok(())
    }

    fn render_linear_vertical<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        inner: Rect,
        style: Style,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        let total_steps = (self.major_ticks as u32).saturating_mul(self.minor_ticks as u32);
        let baseline_x = inner.x + (inner.w as i32 / 3);

        ctx.draw_line_styled(
            baseline_x,
            inner.y,
            baseline_x,
            inner.bottom(),
            StrokeStyle::new(self.tick_color).with_width(1),
        )?;

        let range = (self.max - self.min).max(f32::EPSILON);
        for step in 0..=total_steps {
            let t = step as f32 / total_steps as f32;
            let y = inner.bottom() - (t * (inner.h as f32)) as i32;
            let is_major = step % (self.minor_ticks as u32) == 0;
            let tick_len = if is_major { 8 } else { 4 };

            ctx.draw_line_styled(
                baseline_x,
                y,
                baseline_x + tick_len,
                y,
                StrokeStyle::new(self.tick_color).with_width(if is_major { 2 } else { 1 }),
            )?;

            if is_major && self.show_labels {
                let val = (self.min + t * range).round() as i32;
                let mut label: String<8> = String::new();
                let _ = write!(&mut label, "{}", val);
                ctx.draw_text_in(
                    Rect::new(baseline_x + 12, y - 4, 30, style.font.line_height()),
                    label.as_str(),
                    TextStyle::new(style.text).with_font(style.font),
                )?;
            }
        }

        if self.show_needle {
            let t = ((self.value - self.min) / range).clamp(0.0, 1.0);
            let ny = inner.bottom() - (t * (inner.h as f32)) as i32;
            ctx.draw_line_styled(
                baseline_x - 4,
                ny,
                baseline_x + 12,
                ny,
                StrokeStyle::new(self.needle_color)
                    .with_width(2)
                    .with_cap(StrokeCap::Round),
            )?;
            ctx.fill_circle(baseline_x, ny, 3, self.needle_color)?;
        }

        Ok(())
    }

    fn render_radial<D, C>(
        &self,
        ctx: &mut RenderCtx<'_, D, C>,
        inner: Rect,
        style: Style,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics_core::draw_target::DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        let cx = inner.x + (inner.w as i32 / 2);
        let cy = inner.y + (inner.h as i32 / 2);
        let radius = (inner.w.min(inner.h) / 2).saturating_sub(4);
        if radius < 8 {
            return Ok(());
        }

        let total_steps = (self.major_ticks as u32).saturating_mul(self.minor_ticks as u32);
        let sweep = (self.end_angle - self.start_angle) as f32;
        let range = (self.max - self.min).max(f32::EPSILON);

        for step in 0..=total_steps {
            let t = step as f32 / total_steps as f32;
            let angle = (self.start_angle as f32 + sweep * t).to_radians();
            let is_major = step % (self.minor_ticks as u32) == 0;
            let tick_len = if is_major { 8 } else { 4 };

            let ox = cx + (radius as f32 * angle.cos()) as i32;
            let oy = cy + (radius as f32 * angle.sin()) as i32;
            let ix = cx + ((radius.saturating_sub(tick_len)) as f32 * angle.cos()) as i32;
            let iy = cy + ((radius.saturating_sub(tick_len)) as f32 * angle.sin()) as i32;

            ctx.draw_line_styled(
                ix,
                iy,
                ox,
                oy,
                StrokeStyle::new(self.tick_color).with_width(if is_major { 2 } else { 1 }),
            )?;

            if is_major && self.show_labels && radius > 20 {
                let val = (self.min + t * range).round() as i32;
                let mut label: String<8> = String::new();
                let _ = write!(&mut label, "{}", val);
                let lx = cx + ((radius.saturating_sub(18)) as f32 * angle.cos()) as i32;
                let ly = cy + ((radius.saturating_sub(18)) as f32 * angle.sin()) as i32;
                ctx.draw_text_in(
                    Rect::new(lx - 12, ly - 6, 24, 12),
                    label.as_str(),
                    TextStyle::new(style.text)
                        .with_font(style.font)
                        .with_align(TextAlign::Center),
                )?;
            }
        }

        // Draw center pivot and needle pointer
        if self.show_needle {
            let t = ((self.value - self.min) / range).clamp(0.0, 1.0);
            let needle_angle = (self.start_angle as f32 + sweep * t).to_radians();
            let needle_len = radius.saturating_sub(6) as f32;
            let nx = cx + (needle_len * needle_angle.cos()) as i32;
            let ny = cy + (needle_len * needle_angle.sin()) as i32;

            ctx.draw_line_styled(
                cx,
                cy,
                nx,
                ny,
                StrokeStyle::new(self.needle_color)
                    .with_width(2)
                    .with_cap(StrokeCap::Round),
            )?;
            ctx.fill_circle(cx, cy, 4, self.needle_color)?;
            ctx.stroke_circle(cx, cy, 4, Rgb565::WHITE)?;
        }

        Ok(())
    }
}

impl Widget for ScaleWidget {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Value => Some(PropertyValue::Float(self.value)),
            PropertyKey::Min => Some(PropertyValue::Float(self.min)),
            PropertyKey::Max => Some(PropertyValue::Float(self.max)),
            _ => None,
        }
    }

    fn set_property<'a>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Value, PropertyValue::Float(v)) => {
                self.value = v.clamp(self.min, self.max);
                Ok(())
            }
            (PropertyKey::Min, PropertyValue::Float(m)) => {
                self.min = m;
                Ok(())
            }
            (PropertyKey::Max, PropertyValue::Float(m)) => {
                self.max = m;
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}
