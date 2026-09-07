//! Fixed-size cell-grid matrix widget.
//!
//! Renders an `ROWS × COLS` intensity grid as colored cells — useful for LED
//! matrix simulators, sensor-array heatmaps, status boards, and binary
//! indicator grids. Allocation-free and fully `const`-generic.

use embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
};

use crate::{
    geometry::Rect,
    render::{Compositor, RenderCtx},
    style::Style,
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// Fixed-capacity `ROWS × COLS` cell matrix widget.
///
/// Each cell stores an intensity in `0..=255`. Cells at `0` are left undrawn
/// (transparent). Non-zero cells are colored by linearly interpolating
/// between [`off_color`](MatrixWidget::off_color) and
/// [`on_color`](MatrixWidget::on_color).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatrixWidget<const ROWS: usize = 8, const COLS: usize = 8> {
    /// Row-major cell intensities (`0..=255`).
    pub cells: [[u8; COLS]; ROWS],
    /// Color used at intensity `255`.
    pub on_color: Rgb565,
    /// Color used at intensity `1` (intensity `0` is skipped).
    pub off_color: Rgb565,
    /// Pixel gap between cells (default `1`).
    pub gap: u32,
}

impl<const ROWS: usize, const COLS: usize> Default for MatrixWidget<ROWS, COLS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const ROWS: usize, const COLS: usize> MatrixWidget<ROWS, COLS> {
    /// Create an empty (all-zero) matrix with lime-on-black defaults.
    pub const fn new() -> Self {
        Self {
            cells: [[0; COLS]; ROWS],
            on_color: Rgb565::new(10, 63, 10),
            off_color: Rgb565::new(0, 8, 0),
            gap: 1,
        }
    }

    /// Set the on/off colors.
    pub const fn with_colors(mut self, on: Rgb565, off: Rgb565) -> Self {
        self.on_color = on;
        self.off_color = off;
        self
    }

    /// Set the pixel gap between cells.
    pub const fn with_gap(mut self, gap: u32) -> Self {
        self.gap = gap;
        self
    }

    /// Set a single cell intensity (`0..=255`). Out-of-bounds indices are ignored.
    pub fn set(&mut self, row: usize, col: usize, value: u8) {
        if row < ROWS && col < COLS {
            self.cells[row][col] = value;
        }
    }

    /// Get a cell intensity, or `None` if out of bounds.
    pub fn get(&self, row: usize, col: usize) -> Option<u8> {
        if row < ROWS && col < COLS {
            Some(self.cells[row][col])
        } else {
            None
        }
    }

    /// Fill every cell with `value`.
    pub fn fill(&mut self, value: u8) {
        for row in self.cells.iter_mut() {
            for cell in row.iter_mut() {
                *cell = value;
            }
        }
    }

    /// Clear all cells to zero.
    pub fn clear(&mut self) {
        self.fill(0);
    }

    /// Map intensity `0..=255` to a blended Rgb565 color.
    #[inline]
    fn color_for(&self, intensity: u8) -> Rgb565 {
        if intensity == 0 {
            return self.off_color;
        }
        if intensity == 255 {
            return self.on_color;
        }
        let t = intensity as u32;
        let lerp = |a: u8, b: u8, bits: u32| -> u8 {
            let a = a as u32;
            let b = b as u32;
            (((a * (255 - t)) + (b * t)) / 255) as u8 & ((1u8 << bits).wrapping_sub(1))
        };
        // Rgb565 channel widths: R=5, G=6, B=5
        Rgb565::new(
            lerp(self.off_color.r(), self.on_color.r(), 5),
            lerp(self.off_color.g(), self.on_color.g(), 6),
            lerp(self.off_color.b(), self.on_color.b(), 5),
        )
    }

