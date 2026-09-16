use crate::{
    geometry::{DirtyError, DirtyTracker, Rect},
    haptics::HapticSequencer,
    input::{UiEvent, UiEventFilter, WidgetDispatchPolicy},
    render::RenderQuality,
    style::{Theme, VisualState, WidgetStyle},
    widget::{FocusGroupId, MenuContract, StyleClassId, WidgetId},
    widgets::{TEXTAREA_CAPACITY, WidgetNode},
};
use heapless::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiError {
    WidgetsFull,
    EventsFull,
    DirtyFull,
    NotFound,
}

impl From<DirtyError> for GuiError {
    fn from(_: DirtyError) -> Self {
        Self::DirtyFull
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PressTracker {
    pub(crate) id: WidgetId,
    pub(crate) start_x: i32,
    pub(crate) start_y: i32,
    pub(crate) last_x: i32,
    pub(crate) last_y: i32,
    pub(crate) elapsed_ms: u32,
    pub(crate) long_emitted: bool,
    pub(crate) gesture_emitted: bool,
    pub(crate) repeat_elapsed_ms: u32,
    pub(crate) scroll_velocity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InertiaScroll {
    pub(crate) id: WidgetId,
    pub(crate) velocity: f32,
}

/// An in-progress rubber-band snap-back: `id`'s `ScrollView` offset is
/// overscrolled and animating back to `crate::state::ScrollState::nearest_bound`
/// via `spring` (see `crate::motion::SpringAnimator::rubber_band_snap_back`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollSpringBack {
    pub(crate) id: WidgetId,
    pub(crate) spring: crate::motion::SpringAnimator,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollPhysics {
    pub velocity_threshold: f32,
    pub velocity_decay: f32,
    pub drag_velocity_blend: f32,
    /// When `true`, dragging/flinging a `ScrollView` past `[0, content_h]`
    /// rubber-bands with quadratic resistance instead of hard-clamping, and
    /// snaps back to the boundary with a spring on release — after Flutter's
    /// `BouncingScrollPhysics`. Defaults to `false` (today's hard-clamp
    /// behavior, matching `ClampingScrollPhysics`).
    pub rubber_band: bool,
}

impl Default for ScrollPhysics {
    fn default() -> Self {
        Self {
            velocity_threshold: 0.05,
            velocity_decay: 0.86,
            drag_velocity_blend: 0.4,
            rubber_band: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PressTiming {
    pub long_press_ms: u32,
    pub repeat_delay_ms: u32,
    pub repeat_interval_ms: u32,
}

impl PressTiming {
    pub const fn new(long_press_ms: u32, repeat_delay_ms: u32, repeat_interval_ms: u32) -> Self {
        Self {
            long_press_ms,
            repeat_delay_ms,
            repeat_interval_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetKeyInputPolicy {
    pub raw_select: bool,
    pub raw_back: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyBindingAction {
    Default,
    Ignore,
    Activate,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetKeyBindings {
    pub select: KeyBindingAction,
    pub back: KeyBindingAction,
}

impl Default for WidgetKeyBindings {
    fn default() -> Self {
        Self {
            select: KeyBindingAction::Default,
            back: KeyBindingAction::Default,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextareaSnapshot {
    pub(crate) text_buf: [u8; TEXTAREA_CAPACITY],
    pub(crate) text_len: u8,
    pub(crate) cursor: usize,
    pub(crate) selection: Option<(usize, usize)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextareaHistoryEntry {
    pub(crate) id: WidgetId,
    pub(crate) snapshot: TextareaSnapshot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StateTransition {
    pub(crate) id: WidgetId,
    pub(crate) from: VisualState,
    pub(crate) to: VisualState,
    pub(crate) elapsed_ms: u32,
}

/// A compact GUI context budget suitable for resource-constrained microcontrollers (e.g. Cortex-M0+).
pub type SmallGuiContext<'a> = GuiContext<'a, 16, 8, 8>;

/// A standard balanced GUI context budget suitable for mainstream microcontrollers (e.g. Cortex-M4, ESP32).
pub type StandardGuiContext<'a> = GuiContext<'a, 64, 32, 16>;

/// An expanded GUI context budget suitable for complex multi-screen dashboards and desktop simulation.
pub type LargeGuiContext<'a> = GuiContext<'a, 128, 64, 32>;

pub struct GuiContext<
    'a,
    const NODES: usize = 64,
    const EVENTS: usize = 32,
    const DIRTY: usize = 16,
> {
    pub(crate) viewport: Rect,
    pub(crate) widgets: Vec<WidgetNode<'a>, NODES>,
    pub(crate) subscriptions: Vec<(WidgetId, UiEventFilter), NODES>,
    pub(crate) dispatch_policies: Vec<(WidgetId, WidgetDispatchPolicy), NODES>,
    pub(crate) class_styles: Vec<(StyleClassId, WidgetStyle), NODES>,
    pub(crate) events: Vec<UiEvent, EVENTS>,
    pub(crate) dirty: DirtyTracker<DIRTY>,
    pub(crate) theme: Theme,
    pub(crate) focus: Option<WidgetId>,
    pub(crate) active_focus_group: Option<FocusGroupId>,
    pub(crate) render_quality: RenderQuality,
    pub(crate) long_press_ms: u32,
    pub(crate) textarea_cursor_blink_ms: u32,
    pub(crate) textarea_cursor_blink_elapsed_ms: u32,
    pub(crate) press_repeat_delay_ms: u32,
    pub(crate) press_repeat_interval_ms: u32,
    pub(crate) select_double_window_ms: u32,
    pub(crate) select_elapsed_ms: u32,
    pub(crate) last_select_id: Option<WidgetId>,
    pub(crate) pointer_double_window_ms: u32,
    pub(crate) pointer_elapsed_ms: u32,
    pub(crate) last_pointer_id: Option<WidgetId>,
    pub(crate) pressed: Option<PressTracker>,
    pub(crate) inertia_scroll: Option<InertiaScroll>,
    pub(crate) scroll_spring_back: Option<ScrollSpringBack>,
    pub(crate) scroll_physics: ScrollPhysics,
    pub(crate) state_transition_ms: u32,
    pub(crate) state_transitions: Vec<StateTransition, NODES>,
    pub(crate) widget_press_timings: Vec<(WidgetId, PressTiming), NODES>,
    pub(crate) widget_key_policies: Vec<(WidgetId, WidgetKeyInputPolicy), NODES>,
    pub(crate) widget_key_bindings: Vec<(WidgetId, WidgetKeyBindings), NODES>,
    pub(crate) menu_contract: MenuContract,
    pub(crate) textarea_undo: Vec<TextareaHistoryEntry, NODES>,
    pub(crate) textarea_redo: Vec<TextareaHistoryEntry, NODES>,
    pub(crate) theme_transition_from: Option<Theme>,
    pub(crate) theme_transition_to: Option<Theme>,
    pub(crate) theme_transition_duration_ms: u32,
    pub(crate) theme_transition_elapsed_ms: u32,
    pub(crate) haptic_sequencer: HapticSequencer,
    pub(crate) next_id: u16,
}
