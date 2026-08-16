use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};

use crate::{
    block::Block,
    geometry::{EdgeInsets, Rect},
    render::{Compositor, RenderCtx, TextAlign, TextStyle},
    style::{Border, VisualState, WidgetStyle},
    widget::{PropertyError, PropertyKey, PropertyValue, Widget},
};

/// 2D Table Data Grid Widget with cell selection and 2D navigation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableWidget<'a> {
    pub rows: &'a [&'a [&'a str]],
    pub headers: Option<&'a [&'a str]>,
    pub selected: Option<(usize, usize)>,
    pub separators: bool,
    pub cell_padding: u8,
    pub align: TextAlign,
}

impl<'a> TableWidget<'a> {
    pub const fn new(rows: &'a [&'a [&'a str]]) -> Self {
        Self {
            rows,
            headers: None,
            selected: None,
            separators: true,
            cell_padding: 4,
            align: TextAlign::Left,
        }
    }

    pub const fn with_headers(mut self, headers: &'a [&'a str]) -> Self {
        self.headers = Some(headers);
        self
    }

    pub const fn with_selection(mut self, row: usize, col: usize) -> Self {
        self.selected = Some((row, col));
        self
    }

    pub const fn with_separators(mut self, separators: bool) -> Self {
        self.separators = separators;
        self
    }

    pub const fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Moves the active 2D selection cursor by `(d_row, d_col)`.
    pub fn move_cursor(&mut self, d_row: i32, d_col: i32) {
        if self.rows.is_empty() {
            return;
        }
        let max_rows = self.rows.len();
        let max_cols = self.rows.iter().map(|r| r.len()).max().unwrap_or(1);

        let (cur_r, cur_c) = self.selected.unwrap_or((0, 0));
        let next_r = (cur_r as i32 + d_row).clamp(0, max_rows as i32 - 1) as usize;
        let next_c = (cur_c as i32 + d_col).clamp(0, max_cols as i32 - 1) as usize;
        self.selected = Some((next_r, next_c));
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
        let header_rows = if self.headers.is_some() { 1 } else { 0 };
        let total_rows = self.rows.len() + header_rows;
        if total_rows == 0 {
            return Ok(());
        }

        let max_cols = {
            let data_cols = self.rows.iter().map(|r| r.len()).max().unwrap_or(1);
            let header_cols = self.headers.map(|h| h.len()).unwrap_or(1);
            data_cols.max(header_cols).max(1)
        };

        let row_h = (inner.h / total_rows as u32).max(1);
        let col_w = (inner.w / max_cols as u32).max(1);

        let mut cur_y = inner.y;

        // Render header row if present
        if let Some(headers) = self.headers {
            for c in 0..max_cols {
                let cell_rect = Rect::new(inner.x + (c as u32 * col_w) as i32, cur_y, col_w, row_h);
                // Header cell background tint
                ctx.fill_rect(cell_rect, Rgb565::new(4, 8, 12))?;
                if self.separators {
                    ctx.stroke_rect(cell_rect, Border::one(resolved.border.color))?;
                }
                let text = headers.get(c).copied().unwrap_or("");
                ctx.draw_text_in(
                    cell_rect.inset(EdgeInsets::all(self.cell_padding as i16)),
                    text,
                    TextStyle::new(Rgb565::WHITE)
                        .with_font(resolved.font)
                        .with_align(self.align),
                )?;
            }
            cur_y += row_h as i32;
        }

        // Render data rows
        for (r, cols) in self.rows.iter().enumerate() {
            let row_y = cur_y + (r as u32 * row_h) as i32;
            for c in 0..max_cols {
                let cell_rect = Rect::new(inner.x + (c as u32 * col_w) as i32, row_y, col_w, row_h);
                let is_selected = self.selected == Some((r, c));

                if is_selected {
                    // Highlight selected cell
                    ctx.fill_rect(cell_rect, Rgb565::new(0, 15, 25))?;
                    ctx.stroke_rect(cell_rect, Border::one(Rgb565::new(0, 45, 31)))?;
                } else if self.separators {
                    ctx.stroke_rect(cell_rect, Border::one(resolved.border.color))?;
                }

                let text = cols.get(c).copied().unwrap_or("");
                let text_color = if is_selected {
                    Rgb565::WHITE
                } else {
                    resolved.text
                };

                ctx.draw_text_in(
                    cell_rect.inset(EdgeInsets::all(self.cell_padding as i16)),
                    text,
                    TextStyle::new(text_color)
                        .with_font(resolved.font)
                        .with_align(self.align),
                )?;
            }
        }

        Ok(())
    }
}

impl<'a> Widget for TableWidget<'a> {
    fn render_widget_bounds(&self, _bounds: Rect, _style: &crate::style::Style) {}

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Selected => self.selected.map(|(r, _)| PropertyValue::Int(r as i32)),
            _ => None,
        }
    }

    fn set_property<'p>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'p>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Selected, PropertyValue::Int(r)) => {
                let col = self.selected.map(|(_, c)| c).unwrap_or(0);
                self.selected = Some((r.max(0) as usize, col));
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}
