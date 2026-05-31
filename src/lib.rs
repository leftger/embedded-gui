#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod animation;
pub mod block;
pub mod context;
pub mod font;
pub mod geometry;
pub mod input;
pub mod layout;
pub mod present;
pub mod render;
pub mod screen;
pub mod state;
pub mod style;
#[cfg(feature = "std")]
pub mod test_buffer;
pub mod text;
pub mod widget;
pub mod widgets;

pub use animation::{Easing, Timer, Tween, apply_easing};
pub use block::Block;
pub use context::{GuiContext, GuiError};
pub use font::FontId;
pub use geometry::{DirtyTracker, EdgeInsets, Rect};
pub use input::{
    EventPhaseMask, InputEvent, PointerButton, PointerState, UiEvent, UiEventFilter,
    WidgetDispatchPolicy, WidgetEvent, WidgetEventFilter, WidgetEventKind,
};
pub use layout::{Align, Axis, Constraint, LayoutItem, Length, LinearLayout};
pub use present::PresentRegion;
pub use render::{
    CHAR_HEIGHT, CHAR_WIDTH, RenderCtx, RenderQuality, TextAlign, TextMetrics, TextStyle, TextWrap,
    VerticalAlign,
};
pub use screen::{
    Screen, ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack, ScreenStackError,
    ScreenTransition,
};
pub use state::{ListState, ScrollState, SliderState, TabsState};
pub use style::{
    Border, GradientDirection, LinearGradient, Shadow, StateStyle, Style, Theme, VisualState,
    WidgetStyle,
};
#[cfg(feature = "std")]
pub use test_buffer::TestBuffer;
pub use text::{Line, Span, Text};
pub use widget::{
    EventContext, EventPhase, EventPolicy, FocusGroupId, StatefulWidget, StyleClassId, WidgetFlags,
    WidgetId,
};
pub use widgets::{WidgetKind, WidgetNode};

pub mod prelude {
    pub use crate::{
        Align, Axis, Block, Border, Constraint, DirtyTracker, Easing, EdgeInsets, EventContext,
        EventPhase, EventPhaseMask, EventPolicy, FocusGroupId, FontId, GradientDirection,
        GuiContext, GuiError, InputEvent, LayoutItem, Length, Line, LinearGradient, LinearLayout,
        ListState, PointerButton, PointerState, PresentRegion, Rect, RenderCtx, RenderQuality,
        Screen, ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack, ScreenStackError,
        ScreenTransition, ScrollState, Shadow, SliderState, Span, StateStyle, StatefulWidget,
        Style, StyleClassId, TabsState, Text, TextAlign, TextMetrics, TextStyle, TextWrap, Theme,
        Timer, Tween, UiEvent, UiEventFilter, VerticalAlign, VisualState, WidgetDispatchPolicy,
        WidgetEvent, WidgetEventFilter, WidgetEventKind, WidgetFlags, WidgetId, WidgetKind,
        WidgetStyle, apply_easing,
    };

    #[cfg(feature = "std")]
    pub use crate::TestBuffer;
}
