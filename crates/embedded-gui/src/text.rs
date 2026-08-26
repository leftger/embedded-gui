use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};

use crate::render::{
    CHAR_HEIGHT, CHAR_WIDTH, TextAlign, TextMetrics, TextStyle, TextWrap, VerticalAlign,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span<'a> {
    pub content: &'a str,
    pub style: TextStyle,
}

impl<'a> Span<'a> {
    pub const fn raw(content: &'a str) -> Self {
        Self {
            content,
            style: TextStyle::new(Rgb565::WHITE),
        }
    }

    pub const fn styled(content: &'a str, style: TextStyle) -> Self {
        Self { content, style }
    }

    pub fn width_chars(&self) -> usize {
        self.content.chars().filter(|&ch| ch != '\n').count()
    }

    pub fn metrics(&self) -> TextMetrics {
        TextMetrics {
            width: self.width_chars() as u32 * self.style.font.advance(),
            height: self.style.font.line_height(),
        }
    }
}

impl<'a> From<&'a str> for Span<'a> {
    fn from(value: &'a str) -> Self {
        Self::raw(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line<'a> {
    pub spans: &'a [Span<'a>],
    pub align: TextAlign,
}

impl<'a> Line<'a> {
    pub const fn from_spans(spans: &'a [Span<'a>]) -> Self {
        Self {
            spans,
            align: TextAlign::Left,
        }
    }

    pub const fn aligned(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn width_chars(&self) -> usize {
        self.widest_line_chars(u32::MAX, TextWrap::None)
    }

    pub fn char_count(&self) -> usize {
        self.spans
            .iter()
            .map(|span| span.content.chars().count())
            .sum()
    }

    pub fn metrics(&self) -> TextMetrics {
        let mut width = 0u32;
        let mut height = CHAR_HEIGHT;
        for span in self.spans {
            width = width.saturating_add(
                span.content.chars().filter(|&ch| ch != '\n').count() as u32
                    * span.style.font.advance(),
            );
            height = height.max(span.style.font.line_height());
        }
        TextMetrics { width, height }
    }

    pub fn visual_line_count(&self, max_width: u32, wrap: TextWrap) -> usize {
        let char_count = self.char_count();
        if char_count == 0 {
            return 1;
        }

        let mut lines = 0;
        let mut start = 0;
        while start < char_count {
            let (len, consumed_newline) = self.segment_len_at(start, max_width, wrap);
            lines += 1;
            start += len + usize::from(consumed_newline);
            if len == 0 && !consumed_newline {
                break;
            }
        }
        lines.max(1)
    }

    pub fn widest_line_chars(&self, max_width: u32, wrap: TextWrap) -> usize {
        let char_count = self.char_count();
        let mut widest = 0;
        let mut start = 0;
        while start < char_count {
            let (len, consumed_newline) = self.segment_len_at(start, max_width, wrap);
            widest = widest.max(len);
            start += len + usize::from(consumed_newline);
            if len == 0 && !consumed_newline {
                break;
            }
        }
        widest
    }

    pub fn widest_line_width(&self, max_width: u32, wrap: TextWrap) -> u32 {
        let char_count = self.char_count();
        let mut widest = 0u32;
        let mut start = 0;
        while start < char_count {
            let (len, consumed_newline) = self.segment_len_at(start, max_width, wrap);
            widest = widest.max(self.segment_width(start, len));
            start += len + usize::from(consumed_newline);
            if len == 0 && !consumed_newline {
                break;
            }
        }
        widest
    }

    pub fn max_line_height(&self) -> u32 {
        self.spans
            .iter()
            .map(|span| span.style.font.line_height())
            .max()
            .unwrap_or(CHAR_HEIGHT)
    }

    pub(crate) fn segment_len_at(
        &self,
        start: usize,
        max_width: u32,
        wrap: TextWrap,
    ) -> (usize, bool) {
        let mut len = 0;
        let mut width = 0u32;
        let min_advance = self
            .spans
            .iter()
            .map(|span| span.style.font.advance())
            .min()
            .unwrap_or(CHAR_WIDTH)
            .max(1);
        let limit_width = max_width.max(min_advance);
        let mut last_ws_break = None;

        for (ch, style) in self
            .spans
            .iter()
            .flat_map(|span| span.content.chars().map(move |ch| (ch, span.style)))
            .skip(start)
        {
            if ch == '\n' {
                return (len, true);
            }
            if matches!(wrap, TextWrap::Character | TextWrap::Word) {
                let advance = style.font.advance();
                if len > 0 && width.saturating_add(advance) > limit_width {
                    if matches!(wrap, TextWrap::Word) {
                        if let Some(idx) = last_ws_break {
                            return (idx, false);
                        }
                    }
                    return (len, false);
                }
                width = width.saturating_add(advance);
            }
            if matches!(wrap, TextWrap::Word) && ch.is_whitespace() {
                last_ws_break = Some(len + 1);
            }
            len += 1;
        }

        (len, false)
    }

    pub(crate) fn segment_width(&self, start: usize, len: usize) -> u32 {
        self.spans
            .iter()
            .flat_map(|span| span.content.chars().map(move |ch| (ch, span.style)))
            .enumerate()
            .filter_map(|(idx, (ch, style))| {
                if idx < start || idx >= start + len || ch == '\n' {
                    None
                } else {
                    Some(style.font.advance())
                }
            })
            .sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Text<'a> {
    pub lines: &'a [Line<'a>],
    pub align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub wrap: TextWrap,
    pub line_spacing: u8,
}

