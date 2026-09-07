//! Coverage tests for the text metrics, wrapping, shaping, and string arena.

use embedded_gui::render::{TextAlign, TextWrap, VerticalAlign};
use embedded_gui::text::{
    BasicTextShaper, Line, ShapedGlyph, ShapingConfig, Span, StringArena, Text, TextDirection,
    TextShaper, TextSlice,
};

#[test]
fn span_line_and_text_metrics() {
    static SPANS: [Span; 2] = [Span::raw("Hello "), Span::raw("world\nwide")];
    let line = Line::from_spans(&SPANS).aligned(TextAlign::Center);
    assert_eq!(line.char_count(), 16);
    assert!(line.metrics().width > 0);
    assert!(line.metrics().height > 0);
    assert!(line.visual_line_count(u32::MAX, TextWrap::None) >= 2);
    assert!(line.widest_line_chars(u32::MAX, TextWrap::None) > 0);
    assert!(line.widest_line_width(u32::MAX, TextWrap::None) > 0);
    assert!(line.max_line_height() >= 6);

    static LINES: [Line; 1] = [Line::from_spans(&SPANS)];
    let text = Text::from_lines(&LINES)
        .aligned(TextAlign::Right)
        .vertical_aligned(VerticalAlign::Middle)
        .wrapped(TextWrap::Word)
        .line_spacing(2);
    let metrics = text.metrics(20);
    assert!(metrics.width > 0);
    assert!(metrics.height > 0);
}

#[test]
fn word_and_character_wrapping_metrics() {
    static SPANS: [Span; 1] = [Span::raw("one two three four five")];
    static LINES: [Line; 1] = [Line::from_spans(&SPANS)];

    let word = Text::from_lines(&LINES).wrapped(TextWrap::Word);
    let ch = Text::from_lines(&LINES).wrapped(TextWrap::Character);
    assert!(word.metrics(10).height >= ch.metrics(10).height);
}

#[test]
fn basic_text_shaper_directions_and_arena() {
    let shaper = BasicTextShaper;
    let mut glyphs = heapless::Vec::<ShapedGlyph, 16>::new();
    shaper.shape(
        "abc",
        ShapingConfig {
            direction: TextDirection::Ltr,
            language_tag: None,
            enable_ligatures: true,
        },
        &mut glyphs,
    );
    assert_eq!(glyphs.len(), 3);
    assert_eq!(glyphs[0].ch, 'a');

    glyphs.clear();
    shaper.shape(
        "abc",
        ShapingConfig {
            direction: TextDirection::Rtl,
            language_tag: Some("ar"),
            enable_ligatures: false,
        },
        &mut glyphs,
    );
    assert_eq!(glyphs[0].ch, 'c');

    let mut arena = StringArena::<64>::new();
    assert!(arena.is_empty());
    let s1 = arena.push_str("hello").unwrap();
    let s2 = arena.push_str(" world").unwrap();
    assert_eq!(arena.get(s1), Some("hello"));
    assert_eq!(arena.get(s2), Some(" world"));
    assert_eq!(arena.get(TextSlice::empty()), Some(""));
    assert!(arena.push_str(&"x".repeat(100)).is_none());
    arena.clear();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}
