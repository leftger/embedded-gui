use crate::geometry::{EdgeInsets, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearLayout {
    pub axis: Axis,
    pub gap: u16,
    pub padding: EdgeInsets,
    pub cross_align: Align,
    pub justify: JustifyContent,
}

impl LinearLayout {
    pub const fn column() -> Self {
        Self {
            axis: Axis::Vertical,
            gap: 2,
            padding: EdgeInsets::all(0),
            cross_align: Align::Stretch,
            justify: JustifyContent::Start,
        }
    }

    pub const fn row() -> Self {
        Self {
            axis: Axis::Horizontal,
            gap: 2,
            padding: EdgeInsets::all(0),
            cross_align: Align::Stretch,
            justify: JustifyContent::Start,
        }
    }

    pub const fn flex_row() -> Self {
        Self::row()
    }

    pub const fn flex_column() -> Self {
        Self::column()
    }

    pub const fn with_gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_padding(mut self, padding: impl Into<EdgeInsets>) -> Self {
        self.padding = padding.into();
        self
    }

    pub const fn with_justify(mut self, justify: JustifyContent) -> Self {
        self.justify = justify;
        self
    }

    pub const fn with_cross_align(mut self, cross_align: Align) -> Self {
        self.cross_align = cross_align;
        self
    }

    pub fn arrange(&self, area: Rect, item_count: usize, out: &mut [Rect]) -> usize {
        if item_count == 0 || out.is_empty() {
            return 0;
        }

        let count = item_count.min(out.len());
        let inner = area.inset(self.padding);
        let gap_total = self.gap as u32 * count.saturating_sub(1) as u32;

        match self.axis {
            Axis::Vertical => {
                let each_h = inner.h.saturating_sub(gap_total) / count as u32;
                let mut y = inner.y;
                for slot in out.iter_mut().take(count) {
                    *slot = Rect::new(inner.x, y, inner.w, each_h);
                    y += each_h as i32 + self.gap as i32;
                }
            }
            Axis::Horizontal => {
                let each_w = inner.w.saturating_sub(gap_total) / count as u32;
                let mut x = inner.x;
                for slot in out.iter_mut().take(count) {
                    *slot = Rect::new(x, inner.y, each_w, inner.h);
                    x += each_w as i32 + self.gap as i32;
                }
            }
        }

        count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Constraint {
    /// Request at least this many pixels in the current single-pass solver.
    Min(u32),
    /// Request no more than this many pixels in the current single-pass solver.
    Max(u32),
    /// Request an exact number of pixels.
    Length(u32),
    /// Request a percentage of the available main-axis space after gaps.
    Percent(u8),
    /// Request a ratio of the available main-axis space after gaps.
    Ratio(u32, u32),
    /// Share remaining main-axis space with other fill items by weight.
    Fill(u16),
}

impl Constraint {
    pub const fn length(px: u32) -> Self {
        Self::Length(px)
    }

    pub const fn min(px: u32) -> Self {
        Self::Min(px)
    }

    pub const fn max(px: u32) -> Self {
        Self::Max(px)
    }

    pub const fn percent(percent: u8) -> Self {
        Self::Percent(percent)
    }

    pub const fn ratio(numerator: u32, denominator: u32) -> Self {
        Self::Ratio(numerator, denominator)
    }

    pub const fn fill(weight: u16) -> Self {
        Self::Fill(weight)
    }

    fn fixed_size(self, total: u32) -> Option<u32> {
        match self {
            Self::Length(px) | Self::Min(px) | Self::Max(px) => Some(px),
            Self::Percent(pct) => Some(total.saturating_mul(pct.min(100) as u32) / 100),
            Self::Ratio(num, den) => Some(total.saturating_mul(num) / den.max(1)),
            Self::Fill(_) => None,
        }
    }

    fn clamp(self, value: u32) -> u32 {
        match self {
            Self::Min(px) => value.max(px),
            Self::Max(px) => value.min(px),
            _ => value,
        }
    }

    fn fill_weight(self) -> u32 {
        match self {
            Self::Fill(weight) => weight.max(1) as u32,
            _ => 0,
        }
    }
}

pub type Length = Constraint;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutItem {
    pub main: Constraint,
    pub cross: Constraint,
    pub grow: u16,
    pub shrink: u16,
}

impl LayoutItem {
    pub const fn fixed(main: u32) -> Self {
        Self::length(main)
    }

    pub const fn length(main: u32) -> Self {
        Self {
            main: Constraint::Length(main),
            cross: Constraint::Fill(1),
            grow: 0,
            shrink: 1,
        }
    }

    pub const fn fill() -> Self {
        Self::fill_weight(1)
    }

    pub const fn fill_weight(weight: u16) -> Self {
        Self {
            main: Constraint::Fill(weight),
            cross: Constraint::Fill(1),
            grow: if weight == 0 { 1 } else { weight },
            shrink: 1,
        }
    }

    pub const fn percent(main: u8) -> Self {
        Self {
            main: Constraint::Percent(main),
            cross: Constraint::Fill(1),
            grow: 0,
            shrink: 1,
        }
    }

    pub const fn min(main: u32) -> Self {
        Self {
            main: Constraint::Min(main),
            cross: Constraint::Fill(1),
            grow: 0,
            shrink: 1,
        }
    }

    pub const fn max(main: u32) -> Self {
        Self {
            main: Constraint::Max(main),
            cross: Constraint::Fill(1),
            grow: 0,
            shrink: 1,
        }
    }

    pub const fn ratio(numerator: u32, denominator: u32) -> Self {
        Self {
            main: Constraint::Ratio(numerator, denominator),
            cross: Constraint::Fill(1),
            grow: 0,
            shrink: 1,
        }
    }

    pub const fn with_cross(mut self, cross: Constraint) -> Self {
        self.cross = cross;
        self
    }

    pub const fn with_grow(mut self, grow: u16) -> Self {
        self.grow = grow;
        self
    }

    pub const fn with_shrink(mut self, shrink: u16) -> Self {
        self.shrink = shrink;
        self
    }

    pub const fn flex(main: u32) -> Self {
        Self::length(main).with_grow(1).with_shrink(1)
    }

    pub const fn rigid(main: u32) -> Self {
        Self::length(main).with_grow(0).with_shrink(0)
    }
}

impl LinearLayout {
    /// Arranges items in a deterministic single pass.
    ///
    /// Fixed, percentage, ratio, min, and max requests are assigned before
    /// fill space. If those requests exceed the available main-axis space,
    /// items keep their requested sizes and later items may extend beyond the
    /// layout area; render-time clipping is responsible for trimming pixels.
    /// Weighted fill receives remaining pixels, with any rounding remainder
    /// assigned to the final fill item.
    pub fn arrange_items(&self, area: Rect, items: &[LayoutItem], out: &mut [Rect]) -> usize {
        if items.is_empty() || out.is_empty() {
            return 0;
        }

        let count = items.len().min(out.len());
        let inner = area.inset(self.padding);
        let main_total = match self.axis {
            Axis::Horizontal => inner.w,
            Axis::Vertical => inner.h,
        };
        let cross_total = match self.axis {
            Axis::Horizontal => inner.h,
            Axis::Vertical => inner.w,
        };
        let gap_total = self.gap as u32 * count.saturating_sub(1) as u32;
        let available = main_total.saturating_sub(gap_total);
        let mut fixed = 0u32;
        let mut fill_weight = 0u32;

        for item in items.iter().take(count) {
            if let Some(px) = item.main.fixed_size(available) {
                fixed = fixed.saturating_add(px);
            } else {
                fill_weight = fill_weight.saturating_add(item.main.fill_weight());
            }
        }

        let remaining = available.saturating_sub(fixed);
        let fill_unit = remaining.checked_div(fill_weight).unwrap_or(0);

        let (mut cursor, item_gap) = if fill_weight == 0 && remaining > 0 {
            let total_slack = remaining + gap_total;
            match self.justify {
                JustifyContent::Start => (
                    match self.axis {
                        Axis::Horizontal => inner.x,
                        Axis::Vertical => inner.y,
                    },
                    self.gap as i32,
                ),
                JustifyContent::Center => (
                    match self.axis {
                        Axis::Horizontal => inner.x + (remaining as i32 / 2),
                        Axis::Vertical => inner.y + (remaining as i32 / 2),
                    },
                    self.gap as i32,
                ),
                JustifyContent::End => (
                    match self.axis {
                        Axis::Horizontal => inner.x + remaining as i32,
                        Axis::Vertical => inner.y + remaining as i32,
                    },
                    self.gap as i32,
                ),
                JustifyContent::SpaceBetween => {
                    let step = if count > 1 {
                        total_slack / (count as u32 - 1)
                    } else {
                        0
                    };
                    (
                        match self.axis {
                            Axis::Horizontal => inner.x,
                            Axis::Vertical => inner.y,
                        },
                        step as i32,
                    )
                }
                JustifyContent::SpaceAround => {
                    let step = total_slack / count as u32;
                    let initial = step / 2;
                    (
                        match self.axis {
                            Axis::Horizontal => inner.x + initial as i32,
                            Axis::Vertical => inner.y + initial as i32,
                        },
                        step as i32,
                    )
                }
                JustifyContent::SpaceEvenly => {
                    let step = total_slack / (count as u32 + 1);
                    (
                        match self.axis {
                            Axis::Horizontal => inner.x + step as i32,
                            Axis::Vertical => inner.y + step as i32,
                        },
                        step as i32,
                    )
                }
            }
        } else {
            (
                match self.axis {
                    Axis::Horizontal => inner.x,
                    Axis::Vertical => inner.y,
                },
                self.gap as i32,
            )
        };
        let mut used_fill = 0u32;
        let mut seen_fill_weight = 0u32;

        for (slot, item) in out.iter_mut().zip(items.iter()).take(count) {
            let main = if let Some(px) = item.main.fixed_size(available) {
                px
            } else {
                let weight = item.main.fill_weight();
                seen_fill_weight = seen_fill_weight.saturating_add(weight);
                if seen_fill_weight >= fill_weight {
                    remaining.saturating_sub(used_fill)
                } else {
                    let px = fill_unit.saturating_mul(weight);
                    used_fill = used_fill.saturating_add(px);
                    px
                }
            }
            .min(available);
            let main = item.main.clamp(main).min(available);
            let cross = item
                .cross
                .fixed_size(cross_total)
                .unwrap_or(cross_total)
                .min(cross_total);
            let cross = item.cross.clamp(cross).min(cross_total);
            let cross_offset = match self.cross_align {
                Align::Start | Align::Stretch => 0,
                Align::Center => cross_total.saturating_sub(cross) as i32 / 2,
                Align::End => cross_total.saturating_sub(cross) as i32,
            };
            let cross_size = if matches!(self.cross_align, Align::Stretch) {
                cross_total
            } else {
                cross.min(cross_total)
            };

            *slot = match self.axis {
                Axis::Horizontal => Rect::new(
                    cursor,
                    inner.y + cross_offset,
                    main.min(available),
                    cross_size,
                ),
                Axis::Vertical => Rect::new(
                    inner.x + cross_offset,
                    cursor,
                    cross_size,
                    main.min(available),
                ),
            };
            cursor += main as i32 + item_gap;
        }

        count
    }

    pub fn arrange_items_flex(
        &self,
        area: Rect,
        items: &[LayoutItem],
        out: &mut [Rect],
        enable_grow: bool,
        enable_shrink: bool,
    ) -> usize {
        if items.is_empty() || out.is_empty() {
            return 0;
        }
        let count = items.len().min(out.len());
        let inner = area.inset(self.padding);
        let main_total = match self.axis {
            Axis::Horizontal => inner.w,
            Axis::Vertical => inner.h,
        };
        let cross_total = match self.axis {
            Axis::Horizontal => inner.h,
            Axis::Vertical => inner.w,
        };
        let gap_total = self.gap as u32 * count.saturating_sub(1) as u32;
        let available = main_total.saturating_sub(gap_total);

        let mut grow_total = 0u32;
        let mut shrink_total = 0u32;
        let mut used = 0u32;
        let mut fill_weight = 0u32;
        for (idx, item) in items.iter().take(count).enumerate() {
            if let Some(px) = item.main.fixed_size(available) {
                let main = item.main.clamp(px).min(available);
                out[idx].w = main;
                used = used.saturating_add(main);
            } else {
                out[idx].w = 0;
                fill_weight = fill_weight.saturating_add(item.main.fill_weight());
            }
            grow_total = grow_total.saturating_add(item.grow as u32);
            shrink_total = shrink_total.saturating_add(item.shrink.max(1) as u32);
        }
        let remaining = available.saturating_sub(used);
        let unit = remaining.checked_div(fill_weight).unwrap_or(0);
        if fill_weight > 0 {
            let mut seen = 0u32;
            let mut used_fill = 0u32;
            for (idx, item) in items.iter().take(count).enumerate() {
                if item.main.fill_weight() == 0 {
                    continue;
                }
                let w = item.main.fill_weight();
                seen = seen.saturating_add(w);
                let px = if seen >= fill_weight {
                    remaining.saturating_sub(used_fill)
                } else {
                    let part = unit.saturating_mul(w);
                    used_fill = used_fill.saturating_add(part);
                    part
                };
                let main = item.main.clamp(px).min(available);
                out[idx].w = main;
                used = used.saturating_add(main);
            }
        }

        if enable_grow && used < available && grow_total > 0 {
            let extra = available - used;
            let unit = extra / grow_total;
            let mut seen = 0u32;
            let mut given = 0u32;
            for (idx, item) in items.iter().take(count).enumerate() {
                let w = item.grow as u32;
                if w == 0 {
                    continue;
                }
                seen = seen.saturating_add(w);
                let add = if seen >= grow_total {
                    extra.saturating_sub(given)
                } else {
                    let part = unit.saturating_mul(w);
                    given = given.saturating_add(part);
                    part
                };
                out[idx].w = out[idx].w.saturating_add(add);
            }
        }

        if enable_shrink && used > available && shrink_total > 0 {
            let overflow = used - available;
            let unit = overflow / shrink_total;
            let mut seen = 0u32;
            let mut taken = 0u32;
            for (idx, item) in items.iter().take(count).enumerate() {
                let w = item.shrink.max(1) as u32;
                seen = seen.saturating_add(w);
                let sub = if seen >= shrink_total {
                    overflow.saturating_sub(taken)
                } else {
                    let part = unit.saturating_mul(w);
                    taken = taken.saturating_add(part);
                    part
                };
                out[idx].w = out[idx].w.saturating_sub(sub.min(out[idx].w));
            }
        }

        let mut cursor = match self.axis {
            Axis::Horizontal => inner.x,
            Axis::Vertical => inner.y,
        };
        for idx in 0..count {
            let item = items[idx];
            let main = out[idx].w;
            let cross = item
                .cross
                .fixed_size(cross_total)
                .unwrap_or(cross_total)
                .min(cross_total);
            let cross = item.cross.clamp(cross).min(cross_total);
            let cross_offset = match self.cross_align {
                Align::Start | Align::Stretch => 0,
                Align::Center => cross_total.saturating_sub(cross) as i32 / 2,
                Align::End => cross_total.saturating_sub(cross) as i32,
            };
            let cross_size = if matches!(self.cross_align, Align::Stretch) {
                cross_total
            } else {
                cross.min(cross_total)
            };
            out[idx] = match self.axis {
                Axis::Horizontal => Rect::new(cursor, inner.y + cross_offset, main, cross_size),
                Axis::Vertical => Rect::new(inner.x + cross_offset, cursor, cross_size, main),
            };
            cursor += main as i32 + self.gap as i32;
        }
        count
    }
}

/// Sizing track definition for 2D Grid layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridTrack {
    /// Fixed pixel size.
    Px(u32),
    /// Fractional unit (proportional weight among remaining space).
    Fr(u8),
    /// Sized automatically / evenly.
    Auto,
}

impl GridTrack {
    pub const fn px(pixels: u32) -> Self {
        Self::Px(pixels)
    }

    pub const fn fr(weight: u8) -> Self {
        Self::Fr(weight)
    }

    pub const fn auto() -> Self {
        Self::Auto
    }
}

/// Placement and span of an item within a 2D Grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridPlacement {
    pub col: usize,
    pub row: usize,
    pub col_span: usize,
    pub row_span: usize,
}

impl GridPlacement {
    pub const fn cell(col: usize, row: usize) -> Self {
        Self {
            col,
            row,
            col_span: 1,
            row_span: 1,
        }
    }

    pub const fn span(col: usize, row: usize, col_span: usize, row_span: usize) -> Self {
        Self {
            col,
            row,
            col_span: if col_span == 0 { 1 } else { col_span },
            row_span: if row_span == 0 { 1 } else { row_span },
        }
    }
}

/// 2D CSS-style Grid Layout engine (fixed const capacity, `#![no_std]` zero-allocation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridLayout<const COLS: usize, const ROWS: usize> {
    pub col_tracks: [GridTrack; COLS],
    pub row_tracks: [GridTrack; ROWS],
    pub col_gap: u16,
    pub row_gap: u16,
    pub padding: EdgeInsets,
}

