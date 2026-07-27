use embedded_graphics_core::pixelcolor::Rgb565;

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    geometry::Rect,
    input::{UiEvent, WidgetEvent, WidgetEventKind},
    layout::{Axis, LayoutItem, LinearLayout},
    state::{FeedTimelineState, ListState, ScrollState, SliderState, TabsState},
    widget::{EventContext, EventPhase, EventPolicy, FocusGroupId, WidgetFlags, WidgetId},
    widgets::{KeyboardLayout, SurfaceState, TEXTAREA_CAPACITY, WidgetKind, WidgetNode},
};

use super::*;

impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    GuiContext<'a, NODES, EVENTS, DIRTY>
{
    pub fn set_progress(&mut self, id: WidgetId, value: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::ProgressBar { value: ref mut v } => {
                *v = value.clamp(0.0, 1.0);
                self.dirty.add(rect)?;
                Ok(())
            }
            WidgetKind::PeekReveal {
                progress: ref mut v,
                ..
            } => {
                *v = value.clamp(0.0, 1.0);
                self.dirty.add(rect)?;
                Ok(())
            }
            WidgetKind::SweepingArc {
                progress: ref mut v,
                ..
            } => {
                *v = value.clamp(0.0, 1.0);
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_glance_highlighted(
        &mut self,
        id: WidgetId,
        highlighted: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::GlanceTile {
                highlighted: ref mut h,
                ..
            } => {
                *h = highlighted;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_card_deck_selected(
        &mut self,
        id: WidgetId,
        selected: usize,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::CardDeck {
                titles,
                selected: ref mut current,
            } => {
                *current = selected.min(titles.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tick_reel(&mut self, id: WidgetId, dt_ms: u32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Reel {
                player: ref mut reel,
                ..
            } => {
                reel.tick(dt_ms);
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_state_surface_state(
        &mut self,
        id: WidgetId,
        state: SurfaceState,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::StateSurface {
                state: ref mut current,
                ..
            } => {
                *current = state;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_state_surface_message(
        &mut self,
        id: WidgetId,
        message: &'a str,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::StateSurface {
                message: ref mut current,
                ..
            } => {
                *current = message;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_state_surface_action(
        &mut self,
        id: WidgetId,
        action: Option<&'a str>,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::StateSurface {
                action: ref mut current,
                ..
            } => {
                *current = action;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_state_surface_busy_phase(
        &mut self,
        id: WidgetId,
        phase: f32,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::StateSurface {
                busy_phase: ref mut current,
                ..
            } => {
                *current = phase;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tick_state_surface(
        &mut self,
        id: WidgetId,
        dt_ms: u32,
        cycles_per_sec: f32,
    ) -> Result<(), GuiError> {
        let phase = match self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::StateSurface { busy_phase, .. } => {
                busy_phase + (dt_ms as f32 / 1000.0) * cycles_per_sec
            }
            _ => return Err(GuiError::NotFound),
        };
        self.set_state_surface_busy_phase(id, phase)
    }

    pub fn set_heads_up_ttl(&mut self, id: WidgetId, ttl_ms: u32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::HeadsUpBanner {
                ttl_ms: ref mut current,
                ..
            } => {
                *current = ttl_ms;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tick_heads_up(&mut self, id: WidgetId, dt_ms: u32) -> Result<(), GuiError> {
        let ttl = match self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::HeadsUpBanner { ttl_ms, .. } => ttl_ms.saturating_sub(dt_ms),
            _ => return Err(GuiError::NotFound),
        };
        self.set_heads_up_ttl(id, ttl)
    }

    pub fn set_notification_sheet_open(
        &mut self,
        id: WidgetId,
        open: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::NotificationActionSheet {
                open: ref mut current,
                ..
            } => {
                *current = open;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_notification_sheet_selected(
        &mut self,
        id: WidgetId,
        selected: usize,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::NotificationActionSheet {
                actions,
                selected: ref mut current,
                ..
            } => {
                *current = selected.min(actions.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_menu_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Menu {
                items,
                selected: ref mut current,
            } => {
                *current = selected.min(items.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn menu_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::Menu { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn list_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::List { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_list_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::List {
                items,
                selected: ref mut current,
                ref mut offset,
                visible_rows,
            } => {
                let mut state = ListState::new(*current, *offset, visible_rows);
                state.set_selected(selected, items.len());
                *current = state.selected;
                *offset = state.offset;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn feed_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::FeedTimeline { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_feed_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::FeedTimeline {
                items,
                selected: ref mut current,
                ref mut offset,
                visible_rows,
                ..
            } => {
                let mut state = FeedTimelineState::new(*current, *offset, visible_rows, false);
                state.set_selected(selected, items.len());
                *current = state.selected;
                *offset = state.offset;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_feed_expanded(&mut self, id: WidgetId, expanded: bool) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::FeedTimeline {
                expanded: ref mut current,
                ..
            } => {
                *current = expanded;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_toggle(&mut self, id: WidgetId, on: bool) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Toggle { on: ref mut v, .. } => {
                *v = on;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn toggle_value(&self, id: WidgetId) -> Option<bool> {
        match self.node(id)?.kind {
            WidgetKind::Toggle { on, .. } => Some(on),
            _ => None,
        }
    }

    pub fn set_checked(&mut self, id: WidgetId, checked: bool) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Checkbox {
                checked: ref mut v, ..
            } => {
                *v = checked;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn checked_value(&self, id: WidgetId) -> Option<bool> {
        match self.node(id)?.kind {
            WidgetKind::Checkbox { checked, .. } => Some(checked),
            _ => None,
        }
    }

    pub fn set_slider_value(&mut self, id: WidgetId, value: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Slider {
                value: ref mut v,
                min,
                max,
            } => {
                let mut state = SliderState::new(*v, min, max);
                state.set_value(value);
                *v = state.value;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn slider_value(&self, id: WidgetId) -> Option<f32> {
        match self.node(id)?.kind {
            WidgetKind::Slider { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn set_value_label(&mut self, id: WidgetId, value: i32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::ValueLabel {
                value: ref mut v, ..
            } => {
                *v = value;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_scroll_offset(&mut self, id: WidgetId, offset_y: i32) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::ScrollView {
                offset_y: ref mut v,
                content_h,
            } => {
                let mut state = ScrollState::new(*v, content_h);
                state.set_offset(offset_y);
                *v = state.offset_y;
                self.mark_subtree_dirty(id)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn scroll_offset(&self, id: WidgetId) -> Option<i32> {
        match self.node(id)?.kind {
            WidgetKind::ScrollView { offset_y, .. } => Some(offset_y),
            _ => None,
        }
    }

    pub fn set_tab_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Tabs {
                labels,
                selected: ref mut v,
            } => {
                let mut state = TabsState::new(*v);
                state.set_selected(selected, labels.len());
                *v = state.selected;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tab_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::Tabs { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_toast_ttl(&mut self, id: WidgetId, ttl_ms: u32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Toast {
                ttl_ms: ref mut v, ..
            } => {
                *v = ttl_ms;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tick_toast(&mut self, id: WidgetId, dt_ms: u32) -> Result<(), GuiError> {
        let ttl = match self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::Toast { ttl_ms, .. } => ttl_ms.saturating_sub(dt_ms),
            _ => return Err(GuiError::NotFound),
        };
        self.set_toast_ttl(id, ttl)
    }

    pub fn set_meter_value(&mut self, id: WidgetId, value: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Meter {
                value: ref mut v,
                min,
                max,
            } => {
                *v = value.clamp(min.min(max), min.max(max));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_spinner_phase(&mut self, id: WidgetId, phase: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Spinner { phase: ref mut v } => {
                *v = phase;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn tick_spinner(
        &mut self,
        id: WidgetId,
        dt_ms: u32,
        cycles_per_sec: f32,
    ) -> Result<(), GuiError> {
        let phase = match self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::Spinner { phase } => phase + (dt_ms as f32 / 1000.0) * cycles_per_sec,
            _ => return Err(GuiError::NotFound),
        };
        self.set_spinner_phase(id, phase)
    }

    pub fn set_dropdown_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Dropdown {
                items,
                selected: ref mut current,
                ..
            } => {
                *current = selected.min(items.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn dropdown_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::Dropdown { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_dropdown_open(&mut self, id: WidgetId, open: bool) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Dropdown {
                open: ref mut is_open,
                ..
            } => {
                if *is_open != open {
                    *is_open = open;
                    self.dirty.add(rect)?;
                    self.push_event(if open {
                        UiEvent::Opened(id)
                    } else {
                        UiEvent::Closed(id)
                    })?;
                }
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn dropdown_open(&self, id: WidgetId) -> Option<bool> {
        match self.node(id)?.kind {
            WidgetKind::Dropdown { open, .. } => Some(open),
            _ => None,
        }
    }

    pub fn set_roller_selected(&mut self, id: WidgetId, selected: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Roller {
                items,
                selected: ref mut current,
            } => {
                *current = selected.min(items.len().saturating_sub(1));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn roller_selected(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::Roller { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn set_textarea_text(&mut self, id: WidgetId, text: &'a str) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf: ref mut buf,
                text_len: ref mut len,
                cursor: ref mut c,
                ..
            } => {
                let (next_buf, next_len) = textarea_storage_from_str(text);
                *buf = next_buf;
                *len = next_len;
                *c = (*c).min(textarea_text(buf, *len).chars().count());
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn textarea_text(&self, id: WidgetId) -> Option<&str> {
        match &self.node(id)?.kind {
            WidgetKind::TextArea {
                text_buf, text_len, ..
            } => Some(textarea_text(text_buf, *text_len)),
            _ => None,
        }
    }

    pub fn set_textarea_cursor(&mut self, id: WidgetId, cursor: usize) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor: ref mut current,
                ..
            } => {
                let text = textarea_text(&text_buf, text_len);
                *current = cursor.min(text.chars().count());
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn move_textarea_cursor(&mut self, id: WidgetId, delta: i8) -> Result<(), GuiError> {
        let next = self.textarea_cursor(id).ok_or(GuiError::NotFound)? as i32 + delta as i32;
        self.set_textarea_cursor_with_extend(id, next.max(0) as usize, false)
    }

    pub fn move_textarea_cursor_select(&mut self, id: WidgetId, delta: i8) -> Result<(), GuiError> {
        let next = self.textarea_cursor(id).ok_or(GuiError::NotFound)? as i32 + delta as i32;
        self.set_textarea_cursor_with_extend(id, next.max(0) as usize, true)
    }

    pub fn move_textarea_cursor_word(&mut self, id: WidgetId, delta: i8) -> Result<(), GuiError> {
        let (text, cursor) = match &self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                ..
            } => (textarea_text(text_buf, *text_len), *cursor),
            _ => return Err(GuiError::NotFound),
        };
        let next = if delta >= 0 {
            next_word_boundary(text, cursor)
        } else {
            prev_word_boundary(text, cursor)
        };
        self.set_textarea_cursor_with_extend(id, next, false)
    }

    pub fn move_textarea_cursor_word_select(
        &mut self,
        id: WidgetId,
        delta: i8,
    ) -> Result<(), GuiError> {
        let (text, cursor) = match &self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                ..
            } => (textarea_text(text_buf, *text_len), *cursor),
            _ => return Err(GuiError::NotFound),
        };
        let next = if delta >= 0 {
            next_word_boundary(text, cursor)
        } else {
            prev_word_boundary(text, cursor)
        };
        self.set_textarea_cursor_with_extend(id, next, true)
    }

    pub fn set_textarea_cursor_home(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.set_textarea_cursor(id, 0)
    }

    pub fn set_textarea_cursor_end(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let len = self
            .textarea_text(id)
            .map(|text| text.chars().count())
            .ok_or(GuiError::NotFound)?;
        self.set_textarea_cursor(id, len)
    }

    pub fn set_textarea_cursor_line_home(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let (text, cursor, wrap_cols) = self.textarea_line_context(id)?;
        let (row, _) = textarea_row_col_at_cursor(text, cursor, wrap_cols);
        let next = textarea_cursor_from_row_col(text, row, 0, wrap_cols);
        self.set_textarea_cursor_with_extend(id, next, false)
    }

    pub fn set_textarea_cursor_line_home_select(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let (text, cursor, wrap_cols) = self.textarea_line_context(id)?;
        let (row, _) = textarea_row_col_at_cursor(text, cursor, wrap_cols);
        let next = textarea_cursor_from_row_col(text, row, 0, wrap_cols);
        self.set_textarea_cursor_with_extend(id, next, true)
    }

    pub fn set_textarea_cursor_line_end(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let (text, cursor, wrap_cols) = self.textarea_line_context(id)?;
        let (row, _) = textarea_row_col_at_cursor(text, cursor, wrap_cols);
        let row_end = textarea_row_end_col(text, row, wrap_cols);
        let next = textarea_cursor_from_row_col(text, row, row_end, wrap_cols);
        self.set_textarea_cursor_with_extend(id, next, false)
    }

    pub fn set_textarea_cursor_line_end_select(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let (text, cursor, wrap_cols) = self.textarea_line_context(id)?;
        let (row, _) = textarea_row_col_at_cursor(text, cursor, wrap_cols);
        let row_end = textarea_row_end_col(text, row, wrap_cols);
        let next = textarea_cursor_from_row_col(text, row, row_end, wrap_cols);
        self.set_textarea_cursor_with_extend(id, next, true)
    }

    pub fn textarea_cursor(&self, id: WidgetId) -> Option<usize> {
        match self.node(id)?.kind {
            WidgetKind::TextArea { cursor, .. } => Some(cursor),
            _ => None,
        }
    }

    pub fn set_textarea_selection(
        &mut self,
        id: WidgetId,
        start: usize,
        end: usize,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                selection: ref mut current,
                ..
            } => {
                let text = textarea_text(&text_buf, text_len);
                let len = text.chars().count();
                let start = start.min(len);
                let end = end.min(len);
                *current = Some((start.min(end), start.max(end)));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn clear_textarea_selection(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                selection: ref mut current,
                ..
            } => {
                *current = None;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn textarea_selection(&self, id: WidgetId) -> Option<(usize, usize)> {
        match self.node(id)?.kind {
            WidgetKind::TextArea { selection, .. } => selection,
            _ => None,
        }
    }

    pub fn textarea_cursor_visible(&self, id: WidgetId) -> Option<bool> {
        match self.node(id)?.kind {
            WidgetKind::TextArea { cursor_visible, .. } => Some(cursor_visible),
            _ => None,
        }
    }

    pub fn set_textarea_capabilities(
        &mut self,
        id: WidgetId,
        read_only: bool,
        single_line: bool,
        accept_newline: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                read_only: ref mut ro,
                single_line: ref mut sl,
                accept_newline: ref mut an,
                ..
            } => {
                *ro = read_only;
                *sl = single_line;
                *an = accept_newline && !single_line;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn textarea_insert_char(&mut self, id: WidgetId, ch: char) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let before = self.capture_textarea_snapshot(id)?;
        let mut emit = false;
        if let Some(node) = self.node_mut(id) {
            if let WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                selection,
                read_only,
                single_line,
                accept_newline,
                ..
            } = &mut node.kind
            {
                if *read_only {
                    return Ok(());
                }
                if ch == '\n' && (*single_line || !*accept_newline) {
                    return Ok(());
                }
                let mut chars: heapless::Vec<char, TEXTAREA_CAPACITY> = heapless::Vec::new();
                for c in textarea_text(text_buf, *text_len).chars() {
                    let _ = chars.push(c);
                }
                let original_len = chars.len();
                let original_cursor = *cursor;

                if ch == '\u{8}' {
                    let removed_selection = delete_selection_if_any(&mut chars, cursor, selection);
                    if !removed_selection && *cursor > 0 && *cursor <= chars.len() {
                        chars.remove(*cursor - 1);
                        *cursor -= 1;
                    }
                    if removed_selection
                        || *cursor != original_cursor
                        || chars.len() != original_len
                    {
                        *selection = None;
                        let (next_buf, next_len) = textarea_storage_from_chars(&chars);
                        *text_buf = next_buf;
                        *text_len = next_len;
                        emit = true;
                    }
                } else if ch == '\u{7f}' {
                    let removed_selection = delete_selection_if_any(&mut chars, cursor, selection);
                    if !removed_selection && *cursor < chars.len() {
                        chars.remove(*cursor);
                    }
                    if removed_selection || chars.len() != original_len {
                        *selection = None;
                        let (next_buf, next_len) = textarea_storage_from_chars(&chars);
                        *text_buf = next_buf;
                        *text_len = next_len;
                        emit = true;
                    }
                } else if ch != '\n' || *cursor < TEXTAREA_CAPACITY {
                    if delete_selection_if_any(&mut chars, cursor, selection) {
                        *selection = None;
                    }
                    if chars.len() < TEXTAREA_CAPACITY && *cursor <= chars.len() {
                        let _ = chars.insert(*cursor, ch);
                        *cursor += 1;
                        *selection = None;
                        let (next_buf, next_len) = textarea_storage_from_chars(&chars);
                        *text_buf = next_buf;
                        *text_len = next_len;
                        emit = true;
                    }
                }
            } else {
                return Err(GuiError::NotFound);
            }
        }
        if emit {
            self.push_textarea_undo(id, before);
            self.clear_textarea_redo_for(id);
            self.dirty.add(rect)?;
            self.push_event(UiEvent::TextInput { id, ch })?;
            self.push_event(UiEvent::ValueChanged(id))?;
        }
        Ok(())
    }

    pub(crate) fn textarea_line_context(
        &self,
        id: WidgetId,
    ) -> Result<(&str, usize, usize), GuiError> {
        let node = self.node(id).ok_or(GuiError::NotFound)?;
        match &node.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                ..
            } => {
                let font = node.style.normal.font;
                let inner_w = node.rect.w.saturating_sub(2);
                let cols = (inner_w / font.advance()).max(1) as usize;
                Ok((textarea_text(text_buf, *text_len), *cursor, cols))
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub(crate) fn set_textarea_cursor_with_extend(
        &mut self,
        id: WidgetId,
        cursor: usize,
        extend_selection: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor: ref mut current_cursor,
                ref mut selection,
                ..
            } => {
                let len = textarea_text(&text_buf, text_len).chars().count();
                let next = cursor.min(len);
                if extend_selection {
                    let anchor = match *selection {
                        Some((start, end)) => {
                            if *current_cursor == start {
                                end
                            } else {
                                start
                            }
                        }
                        None => *current_cursor,
                    };
                    if anchor == next {
                        *selection = None;
                    } else {
                        *selection = Some((anchor.min(next), anchor.max(next)));
                    }
                } else {
                    *selection = None;
                }
                *current_cursor = next;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub(crate) fn capture_textarea_snapshot(
        &self,
        id: WidgetId,
    ) -> Result<TextareaSnapshot, GuiError> {
        match self.node(id).ok_or(GuiError::NotFound)?.kind {
            WidgetKind::TextArea {
                text_buf,
                text_len,
                cursor,
                selection,
                ..
            } => Ok(TextareaSnapshot {
                text_buf,
                text_len,
                cursor,
                selection,
            }),
            _ => Err(GuiError::NotFound),
        }
    }

    pub(crate) fn apply_textarea_snapshot(
        &mut self,
        id: WidgetId,
        snap: TextareaSnapshot,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::TextArea {
                text_buf: ref mut buf,
                text_len: ref mut len,
                cursor: ref mut c,
                selection: ref mut sel,
                ..
            } => {
                *buf = snap.text_buf;
                *len = snap.text_len;
                *c = snap.cursor;
                *sel = snap.selection;
                self.dirty.add(rect)?;
                self.push_event(UiEvent::ValueChanged(id))
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub(crate) fn push_textarea_undo(&mut self, id: WidgetId, snapshot: TextareaSnapshot) {
        if self.textarea_undo.len() == self.textarea_undo.capacity() {
            self.textarea_undo.remove(0);
        }
        let _ = self
            .textarea_undo
            .push(TextareaHistoryEntry { id, snapshot });
    }

    pub(crate) fn push_textarea_redo(&mut self, id: WidgetId, snapshot: TextareaSnapshot) {
        if self.textarea_redo.len() == self.textarea_redo.capacity() {
            self.textarea_redo.remove(0);
        }
        let _ = self
            .textarea_redo
            .push(TextareaHistoryEntry { id, snapshot });
    }

    pub(crate) fn clear_textarea_redo_for(&mut self, id: WidgetId) {
        let mut i = 0usize;
        while i < self.textarea_redo.len() {
            if self.textarea_redo[i].id == id {
                self.textarea_redo.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub(crate) fn textarea_undo(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let Some(pos) = self.textarea_undo.iter().rposition(|entry| entry.id == id) else {
            return Ok(());
        };
        let current = self.capture_textarea_snapshot(id)?;
        let prior = self.textarea_undo.remove(pos).snapshot;
        self.push_textarea_redo(id, current);
        self.apply_textarea_snapshot(id, prior)
    }

    pub(crate) fn textarea_redo(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let Some(pos) = self.textarea_redo.iter().rposition(|entry| entry.id == id) else {
            return Ok(());
        };
        let current = self.capture_textarea_snapshot(id)?;
        let next = self.textarea_redo.remove(pos).snapshot;
        self.push_textarea_undo(id, current);
        self.apply_textarea_snapshot(id, next)
    }

    pub fn textarea_backspace(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.textarea_insert_char(id, '\u{8}')
    }

    pub fn textarea_delete_forward(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.textarea_insert_char(id, '\u{7f}')
    }

    pub fn keyboard_selected_key(&self, id: WidgetId) -> Option<char> {
        match self.node(id)?.kind {
            WidgetKind::Keyboard {
                keys,
                alt_keys,
                selected,
                layout,
                ..
            } => keyboard_char_for_layout(keys, alt_keys, selected, layout),
            _ => None,
        }
    }

    pub fn keyboard_layout(&self, id: WidgetId) -> Option<KeyboardLayout> {
        match self.node(id)?.kind {
            WidgetKind::Keyboard { layout, .. } => Some(layout),
            _ => None,
        }
    }

    pub fn set_keyboard_layout(
        &mut self,
        id: WidgetId,
        layout: KeyboardLayout,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Keyboard {
                layout: ref mut current,
                ..
            } => {
                *current = layout;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_keyboard_target(
        &mut self,
        id: WidgetId,
        target: Option<WidgetId>,
    ) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Keyboard {
                target: ref mut current,
                ..
            } => {
                *current = target;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_gauge_value(&mut self, id: WidgetId, value: f32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Gauge {
                value: ref mut v,
                min,
                max,
                ..
            }
            | WidgetKind::ArcGauge {
                value: ref mut v,
                min,
                max,
                ..
            }
            | WidgetKind::GaugeNeedle {
                value: ref mut v,
                min,
                max,
                ..
            } => {
                *v = value.clamp(min.min(max), min.max(max));
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_gauge_ticks(
        &mut self,
        id: WidgetId,
        major_ticks: u8,
        minor_ticks: u8,
        show_value: bool,
    ) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        match node.kind {
            WidgetKind::Gauge {
                major_ticks: ref mut major,
                minor_ticks: ref mut minor,
                show_value: ref mut show,
                ..
            }
            | WidgetKind::ArcGauge {
                major_ticks: ref mut major,
                minor_ticks: ref mut minor,
                show_value: ref mut show,
                ..
            } => {
                *major = major_ticks.max(1);
                *minor = minor_ticks.max(1);
                *show = show_value;
                self.dirty.add(rect)?;
                Ok(())
            }
            _ => Err(GuiError::NotFound),
        }
    }

    pub fn set_widget_rect(&mut self, id: WidgetId, rect: Rect) -> Result<(), GuiError> {
        let old = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.rect = rect;
        self.dirty.add(old)?;
        self.mark_subtree_dirty(id)?;
        Ok(())
    }

    pub fn set_widget_x(&mut self, id: WidgetId, x: i32) -> Result<(), GuiError> {
        let mut rect = self.node(id).ok_or(GuiError::NotFound)?.rect;
        rect.x = x;
        self.set_widget_rect(id, rect)
    }

    pub fn set_widget_y(&mut self, id: WidgetId, y: i32) -> Result<(), GuiError> {
        let mut rect = self.node(id).ok_or(GuiError::NotFound)?.rect;
        rect.y = y;
        self.set_widget_rect(id, rect)
    }

    pub fn set_widget_width(&mut self, id: WidgetId, w: u32) -> Result<(), GuiError> {
        let mut rect = self.node(id).ok_or(GuiError::NotFound)?.rect;
        rect.w = w.max(1);
        self.set_widget_rect(id, rect)
    }

    pub fn set_widget_height(&mut self, id: WidgetId, h: u32) -> Result<(), GuiError> {
        let mut rect = self.node(id).ok_or(GuiError::NotFound)?.rect;
        rect.h = h.max(1);
        self.set_widget_rect(id, rect)
    }

    pub fn set_widget_opacity(&mut self, id: WidgetId, opacity: u8) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.style.normal.opacity = opacity;
        node.style.focused.opacity = opacity;
        node.style.pressed.opacity = opacity;
        node.style.disabled.opacity = opacity;
        self.mark_subtree_dirty(id)
    }

    pub fn set_widget_corner_radius(&mut self, id: WidgetId, radius: u8) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.style.normal.corner_radius = radius;
        node.style.focused.corner_radius = radius;
        node.style.pressed.corner_radius = radius;
        node.style.disabled.corner_radius = radius;
        self.mark_subtree_dirty(id)
    }

    pub fn set_widget_accent(&mut self, id: WidgetId, accent: Rgb565) -> Result<(), GuiError> {
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        node.style.normal.accent = accent;
        node.style.focused.accent = accent;
        node.style.pressed.accent = accent;
        node.style.disabled.accent = accent;
        self.mark_subtree_dirty(id)
    }

    pub fn set_widget_parent(
        &mut self,
        id: WidgetId,
        parent: Option<WidgetId>,
    ) -> Result<(), GuiError> {
        if let Some(parent) = parent {
            self.node(parent).ok_or(GuiError::NotFound)?;
        }
        self.node_mut(id).ok_or(GuiError::NotFound)?.parent = parent;
        self.mark_subtree_dirty(id)?;
        Ok(())
    }

    pub fn add_child(&mut self, parent: WidgetId, child: WidgetId) -> Result<(), GuiError> {
        self.set_widget_parent(child, Some(parent))
    }

    pub fn children_of(&self, parent: WidgetId) -> impl Iterator<Item = &WidgetNode<'a>> + '_ {
        self.widgets
            .iter()
            .filter(move |node| node.parent == Some(parent))
    }

    #[inline]
    pub fn absolute_rect(&self, id: WidgetId) -> Option<Rect> {
        let node = self.node(id)?;
        if node.parent.is_none() {
            return Some(node.rect);
        }
        let mut rect = node.rect;
        let mut parent = node.parent;
        let mut depth = 0;
        while let Some(parent_id) = parent {
            if depth >= NODES {
                return None;
            }
            let parent_node = self.node(parent_id)?;
            rect.x += parent_node.rect.x;
            rect.y += parent_node.rect.y;
            parent = parent_node.parent;
            depth += 1;
        }
        Some(rect)
    }

    pub fn set_flag(
        &mut self,
        id: WidgetId,
        flag: WidgetFlags,
        enabled: bool,
    ) -> Result<(), GuiError> {
        let was_set = self.has_flag(id, flag)?;
        let before_state = self.current_visual_state(id);
        self.mark_subtree_dirty(id)?;
        self.node_mut(id)
            .ok_or(GuiError::NotFound)?
            .flags
            .set(flag, enabled);
        if flag == WidgetFlags::DISABLED
            && enabled
            && self.pressed.is_some_and(|pressed| pressed.id == id)
        {
            self.pressed = None;
        }
        self.mark_subtree_dirty(id)?;
        if self
            .focus
            .is_some_and(|focus| !self.effective_focusable(focus))
        {
            self.focus = None;
            self.ensure_focus();
        }
        if flag == WidgetFlags::DISABLED && was_set != enabled {
            let after_state = self.current_visual_state(id);
            self.start_state_transition(id, before_state, after_state);
        }
        Ok(())
    }

    pub fn has_flag(&self, id: WidgetId, flag: WidgetFlags) -> Result<bool, GuiError> {
        Ok(self
            .node(id)
            .ok_or(GuiError::NotFound)?
            .flags
            .contains(flag))
    }

    pub fn insert_flag(&mut self, id: WidgetId, flag: WidgetFlags) -> Result<(), GuiError> {
        self.set_flag(id, flag, true)
    }

    pub fn remove_flag(&mut self, id: WidgetId, flag: WidgetFlags) -> Result<(), GuiError> {
        self.set_flag(id, flag, false)
    }

    pub fn set_hidden(&mut self, id: WidgetId, hidden: bool) -> Result<(), GuiError> {
        self.set_flag(id, WidgetFlags::HIDDEN, hidden)
    }

    pub fn set_disabled(&mut self, id: WidgetId, disabled: bool) -> Result<(), GuiError> {
        self.set_flag(id, WidgetFlags::DISABLED, disabled)
    }

    pub fn set_clickable(&mut self, id: WidgetId, clickable: bool) -> Result<(), GuiError> {
        self.set_flag(id, WidgetFlags::CLICKABLE, clickable)
    }

    pub fn set_scrollable(&mut self, id: WidgetId, scrollable: bool) -> Result<(), GuiError> {
        self.set_flag(id, WidgetFlags::SCROLLABLE, scrollable)
    }

    pub fn set_visible(&mut self, id: WidgetId, visible: bool) -> Result<(), GuiError> {
        self.set_hidden(id, !visible)
    }

    pub fn set_enabled(&mut self, id: WidgetId, enabled: bool) -> Result<(), GuiError> {
        self.set_disabled(id, !enabled)
    }

    pub fn event_path<const M: usize>(
        &self,
        target: WidgetId,
        out: &mut heapless::Vec<EventContext, M>,
    ) -> Result<usize, GuiError> {
        self.node(target).ok_or(GuiError::NotFound)?;
        out.clear();

        let mut chain = heapless::Vec::<WidgetId, NODES>::new();
        let mut current = Some(target);
        while let Some(id) = current {
            chain.push(id).map_err(|_| GuiError::WidgetsFull)?;
            current = self.node(id).ok_or(GuiError::NotFound)?.parent;
        }

        for id in chain.iter().rev().copied().filter(|&id| id != target) {
            out.push(EventContext {
                target,
                current: id,
                phase: EventPhase::Capture,
            })
            .map_err(|_| GuiError::EventsFull)?;
        }

        out.push(EventContext {
            target,
            current: target,
            phase: EventPhase::Target,
        })
        .map_err(|_| GuiError::EventsFull)?;

        for id in chain.iter().copied().skip(1) {
            out.push(EventContext {
                target,
                current: id,
                phase: EventPhase::Bubble,
            })
            .map_err(|_| GuiError::EventsFull)?;
        }

        Ok(out.len())
    }

    pub fn widget_event_path<const M: usize>(
        &self,
        target: WidgetId,
        kind: WidgetEventKind,
        out: &mut heapless::Vec<WidgetEvent, M>,
    ) -> Result<usize, GuiError> {
        self.node(target).ok_or(GuiError::NotFound)?;
        out.clear();

        let mut chain = heapless::Vec::<WidgetId, NODES>::new();
        let mut current = Some(target);
        while let Some(id) = current {
            chain.push(id).map_err(|_| GuiError::WidgetsFull)?;
            current = self.node(id).ok_or(GuiError::NotFound)?.parent;
        }

        for id in chain.iter().rev().copied().filter(|&id| id != target) {
            out.push(WidgetEvent {
                target,
                current: id,
                phase: EventPhase::Capture,
                kind,
            })
            .map_err(|_| GuiError::EventsFull)?;
        }

        out.push(WidgetEvent {
            target,
            current: target,
            phase: EventPhase::Target,
            kind,
        })
        .map_err(|_| GuiError::EventsFull)?;

        if self.has_flag(target, WidgetFlags::EVENT_BUBBLE)? {
            for id in chain.iter().copied().skip(1) {
                out.push(WidgetEvent {
                    target,
                    current: id,
                    phase: EventPhase::Bubble,
                    kind,
                })
                .map_err(|_| GuiError::EventsFull)?;
            }
        }

        Ok(out.len())
    }

    pub fn dispatch_widget_event<const M: usize, F>(
        &self,
        target: WidgetId,
        kind: WidgetEventKind,
        scratch: &mut heapless::Vec<WidgetEvent, M>,
        mut handler: F,
    ) -> Result<(), GuiError>
    where
        F: FnMut(WidgetEvent) -> EventPolicy,
    {
        self.widget_event_path(target, kind, scratch)?;
        for event in scratch.iter().copied() {
            let handler_policy = handler(event);
            if matches!(handler_policy, EventPolicy::Stop)
                || self.stop_due_to_builtin_widget_behavior(event)
                || self.stop_due_to_registered_policy(event)
            {
                break;
            }
        }
        Ok(())
    }

    pub fn mark_subtree_dirty(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        self.dirty.add(rect)?;
        let child_ids: heapless::Vec<WidgetId, NODES> = self
            .widgets
            .iter()
            .filter(|node| node.parent == Some(id))
            .map(|node| node.id)
            .collect();
        for child in child_ids {
            self.mark_subtree_dirty(child)?;
        }
        Ok(())
    }

    pub fn set_focus_group(&mut self, id: WidgetId, group: FocusGroupId) -> Result<(), GuiError> {
        self.node_mut(id).ok_or(GuiError::NotFound)?.focus_group = group;
        Ok(())
    }

    pub fn set_active_focus_group(&mut self, group: Option<FocusGroupId>) {
        self.active_focus_group = group;
        if let Some(focus) = self.focus {
            let still_valid = self.node(focus).is_some_and(|node| {
                group.is_none_or(|active| node.focus_group == active)
                    && self.effective_focusable(focus)
            });
            if !still_valid {
                self.focus = None;
                self.ensure_focus();
            }
        }
    }

    pub fn apply_layout(
        &mut self,
        layout: LinearLayout,
        area: Rect,
        ids: &[WidgetId],
    ) -> Result<usize, GuiError> {
        let mut rects = [Rect::empty(); 16];
        let count = layout.arrange(area, ids.len().min(rects.len()), &mut rects);
        for (id, rect) in ids.iter().copied().zip(rects).take(count) {
            self.set_widget_rect(id, rect)?;
        }
        Ok(count)
    }

    pub fn apply_layout_flex(
        &mut self,
        layout: LinearLayout,
        area: Rect,
        ids: &[WidgetId],
        items: &[LayoutItem],
        enable_grow: bool,
        enable_shrink: bool,
    ) -> Result<usize, GuiError> {
        let mut rects = [Rect::empty(); 16];
        let count = ids.len().min(items.len()).min(rects.len());
        let laid_out = layout.arrange_items_flex(
            area,
            &items[..count],
            &mut rects,
            enable_grow,
            enable_shrink,
        );
        for (id, rect) in ids.iter().copied().zip(rects).take(laid_out) {
            self.set_widget_rect(id, rect)?;
        }
        Ok(laid_out)
    }

    pub fn apply_layout_intrinsic(
        &mut self,
        layout: LinearLayout,
        area: Rect,
        ids: &[WidgetId],
    ) -> Result<usize, GuiError> {
        self.apply_layout_intrinsic_with_cross(layout, area, ids, false)
    }

    pub fn apply_layout_intrinsic_with_cross(
        &mut self,
        layout: LinearLayout,
        area: Rect,
        ids: &[WidgetId],
        preserve_cross: bool,
    ) -> Result<usize, GuiError> {
        let mut specs = [LayoutItem::fill(); 16];
        let mut rects = [Rect::empty(); 16];
        let count = ids.len().min(specs.len()).min(rects.len());

        for (idx, id) in ids.iter().copied().take(count).enumerate() {
            let (w, h) = self.intrinsic_size(id).ok_or(GuiError::NotFound)?;
            specs[idx] = match layout.axis {
                Axis::Horizontal => LayoutItem::length(w).with_cross(if preserve_cross {
                    crate::layout::Constraint::Length(h)
                } else {
                    crate::layout::Constraint::Fill(1)
                }),
                Axis::Vertical => LayoutItem::length(h).with_cross(if preserve_cross {
                    crate::layout::Constraint::Length(w)
                } else {
                    crate::layout::Constraint::Fill(1)
                }),
            };
        }

        let laid_out = layout.arrange_items(area, &specs[..count], &mut rects);
        for (id, rect) in ids.iter().copied().zip(rects).take(laid_out) {
            self.set_widget_rect(id, rect)?;
        }
        Ok(laid_out)
    }
}
