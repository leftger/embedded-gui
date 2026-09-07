//! Coverage tests for the motion/animation layer.
//!
//! The motion modules were the largest 0%-coverage block in the embedded-gui
//! workspace. These tests drive transitions, timelines, widget animations, and
//! cinematic helpers through their public APIs.

use embedded_graphics_core::pixelcolor::{Rgb565, WebColors};
use embedded_gui::motion::{
    ActiveScreenTransition, AnimatedProperty, Animation, AnimationConflictPolicy, AnimationGroup,
    AnimationSequence, CardDeckDirection, CardDeckState, CardStory, CinematicPreset,
    ComposedAnimation, ComposedAnimationCallbacks, ComposedAnimationPlayer, CompositionControls,
    CompositionMode, Easing, Keyframe, KeyframeTrack, KeyframeTrackCallbacks, PathPoint,
    PeekRevealSpec, ScreenTransitionEffect, ScreenTransitionOrigin, ScreenTransitionRunner,
    ScreenTransitionSample, ScreenTransitionSpec, SequencePlayer, SequenceRepeatMode,
    TimelineMotionPreset, TransitionPreset, WidgetAnimator, WidgetKeyframeState,
    WidgetPropertyKeyframe, animate_peek_reveal,
};
use embedded_gui::prelude::*;
use embedded_gui::widget::WidgetId;

#[test]
fn transition_spec_presets_and_all_samples() {
    for preset in [
        TransitionPreset::None,
        TransitionPreset::WindowPush,
        TransitionPreset::WindowPop,
        TransitionPreset::WindowPushRound,
        TransitionPreset::WindowPopRound,
        TransitionPreset::ShutterUp,
        TransitionPreset::ShutterDown,
        TransitionPreset::ShutterLeft,
        TransitionPreset::ShutterRight,
        TransitionPreset::RoundFlipToLauncher,
        TransitionPreset::RoundFlipFromLauncher,
        TransitionPreset::PortHoleUp,
        TransitionPreset::PortHoleDown,
        TransitionPreset::PortHoleLeft,
        TransitionPreset::PortHoleRight,
        TransitionPreset::ModalPresent,
        TransitionPreset::ModalDismiss,
        TransitionPreset::TimelineSlide,
        TransitionPreset::TimelinePeekIn,
        TransitionPreset::TimelinePeekOut,
        TransitionPreset::TimelinePinExpand,
        TransitionPreset::TimelineScrubSettle,
        TransitionPreset::Fade,
    ] {
        let spec = preset.spec();
        let _ = spec.duration_ms;
        let _ = preset.effect();
    }

    let specs = [
        ScreenTransitionSpec::none(),
        ScreenTransitionSpec::fade(100),
        ScreenTransitionSpec::slide_left(100),
        ScreenTransitionSpec::slide_right(100),
        ScreenTransitionSpec::slide_up(100),
        ScreenTransitionSpec::slide_down(100),
        ScreenTransitionSpec::push_moook(100),
        ScreenTransitionSpec::pop_moook(100),
        ScreenTransitionSpec::shutter_left(100),
        ScreenTransitionSpec::shutter_right(100),
        ScreenTransitionSpec::shutter_up(100),
        ScreenTransitionSpec::shutter_down(100),
        ScreenTransitionSpec::round_flip_left(100),
        ScreenTransitionSpec::round_flip_right(100),
        ScreenTransitionSpec::port_hole_left(100),
        ScreenTransitionSpec::port_hole_right(100),
        ScreenTransitionSpec::port_hole_up(100),
        ScreenTransitionSpec::port_hole_down(100),
        ScreenTransitionSpec::modal_slide_up(100),
        ScreenTransitionSpec::modal_slide_down(100),
        ScreenTransitionSpec::zoom(100),
        ScreenTransitionSpec::circular_reveal(100),
        ScreenTransitionSpec::wipe_left(100),
        ScreenTransitionSpec::wipe_right(100),
        ScreenTransitionSpec::wipe_up(100),
        ScreenTransitionSpec::wipe_down(100),
    ];
    for mut spec in specs {
        spec = spec
            .with_origin(ScreenTransitionOrigin::TopRight)
            .with_easing(Easing::OutBounce);
        assert!(spec.duration_ms > 0 || spec.effect == ScreenTransitionEffect::None);
    }

    let effects = [
        ScreenTransitionEffect::None,
        ScreenTransitionEffect::Fade,
        ScreenTransitionEffect::SlideLeft,
        ScreenTransitionEffect::SlideRight,
        ScreenTransitionEffect::SlideUp,
        ScreenTransitionEffect::SlideDown,
        ScreenTransitionEffect::PushMoook,
        ScreenTransitionEffect::PopMoook,
        ScreenTransitionEffect::Zoom,
        ScreenTransitionEffect::CircularReveal,
        ScreenTransitionEffect::WipeLeft,
        ScreenTransitionEffect::WipeRight,
        ScreenTransitionEffect::WipeUp,
        ScreenTransitionEffect::WipeDown,
        ScreenTransitionEffect::ShutterLeft,
        ScreenTransitionEffect::ShutterRight,
        ScreenTransitionEffect::ShutterUp,
        ScreenTransitionEffect::ShutterDown,
        ScreenTransitionEffect::RoundFlipLeft,
        ScreenTransitionEffect::RoundFlipRight,
        ScreenTransitionEffect::PortHoleLeft,
        ScreenTransitionEffect::PortHoleRight,
        ScreenTransitionEffect::PortHoleUp,
        ScreenTransitionEffect::PortHoleDown,
        ScreenTransitionEffect::ModalSlideUp,
        ScreenTransitionEffect::ModalSlideDown,
    ];
    for effect in effects {
        for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let active = ActiveScreenTransition {
                from: None,
                to: None,
                effect,
                origin: ScreenTransitionOrigin::BottomLeft,
                progress,
            };
            let _ = active.opacity_u8();
            let _ = active.slide_offset_x(120);
            let _ = active.slide_offset_y(80);
            let _: ScreenTransitionSample = active.sample(120, 80);
        }
    }
}

