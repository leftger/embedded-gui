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
}
