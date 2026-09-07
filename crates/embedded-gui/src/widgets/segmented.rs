//! 7-segment and 14-segment alphanumeric displays for `#![no_std]` MCU readouts.
//!
//! Provides zero-font RAM bitmask decoders for digits, hexadecimal, and full
//! alphanumeric characters, rendering crisp digital clock, multimeter, and gauge values.

use embedded_graphics_core::pixelcolor::Rgb565;

use crate::geometry::Rect;

// ---------------------------------------------------------------------------
// 7-Segment Bitmask Constants
// ---------------------------------------------------------------------------
//   AAA
//  F   B
//  F   B
//   GGG
//  E   C
//  E   C
//   DDD   (DP)
pub const SEG_7_A: u8 = 1 << 0;
pub const SEG_7_B: u8 = 1 << 1;
pub const SEG_7_C: u8 = 1 << 2;
pub const SEG_7_D: u8 = 1 << 3;
pub const SEG_7_E: u8 = 1 << 4;
pub const SEG_7_F: u8 = 1 << 5;
pub const SEG_7_G: u8 = 1 << 6;
pub const SEG_7_DP: u8 = 1 << 7;

/// Encodes an ASCII character into a 7-segment bitmask.
pub const fn encode_7seg(c: char) -> u8 {
    match c {
        '0' => SEG_7_A | SEG_7_B | SEG_7_C | SEG_7_D | SEG_7_E | SEG_7_F,
        '1' => SEG_7_B | SEG_7_C,
        '2' => SEG_7_A | SEG_7_B | SEG_7_D | SEG_7_E | SEG_7_G,
        '3' => SEG_7_A | SEG_7_B | SEG_7_C | SEG_7_D | SEG_7_G,
        '4' => SEG_7_B | SEG_7_C | SEG_7_F | SEG_7_G,
        '5' => SEG_7_A | SEG_7_C | SEG_7_D | SEG_7_F | SEG_7_G,
        '6' => SEG_7_A | SEG_7_C | SEG_7_D | SEG_7_E | SEG_7_F | SEG_7_G,
        '7' => SEG_7_A | SEG_7_B | SEG_7_C,
        '8' => SEG_7_A | SEG_7_B | SEG_7_C | SEG_7_D | SEG_7_E | SEG_7_F | SEG_7_G,
        '9' => SEG_7_A | SEG_7_B | SEG_7_C | SEG_7_D | SEG_7_F | SEG_7_G,
        'a' | 'A' => SEG_7_A | SEG_7_B | SEG_7_C | SEG_7_E | SEG_7_F | SEG_7_G,
        'b' | 'B' => SEG_7_C | SEG_7_D | SEG_7_E | SEG_7_F | SEG_7_G,
        'c' => SEG_7_D | SEG_7_E | SEG_7_G,
        'C' => SEG_7_A | SEG_7_D | SEG_7_E | SEG_7_F,
        'd' | 'D' => SEG_7_B | SEG_7_C | SEG_7_D | SEG_7_E | SEG_7_G,
        'e' | 'E' => SEG_7_A | SEG_7_D | SEG_7_E | SEG_7_F | SEG_7_G,
        'f' | 'F' => SEG_7_A | SEG_7_E | SEG_7_F | SEG_7_G,
        'h' | 'H' => SEG_7_B | SEG_7_C | SEG_7_E | SEG_7_F | SEG_7_G,
        'l' | 'L' => SEG_7_D | SEG_7_E | SEG_7_F,
        'o' | 'O' => SEG_7_C | SEG_7_D | SEG_7_E | SEG_7_G,
        'p' | 'P' => SEG_7_A | SEG_7_B | SEG_7_E | SEG_7_F | SEG_7_G,
        'r' | 'R' => SEG_7_E | SEG_7_G,
        'u' | 'U' => SEG_7_B | SEG_7_C | SEG_7_D | SEG_7_E | SEG_7_F,
        '-' => SEG_7_G,
        '_' => SEG_7_D,
        '.' => SEG_7_DP,
        '=' => SEG_7_D | SEG_7_G,
        '°' => SEG_7_A | SEG_7_B | SEG_7_F | SEG_7_G,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// 14-Segment Bitmask Constants (Starburst alphanumeric)
// ---------------------------------------------------------------------------
//   A1  A2
//  F \ | / B
//  F  H J K B
//   G1  G2
//  E  L M N C
//  E / | \ C
//   D1  D2   (DP)
pub const SEG_14_A1: u16 = 1 << 0;
pub const SEG_14_A2: u16 = 1 << 1;
pub const SEG_14_B: u16 = 1 << 2;
pub const SEG_14_C: u16 = 1 << 3;
pub const SEG_14_D1: u16 = 1 << 4;
pub const SEG_14_D2: u16 = 1 << 5;
pub const SEG_14_E: u16 = 1 << 6;
pub const SEG_14_F: u16 = 1 << 7;
pub const SEG_14_G1: u16 = 1 << 8;
pub const SEG_14_G2: u16 = 1 << 9;
pub const SEG_14_H: u16 = 1 << 10;
pub const SEG_14_J: u16 = 1 << 11;
pub const SEG_14_K: u16 = 1 << 12;
pub const SEG_14_L: u16 = 1 << 13;
pub const SEG_14_M: u16 = 1 << 14;
pub const SEG_14_N: u16 = 1 << 15;

/// Encodes an alphanumeric character into a 14-segment bitmask.
pub const fn encode_14seg(c: char) -> u16 {
    match c {
        '0' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_E
                | SEG_14_F
                | SEG_14_K
                | SEG_14_L
        }
        '1' => SEG_14_B | SEG_14_C | SEG_14_K,
        '2' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_G1
                | SEG_14_G2
                | SEG_14_E
                | SEG_14_D1
                | SEG_14_D2
        }
        '3' => SEG_14_A1 | SEG_14_A2 | SEG_14_B | SEG_14_C | SEG_14_D1 | SEG_14_D2 | SEG_14_G2,
        '4' => SEG_14_F | SEG_14_G1 | SEG_14_G2 | SEG_14_B | SEG_14_C,
        '5' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_F
                | SEG_14_G1
                | SEG_14_G2
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
        }
        '6' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_F
                | SEG_14_E
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_C
                | SEG_14_G1
                | SEG_14_G2
        }
        '7' => SEG_14_A1 | SEG_14_A2 | SEG_14_B | SEG_14_C,
        '8' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_E
                | SEG_14_F
                | SEG_14_G1
                | SEG_14_G2
        }
        '9' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_F
                | SEG_14_G1
                | SEG_14_G2
        }
        'A' | 'a' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_C
                | SEG_14_E
                | SEG_14_F
                | SEG_14_G1
                | SEG_14_G2
        }
        'B' | 'b' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_J
                | SEG_14_M
                | SEG_14_G2
        }
        'C' | 'c' => SEG_14_A1 | SEG_14_A2 | SEG_14_F | SEG_14_E | SEG_14_D1 | SEG_14_D2,
        'D' | 'd' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_J
                | SEG_14_M
        }
        'E' | 'e' => {
            SEG_14_A1 | SEG_14_A2 | SEG_14_F | SEG_14_E | SEG_14_D1 | SEG_14_D2 | SEG_14_G1
        }
        'F' | 'f' => SEG_14_A1 | SEG_14_A2 | SEG_14_F | SEG_14_E | SEG_14_G1,
        'G' | 'g' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_F
                | SEG_14_E
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_C
                | SEG_14_G2
        }
        'H' | 'h' => SEG_14_F | SEG_14_E | SEG_14_B | SEG_14_C | SEG_14_G1 | SEG_14_G2,
        'I' | 'i' => SEG_14_A1 | SEG_14_A2 | SEG_14_D1 | SEG_14_D2 | SEG_14_J | SEG_14_M,
        'J' | 'j' => SEG_14_B | SEG_14_C | SEG_14_D1 | SEG_14_D2 | SEG_14_E,
        'K' | 'k' => SEG_14_F | SEG_14_E | SEG_14_G1 | SEG_14_K | SEG_14_N,
        'L' | 'l' => SEG_14_F | SEG_14_E | SEG_14_D1 | SEG_14_D2,
        'M' | 'm' => SEG_14_F | SEG_14_E | SEG_14_B | SEG_14_C | SEG_14_H | SEG_14_K,
        'N' | 'n' => SEG_14_F | SEG_14_E | SEG_14_B | SEG_14_C | SEG_14_H | SEG_14_N,
        'O' | 'o' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_E
                | SEG_14_F
        }
        'P' | 'p' => SEG_14_A1 | SEG_14_A2 | SEG_14_B | SEG_14_E | SEG_14_F | SEG_14_G1 | SEG_14_G2,
        'Q' | 'q' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
                | SEG_14_E
                | SEG_14_F
                | SEG_14_N
        }
        'R' | 'r' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_B
                | SEG_14_E
                | SEG_14_F
                | SEG_14_G1
                | SEG_14_G2
                | SEG_14_N
        }
        'S' | 's' => {
            SEG_14_A1
                | SEG_14_A2
                | SEG_14_F
                | SEG_14_G1
                | SEG_14_G2
                | SEG_14_C
                | SEG_14_D1
                | SEG_14_D2
        }
        'T' | 't' => SEG_14_A1 | SEG_14_A2 | SEG_14_J | SEG_14_M,
        'U' | 'u' => SEG_14_F | SEG_14_E | SEG_14_B | SEG_14_C | SEG_14_D1 | SEG_14_D2,
        'V' | 'v' => SEG_14_F | SEG_14_E | SEG_14_L | SEG_14_K,
        'W' | 'w' => SEG_14_F | SEG_14_E | SEG_14_B | SEG_14_C | SEG_14_L | SEG_14_N,
        'X' | 'x' => SEG_14_H | SEG_14_K | SEG_14_L | SEG_14_N,
        'Y' | 'y' => SEG_14_H | SEG_14_K | SEG_14_M,
        'Z' | 'z' => SEG_14_A1 | SEG_14_A2 | SEG_14_D1 | SEG_14_D2 | SEG_14_K | SEG_14_L,
        '-' => SEG_14_G1 | SEG_14_G2,
        '+' => SEG_14_G1 | SEG_14_G2 | SEG_14_J | SEG_14_M,
        '*' => {
            SEG_14_G1 | SEG_14_G2 | SEG_14_J | SEG_14_M | SEG_14_H | SEG_14_K | SEG_14_L | SEG_14_N
        }
        '/' => SEG_14_K | SEG_14_L,
        '\\' => SEG_14_H | SEG_14_N,
        '|' => SEG_14_J | SEG_14_M,
        _ => 0,
    }
}

