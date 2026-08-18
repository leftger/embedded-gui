#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListState {
    pub selected: usize,
    pub offset: usize,
    pub visible_rows: usize,
}

impl ListState {
    pub const fn new(selected: usize, offset: usize, visible_rows: usize) -> Self {
        Self {
            selected,
            offset,
            visible_rows,
        }
    }

    pub fn set_selected(&mut self, selected: usize, len: usize) -> bool {
        let next = selected.min(len.saturating_sub(1));
        let changed = next != self.selected;
        self.selected = next;
        self.keep_selected_visible();
        changed
    }

    pub fn next(&mut self, len: usize) -> bool {
        self.bump(len, 1)
    }

    pub fn previous(&mut self, len: usize) -> bool {
        self.bump(len, -1)
    }

    pub fn bump(&mut self, len: usize, delta: i8) -> bool {
        if len == 0 {
            return false;
        }
        let next = if delta >= 0 {
            (self.selected + 1) % len
        } else if self.selected == 0 {
            len - 1
        } else {
            self.selected - 1
        };
        self.set_selected(next, len)
    }

    pub fn keep_selected_visible(&mut self) {
        let rows = self.visible_rows.max(1);
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset.saturating_add(rows) {
            self.offset = self.selected.saturating_add(1).saturating_sub(rows);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TabsState {
    pub selected: usize,
}

impl TabsState {
    pub const fn new(selected: usize) -> Self {
        Self { selected }
    }

    pub fn set_selected(&mut self, selected: usize, len: usize) -> bool {
        let next = selected.min(len.saturating_sub(1));
        let changed = next != self.selected;
        self.selected = next;
        changed
    }

    pub fn next(&mut self, len: usize) -> bool {
        self.bump(len, 1)
    }

    pub fn previous(&mut self, len: usize) -> bool {
        self.bump(len, -1)
    }

    pub fn bump(&mut self, len: usize, delta: i8) -> bool {
        if len == 0 {
            return false;
        }
        let next = if delta >= 0 {
            (self.selected + 1) % len
        } else if self.selected == 0 {
            len - 1
        } else {
            self.selected - 1
        };
        self.set_selected(next, len)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollState {
    pub offset_y: i32,
    pub content_h: u32,
}

impl ScrollState {
    pub const fn new(offset_y: i32, content_h: u32) -> Self {
        Self {
            offset_y,
            content_h,
        }
    }

    pub fn set_offset(&mut self, offset_y: i32) -> bool {
        let next = offset_y.clamp(0, self.content_h as i32);
        let changed = next != self.offset_y;
        self.offset_y = next;
        changed
    }

    pub fn scroll_by(&mut self, delta_y: i32) -> bool {
        self.set_offset(self.offset_y.saturating_add(delta_y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderState {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeedTimelineState {
    pub selected: usize,
    pub offset: usize,
    pub visible_rows: usize,
    pub expanded: bool,
}

impl FeedTimelineState {
    pub const fn new(selected: usize, offset: usize, visible_rows: usize, expanded: bool) -> Self {
        Self {
            selected,
            offset,
            visible_rows,
            expanded,
        }
    }

    pub fn set_selected(&mut self, selected: usize, len: usize) -> bool {
        let next = selected.min(len.saturating_sub(1));
        let changed = next != self.selected;
        self.selected = next;
        self.keep_selected_visible();
        changed
    }

    pub fn bump(&mut self, len: usize, delta: i8) -> bool {
        if len == 0 {
            return false;
        }
        let next = if delta >= 0 {
            (self.selected + 1) % len
        } else if self.selected == 0 {
            len - 1
        } else {
            self.selected - 1
        };
        self.set_selected(next, len)
    }

    pub fn set_expanded(&mut self, expanded: bool) -> bool {
        let changed = self.expanded != expanded;
        self.expanded = expanded;
        changed
    }

    pub fn keep_selected_visible(&mut self) {
        let rows = self.visible_rows.max(1);
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset.saturating_add(rows) {
            self.offset = self.selected.saturating_add(1).saturating_sub(rows);
        }
    }
}

impl SliderState {
    pub const fn new(value: f32, min: f32, max: f32) -> Self {
        Self { value, min, max }
    }

    pub fn set_value(&mut self, value: f32) -> bool {
        let next = value.clamp(self.min.min(self.max), self.min.max(self.max));
        let changed = (next - self.value).abs() > f32::EPSILON;
        self.value = next;
        changed
    }

    pub fn step_by(&mut self, direction: f32) -> bool {
        let step = ((self.max - self.min).abs() / 20.0).max(0.01);
        self.set_value(self.value + step * direction)
    }
}

// -----------------------------------------------------------------------------
// Pillar 1: Zero-Allocation Reactive Property & Callback System
// -----------------------------------------------------------------------------

use crate::widget::WidgetId;

/// A reactive signal holding a value `T` with zero-allocation dirty tracking
/// and compile-time bounded subscriber notifications.
#[derive(Clone, Debug)]
pub struct Signal<T, const MAX_SUBSCRIBERS: usize = 4> {
    value: T,
    subscribers: heapless::Vec<WidgetId, MAX_SUBSCRIBERS>,
    dirty: bool,
    version: u32,
}

pub type PropertySignal<T> = Signal<T, 4>;

impl<T: Copy + PartialEq, const N: usize> Signal<T, N> {
    pub const fn new(value: T) -> Self {
        Self {
            value,
            subscribers: heapless::Vec::new(),
            dirty: false,
            version: 0,
        }
    }

    #[inline]
    pub const fn get(&self) -> T {
        self.value
    }

    /// Sets a new value. If the value changes, marks the signal dirty,
    /// increments its version, and returns true.
    pub fn set(&mut self, new_value: T) -> bool {
        if self.value != new_value {
            self.value = new_value;
            self.dirty = true;
            self.version = self.version.wrapping_add(1);
            true
        } else {
            false
        }
    }

    /// Registers a widget to be notified when this signal mutates.
    pub fn subscribe(&mut self, widget_id: WidgetId) -> bool {
        if !self.subscribers.contains(&widget_id) {
            self.subscribers.push(widget_id).is_ok()
        } else {
            true
        }
    }

    /// Unregisters a widget from notifications.
    pub fn unsubscribe(&mut self, widget_id: WidgetId) {
        if let Some(pos) = self.subscribers.iter().position(|&id| id == widget_id) {
            self.subscribers.swap_remove(pos);
        }
    }

    #[inline]
    pub fn subscribers(&self) -> &[WidgetId] {
        self.subscribers.as_slice()
    }

    #[inline]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    #[inline]
    pub const fn version(&self) -> u32 {
        self.version
    }
}

/// A zero-allocation event callback slot holding an optional function pointer.
#[derive(Clone, Copy, Debug, Default)]
pub struct CallbackSlot<E> {
    handler: Option<fn(E)>,
}

impl<E> CallbackSlot<E> {
    pub const fn empty() -> Self {
        Self { handler: None }
    }

    pub const fn new(handler: fn(E)) -> Self {
        Self {
            handler: Some(handler),
        }
    }

    pub fn set(&mut self, handler: fn(E)) {
        self.handler = Some(handler);
    }

    pub fn clear(&mut self) {
        self.handler = None;
    }

    pub fn emit(&self, event: E) {
        if let Some(handler) = self.handler {
            handler(event);
        }
    }

    pub const fn is_bound(&self) -> bool {
        self.handler.is_some()
    }
}

// -----------------------------------------------------------------------------
// Pillar 2: Declarative State Machines & Transition Interpolation
// -----------------------------------------------------------------------------

pub use crate::style::VisualState;

/// Defines an animated transition between two visual states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateTransition {
    pub from: Option<VisualState>,
    pub to: VisualState,
    pub duration_ticks: u32,
}

impl StateTransition {
    pub const fn new(from: Option<VisualState>, to: VisualState, duration_ticks: u32) -> Self {
        Self {
            from,
            to,
            duration_ticks,
        }
    }
}

/// Zero-allocation widget state machine with transition progress interpolation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetStateMachine {
    current: VisualState,
    previous: VisualState,
    target: VisualState,
    progress: f32,
    duration_ticks: u32,
    elapsed_ticks: u32,
}

impl Default for WidgetStateMachine {
    fn default() -> Self {
        Self::new(VisualState::Normal)
    }
}

impl WidgetStateMachine {
    pub const fn new(initial: VisualState) -> Self {
        Self {
            current: initial,
            previous: initial,
            target: initial,
            progress: 1.0,
            duration_ticks: 0,
            elapsed_ticks: 0,
        }
    }

    #[inline]
    pub const fn current(&self) -> VisualState {
        self.current
    }

    #[inline]
    pub const fn target(&self) -> VisualState {
        self.target
    }

    #[inline]
    pub const fn progress(&self) -> f32 {
        self.progress
    }

    #[inline]
    pub const fn is_animating(&self) -> bool {
        self.progress < 1.0
    }

    /// Transitions to a target state over `duration_ticks`.
    pub fn transition_to(&mut self, target: VisualState, duration_ticks: u32) -> bool {
        if self.target == target && self.current == target {
            return false;
        }

        self.previous = self.current;
        self.target = target;
        self.duration_ticks = duration_ticks;
        self.elapsed_ticks = 0;

        if duration_ticks == 0 {
            self.current = target;
            self.progress = 1.0;
        } else {
            self.progress = 0.0;
        }
        true
    }

    /// Ticks the transition animation forward. Returns true if state or progress updated.
    pub fn tick(&mut self, delta_ticks: u32) -> bool {
        if self.progress >= 1.0 {
            return false;
        }

        self.elapsed_ticks = self.elapsed_ticks.saturating_add(delta_ticks);
        if self.elapsed_ticks >= self.duration_ticks {
            self.current = self.target;
            self.progress = 1.0;
        } else {
            self.progress = self.elapsed_ticks as f32 / self.duration_ticks as f32;
        }
        true
    }

    /// Interpolates a scalar value (e.g. opacity, coordinate, scale) between the previous and current state.
    #[inline]
    pub fn lerp_scalar(&self, start_val: f32, target_val: f32) -> f32 {
        start_val + (target_val - start_val) * self.progress
    }
}

// -----------------------------------------------------------------------------
// Pillar 3: Model-View Data Models & Repeater Trait
// -----------------------------------------------------------------------------

/// Change notification event emitted by a data model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelChange {
    RowChanged(usize),
    RowInserted(usize),
    RowRemoved(usize),
    Reset,
}

/// Zero-allocation trait for data models that can be bound to collection widgets / repeaters.
pub trait GuiModel<T> {
    fn row_count(&self) -> usize;
    fn row_data(&self, index: usize) -> Option<T>;
}

/// A slice-backed `GuiModel` implementing zero-allocation list binding.
#[derive(Clone, Copy, Debug)]
pub struct SliceModel<'a, T> {
    items: &'a [T],
}

impl<'a, T> SliceModel<'a, T> {
    pub const fn new(items: &'a [T]) -> Self {
        Self { items }
    }
}

impl<'a, T: Clone> GuiModel<T> for SliceModel<'a, T> {
    #[inline]
    fn row_count(&self) -> usize {
        self.items.len()
    }

    #[inline]
    fn row_data(&self, index: usize) -> Option<T> {
        self.items.get(index).cloned()
    }
}
