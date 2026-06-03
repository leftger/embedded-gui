//! Named screen transition presets with default durations and easing.
//!
//! Durations assume a 30 Hz frame interval (`FRAME_INTERVAL_MS` = 33 ms).

use crate::{
    animation::Easing,
    animation_timing::{
        DEFAULT_DURATION_MS, MOOOK_DURATION_MS, PORT_HOLE_DURATION_MS, SHUTTER_DURATION_MS,
    },
    screen_transition::{ScreenTransitionEffect, ScreenTransitionSpec},
};

/// Built-in screen transition presets for common navigation patterns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionPreset {
    /// Instant cut (no animation).
    None,
    /// Horizontal push onto the stack (spatial moook curve).
    WindowPush,
    /// Horizontal pop off the stack (spatial moook curve).
    WindowPop,
    /// Round-style push (two-phase port-hole slide).
    WindowPushRound,
    /// Round-style pop (two-phase port-hole slide).
    WindowPopRound,
    /// Directional shutter wipes.
    ShutterUp,
    ShutterDown,
    ShutterLeft,
    ShutterRight,
    /// Card-style flip toward launcher.
    RoundFlipToLauncher,
    /// Card-style flip from launcher.
    RoundFlipFromLauncher,
    /// Two-phase port-hole slides.
    PortHoleUp,
    PortHoleDown,
    PortHoleLeft,
    PortHoleRight,
    /// Modal presented from the bottom.
    ModalPresent,
    /// Modal dismissed toward the bottom.
    ModalDismiss,
    /// Timeline-style horizontal slide.
    TimelineSlide,
    /// Timeline peek card enters with a soft rightward reveal.
    TimelinePeekIn,
    /// Timeline peek card exits with a soft leftward collapse.
    TimelinePeekOut,
    /// Timeline pin details expand from lower edge.
    TimelinePinExpand,
    /// Timeline scrub release settles quickly.
    TimelineScrubSettle,
    /// Cross-fade.
    Fade,
}

impl TransitionPreset {
    pub const fn spec(self) -> ScreenTransitionSpec {
        match self {
            Self::None => ScreenTransitionSpec::none(),
            Self::WindowPush => ScreenTransitionSpec::push_moook(MOOOK_DURATION_MS),
            Self::WindowPop => ScreenTransitionSpec::pop_moook(MOOOK_DURATION_MS),
            Self::WindowPushRound => ScreenTransitionSpec::port_hole_left(PORT_HOLE_DURATION_MS),
            Self::WindowPopRound => ScreenTransitionSpec::port_hole_right(PORT_HOLE_DURATION_MS),
            Self::ShutterUp => ScreenTransitionSpec::shutter_up(SHUTTER_DURATION_MS),
            Self::ShutterDown => ScreenTransitionSpec::shutter_down(SHUTTER_DURATION_MS),
            Self::ShutterLeft => ScreenTransitionSpec::shutter_left(SHUTTER_DURATION_MS),
            Self::ShutterRight => ScreenTransitionSpec::shutter_right(SHUTTER_DURATION_MS),
            Self::RoundFlipToLauncher => {
                ScreenTransitionSpec::round_flip_right(PORT_HOLE_DURATION_MS)
            }
            Self::RoundFlipFromLauncher => {
                ScreenTransitionSpec::round_flip_left(PORT_HOLE_DURATION_MS)
            }
            Self::PortHoleUp => ScreenTransitionSpec::port_hole_up(PORT_HOLE_DURATION_MS),
            Self::PortHoleDown => ScreenTransitionSpec::port_hole_down(PORT_HOLE_DURATION_MS),
            Self::PortHoleLeft => ScreenTransitionSpec::port_hole_left(PORT_HOLE_DURATION_MS),
            Self::PortHoleRight => ScreenTransitionSpec::port_hole_right(PORT_HOLE_DURATION_MS),
            Self::ModalPresent => ScreenTransitionSpec::modal_slide_up(DEFAULT_DURATION_MS),
            Self::ModalDismiss => ScreenTransitionSpec::modal_slide_down(DEFAULT_DURATION_MS),
            Self::TimelineSlide => {
                ScreenTransitionSpec::slide_left(DEFAULT_DURATION_MS).with_easing(Easing::EaseInOut)
            }
            Self::TimelinePeekIn => ScreenTransitionSpec::slide_right(MOOOK_DURATION_MS / 2)
                .with_easing(Easing::OutBack),
            Self::TimelinePeekOut => {
                ScreenTransitionSpec::slide_left(MOOOK_DURATION_MS / 2).with_easing(Easing::InSine)
            }
            Self::TimelinePinExpand => ScreenTransitionSpec::modal_slide_up(MOOOK_DURATION_MS / 2)
                .with_easing(Easing::OutCubic),
            Self::TimelineScrubSettle => ScreenTransitionSpec::slide_left(DEFAULT_DURATION_MS / 2)
                .with_easing(Easing::OutBounce),
            Self::Fade => ScreenTransitionSpec::fade(DEFAULT_DURATION_MS),
        }
    }

    pub const fn effect(self) -> ScreenTransitionEffect {
        self.spec().effect
    }
}

impl From<TransitionPreset> for ScreenTransitionSpec {
    fn from(value: TransitionPreset) -> Self {
        value.spec()
    }
}