impl<const COLS: usize, const ROWS: usize> GridLayout<COLS, ROWS> {
    pub const fn new(col_tracks: [GridTrack; COLS], row_tracks: [GridTrack; ROWS]) -> Self {
        Self {
            col_tracks,
            row_tracks,
            col_gap: 2,
            row_gap: 2,
            padding: EdgeInsets::all(0),
        }
    }

    pub const fn uniform(col_gap: u16, row_gap: u16) -> Self {
        Self {
            col_tracks: [GridTrack::Auto; COLS],
            row_tracks: [GridTrack::Auto; ROWS],
            col_gap,
            row_gap,
            padding: EdgeInsets::all(0),
        }
    }

    pub const fn with_gap(mut self, gap: u16) -> Self {
        self.col_gap = gap;
        self.row_gap = gap;
        self
    }

    pub const fn with_col_gap(mut self, gap: u16) -> Self {
        self.col_gap = gap;
        self
    }

    pub const fn with_row_gap(mut self, gap: u16) -> Self {
        self.row_gap = gap;
        self
    }

    pub fn with_padding(mut self, padding: impl Into<EdgeInsets>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Resolves track coordinates (positions and sizes) for a set of grid tracks.
    fn resolve_tracks(
        available: u32,
        start_pos: i32,
        tracks: &[GridTrack],
        gap: u16,
        out_pos: &mut [i32],
        out_sizes: &mut [u32],
    ) {
        let n = tracks.len();
        if n == 0 {
            return;
        }

        let total_gaps = ((n.saturating_sub(1)) as u32).saturating_mul(gap as u32);
        let space = available.saturating_sub(total_gaps);

        let mut fixed_sum: u32 = 0;
        let mut total_fr: u32 = 0;
        let mut auto_count: u32 = 0;

        for track in tracks {
            match *track {
                GridTrack::Px(px) => fixed_sum = fixed_sum.saturating_add(px),
                GridTrack::Fr(fr) => total_fr = total_fr.saturating_add(fr as u32),
                GridTrack::Auto => auto_count = auto_count.saturating_add(1),
            }
        }

        let remaining = space.saturating_sub(fixed_sum);

        // Auto tracks are treated as 1fr if fr tracks are also present or evenly divided
        if auto_count > 0 && total_fr == 0 {
            total_fr = auto_count;
        } else if auto_count > 0 {
            total_fr = total_fr.saturating_add(auto_count);
        }

        // Calculate sizes
        for (i, track) in tracks.iter().enumerate() {
            let size = match *track {
                GridTrack::Px(px) => px,
                GridTrack::Fr(fr) => (remaining * fr as u32).checked_div(total_fr).unwrap_or(0),
                GridTrack::Auto => remaining.checked_div(total_fr).unwrap_or(0),
            };
            out_sizes[i] = size;
        }

        // Compute starting positions
        let mut cur_pos = start_pos;
        for i in 0..n {
            out_pos[i] = cur_pos;
            cur_pos += out_sizes[i] as i32 + gap as i32;
        }
    }

    /// Arranges items into calculated grid cell bounds according to their `GridPlacement`.
    pub fn arrange_cells(
        &self,
        container: Rect,
        placements: &[GridPlacement],
        out: &mut [Rect],
    ) -> usize {
        let inner = container.inset(self.padding);
        let mut col_pos = [0i32; COLS];
        let mut col_sizes = [0u32; COLS];
        let mut row_pos = [0i32; ROWS];
        let mut row_sizes = [0u32; ROWS];

        Self::resolve_tracks(
            inner.w,
            inner.x,
            &self.col_tracks,
            self.col_gap,
            &mut col_pos,
            &mut col_sizes,
        );

        Self::resolve_tracks(
            inner.h,
            inner.y,
            &self.row_tracks,
            self.row_gap,
            &mut row_pos,
            &mut row_sizes,
        );

        let count = placements.len().min(out.len());
        for i in 0..count {
            let p = placements[i];
            let col = p.col.min(COLS.saturating_sub(1));
            let row = p.row.min(ROWS.saturating_sub(1));
            let col_end = (col + p.col_span).min(COLS);
            let row_end = (row + p.row_span).min(ROWS);

            let x = col_pos[col];
            let y = row_pos[row];

            let mut w: u32 = 0;
            for (c, &size) in col_sizes.iter().enumerate().take(col_end).skip(col) {
                w += size;
                if c + 1 < col_end {
                    w += self.col_gap as u32;
                }
            }

            let mut h: u32 = 0;
            for (r, &size) in row_sizes.iter().enumerate().take(row_end).skip(row) {
                h += size;
                if r + 1 < row_end {
                    h += self.row_gap as u32;
                }
            }

            out[i] = Rect::new(x, y, w, h);
        }

        count
    }
}

impl crate::geometry::FluentBuilder for LinearLayout {}
impl<const COLS: usize, const ROWS: usize> crate::geometry::FluentBuilder
    for GridLayout<COLS, ROWS>
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_layout_column_presets() {
        let col = LinearLayout::column();
        assert_eq!(col.axis, Axis::Vertical);
        assert_eq!(col.gap, 2);
        assert_eq!(col.cross_align, Align::Stretch);

        let row = LinearLayout::row();
        assert_eq!(row.axis, Axis::Horizontal);
        assert_eq!(row.gap, 2);
        assert_eq!(row.cross_align, Align::Stretch);
    }

