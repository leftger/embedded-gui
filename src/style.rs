use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};

use crate::{font::FontId, geometry::EdgeInsets};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Border {
    pub color: Rgb565,
    pub width: u8,
}

impl Border {
    pub const fn none() -> Self {
        Self {
            color: Rgb565::BLACK,
            width: 0,
        }
    }

    pub const fn one(color: Rgb565) -> Self {
        Self { color, width: 1 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shadow {
    pub color: Rgb565,
    pub opacity: u8,
    pub offset_x: i8,
    pub offset_y: i8,
    pub spread: u8,
}

impl Shadow {
    pub const fn none() -> Option<Self> {
        None
    }

    pub const fn soft() -> Self {
        Self {
            color: Rgb565::BLACK,
            opacity: 96,
            offset_x: 1,
            offset_y: 2,
            spread: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientDirection {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearGradient {
    pub start: Rgb565,
    pub end: Rgb565,
    pub direction: GradientDirection,
}

impl LinearGradient {
    pub const fn vertical(start: Rgb565, end: Rgb565) -> Self {
        Self {
            start,
            end,
            direction: GradientDirection::Vertical,
        }
    }

    pub const fn horizontal(start: Rgb565, end: Rgb565) -> Self {
        Self {
            start,
            end,
            direction: GradientDirection::Horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub background: Option<Rgb565>,
    pub gradient: Option<LinearGradient>,
    pub font: FontId,
    pub foreground: Rgb565,
    pub text: Rgb565,
    pub accent: Rgb565,
    pub opacity: u8,
    pub corner_radius: u8,
    pub shadow: Option<Shadow>,
    pub border: Border,
    pub padding: EdgeInsets,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            background: None,
            gradient: None,
            font: FontId::Tiny3x5,
            foreground: Rgb565::WHITE,
            text: Rgb565::WHITE,
            accent: Rgb565::new(0, 42, 31),
            opacity: 255,
            corner_radius: 0,
            shadow: Shadow::none(),
            border: Border::none(),
            padding: EdgeInsets::all(0),
        }
    }

    pub const fn panel() -> Self {
        Self {
            background: Some(Rgb565::new(2, 4, 8)),
            gradient: Some(LinearGradient::vertical(
                Rgb565::new(4, 8, 12),
                Rgb565::new(1, 2, 5),
            )),
            font: FontId::Tiny3x5,
            foreground: Rgb565::WHITE,
            text: Rgb565::WHITE,
            accent: Rgb565::new(0, 42, 31),
            opacity: 255,
            corner_radius: 2,
            shadow: Some(Shadow::soft()),
            border: Border::one(Rgb565::new(8, 16, 20)),
            padding: EdgeInsets::all(2),
        }
    }

    pub const fn label() -> Self {
        Self {
            background: None,
            gradient: None,
            font: FontId::Tiny3x5,
            foreground: Rgb565::WHITE,
            text: Rgb565::WHITE,
            accent: Rgb565::new(0, 42, 31),
            opacity: 255,
            corner_radius: 0,
            shadow: Shadow::none(),
            border: Border::none(),
            padding: EdgeInsets::all(0),
        }
    }

    pub const fn button() -> Self {
        Self {
            background: Some(Rgb565::new(4, 8, 12)),
            gradient: Some(LinearGradient::vertical(
                Rgb565::new(6, 12, 16),
                Rgb565::new(2, 4, 8),
            )),
            font: FontId::Medium4x7,
            foreground: Rgb565::WHITE,
            text: Rgb565::WHITE,
            accent: Rgb565::new(0, 48, 40),
            opacity: 255,
            corner_radius: 2,
            shadow: Some(Shadow {
                color: Rgb565::BLACK,
                opacity: 88,
                offset_x: 1,
                offset_y: 1,
                spread: 1,
            }),
            border: Border::one(Rgb565::new(12, 24, 28)),
            padding: EdgeInsets::symmetric(3, 2),
        }
    }

    pub const fn progress() -> Self {
        Self {
            background: Some(Rgb565::new(3, 4, 5)),
            gradient: Some(LinearGradient::horizontal(
                Rgb565::new(3, 5, 6),
                Rgb565::new(1, 2, 3),
            )),
            font: FontId::Tiny3x5,
            foreground: Rgb565::new(0, 50, 18),
            text: Rgb565::WHITE,
            accent: Rgb565::new(0, 50, 18),
            opacity: 255,
            corner_radius: 1,
            shadow: Shadow::none(),
            border: Border::one(Rgb565::new(9, 14, 14)),
            padding: EdgeInsets::all(1),
        }
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        if selected {
            self.background = Some(self.accent);
            self.border = Border::one(Rgb565::WHITE);
        }
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateStyle {
    pub style: Style,
}

impl StateStyle {
    pub const fn new(style: Style) -> Self {
        Self { style }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetStyle {
    pub normal: Style,
    pub focused: Style,
    pub pressed: Style,
    pub disabled: Style,
}

impl WidgetStyle {
    pub const fn new(normal: Style) -> Self {
        Self {
            normal,
            focused: normal.selected(true),
            pressed: normal.selected(true),
            disabled: Style {
                background: normal.background,
                gradient: normal.gradient,
                font: normal.font,
                foreground: Rgb565::new(8, 12, 12),
                text: Rgb565::new(12, 18, 18),
                accent: normal.accent,
                opacity: 170,
                corner_radius: normal.corner_radius,
                shadow: normal.shadow,
                border: normal.border,
                padding: normal.padding,
            },
        }
    }

    pub const fn with_focused(mut self, focused: Style) -> Self {
        self.focused = focused;
        self
    }

    pub const fn with_pressed(mut self, pressed: Style) -> Self {
        self.pressed = pressed;
        self
    }

    pub const fn with_disabled(mut self, disabled: Style) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn resolve(self, state: VisualState) -> Style {
        match state {
            VisualState::Normal => self.normal,
            VisualState::Focused => self.focused,
            VisualState::Pressed => self.pressed,
            VisualState::Disabled => self.disabled,
        }
    }

    pub const fn with_state_override(mut self, state: VisualState, style: Style) -> Self {
        match state {
            VisualState::Normal => self.normal = style,
            VisualState::Focused => self.focused = style,
            VisualState::Pressed => self.pressed = style,
            VisualState::Disabled => self.disabled = style,
        }
        self
    }

    pub fn resolve_interpolated(self, from: VisualState, to: VisualState, t: f32) -> Style {
        let a = self.resolve(from);
        let b = self.resolve(to);
        lerp_style(a, b, t)
    }
}

impl From<Style> for WidgetStyle {
    fn from(style: Style) -> Self {
        Self::new(style)
    }
}

impl From<StateStyle> for WidgetStyle {
    fn from(style: StateStyle) -> Self {
        Self::new(style.style)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub panel: Style,
    pub label: Style,
    pub button: Style,
    pub progress: Style,
    pub toggle: Style,
    pub checkbox: Style,
    pub slider: Style,
    pub value_label: Style,
    pub icon_button: Style,
    pub list: Style,
    pub dialog: Style,
    pub toast: Style,
    pub tabs: Style,
    pub meter: Style,
    pub focus_ring: Rgb565,
}

impl Theme {
    pub const fn dark() -> Self {
        Self {
            panel: Style::panel(),
            label: Style::label(),
            button: Style::button(),
            progress: Style::progress(),
            toggle: Style::button(),
            checkbox: Style::button(),
            slider: Style::button(),
            value_label: Style::panel(),
            icon_button: Style::button(),
            list: Style::button(),
            dialog: Style {
                background: Some(Rgb565::new(5, 8, 14)),
                gradient: Some(LinearGradient::vertical(
                    Rgb565::new(7, 12, 18),
                    Rgb565::new(2, 4, 8),
                )),
                font: FontId::Scaled6x10,
                foreground: Rgb565::WHITE,
                text: Rgb565::WHITE,
                accent: Rgb565::new(31, 44, 0),
                opacity: 255,
                corner_radius: 3,
                shadow: Some(Shadow {
                    color: Rgb565::BLACK,
                    opacity: 120,
                    offset_x: 2,
                    offset_y: 2,
                    spread: 3,
                }),
                border: Border::one(Rgb565::WHITE),
                padding: EdgeInsets::all(4),
            },
            toast: Style {
                background: Some(Rgb565::new(8, 10, 2)),
                gradient: Some(LinearGradient::vertical(
                    Rgb565::new(10, 14, 4),
                    Rgb565::new(5, 6, 1),
                )),
                font: FontId::Medium4x7,
                foreground: Rgb565::WHITE,
                text: Rgb565::WHITE,
                accent: Rgb565::new(31, 48, 0),
                opacity: 255,
                corner_radius: 2,
                shadow: Some(Shadow {
                    color: Rgb565::BLACK,
                    opacity: 72,
                    offset_x: 1,
                    offset_y: 1,
                    spread: 1,
                }),
                border: Border::one(Rgb565::new(18, 22, 6)),
                padding: EdgeInsets::symmetric(4, 2),
            },
            tabs: Style::button(),
            meter: Style::progress(),
            focus_ring: Rgb565::new(31, 56, 0),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

pub fn lerp_style(a: Style, b: Style, t: f32) -> Style {
    let t = t.clamp(0.0, 1.0);
    let blend = |c1: Rgb565, c2: Rgb565| {
        let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
        Rgb565::new(lerp(c1.r(), c2.r()), lerp(c1.g(), c2.g()), lerp(c1.b(), c2.b()))
    };
    Style {
        background: Some(blend(
            a.background.unwrap_or(Rgb565::BLACK),
            b.background.unwrap_or(Rgb565::BLACK),
        )),
        gradient: a.gradient.or(b.gradient),
        font: a.font,
        foreground: blend(a.foreground, b.foreground),
        text: blend(a.text, b.text),
        accent: blend(a.accent, b.accent),
        opacity: (a.opacity as f32 + (b.opacity as f32 - a.opacity as f32) * t) as u8,
        corner_radius: (a.corner_radius as f32 + (b.corner_radius as f32 - a.corner_radius as f32) * t)
            as u8,
        shadow: a.shadow.or(b.shadow),
        border: Border {
            color: blend(a.border.color, b.border.color),
            width: (a.border.width as f32 + (b.border.width as f32 - a.border.width as f32) * t)
                as u8,
        },
        padding: a.padding,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleTransition {
    pub from: VisualState,
    pub to: VisualState,
    pub animation: crate::Animation,
}

impl StyleTransition {
    pub const fn new(
        from: VisualState,
        to: VisualState,
        duration_ms: u32,
        easing: crate::Easing,
    ) -> Self {
        Self {
            from,
            to,
            animation: crate::Animation::new(0.0, 1.0, duration_ms, easing),
        }
    }

    pub fn tick(&mut self, dt_ms: u32) {
        self.animation.tick(dt_ms);
    }

    pub fn style(&self, styles: WidgetStyle) -> Style {
        styles.resolve_interpolated(self.from, self.to, self.animation.value())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualState {
    Normal,
    Focused,
    Pressed,
    Disabled,
}
