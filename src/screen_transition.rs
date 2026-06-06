use heapless::Vec;

#[cfg(not(feature = "std"))]
use crate::math::F32Ext as _;
use crate::{
    animation::{Animation, Easing},
    animation_timing::{self, timing_half_phase, timing_shutter_phase},
    context::GuiContext,
    geometry::Rect,
    screen::{ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack, ScreenStackError},
};

/// Screen transition visual effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScreenTransitionEffect {
    #[default]
    None,
    Fade,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    /// Rectangular push: incoming from the right with moook easing.
    PushMoook,
    /// Rectangular pop: incoming from the left with moook easing.
    PopMoook,
    Zoom,
    CircularReveal,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    /// Two-phase directional shutter wipe.
    ShutterLeft,
    ShutterRight,
    ShutterUp,
    ShutterDown,
    /// Round-display card flip (vertical clip).
    RoundFlipLeft,
    RoundFlipRight,
    /// Two-phase slide with a mid-transition seam.
    PortHoleLeft,
    PortHoleRight,
    PortHoleUp,
    PortHoleDown,
    /// Modal overlay slide from top or bottom.
    ModalSlideUp,
    ModalSlideDown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScreenTransitionOrigin {
    #[default]
    Center,
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenTransitionSpec {
    pub effect: ScreenTransitionEffect,
    pub duration_ms: u32,
    pub origin: ScreenTransitionOrigin,
    pub easing: Easing,
}

impl ScreenTransitionSpec {
    pub const fn none() -> Self {
        Self {
            effect: ScreenTransitionEffect::None,
            duration_ms: 0,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn fade(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::Fade,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn slide_left(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::SlideLeft,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn slide_right(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::SlideRight,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn slide_up(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::SlideUp,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn slide_down(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::SlideDown,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn push_moook(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::PushMoook,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::Moook,
        }
    }

    pub const fn pop_moook(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::PopMoook,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::Moook,
        }
    }

    pub const fn shutter_left(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::ShutterLeft,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::EaseInOut,
        }
    }

    pub const fn shutter_right(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::ShutterRight,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::EaseInOut,
        }
    }

    pub const fn shutter_up(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::ShutterUp,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::EaseInOut,
        }
    }

    pub const fn shutter_down(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::ShutterDown,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::EaseInOut,
        }
    }

    pub const fn round_flip_left(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::RoundFlipLeft,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::Linear,
        }
    }

    pub const fn round_flip_right(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::RoundFlipRight,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::Linear,
        }
    }

    pub const fn port_hole_left(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::PortHoleLeft,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::EaseInOut,
        }
    }

    pub const fn port_hole_right(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::PortHoleRight,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::EaseInOut,
        }
    }

    pub const fn port_hole_up(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::PortHoleUp,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::EaseInOut,
        }
    }

    pub const fn port_hole_down(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::PortHoleDown,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::EaseInOut,
        }
    }

    pub const fn modal_slide_up(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::ModalSlideUp,
            duration_ms,
            origin: ScreenTransitionOrigin::Bottom,
            easing: Easing::EaseOut,
        }
    }

    pub const fn modal_slide_down(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::ModalSlideDown,
            duration_ms,
            origin: ScreenTransitionOrigin::Top,
            easing: Easing::EaseOut,
        }
    }

    pub const fn zoom(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::Zoom,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn circular_reveal(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::CircularReveal,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn wipe_left(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::WipeLeft,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn wipe_right(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::WipeRight,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn wipe_up(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::WipeUp,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn wipe_down(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::WipeDown,
            duration_ms,
            origin: ScreenTransitionOrigin::Center,
            easing: Easing::InOutSine,
        }
    }

    pub const fn with_origin(mut self, origin: ScreenTransitionOrigin) -> Self {
        self.origin = origin;
        self
    }

    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveScreenTransition {
    pub from: Option<ScreenId>,
    pub to: Option<ScreenId>,
    pub effect: ScreenTransitionEffect,
    pub origin: ScreenTransitionOrigin,
    pub progress: f32,
}

impl ActiveScreenTransition {
    pub fn opacity_u8(&self) -> u8 {
        (self.progress.clamp(0.0, 1.0) * 255.0) as u8
    }

    pub fn slide_offset_x(&self, width: u32) -> i32 {
        let t = eased_progress(self.progress, self.effect);
        let px = (width as f32 * t).round() as i32;
        match self.effect {
            ScreenTransitionEffect::SlideLeft
            | ScreenTransitionEffect::ShutterLeft
            | ScreenTransitionEffect::PortHoleLeft => -px,
            ScreenTransitionEffect::SlideRight
            | ScreenTransitionEffect::PushMoook
            | ScreenTransitionEffect::ShutterRight
            | ScreenTransitionEffect::PortHoleRight
            | ScreenTransitionEffect::RoundFlipRight => px,
            ScreenTransitionEffect::PopMoook => px,
            _ => 0,
        }
    }

    pub fn slide_offset_y(&self, height: u32) -> i32 {
        let t = eased_progress(self.progress, self.effect);
        let px = (height as f32 * t).round() as i32;
        match self.effect {
            ScreenTransitionEffect::SlideUp
            | ScreenTransitionEffect::ShutterUp
            | ScreenTransitionEffect::PortHoleUp
            | ScreenTransitionEffect::ModalSlideUp => -px,
            ScreenTransitionEffect::SlideDown
            | ScreenTransitionEffect::ShutterDown
            | ScreenTransitionEffect::PortHoleDown
            | ScreenTransitionEffect::ModalSlideDown => px,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenTransitionSample {
    pub outgoing_offset_x: i32,
    pub outgoing_offset_y: i32,
    pub incoming_offset_x: i32,
    pub incoming_offset_y: i32,
    pub outgoing_opacity: u8,
    pub incoming_opacity: u8,
    pub outgoing_clip: Option<Rect>,
    pub incoming_clip: Option<Rect>,
}

fn eased_progress(progress: f32, effect: ScreenTransitionEffect) -> f32 {
    match effect {
        ScreenTransitionEffect::PushMoook | ScreenTransitionEffect::PopMoook => {
            animation_timing::moook_curve(progress)
        }
        _ => progress.clamp(0.0, 1.0),
    }
}

fn shutter_offset(progress: f32, viewport: u32, horizontal: bool, negative: bool) -> (i32, i32) {
    let (phase_t, first_half) = timing_shutter_phase(progress);
    let span = viewport as i32;
    let sign = if negative { -1 } else { 1 };
    if horizontal {
        if first_half {
            (sign * -((span as f32 * phase_t).round() as i32), 0)
        } else {
            (
                sign * span,
                sign * (span - (span as f32 * phase_t).round() as i32),
            )
        }
    } else if first_half {
        (0, sign * -((span as f32 * phase_t).round() as i32))
    } else {
        (0, sign * (span - (span as f32 * phase_t).round() as i32))
    }
}

fn port_hole_offsets(
    progress: f32,
    viewport_w: u32,
    viewport_h: u32,
    horizontal: bool,
    negative: bool,
) -> (i32, i32, i32, i32) {
    let viewport = if horizontal { viewport_w } else { viewport_h };
    let (out, inc) = {
        let (phase_t, first_half) = timing_half_phase(progress);
        let gap = (viewport as f32 * 80.0 / 180.0).round() as i32;
        let full = viewport as i32;
        let sign = if negative { -1 } else { 1 };
        if first_half {
            (sign * (full - (gap as f32 * phase_t) as i32), sign * full)
        } else {
            (sign * gap, sign * ((gap as f32 * (1.0 - phase_t)) as i32))
        }
    };
    if horizontal {
        (out, 0, inc, 0)
    } else {
        (0, out, 0, inc)
    }
}

impl ActiveScreenTransition {
    pub fn sample(&self, viewport_w: u32, viewport_h: u32) -> ScreenTransitionSample {
        match self.effect {
            ScreenTransitionEffect::Fade => {
                let incoming = self.opacity_u8();
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: 0,
                    incoming_offset_x: 0,
                    incoming_offset_y: 0,
                    outgoing_opacity: 255u8.saturating_sub(incoming),
                    incoming_opacity: incoming,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::SlideLeft | ScreenTransitionEffect::PushMoook => {
                let out = self.slide_offset_x(viewport_w);
                ScreenTransitionSample {
                    outgoing_offset_x: out,
                    outgoing_offset_y: 0,
                    incoming_offset_x: out + viewport_w as i32,
                    incoming_offset_y: 0,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::SlideRight | ScreenTransitionEffect::PopMoook => {
                let out = self.slide_offset_x(viewport_w);
                ScreenTransitionSample {
                    outgoing_offset_x: out,
                    outgoing_offset_y: 0,
                    incoming_offset_x: out - viewport_w as i32,
                    incoming_offset_y: 0,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::SlideUp | ScreenTransitionEffect::ModalSlideUp => {
                let out = self.slide_offset_y(viewport_h);
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: out,
                    incoming_offset_x: 0,
                    incoming_offset_y: out + viewport_h as i32,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::SlideDown | ScreenTransitionEffect::ModalSlideDown => {
                let out = self.slide_offset_y(viewport_h);
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: out,
                    incoming_offset_x: 0,
                    incoming_offset_y: out - viewport_h as i32,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::ShutterLeft => {
                let (ox, ix) = shutter_offset(self.progress, viewport_w, true, true);
                ScreenTransitionSample {
                    outgoing_offset_x: ox,
                    outgoing_offset_y: 0,
                    incoming_offset_x: ix,
                    incoming_offset_y: 0,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::ShutterRight => {
                let (ox, ix) = shutter_offset(self.progress, viewport_w, true, false);
                ScreenTransitionSample {
                    outgoing_offset_x: ox,
                    outgoing_offset_y: 0,
                    incoming_offset_x: ix,
                    incoming_offset_y: 0,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::ShutterUp => {
                let (oy, iy) = shutter_offset(self.progress, viewport_h, false, true);
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: oy,
                    incoming_offset_x: 0,
                    incoming_offset_y: iy,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::ShutterDown => {
                let (oy, iy) = shutter_offset(self.progress, viewport_h, false, false);
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: oy,
                    incoming_offset_x: 0,
                    incoming_offset_y: iy,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::PortHoleLeft => {
                let (ox, oy, ix, iy) =
                    port_hole_offsets(self.progress, viewport_w, viewport_h, true, true);
                ScreenTransitionSample {
                    outgoing_offset_x: ox,
                    outgoing_offset_y: oy,
                    incoming_offset_x: ix,
                    incoming_offset_y: iy,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::PortHoleRight => {
                let (ox, oy, ix, iy) =
                    port_hole_offsets(self.progress, viewport_w, viewport_h, true, false);
                ScreenTransitionSample {
                    outgoing_offset_x: ox,
                    outgoing_offset_y: oy,
                    incoming_offset_x: ix,
                    incoming_offset_y: iy,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::PortHoleUp => {
                let (ox, oy, ix, iy) =
                    port_hole_offsets(self.progress, viewport_w, viewport_h, false, true);
                ScreenTransitionSample {
                    outgoing_offset_x: ox,
                    outgoing_offset_y: oy,
                    incoming_offset_x: ix,
                    incoming_offset_y: iy,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::PortHoleDown => {
                let (ox, oy, ix, iy) =
                    port_hole_offsets(self.progress, viewport_w, viewport_h, false, false);
                ScreenTransitionSample {
                    outgoing_offset_x: ox,
                    outgoing_offset_y: oy,
                    incoming_offset_x: ix,
                    incoming_offset_y: iy,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::RoundFlipLeft | ScreenTransitionEffect::RoundFlipRight => {
                let (out_clip, in_clip) = round_flip_clip(viewport_w, viewport_h, self.progress);
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: 0,
                    incoming_offset_x: 0,
                    incoming_offset_y: 0,
                    outgoing_opacity: if self.progress < 0.5 { 255 } else { 128 },
                    incoming_opacity: if self.progress < 0.5 { 128 } else { 255 },
                    outgoing_clip: out_clip,
                    incoming_clip: in_clip,
                }
            }
            ScreenTransitionEffect::None => ScreenTransitionSample {
                outgoing_offset_x: 0,
                outgoing_offset_y: 0,
                incoming_offset_x: 0,
                incoming_offset_y: 0,
                outgoing_opacity: 255,
                incoming_opacity: 255,
                outgoing_clip: None,
                incoming_clip: None,
            },
            ScreenTransitionEffect::Zoom => {
                let incoming = self.opacity_u8();
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: 0,
                    incoming_offset_x: 0,
                    incoming_offset_y: 0,
                    outgoing_opacity: 255u8.saturating_sub(incoming / 2),
                    incoming_opacity: incoming,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::CircularReveal => {
                let incoming = self.opacity_u8();
                let clip = reveal_clip(viewport_w, viewport_h, self.progress, self.origin);
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: 0,
                    incoming_offset_x: 0,
                    incoming_offset_y: 0,
                    outgoing_opacity: 255u8.saturating_sub(incoming / 4),
                    incoming_opacity: incoming,
                    outgoing_clip: None,
                    incoming_clip: Some(clip),
                }
            }
            ScreenTransitionEffect::WipeLeft
            | ScreenTransitionEffect::WipeRight
            | ScreenTransitionEffect::WipeUp
            | ScreenTransitionEffect::WipeDown => {
                let clip = wipe_clip(viewport_w, viewport_h, self.progress, self.effect);
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    outgoing_offset_y: 0,
                    incoming_offset_x: 0,
                    incoming_offset_y: 0,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: Some(clip),
                }
            }
        }
    }
}

fn round_flip_clip(
    viewport_w: u32,
    viewport_h: u32,
    progress: f32,
) -> (Option<Rect>, Option<Rect>) {
    let h = viewport_h as i32;
    let mid = h / 2;
    let scale = if progress < 0.5 {
        1.0 - progress * 2.0
    } else {
        (progress - 0.5) * 2.0
    };
    let visible = ((h as f32 * scale).round() as i32).max(1);
    let top = mid - visible / 2;
    let clip = Rect::new(0, top.max(0), viewport_w, visible.max(1) as u32);
    if progress < 0.5 {
        (None, Some(clip))
    } else {
        (Some(clip), Some(clip))
    }
}

fn reveal_clip(
    viewport_w: u32,
    viewport_h: u32,
    progress: f32,
    origin: ScreenTransitionOrigin,
) -> Rect {
    let (cx, cy) = origin_point(viewport_w, viewport_h, origin);
    let max_radius = (((viewport_w as f32).hypot(viewport_h as f32)) * 0.5).ceil() as i32;
    let radius = ((max_radius as f32) * progress.clamp(0.0, 1.0)).ceil() as i32;
    let left = (cx - radius).clamp(0, viewport_w as i32);
    let top = (cy - radius).clamp(0, viewport_h as i32);
    let right = (cx + radius).clamp(0, viewport_w as i32);
    let bottom = (cy + radius).clamp(0, viewport_h as i32);
    Rect::new(
        left,
        top,
        (right - left).max(0) as u32,
        (bottom - top).max(0) as u32,
    )
}

fn origin_point(viewport_w: u32, viewport_h: u32, origin: ScreenTransitionOrigin) -> (i32, i32) {
    let mid_x = viewport_w as i32 / 2;
    let mid_y = viewport_h as i32 / 2;
    let max_x = viewport_w as i32;
    let max_y = viewport_h as i32;
    match origin {
        ScreenTransitionOrigin::Center => (mid_x, mid_y),
        ScreenTransitionOrigin::TopLeft => (0, 0),
        ScreenTransitionOrigin::Top => (mid_x, 0),
        ScreenTransitionOrigin::TopRight => (max_x, 0),
        ScreenTransitionOrigin::Left => (0, mid_y),
        ScreenTransitionOrigin::Right => (max_x, mid_y),
        ScreenTransitionOrigin::BottomLeft => (0, max_y),
        ScreenTransitionOrigin::Bottom => (mid_x, max_y),
        ScreenTransitionOrigin::BottomRight => (max_x, max_y),
    }
}

fn wipe_clip(
    viewport_w: u32,
    viewport_h: u32,
    progress: f32,
    effect: ScreenTransitionEffect,
) -> Rect {
    let w = viewport_w as i32;
    let h = viewport_h as i32;
    let p = progress.clamp(0.0, 1.0);
    match effect {
        ScreenTransitionEffect::WipeLeft => {
            let visible = (w as f32 * p).round() as i32;
            Rect::new(0, 0, visible.max(0) as u32, viewport_h)
        }
        ScreenTransitionEffect::WipeRight => {
            let visible = (w as f32 * p).round() as i32;
            Rect::new((w - visible).max(0), 0, visible.max(0) as u32, viewport_h)
        }
        ScreenTransitionEffect::WipeUp => {
            let visible = (h as f32 * p).round() as i32;
            Rect::new(0, 0, viewport_w, visible.max(0) as u32)
        }
        ScreenTransitionEffect::WipeDown => {
            let visible = (h as f32 * p).round() as i32;
            Rect::new(0, (h - visible).max(0), viewport_w, visible.max(0) as u32)
        }
        _ => Rect::new(0, 0, viewport_w, viewport_h),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenTransitionRunner {
    animation: Option<Animation>,
    active: Option<ActiveScreenTransition>,
}

impl ScreenTransitionRunner {
    pub const fn new() -> Self {
        Self {
            animation: None,
            active: None,
        }
    }

    pub fn apply<const N: usize, const M: usize>(
        &mut self,
        stack: &mut ScreenStack<N>,
        command: ScreenCommand,
        spec: ScreenTransitionSpec,
        lifecycle_events: &mut Vec<ScreenLifecycleEvent, M>,
    ) -> Result<(), ScreenStackError> {
        let transition = stack.apply_lifecycle(command, lifecycle_events)?;
        if spec.effect == ScreenTransitionEffect::None || spec.duration_ms == 0 {
            self.animation = None;
            self.active = None;
            return Ok(());
        }
        let anim = Animation::new(0.0, 1.0, spec.duration_ms, spec.easing);
        self.animation = Some(anim);
        self.active = Some(ActiveScreenTransition {
            from: transition.from,
            to: transition.to,
            effect: spec.effect,
            origin: spec.origin,
            progress: 0.0,
        });
        Ok(())
    }

    pub fn tick(&mut self, dt_ms: u32) {
        let Some(animation) = self.animation.as_mut() else {
            return;
        };
        animation.tick(dt_ms);
        if let Some(active) = self.active.as_mut() {
            active.progress = animation.value();
        }
        if animation.is_done() {
            self.animation = None;
            self.active = None;
        }
    }

    pub fn active(&self) -> Option<ActiveScreenTransition> {
        self.active
    }
}

impl Default for ScreenTransitionRunner {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_transition_pair<'a, D, const NODES: usize, const EVENTS: usize, const DIRTY: usize>(
    target: &mut D,
    outgoing: &GuiContext<'a, NODES, EVENTS, DIRTY>,
    incoming: &GuiContext<'a, NODES, EVENTS, DIRTY>,
    active: ActiveScreenTransition,
    viewport_w: u32,
    viewport_h: u32,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<
            Color = embedded_graphics_core::pixelcolor::Rgb565,
        >,
{
    let sample = active.sample(viewport_w, viewport_h);
    if let Some(clip) = sample.outgoing_clip {
        outgoing.render_with_offset_opacity_and_clip(
            target,
            sample.outgoing_offset_x,
            sample.outgoing_offset_y,
            sample.outgoing_opacity,
            clip,
        )?;
    } else {
        outgoing.render_with_offset_and_opacity(
            target,
            sample.outgoing_offset_x,
            sample.outgoing_offset_y,
            sample.outgoing_opacity,
        )?;
    }
    if let Some(clip) = sample.incoming_clip {
        incoming.render_with_offset_opacity_and_clip(
            target,
            sample.incoming_offset_x,
            sample.incoming_offset_y,
            sample.incoming_opacity,
            clip,
        )?;
    } else {
        incoming.render_with_offset_and_opacity(
            target,
            sample.incoming_offset_x,
            sample.incoming_offset_y,
            sample.incoming_opacity,
        )?;
    }
    Ok(())
}
