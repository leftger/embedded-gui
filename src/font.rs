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
}

impl FontId {
    pub const fn advance(self) -> u32 {
        match self {
            Self::Tiny3x5 => 4,
            Self::Medium4x7 => 5,
            Self::Scaled6x10 => 7,
        }
    }

    pub const fn line_height(self) -> u32 {
        match self {
            Self::Tiny3x5 => 6,
            Self::Medium4x7 => 8,
            Self::Scaled6x10 => 11,
        }
    }
}

pub const fn packed_font(font: FontId) -> &'static PackedFont {
    match font {
        FontId::Tiny3x5 => &ASCII_3X5_FONT,
        FontId::Medium4x7 => &ASCII_4X7_FONT,
        FontId::Scaled6x10 => &ASCII_3X5_FONT,
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
    let fallback = ('?' as u8).saturating_sub(packed.first_char) as usize;
    packed.glyphs.get(fallback).copied().unwrap_or([0; 5])
}
