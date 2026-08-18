#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod block;
pub mod completion;
pub mod context;
pub mod display_backend;
#[cfg(feature = "embassy")]
pub mod embassy;
pub mod font;
pub mod framebuffer;
pub mod geometry;
pub mod haptics;
pub mod image;
pub mod input;
#[cfg(any(
    feature = "embedded-text",
    feature = "embedded-layout",
    feature = "embedded-3dgfx"
))]
pub mod interop;
pub mod layout;
mod math;
pub mod mono;
pub mod motion;
pub mod palette;
pub mod pdc;
pub use motion as animation;
pub use motion as animation_timeline;
pub use motion as animation_timing;
pub use motion as cinematic;
pub use motion as screen_transition;
pub use motion as transition_preset;
pub use motion as widget_animation;
pub mod i18n;
pub mod present;
pub mod render;
pub mod round;
pub mod screen;
pub mod state;
pub mod style;
pub mod swapchain;
#[cfg(feature = "std")]
pub mod test_buffer;
pub mod text;
pub mod visual_widgets;
pub mod widget;
pub mod widgets;

pub use haptics::{HapticPattern, HapticSequencer};
pub use i18n::{LanguageId, TranslationEntry, TranslationTable};

pub use visual_widgets::{BusyWheel, GaugeWidget};
#[cfg(feature = "embedded-dsp")]
pub use visual_widgets::{SpectrumAnalyzerWidget, TouchInputFilter};