    #[test]
    fn test_layout_arrange_row() {
        let layout = LinearLayout {
            axis: Axis::Horizontal,
            gap: 5,
            padding: EdgeInsets::all(10),
            cross_align: Align::Stretch,
            justify: JustifyContent::Start,
        };

        let container = Rect::new(0, 0, 100, 50);
        let items = [
            LayoutItem::fixed(20),
            LayoutItem::fill(),
            LayoutItem::fixed(30),
        ];
        let mut out = [Rect::empty(); 3];

        let arranged = layout.arrange_items(container, &items, &mut out);
        assert_eq!(arranged, 3);

        // Container width 100 - padding 20 = 80 main total.
        // Item 0: w=20, x=10
        assert_eq!(out[0].x, 10);
        assert_eq!(out[0].w, 20);

        // Item 1: x = 10 + 20 + 5 = 35, w = 20
        assert_eq!(out[1].x, 35);
        assert_eq!(out[1].w, 20);

        // Item 2: x = 35 + 20 + 5 = 60, w = 30
        assert_eq!(out[2].x, 60);
        assert_eq!(out[2].w, 30);
    }

    #[test]
    fn test_linear_layout_justify_content() {
        let layout = LinearLayout::row()
            .with_gap(0)
            .with_justify(JustifyContent::SpaceBetween);

        assert_eq!(layout.justify, JustifyContent::SpaceBetween);
    }

