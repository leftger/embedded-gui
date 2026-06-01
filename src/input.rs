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
            | Self::Closed => {
                WidgetEventFilter::ACTIVATE
            }
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
