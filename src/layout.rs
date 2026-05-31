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
pub struct LinearLayout {
    pub axis: Axis,
    pub gap: u16,
    pub padding: EdgeInsets,
    pub cross_align: Align,
}

impl LinearLayout {
    pub const fn column() -> Self {
        Self {
            axis: Axis::Vertical,
            gap: 2,
            padding: EdgeInsets::all(0),
            cross_align: Align::Stretch,
        }
    }

    pub const fn row() -> Self {
        Self {
            axis: Axis::Horizontal,
            gap: 2,
            padding: EdgeInsets::all(0),
            cross_align: Align::Stretch,
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

    pub const fn with_padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
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
        let fill_unit = if fill_weight > 0 {
            remaining / fill_weight
        } else {
            0
        };

        let mut cursor = match self.axis {
            Axis::Horizontal => inner.x,
            Axis::Vertical => inner.y,
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
            cursor += main as i32 + self.gap as i32;
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
        if fill_weight > 0 {
            let remaining = available.saturating_sub(used);
            let unit = remaining / fill_weight;
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