impl<'a> Text<'a> {
    pub const fn from_lines(lines: &'a [Line<'a>]) -> Self {
        Self {
            lines,
            align: TextAlign::Left,
            vertical_align: VerticalAlign::Top,
            wrap: TextWrap::None,
            line_spacing: 1,
        }
    }

    pub const fn aligned(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub const fn vertical_aligned(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = align;
        self
    }

    pub const fn wrapped(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub const fn line_spacing(mut self, spacing: u8) -> Self {
        self.line_spacing = spacing;
        self
    }

    pub fn metrics(&self, max_width: u32) -> TextMetrics {
        let mut lines = 0usize;
        let mut widest = 0u32;
        let mut max_line_height = CHAR_HEIGHT;

        for line in self.lines {
            lines += line.visual_line_count(max_width, self.wrap);
            widest = widest.max(line.widest_line_width(max_width, self.wrap));
            max_line_height = max_line_height.max(line.max_line_height());
        }

        let lines = lines.max(1);
        TextMetrics {
            width: widest.min(max_width),
            height: lines as u32 * max_line_height
                + lines.saturating_sub(1) as u32 * self.line_spacing as u32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapingConfig {
    pub direction: TextDirection,
    pub language_tag: Option<&'static str>,
    pub enable_ligatures: bool,
}

impl Default for ShapingConfig {
    fn default() -> Self {
        Self {
            direction: TextDirection::Ltr,
            language_tag: None,
            enable_ligatures: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapedGlyph {
    pub ch: char,
    pub x_advance: i16,
}

pub trait TextShaper {
    fn shape<const N: usize>(
        &self,
        text: &str,
        config: ShapingConfig,
        out: &mut heapless::Vec<ShapedGlyph, N>,
    );
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BasicTextShaper;

impl TextShaper for BasicTextShaper {
    fn shape<const N: usize>(
        &self,
        text: &str,
        config: ShapingConfig,
        out: &mut heapless::Vec<ShapedGlyph, N>,
    ) {
        out.clear();
        let iter = text.chars();
        if matches!(config.direction, TextDirection::Rtl) {
            for ch in iter.rev() {
                let _ = out.push(ShapedGlyph { ch, x_advance: 1 });
            }
        } else {
            for ch in iter {
                let _ = out.push(ShapedGlyph { ch, x_advance: 1 });
            }
        }
    }
}

/// A compact reference into a [`StringArena`] text slab.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextSlice {
    pub offset: u16,
    pub len: u16,
}

impl TextSlice {
    pub const fn empty() -> Self {
        Self { offset: 0, len: 0 }
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// An allocation-free, fixed-capacity UTF-8 text arena / slab.
///
/// Stores string data compactly in a single contiguous byte buffer,
/// avoiding per-node or per-widget fixed string storage overhead.
#[derive(Clone, Debug)]
pub struct StringArena<const CAP: usize = 1024> {
    buffer: heapless::Vec<u8, CAP>,
}

impl<const CAP: usize> Default for StringArena<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> StringArena<CAP> {
    pub const fn new() -> Self {
        Self {
            buffer: heapless::Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn capacity(&self) -> usize {
        CAP
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Allocates a string slice into the arena, returning a compact [`TextSlice`].
    pub fn push_str(&mut self, s: &str) -> Option<TextSlice> {
        let offset = self.buffer.len();
        let len = s.len();
        if offset + len > CAP || offset > u16::MAX as usize || len > u16::MAX as usize {
            return None;
        }
        self.buffer.extend_from_slice(s.as_bytes()).ok()?;
        Some(TextSlice {
            offset: offset as u16,
            len: len as u16,
        })
    }

    /// Retrieves a string reference using a [`TextSlice`].
    pub fn get(&self, slice: TextSlice) -> Option<&str> {
        let start = slice.offset as usize;
        let end = start + slice.len as usize;
        if end <= self.buffer.len() {
            core::str::from_utf8(&self.buffer[start..end]).ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_arena_push_and_get() {
        let mut arena: StringArena<128> = StringArena::new();
        let s1 = arena.push_str("Hello").unwrap();
        let s2 = arena.push_str("World!").unwrap();

        assert_eq!(arena.get(s1), Some("Hello"));
        assert_eq!(arena.get(s2), Some("World!"));
        assert_eq!(arena.len(), 11);

        arena.clear();
        assert_eq!(arena.len(), 0);
    }
}
