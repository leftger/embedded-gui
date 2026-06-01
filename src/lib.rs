#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod animation;
pub mod animation_timeline;
pub mod block;
pub mod context;
pub mod font;
pub mod geometry;
pub mod input;
pub mod image;
pub mod layout;
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
    Animation, AnimationError, AnimationHandlers, AnimationId, AnimationManager, AnimationManagerCallbacks, AnimationState, Easing,
    InertiaAnimator, PathAnimator, PathPoint, RepeatMode, SpringAnimator, Timer, Tween,
    apply_easing,
};
pub use animation_timeline::{
    AnimationGroup, AnimationSequence, Keyframe, KeyframeTrack, KeyframeTrackCallbacks,
    SequencePlayer, SequencePlayerStatus, SequenceRepeatMode, TimelineError, TimelineStep,
};
pub use block::Block;
pub use context::{
    GuiContext, GuiError, KeyBindingAction, PressTiming, WidgetKeyBindings, WidgetKeyInputPolicy,
};
pub use font::FontId;
pub use geometry::{DirtyTracker, EdgeInsets, Rect};
pub use input::{
    EventPhaseMask, InputEvent, PointerButton, PointerState, UiEvent, UiEventFilter,
    WidgetDispatchPolicy, WidgetEvent, WidgetEventFilter, WidgetEventKind,
};
pub use image::{ImageAtlas, ImageAtlasEntry, ImageFit, ImageRef, SpriteSheet};
#[cfg(all(feature = "std", feature = "image-decode"))]
pub use image::{
    BasicImageDecoder, EncodedImageFormat, ImageDecodeError, ImageDecoder, decode_image_auto,
    decode_image_with, decode_ppm_ascii,
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
    ActiveScreenTransition, ScreenTransitionEffect, ScreenTransitionRunner, ScreenTransitionSample,
    ScreenTransitionSpec, render_transition_pair,
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
pub use widget_animation::{
    AnimatedProperty, AnimationConflictPolicy, BindingSnapshot, WidgetAnimationCallbacks,
    WidgetAnimationError, WidgetAnimator,
};
pub use widget_animation::presets;
pub use widgets::{ChartMode, KeyboardLayout, WidgetKind, WidgetNode};

pub mod prelude {
    pub use crate::{
        Align, AnimatedProperty, Animation, AnimationError, AnimationGroup, AnimationId,
        AnimationHandlers, AnimationManager, AnimationManagerCallbacks, AnimationSequence, AnimationState, Axis, Block, Border, Constraint,
        DirtyTracker, Easing, EdgeInsets, EventContext, EventPhase, EventPhaseMask, EventPolicy,
        FocusGroupId, FontId, GradientDirection, GuiContext, GuiError, ImageAtlas,
        PressTiming, WidgetKeyInputPolicy, WidgetKeyBindings, KeyBindingAction,
        ImageAtlasEntry, ImageFit, ImageRef, InputEvent, Keyframe, KeyframeTrack,
        KeyframeTrackCallbacks, LayoutItem, Length, Line, LinearGradient, LinearLayout,
        ListState, PointerButton, PointerState, PresentRegion, Rect, RenderCtx, RenderQuality,
        RepeatMode, Screen, ScreenCommand, ScreenId,
        ScreenLifecycleEvent, ScreenStack, ScreenStackError, ScreenTransition,
        ScreenTransitionEffect, ScreenTransitionRunner, ScreenTransitionSpec,
        ScreenTransitionSample, ActiveScreenTransition, ScrollState, SequencePlayer, SpriteSheet,
        SequencePlayerStatus, SequenceRepeatMode, Shadow, SliderState, Span, StateStyle,
        StatefulWidget, Style, StyleTransition, SpringAnimator, InertiaAnimator, PathPoint,
        PathAnimator, lerp_style, render_transition_pair,
        StyleClassId, TabsState, Text, TextAlign, TextMetrics, TextStyle, TextWrap, Theme,
        TextOverflow, TextOverflowPolicy, EllipsisMode, StrokeStyle, StrokeCap, StrokeJoin,
        AntiAliasMode, BlendMode, ColorFormat, LayerState, RenderBackendCaps, Transform2D,
        TimelineError, TimelineStep, Timer, Tween, UiEvent, UiEventFilter, VerticalAlign,
        VisualState, WidgetAnimationCallbacks, WidgetAnimationError, WidgetAnimator,
        AnimationConflictPolicy, BindingSnapshot, WidgetDispatchPolicy, WidgetEvent,
        WidgetEventFilter, WidgetEventKind, WidgetFlags, WidgetId, WidgetKind, WidgetStyle, KeyboardLayout,
        ChartMode,
        presets,
        TextShaper, BasicTextShaper, ShapingConfig, ShapedGlyph, TextDirection,
        apply_easing,
    };

    #[cfg(all(feature = "std", feature = "image-decode"))]
    pub use crate::{
        BasicImageDecoder, EncodedImageFormat, ImageDecodeError, ImageDecoder, LayerCanvas,
        TestBuffer,
        decode_image_auto, decode_image_with, decode_ppm_ascii,
    };

    #[cfg(all(feature = "std", not(feature = "image-decode")))]
    pub use crate::{LayerCanvas, TestBuffer};
}
