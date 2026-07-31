#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    geometry::Rect,
    input::{
        InputEvent, PointerState, UiEvent, UiEventFilter, WidgetDispatchPolicy, WidgetEvent,
        WidgetEventKind,
    },
    state::{FeedTimelineState, ListState, ScrollState, SliderState, TabsState},
    style::{VisualState, WidgetStyle},
    widget::{EventPhase, EventPolicy, WidgetId},
    widgets::{KeyboardLayout, TEXTAREA_CAPACITY, WidgetKind, WidgetNode},
    haptics::HapticPattern,
};

use super::*;

impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
    GuiContext<'a, NODES, EVENTS, DIRTY>
{
    pub fn handle_input(&mut self, event: InputEvent) -> Result<(), GuiError> {
        match event {
            InputEvent::Home => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.set_textarea_cursor_line_home(id)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::End => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.set_textarea_cursor_line_end(id)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::WordLeft => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_word(id, -1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::WordRight => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_word(id, 1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::Undo => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.textarea_undo(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::Redo => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.textarea_redo(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::SelectLeft => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_select(id, -1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectRight => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_select(id, 1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectHome => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.set_textarea_cursor_line_home_select(id)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectEnd => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.set_textarea_cursor_line_end_select(id)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectWordLeft => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_word_select(id, -1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::SelectWordRight => {
                if let Some(id) = self.focus {
                    if matches!(
                        self.node(id).map(|n| n.kind),
                        Some(WidgetKind::TextArea { .. })
                    ) {
                        self.move_textarea_cursor_word_select(id, 1)?;
                        return Ok(());
                    }
                }
                Ok(())
            }
            InputEvent::Up => {
                if !self.adjust_focused_selection(-1)? {
                    self.focus_prev()?;
                }
                Ok(())
            }
            InputEvent::Down => {
                if !self.adjust_focused_selection(1)? {
                    self.focus_next()?;
                }
                Ok(())
            }
            InputEvent::Left => {
                if !self.adjust_focused_scalar(-1.0)? {
                    self.focus_prev()?;
                }
                Ok(())
            }
            InputEvent::Right => {
                if !self.adjust_focused_scalar(1.0)? {
                    self.focus_next()?;
                }
                Ok(())
            }
            InputEvent::Encoder { delta } if delta > 0 => {
                if !self.adjust_focused_selection(1)? {
                    self.focus_next()?;
                }
                Ok(())
            }
            InputEvent::Encoder { delta } if delta < 0 => {
                if !self.adjust_focused_selection(-1)? {
                    self.focus_prev()?;
                }
                Ok(())
            }
            InputEvent::Select => {
                if let Some(id) = self.focus {
                    match self.key_bindings_for(id).select {
                        KeyBindingAction::Default | KeyBindingAction::Activate => {
                            self.handle_select_activation(id)?
                        }
                        KeyBindingAction::Back => self.handle_back_action()?,
                        KeyBindingAction::Ignore => {}
                    }
                }
                Ok(())
            }
            InputEvent::SelectPressed => {
                if let Some(id) = self.focus {
                    if self.key_input_policy_for(id).raw_select {
                        self.dispatch_key_pressed(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::SelectReleased => {
                if let Some(id) = self.focus {
                    if self.key_input_policy_for(id).raw_select {
                        self.dispatch_key_released(id)?;
                        self.handle_select_activation(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::Back => {
                if let Some(id) = self.focus {
                    match self.key_bindings_for(id).back {
                        KeyBindingAction::Default | KeyBindingAction::Back => {
                            self.handle_back_action()
                        }
                        KeyBindingAction::Activate => self.handle_select_activation(id),
                        KeyBindingAction::Ignore => Ok(()),
                    }
                } else {
                    self.handle_back_action()
                }
            }
            InputEvent::BackPressed => {
                if let Some(id) = self.focus {
                    if self.key_input_policy_for(id).raw_back {
                        self.dispatch_key_pressed(id)?;
                    }
                }
                Ok(())
            }
            InputEvent::BackReleased => {
                if let Some(id) = self.focus {
                    if self.key_input_policy_for(id).raw_back {
                        self.dispatch_key_released(id)?;
                        return self.handle_back_action();
                    }
                }
                Ok(())
            }
            InputEvent::Pointer {
                x,
                y,
                state: PointerState::Pressed,
                ..
            } => self.handle_pointer_pressed(x, y),
            InputEvent::Pointer {
                x,
                y,
                state: PointerState::Released,
                ..
            } => self.handle_pointer_released(x, y),
            InputEvent::Pointer {
                x,
                y,
                state: PointerState::Moved,
                ..
            } => self.handle_pointer_moved(x, y),
            _ => Ok(()),
        }
    }

    pub fn tick_input(&mut self, dt_ms: u32) -> Result<(), GuiError> {
        if self.last_select_id.is_some() {
            self.select_elapsed_ms = self.select_elapsed_ms.saturating_add(dt_ms);
            if self.select_elapsed_ms > self.select_double_window_ms {
                self.last_select_id = None;
                self.select_elapsed_ms = 0;
            }
        }
        if self.last_pointer_id.is_some() {
            self.pointer_elapsed_ms = self.pointer_elapsed_ms.saturating_add(dt_ms);
            if self.pointer_elapsed_ms > self.pointer_double_window_ms {
                self.last_pointer_id = None;
                self.pointer_elapsed_ms = 0;
            }
        }
        self.tick_state_transitions(dt_ms)?;
        self.tick_theme_transition(dt_ms)?;
        self.haptic_sequencer.tick(dt_ms);
        self.tick_rle_animations(dt_ms)?;
        if let Some(mut inertia) = self.inertia_scroll {
            if inertia.velocity.abs() < self.scroll_physics.velocity_threshold {
                self.inertia_scroll = None;
            } else {
                let current = self.scroll_offset(inertia.id).unwrap_or(0);
                let delta = (inertia.velocity * (dt_ms as f32 / 16.0)).round() as i32;
                if delta != 0 {
                    let next = current.saturating_sub(delta);
                    if next != current {
                        self.set_scroll_offset(inertia.id, next)?;
                        self.push_event(UiEvent::Scroll {
                            id: inertia.id,
                            delta: next - current,
                        })?;
                    }
                }
                inertia.velocity *= self
                    .scroll_physics
                    .velocity_decay
                    .powf((dt_ms as f32 / 16.0).max(1.0));
                self.inertia_scroll = Some(inertia);
            }
        }
        self.tick_textarea_cursor_blink(dt_ms)?;
        let Some(mut pressed) = self.pressed else {
            return Ok(());
        };
        if !self.effective_visible(pressed.id) || !self.effective_enabled(pressed.id) {
            self.pressed = None;
            return Ok(());
        }
        let timing = self.press_timing_for(pressed.id);
        pressed.elapsed_ms = pressed.elapsed_ms.saturating_add(dt_ms);
        pressed.repeat_elapsed_ms = pressed.repeat_elapsed_ms.saturating_add(dt_ms);
        if !pressed.long_emitted && pressed.elapsed_ms >= timing.long_press_ms {
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(
                pressed.id,
                WidgetEventKind::LongPressed,
                &mut events,
                |_| EventPolicy::Continue,
            )?;
            self.push_event(UiEvent::LongPressed(pressed.id))?;
            self.play_haptic(HapticPattern::LongPress);
            pressed.long_emitted = true;
        }
        if pressed.repeat_elapsed_ms >= timing.repeat_delay_ms
            && self.repeatable_widget(pressed.id)
            && pressed.long_emitted
        {
            let intervals =
                (pressed.repeat_elapsed_ms - timing.repeat_delay_ms) / timing.repeat_interval_ms;
            if intervals > 0 {
                self.dispatch_repeat_activation(pressed.id)?;
                pressed.repeat_elapsed_ms = timing.repeat_delay_ms;
            }
        }
        self.pressed = Some(pressed);
        Ok(())
    }

    pub fn pop_event(&mut self) -> Option<UiEvent> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0))
        }
    }

    pub fn set_event_filter(
        &mut self,
        id: WidgetId,
        filter: UiEventFilter,
    ) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some((_, current)) = self
            .subscriptions
            .iter_mut()
            .find(|(sub_id, _)| *sub_id == id)
        {
            *current = filter;
            return Ok(());
        }
        self.subscriptions
            .push((id, filter))
            .map_err(|_| GuiError::WidgetsFull)
    }

    pub fn event_filter(&self, id: WidgetId) -> Result<UiEventFilter, GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        Ok(self
            .subscriptions
            .iter()
            .find(|(sub_id, _)| *sub_id == id)
            .map(|(_, filter)| *filter)
            .unwrap_or(UiEventFilter::ALL))
    }

    pub fn clear_event_filter(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some(pos) = self
            .subscriptions
            .iter()
            .position(|(sub_id, _)| *sub_id == id)
        {
            self.subscriptions.remove(pos);
        }
        Ok(())
    }

    pub fn set_dispatch_policy(
        &mut self,
        id: WidgetId,
        policy: WidgetDispatchPolicy,
    ) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some((_, current)) = self
            .dispatch_policies
            .iter_mut()
            .find(|(policy_id, _)| *policy_id == id)
        {
            *current = policy;
            return Ok(());
        }
        self.dispatch_policies
            .push((id, policy))
            .map_err(|_| GuiError::WidgetsFull)
    }

    pub fn dispatch_policy(&self, id: WidgetId) -> Result<Option<WidgetDispatchPolicy>, GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        Ok(self
            .dispatch_policies
            .iter()
            .find(|(policy_id, _)| *policy_id == id)
            .map(|(_, policy)| *policy))
    }

    pub fn clear_dispatch_policy(&mut self, id: WidgetId) -> Result<(), GuiError> {
        self.node(id).ok_or(GuiError::NotFound)?;
        if let Some(pos) = self
            .dispatch_policies
            .iter()
            .position(|(policy_id, _)| *policy_id == id)
        {
            self.dispatch_policies.remove(pos);
        }
        Ok(())
    }

    pub(crate) fn add_widget<S>(
        &mut self,
        rect: Rect,
        kind: WidgetKind<'a>,
        style: S,
    ) -> Result<WidgetId, GuiError>
    where
        S: Into<WidgetStyle>,
    {
        let id = WidgetId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1).max(1);
        let node = WidgetNode::new(id, rect, kind, style);
        self.widgets.push(node).map_err(|_| GuiError::WidgetsFull)?;
        self.dirty.add(rect)?;
        Ok(id)
    }

    pub(crate) fn node(&self, id: WidgetId) -> Option<&WidgetNode<'a>> {
        self.widgets.iter().find(|node| node.id == id)
    }

    pub(crate) fn intrinsic_size(&self, id: WidgetId) -> Option<(u32, u32)> {
        let node = self.node(id)?;
        let style = node.style.resolve(VisualState::Normal);
        let pad_x = style.padding.left.max(0) as u32 + style.padding.right.max(0) as u32;
        let pad_y = style.padding.top.max(0) as u32 + style.padding.bottom.max(0) as u32;
        let border = style.border.width as u32 * 2;
        let text_width = |text: &str| text.chars().count() as u32 * style.font.advance();
        let text_height = style.font.line_height();

        let content = match node.kind {
            WidgetKind::Label(text) => (text_width(text), text_height),
            WidgetKind::Button(text) => (text_width(text).saturating_add(6), text_height),
            WidgetKind::Toggle { label, .. } => (text_width(label).saturating_add(12), text_height),
            WidgetKind::Checkbox { label, .. } => {
                (text_width(label).saturating_add(10), text_height)
            }
            WidgetKind::ValueLabel { label, .. } => {
                (text_width(label).saturating_add(16), text_height)
            }
            WidgetKind::IconButton { label, .. } => {
                (text_width(label).saturating_add(10), text_height)
            }
            WidgetKind::Tabs { labels, .. } => {
                let max = labels.iter().map(|s| text_width(s)).max().unwrap_or(0);
                (
                    max.saturating_mul(labels.len() as u32).saturating_add(4),
                    text_height,
                )
            }
            WidgetKind::Dialog { title, body } => {
                let w = text_width(title).max(text_width(body)).saturating_add(8);
                (w, text_height.saturating_mul(3))
            }
            WidgetKind::Toast { text, .. } => (
                text_width(text).saturating_add(8),
                text_height.saturating_add(2),
            ),
            WidgetKind::Dropdown {
                items, selected, ..
            } => (
                text_width(items.get(selected).copied().unwrap_or("-")).saturating_add(10),
                text_height.saturating_add(2),
            ),
            WidgetKind::TextArea {
                text_buf,
                text_len,
                placeholder,
                ..
            } => (
                text_width(if text_len == 0 {
                    placeholder
                } else {
                    textarea_text(&text_buf, text_len)
                })
                .saturating_add(10),
                text_height.saturating_add(4),
            ),
            WidgetKind::Keyboard { keys, cols, .. } => {
                let cols = cols.max(1) as u32;
                let rows = (keys.len() as u32).div_ceil(cols).max(1);
                (
                    cols.saturating_mul(style.font.advance().saturating_add(4)),
                    rows.saturating_mul(style.font.line_height().saturating_add(4)),
                )
            }
            WidgetKind::List {
                items,
                visible_rows,
                ..
            }
            | WidgetKind::CircularList {
                items,
                visible_rows,
                ..
            } => {
                let max = items.iter().map(|s| text_width(s)).max().unwrap_or(0);
                (
                    max.saturating_add(6),
                    (text_height.saturating_add(2))
                        .saturating_mul(visible_rows as u32)
                        .max(text_height),
                )
            }
            WidgetKind::Menu { items, .. } => {
                let max = items.iter().map(|s| text_width(s)).max().unwrap_or(0);
                (
                    max.saturating_add(6),
                    (text_height.saturating_add(2))
                        .saturating_mul(items.len() as u32)
                        .max(text_height),
                )
            }
            WidgetKind::FeedTimeline {
                items,
                visible_rows,
                expanded,
                ..
            } => {
                let max = items.iter().map(|s| text_width(s)).max().unwrap_or(0);
                let row_h = if expanded {
                    text_height.saturating_mul(2).saturating_add(2)
                } else {
                    text_height.saturating_add(2)
                };
                (
                    max.saturating_add(8),
                    row_h.saturating_mul(visible_rows as u32).max(text_height),
                )
            }
            _ => (node.rect.w.max(1), node.rect.h.max(1)),
        };

        Some((
            content
                .0
                .saturating_add(pad_x)
                .saturating_add(border)
                .max(1),
            content
                .1
                .saturating_add(pad_y)
                .saturating_add(border)
                .max(1),
        ))
    }

    pub(crate) fn node_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode<'a>> {
        self.widgets.iter_mut().find(|node| node.id == id)
    }

    pub(crate) fn effective_visible(&self, id: WidgetId) -> bool {
        let mut current = Some(id);
        let mut depth = 0;
        while let Some(widget_id) = current {
            if depth >= NODES {
                return false;
            }
            let Some(node) = self.node(widget_id) else {
                return false;
            };
            if node.hidden() {
                return false;
            }
            current = node.parent;
            depth += 1;
        }
        true
    }

    pub(crate) fn inherited_clip(&self, id: WidgetId) -> Option<Rect> {
        let mut clip = self.viewport;
        let mut chain = heapless::Vec::<WidgetId, NODES>::new();
        let mut current = Some(id);
        while let Some(widget_id) = current {
            chain.push(widget_id).ok()?;
            current = self.node(widget_id)?.parent;
        }
        for widget_id in chain.iter().rev().copied() {
            let node = self.node(widget_id)?;
            if widget_id == id || node.clips_children() {
                clip = clip.intersection(self.absolute_rect(widget_id)?);
            }
            if clip.is_empty() {
                return None;
            }
        }
        Some(clip)
    }

    pub(crate) fn effective_enabled(&self, id: WidgetId) -> bool {
        let mut current = Some(id);
        let mut depth = 0;
        while let Some(widget_id) = current {
            if depth >= NODES {
                return false;
            }
            let Some(node) = self.node(widget_id) else {
                return false;
            };
            if node.disabled() {
                return false;
            }
            current = node.parent;
            depth += 1;
        }
        true
    }

    pub(crate) fn effective_focusable(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| {
            self.node_in_active_group(node)
                && node.focusable()
                && self.effective_visible(id)
                && self.effective_enabled(id)
        })
    }

    pub(crate) fn ensure_focus(&mut self) {
        if self.focus.is_none() {
            self.focus = self
                .widgets
                .iter()
                .find(|node| self.effective_focusable(node.id))
                .map(|n| n.id);
        }
    }

    pub(crate) fn focus_next(&mut self) -> Result<(), GuiError> {
        self.move_focus(1)
    }

    pub(crate) fn focus_prev(&mut self) -> Result<(), GuiError> {
        self.move_focus(-1)
    }

    pub(crate) fn move_focus(&mut self, delta: i8) -> Result<(), GuiError> {
        let focusable = self
            .widgets
            .iter()
            .filter(|node| self.effective_focusable(node.id))
            .count();
        if focusable == 0 {
            return Ok(());
        }

        let current_pos = self
            .widgets
            .iter()
            .filter(|node| self.effective_focusable(node.id))
            .position(|node| Some(node.id) == self.focus)
            .unwrap_or(0);

        let next_pos = if delta >= 0 {
            (current_pos + 1) % focusable
        } else if current_pos == 0 {
            focusable - 1
        } else {
            current_pos - 1
        };

        let next = self
            .widgets
            .iter()
            .filter(|node| self.effective_focusable(node.id))
            .nth(next_pos)
            .map(|node| node.id);
        self.set_focus(next)
    }

    pub(crate) fn adjust_focused_selection(&mut self, delta: i8) -> Result<bool, GuiError> {
        let Some(id) = self.focus else {
            return Ok(false);
        };
        let wrap_navigation = self.menu_contract.wrap_navigation;

        let mut changed_rect = None;
        let mut changed = false;

        if let Some(node) = self.node_mut(id) {
            match node.kind {
                WidgetKind::Menu {
                    items,
                    selected: ref mut current,
                } => {
                    if items.is_empty() {
                        return Ok(true);
                    }
                    changed = bump_index_with_wrap(current, items.len(), delta, wrap_navigation);
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Dropdown {
                    items,
                    selected: ref mut current,
                    open,
                } => {
                    if !open {
                        return Ok(false);
                    }
                    if items.is_empty() {
                        return Ok(true);
                    }
                    changed = bump_index_with_wrap(current, items.len(), delta, wrap_navigation);
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Roller {
                    items,
                    selected: ref mut current,
                } => {
                    if items.is_empty() {
                        return Ok(true);
                    }
                    changed = bump_index_with_wrap(current, items.len(), delta, wrap_navigation);
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Keyboard {
                    keys,
                    selected: ref mut current,
                    ..
                } => {
                    if keys.is_empty() {
                        return Ok(true);
                    }
                    changed = bump_index_with_wrap(current, keys.len(), delta, wrap_navigation);
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::List {
                    items,
                    selected: ref mut current,
                    ref mut offset,
                    visible_rows,
                }
                | WidgetKind::CircularList {
                    items,
                    selected: ref mut current,
                    ref mut offset,
                    visible_rows,
                } => {
                    if items.is_empty() {
                        return Ok(true);
                    }
                    let mut state = ListState::new(*current, *offset, visible_rows);
                    let mut next = state.selected;
                    changed = bump_index_with_wrap(&mut next, items.len(), delta, wrap_navigation);
                    if changed {
                        let _ = state.set_selected(next, items.len());
                    }
                    *current = state.selected;
                    *offset = state.offset;
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::FeedTimeline {
                    items,
                    selected: ref mut current,
                    ref mut offset,
                    visible_rows,
                    expanded,
                } => {
                    if items.is_empty() {
                        return Ok(true);
                    }
                    let mut state =
                        FeedTimelineState::new(*current, *offset, visible_rows, expanded);
                    let mut next = state.selected;
                    changed = bump_index_with_wrap(&mut next, items.len(), delta, wrap_navigation);
                    if changed {
                        let _ = state.set_selected(next, items.len());
                    }
                    *current = state.selected;
                    *offset = state.offset;
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::ScrollView {
                    offset_y: ref mut offset,
                    content_h,
                } => {
                    let mut state = ScrollState::new(*offset, content_h);
                    changed = state.scroll_by(delta as i32 * 8);
                    *offset = state.offset_y;
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::AutoComplete {
                    filtered: _,
                    filter_count,
                    selected: ref mut current,
                    expanded,
                    ..
                } => {
                    if !expanded || filter_count == 0 {
                        return Ok(false);
                    }
                    let idx = current.unwrap_or(0);
                    let mut next = idx;
                    changed = bump_index_with_wrap(&mut next, filter_count as usize, delta, wrap_navigation);
                    *current = Some(next);
                    changed_rect = changed.then_some(node.rect);
                }
                _ => return Ok(false),
            }
        }

        if let Some(rect) = changed_rect {
            self.dirty.add(rect)?;
        }
        if changed {
            self.push_event(UiEvent::ValueChanged(id))?;
        }
        Ok(true)
    }

    pub(crate) fn adjust_focused_scalar(&mut self, direction: f32) -> Result<bool, GuiError> {
        let Some(id) = self.focus else {
            return Ok(false);
        };

        let mut changed_rect = None;
        let mut changed = false;

        if let Some(node) = self.node_mut(id) {
            match node.kind {
                WidgetKind::Slider {
                    value: ref mut current,
                    min,
                    max,
                } => {
                    let mut state = SliderState::new(*current, min, max);
                    changed = state.step_by(direction);
                    *current = state.value;
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Tabs {
                    labels,
                    selected: ref mut current,
                } => {
                    if labels.is_empty() {
                        return Ok(true);
                    }
                    let mut state = TabsState::new(*current);
                    changed = state.bump(labels.len(), if direction >= 0.0 { 1 } else { -1 });
                    *current = state.selected;
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::TextArea {
                    text_buf,
                    text_len,
                    cursor: ref mut current,
                    ..
                } => {
                    let text = textarea_text(&text_buf, text_len);
                    let len = text.chars().count();
                    if direction >= 0.0 {
                        let next = (*current + 1).min(len);
                        changed = next != *current;
                        *current = next;
                    } else {
                        let next = current.saturating_sub(1);
                        changed = next != *current;
                        *current = next;
                    }
                    changed_rect = changed.then_some(node.rect);
                }
                WidgetKind::Dial {
                    value: ref mut current,
                    min,
                    max,
                } => {
                    let mut state = SliderState::new(*current, min, max);
                    changed = state.step_by(direction);
                    *current = state.value;
                    changed_rect = changed.then_some(node.rect);
                }
                _ => return Ok(false),
            }
        }

        if let Some(rect) = changed_rect {
            self.dirty.add(rect)?;
        }
        if changed {
            self.push_event(UiEvent::ValueChanged(id))?;
        }
        Ok(true)
    }

    pub(crate) fn activate_focused(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut changed_rect = None;
        let mut changed = false;
        let mut dropdown_state_event = None;
        let select_opens_dropdown = self.menu_contract.select_opens_dropdown;

        if let Some(node) = self.node_mut(id) {
            match node.kind {
                WidgetKind::Toggle { on: ref mut v, .. } => {
                    *v = !*v;
                    changed = true;
                    changed_rect = Some(node.rect);
                }
                WidgetKind::Checkbox {
                    checked: ref mut v, ..
                } => {
                    *v = !*v;
                    changed = true;
                    changed_rect = Some(node.rect);
                }
                WidgetKind::Keyboard {
                    keys,
                    alt_keys,
                    selected,
                    layout,
                    target,
                    ..
                } => {
                    if let Some(ch) = keyboard_char_for_layout(keys, alt_keys, selected, layout) {
                        changed = true;
                        changed_rect = Some(node.rect);
                        if let Some(target) = target {
                            let _ = self.push_event(UiEvent::TextInput { id: target, ch });
                            let _ = self.push_event(UiEvent::ValueChanged(target));
                        }
                    }
                }
                WidgetKind::Dropdown {
                    open: ref mut is_open,
                    ..
                } if select_opens_dropdown => {
                    *is_open = !*is_open;
                    changed = true;
                    changed_rect = Some(node.rect);
                    dropdown_state_event = Some(*is_open);
                }
                _ => {}
            }
        }

        if let Some(open) = dropdown_state_event {
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(
                id,
                if open {
                    WidgetEventKind::Opened
                } else {
                    WidgetEventKind::Closed
                },
                &mut events,
                |_| EventPolicy::Continue,
            )?;
            self.push_event(if open {
                UiEvent::Opened(id)
            } else {
                UiEvent::Closed(id)
            })?;
        }

        if let Some(rect) = changed_rect {
            self.dirty.add(rect)?;
        }
        if changed {
            self.push_event(UiEvent::ValueChanged(id))?;
        }
        Ok(())
    }

    pub(crate) fn node_in_active_group(&self, node: &WidgetNode<'_>) -> bool {
        self.active_focus_group
            .is_none_or(|group| node.focus_group == group)
    }

    pub(crate) fn handle_pointer_pressed(&mut self, x: i32, y: i32) -> Result<(), GuiError> {
        let hit = self.pointer_hit(x, y, true);

        if let Some(id) = hit {
            self.dispatch_activation(id, true)?;
            self.update_dial_value_at_pointer(id, x, y)?;
            self.pressed = Some(PressTracker {
                id,
                start_x: x,
                start_y: y,
                last_x: x,
                last_y: y,
                elapsed_ms: 0,
                long_emitted: false,
                gesture_emitted: false,
                repeat_elapsed_ms: 0,
                scroll_velocity: 0.0,
            });
            self.inertia_scroll = None;
        }
        Ok(())
    }

    pub(crate) fn handle_pointer_released(&mut self, _x: i32, _y: i32) -> Result<(), GuiError> {
        let mut released_id = None;
        if let Some(pressed) = self.pressed {
            if let Some(scroll_id) = self.scrollable_ancestor(pressed.id) {
                if pressed.scroll_velocity.abs() > self.scroll_physics.velocity_threshold {
                    self.inertia_scroll = Some(InertiaScroll {
                        id: scroll_id,
                        velocity: pressed.scroll_velocity,
                    });
                }
            }
            released_id = Some(pressed.id);
        }
        self.pressed = None;
        if let Some(id) = released_id {
            let to = if !self.effective_enabled(id) {
                VisualState::Disabled
            } else if Some(id) == self.focus {
                VisualState::Focused
            } else {
                VisualState::Normal
            };
            self.start_state_transition(id, VisualState::Pressed, to);
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(id, WidgetEventKind::Released, &mut events, |_| {
                EventPolicy::Continue
            })?;
            self.push_event(UiEvent::Released(id))?;
            self.push_event(UiEvent::PointerReleased(id))?;
            let double_pointer = self.last_pointer_id == Some(id)
                && self.pointer_elapsed_ms <= self.pointer_double_window_ms;
            if double_pointer {
                self.dispatch_double_clicked(id)?;
                self.last_pointer_id = None;
                self.pointer_elapsed_ms = 0;
            } else {
                self.last_pointer_id = Some(id);
                self.pointer_elapsed_ms = 0;
            }
        }
        Ok(())
    }

    pub(crate) fn handle_pointer_moved(&mut self, x: i32, y: i32) -> Result<(), GuiError> {
        let Some(mut pressed) = self.pressed else {
            return Ok(());
        };
        self.update_dial_value_at_pointer(pressed.id, x, y)?;
        let dy = y - pressed.last_y;
        pressed.last_x = x;
        pressed.last_y = y;

        let moved_from_start =
            (x - pressed.start_x).unsigned_abs() + (y - pressed.start_y).unsigned_abs();
        if !pressed.gesture_emitted && moved_from_start >= 6 {
            let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
            self.dispatch_widget_event(pressed.id, WidgetEventKind::Gesture, &mut events, |_| {
                EventPolicy::Continue
            })?;
            self.push_event(UiEvent::Gesture(pressed.id))?;
            pressed.gesture_emitted = true;
        }

        if let Some(scroll_id) = self.scrollable_ancestor(pressed.id) {
            let current = self.scroll_offset(scroll_id).unwrap_or(0);
            let next = current.saturating_sub(dy);
            if next != current {
                self.set_scroll_offset(scroll_id, next)?;
                self.push_event(UiEvent::Scroll {
                    id: scroll_id,
                    delta: next - current,
                })?;
                let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
                self.dispatch_widget_event(
                    scroll_id,
                    WidgetEventKind::Scroll {
                        delta: next - current,
                    },
                    &mut events,
                    |_| EventPolicy::Continue,
                )?;
            }
            let blend = self.scroll_physics.drag_velocity_blend;
            pressed.scroll_velocity = pressed.scroll_velocity * (1.0 - blend) + (dy as f32) * blend;
        }
        self.pressed = Some(pressed);
        Ok(())
    }

    pub(crate) fn dispatch_activation(
        &mut self,
        id: WidgetId,
        is_pointer: bool,
    ) -> Result<(), GuiError> {
        let is_autocomplete = matches!(
            self.node(id).map(|n| &n.kind),
            Some(WidgetKind::AutoComplete { .. })
        );
        if is_autocomplete {
            self.autocomplete_confirm_selection(id)?;
            return Ok(());
        }

        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::Pressed, &mut events, |_| {
            EventPolicy::Continue
        })?;
        if self.effective_focusable(id) {
            self.set_focus(Some(id))?;
        }
        self.push_event(UiEvent::Pressed(id))?;
        if is_pointer {
            self.push_event(UiEvent::PointerPressed(id))?;
        }
        let from = if Some(id) == self.focus {
            VisualState::Focused
        } else {
            VisualState::Normal
        };
        self.start_state_transition(id, from, VisualState::Pressed);

        self.activate_focused(id)?;
        self.dispatch_widget_event(id, WidgetEventKind::Clicked, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Clicked(id))?;
        self.play_haptic(HapticPattern::Click);
        self.push_event(UiEvent::Activate(id))?;
        Ok(())
    }

    pub(crate) fn dispatch_repeat_activation(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::Clicked, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Clicked(id))?;
        self.play_haptic(HapticPattern::Click);
        self.push_event(UiEvent::Activate(id))
    }

    pub(crate) fn dispatch_double_clicked(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::DoubleClicked, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::DoubleClicked(id))?;
        self.play_haptic(HapticPattern::DoubleClick);
        Ok(())
    }

    pub(crate) fn dispatch_key_pressed(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::Pressed, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Pressed(id))
    }

    pub(crate) fn dispatch_key_released(&mut self, id: WidgetId) -> Result<(), GuiError> {
        let mut events = heapless::Vec::<WidgetEvent, NODES>::new();
        self.dispatch_widget_event(id, WidgetEventKind::Released, &mut events, |_| {
            EventPolicy::Continue
        })?;
        self.push_event(UiEvent::Released(id))
    }

    pub(crate) fn repeatable_widget(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| {
            matches!(
                node.kind,
                WidgetKind::Button(_) | WidgetKind::IconButton { .. }
            )
        })
    }

    pub(crate) fn pointer_hit(&self, x: i32, y: i32, clickable_only: bool) -> Option<WidgetId> {
        self.widgets
            .iter()
            .rev()
            .find(|node| {
                (!clickable_only || node.clickable())
                    && self.effective_visible(node.id)
                    && self.effective_enabled(node.id)
                    && self
                        .absolute_rect(node.id)
                        .is_some_and(|rect| rect.contains(x, y))
            })
            .map(|node| node.id)
    }

    pub(crate) fn scrollable_ancestor(&self, id: WidgetId) -> Option<WidgetId> {
        let mut current = Some(id);
        let mut depth = 0usize;
        while let Some(widget_id) = current {
            if depth >= NODES {
                return None;
            }
            let node = self.node(widget_id)?;
            if node.scrollable() {
                return Some(widget_id);
            }
            current = node.parent;
            depth += 1;
        }
        None
    }

    pub(crate) fn mark_focus_pair(
        &mut self,
        old: Option<WidgetId>,
        new: Option<WidgetId>,
    ) -> Result<(), GuiError> {
        if let Some(id) = old {
            if let Some(rect) = self.absolute_rect(id) {
                self.dirty.add(rect)?;
            }
        }
        if let Some(id) = new {
            if let Some(rect) = self.absolute_rect(id) {
                self.dirty.add(rect)?;
            }
        }
        Ok(())
    }

    pub(crate) fn start_focus_transitions(&mut self, old: Option<WidgetId>, new: Option<WidgetId>) {
        if self.state_transition_ms == 0 {
            return;
        }
        if let Some(id) = old {
            self.start_state_transition(id, VisualState::Focused, VisualState::Normal);
        }
        if let Some(id) = new {
            self.start_state_transition(id, VisualState::Normal, VisualState::Focused);
        }
    }

    pub(crate) fn start_state_transition(
        &mut self,
        id: WidgetId,
        from: VisualState,
        to: VisualState,
    ) {
        if self.state_transition_ms == 0 || from == to {
            return;
        }
        if let Some(entry) = self
            .state_transitions
            .iter_mut()
            .find(|entry| entry.id == id)
        {
            *entry = StateTransition {
                id,
                from,
                to,
                elapsed_ms: 0,
            };
            return;
        }
        if self.state_transitions.len() == self.state_transitions.capacity() {
            self.state_transitions.remove(0);
        }
        let _ = self.state_transitions.push(StateTransition {
            id,
            from,
            to,
            elapsed_ms: 0,
        });
    }

    pub(crate) fn tick_state_transitions(&mut self, dt_ms: u32) -> Result<(), GuiError> {
        if self.state_transitions.is_empty() || self.state_transition_ms == 0 {
            return Ok(());
        }
        let mut i = 0usize;
        let mut completed_pressed = heapless::Vec::<WidgetId, NODES>::new();
        while i < self.state_transitions.len() {
            let mut remove = false;
            let id;
            let to;
            {
                let entry = &mut self.state_transitions[i];
                entry.elapsed_ms = entry.elapsed_ms.saturating_add(dt_ms);
                if entry.elapsed_ms >= self.state_transition_ms {
                    remove = true;
                }
                id = entry.id;
                to = entry.to;
            }
            if let Some(rect) = self.absolute_rect(id) {
                self.dirty.add(rect)?;
            }
            if remove {
                if to == VisualState::Pressed {
                    let _ = completed_pressed.push(id);
                }
                self.state_transitions.remove(i);
            } else {
                i += 1;
            }
        }
        for id in completed_pressed {
            // Pointer-held presses keep visual pressed state until release.
            if self.pressed.is_some_and(|pressed| pressed.id == id) {
                continue;
            }
            let to = self.resting_visual_state(id);
            self.start_state_transition(id, VisualState::Pressed, to);
        }
        Ok(())
    }

    pub(crate) fn state_transition_progress(
        &self,
        id: WidgetId,
    ) -> Option<(VisualState, VisualState, f32)> {
        let duration = self.state_transition_ms.max(1);
        self.state_transitions
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| {
                let t = (entry.elapsed_ms as f32 / duration as f32).clamp(0.0, 1.0);
                (entry.from, entry.to, t)
            })
    }

    pub(crate) fn set_textarea_cursor_visible(&mut self, id: Option<WidgetId>, visible: bool) {
        let Some(id) = id else {
            return;
        };
        let Some(rect) = self.absolute_rect(id) else {
            return;
        };
        let Some(node) = self.node_mut(id) else {
            return;
        };
        if let WidgetKind::TextArea {
            cursor_visible: ref mut current,
            ..
        } = node.kind
        {
            *current = visible;
            let _ = self.dirty.add(rect);
        }
    }

    pub(crate) fn tick_theme_transition(&mut self, dt_ms: u32) -> Result<(), GuiError> {
        if let (Some(from), Some(to)) = (self.theme_transition_from, self.theme_transition_to) {
            let elapsed = self.theme_transition_elapsed_ms.saturating_add(dt_ms);
            self.theme_transition_elapsed_ms = elapsed;
            if elapsed >= self.theme_transition_duration_ms {
                self.theme = to;
                self.theme_transition_from = None;
                self.theme_transition_to = None;
            } else {
                let t = elapsed as f32 / self.theme_transition_duration_ms as f32;
                self.theme = crate::style::lerp_theme(from, to, t);
            }
            self.dirty.mark_all(self.viewport)?;
        }
        Ok(())
    }

    pub(crate) fn tick_textarea_cursor_blink(&mut self, dt_ms: u32) -> Result<(), GuiError> {
        let Some(id) = self.focus else {
            return Ok(());
        };
        let is_textarea = matches!(
            self.node(id).map(|n| n.kind),
            Some(WidgetKind::TextArea { .. })
        );
        if !is_textarea {
            return Ok(());
        }
        self.textarea_cursor_blink_elapsed_ms =
            self.textarea_cursor_blink_elapsed_ms.saturating_add(dt_ms);
        if self.textarea_cursor_blink_elapsed_ms < self.textarea_cursor_blink_ms {
            return Ok(());
        }
        self.textarea_cursor_blink_elapsed_ms = 0;
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        if let WidgetKind::TextArea {
            cursor_visible: ref mut visible,
            ..
        } = node.kind
        {
            *visible = !*visible;
            self.dirty.add(rect)?;
        }
        Ok(())
    }

    pub(crate) fn push_event(&mut self, event: UiEvent) -> Result<(), GuiError> {
        if self.should_emit_event(event)? {
            self.events.push(event).map_err(|_| GuiError::EventsFull)?;
        }
        Ok(())
    }

    pub(crate) fn should_emit_event(&self, event: UiEvent) -> Result<bool, GuiError> {
        let Some(target) = event.target() else {
            return Ok(true);
        };
        let filter = self.event_filter(target)?;
        Ok(filter.contains(event.filter()))
    }

    pub(crate) fn stop_due_to_builtin_widget_behavior(&self, event: WidgetEvent) -> bool {
        if event.phase != EventPhase::Capture || event.current == event.target {
            return false;
        }
        let is_pointer_kind = matches!(
            event.kind,
            WidgetEventKind::Pressed | WidgetEventKind::Released | WidgetEventKind::Clicked
        );
        is_pointer_kind
            && self
                .node(event.current)
                .is_some_and(|node| matches!(node.kind, WidgetKind::ScrollView { .. }))
    }

    pub(crate) fn stop_due_to_registered_policy(&self, event: WidgetEvent) -> bool {
        self.dispatch_policies
            .iter()
            .find(|(id, _)| *id == event.current)
            .is_some_and(|(_, policy)| policy.stop && policy.allows(event.kind, event.phase))
    }

    pub(crate) fn resting_visual_state(&self, id: WidgetId) -> VisualState {
        if !self.effective_enabled(id) {
            VisualState::Disabled
        } else if Some(id) == self.focus {
            VisualState::Focused
        } else {
            VisualState::Normal
        }
    }

    pub(crate) fn current_visual_state(&self, id: WidgetId) -> VisualState {
        if self.pressed.is_some_and(|pressed| pressed.id == id) {
            VisualState::Pressed
        } else {
            self.resting_visual_state(id)
        }
    }

    pub(crate) fn press_timing_for(&self, id: WidgetId) -> PressTiming {
        self.widget_press_timings
            .iter()
            .find(|(timing_id, _)| *timing_id == id)
            .map(|(_, timing)| *timing)
            .unwrap_or(PressTiming {
                long_press_ms: self.long_press_ms,
                repeat_delay_ms: self.press_repeat_delay_ms,
                repeat_interval_ms: self.press_repeat_interval_ms,
            })
    }

    pub(crate) fn key_input_policy_for(&self, id: WidgetId) -> WidgetKeyInputPolicy {
        self.widget_key_policies
            .iter()
            .find(|(policy_id, _)| *policy_id == id)
            .map(|(_, policy)| *policy)
            .unwrap_or_default()
    }

    pub(crate) fn key_bindings_for(&self, id: WidgetId) -> WidgetKeyBindings {
        self.widget_key_bindings
            .iter()
            .find(|(binding_id, _)| *binding_id == id)
            .map(|(_, bindings)| *bindings)
            .unwrap_or_default()
    }

    pub(crate) fn handle_select_activation(&mut self, id: WidgetId) -> Result<(), GuiError> {
        if let Some(node) = self.node(id) {
            if self.menu_contract.select_toggles_feed_expanded
                && matches!(node.kind, WidgetKind::FeedTimeline { .. })
            {
                let expanded = if let WidgetKind::FeedTimeline { expanded, .. } = node.kind {
                    expanded
                } else {
                    false
                };
                self.set_feed_expanded(id, !expanded)?;
                self.push_event(UiEvent::ValueChanged(id))?;
            }
        }
        let double_select = self.last_select_id == Some(id)
            && self.select_elapsed_ms <= self.select_double_window_ms;
        self.dispatch_activation(id, false)?;
        if double_select {
            self.dispatch_double_clicked(id)?;
            self.last_select_id = None;
            self.select_elapsed_ms = 0;
        } else {
            self.last_select_id = Some(id);
            self.select_elapsed_ms = 0;
        }
        Ok(())
    }

    pub(crate) fn handle_back_action(&mut self) -> Result<(), GuiError> {
        if let Some(id) = self.focus {
            if matches!(
                self.node(id).map(|n| n.kind),
                Some(WidgetKind::TextArea { .. })
            ) {
                self.textarea_backspace(id)?;
                return Ok(());
            }
            if matches!(
                self.node(id).map(|n| n.kind),
                Some(WidgetKind::AutoComplete { .. })
            ) {
                let expanded = if let Some(WidgetKind::AutoComplete { expanded, .. }) = self.node(id).map(|n| n.kind) {
                    expanded
                } else {
                    false
                };
                if expanded {
                    let has_chars = if let Some(WidgetKind::AutoComplete { text_len, .. }) = self.node(id).map(|n| n.kind) {
                        text_len > 0
                    } else {
                        false
                    };
                    if has_chars {
                        self.delete_autocomplete_char(id)?;
                    } else {
                        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
                        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
                        if let WidgetKind::AutoComplete { expanded, .. } = &mut node.kind {
                            *expanded = false;
                        }
                        self.dirty.add(rect)?;
                    }
                    return Ok(());
                }
            }
            if matches!(
                self.node(id).map(|n| n.kind),
                Some(WidgetKind::Dropdown { open: true, .. })
            ) && self.menu_contract.back_closes_dropdown
            {
                self.set_dropdown_open(id, false)?;
                return Ok(());
            }
            if matches!(
                self.node(id).map(|n| n.kind),
                Some(WidgetKind::NotificationActionSheet { open: true, .. })
            ) && self.menu_contract.back_closes_notification_sheet
            {
                self.set_notification_sheet_open(id, false)?;
                return Ok(());
            }
        }
        self.push_event(UiEvent::Back)
    }

    pub(crate) fn update_dial_value_at_pointer(&mut self, id: WidgetId, x: i32, y: i32) -> Result<(), GuiError> {
        let rect = self.absolute_rect(id).ok_or(GuiError::NotFound)?;
        let node = self.node_mut(id).ok_or(GuiError::NotFound)?;
        if let WidgetKind::Dial { value, min, max } = &mut node.kind {
            let cx = rect.x + rect.w as i32 / 2;
            let cy = rect.y + rect.h as i32 / 2;
            let dx = (x - cx) as f32;
            let dy = (y - cy) as f32;
            if dx != 0.0 || dy != 0.0 {
                #[cfg(not(feature = "std"))]
                use crate::math::F32Ext as _;
                
                let angle_rad = dy.atan2(dx);
                let mut angle_norm = angle_rad + core::f32::consts::PI;
                if angle_norm < 0.0 {
                    angle_norm += 2.0 * core::f32::consts::PI;
                }
                let progress = (angle_norm / (2.0 * core::f32::consts::PI)).clamp(0.0, 1.0);
                let next_val = *min + progress * (*max - *min);
                if (*value - next_val).abs() > 0.001 {
                    *value = next_val;
                    self.dirty.add(rect)?;
                    self.push_event(UiEvent::ValueChanged(id))?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn tick_rle_animations(&mut self, dt_ms: u32) -> Result<(), GuiError> {
        let mut dirty_rects = heapless::Vec::<Rect, NODES>::new();
        for node in self.widgets.iter_mut() {
            if let WidgetKind::RlePlayer {
                total_frames,
                ref mut current_frame,
                ref mut elapsed_ms,
                frame_duration_ms,
                ..
            } = node.kind {
                if total_frames > 1 && frame_duration_ms > 0 {
                    *elapsed_ms = elapsed_ms.saturating_add(dt_ms);
                    if *elapsed_ms >= frame_duration_ms {
                        let frames_to_advance = *elapsed_ms / frame_duration_ms;
                        *elapsed_ms %= frame_duration_ms;
                        *current_frame = (*current_frame + frames_to_advance as usize) % total_frames;
                        let _ = dirty_rects.push(node.rect);
                    }
                }
            }
        }
        for rect in dirty_rects {
            self.dirty.add(rect)?;
        }
        Ok(())
    }
}

pub(crate) fn bump_index_with_wrap(current: &mut usize, len: usize, delta: i8, wrap: bool) -> bool {
    if len == 0 {
        return false;
    }
    let next = if delta >= 0 {
        if *current + 1 >= len {
            if wrap { 0 } else { *current }
        } else {
            *current + 1
        }
    } else if *current == 0 {
        if wrap { len - 1 } else { *current }
    } else {
        *current - 1
    };
    if next != *current {
        *current = next;
        true
    } else {
        false
    }
}

pub(crate) fn keyboard_char_for_layout(
    keys: &[char],
    alt_keys: Option<&[char]>,
    selected: usize,
    layout: KeyboardLayout,
) -> Option<char> {
    let base = keys.get(selected).copied()?;
    Some(match layout {
        KeyboardLayout::Normal => base,
        KeyboardLayout::Shift => {
            if base.is_ascii_alphabetic() {
                base.to_ascii_uppercase()
            } else {
                base
            }
        }
        KeyboardLayout::Symbols => alt_keys
            .and_then(|keys| keys.get(selected).copied())
            .unwrap_or('#'),
    })
}

pub(crate) fn textarea_text(buf: &[u8; TEXTAREA_CAPACITY], len: u8) -> &str {
    let used = (len as usize).min(TEXTAREA_CAPACITY);
    core::str::from_utf8(&buf[..used]).unwrap_or("")
}

pub(crate) fn textarea_storage_from_str(text: &str) -> ([u8; TEXTAREA_CAPACITY], u8) {
    let mut out = [0u8; TEXTAREA_CAPACITY];
    let mut len = 0usize;
    for ch in text.chars() {
        let mut tmp = [0u8; 4];
        let enc = ch.encode_utf8(&mut tmp).as_bytes();
        if len + enc.len() > TEXTAREA_CAPACITY {
            break;
        }
        out[len..len + enc.len()].copy_from_slice(enc);
        len += enc.len();
    }
    (out, len as u8)
}

pub(crate) fn textarea_storage_from_chars(
    chars: &heapless::Vec<char, TEXTAREA_CAPACITY>,
) -> ([u8; TEXTAREA_CAPACITY], u8) {
    let mut out = [0u8; TEXTAREA_CAPACITY];
    let mut len = 0usize;
    for ch in chars {
        let mut tmp = [0u8; 4];
        let enc = ch.encode_utf8(&mut tmp).as_bytes();
        if len + enc.len() > TEXTAREA_CAPACITY {
            break;
        }
        out[len..len + enc.len()].copy_from_slice(enc);
        len += enc.len();
    }
    (out, len as u8)
}

pub(crate) fn char_at(text: &str, idx: usize) -> Option<char> {
    text.chars().nth(idx)
}

pub(crate) fn prev_word_boundary(text: &str, cursor: usize) -> usize {
    let mut pos = cursor.min(text.chars().count());
    while pos > 0 && char_at(text, pos - 1).is_some_and(|ch| ch.is_whitespace()) {
        pos -= 1;
    }
    while pos > 0 && char_at(text, pos - 1).is_some_and(|ch| !ch.is_whitespace()) {
        pos -= 1;
    }
    pos
}

pub(crate) fn next_word_boundary(text: &str, cursor: usize) -> usize {
    let len = text.chars().count();
    let mut pos = cursor.min(len);
    while pos < len && char_at(text, pos).is_some_and(|ch| !ch.is_whitespace()) {
        pos += 1;
    }
    while pos < len && char_at(text, pos).is_some_and(|ch| ch.is_whitespace()) {
        pos += 1;
    }
    pos
}

pub(crate) fn delete_selection_if_any(
    chars: &mut heapless::Vec<char, TEXTAREA_CAPACITY>,
    cursor: &mut usize,
    selection: &mut Option<(usize, usize)>,
) -> bool {
    let Some((start, end)) = *selection else {
        return false;
    };
    let start = start.min(end).min(chars.len());
    let end = end.max(start).min(chars.len());
    if end <= start {
        *selection = None;
        *cursor = start;
        return false;
    }
    for _ in start..end {
        chars.remove(start);
    }
    *cursor = start;
    *selection = None;
    true
}

pub(crate) fn textarea_row_col_at_cursor(
    text: &str,
    cursor: usize,
    wrap_cols: usize,
) -> (usize, usize) {
    let mut row = 0usize;
    let mut col = 0usize;
    for ch in text.chars().take(cursor) {
        if ch == '\n' {
            row += 1;
            col = 0;
            continue;
        }
        col += 1;
        if col >= wrap_cols {
            row += 1;
            col = 0;
        }
    }
    (row, col)
}

pub(crate) fn textarea_cursor_from_row_col(
    text: &str,
    target_row: usize,
    target_col: usize,
    wrap_cols: usize,
) -> usize {
    let mut row = 0usize;
    let mut col = 0usize;
    let mut idx = 0usize;
    for ch in text.chars() {
        if row == target_row && col >= target_col {
            break;
        }
        if ch == '\n' {
            if row == target_row {
                break;
            }
            row += 1;
            col = 0;
            idx += 1;
            continue;
        }
        idx += 1;
        col += 1;
        if col >= wrap_cols {
            if row == target_row {
                break;
            }
            row += 1;
            col = 0;
        }
    }
    idx
}

pub(crate) fn textarea_row_end_col(text: &str, target_row: usize, wrap_cols: usize) -> usize {
    let mut row = 0usize;
    let mut col = 0usize;
    for ch in text.chars() {
        if row == target_row {
            if ch == '\n' {
                break;
            }
            col += 1;
            if col >= wrap_cols {
                break;
            }
        } else if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
            if col >= wrap_cols {
                row += 1;
                col = 0;
            }
        }
    }
    col
}