    /// Render the matrix into `bounds`.
    pub fn render<D, C>(&self, bounds: Rect, ctx: &mut RenderCtx<'_, D, C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
    {
        if ROWS == 0 || COLS == 0 || bounds.w == 0 || bounds.h == 0 {
            return Ok(());
        }

        let gap_x = self.gap * (COLS as u32).saturating_sub(1);
        let gap_y = self.gap * (ROWS as u32).saturating_sub(1);
        let cell_w = (bounds.w.saturating_sub(gap_x) / COLS as u32).max(1);
        let cell_h = (bounds.h.saturating_sub(gap_y) / ROWS as u32).max(1);

        for r in 0..ROWS {
            for c in 0..COLS {
                let intensity = self.cells[r][c];
                if intensity == 0 {
                    continue;
                }
                let x = bounds.x + (c as u32 * (cell_w + self.gap)) as i32;
                let y = bounds.y + (r as u32 * (cell_h + self.gap)) as i32;
                let rect = Rect::new(x, y, cell_w, cell_h);
                ctx.fill_rect(rect, self.color_for(intensity))?;
            }
        }

        Ok(())
    }
}

impl<const ROWS: usize, const COLS: usize> Widget for MatrixWidget<ROWS, COLS> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &Style) {}

    fn get_property(&self, _key: PropertyKey) -> Option<PropertyValue<'_>> {
        None
    }

    fn set_property<'a>(
        &mut self,
        _key: PropertyKey,
        _val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        Err(PropertyError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;
    use embedded_graphics_core::pixelcolor::RgbColor;

    #[test]
    fn test_matrix_set_get_clear() {
        let mut m = MatrixWidget::<4, 4>::new();
        m.set(1, 2, 200);
        assert_eq!(m.get(1, 2), Some(200));
        assert_eq!(m.get(0, 0), Some(0));
        assert_eq!(m.get(9, 9), None);

        m.clear();
        assert_eq!(m.get(1, 2), Some(0));

        m.fill(255);
        assert_eq!(m.get(3, 3), Some(255));
    }

    #[test]
    fn test_matrix_render_lit_cells() {
        let mut m = MatrixWidget::<2, 2>::new()
            .with_gap(0)
            .with_colors(Rgb565::new(31, 63, 31), Rgb565::new(0, 0, 0));
        m.set(0, 0, 255);
        m.set(1, 1, 255);

        // 16×16 fb → each cell is 8×8
        let mut fb = Framebuffer::<256>::new(16, 16);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 16, 16));
        assert!(m.render(Rect::new(0, 0, 16, 16), &mut ctx).is_ok());

        let pixels = fb.pixels();
        // Top-left cell lit
        assert_ne!(pixels[0], Rgb565::new(0, 0, 0));
        // Top-right cell dark
        assert_eq!(pixels[8], Rgb565::new(0, 0, 0));
        // Bottom-left dark
        assert_eq!(pixels[8 * 16], Rgb565::new(0, 0, 0));
        // Bottom-right lit
        assert_ne!(pixels[8 * 16 + 8], Rgb565::new(0, 0, 0));
    }

    #[test]
    fn test_matrix_color_lerp_endpoints() {
        let m =
            MatrixWidget::<1, 1>::new().with_colors(Rgb565::new(31, 63, 31), Rgb565::new(0, 0, 0));
        assert_eq!(m.color_for(0).r(), 0);
        assert_eq!(m.color_for(255).r(), 31);
        assert_eq!(m.color_for(255).g(), 63);
    }

    #[test]
    fn matrix_mid_intensity_properties_oob_and_empty_render() {
        let m = MatrixWidget::<2, 2>::default();
        assert!(m.color_for(128).g() > 8);
        assert_eq!(m.get_property(PropertyKey::Value), None);
        let mut m = m;
        assert!(
            m.set_property(PropertyKey::Value, PropertyValue::Int(1))
                .is_err()
        );

        m.set(99, 99, 255);
        assert_eq!(m.get(99, 99), None);

        let zero: MatrixWidget<0, 0> = MatrixWidget::new();
        let mut fb = Framebuffer::<16>::new(4, 4);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 4, 4));
        zero.render(Rect::new(0, 0, 4, 4), &mut ctx).unwrap();
        m.render(Rect::new(0, 0, 0, 0), &mut ctx).unwrap();
    }
}
