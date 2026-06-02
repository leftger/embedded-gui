#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod animation;
pub mod animation_timeline;
pub mod animation_timing;
pub mod transition_preset;
pub mod block;
pub mod context;
pub mod font;
pub mod geometry;
pub mod image;
pub mod input;
pub mod layout;
mod math;
pub mod present;
pub mod render;
pub mod screen;
pub mod screen_transition;
pub mod state;
pub mod style;
#[cfg(feature = "std")]
pub mod test_buffer;
pub mod text;
pub mod widget;
pub mod widget_animation;
pub mod widgets;

pub use animation::{
    Animation, AnimationError, AnimationHandlers, AnimationId, AnimationManager,
    AnimationManagerCallbacks, AnimationState, Easing, InertiaAnimator, PathAnimator, PathPoint,
    RepeatMode, SpringAnimator, Timer, Tween, apply_easing,
};
pub use animation_timing::{
    DEFAULT_DURATION_MS, FRAME_INTERVAL_MS, MOOOK_DURATION_MS, NORMALIZED_MAX,
    PORT_HOLE_DURATION_MS, SHUTTER_DURATION_MS, interpolate_moook, moook_curve, moook_duration_ms,
    timing_half_phase,
    timing_scaled, timing_shutter_phase,
};
pub use transition_preset::TransitionPreset;
pub use animation_timeline::{
    AnimationGroup, AnimationSequence, ComposedAnimation, ComposedAnimationCallbacks,
    ComposedAnimationPlayer, ComposedAnimationStatus, CompositionControls, CompositionMode,
    Keyframe, KeyframeTrack, KeyframeTrackCallbacks, SequencePlayer, SequencePlayerStatus,
    SequenceRepeatMode, TimelineError, TimelineStep,
};
pub use block::Block;
pub use context::{
    GuiContext, GuiError, KeyBindingAction, PressTiming, WidgetKeyBindings, WidgetKeyInputPolicy,
};
pub use font::FontId;
pub use geometry::{DirtyTracker, EdgeInsets, Rect};
#[cfg(all(feature = "std", feature = "image-decode"))]
pub use image::{
    BasicImageDecoder, EncodedImageFormat, ImageDecodeError, ImageDecoder, decode_image_auto,
    decode_image_with, decode_ppm_ascii,
};
pub use image::{ImageAtlas, ImageAtlasEntry, ImageFit, ImageRef, SpriteSheet};
pub use input::{
    EventPhaseMask, InputEvent, PointerButton, PointerState, UiEvent, UiEventFilter,
    WidgetDispatchPolicy, WidgetEvent, WidgetEventFilter, WidgetEventKind,
};
pub use layout::{Align, Axis, Constraint, LayoutItem, Length, LinearLayout};
pub use present::PresentRegion;
pub use render::{
    AntiAliasMode, BlendMode, CHAR_HEIGHT, CHAR_WIDTH, ColorFormat, EllipsisMode, LayerState,
    RenderBackendCaps, RenderCtx, RenderQuality, StrokeCap, StrokeJoin, StrokeStyle, TextAlign,
    TextMetrics, TextOverflow, TextOverflowPolicy, TextStyle, TextWrap, Transform2D, VerticalAlign,
};
pub use screen::{
    Screen, ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack, ScreenStackError,
    ScreenTransition,
};
pub use screen_transition::{
    ActiveScreenTransition, ScreenTransitionEffect, ScreenTransitionOrigin, ScreenTransitionRunner,
    ScreenTransitionSample, ScreenTransitionSpec, render_transition_pair,
};
pub use state::{ListState, ScrollState, SliderState, TabsState};
pub use style::{
    Border, GradientDirection, LinearGradient, Shadow, StateStyle, Style, StyleTransition, Theme,
    VisualState, WidgetStyle, lerp_style,
};
#[cfg(feature = "std")]
pub use test_buffer::{LayerCanvas, TestBuffer};
pub use text::{
    BasicTextShaper, Line, ShapedGlyph, ShapingConfig, Span, Text, TextDirection, TextShaper,
};
pub use widget::{
    EventContext, EventPhase, EventPolicy, FocusGroupId, StatefulWidget, StyleClassId, WidgetFlags,
    WidgetId,
};
pub use widget_animation::presets;
pub use widget_animation::{
    AnimatedProperty, AnimationConflictPolicy, BindingSnapshot, WidgetAnimationCallbacks,
    WidgetAnimationError, WidgetAnimator, WidgetKeyframeState, WidgetPropertyKeyframe,
};
pub use widgets::{ChartMode, KeyboardLayout, WidgetKind, WidgetNode};

