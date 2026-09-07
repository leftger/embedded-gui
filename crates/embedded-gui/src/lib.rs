#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod adapter;
pub mod block;
pub mod colors;
pub mod completion;
pub mod context;
pub mod display_backend;
#[cfg(feature = "embassy")]
pub mod embassy;
pub mod font;
pub mod framebuffer;
pub mod geometry;
pub mod gradient;
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
pub mod nine_slice;
pub mod palette;
pub mod pdc;
pub mod power;
pub mod profiler;
pub mod quantize;
pub mod spline;
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
pub mod view;
pub mod visual_widgets;
pub mod widget;
pub mod widgets;

pub use geometry::FluentBuilder;
pub use text::{StringArena, TextSlice};
pub use view::{FlexBuilder, Render, ViewContext};

pub use haptics::{HapticPattern, HapticSequencer};
pub use i18n::{LanguageId, TranslationEntry, TranslationTable};

pub use visual_widgets::{BusyWheel, GaugeWidget};
#[cfg(feature = "embedded-dsp")]
pub use visual_widgets::{SpectrumAnalyzerWidget, TouchInputFilter};

pub use adapter::{ColorConvertedDrawTarget, DrawTargetColorExt};
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
pub use colors::ColorOps;
pub use gradient::{ColorStop, MultiStopGradient};
pub use motion::stopwatch::{PlaybackMode, Stopwatch};
pub use nine_slice::{BorderRect, NineSlice, NineSliceLayout, SlicePair, SliceScaleMode};
pub use power::{PowerConfig, PowerEvent, PowerManager, PowerState};
pub use profiler::{DirtyRectVisualizer, FrameBudgetTracker};
pub use quantize::{
    BAYER_2X2, BAYER_4X4, DirtyBandAccumulator, bayer_1bit, bayer_2bit, bayer_4bit,
    bayer_dither_rgb565,
};
pub use spline::CatmullRomSpline;
pub use widgets::segmented::{SegmentStyle, SevenSegmentDisplay, encode_7seg, encode_14seg};

/// Canonical pixel color type for the active target configuration.
///
/// Defaults to [`embedded_graphics_core::pixelcolor::Rgb565`], but can be selected at compile time via the
/// `color-rgb888` or `color-gray8` feature flags for zero-cost hardware specialization.
#[cfg(feature = "color-rgb888")]
pub type ActiveColor = embedded_graphics_core::pixelcolor::Rgb888;
#[cfg(all(feature = "color-gray8", not(feature = "color-rgb888")))]
pub type ActiveColor = embedded_graphics_core::pixelcolor::Gray8;
#[cfg(not(any(feature = "color-rgb888", feature = "color-gray8")))]
pub type ActiveColor = embedded_graphics_core::pixelcolor::Rgb565;

/// Alias for [`ActiveColor`].
pub type CanvasColor = ActiveColor;

