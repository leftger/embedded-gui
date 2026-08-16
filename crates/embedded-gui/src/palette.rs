//! Semantic color roles and display modes for HUD / instrument panels.
//!
//! Consumers map UI elements to [`InkRole`] values and resolve them through a
//! [`DisplayPalette`] for the active [`DisplayMode`]. This replaces ad-hoc
//! tagging tricks (such as encoding highlight ink in a color channel LSB) with
//! an explicit normal vs. stealth palette.

use embedded_graphics_core::pixelcolor::Rgb565;

/// Active display appearance mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayMode {
    #[default]
    Normal,
    /// Low-brightness, reduced-emission palette for covert / night use.
    Stealth,
}

/// Semantic ink roles used when drawing HUD elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InkRole {
    Background,
    Primary,
    Highlight,
    Accent,
    Muted,
}

/// Normal and stealth colors for each [`InkRole`].
#[derive(Clone, Copy, Debug)]
pub struct DisplayPalette {
    pub mode: DisplayMode,
    pub normal: RoleColors,
    pub stealth: RoleColors,
}

/// Concrete RGB565 values for each ink role in one mode.
#[derive(Clone, Copy, Debug)]
pub struct RoleColors {
    pub background: Rgb565,
    pub primary: Rgb565,
    pub highlight: Rgb565,
    pub accent: Rgb565,
    pub muted: Rgb565,
}

impl RoleColors {
    pub const fn new(
        background: Rgb565,
        primary: Rgb565,
        highlight: Rgb565,
        accent: Rgb565,
        muted: Rgb565,
    ) -> Self {
        Self {
            background,
            primary,
            highlight,
            accent,
            muted,
        }
    }

    pub const fn resolve(self, role: InkRole) -> Rgb565 {
        match role {
            InkRole::Background => self.background,
            InkRole::Primary => self.primary,
            InkRole::Highlight => self.highlight,
            InkRole::Accent => self.accent,
            InkRole::Muted => self.muted,
        }
    }
}

impl DisplayPalette {
    pub const fn new(normal: RoleColors, stealth: RoleColors) -> Self {
        Self {
            mode: DisplayMode::Normal,
            normal,
            stealth,
        }
    }

    pub const fn with_mode(self, mode: DisplayMode) -> Self {
        Self { mode, ..self }
    }

    pub fn set_mode(&mut self, mode: DisplayMode) {
        self.mode = mode;
    }

    pub fn resolve(&self, role: InkRole) -> Rgb565 {
        match self.mode {
            DisplayMode::Normal => self.normal.resolve(role),
            DisplayMode::Stealth => self.stealth.resolve(role),
        }
    }
}
