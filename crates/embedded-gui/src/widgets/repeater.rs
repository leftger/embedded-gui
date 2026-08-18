use embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
};

use crate::{
    geometry::Rect,
    render::{Compositor, Dither, RenderCtx},
    state::GuiModel,
    style::{Border, Style, VisualState},
    widget::{
        EventContext, EventPolicy, PropertyError, PropertyKey, PropertyValue, Widget, WidgetFlags,
        WidgetId,
    },
};

/// A zero-allocation repeater widget that binds a `GuiModel<T>` to repeated rows/items.
#[derive(Clone, Debug)]
pub struct RepeaterWidget<const MAX_VISIBLE: usize = 8> {
    pub item_height: u32,
    pub spacing: u32,
    pub selected: usize,
    pub scroll_offset: usize,
    pub total_count: usize,
    pub border: Option<Border>,
    pub background: Option<Rgb565>,
    pub selected_bg: Option<Rgb565>,
    pub flags: WidgetFlags,
}

impl<const MAX_VISIBLE: usize> Default for RepeaterWidget<MAX_VISIBLE> {
    fn default() -> Self {
        Self::new(24)
    }
}

impl<const MAX_VISIBLE: usize> RepeaterWidget<MAX_VISIBLE> {
    pub const fn new(item_height: u32) -> Self {
        Self {
            item_height,
            spacing: 2,
            selected: 0,
            scroll_offset: 0,
            total_count: 0,
            border: None,
            background: None,
            selected_bg: Some(Rgb565::new(4, 12, 20)),
            flags: WidgetFlags::from_bits(
                WidgetFlags::CLICKABLE.bits() | WidgetFlags::FOCUSABLE.bits(),
            ),
        }
    }

    pub const fn with_spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    pub const fn with_background(mut self, bg: Rgb565) -> Self {
        self.background = Some(bg);
        self
    }

    pub fn set_selected(&mut self, selected: usize) -> bool {
        let next = selected.min(self.total_count.saturating_sub(1));
        let changed = next != self.selected;
        self.selected = next;
        self.keep_selected_visible();
        changed
    }

    pub fn bump_selection(&mut self, delta: i8) -> bool {
        if self.total_count == 0 {
            return false;
        }
        let next = if delta >= 0 {
            (self.selected + 1) % self.total_count
        } else if self.selected == 0 {
            self.total_count.saturating_sub(1)
        } else {
            self.selected - 1
        };
        self.set_selected(next)
    }

    pub fn keep_selected_visible(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset.saturating_add(MAX_VISIBLE) {
            self.scroll_offset = self.selected.saturating_add(1).saturating_sub(MAX_VISIBLE);
        }
    }

    /// Calculates the layout bounds of a visible item index.
    pub fn item_bounds(&self, bounds: Rect, visible_idx: usize) -> Rect {
        let y_offset = (visible_idx as u32) * (self.item_height + self.spacing);
        Rect::new(
            bounds.x,
            bounds.y + y_offset as i32,
            bounds.w,
            self.item_height,
        )
    }

    /// Renders repeated model items within `bounds` using a render callback.
    pub fn render_model<M, T, D, C, F>(
        &self,
        bounds: Rect,
        model: &M,
        ctx: &mut RenderCtx<'_, D, C>,
        mut render_item: F,
    ) -> Result<(), D::Error>
    where
        M: GuiModel<T>,
        D: DrawTarget<Color = Rgb565>,
        C: Compositor<D>,
        F: FnMut(&T, usize, bool, Rect, &mut RenderCtx<'_, D, C>) -> Result<(), D::Error>,
    {
        let count = model.row_count();
        let visible_count = MAX_VISIBLE.min(count.saturating_sub(self.scroll_offset));

        if let Some(bg) = self.background {
            ctx.fill_rect(bounds, bg)?;
        }

        for i in 0..visible_count {
            let actual_idx = self.scroll_offset + i;
            if let Some(item) = model.row_data(actual_idx) {
                let item_rect = self.item_bounds(bounds, i);
                let is_selected = actual_idx == self.selected;

                if is_selected {
                    if let Some(sel_bg) = self.selected_bg {
                        ctx.fill_rect(item_rect, sel_bg)?;
                    }
                }

                render_item(&item, actual_idx, is_selected, item_rect, ctx)?;
            }
        }

        Ok(())
    }
}

impl<const MAX_VISIBLE: usize> Widget for RepeaterWidget<MAX_VISIBLE> {
    fn handle_widget_event(
        &mut self,
        event: &crate::input::UiEvent,
        _ctx: &mut EventContext,
    ) -> EventPolicy {
        match event {
            crate::input::UiEvent::Scroll { delta, .. } => {
                if *delta > 0 {
                    self.bump_selection(1);
                    EventPolicy::Stop
                } else if *delta < 0 {
                    self.bump_selection(-1);
                    EventPolicy::Stop
                } else {
                    EventPolicy::Continue
                }
            }
            _ => EventPolicy::Continue,
        }
    }

    fn get_property(&self, key: PropertyKey) -> Option<PropertyValue<'_>> {
        match key {
            PropertyKey::Selected => Some(PropertyValue::Usize(self.selected)),
            PropertyKey::Offset => Some(PropertyValue::Usize(self.scroll_offset)),
            PropertyKey::Value => Some(PropertyValue::Usize(self.total_count)),
            _ => None,
        }
    }

    fn set_property<'a>(
        &mut self,
        key: PropertyKey,
        val: PropertyValue<'a>,
    ) -> Result<(), PropertyError> {
        match (key, val) {
            (PropertyKey::Selected, PropertyValue::Usize(s)) => {
                self.set_selected(s);
                Ok(())
            }
            (PropertyKey::Offset, PropertyValue::Usize(off)) => {
                self.scroll_offset = off;
                Ok(())
            }
            (PropertyKey::Value, PropertyValue::Usize(cnt)) => {
                self.total_count = cnt;
                Ok(())
            }
            _ => Err(PropertyError::NotFound),
        }
    }
}
