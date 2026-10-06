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

#[test]
fn basic_text_shaper_rtl_digits_and_bracket_mirroring() {
    let shaper = BasicTextShaper;
    let mut glyphs = heapless::Vec::<ShapedGlyph, 32>::new();

    // In RTL, digits preserve LTR reading order while brackets are mirrored
    shaper.shape(
        "(abc 42)",
        ShapingConfig {
            direction: TextDirection::Rtl,
            language_tag: Some("ar"),
            enable_ligatures: false,
        },
        &mut glyphs,
    );

    let result_chars: heapless::Vec<char, 32> = glyphs.iter().map(|g| g.ch).collect();
    // '(' becomes ')', ')' becomes '('
    // "abc" reversed -> "cba"
    // "42" digits stay "42" (not reversed to "24")
    // Original: '(' 'a' 'b' 'c' ' ' '4' '2' ')'
    // Reversed: ')' '2' '4' ' ' 'c' 'b' 'a' '('
    // Mirrored brackets: '(' '2' '4' ' ' 'c' 'b' 'a' ')'
    // Digit un-reversed: '(' '4' '2' ' ' 'c' 'b' 'a' ')'
    let expected = ['(', '4', '2', ' ', 'c', 'b', 'a', ')'];
    assert_eq!(result_chars.as_slice(), &expected);
}

#[test]
fn sparse_bitmap_font_lookup_and_rendering() {
    use embedded_gui::font::{Font, FontId, SparseBitmapFont};

    static GLYPH_A: [u8; 8] = [0x18, 0x3C, 0x66, 0x7E, 0x66, 0x66, 0x00, 0x00];
    static GLYPH_ACCENT_E: [u8; 8] = [0x0C, 0x18, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00]; // é

    // Sorted glyphs
    static GLYPHS: [(char, &[u8]); 2] = [('A', &GLYPH_A), ('é', &GLYPH_ACCENT_E)];

    static FONT: SparseBitmapFont = SparseBitmapFont::new(8, 8, 8, 8, 1, &GLYPHS);

    assert_eq!(FONT.advance(), 8);
    assert_eq!(FONT.line_height(), 8);
    assert!(FONT.glyph_bytes('A').is_some());
    assert!(FONT.glyph_bytes('é').is_some());
    assert!(FONT.glyph_bytes('Z').is_none());

    let mut pixel_count = 0;
    FONT.draw_glyph('A', &mut |_x, _y| {
        pixel_count += 1;
    });
    assert!(pixel_count > 0);

    let font_id: FontId = (&FONT).into();
    assert!(matches!(font_id, FontId::Dynamic(_)));
}