#[test]
fn screen_transition_runner_apply_and_tick() {
    let mut runner = ScreenTransitionRunner::new();
    assert_eq!(runner.active(), None);
    runner.tick(16);
    // A fresh runner without an applied transition stays inactive.
    assert_eq!(runner.active(), None);
}

#[test]
fn timeline_keyframe_sequence_group_and_composition() {
    let mut track = KeyframeTrack::<4>::new();
    track
        .push(Keyframe {
            value: 0.0,
            duration_ms: 10,
            easing: Easing::Linear,
        })
        .unwrap();
    track
        .push(Keyframe {
            value: 1.0,
            duration_ms: 10,
            easing: Easing::OutCubic,
        })
        .unwrap();
    track.set_callbacks(KeyframeTrackCallbacks {
        on_segment_start: Some(|_, _, _| {}),
        on_segment_complete: Some(|_, _| {}),
    });
    track.reset(0.0);
    track.tick(5).unwrap();
    assert!(track.value().is_some());
    track.tick(30).unwrap();
    track.tick(30).unwrap();
    assert!(track.is_done());
    assert_eq!(track.value(), Some(1.0));

    let mut sequence = AnimationSequence::<4>::new();
    sequence.push_delay(2).unwrap();
    sequence
        .push_animation(Animation::new(0.0, 1.0, 10, Easing::Linear))
        .unwrap();
    sequence.push_label(9).unwrap();
    assert_eq!(sequence.find_label(9), Some(2));
    let mut player = SequencePlayer::<2, 4>::new(sequence);
    player.set_repeat_mode(SequenceRepeatMode::Loop);
    assert!(!player.is_done());
    player.tick(50).unwrap();
    player.tick(50).unwrap();
    let _ = player.active_value();

    let mut seq2 = AnimationSequence::<2>::new();
    seq2.push_label(1).unwrap();
    let mut p2 = SequencePlayer::<2, 2>::new(seq2);
    p2.seek_to_label(1).unwrap();
    assert!(p2.seek_to_label(99).is_err());

    let mut group = AnimationGroup::<2>::new();
    group
        .push(Animation::new(0.0, 1.0, 5, Easing::Linear))
        .unwrap();
    assert!(
        group
            .push(Animation::new(0.0, 1.0, 5, Easing::Linear))
            .is_ok()
    );

    let mut composition = ComposedAnimation::<2>::new(CompositionMode::Sequence);
    composition
        .push(Animation::new(0.0, 1.0, 10, Easing::Linear))
        .unwrap();
    composition = composition.with_controls(CompositionControls {
        start_delay_ms: 0,
        repeat_count: Some(1),
        reverse: true,
    });
    let mut composed = ComposedAnimationPlayer::<2, 2>::new(composition);
    composed.set_callbacks(ComposedAnimationCallbacks {
        on_cycle_start: None,
        on_cycle_complete: None,
        on_done: None,
    });
    composed.set_paused(true);
    composed.tick(5).unwrap();
    composed.set_paused(false);
    composed.tick(100).unwrap();
    composed.tick(100).unwrap();
    assert!(composed.status().done);
    composed.restart();
    composed.stop();
    assert!(composed.status().done);
}

