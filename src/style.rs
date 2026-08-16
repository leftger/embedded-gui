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
pub struct AlphaLinearGradient {
    pub start_color: Rgb565,
    pub start_alpha: u8,
    pub end_color: Rgb565,
    pub end_alpha: u8,
    pub direction: GradientDirection,
}

impl AlphaLinearGradient {
    pub const fn new(
        start_color: Rgb565,
        start_alpha: u8,
        end_color: Rgb565,
        end_alpha: u8,
        direction: GradientDirection,
    ) -> Self {
        Self {
            start_color,
            start_alpha,
            end_color,
            end_alpha,
            direction,
        }
    }

    pub const fn vertical(
        start_color: Rgb565,
        start_alpha: u8,
        end_color: Rgb565,
        end_alpha: u8,
    ) -> Self {
        Self::new(
            start_color,
            start_alpha,
            end_color,
            end_alpha,
            GradientDirection::Vertical,
        )
    }

    pub const fn horizontal(
        start_color: Rgb565,
        start_alpha: u8,
        end_color: Rgb565,
        end_alpha: u8,
    ) -> Self {
        Self::new(
            start_color,
            start_alpha,
            end_color,
            end_alpha,
            GradientDirection::Horizontal,
        )
    }

    pub fn sample(&self, t: u8) -> (Rgb565, u8) {
        let color = lerp_rgb565_public(self.start_color, self.end_color, t);
        let alpha = lerp_u8(self.start_alpha, self.end_alpha, t);
        (color, alpha)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlphaRadialGradient {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
    pub start_color: Rgb565,
    pub start_alpha: u8,
    pub end_color: Rgb565,
    pub end_alpha: u8,
}

impl AlphaRadialGradient {
    pub fn new(
        center_x: f32,
        center_y: f32,
        radius: f32,
        start_color: Rgb565,
        start_alpha: u8,
        end_color: Rgb565,
        end_alpha: u8,
    ) -> Self {
        Self {
            center_x,
            center_y,
            radius: if radius <= 0.0 { 1.0 } else { radius },
            start_color,
            start_alpha,
            end_color,
            end_alpha,
        }
    }

    pub fn sample_at_dist(&self, dist: f32) -> (Rgb565, u8) {
        let t = (dist / self.radius).clamp(0.0, 1.0);
        let t_u8 = (t * 255.0) as u8;
        let color = lerp_rgb565_public(self.start_color, self.end_color, t_u8);
        let alpha = lerp_u8(self.start_alpha, self.end_alpha, t_u8);
        (color, alpha)
    }
}

#[inline]
pub fn lerp_u8(a: u8, b: u8, t: u8) -> u8 {
    let t = t as u32;
    let inv = 255u32 - t;
    (((a as u32 * inv) + (b as u32 * t)) / 255) as u8
}

#[inline]
pub fn lerp_rgb565_public(a: Rgb565, b: Rgb565, t: u8) -> Rgb565 {
    let t = t as u16;
    let inv = 255u16.saturating_sub(t);
    let r = ((a.r() as u16 * inv) + (b.r() as u16 * t)) / 255;
    let g = ((a.g() as u16 * inv) + (b.g() as u16 * t)) / 255;
    let bb = ((a.b() as u16 * inv) + (b.b() as u16 * t)) / 255;
    Rgb565::new(r as u8, g as u8, bb as u8)
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

    pub const fn with_font_id(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    pub fn with_font(mut self, font: impl Into<FontId>) -> Self {
        self.font = font.into();
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
        Rgb565::new(
            lerp(c1.r(), c2.r()),
            lerp(c1.g(), c2.g()),
            lerp(c1.b(), c2.b()),
        )
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
        corner_radius: (a.corner_radius as f32
            + (b.corner_radius as f32 - a.corner_radius as f32) * t) as u8,
        shadow: a.shadow.or(b.shadow),
        border: Border {
            color: blend(a.border.color, b.border.color),
            width: (a.border.width as f32 + (b.border.width as f32 - a.border.width as f32) * t)
                as u8,
        },
        padding: a.padding,
    }
}

pub fn lerp_theme(a: Theme, b: Theme, t: f32) -> Theme {
    let t = t.clamp(0.0, 1.0);
    let blend_color = |c1: Rgb565, c2: Rgb565| {
        let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
        Rgb565::new(
            lerp(c1.r(), c2.r()),
            lerp(c1.g(), c2.g()),
            lerp(c1.b(), c2.b()),
        )
    };
    Theme {
        panel: lerp_style(a.panel, b.panel, t),
        label: lerp_style(a.label, b.label, t),
        button: lerp_style(a.button, b.button, t),
        progress: lerp_style(a.progress, b.progress, t),
        toggle: lerp_style(a.toggle, b.toggle, t),
        checkbox: lerp_style(a.checkbox, b.checkbox, t),
        slider: lerp_style(a.slider, b.slider, t),
        value_label: lerp_style(a.value_label, b.value_label, t),
        icon_button: lerp_style(a.icon_button, b.icon_button, t),
        list: lerp_style(a.list, b.list, t),
        dialog: lerp_style(a.dialog, b.dialog, t),
        toast: lerp_style(a.toast, b.toast, t),
        tabs: lerp_style(a.tabs, b.tabs, t),
        meter: lerp_style(a.meter, b.meter, t),
        focus_ring: blend_color(a.focus_ring, b.focus_ring),
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

/// Targetable sub-component part of a widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetPart {
    Main,
    Indicator,
    Knob,
    Scrollbar,
    Custom(u8),
}

/// Bitmask representing active visual states (supports combining states like CHECKED | PRESSED).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct VisualStateMask(pub u8);

impl VisualStateMask {
    pub const NORMAL: Self = Self(1 << 0);
    pub const FOCUSED: Self = Self(1 << 1);
    pub const PRESSED: Self = Self(1 << 2);
    pub const DISABLED: Self = Self(1 << 3);
    pub const CHECKED: Self = Self(1 << 4);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn from_visual_state(state: VisualState) -> Self {
        match state {
            VisualState::Normal => Self::NORMAL,
            VisualState::Focused => Self::FOCUSED,
            VisualState::Pressed => Self::PRESSED,
            VisualState::Disabled => Self::DISABLED,
        }
    }
}

/// A cascading rule that applies a style to a specific widget part under matching visual states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartStyleRule {
    pub part: WidgetPart,
    pub state_mask: VisualStateMask,
    pub style: Style,
}

/// A multi-part style descriptor managing distinct visual rules for parts of a compound widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiPartStyle<const RULES: usize = 4> {
    pub base_style: Style,
    rules: [Option<PartStyleRule>; RULES],
}

impl<const RULES: usize> Default for MultiPartStyle<RULES> {
    fn default() -> Self {
        Self::new(Style::new())
    }
}

impl<const RULES: usize> MultiPartStyle<RULES> {
    pub const fn new(base_style: Style) -> Self {
        Self {
            base_style,
            rules: [None; RULES],
        }
    }

    pub const fn with_part_rule(
        mut self,
        part: WidgetPart,
        state_mask: VisualStateMask,
        style: Style,
    ) -> Self {
        let mut i = 0;
        while i < RULES {
            if self.rules[i].is_none() {
                self.rules[i] = Some(PartStyleRule {
                    part,
                    state_mask,
                    style,
                });
                return self;
            }
            i += 1;
        }
        self
    }

    pub fn resolve(&self, part: WidgetPart, state: VisualState) -> Style {
        let mask = VisualStateMask::from_visual_state(state);
        self.resolve_mask(part, mask)
    }

    pub fn resolve_mask(&self, part: WidgetPart, state_mask: VisualStateMask) -> Style {
        for rule in self.rules.iter().flatten() {
            if rule.part == part && (rule.state_mask.0 == 0 || rule.state_mask.contains(state_mask))
            {
                return rule.style;
            }
        }
        self.base_style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::pixelcolor::WebColors;

    #[test]
    fn test_border_and_shadow_presets() {
        let none_border = Border::none();
        assert_eq!(none_border.width, 0);

        let one_border = Border::one(Rgb565::CSS_RED);
        assert_eq!(one_border.width, 1);
        assert_eq!(one_border.color, Rgb565::CSS_RED);

        assert_eq!(Shadow::none(), None);
        let soft_shadow = Shadow::soft();
        assert_eq!(soft_shadow.opacity, 96);
        assert_eq!(soft_shadow.offset_x, 1);
    }

    #[test]
    fn test_style_resolution_and_state_overrides() {
        let base_style = Style {
            background: Some(Rgb565::CSS_BLUE),
            gradient: None,
            font: FontId::Tiny3x5,
            foreground: Rgb565::CSS_WHITE,
            text: Rgb565::CSS_WHITE,
            accent: Rgb565::CSS_RED,
            opacity: 255,
            corner_radius: 0,
            shadow: Shadow::none(),
            border: Border::none(),
            padding: EdgeInsets::all(4),
        };

        let focused_style = Style {
            background: Some(Rgb565::CSS_YELLOW),
            ..base_style
        };

        let widget_style =
            WidgetStyle::new(base_style).with_state_override(VisualState::Focused, focused_style);

        assert_eq!(widget_style.resolve(VisualState::Normal), base_style);
        assert_eq!(widget_style.resolve(VisualState::Focused), focused_style);
        assert_eq!(
            widget_style.resolve(VisualState::Pressed),
            base_style.selected(true)
        );
    }

    #[test]
    fn test_style_lerp_and_transition() {
        let s1 = Style {
            background: Some(Rgb565::new(0, 0, 0)),
            gradient: None,
            font: FontId::Tiny3x5,
            foreground: Rgb565::new(0, 0, 0),
            text: Rgb565::new(0, 0, 0),
            accent: Rgb565::new(0, 0, 0),
            opacity: 0,
            corner_radius: 0,
            shadow: Shadow::none(),
            border: Border::none(),
            padding: EdgeInsets::all(0),
        };

        let s2 = Style {
            background: Some(Rgb565::new(31, 63, 31)),
            gradient: None,
            font: FontId::Tiny3x5,
            foreground: Rgb565::new(31, 63, 31),
            text: Rgb565::new(31, 63, 31),
            accent: Rgb565::new(31, 63, 31),
            opacity: 255,
            corner_radius: 8,
            shadow: Shadow::none(),
            border: Border::one(Rgb565::new(31, 63, 31)),
            padding: EdgeInsets::all(10),
        };

        let mid = lerp_style(s1, s2, 0.5);
        assert!((mid.background.unwrap().r() as i32 - 15).abs() <= 1);
        assert_eq!(mid.corner_radius, 4);

        let widget_style = WidgetStyle::new(s1).with_state_override(VisualState::Focused, s2);

        let mut transition = StyleTransition::new(
            VisualState::Normal,
            VisualState::Focused,
            100,
            crate::Easing::Linear,
        );

        transition.tick(50);
        let current_style = transition.style(widget_style);
        assert_eq!(current_style.corner_radius, 4);
    }

    #[test]
    fn test_multipart_style_resolution() {
        let base_style = Style::new();
        let knob_style = Style {
            corner_radius: 10,
            ..base_style
        };
        let indicator_pressed_style = Style {
            corner_radius: 5,
            ..base_style
        };

        let multipart = MultiPartStyle::<4>::new(base_style)
            .with_part_rule(WidgetPart::Knob, VisualStateMask::empty(), knob_style)
            .with_part_rule(
                WidgetPart::Indicator,
                VisualStateMask::PRESSED,
                indicator_pressed_style,
            );

        // Knob gets knob_style regardless of normal state because state_mask is empty/wildcard
        assert_eq!(
            multipart.resolve(WidgetPart::Knob, VisualState::Normal),
            knob_style
        );

        // Indicator in Normal state falls back to base_style
        assert_eq!(
            multipart.resolve(WidgetPart::Indicator, VisualState::Normal),
            base_style
        );

        // Indicator in Pressed state resolves to indicator_pressed_style
        assert_eq!(
            multipart.resolve(WidgetPart::Indicator, VisualState::Pressed),
            indicator_pressed_style
        );
    }
}
