use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
};

use crate::{
    font::{FontId, glyph_rows},
    geometry::Rect,
    style::{Border, GradientDirection, LinearGradient},
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextStyle {
    pub color: Rgb565,
    pub font: FontId,
    pub opacity: u8,
    pub align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub wrap: TextWrap,
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

    pub const fn with_opacity(mut self, opacity: u8) -> Self {
        self.opacity = opacity;
        self
    }

    pub const fn with_font(mut self, font: FontId) -> Self {
        self.font = font;
        self
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

pub struct RenderCtx<'a, D>
where
    D: DrawTarget<Color = Rgb565>,
{
    target: &'a mut D,
    clip: Rect,
    dirty: Option<Rect>,
    quality: RenderQuality,
}

impl<'a, D> RenderCtx<'a, D>
where
    D: DrawTarget<Color = Rgb565>,
{
    pub fn new(target: &'a mut D, viewport: Rect) -> Self {
        Self {
            target,
            clip: viewport,
            dirty: None,
            quality: RenderQuality::High,
        }
    }

    pub fn with_dirty(target: &'a mut D, viewport: Rect, dirty: Rect) -> Self {
        Self {
            target,
            clip: viewport,
            dirty: Some(dirty),
            quality: RenderQuality::High,
        }
    }

    pub const fn clip(&self) -> Rect {
        self.clip
    }

    pub fn set_clip(&mut self, clip: Rect) {
        self.clip = clip;
    }

    pub const fn quality(&self) -> RenderQuality {
        self.quality
    }

    pub fn set_quality(&mut self, quality: RenderQuality) {
        self.quality = quality;
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

    pub fn fill_rect(&mut self, rect: Rect, color: Rgb565) -> Result<(), D::Error> {
        self.fill_rect_alpha(rect, color, 255)
    }

    pub fn fill_rect_alpha(
        &mut self,
        rect: Rect,
        color: Rgb565,
        opacity: u8,
    ) -> Result<(), D::Error> {
        self.fill_rounded_rect_alpha(rect, 0, color, opacity)
    }

    pub fn fill_rounded_rect(
        &mut self,
        rect: Rect,
        radius: u8,
        color: Rgb565,
    ) -> Result<(), D::Error> {
        self.fill_rounded_rect_alpha(rect, radius, color, 255)
    }

    pub fn fill_rounded_rect_alpha(
        &mut self,
        rect: Rect,
        radius: u8,
        color: Rgb565,
        opacity: u8,
    ) -> Result<(), D::Error> {
        let draw = self.visible_rect(rect);
        if draw.is_empty() || opacity == 0 {
            return Ok(());
        }
        let radius = radius.min((rect.w.min(rect.h) / 2) as u8);

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
        rect: Rect,
        radius: u8,
        gradient: LinearGradient,
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

    pub fn stroke_rect(&mut self, rect: Rect, border: Border) -> Result<(), D::Error> {
        self.stroke_rect_alpha(rect, border, 255)
    }

    pub fn stroke_rect_alpha(
        &mut self,
        rect: Rect,
        border: Border,
        opacity: u8,
    ) -> Result<(), D::Error> {
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
        rect: Rect,
        radius: u8,
        border: Border,
    ) -> Result<(), D::Error> {
        self.stroke_rounded_rect_alpha(rect, radius, border, 255)
    }

    pub fn stroke_rounded_rect_alpha(
        &mut self,
        rect: Rect,
        radius: u8,
        border: Border,
        opacity: u8,
    ) -> Result<(), D::Error> {
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
        font: FontId,
    ) -> Result<(), D::Error> {
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
        rect: Rect,
        text: &str,
        style: TextStyle,
    ) -> Result<(), D::Error> {
        self.draw_text_in_with_font(rect, text, style, style.font)
    }

    pub fn draw_text_in_with_font(
        &mut self,
        rect: Rect,
        text: &str,
        style: TextStyle,
        font: FontId,
    ) -> Result<(), D::Error> {
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
        while start < char_count {
            let (len, consumed_newline) = line_len_at(text, start, max_chars, style.wrap);
            let line_w = len as u32 * advance;
            let x = match style.align {
                TextAlign::Left => rect.x,
                TextAlign::Center => rect.x + rect.w.saturating_sub(line_w) as i32 / 2,
                TextAlign::Right => rect.x + rect.w.saturating_sub(line_w) as i32,
            };
            self.draw_chars_with_font(x, y, text, start, len, style.color, style.opacity, font)?;
            y += line_step as i32;
            start += len + usize::from(consumed_newline);
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

    pub fn text_metrics_with_font(text: &str, font: FontId) -> TextMetrics {
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
        font: FontId,
    ) -> TextMetrics {
        let max_chars = (max_width / font.advance()).max(1) as usize;
        let lines = count_lines(text, max_chars, wrap).max(1);
        let widest = widest_line(text, max_chars, wrap) as u32 * font.advance();
        TextMetrics {
            width: widest.min(max_width),
            height: lines as u32 * font.line_height() + lines.saturating_sub(1) as u32,
        }
    }

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
    ) -> Result<(), D::Error> {
        let advance = font.advance() as i32;
        let mut cursor_x = x;
        for ch in text.chars().skip(start).take(len) {
            self.draw_char_with_font(cursor_x, y, ch, color, opacity, font)?;
            cursor_x += advance;
        }
        Ok(())
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
        match font {
            FontId::Tiny3x5 => {
                for (row, bits) in glyph.iter().enumerate() {
                    for col in 0..3 {
                        if bits & (1 << (2 - col)) != 0 {
                            self.pixel(x + col, y + row as i32, color, opacity)?;
                        }
                    }
                }
            }
            FontId::Medium4x7 => {
                for (row, bits) in glyph.iter().enumerate() {
                    let y_row = row as i32 + (row as i32 / 2);
                    for col in 0..3 {
                        if bits & (1 << (2 - col)) != 0 {
                            self.pixel(x + col, y + y_row, color, opacity)?;
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
        }
        Ok(())
    }

    fn pixel(&mut self, x: i32, y: i32, color: Rgb565, opacity: u8) -> Result<(), D::Error> {
        if !self.clip.contains(x, y) {
            return Ok(());
        }
        if let Some(dirty) = self.dirty {
            if !dirty.contains(x, y) {
                return Ok(());
            }
        }
        if !should_draw_at_opacity(x, y, opacity) {
            return Ok(());
        }
        self.target.draw_iter([Pixel(Point::new(x, y), color)])
    }

    fn visible_rect(&self, rect: Rect) -> Rect {
        let mut draw = rect.intersection(self.clip);
        if let Some(dirty) = self.dirty {
            draw = draw.intersection(dirty);
        }
        draw
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
    };

    for ch in text.chars().skip(start) {
        if ch == '\n' {
            return (len, true);
        }
        if len >= limit {
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