pub use animation::{
    Animation, AnimationError, AnimationHandlers, AnimationId, AnimationManager,
    AnimationManagerCallbacks, AnimationState, Easing, InertiaAnimator, PathAnimator, PathPoint,
    RepeatMode, SpringAnimator, Timer, Tween, apply_easing,
};
pub use animation_timeline::{
    AnimationGroup, AnimationSequence, ComposedAnimation, ComposedAnimationCallbacks,
    ComposedAnimationPlayer, ComposedAnimationStatus, CompositionControls, CompositionMode,
    Keyframe, KeyframeTrack, KeyframeTrackCallbacks, SequencePlayer, SequencePlayerStatus,
    SequenceRepeatMode, TimelineError, TimelineStep,
};
pub use animation_timing::{
    DEFAULT_DURATION_MS, FRAME_INTERVAL_MS, MOOOK_DURATION_MS, NORMALIZED_MAX,
    PORT_HOLE_DURATION_MS, SHUTTER_DURATION_MS, interpolate_moook, moook_curve, moook_duration_ms,
    timing_half_phase, timing_scaled, timing_shutter_phase,
};
pub use block::Block;
pub use cinematic::{
    CardDeckDirection, CardDeckState, CardStory, CardStoryTransition, CinematicPreset,
    GlanceTileSpec, MotionTokens, PeekRevealSpec, TimelineMotionPreset, animate_glance_focus,
    animate_peek_reveal, apply_carddeck_visibility, setup_card_story, setup_launcher_glance,
    setup_launcher_glance_with_tokens, setup_peek_timeline, setup_peek_timeline_with_tokens,
};
pub use completion::{CompletionSlot, WaitTransfer, WaitTransferFuture};
pub use context::{
    GuiContext, GuiError, KeyBindingAction, PressTiming, WidgetBuilder, WidgetKeyBindings,
    WidgetKeyInputPolicy,
};
pub use display_backend::{
    AsyncDmaTransfer, DisplayBackend, DisplayError, DisplayRegion, DmaTransfer, SimulatorBackend,
    TransferError,
};
#[cfg(feature = "embassy")]
pub use embassy::{EmbassyWaitTransfer, EmbassyWaitTransferFuture, FrameClock};
pub use embedded_graphics_framebuf::{
    FrameBuf,
    backends::{DMACapableFrameBufferBackend, EndianCorrectedBuffer, EndianCorrection},
};
pub use font::{BitmapFont, Font, FontId, PackedFont};
pub use framebuffer::{Framebuffer, FramebufferGray8, FramebufferRgba8888, Rgba8888};
pub use geometry::{Anchor, DirtyTracker, EdgeInsets, HorizontalAlign, Rect};
#[cfg(all(feature = "std", feature = "image-decode"))]
pub use image::{
    BasicImageDecoder, EncodedImageFormat, ImageDecodeError, ImageDecoder, decode_image_auto,
    decode_image_with, decode_ppm_ascii,
};
pub use image::{
    ImageAtlas, ImageAtlasEntry, ImageFit, ImageRef, ReelFrame, ReelPlayer, SpriteSheet, TileMode,
    TileRef,
};
pub use input::{
    EventPhaseMask, InputEvent, PointerButton, PointerState, UiEvent, UiEventFilter,
    WidgetDispatchPolicy, WidgetEvent, WidgetEventFilter, WidgetEventKind,
};
pub use layout::{
    Align, Axis, Constraint, GridLayout, GridPlacement, GridTrack, JustifyContent, LayoutItem,
    Length, LinearLayout,
};
pub use mono::{IconAlign, IconPart, MonoBitmap};
pub use palette::{DisplayMode, DisplayPalette, InkRole, RoleColors};
pub use pdc::{PdcCommand, PdcCommandType, PdcImage, PdcPrecisePoint};
pub use present::PresentRegion;
pub use render::{
    AntiAliasMode, Blend, BlendMode, CHAR_HEIGHT, CHAR_WIDTH, ColorFormat, Compositor, Dither,
    DrawTask, DrawTaskQueue, DrawUnit, EllipsisMode, LayerState, PartialBandBuffer, PathVerb,
    PixelRead, RenderBackendCaps, RenderCtx, RenderQuality, SoftwareDrawUnit, StrokeCap,
    StrokeJoin, StrokeStyle, TextAlign, TextMetrics, TextOverflow, TextOverflowPolicy, TextStyle,
    TextWrap, Transform2D, VectorPath, VerticalAlign, dispatch_draw_tasks,
};
pub use render::{LineBufferRenderer, ScanlineTarget};
pub use round::{UnobstructedArea, circle_chord_width, round_screen_line_bounds};
pub use screen::{
    Screen, ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack, ScreenStackError,
    ScreenTransition,
};
pub use screen_transition::{
    ActiveScreenTransition, ScreenTransitionEffect, ScreenTransitionOrigin, ScreenTransitionRunner,
    ScreenTransitionSample, ScreenTransitionSpec, composite_framebuffer_fade,
    composite_framebuffer_scaled_y, fade_outgoing_opacity, render_transition_pair,
};
pub use state::{
    CallbackSlot, FeedTimelineState, GuiModel, ListState, ModelChange, PropertySignal, ScrollState,
    Signal, SliceModel, SliderState, StateTransition, TabsState, WidgetStateMachine,
};
pub use style::{
    AlphaLinearGradient, AlphaRadialGradient, Border, GradientDirection, LinearGradient,
    MultiPartStyle, PartStyleRule, Shadow, StateStyle, Style, StyleTransition, Theme, VisualState,
    VisualStateMask, WidgetPart, WidgetStyle, lerp_style,
};
pub use swapchain::{StandardSwapChain, SwapChain};
#[cfg(feature = "triple-buffering")]
pub use swapchain::{StandardTripleSwapChain, TripleSwapChain};
#[cfg(feature = "std")]
pub use test_buffer::{LayerCanvas, TestBuffer};
pub use text::{
    BasicTextShaper, Line, ShapedGlyph, ShapingConfig, Span, Text, TextDirection, TextShaper,
};
pub use transition_preset::TransitionPreset;
pub use widget::{
    EventContext, EventPhase, EventPolicy, FocusGroupId, MenuContract, PropertyError, PropertyKey,
    PropertyValue, StatefulWidget, StyleClassId, Widget, WidgetFlags, WidgetId,
};
pub use widget_animation::presets;
pub use widget_animation::{
    AnimatedProperty, AnimationConflictPolicy, BindingSnapshot, WidgetAnimationCallbacks,
    WidgetAnimationError, WidgetAnimator, WidgetKeyframeState, WidgetPropertyKeyframe,
};
pub use widgets::{
    ActionBarWidget, ActionMenuError, ActionMenuItem, ActionMenuWidget, ActionableDialogWidget,
    BatteryState, ConfirmationDialogWidget, ContentIndicatorDirection, ContentIndicatorWidget,
    CrumbsIndicatorWidget, DialogAction, DialogError, DialogType, NotificationAction,
    NotificationError, NotificationPriority, NotificationSheetWidget, NumberPickerWidget,
    PeekBannerWidget, PickerError, RepeaterWidget, RichTextError, RichTextNodeWidget, ScaleMode,
    ScaleWidget, SelectionWidget, SpinboxWidget, StatusBarError, StatusBarMode, StatusBarWidget,
    TableWidget, TextSpan, TimeFormat, TimePickerField, TimePickerWidget, TimelineNodeState,
    TimelineNodeWidget,
};
pub use widgets::{
    CarouselSpec, ChartMode, CompositeIconSpec, KeyboardLayout, NotificationLevel, SurfaceState,
    WidgetKind, WidgetNode,
};

pub mod prelude {
    /// Re-exported so generated code can name colors without the consumer
    /// depending on `embedded-graphics-core` directly.
    pub use embedded_graphics_core::pixelcolor::Rgb565;

