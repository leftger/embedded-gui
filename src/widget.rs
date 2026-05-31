use embedded_graphics_core::{draw_target::DrawTarget, pixelcolor::Rgb565};

use crate::{geometry::Rect, render::RenderCtx};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetFlags(u16);

impl WidgetFlags {
    pub const HIDDEN: Self = Self(1 << 0);
    pub const DISABLED: Self = Self(1 << 1);
    pub const CLICKABLE: Self = Self(1 << 2);
    pub const SCROLLABLE: Self = Self(1 << 3);
    pub const FOCUSABLE: Self = Self(1 << 4);
    pub const CLIP_CHILDREN: Self = Self(1 << 5);
    pub const EVENT_BUBBLE: Self = Self(1 << 6);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }

    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    pub fn remove(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }

    pub fn set(&mut self, flag: Self, enabled: bool) {
        if enabled {
            self.insert(flag);
        } else {
            self.remove(flag);
        }
    }
}

impl core::ops::BitOr for WidgetFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for WidgetFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetId(pub u16);

impl WidgetId {
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

pub trait StatefulWidget<State> {
    fn render_stateful<D>(
        &self,
        area: Rect,
        state: &mut State,
        ctx: &mut RenderCtx<'_, D>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPhase {
    Capture,
    Target,
    Bubble,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPolicy {
    Continue,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventContext {
    pub target: WidgetId,
    pub current: WidgetId,
    pub phase: EventPhase,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusGroupId(pub u8);

impl FocusGroupId {
    pub const ROOT: Self = Self(0);

    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleClassId(pub u8);

impl StyleClassId {
    pub const NONE: Self = Self(0);

    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}