pub mod prelude {
    pub use crate::{
        ActiveScreenTransition, Align, AnimatedProperty, Animation, AnimationConflictPolicy,
        AnimationError, AnimationGroup, AnimationHandlers, AnimationId, AnimationManager,
        AnimationManagerCallbacks, AnimationSequence, AnimationState, AntiAliasMode, Axis,
        BasicTextShaper, BindingSnapshot, BlendMode, Block, Border, ChartMode, ColorFormat,
        ComposedAnimation, ComposedAnimationCallbacks, ComposedAnimationPlayer,
        ComposedAnimationStatus, CompositionControls, CompositionMode, Constraint, DirtyTracker,
        Easing, EdgeInsets, EllipsisMode, EventContext, EventPhase, EventPhaseMask, EventPolicy,
        FocusGroupId, FontId, GradientDirection, GuiContext, GuiError, ImageAtlas, ImageAtlasEntry,
        ImageFit, ImageRef, InertiaAnimator, InputEvent, KeyBindingAction, KeyboardLayout,
        Keyframe, KeyframeTrack, KeyframeTrackCallbacks, LayerState, LayoutItem, Length, Line,
        LinearGradient, LinearLayout, ListState, PathAnimator, PathPoint, PointerButton,
        PointerState, PresentRegion, PressTiming, Rect, RenderBackendCaps, RenderCtx,
        RenderQuality, RepeatMode, Screen, ScreenCommand, ScreenId, ScreenLifecycleEvent,
        ScreenStack, ScreenStackError, ScreenTransition, ScreenTransitionEffect,
        TransitionPreset, ScreenTransitionOrigin, ScreenTransitionRunner, ScreenTransitionSample,
        ScreenTransitionSpec, ScrollState, SequencePlayer, SequencePlayerStatus,
        SequenceRepeatMode, Shadow, ShapedGlyph, ShapingConfig, SliderState, Span, SpringAnimator,
        SpriteSheet, StateStyle, StatefulWidget, StrokeCap, StrokeJoin, StrokeStyle, Style,
        StyleClassId, StyleTransition, TabsState, Text, TextAlign, TextDirection, TextMetrics,
        TextOverflow, TextOverflowPolicy, TextShaper, TextStyle, TextWrap, Theme, TimelineError,
        TimelineStep, Timer, Transform2D, Tween, UiEvent, UiEventFilter, VerticalAlign,
        VisualState, WidgetAnimationCallbacks, WidgetAnimationError, WidgetAnimator,
        WidgetDispatchPolicy, WidgetEvent, WidgetEventFilter, WidgetEventKind, WidgetFlags,
        WidgetId, WidgetKeyBindings, WidgetKeyInputPolicy, WidgetKeyframeState, WidgetKind,
        WidgetPropertyKeyframe, WidgetStyle, apply_easing, lerp_style, presets,
        render_transition_pair,
    };

    #[cfg(all(feature = "std", feature = "image-decode"))]
    pub use crate::{
        BasicImageDecoder, EncodedImageFormat, ImageDecodeError, ImageDecoder, LayerCanvas,
        TestBuffer, decode_image_auto, decode_image_with, decode_ppm_ascii,
    };

    #[cfg(all(feature = "std", not(feature = "image-decode")))]
    pub use crate::{LayerCanvas, TestBuffer};
}
