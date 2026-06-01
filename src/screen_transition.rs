use heapless::Vec;

use crate::{
    animation::{Animation, Easing},
    context::GuiContext,
    geometry::Rect,
    screen::{ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack, ScreenStackError},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScreenTransitionEffect {
    #[default]
    None,
    Fade,
    SlideLeft,
    SlideRight,
    Zoom,
    CircularReveal,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
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
        let px = (width as f32 * self.progress.clamp(0.0, 1.0)).round() as i32;
        match self.effect {
            ScreenTransitionEffect::SlideLeft => -px,
            ScreenTransitionEffect::SlideRight => px,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenTransitionSample {
    pub outgoing_offset_x: i32,
    pub incoming_offset_x: i32,
    pub outgoing_opacity: u8,
    pub incoming_opacity: u8,
    pub outgoing_clip: Option<Rect>,
    pub incoming_clip: Option<Rect>,
}

impl ActiveScreenTransition {
    pub fn sample(&self, viewport_w: u32, viewport_h: u32) -> ScreenTransitionSample {
        match self.effect {
            ScreenTransitionEffect::Fade => {
                let incoming = self.opacity_u8();
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    incoming_offset_x: 0,
                    outgoing_opacity: 255u8.saturating_sub(incoming),
                    incoming_opacity: incoming,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::SlideLeft => {
                let out = self.slide_offset_x(viewport_w);
                ScreenTransitionSample {
                    outgoing_offset_x: out,
                    incoming_offset_x: out + viewport_w as i32,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::SlideRight => {
                let out = self.slide_offset_x(viewport_w);
                ScreenTransitionSample {
                    outgoing_offset_x: out,
                    incoming_offset_x: out - viewport_w as i32,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: None,
                }
            }
            ScreenTransitionEffect::None => ScreenTransitionSample {
                outgoing_offset_x: 0,
                incoming_offset_x: 0,
                outgoing_opacity: 255,
                incoming_opacity: 255,
                outgoing_clip: None,
                incoming_clip: None,
            },
            ScreenTransitionEffect::Zoom => {
                let incoming = self.opacity_u8();
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    incoming_offset_x: 0,
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
                    incoming_offset_x: 0,
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
                    incoming_offset_x: 0,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                    outgoing_clip: None,
                    incoming_clip: Some(clip),
                }
            }
        }
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
    Rect::new(left, top, (right - left).max(0) as u32, (bottom - top).max(0) as u32)
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

fn wipe_clip(viewport_w: u32, viewport_h: u32, progress: f32, effect: ScreenTransitionEffect) -> Rect {
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

pub fn render_transition_pair<
    'a,
    D,
    const NODES: usize,
    const EVENTS: usize,
    const DIRTY: usize,
>(
    target: &mut D,
    outgoing: &GuiContext<'a, NODES, EVENTS, DIRTY>,
    incoming: &GuiContext<'a, NODES, EVENTS, DIRTY>,
    active: ActiveScreenTransition,
    viewport_w: u32,
    viewport_h: u32,
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = embedded_graphics_core::pixelcolor::Rgb565>,
{
    let sample = active.sample(viewport_w, viewport_h);
    if let Some(clip) = sample.outgoing_clip {
        outgoing.render_with_offset_opacity_and_clip(
            target,
            sample.outgoing_offset_x,
            0,
            sample.outgoing_opacity,
            clip,
        )?;
    } else {
        outgoing.render_with_offset_and_opacity(
            target,
            sample.outgoing_offset_x,
            0,
            sample.outgoing_opacity,
        )?;
    }
    if let Some(clip) = sample.incoming_clip {
        incoming.render_with_offset_opacity_and_clip(
            target,
            sample.incoming_offset_x,
            0,
            sample.incoming_opacity,
            clip,
        )?;
    } else {
        incoming.render_with_offset_and_opacity(
            target,
            sample.incoming_offset_x,
            0,
            sample.incoming_opacity,
        )?;
    }
    Ok(())
}
