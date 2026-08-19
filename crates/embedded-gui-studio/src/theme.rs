//! Display-theme color palettes shared by the RGB565 renderer and Studio UI.

use crate::types::DisplayTheme;
use eframe::egui::Color32;

/// Semantic colors resolved from KDL style tokens.
pub struct ThemePalette {
    pub display_bg: Color32,
    pub card_bg: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub success: Color32,
    pub danger: Color32,
}

impl ThemePalette {
    pub fn for_theme(theme: DisplayTheme) -> Self {
        match theme {
            DisplayTheme::DarkTft => Self {
                display_bg: Color32::from_rgb(18, 20, 24),
                card_bg: Color32::from_rgb(30, 33, 40),
                border: Color32::from_rgb(55, 62, 75),
                text_primary: Color32::from_rgb(230, 235, 245),
                text_dim: Color32::from_rgb(140, 150, 165),
                accent: Color32::from_rgb(45, 110, 220),
                success: Color32::from_rgb(40, 190, 110),
                danger: Color32::from_rgb(220, 50, 50),
            },
            DisplayTheme::LightTft => Self {
                display_bg: Color32::from_rgb(240, 244, 248),
                card_bg: Color32::WHITE,
                border: Color32::from_rgb(205, 215, 225),
                text_primary: Color32::from_rgb(20, 25, 35),
                text_dim: Color32::from_rgb(90, 100, 115),
                accent: Color32::from_rgb(25, 95, 210),
                success: Color32::from_rgb(30, 160, 90),
                danger: Color32::from_rgb(210, 40, 40),
            },
            DisplayTheme::AmberPhosphor => Self {
                display_bg: Color32::from_rgb(15, 10, 4),
                card_bg: Color32::from_rgb(30, 20, 6),
                border: Color32::from_rgb(140, 95, 20),
                text_primary: Color32::from_rgb(255, 180, 40),
                text_dim: Color32::from_rgb(180, 125, 25),
                accent: Color32::from_rgb(255, 160, 20),
                success: Color32::from_rgb(255, 195, 50),
                danger: Color32::from_rgb(255, 90, 20),
            },
            DisplayTheme::EmeraldGreen => Self {
                display_bg: Color32::from_rgb(4, 15, 8),
                card_bg: Color32::from_rgb(8, 30, 15),
                border: Color32::from_rgb(25, 120, 55),
                text_primary: Color32::from_rgb(50, 255, 120),
                text_dim: Color32::from_rgb(35, 175, 80),
                accent: Color32::from_rgb(40, 230, 100),
                success: Color32::from_rgb(80, 255, 140),
                danger: Color32::from_rgb(255, 140, 40),
            },
            DisplayTheme::MonochromeOled => Self {
                display_bg: Color32::BLACK,
                card_bg: Color32::from_rgb(16, 16, 16),
                border: Color32::WHITE,
                text_primary: Color32::WHITE,
                text_dim: Color32::from_rgb(180, 180, 180),
                accent: Color32::WHITE,
                success: Color32::WHITE,
                danger: Color32::WHITE,
            },
            DisplayTheme::SoftUi => Self {
                display_bg: Color32::from_rgb(238, 243, 247),
                card_bg: Color32::from_rgb(250, 251, 252),
                border: Color32::from_rgb(220, 221, 222),
                text_primary: Color32::from_rgb(33, 36, 41),
                text_dim: Color32::from_rgb(120, 126, 135),
                accent: Color32::from_rgb(58, 125, 214),
                success: Color32::from_rgb(34, 170, 94),
                danger: Color32::from_rgb(214, 62, 62),
            },
        }
    }
}
