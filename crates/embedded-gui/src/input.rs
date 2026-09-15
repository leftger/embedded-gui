use crate::widget::{EventPhase, WidgetId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerState {
    Pressed,
    Released,
    Moved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    Up,
    Down,
    Left,
    Right,
    SelectLeft,
    SelectRight,
    Home,
    End,
    SelectHome,
    SelectEnd,
    WordLeft,
    WordRight,
    SelectWordLeft,
    SelectWordRight,
    Undo,
    Redo,
    Select,
    SelectPressed,
    SelectReleased,
    Back,
    BackPressed,
    BackReleased,
    Encoder {
        delta: i8,
    },
    Pointer {
        x: i32,
        y: i32,
        state: PointerState,
        button: PointerButton,
    },
}

/// Direction for 2D spatial focus navigation across focusable widgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiEvent {
    FocusChanged {
        old: Option<WidgetId>,
        new: Option<WidgetId>,
    },
    Activate(WidgetId),
    Back,
    Pressed(WidgetId),
    Released(WidgetId),
    Clicked(WidgetId),
    DoubleClicked(WidgetId),
    LongPressed(WidgetId),
    Opened(WidgetId),
    Closed(WidgetId),
    PointerPressed(WidgetId),
    PointerReleased(WidgetId),
    Gesture(WidgetId),
    ValueChanged(WidgetId),
    TextInput {
        id: WidgetId,
        ch: char,
    },
    Focused(WidgetId),
    Defocused(WidgetId),
    Scroll {
        id: WidgetId,
        delta: i32,
    },
    LayoutChanged(WidgetId),
    StyleChanged(WidgetId),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiEventFilter(u16);

impl UiEventFilter {
    pub const FOCUS: Self = Self(1 << 0);
    pub const ACTIVATE: Self = Self(1 << 1);
    pub const BACK: Self = Self(1 << 2);
    pub const POINTER: Self = Self(1 << 3);
    pub const VALUE: Self = Self(1 << 4);
    pub const SCROLL: Self = Self(1 << 5);
    pub const LAYOUT: Self = Self(1 << 6);
    pub const STYLE: Self = Self(1 << 7);

    pub const ALL: Self = Self(
        Self::FOCUS.0
            | Self::ACTIVATE.0
            | Self::BACK.0
            | Self::POINTER.0
            | Self::VALUE.0
            | Self::SCROLL.0
            | Self::LAYOUT.0
            | Self::STYLE.0,
    );

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

impl core::ops::BitOr for UiEventFilter {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl UiEvent {
    pub const fn target(self) -> Option<WidgetId> {
        match self {
            Self::FocusChanged { new, .. } => new,
            Self::Activate(id)
            | Self::Pressed(id)
            | Self::Released(id)
            | Self::Clicked(id)
            | Self::DoubleClicked(id)
            | Self::LongPressed(id)
            | Self::Opened(id)
            | Self::Closed(id)
            | Self::PointerPressed(id)
            | Self::PointerReleased(id)
            | Self::Gesture(id)
            | Self::ValueChanged(id)
            | Self::TextInput { id, .. }
            | Self::Focused(id)
            | Self::Defocused(id)
            | Self::LayoutChanged(id)
            | Self::StyleChanged(id) => Some(id),
            Self::Scroll { id, .. } => Some(id),
            Self::Back => None,
        }
    }

    pub const fn filter(self) -> UiEventFilter {
        match self {
            Self::FocusChanged { .. } | Self::Focused(_) | Self::Defocused(_) => {
                UiEventFilter::FOCUS
            }
            Self::Activate(_)
            | Self::Pressed(_)
            | Self::Released(_)
            | Self::Clicked(_)
            | Self::DoubleClicked(_)
            | Self::LongPressed(_)
            | Self::Opened(_)
            | Self::Closed(_) => UiEventFilter::ACTIVATE,
            Self::Back => UiEventFilter::BACK,
            Self::PointerPressed(_) | Self::PointerReleased(_) | Self::Gesture(_) => {
                UiEventFilter::POINTER
            }
            Self::ValueChanged(_) | Self::TextInput { .. } => UiEventFilter::VALUE,
            Self::Scroll { .. } => UiEventFilter::SCROLL,
            Self::LayoutChanged(_) => UiEventFilter::LAYOUT,
            Self::StyleChanged(_) => UiEventFilter::STYLE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetEventKind {
    Pressed,
    Released,
    Clicked,
    DoubleClicked,
    LongPressed,
    Opened,
    Closed,
    ValueChanged,
    Focused,
    Defocused,
    Scroll { delta: i32 },
    Gesture,
    LayoutChanged,
    StyleChanged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetEvent {
    pub target: WidgetId,
    pub current: WidgetId,
    pub phase: EventPhase,
    pub kind: WidgetEventKind,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetEventFilter(u16);

impl WidgetEventFilter {
    pub const POINTER: Self = Self(1 << 0);
    pub const ACTIVATE: Self = Self(1 << 1);
    pub const VALUE: Self = Self(1 << 2);
    pub const FOCUS: Self = Self(1 << 3);
    pub const SCROLL: Self = Self(1 << 4);
    pub const LAYOUT: Self = Self(1 << 5);
    pub const STYLE: Self = Self(1 << 6);

    pub const ALL: Self = Self(
        Self::POINTER.0
            | Self::ACTIVATE.0
            | Self::VALUE.0
            | Self::FOCUS.0
            | Self::SCROLL.0
            | Self::LAYOUT.0
            | Self::STYLE.0,
    );

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for WidgetEventFilter {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl WidgetEventKind {
    pub const fn filter(self) -> WidgetEventFilter {
        match self {
            Self::Pressed | Self::Released | Self::Gesture => WidgetEventFilter::POINTER,
            Self::Clicked
            | Self::DoubleClicked
            | Self::LongPressed
            | Self::Opened
            | Self::Closed => WidgetEventFilter::ACTIVATE,
            Self::ValueChanged => WidgetEventFilter::VALUE,
            Self::Focused | Self::Defocused => WidgetEventFilter::FOCUS,
            Self::Scroll { .. } => WidgetEventFilter::SCROLL,
            Self::LayoutChanged => WidgetEventFilter::LAYOUT,
            Self::StyleChanged => WidgetEventFilter::STYLE,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventPhaseMask(u8);

impl EventPhaseMask {
    pub const CAPTURE: Self = Self(1 << 0);
    pub const TARGET: Self = Self(1 << 1);
    pub const BUBBLE: Self = Self(1 << 2);
    pub const ALL: Self = Self(Self::CAPTURE.0 | Self::TARGET.0 | Self::BUBBLE.0);

    pub const fn contains(self, phase: EventPhase) -> bool {
        let bit = match phase {
            EventPhase::Capture => Self::CAPTURE.0,
            EventPhase::Target => Self::TARGET.0,
            EventPhase::Bubble => Self::BUBBLE.0,
        };
        self.0 & bit == bit
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetDispatchPolicy {
    pub kinds: WidgetEventFilter,
    pub phases: EventPhaseMask,
    pub stop: bool,
}

impl WidgetDispatchPolicy {
    pub const fn stop(kinds: WidgetEventFilter, phases: EventPhaseMask) -> Self {
        Self {
            kinds,
            phases,
            stop: true,
        }
    }

    pub const fn allows(self, kind: WidgetEventKind, phase: EventPhase) -> bool {
        self.kinds.contains(kind.filter()) && self.phases.contains(phase)
    }
}

/// PebbleOS-inspired tactile button click recognizer.
///
/// Distinguishes single clicks, multi-clicks (double/triple), long-press threshold
/// triggers, and hold-repeat stream events with deterministic timing windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickEvent {
    SingleClick,
    MultiClick(u8),
    LongClickStart,
    LongClickEnd,
    HoldRepeat(u16),
}

#[derive(Clone, Debug)]
pub struct ClickRecognizer {
    pub multi_click_timeout_ms: u32,
    pub long_press_threshold_ms: u32,
    pub repeat_interval_ms: u32,

    is_down: bool,
    down_duration_ms: u32,
    idle_time_ms: u32,
    click_count: u8,
    long_press_fired: bool,
    repeat_counter: u16,
    last_repeat_ms: u32,
}

impl Default for ClickRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickRecognizer {
    pub const fn new() -> Self {
        Self {
            multi_click_timeout_ms: 300,
            long_press_threshold_ms: 500,
            repeat_interval_ms: 100,

            is_down: false,
            down_duration_ms: 0,
            idle_time_ms: 0,
            click_count: 0,
            long_press_fired: false,
            repeat_counter: 0,
            last_repeat_ms: 0,
        }
    }

    /// Notify that the button transitioned to the pressed state.
    pub fn on_press(&mut self) {
        self.is_down = true;
        self.down_duration_ms = 0;
        self.long_press_fired = false;
        self.repeat_counter = 0;
        self.last_repeat_ms = 0;
    }

    /// Notify that the button transitioned to the released state.
    ///
    /// Returns a `ClickEvent` immediately if a long press ended.
    pub fn on_release(&mut self) -> Option<ClickEvent> {
        if !self.is_down {
            return None;
        }
        self.is_down = false;

        if self.long_press_fired {
            self.long_press_fired = false;
            return Some(ClickEvent::LongClickEnd);
        }

        // Short press recorded
        self.click_count = self.click_count.saturating_add(1);
        self.idle_time_ms = 0;
        None
    }

    /// Advance time by `dt_ms` milliseconds and evaluate gestures.
    pub fn update(&mut self, dt_ms: u32) -> Option<ClickEvent> {
        if self.is_down {
            self.down_duration_ms += dt_ms;

            if !self.long_press_fired && self.down_duration_ms >= self.long_press_threshold_ms {
                self.long_press_fired = true;
                self.click_count = 0;
                self.last_repeat_ms = self.down_duration_ms;
                return Some(ClickEvent::LongClickStart);
            }

            if self.long_press_fired && self.repeat_interval_ms > 0 {
                let since_last = self.down_duration_ms.saturating_sub(self.last_repeat_ms);
                if since_last >= self.repeat_interval_ms {
                    self.last_repeat_ms = self.down_duration_ms;
                    self.repeat_counter = self.repeat_counter.saturating_add(1);
                    return Some(ClickEvent::HoldRepeat(self.repeat_counter));
                }
            }
        } else if self.click_count > 0 {
            self.idle_time_ms += dt_ms;
            if self.idle_time_ms >= self.multi_click_timeout_ms {
                let count = self.click_count;
                self.click_count = 0;
                self.idle_time_ms = 0;
                return if count == 1 {
                    Some(ClickEvent::SingleClick)
                } else {
                    Some(ClickEvent::MultiClick(count))
                };
            }
        }

        None
    }

    /// Reset recognizer state.
    pub fn reset(&mut self) {
        self.is_down = false;
        self.down_duration_ms = 0;
        self.idle_time_ms = 0;
        self.click_count = 0;
        self.long_press_fired = false;
        self.repeat_counter = 0;
    }
}

/// Disambiguates competing gesture recognizers sharing one pointer stream
/// (e.g. tap vs. long-press vs. drag on the same widget), after Flutter's
/// `GestureArenaManager` (`packages/flutter/lib/src/gestures/arena.dart`).
///
/// Unlike Flutter's version, members aren't trait objects notified via
/// callbacks (`acceptGesture`/`rejectGesture`) — recognizers are plain `u8`
/// ids supplied by the caller, and the arena only tracks the eventual
/// winner. Callers poll [`GestureArena::winner`] (or [`GestureArena::accepted`]/
/// [`GestureArena::rejected`] for a specific id) instead of receiving a
/// callback, matching this crate's poll-based (`update(dt_ms)`) recognizer
/// style and avoiding the need for `dyn` dispatch or heap allocation.
///
/// Resolution rules (identical to Flutter's arena):
/// - While open, a member may self-declare victory via [`accept`](Self::accept);
///   that becomes the "eager winner" but doesn't resolve the arena yet.
/// - [`close`](Self::close) stops new members from joining and resolves
///   immediately if only one member remains, or if there's an eager winner.
/// - [`sweep`](Self::sweep) (called once the pointer is released) picks the
///   first remaining member as a default winner, unless the arena is
///   [`hold`](Self::hold)-ing (e.g. a long-press recognizer still deciding),
///   in which case the sweep is deferred until [`release`](Self::release).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GestureArena<const N: usize> {
    members: heapless::Vec<u8, N>,
    is_open: bool,
    is_held: bool,
    has_pending_sweep: bool,
    eager_winner: Option<u8>,
    winner: Option<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureArenaError {
    /// The arena is already full (`N` members already joined).
    Full,
    /// [`GestureArena::add`] was called after [`GestureArena::close`].
    Closed,
}

impl<const N: usize> Default for GestureArena<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> GestureArena<N> {
    pub const fn new() -> Self {
        Self {
            members: heapless::Vec::new(),
            is_open: true,
            is_held: false,
            has_pending_sweep: false,
            eager_winner: None,
            winner: None,
        }
    }

    /// Registers a recognizer while the arena is still open.
    pub fn add(&mut self, id: u8) -> Result<(), GestureArenaError> {
        if !self.is_open {
            return Err(GestureArenaError::Closed);
        }
        self.members.push(id).map_err(|_| GestureArenaError::Full)
    }

    /// Stops accepting new members and resolves immediately if possible
    /// (single member, or an eager winner already self-declared).
    pub fn close(&mut self) {
        if !self.is_open {
            return;
        }
        self.is_open = false;
        self.try_resolve();
    }

    /// A member claims victory. While the arena is still open this only
    /// records `id` as the eager winner (resolution happens at [`close`](Self::close));
    /// once closed, it resolves the arena in `id`'s favor immediately.
    pub fn accept(&mut self, id: u8) {
        if self.winner.is_some() {
            return;
        }
        if self.is_open {
            if self.eager_winner.is_none() {
                self.eager_winner = Some(id);
            }
        } else {
            self.resolve_in_favor_of(id);
        }
    }

    /// A member concedes defeat and leaves the arena.
    pub fn reject(&mut self, id: u8) {
        if self.winner.is_some() {
            return;
        }
        if self.eager_winner == Some(id) {
            self.eager_winner = None;
        }
        if let Some(pos) = self.members.iter().position(|&m| m == id) {
            self.members.remove(pos);
        }
        if !self.is_open {
            self.try_resolve();
        }
    }

    /// Forces resolution in favor of the first remaining member, unless the
    /// arena is held (see [`hold`](Self::hold)). Returns the winner, if any.
    pub fn sweep(&mut self) -> Option<u8> {
        if self.winner.is_some() {
            return self.winner;
        }
        if self.is_held {
            self.has_pending_sweep = true;
            return None;
        }
        if let Some(&first) = self.members.first() {
            self.resolve_in_favor_of(first);
        }
        self.winner
    }

    /// Defers [`sweep`](Self::sweep) until [`release`](Self::release) is
    /// called, for a recognizer (e.g. long-press) that needs more time than
    /// a single pointer-up to decide.
    pub fn hold(&mut self) {
        self.is_held = true;
    }

    /// Releases a [`hold`](Self::hold); if a sweep was attempted while held,
    /// it runs now. Returns the winner, if the release triggered one.
    pub fn release(&mut self) -> Option<u8> {
        self.is_held = false;
        if self.has_pending_sweep {
            self.has_pending_sweep = false;
            return self.sweep();
        }
        None
    }

    /// The resolved winner, if the arena has settled.
    pub const fn winner(&self) -> Option<u8> {
        self.winner
    }

    /// Whether `id` won the arena.
    pub fn accepted(&self, id: u8) -> bool {
        self.winner == Some(id)
    }

    /// Whether `id` lost the arena (only meaningful once resolved).
    pub fn rejected(&self, id: u8) -> bool {
        self.winner.is_some() && self.winner != Some(id)
    }

    /// Whether the arena has resolved to a winner.
    pub const fn is_resolved(&self) -> bool {
        self.winner.is_some()
    }

    /// Clears all state so the arena can be reused for the next pointer.
    pub fn reset(&mut self) {
        self.members.clear();
        self.is_open = true;
        self.is_held = false;
        self.has_pending_sweep = false;
        self.eager_winner = None;
        self.winner = None;
    }

    fn try_resolve(&mut self) {
        if self.members.len() == 1 {
            let only = self.members[0];
            self.resolve_in_favor_of(only);
        } else if let Some(w) = self.eager_winner {
            self.resolve_in_favor_of(w);
        }
    }

    fn resolve_in_favor_of(&mut self, id: u8) {
        self.winner = Some(id);
        self.members.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_recognizer_single_click() {
        let mut recognizer = ClickRecognizer::new();
        recognizer.on_press();
        recognizer.update(50);
        assert_eq!(recognizer.on_release(), None);

        // Advance past multi-click timeout (300ms)
        let evt = recognizer.update(305);
        assert_eq!(evt, Some(ClickEvent::SingleClick));
    }

    #[test]
    fn test_click_recognizer_double_click() {
        let mut recognizer = ClickRecognizer::new();
        recognizer.on_press();
        recognizer.update(50);
        recognizer.on_release();

        recognizer.update(50); // Pause between clicks
        recognizer.on_press();
        recognizer.update(50);
        recognizer.on_release();

        // Expire multi-click window
        let evt = recognizer.update(305);
        assert_eq!(evt, Some(ClickEvent::MultiClick(2)));
    }

    #[test]
    fn test_click_recognizer_long_press_and_repeat() {
        let mut recognizer = ClickRecognizer::new();
        recognizer.on_press();

        // Wait past 500ms
        let evt1 = recognizer.update(505);
        assert_eq!(evt1, Some(ClickEvent::LongClickStart));

        // Hold for 100ms more -> first repeat
        let evt2 = recognizer.update(105);
        assert_eq!(evt2, Some(ClickEvent::HoldRepeat(1)));

        // Release
        let evt3 = recognizer.on_release();
        assert_eq!(evt3, Some(ClickEvent::LongClickEnd));
    }

    #[test]
    fn test_event_filters_masks_and_ui_event_targets() {
        let mut filter = UiEventFilter::empty();
        assert!(UiEventFilter::ALL.contains(UiEventFilter::FOCUS));
        filter.insert(UiEventFilter::POINTER);
        filter.insert(UiEventFilter::VALUE | UiEventFilter::BACK);
        assert!(filter.contains(UiEventFilter::POINTER));
        assert!(filter.contains(UiEventFilter::VALUE));
        filter.remove(UiEventFilter::POINTER);
        assert!(!filter.contains(UiEventFilter::POINTER));

        let id = WidgetId::new(3);
        let events = [
            UiEvent::FocusChanged {
                old: None,
                new: Some(id),
            },
            UiEvent::Activate(id),
            UiEvent::Back,
            UiEvent::Pressed(id),
            UiEvent::Released(id),
            UiEvent::Clicked(id),
            UiEvent::DoubleClicked(id),
            UiEvent::LongPressed(id),
            UiEvent::Opened(id),
            UiEvent::Closed(id),
            UiEvent::PointerPressed(id),
            UiEvent::PointerReleased(id),
            UiEvent::Gesture(id),
            UiEvent::ValueChanged(id),
            UiEvent::TextInput { id, ch: 'x' },
            UiEvent::Focused(id),
            UiEvent::Defocused(id),
            UiEvent::Scroll { id, delta: 1 },
            UiEvent::LayoutChanged(id),
            UiEvent::StyleChanged(id),
        ];
        for e in events {
            let _ = e.target();
            let _ = e.filter();
        }
        assert_eq!(UiEvent::Back.target(), None);

        let mut wf = WidgetEventFilter::ALL;
        assert!(wf.contains(WidgetEventFilter::POINTER));
        wf = WidgetEventFilter::POINTER | WidgetEventFilter::VALUE;
        assert!(wf.contains(WidgetEventFilter::POINTER));
        assert!(!wf.contains(WidgetEventFilter::FOCUS));

        let kinds = [
            WidgetEventKind::Pressed,
            WidgetEventKind::Released,
            WidgetEventKind::Clicked,
            WidgetEventKind::DoubleClicked,
            WidgetEventKind::LongPressed,
            WidgetEventKind::Opened,
            WidgetEventKind::Closed,
            WidgetEventKind::ValueChanged,
            WidgetEventKind::Focused,
            WidgetEventKind::Defocused,
            WidgetEventKind::Scroll { delta: 1 },
            WidgetEventKind::Gesture,
            WidgetEventKind::LayoutChanged,
            WidgetEventKind::StyleChanged,
        ];
        for k in kinds {
            let _ = k.filter();
        }

        assert!(EventPhaseMask::ALL.contains(EventPhase::Capture));
        assert!(EventPhaseMask::ALL.contains(EventPhase::Bubble));
        let policy = WidgetDispatchPolicy::stop(WidgetEventFilter::POINTER, EventPhaseMask::ALL);
        assert!(policy.stop);
        assert!(policy.allows(WidgetEventKind::Pressed, EventPhase::Target));
        assert!(!policy.allows(WidgetEventKind::Clicked, EventPhase::Target));
    }

    #[test]
    fn test_click_recognizer_reset_and_release_without_press() {
        let mut recognizer = ClickRecognizer::default();
        assert_eq!(recognizer.on_release(), None);
        recognizer.on_press();
        recognizer.reset();
        assert_eq!(recognizer.on_release(), None);
    }

    #[test]
    fn gesture_arena_default_sweep_picks_first_member() {
        let mut arena = GestureArena::<4>::new();
        arena.add(0).unwrap();
        arena.add(1).unwrap();
        arena.add(2).unwrap();
        assert!(!arena.is_resolved());

        arena.close();
        assert!(!arena.is_resolved(), "still 3 members, no eager winner");

        assert_eq!(arena.sweep(), Some(0));
        assert!(arena.accepted(0));
        assert!(arena.rejected(1));
        assert!(arena.rejected(2));
    }

    #[test]
    fn gesture_arena_reduces_to_sole_member_on_close() {
        let mut arena = GestureArena::<4>::new();
        arena.add(7).unwrap();
        arena.close();
        assert_eq!(arena.winner(), Some(7));
    }

    #[test]
    fn gesture_arena_eager_winner_resolves_on_close() {
        let mut arena = GestureArena::<4>::new();
        arena.add(0).unwrap();
        arena.add(1).unwrap();
        arena.accept(1); // self-declares while still open: eager winner only
        assert!(!arena.is_resolved());
        arena.close();
        assert_eq!(arena.winner(), Some(1));
        assert!(arena.rejected(0));
    }

    #[test]
    fn gesture_arena_reject_can_collapse_to_default_winner() {
        let mut arena = GestureArena::<4>::new();
        arena.add(0).unwrap();
        arena.add(1).unwrap();
        arena.close();
        arena.reject(0);
        // Rejecting down to a single remaining member resolves immediately.
        assert_eq!(arena.winner(), Some(1));
    }

    #[test]
    fn gesture_arena_hold_defers_sweep_until_release() {
        let mut arena = GestureArena::<4>::new();
        arena.add(0).unwrap();
        arena.add(1).unwrap();
        arena.close();
        arena.hold();
        assert_eq!(arena.sweep(), None, "sweep deferred while held");
        assert!(!arena.is_resolved());
        assert_eq!(arena.release(), Some(0), "pending sweep runs on release");
    }

    #[test]
    fn gesture_arena_accept_after_close_wins_immediately() {
        let mut arena = GestureArena::<4>::new();
        arena.add(0).unwrap();
        arena.add(1).unwrap();
        arena.close();
        arena.accept(1);
        assert_eq!(arena.winner(), Some(1));
        assert!(arena.rejected(0));
    }

    #[test]
    fn gesture_arena_capacity_and_closed_errors() {
        let mut arena = GestureArena::<2>::new();
        arena.add(0).unwrap();
        arena.add(1).unwrap();
        assert_eq!(arena.add(2), Err(GestureArenaError::Full));

        let mut arena2 = GestureArena::<2>::new();
        arena2.close();
        assert_eq!(arena2.add(0), Err(GestureArenaError::Closed));
    }

    #[test]
    fn gesture_arena_reset_allows_reuse() {
        let mut arena = GestureArena::<4>::new();
        arena.add(0).unwrap();
        arena.close();
        assert_eq!(arena.winner(), Some(0));
        arena.reset();
        assert!(!arena.is_resolved());
        arena.add(5).unwrap();
        arena.close();
        assert_eq!(arena.winner(), Some(5));
    }
}
