//! High-level cinematic UI building blocks inspired by smartwatch UX patterns.
//! These helpers stay no_std friendly and compose with existing widgets/animators.

use crate::{
    animation::Easing,
    context::{GuiContext, GuiError},
    widget::WidgetId,
    widget_animation::{AnimationConflictPolicy, WidgetAnimationError, WidgetAnimator},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeekRevealSpec {
    pub dot_px: u32,
    pub icon_expand_px: u32,
    pub icon_duration_ms: u32,
    pub text_stagger_ms: u32,
    pub text_duration_ms: u32,
}

impl Default for PeekRevealSpec {
    fn default() -> Self {
        Self {
            dot_px: 3,
            icon_expand_px: 24,
            icon_duration_ms: 300,
            text_stagger_ms: 90,
            text_duration_ms: 160,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlanceTileSpec {
    pub focus_bump_px: i32,
    pub focus_slide_px: i32,
    pub focus_duration_ms: u32,
    pub dim_opacity: u8,
}

impl Default for GlanceTileSpec {
    fn default() -> Self {
        Self {
            focus_bump_px: 3,
            focus_slide_px: 6,
            focus_duration_ms: 120,
            dim_opacity: 170,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardDeckDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardDeckState {
    current: usize,
    len: usize,
}

impl CardDeckState {
    pub const fn new(len: usize) -> Self {
        Self { current: 0, len }
    }

    pub const fn current(&self) -> usize {
        self.current
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn set_len(&mut self, len: usize) {
        self.len = len;
        if self.current >= self.len {
            self.current = self.len.saturating_sub(1);
        }
    }

    pub fn move_next(&mut self) -> Option<CardDeckDirection> {
        if self.current + 1 < self.len {
            self.current += 1;
            Some(CardDeckDirection::Forward)
        } else {
            None
        }
    }

    pub fn move_prev(&mut self) -> Option<CardDeckDirection> {
        if self.current > 0 {
            self.current -= 1;
            Some(CardDeckDirection::Backward)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineMotionPreset {
    PeekIn,
    PeekOut,
    PinExpand,
    ScrubSettle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CinematicPreset {
    PeekTimeline,
    LauncherGlance,
    CardStory,
}

impl CinematicPreset {
    pub const fn name(self) -> &'static str {
        match self {
            Self::PeekTimeline => "peek-timeline",
            Self::LauncherGlance => "launcher-glance",
            Self::CardStory => "card-story",
        }
    }
}

impl TimelineMotionPreset {
    pub const fn duration_ms(self) -> u32 {
        match self {
            Self::PeekIn | Self::PeekOut => 220,
            Self::PinExpand => 260,
            Self::ScrubSettle => 140,
        }
    }

    pub const fn easing(self) -> Easing {
        match self {
            Self::PeekIn => Easing::OutBack,
            Self::PeekOut => Easing::InSine,
            Self::PinExpand => Easing::OutCubic,
            Self::ScrubSettle => Easing::OutBounce,
        }
    }
}

pub fn animate_peek_reveal<const TRACKS: usize, const BINDINGS: usize>(
    animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
    icon_widget: WidgetId,
    title_widget: Option<WidgetId>,
    subtitle_widget: Option<WidgetId>,
    base_x: i32,
    base_y: i32,
    spec: PeekRevealSpec,
) -> Result<(), WidgetAnimationError> {
    let dot = spec.dot_px.max(1);
    animator.animate_widget_width(icon_widget, dot, spec.icon_expand_px.max(dot), spec.icon_duration_ms, Easing::OutBack)?;
    animator.animate_widget_height(icon_widget, dot, spec.icon_expand_px.max(dot), spec.icon_duration_ms, Easing::OutBack)?;
    animator.animate_opacity(icon_widget, 180, 255, spec.icon_duration_ms, Easing::OutSine)?;

    if let Some(title) = title_widget {
        let title_anim = crate::animation::Animation::new(
            (base_y + 4) as f32,
            base_y as f32,
            spec.text_duration_ms,
            Easing::OutCubic,
        )
        .with_delay(spec.text_stagger_ms);
        animator.bind_property_with_policy(
            title,
            crate::widget_animation::AnimatedProperty::WidgetY,
            title_anim,
            AnimationConflictPolicy::Replace,
        )?;
        animator.animate_opacity(title, 0, 255, spec.text_duration_ms + spec.text_stagger_ms, Easing::OutSine)?;
    }

    if let Some(subtitle) = subtitle_widget {
        let subtitle_anim = crate::animation::Animation::new(
            (base_x - 6) as f32,
            base_x as f32,
            spec.text_duration_ms,
            Easing::OutSine,
        )
        .with_delay(spec.text_stagger_ms.saturating_mul(2));
        animator.bind_property_with_policy(
            subtitle,
            crate::widget_animation::AnimatedProperty::WidgetX,
            subtitle_anim,
            AnimationConflictPolicy::Replace,
        )?;
        animator.animate_opacity(
            subtitle,
            0,
            255,
            spec.text_duration_ms + spec.text_stagger_ms.saturating_mul(2),
            Easing::OutSine,
        )?;
    }

    Ok(())
}

pub fn animate_glance_focus<const TRACKS: usize, const BINDINGS: usize>(
    animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
    focused: WidgetId,
    neighbors: &[WidgetId],
    base_x: i32,
    base_y: i32,
    spec: GlanceTileSpec,
) -> Result<(), WidgetAnimationError> {
    animator.preset_selection_bump_settle(focused, base_y, spec.focus_bump_px, spec.focus_duration_ms)?;
    animator.animate_widget_x(
        focused,
        base_x,
        base_x.saturating_add(spec.focus_slide_px),
        spec.focus_duration_ms,
        Easing::OutSine,
    )?;
    animator.animate_opacity(focused, 200, 255, spec.focus_duration_ms, Easing::OutSine)?;

    for neighbor in neighbors.iter().copied() {
        animator.animate_widget_x(
            neighbor,
            base_x,
            base_x.saturating_sub((spec.focus_slide_px / 2).max(1)),
            spec.focus_duration_ms,
            Easing::OutSine,
        )?;
        animator.animate_opacity(neighbor, 255, spec.dim_opacity, spec.focus_duration_ms, Easing::OutSine)?;
    }
    Ok(())
}

pub fn apply_carddeck_visibility<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>(
    gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    cards: &[WidgetId],
    active: usize,
) -> Result<(), GuiError> {
    for (idx, id) in cards.iter().copied().enumerate() {
        gui.set_hidden(id, idx != active)?;
    }
    Ok(())
}

pub fn setup_peek_timeline<const TRACKS: usize, const BINDINGS: usize>(
    animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
    peek_widget: WidgetId,
    title_widget: Option<WidgetId>,
    subtitle_widget: Option<WidgetId>,
    base_x: i32,
    base_y: i32,
) -> Result<(), WidgetAnimationError> {
    animate_peek_reveal(
        animator,
        peek_widget,
        title_widget,
        subtitle_widget,
        base_x,
        base_y,
        PeekRevealSpec::default(),
    )
}

pub fn setup_launcher_glance<const TRACKS: usize, const BINDINGS: usize>(
    animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
    focused: WidgetId,
    neighbors: &[WidgetId],
    base_x: i32,
    base_y: i32,
) -> Result<(), WidgetAnimationError> {
    animate_glance_focus(
        animator,
        focused,
        neighbors,
        base_x,
        base_y,
        GlanceTileSpec::default(),
    )
}

pub fn setup_card_story<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>(
    gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    cards: &[WidgetId],
    state: &CardDeckState,
) -> Result<(), GuiError> {
    if state.is_empty() {
        return Ok(());
    }
    apply_carddeck_visibility(gui, cards, state.current())
}
