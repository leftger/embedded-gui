//! 1-bit-per-pixel bitmaps and the composite icons built from them.
//!
//! Unlike [`ImageRef`](crate::image::ImageRef), the ink color is supplied at
//! draw time, so one asset can be tinted per state. Composite icons stack
//! several such parts at fixed offsets and toggle each part independently,
//! which is how multi-part status glyphs stay a single addressable widget.

use embedded_graphics_core::pixelcolor::Rgb565;

/// A 1-bit-per-pixel bitmap: `bits` is row-major, MSB-first, each row padded to
/// a byte boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonoBitmap<'a> {
    pub width: u32,
    pub height: u32,
    pub bits: &'a [u8],
}

impl<'a> MonoBitmap<'a> {
    pub const fn new(width: u32, height: u32, bits: &'a [u8]) -> Self {
        Self {
            width,
            height,
            bits,
        }
    }

    /// Bytes per row, including the padding to the next byte boundary.
    #[inline]
    pub const fn stride(&self) -> u32 {
        self.width.div_ceil(8)
    }

    /// Returns true when the source pixel at `(x, y)` is ink.
    #[inline]
    pub fn is_ink(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let idx = (y * self.stride() + x / 8) as usize;
        match self.bits.get(idx) {
            Some(byte) => byte & (0x80 >> (x % 8)) != 0,
            None => false,
        }
    }
}

/// One layer of a [`WidgetKind::CompositeIcon`](crate::widgets::WidgetKind).
///
/// Offsets are in unscaled source pixels so a part keeps its position when the
/// icon's `scale` changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconPart<'a> {
    pub bitmap: MonoBitmap<'a>,
    pub dx: i32,
    pub dy: i32,
    /// Drives state: a hidden part leaves its region untouched, so the icon
    /// reads as incomplete rather than punching a hole in the backdrop.
    pub visible: bool,
    /// Per-part ink override; falls back to the icon's ink color.
    pub tint: Option<Rgb565>,
}

impl<'a> IconPart<'a> {
    pub const fn new(bitmap: MonoBitmap<'a>, dx: i32, dy: i32) -> Self {
        Self {
            bitmap,
            dx,
            dy,
            visible: true,
            tint: None,
        }
    }

    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub const fn with_tint(mut self, tint: Rgb565) -> Self {
        self.tint = Some(tint);
        self
    }
}

/// Placement of a composite icon's parts inside its widget rect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconAlign {
    TopLeft,
    #[default]
    Center,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_msb_first_rows_with_byte_padding() {
        // 9px wide => 2 bytes per row, second row starts at byte 2.
        let bitmap = MonoBitmap::new(9, 2, &[0b1000_0000, 0b1000_0000, 0b0000_0001, 0b0000_0000]);
        assert_eq!(bitmap.stride(), 2);
        assert!(bitmap.is_ink(0, 0));
        assert!(bitmap.is_ink(8, 0));
        assert!(!bitmap.is_ink(1, 0));
        assert!(bitmap.is_ink(7, 1));
        assert!(!bitmap.is_ink(9, 0));
        assert!(!bitmap.is_ink(0, 2));
    }
}