/// Visual styling configuration for segment displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentStyle {
    /// Color of active (illuminated) segments.
    pub on_color: Rgb565,
    /// Color of inactive (unlit) segments (providing an LCD/LED ghosting effect).
    pub off_color: Option<Rgb565>,
    /// Segment thickness in pixels.
    pub segment_thickness: u8,
    /// Spacing between digits in pixels.
    pub digit_spacing: u8,
}

impl SegmentStyle {
    pub const fn new(on_color: Rgb565) -> Self {
        Self {
            on_color,
            off_color: None,
            segment_thickness: 2,
            digit_spacing: 4,
        }
    }

    pub const fn with_ghosting(mut self, off_color: Rgb565) -> Self {
        self.off_color = Some(off_color);
        self
    }
}

/// A 7-segment multi-character display widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SevenSegmentDisplay<'a> {
    pub text: &'a str,
    pub style: SegmentStyle,
}

impl<'a> SevenSegmentDisplay<'a> {
    pub const fn new(text: &'a str, style: SegmentStyle) -> Self {
        Self { text, style }
    }

    /// Calculates bounding rectangles for each 7-segment digit bar inside `digit_rect`.
    pub fn digit_segments(digit_rect: Rect, thickness: u8) -> [Rect; 7] {
        let t = thickness as u32;
        let x = digit_rect.x;
        let y = digit_rect.y;
        let w = digit_rect.w;
        let h = digit_rect.h;
        let mid_y = y + (h as i32 - t as i32) / 2;
        let bot_y = y + h as i32 - t as i32;
        let right_x = x + w as i32 - t as i32;
        let half_h = (h.saturating_sub(t * 3)) / 2;

        [
            // A: Top horizontal
            Rect::new(x + t as i32, y, w.saturating_sub(t * 2), t),
            // B: Top-right vertical
            Rect::new(right_x, y + t as i32, t, half_h),
            // C: Bottom-right vertical
            Rect::new(right_x, mid_y + t as i32, t, half_h),
            // D: Bottom horizontal
            Rect::new(x + t as i32, bot_y, w.saturating_sub(t * 2), t),
            // E: Bottom-left vertical
            Rect::new(x, mid_y + t as i32, t, half_h),
            // F: Top-left vertical
            Rect::new(x, y + t as i32, t, half_h),
            // G: Middle horizontal
            Rect::new(x + t as i32, mid_y, w.saturating_sub(t * 2), t),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_7seg_encoding() {
        assert_eq!(encode_7seg('0'), 0x3F);
        assert_eq!(encode_7seg('1'), 0x06);
        assert_eq!(encode_7seg('8'), 0x7F);
        assert_eq!(encode_7seg('-'), 0x40);
        assert_eq!(encode_7seg(' '), 0x00);
    }

    #[test]
    fn test_14seg_encoding() {
        assert_ne!(encode_14seg('A'), 0);
        assert_ne!(encode_14seg('Z'), 0);
        assert_ne!(encode_14seg('+'), 0);
        assert_eq!(encode_14seg(' '), 0);
    }

    #[test]
    fn test_digit_segment_geometry() {
        let rect = Rect::new(10, 10, 20, 40);
        let segs = SevenSegmentDisplay::digit_segments(rect, 2);
        assert_eq!(segs.len(), 7);
        // Top segment A width = 20 - 4 = 16
        assert_eq!(segs[0].w, 16);
        assert_eq!(segs[0].h, 2);
    }

    #[test]
    fn test_7seg_and_14seg_all_ascii_and_symbols() {
        for c in (0u32..=127).filter_map(char::from_u32) {
            let _ = encode_7seg(c);
            let _ = encode_14seg(c);
        }
        let _ = encode_7seg('°');
        let _ = encode_14seg('*');

        for digit in '0'..='9' {
            assert_ne!(encode_7seg(digit), 0);
            assert_ne!(encode_14seg(digit), 0);
        }
    }
}
