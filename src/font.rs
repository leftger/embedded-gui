include!(concat!(env!("OUT_DIR"), "/generated_ascii_3x5.rs"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedFont {
    pub first_char: u8,
    pub advance: u8,
    pub line_height: u8,
    pub glyphs: &'static [[u8; 5]],
}

pub static ASCII_3X5_FONT: PackedFont = PackedFont {
    first_char: 32,
    advance: 4,
    line_height: 6,
    glyphs: &ASCII_3X5_GLYPHS,
};

pub static ASCII_4X7_FONT: PackedFont = PackedFont {
    first_char: 32,
    advance: 5,
    line_height: 8,
    glyphs: &ASCII_4X7_GLYPHS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontId {
    Tiny3x5,
    Medium4x7,
    Scaled6x10,
    Vector(u8),
    Custom(&'static PackedFont),
}

impl FontId {
    pub const fn advance(self) -> u32 {
        match self {
            Self::Tiny3x5 => 4,
            Self::Medium4x7 => 5,
            Self::Scaled6x10 => 7,
            Self::Vector(scale) => (8 * scale) as u32,
            Self::Custom(font) => font.advance as u32,
        }
    }

    pub const fn line_height(self) -> u32 {
        match self {
            Self::Tiny3x5 => 6,
            Self::Medium4x7 => 8,
            Self::Scaled6x10 => 11,
            Self::Vector(scale) => (12 * scale) as u32,
            Self::Custom(font) => font.line_height as u32,
        }
    }
}

pub const fn packed_font(font: FontId) -> &'static PackedFont {
    match font {
        FontId::Tiny3x5 => &ASCII_3X5_FONT,
        FontId::Medium4x7 => &ASCII_4X7_FONT,
        FontId::Scaled6x10 => &ASCII_3X5_FONT,
        FontId::Vector(_) => &ASCII_3X5_FONT,
        FontId::Custom(font) => font,
    }
}

pub fn get_vector_glyph(ch: char) -> &'static [(u8, u8)] {
    match ch {
        ' ' => &[],
        '0' => &[(2,0), (6,0), (6,10), (2,10), (2,0), (0xFF, 0xFF), (2,10), (6,0)],
        '1' => &[(4,0), (4,10), (0xFF, 0xFF), (2,2), (4,0), (0xFF, 0xFF), (2,10), (6,10)],
        '2' => &[(2,0), (6,0), (6,5), (2,5), (2,10), (6,10)],
        '3' => &[(2,0), (6,0), (6,10), (2,10), (0xFF, 0xFF), (2,5), (6,5)],
        '4' => &[(2,0), (2,5), (6,5), (0xFF, 0xFF), (6,0), (6,10)],
        '5' => &[(6,0), (2,0), (2,5), (6,5), (6,10), (2,10)],
        '6' => &[(6,0), (2,0), (2,10), (6,10), (6,5), (2,5)],
        '7' => &[(2,0), (6,0), (2,10)],
        '8' => &[(2,0), (6,0), (6,10), (2,10), (2,0), (0xFF, 0xFF), (2,5), (6,5)],
        '9' => &[(6,5), (2,5), (2,0), (6,0), (6,10), (2,10)],
        'A' | 'a' => &[(2,10), (4,0), (6,10), (0xFF, 0xFF), (3,5), (5,5)],
        'B' | 'b' => &[(2,0), (5,0), (6,2), (6,4), (5,5), (2,5), (5,5), (6,6), (6,8), (5,10), (2,10), (2,0)],
        'C' | 'c' => &[(6,0), (2,0), (2,10), (6,10)],
        'D' | 'd' => &[(2,0), (5,0), (6,3), (6,7), (5,10), (2,10), (2,0)],
        'E' | 'e' => &[(6,0), (2,0), (2,10), (6,10), (0xFF, 0xFF), (2,5), (5,5)],
        'F' | 'f' => &[(6,0), (2,0), (2,10), (0xFF, 0xFF), (2,5), (5,5)],
        'G' | 'g' => &[(6,2), (6,0), (2,0), (2,10), (6,10), (6,5), (4,5)],
        'H' | 'h' => &[(2,0), (2,10), (0xFF, 0xFF), (6,0), (6,10), (0xFF, 0xFF), (2,5), (6,5)],
        'I' | 'i' => &[(4,0), (4,10), (0xFF, 0xFF), (2,0), (6,0), (0xFF, 0xFF), (2,10), (6,10)],
        'J' | 'j' => &[(6,0), (6,8), (4,10), (2,8)],
        'K' | 'k' => &[(2,0), (2,10), (0xFF, 0xFF), (6,0), (2,5), (6,10)],
        'L' | 'l' => &[(2,0), (2,10), (6,10)],
        'M' | 'm' => &[(2,10), (2,0), (4,5), (6,0), (6,10)],
        'N' | 'n' => &[(2,10), (2,0), (6,10), (6,0)],
        'O' | 'o' => &[(2,0), (6,0), (6,10), (2,10), (2,0)],
        'P' | 'p' => &[(2,10), (2,0), (6,0), (6,5), (2,5)],
        'Q' | 'q' => &[(2,0), (6,0), (6,10), (2,10), (2,0), (0xFF, 0xFF), (4,7), (7,10)],
        'R' | 'r' => &[(2,10), (2,0), (6,0), (6,5), (2,5), (0xFF, 0xFF), (4,5), (6,10)],
        'S' | 's' => &[(6,0), (2,0), (2,5), (6,5), (6,10), (2,10)],
        'T' | 't' => &[(2,0), (6,0), (0xFF, 0xFF), (4,0), (4,10)],
        'U' | 'u' => &[(2,0), (2,10), (6,10), (6,0)],
        'V' | 'v' => &[(2,0), (4,10), (6,0)],
        'W' | 'w' => &[(2,0), (2,10), (4,5), (6,10), (6,0)],
        'X' | 'x' => &[(2,0), (6,10), (0xFF, 0xFF), (6,0), (2,10)],
        'Y' | 'y' => &[(2,0), (4,5), (6,0), (0xFF, 0xFF), (4,5), (4,10)],
        'Z' | 'z' => &[(2,0), (6,0), (2,10), (6,10)],
        '-' => &[(2,5), (6,5)],
        '+' => &[(2,5), (6,5), (0xFF, 0xFF), (4,2), (4,8)],
        '.' => &[(4,9), (4,10)],
        ':' => &[(4,2), (4,3), (0xFF, 0xFF), (4,7), (4,8)],
        '/' => &[(2,10), (6,0)],
        _ => &[(2,0), (6,0), (6,10), (2,10), (2,0), (2,0), (6,10)],
    }
}

pub fn glyph_rows(font: FontId, ch: char) -> [u8; 5] {
    let packed = packed_font(font);
    let code = ch as u32;
    if code >= packed.first_char as u32 {
        let idx = (code as usize).saturating_sub(packed.first_char as usize);
        if idx < packed.glyphs.len() {
            return packed.glyphs[idx];
        }
    }
    let fallback = b'?'.saturating_sub(packed.first_char) as usize;
    packed.glyphs.get(fallback).copied().unwrap_or([0; 5])
}

#[cfg(test)]
mod tests {
    use super::*;

    static MY_CUSTOM_GLYPHS: [[u8; 5]; 2] = [
        [0b111, 0b101, 0b111, 0b101, 0b101], // Space / 'A'
        [0b111, 0b111, 0b111, 0b111, 0b111],
    ];

    static MY_CUSTOM_FONT: PackedFont = PackedFont {
        first_char: 32,
        advance: 8,
        line_height: 12,
        glyphs: &MY_CUSTOM_GLYPHS,
    };

    #[test]
    fn test_custom_font_id() {
        let font_id = FontId::Custom(&MY_CUSTOM_FONT);
        assert_eq!(font_id.advance(), 8);
        assert_eq!(font_id.line_height(), 12);
        assert_eq!(packed_font(font_id).first_char, 32);
        assert_eq!(
            glyph_rows(font_id, ' '),
            [0b111, 0b101, 0b111, 0b101, 0b101]
        );
    }
}
