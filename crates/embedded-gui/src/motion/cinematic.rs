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
pub struct MotionTokens {
    pub peek_dot_px: u32,
    pub peek_icon_expand_px: u32,
    pub peek_icon_duration_ms: u32,
    pub peek_text_stagger_ms: u32,
    pub peek_text_duration_ms: u32,
    pub glance_focus_bump_px: i32,
    pub glance_focus_slide_px: i32,
    pub glance_focus_duration_ms: u32,
    pub glance_dim_opacity: u8,
}

impl Default for MotionTokens {
    fn default() -> Self {
        Self {
            peek_dot_px: 3,
            peek_icon_expand_px: 24,
            peek_icon_duration_ms: 300,
            peek_text_stagger_ms: 90,
            peek_text_duration_ms: 160,
            glance_focus_bump_px: 3,
            glance_focus_slide_px: 6,
            glance_focus_duration_ms: 120,
            glance_dim_opacity: 170,
        }
    }
}

impl MotionTokens {
    pub const fn to_peek_spec(self) -> PeekRevealSpec {
        PeekRevealSpec {
            dot_px: self.peek_dot_px,
            icon_expand_px: self.peek_icon_expand_px,
            icon_duration_ms: self.peek_icon_duration_ms,
            text_stagger_ms: self.peek_text_stagger_ms,
            text_duration_ms: self.peek_text_duration_ms,
        }
    }