#[test]
fn widget_animator_binding_family_without_gui() {
    let mut animator = WidgetAnimator::<32, 32>::new();
    let id = WidgetId::new(1);
    animator
        .animate_progress(id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_meter(id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_slider_value(id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_scroll_offset_y(id, 0, 100, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_tab_selected(id, 0, 1, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_dropdown_selected(id, 0, 1, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_roller_selected(id, 0, 1, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_gauge_value(id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_spinner_phase(id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_x(id, 0, 10, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_y(id, 0, 10, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_width(id, 10, 20, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_height(id, 10, 20, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_opacity(id, 0, 255, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_opacity_with_custom_interpolator(
            id,
            0,
            255,
            20,
            Easing::Linear,
            |t, _, _| t,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_widget_keyframes(
            id,
            WidgetKeyframeState {
                x: 0,
                y: 0,
                opacity: 255,
            },
            &[WidgetPropertyKeyframe {
                x: Some(10),
                y: Some(10),
                opacity: Some(100),
                duration_ms: 10,
                easing: Easing::Linear,
            }],
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .pulse_opacity(id, 10, 20, 20, Easing::Linear)
        .unwrap();
    animator
        .ping_pong_progress(id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_corner_radius(id, 0, 8, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_accent_color(id, Rgb565::CSS_RED, Rgb565::CSS_BLUE, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_path(
            id,
            &[PathPoint::new(0.0, 0.0), PathPoint::new(10.0, 10.0)],
            20,
            Easing::Linear,
        )
        .unwrap();

    assert!(animator.is_animating_widget(id));
    assert!(animator.is_animating_widget_property(id, AnimatedProperty::Progress));
    assert!(animator.active_count() > 0);
    let stopped = animator.stop_widget(id);
    assert!(stopped > 0);
    assert_eq!(animator.active_count(), 0);
}

#[test]
fn widget_animator_tick_updates_real_widgets() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 128, 64));
    let progress_id = gui
        .add_progress_bar(Rect::new(0, 0, 64, 10), 0.0, Style::progress())
        .unwrap();
    let slider_id = gui
        .add_slider(Rect::new(0, 20, 64, 10), 0.0, 0.0, 1.0, Style::panel())
        .unwrap();

    let mut animator = WidgetAnimator::<8, 8>::new();
    animator
        .animate_progress(progress_id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_slider_value(slider_id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();

    animator.tick(10, &mut gui).unwrap();
    animator.tick(20, &mut gui).unwrap();
    assert_eq!(animator.active_count(), 0);
    assert!((gui.slider_value(slider_id).unwrap() - 1.0).abs() < 1e-5);
}

#[test]
fn cinematic_deck_story_tokens_and_peek_animation() {
    let mut deck = CardDeckState::new(3);
    assert_eq!(deck.current(), 0);
    assert_eq!(deck.move_next(), Some(CardDeckDirection::Forward));
    assert_eq!(deck.move_prev(), Some(CardDeckDirection::Backward));
    assert!(deck.move_prev().is_none());
    deck.set_len(0);
    assert!(deck.is_empty());

    let cards = [WidgetId::new(1), WidgetId::new(2), WidgetId::new(3)];
    let mut story = CardStory::new(&cards, TimelineMotionPreset::PeekIn).with_slide_px(20);
    assert_eq!(story.state().len(), 3);
    assert_eq!(story.current_widget(), Some(WidgetId::new(1)));
    let t = story.next().unwrap();
    assert_eq!(t.from, WidgetId::new(1));
    assert_eq!(t.to, WidgetId::new(2));
    let _ = story.prev().unwrap();

    let mut animator = WidgetAnimator::<16, 16>::new();
    let _ = t.animate(&mut animator, 50);
    let spec = PeekRevealSpec {
        dot_px: 2,
        icon_expand_px: 16,
        icon_duration_ms: 10,
        text_stagger_ms: 5,
        text_duration_ms: 10,
    };
    animate_peek_reveal(
        &mut animator,
        WidgetId::new(1),
        Some(WidgetId::new(2)),
        Some(WidgetId::new(3)),
        0,
        0,
        spec,
    )
    .unwrap();

    assert_eq!(CinematicPreset::PeekTimeline.name(), "peek-timeline");
    assert_eq!(CinematicPreset::LauncherGlance.name(), "launcher-glance");
    assert_eq!(CinematicPreset::CardStory.name(), "card-story");
    assert_eq!(TimelineMotionPreset::PeekIn.duration_ms(), 220);
    assert_eq!(TimelineMotionPreset::ScrubSettle.duration_ms(), 140);
}

#[test]
fn widget_animator_extended_policies_presets_and_stops() {
    let mut animator = WidgetAnimator::<64, 64>::new();
    let id = WidgetId::new(11);
    let id2 = WidgetId::new(12);

    animator
        .animate_slider_value_with_policy(
            id,
            0.0,
            1.0,
            20,
            Easing::Linear,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_scroll_offset_y_with_policy(
            id,
            0,
            10,
            20,
            Easing::Linear,
            AnimationConflictPolicy::Queue,
        )
        .unwrap();
    animator
        .animate_tab_selected(id, 0, 1, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_dropdown_selected(id, 0, 1, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_roller_selected(id, 0, 1, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_gauge_value(id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_spinner_phase(id, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();

    animator
        .animate_widget_x_with_policy(
            id,
            0,
            5,
            10,
            Easing::Linear,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_widget_x_with_custom_interpolator(
            id,
            0,
            5,
            10,
            Easing::Linear,
            |t, _, _| t,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_widget_y_with_policy(
            id,
            0,
            5,
            10,
            Easing::Linear,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_widget_y_with_custom_curve(
            id,
            0,
            5,
            10,
            Easing::Linear,
            |t| t,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_widget_width_with_policy(
            id,
            1,
            2,
            10,
            Easing::Linear,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_widget_height_with_policy(
            id,
            1,
            2,
            10,
            Easing::Linear,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_opacity_with_policy(
            id,
            0,
            255,
            10,
            Easing::Linear,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .bind_property(
            id,
            AnimatedProperty::CornerRadius,
            Animation::new(0.0, 4.0, 10, Easing::Linear),
        )
        .unwrap();
    animator
        .bind_property_with_policy(
            id,
            AnimatedProperty::CornerRadius,
            Animation::new(4.0, 8.0, 10, Easing::Linear),
            AnimationConflictPolicy::Queue,
        )
        .unwrap();

    animator
        .stagger_widget_x(&[id, id2], 0, 10, 20, 5, Easing::Linear)
        .unwrap();
    animator.preset_fade_in_up(id, 20, 0, 10).unwrap();
    animator.preset_attention_shake(id, 10, 3, 30).unwrap();
    animator
        .preset_selection_bump_settle(id, 20, 4, 30)
        .unwrap();

    assert!(animator.is_animating_widget(id));
    let stopped = animator.stop_widget_property(id, AnimatedProperty::WidgetX);
    assert!(stopped > 0);
    assert!(animator.active_count() > 0);
    let _ = animator.stop_widget(id2);

    // Test animator tick with real widgets for scroll/corner/opacity properties.
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 128, 64));
    let label_id = gui
        .add_label(Rect::new(0, 0, 20, 10), "x", Style::label())
        .unwrap();
    let mut real = WidgetAnimator::<8, 8>::new();
    real.animate_widget_x(label_id, 0, 10, 5, Easing::Linear)
        .unwrap();
    real.animate_opacity(label_id, 0, 255, 5, Easing::Linear)
        .unwrap();
    real.tick(10, &mut gui).unwrap();
    assert_eq!(real.active_count(), 0);
}

#[test]
fn widget_animator_tick_all_property_kinds() {
    static ITEMS: [&str; 3] = ["A", "B", "C"];
    let mut gui = GuiContext::<32, 16, 16>::new(Rect::new(0, 0, 200, 200));
    let progress = gui
        .add_progress_bar(Rect::new(0, 0, 20, 5), 0.0, Style::progress())
        .unwrap();
    let meter = gui
        .add_meter(Rect::new(0, 10, 20, 5), 0.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let slider = gui
        .add_slider(Rect::new(0, 20, 20, 5), 0.0, 0.0, 1.0, Style::panel())
        .unwrap();
    let scroll = gui
        .add_scroll_view(Rect::new(0, 30, 20, 5), 0, 100, Style::panel())
        .unwrap();
    let tabs = gui
        .add_tabs(Rect::new(0, 40, 30, 5), &ITEMS, 0, Style::panel())
        .unwrap();
    let dropdown = gui
        .add_dropdown(Rect::new(0, 50, 30, 5), &ITEMS, 0, Style::panel())
        .unwrap();
    let roller = gui
        .add_roller(Rect::new(0, 60, 30, 5), &ITEMS, 0, Style::panel())
        .unwrap();
    let gauge = gui
        .add_gauge(Rect::new(0, 70, 20, 20), 0.0, 0.0, 10.0, Style::panel())
        .unwrap();
    let spinner = gui
        .add_spinner(Rect::new(0, 95, 10, 10), 0.0, Style::panel())
        .unwrap();
    let label = gui
        .add_label(Rect::new(0, 110, 20, 5), "x", Style::label())
        .unwrap();

    let mut animator = WidgetAnimator::<32, 32>::new();
    animator
        .animate_progress(progress, 0.0, 1.0, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_meter(meter, 0.0, 10.0, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_slider_value(slider, 0.0, 1.0, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_scroll_offset_y(scroll, 0, 50, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_tab_selected(tabs, 0, 1, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_dropdown_selected(dropdown, 0, 1, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_roller_selected(roller, 0, 1, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_gauge_value(gauge, 0.0, 10.0, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_spinner_phase(spinner, 0.0, 1.0, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_x(label, 0, 10, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_y(label, 0, 10, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_width(label, 20, 30, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_height(label, 5, 10, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_opacity(label, 0, 255, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_corner_radius(label, 0, 4, 1, Easing::Linear)
        .unwrap();
    animator
        .animate_accent_color(label, Rgb565::CSS_RED, Rgb565::CSS_BLUE, 1, Easing::Linear)
        .unwrap();

    animator.tick(5, &mut gui).unwrap();
    assert_eq!(animator.active_count(), 0);
}