    pub use crate::{
        ActiveScreenTransition, Align, AlphaLinearGradient, AlphaRadialGradient, AnimatedProperty,
        Animation, AnimationConflictPolicy, AnimationError, AnimationGroup, AnimationHandlers,
        AnimationId, AnimationManager, AnimationManagerCallbacks, AnimationSequence,
        AnimationState, AntiAliasMode, Axis, BasicTextShaper, BindingSnapshot, BitmapFont, Blend,
        BlendMode, Block, Border, CallbackSlot, CardDeckDirection, CardDeckState, CardStory,
        CardStoryTransition, CarouselSpec, ChartMode, CinematicPreset, ColorFormat,
        ComposedAnimation, ComposedAnimationCallbacks, ComposedAnimationPlayer,
        ComposedAnimationStatus, CompositeIconSpec, CompositionControls, CompositionMode,
        Compositor, Constraint, DirtyTracker, Dither, Easing, EdgeInsets, EllipsisMode,
        EventContext, EventPhase, EventPhaseMask, EventPolicy, FeedTimelineState, FocusGroupId,
        Font, FontId, Framebuffer, FramebufferGray8, FramebufferRgba8888, GlanceTileSpec,
        GradientDirection, GridLayout, GridPlacement, GridTrack, GuiContext, GuiError, GuiModel,
        HapticPattern, HapticSequencer, IconAlign, IconPart, ImageAtlas, ImageAtlasEntry, ImageFit,
        ImageRef, InertiaAnimator, InputEvent, KeyBindingAction, KeyboardLayout, Keyframe,
        KeyframeTrack, KeyframeTrackCallbacks, LanguageId, LayerState, LayoutItem, Length, Line,
        LineBufferRenderer, LinearGradient, LinearLayout, ListState, MenuContract, ModelChange,
        MonoBitmap, MotionTokens, NotificationLevel, PackedFont, PathAnimator, PathPoint, PathVerb,
        PeekRevealSpec, PixelRead, PointerButton, PointerState, PresentRegion, PressTiming,
        PropertySignal, Rect, ReelFrame, ReelPlayer, RenderBackendCaps, RenderCtx, RenderQuality,
        RepeatMode, RepeaterWidget, Rgba8888, ScaleMode, ScaleWidget, ScanlineTarget, Screen,
        ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack, ScreenStackError,
        ScreenTransition, ScreenTransitionEffect, ScreenTransitionOrigin, ScreenTransitionRunner,
        ScreenTransitionSample, ScreenTransitionSpec, ScrollState, SequencePlayer,
        SequencePlayerStatus, SequenceRepeatMode, Shadow, ShapedGlyph, ShapingConfig, Signal,
        SliceModel, SliderState, Span, SpinboxWidget, SpringAnimator, SpriteSheet, StateStyle,
        StateTransition, StatefulWidget, StrokeCap, StrokeJoin, StrokeStyle, Style, StyleClassId,
        StyleTransition, SurfaceState, TableWidget, TabsState, Text, TextAlign, TextDirection,
        TextMetrics, TextOverflow, TextOverflowPolicy, TextShaper, TextStyle, TextWrap, Theme,
        TileMode, TileRef, TimelineError, TimelineMotionPreset, TimelineStep, Timer, Transform2D,
        TransitionPreset, TranslationEntry, TranslationTable, Tween, UiEvent, UiEventFilter,
        VectorPath, VerticalAlign, VisualState, WidgetAnimationCallbacks, WidgetAnimationError,
        WidgetAnimator, WidgetDispatchPolicy, WidgetEvent, WidgetEventFilter, WidgetEventKind,
        WidgetFlags, WidgetId, WidgetKeyBindings, WidgetKeyInputPolicy, WidgetKeyframeState,
        WidgetKind, WidgetNode, WidgetPropertyKeyframe, WidgetStateMachine, WidgetStyle,
        animate_glance_focus, animate_peek_reveal, apply_carddeck_visibility, apply_easing,
        lerp_style, presets, render_transition_pair, setup_card_story, setup_launcher_glance,
        setup_launcher_glance_with_tokens, setup_peek_timeline, setup_peek_timeline_with_tokens,
    };

    #[cfg(all(feature = "std", feature = "image-decode"))]
    pub use crate::{
        BasicImageDecoder, EncodedImageFormat, ImageDecodeError, ImageDecoder, LayerCanvas,
        TestBuffer, decode_image_auto, decode_image_with, decode_ppm_ascii,
    };

    #[cfg(all(feature = "std", not(feature = "image-decode")))]
    pub use crate::{LayerCanvas, TestBuffer};

    #[cfg(feature = "macros")]
    pub use embedded_gui_macros::{gui_kdl, include_gui};
}

#[cfg(feature = "macros")]
pub use embedded_gui_macros::{gui_kdl, include_gui};
