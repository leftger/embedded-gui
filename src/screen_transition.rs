use heapless::Vec;

use crate::{
    animation::{Animation, Easing},
    context::GuiContext,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenTransitionSpec {
    pub effect: ScreenTransitionEffect,
    pub duration_ms: u32,
}

impl ScreenTransitionSpec {
    pub const fn none() -> Self {
        Self {
            effect: ScreenTransitionEffect::None,
            duration_ms: 0,
        }
    }

    pub const fn fade(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::Fade,
            duration_ms,
        }
    }

    pub const fn slide_left(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::SlideLeft,
            duration_ms,
        }
    }

    pub const fn slide_right(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::SlideRight,
            duration_ms,
        }
    }

    pub const fn zoom(duration_ms: u32) -> Self {
        Self {
            effect: ScreenTransitionEffect::Zoom,
            duration_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveScreenTransition {
    pub from: Option<ScreenId>,
    pub to: Option<ScreenId>,
    pub effect: ScreenTransitionEffect,
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
}

impl ActiveScreenTransition {
    pub fn sample(&self, viewport_w: u32) -> ScreenTransitionSample {
        match self.effect {
            ScreenTransitionEffect::Fade => {
                let incoming = self.opacity_u8();
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    incoming_offset_x: 0,
                    outgoing_opacity: 255u8.saturating_sub(incoming),
                    incoming_opacity: incoming,
                }
            }
            ScreenTransitionEffect::SlideLeft => {
                let out = self.slide_offset_x(viewport_w);
                ScreenTransitionSample {
                    outgoing_offset_x: out,
                    incoming_offset_x: out + viewport_w as i32,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                }
            }
            ScreenTransitionEffect::SlideRight => {
                let out = self.slide_offset_x(viewport_w);
                ScreenTransitionSample {
                    outgoing_offset_x: out,
                    incoming_offset_x: out - viewport_w as i32,
                    outgoing_opacity: 255,
                    incoming_opacity: 255,
                }
            }
            ScreenTransitionEffect::None => ScreenTransitionSample {
                outgoing_offset_x: 0,
                incoming_offset_x: 0,
                outgoing_opacity: 255,
                incoming_opacity: 255,
            },
            ScreenTransitionEffect::Zoom => {
                let incoming = self.opacity_u8();
                ScreenTransitionSample {
                    outgoing_offset_x: 0,
                    incoming_offset_x: 0,
                    outgoing_opacity: 255u8.saturating_sub(incoming / 2),
                    incoming_opacity: incoming,
                }
            }
        }
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
        let anim = Animation::new(0.0, 1.0, spec.duration_ms, Easing::InOutSine);
        self.animation = Some(anim);
        self.active = Some(ActiveScreenTransition {
            from: transition.from,
            to: transition.to,
            effect: spec.effect,
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
) -> Result<(), D::Error>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = embedded_graphics_core::pixelcolor::Rgb565>,
{
    let sample = active.sample(viewport_w);
    outgoing.render_with_offset_and_opacity(
        target,
        sample.outgoing_offset_x,
        0,
        sample.outgoing_opacity,
    )?;
    incoming.render_with_offset_and_opacity(
        target,
        sample.incoming_offset_x,
        0,
        sample.incoming_opacity,
    )?;
    Ok(())
}