    #[test]
    fn test_grid_layout_resolution_and_spans() {
        let grid = GridLayout::<3, 2>::new(
            [GridTrack::Px(50), GridTrack::Fr(1), GridTrack::Fr(2)],
            [GridTrack::Px(30), GridTrack::Fr(1)],
        )
        .with_col_gap(10)
        .with_row_gap(5)
        .with_padding(EdgeInsets::all(10));

        let container = Rect::new(0, 0, 320, 240);
        let placements = [
            GridPlacement::cell(0, 0),       // Top-left fixed 50x30
            GridPlacement::span(1, 0, 2, 1), // Top row spanning cols 1 & 2
            GridPlacement::span(0, 1, 3, 1), // Bottom row spanning all 3 cols
        ];
        let mut out = [Rect::empty(); 3];

        let count = grid.arrange_cells(container, &placements, &mut out);
        assert_eq!(count, 3);

        // Item 0 (0,0):
        assert_eq!(out[0].x, 10);
        assert_eq!(out[0].y, 10);
        assert_eq!(out[0].w, 50);
        assert_eq!(out[0].h, 30);

        // Item 1 (1,0, span col 2):
        assert_eq!(out[1].x, 70); // 10 + 50 + 10 = 70
        assert_eq!(out[1].y, 10);
        assert_eq!(out[1].h, 30);

        // Item 2 (0,1, span col 3):
        assert_eq!(out[2].x, 10);
        assert_eq!(out[2].y, 45); // 10 + 30 + 5 = 45
    }
}