/// Alias for [`ActiveColor`].
///
/// Note: Omitted from [`prelude`] to avoid colliding with [`embedded_graphics_core::Pixel`]
/// when both preludes are glob-imported in the same module.
pub type Pixel = ActiveColor;

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
pub use framebuffer::{
    Framebuffer, FramebufferGray8, FramebufferRgba8888, FramebufferSlice, Rgba8888,
};
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
    EventPhaseMask, InputEvent, NavDirection, PointerButton, PointerState, UiEvent, UiEventFilter,
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
    DrawTask, DrawTaskQueue, DrawUnit, EllipsisMode, Hardware2DAccelerator, LayerState,
    PartialBandBuffer, PathVerb, PixelRead, RenderBackendCaps, RenderCtx, RenderQuality,
    Software2DAccelerator, SoftwareDrawUnit, StrokeCap, StrokeDash, StrokeJoin, StrokeStyle,
    TextAlign, TextMetrics, TextOverflow, TextOverflowPolicy, TextStyle, TextWrap, Transform2D,
    VectorPath, VerticalAlign, dispatch_draw_tasks,
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
    ActionBarSlot, ActionBarWidget, ActionMenuError, ActionMenuItem, ActionMenuWidget,
    ActionableDialogWidget, ArcGaugeWidget, BandMeterWidget, BatteryState, CompassMode,
    CompassWidget, ConfirmationDialogWidget, ContentIndicatorDirection, ContentIndicatorWidget,
    CrumbsIndicatorWidget, DialogAction, DialogError, DialogType, GaugeThreshold, InverterWidget,
    MatrixWidget, MeterOrientation, NotificationAction, NotificationError, NotificationPriority,
    NotificationSheetWidget, NumberPickerWidget, PeekBannerWidget, PickerError, ProgressBarWidget,
    RepeaterWidget, RichTextError, RichTextNodeWidget, ScaleMode, ScaleWidget, SelectionWidget,
    SparklineWidget, SpinboxWidget, StatusBarError, StatusBarMode, StatusBarWidget, TableWidget,
    TextSpan, TimeFormat, TimePickerField, TimePickerWidget, TimelineNodeState, TimelineNodeWidget,
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
        ActionBarSlot, ActionBarWidget, ActiveColor, ActiveScreenTransition, Align,
        AlphaLinearGradient, AlphaRadialGradient, AnimatedProperty, Animation,
        AnimationConflictPolicy, AnimationError, AnimationGroup, AnimationHandlers, AnimationId,
        AnimationManager, AnimationManagerCallbacks, AnimationSequence, AnimationState,
        AntiAliasMode, ArcGaugeWidget, Axis, BandMeterWidget, BasicTextShaper, BindingSnapshot,
        BitmapFont, Blend, BlendMode, Block, Border, BorderRect, CallbackSlot, CanvasColor,
        CardDeckDirection, CardDeckState, CardStory, CardStoryTransition, CarouselSpec,
        CatmullRomSpline, ChartMode, CinematicPreset, ColorConvertedDrawTarget, ColorFormat,
        ColorOps, ColorStop, CompassMode, CompassWidget, ComposedAnimation,
        ComposedAnimationCallbacks, ComposedAnimationPlayer, ComposedAnimationStatus,
        CompositeIconSpec, CompositionControls, CompositionMode, Compositor, Constraint,
        DirtyBandAccumulator, DirtyRectVisualizer, DirtyTracker, Dither, DrawTargetColorExt,
        Easing, EdgeInsets, EllipsisMode, EventContext, EventPhase, EventPhaseMask, EventPolicy,
        FeedTimelineState, FlexBuilder, FluentBuilder, FocusGroupId, Font, FontId,
        FrameBudgetTracker, Framebuffer, FramebufferGray8, FramebufferRgba8888, FramebufferSlice,
        GaugeThreshold, GlanceTileSpec, GradientDirection, GridLayout, GridPlacement, GridTrack,
        GuiContext, GuiError, GuiModel, HapticPattern, HapticSequencer, Hardware2DAccelerator,
        IconAlign, IconPart, ImageAtlas, ImageAtlasEntry, ImageFit, ImageRef, InertiaAnimator,
        InputEvent, InverterWidget, KeyBindingAction, KeyboardLayout, Keyframe, KeyframeTrack,
        KeyframeTrackCallbacks, LanguageId, LayerState, LayoutItem, Length, Line,
        LineBufferRenderer, LinearGradient, LinearLayout, ListState, MatrixWidget, MenuContract,
        MeterOrientation, ModelChange, MonoBitmap, MotionTokens, MultiStopGradient, NavDirection,
        NineSlice, NineSliceLayout, NotificationLevel, PackedFont, PathAnimator, PathPoint,
        PathVerb, PeekRevealSpec, PixelRead, PlaybackMode, PointerButton, PointerState,
        PowerConfig, PowerEvent, PowerManager, PowerState, PresentRegion, PressTiming,
        ProgressBarWidget, PropertySignal, Rect, ReelFrame, ReelPlayer, Render, RenderBackendCaps,
        RenderCtx, RenderQuality, RepeatMode, RepeaterWidget, Rgba8888, ScaleMode, ScaleWidget,
        ScanlineTarget, Screen, ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack,
        ScreenStackError, ScreenTransition, ScreenTransitionEffect, ScreenTransitionOrigin,
        ScreenTransitionRunner, ScreenTransitionSample, ScreenTransitionSpec, ScrollState,
        SegmentStyle, SequencePlayer, SequencePlayerStatus, SequenceRepeatMode,
        SevenSegmentDisplay, Shadow, ShapedGlyph, ShapingConfig, Signal, SliceModel, SlicePair,
        SliceScaleMode, SliderState, Software2DAccelerator, Span, SparklineWidget, SpinboxWidget,
        SpringAnimator, SpriteSheet, StateStyle, StateTransition, StatefulWidget, Stopwatch,
        StringArena, StrokeCap, StrokeDash, StrokeJoin, StrokeStyle, Style, StyleClassId,
        StyleTransition, SurfaceState, TableWidget, TabsState, Text, TextAlign, TextDirection,
        TextMetrics, TextOverflow, TextOverflowPolicy, TextShaper, TextSlice, TextStyle, TextWrap,
        Theme, TileMode, TileRef, TimelineError, TimelineMotionPreset, TimelineStep, Timer,
        Transform2D, TransitionPreset, TranslationEntry, TranslationTable, Tween, UiEvent,
        UiEventFilter, VectorPath, VerticalAlign, ViewContext, VisualState,
        WidgetAnimationCallbacks, WidgetAnimationError, WidgetAnimator, WidgetDispatchPolicy,
        WidgetEvent, WidgetEventFilter, WidgetEventKind, WidgetFlags, WidgetId, WidgetKeyBindings,
        WidgetKeyInputPolicy, WidgetKeyframeState, WidgetKind, WidgetNode, WidgetPropertyKeyframe,
        WidgetStateMachine, WidgetStyle, animate_glance_focus, animate_peek_reveal,
        apply_carddeck_visibility, apply_easing, bayer_1bit, bayer_2bit, bayer_4bit,
        bayer_dither_rgb565, colors, encode_7seg, encode_14seg, lerp_style, presets,
        render_transition_pair, setup_card_story, setup_launcher_glance,
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