    pub const fn to_glance_spec(self) -> GlanceTileSpec {
        GlanceTileSpec {
            focus_bump_px: self.glance_focus_bump_px,
            focus_slide_px: self.glance_focus_slide_px,
            focus_duration_ms: self.glance_focus_duration_ms,
            dim_opacity: self.glance_dim_opacity,
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
pub struct CardStory<'a> {
    cards: &'a [WidgetId],
    state: CardDeckState,
    transition: TimelineMotionPreset,
    slide_px: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardStoryTransition {
    pub from: WidgetId,
    pub to: WidgetId,
    pub direction: CardDeckDirection,
    pub preset: TimelineMotionPreset,
    pub slide_px: i32,
}

impl<'a> CardStory<'a> {
    pub fn new(cards: &'a [WidgetId], transition: TimelineMotionPreset) -> Self {
        Self {
            cards,
            state: CardDeckState::new(cards.len()),
            transition,
            slide_px: 14,
        }
    }

    pub fn with_slide_px(mut self, slide_px: i32) -> Self {
        self.slide_px = slide_px.max(1);
        self
    }

    pub const fn state(&self) -> &CardDeckState {
        &self.state
    }

    pub fn current_widget(&self) -> Option<WidgetId> {
        self.cards.get(self.state.current()).copied()
    }

    pub fn apply<'g, const NODES: usize, const EVENTS: usize, const DIRTY: usize>(
        &self,
        gui: &mut GuiContext<'g, NODES, EVENTS, DIRTY>,
    ) -> Result<(), GuiError> {
        apply_carddeck_visibility(gui, self.cards, self.state.current())
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<CardStoryTransition> {
        let from_idx = self.state.current();
        self.state.move_next()?;
        let to_idx = self.state.current();
        Some(CardStoryTransition {
            from: self.cards[from_idx],
            to: self.cards[to_idx],
            direction: CardDeckDirection::Forward,
            preset: self.transition,
            slide_px: self.slide_px,
        })
    }

    pub fn prev(&mut self) -> Option<CardStoryTransition> {
        let from_idx = self.state.current();
        self.state.move_prev()?;
        let to_idx = self.state.current();
        Some(CardStoryTransition {
            from: self.cards[from_idx],
            to: self.cards[to_idx],
            direction: CardDeckDirection::Backward,
            preset: self.transition,
            slide_px: self.slide_px,
        })
    }

    pub fn jump_to(&mut self, index: usize) -> Option<CardStoryTransition> {
        if self.cards.is_empty() {
            return None;
        }
        let clamped = index.min(self.cards.len() - 1);
        let from_idx = self.state.current();
        if clamped == from_idx {
            return None;
        }
        let direction = if clamped > from_idx {
            CardDeckDirection::Forward
        } else {
            CardDeckDirection::Backward
        };
        self.state.current = clamped;
        Some(CardStoryTransition {
            from: self.cards[from_idx],
            to: self.cards[clamped],
            direction,
            preset: self.transition,
            slide_px: self.slide_px,
        })
    }
}

impl CardStoryTransition {
    pub fn animate<const TRACKS: usize, const BINDINGS: usize>(
        self,
        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
        base_x: i32,
    ) -> Result<(), WidgetAnimationError> {
        let duration = self.preset.duration_ms();
        let easing = self.preset.easing();
        let delta = match self.direction {
            CardDeckDirection::Forward => self.slide_px,
            CardDeckDirection::Backward => -self.slide_px,
        };
        animator.animate_widget_x(self.from, base_x, base_x - delta, duration, easing)?;
        animator.animate_opacity(self.from, 255, 90, duration, Easing::OutSine)?;
        animator.animate_widget_x(self.to, base_x + delta, base_x, duration, easing)?;
        animator.animate_opacity(self.to, 90, 255, duration, Easing::OutSine)?;
        Ok(())
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
    animator.animate_widget_width(
        icon_widget,
        dot,
        spec.icon_expand_px.max(dot),
        spec.icon_duration_ms,
        Easing::OutBack,
    )?;
    animator.animate_widget_height(
        icon_widget,
        dot,
        spec.icon_expand_px.max(dot),
        spec.icon_duration_ms,
        Easing::OutBack,
    )?;
    animator.animate_opacity(
        icon_widget,
        180,
        255,
        spec.icon_duration_ms,
        Easing::OutSine,
    )?;

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
        animator.animate_opacity(
            title,
            0,
            255,
            spec.text_duration_ms + spec.text_stagger_ms,
            Easing::OutSine,
        )?;
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
    animator.preset_selection_bump_settle(
        focused,
        base_y,
        spec.focus_bump_px,
        spec.focus_duration_ms,
    )?;
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
        animator.animate_opacity(
            neighbor,
            255,
            spec.dim_opacity,
            spec.focus_duration_ms,
            Easing::OutSine,
        )?;
    }
    Ok(())
}

pub fn apply_carddeck_visibility<
    'a,
    const NODES: usize,
    const EVENTS: usize,
    const DIRTY: usize,
>(
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
    setup_peek_timeline_with_tokens(
        animator,
        peek_widget,
        title_widget,
        subtitle_widget,
        base_x,
        base_y,
        MotionTokens::default(),
    )
}

pub fn setup_peek_timeline_with_tokens<const TRACKS: usize, const BINDINGS: usize>(
    animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
    peek_widget: WidgetId,
    title_widget: Option<WidgetId>,
    subtitle_widget: Option<WidgetId>,
    base_x: i32,
    base_y: i32,
    tokens: MotionTokens,
) -> Result<(), WidgetAnimationError> {
    animate_peek_reveal(
        animator,
        peek_widget,
        title_widget,
        subtitle_widget,
        base_x,
        base_y,
        tokens.to_peek_spec(),
    )
}

pub fn setup_launcher_glance<const TRACKS: usize, const BINDINGS: usize>(
    animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
    focused: WidgetId,
    neighbors: &[WidgetId],
    base_x: i32,
    base_y: i32,
) -> Result<(), WidgetAnimationError> {
    setup_launcher_glance_with_tokens(
        animator,
        focused,
        neighbors,
        base_x,
        base_y,
        MotionTokens::default(),
    )
}

pub fn setup_launcher_glance_with_tokens<const TRACKS: usize, const BINDINGS: usize>(
    animator: &mut WidgetAnimator<TRACKS, BINDINGS>,
    focused: WidgetId,
    neighbors: &[WidgetId],
    base_x: i32,
    base_y: i32,
    tokens: MotionTokens,
) -> Result<(), WidgetAnimationError> {
    animate_glance_focus(
        animator,
        focused,
        neighbors,
        base_x,
        base_y,
        tokens.to_glance_spec(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_tokens_and_card_deck_state() {
        let peek = PeekRevealSpec::default();
        assert_eq!(peek.dot_px, 3);
        let glance = GlanceTileSpec::default();
        assert_eq!(glance.focus_duration_ms, 120);

        let tokens = MotionTokens::default();
        assert_eq!(tokens.to_peek_spec(), peek);
        assert_eq!(tokens.to_glance_spec(), glance);

        let mut deck = CardDeckState::new(3);
        assert_eq!(deck.current(), 0);
        assert_eq!(deck.len(), 3);
        assert!(!deck.is_empty());
        assert_eq!(deck.move_next(), Some(CardDeckDirection::Forward));
        assert_eq!(deck.move_next(), Some(CardDeckDirection::Forward));
        assert_eq!(deck.move_next(), None);
        assert_eq!(deck.move_prev(), Some(CardDeckDirection::Backward));
        assert_eq!(deck.move_prev(), Some(CardDeckDirection::Backward));
        assert_eq!(deck.move_prev(), None);

        deck.set_len(0);
        assert!(deck.is_empty());
        deck.set_len(2);
        assert_eq!(deck.current(), 0);
    }

    #[test]
    fn card_story_transitions_and_presets() {
        let cards = [WidgetId::new(1), WidgetId::new(2), WidgetId::new(3)];
        let mut story = CardStory::new(&cards, TimelineMotionPreset::PeekIn).with_slide_px(24);
        assert_eq!(story.current_widget(), Some(WidgetId::new(1)));
        assert_eq!(story.state().current(), 0);

        let next = story.next().unwrap();
        assert_eq!(next.from, WidgetId::new(1));
        assert_eq!(next.to, WidgetId::new(2));
        assert_eq!(next.direction, CardDeckDirection::Forward);
        assert_eq!(next.slide_px, 24);

        let prev = story.prev().unwrap();
        assert_eq!(prev.from, WidgetId::new(2));
        assert_eq!(prev.to, WidgetId::new(1));

        let jump = story.jump_to(2).unwrap();
        assert_eq!(jump.to, WidgetId::new(3));
        assert!(story.jump_to(2).is_none());
        assert!(story.jump_to(0).is_some());
        assert!(story.jump_to(99).is_some());

        let mut empty = CardStory::new(&[], TimelineMotionPreset::PeekOut);
        assert!(empty.next().is_none());
        assert!(empty.prev().is_none());
        assert!(empty.jump_to(0).is_none());

        assert_eq!(TimelineMotionPreset::PeekIn.duration_ms(), 220);
        assert_eq!(TimelineMotionPreset::PeekOut.duration_ms(), 220);
        assert_eq!(TimelineMotionPreset::PinExpand.duration_ms(), 260);
        assert_eq!(TimelineMotionPreset::ScrubSettle.duration_ms(), 140);

        let _ = TimelineMotionPreset::PeekIn.easing();
        let _ = TimelineMotionPreset::PeekOut.easing();
        let _ = TimelineMotionPreset::PinExpand.easing();
        let _ = TimelineMotionPreset::ScrubSettle.easing();

        assert_eq!(CinematicPreset::PeekTimeline.name(), "peek-timeline");
        assert_eq!(CinematicPreset::LauncherGlance.name(), "launcher-glance");
        assert_eq!(CinematicPreset::CardStory.name(), "card-story");
    }

    #[test]
    fn cinematic_setup_helpers_and_story_apply() {
        let mut gui: GuiContext<12, 8, 8> =
            GuiContext::new(crate::geometry::Rect::new(0, 0, 200, 100));
        let id1 = gui
            .add_label(
                crate::geometry::Rect::new(0, 0, 20, 10),
                "1",
                crate::style::Style::label(),
            )
            .unwrap();
        let id2 = gui
            .add_label(
                crate::geometry::Rect::new(0, 20, 20, 10),
                "2",
                crate::style::Style::label(),
            )
            .unwrap();
        let id3 = gui
            .add_label(
                crate::geometry::Rect::new(0, 40, 20, 10),
                "3",
                crate::style::Style::label(),
            )
            .unwrap();
        let cards = [id1, id2, id3];
        let mut deck = CardDeckState::new(3);
        setup_card_story(&mut gui, &cards, &deck).unwrap();
        deck.move_next();
        setup_card_story(&mut gui, &cards, &deck).unwrap();

        let mut story = CardStory::new(&cards, TimelineMotionPreset::PeekIn);
        story.next();
        story.apply(&mut gui).unwrap();

        let mut animator = WidgetAnimator::<32, 32>::new();
        animate_glance_focus(
            &mut animator,
            id1,
            &[id2, id3],
            10,
            20,
            GlanceTileSpec::default(),
        )
        .unwrap();
        setup_peek_timeline(&mut animator, id1, Some(id2), Some(id3), 10, 20).unwrap();
        setup_peek_timeline_with_tokens(
            &mut animator,
            id1,
            Some(id2),
            None,
            10,
            20,
            MotionTokens::default(),
        )
        .unwrap();
        setup_launcher_glance(&mut animator, id1, &[id2, id3], 10, 20).unwrap();
        setup_launcher_glance_with_tokens(
            &mut animator,
            id1,
            &[id2],
            10,
            20,
            MotionTokens::default(),
        )
        .unwrap();
        assert!(animator.active_count() > 0);
    }
}
