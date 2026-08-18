//! Studio types, hardware profiles, display themes, screen transitions, and drag state definitions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioTab {
    VisualPreview,
    RustCodegen,
    AstHierarchy,
    AssetBrowser,
    ScreenFlow,
    Profiler,
    SignalPlayground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioMode {
    Design,
    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayTheme {
    DarkTft,
    LightTft,
    AmberPhosphor,
    EmeraldGreen,
    MonochromeOled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareProfile {
    Custom,
    /// Panel size reported by an attached display agent during the handshake.
    /// Named separately from the canned profiles so the UI never claims the
    /// board is a device it merely shares a resolution with.
    Detected {
        width: u32,
        height: u32,
    },
    Esp32S3Box,
    Stm32H7Capacitive,
    RoundWearableWatch,
    Waveshare43,
    Ssd1306Oled,
    /// SSD1357 RGB OLED, typically 96×64 (common wearable / compact module size).
    Ssd1357,
}

impl HardwareProfile {
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            Self::Custom => None,
            Self::Detected { width, height } => Some((*width, *height)),
            Self::Esp32S3Box => Some((320, 240)),
            Self::Stm32H7Capacitive => Some((480, 272)),
            Self::RoundWearableWatch => Some((240, 240)),
            Self::Waveshare43 => Some((800, 480)),
            Self::Ssd1306Oled => Some((128, 64)),
            Self::Ssd1357 => Some((96, 64)),
        }
    }

    pub fn bpp(&self) -> u32 {
        match self {
            Self::Ssd1306Oled => 1,
            _ => 16,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Custom => "Custom Screen".to_string(),
            Self::Detected { width, height } => {
                format!("Connected Display ({width}×{height})")
            }
            Self::Esp32S3Box => "ESP32-S3 Box (320×240 IPS)".to_string(),
            Self::Stm32H7Capacitive => "STM32-H7 Touch (480×272 TFT)".to_string(),
            Self::RoundWearableWatch => "Round Watch (240×240 GC9A01)".to_string(),
            Self::Waveshare43 => "Waveshare 4.3\" (800×480 RGB565)".to_string(),
            Self::Ssd1306Oled => "SSD1306 OLED (128×64 Mono 1-bit)".to_string(),
            Self::Ssd1357 => "SSD1357 OLED (96×64 RGB565)".to_string(),
        }
    }

    /// Stable slug for `project.kdl` `panel="..."` attributes.
    pub fn panel_slug(self) -> Option<&'static str> {
        match self {
            Self::Esp32S3Box => Some("esp32_s3_box"),
            Self::Stm32H7Capacitive => Some("stm32_h7_cap"),
            Self::RoundWearableWatch => Some("gc9a01"),
            Self::Waveshare43 => Some("waveshare_43"),
            Self::Ssd1306Oled => Some("ssd1306"),
            Self::Ssd1357 => Some("ssd1357"),
            Self::Custom | Self::Detected { .. } => None,
        }
    }

    pub fn from_panel_slug(slug: &str) -> Option<Self> {
        match slug.to_ascii_lowercase().as_str() {
            "esp32_s3_box" | "esp32s3box" => Some(Self::Esp32S3Box),
            "stm32_h7_cap" | "stm32h7" => Some(Self::Stm32H7Capacitive),
            "gc9a01" | "round_watch" => Some(Self::RoundWearableWatch),
            "waveshare_43" | "waveshare43" => Some(Self::Waveshare43),
            "ssd1306" => Some(Self::Ssd1306Oled),
            "ssd1357" => Some(Self::Ssd1357),
            _ => None,
        }
    }

    /// Picks a canned profile when dimensions match exactly; otherwise Custom.
    pub fn from_dimensions(width: u32, height: u32) -> Self {
        for candidate in [
            Self::Ssd1357,
            Self::Ssd1306Oled,
            Self::Esp32S3Box,
            Self::RoundWearableWatch,
            Self::Stm32H7Capacitive,
            Self::Waveshare43,
        ] {
            if candidate.dimensions() == Some((width, height)) {
                return candidate;
            }
        }
        Self::Custom
    }
}

#[cfg(test)]
mod hardware_profile_tests {
    use super::HardwareProfile;

    #[test]
    fn detected_panel_drives_screen_dimensions() {
        let detected = HardwareProfile::Detected {
            width: 320,
            height: 240,
        };
        assert_eq!(detected.dimensions(), Some((320, 240)));
        assert_eq!(detected.bpp(), 16);
    }

    /// A detected panel must stay distinct from a canned profile of the same
    /// size, so the UI does not misname the attached board.
    #[test]
    fn detected_panel_is_distinct_from_matching_preset() {
        let detected = HardwareProfile::Detected {
            width: 320,
            height: 240,
        };
        assert_eq!(
            detected.dimensions(),
            HardwareProfile::Esp32S3Box.dimensions()
        );
        assert_ne!(detected, HardwareProfile::Esp32S3Box);
        assert!(detected.name().contains("Connected Display"));
    }

    #[test]
    fn custom_profile_leaves_screen_size_alone() {
        assert_eq!(HardwareProfile::Custom.dimensions(), None);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionStyle {
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Fade,
    Dissolve,
    ZoomPush,
    Instant,
}

impl TransitionStyle {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SlideLeft => "Slide Left (300ms)",
            Self::SlideRight => "Slide Right (300ms)",
            Self::SlideUp => "Slide Up (300ms)",
            Self::SlideDown => "Slide Down (300ms)",
            Self::Fade => "Fade (200ms)",
            Self::Dissolve => "Dissolve (250ms)",
            Self::ZoomPush => "Zoom Push (300ms)",
            Self::Instant => "Instant",
        }
    }

    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            Self::SlideLeft => "SlideLeft",
            Self::SlideRight => "SlideRight",
            Self::SlideUp => "SlideUp",
            Self::SlideDown => "SlideDown",
            Self::Fade => "Fade",
            Self::Dissolve => "Dissolve",
            Self::ZoomPush => "ZoomPush",
            Self::Instant => "Instant",
        }
    }

    pub fn from_code(s: &str) -> Self {
        match s {
            "SlideLeft" => Self::SlideLeft,
            "SlideRight" => Self::SlideRight,
            "SlideUp" => Self::SlideUp,
            "SlideDown" => Self::SlideDown,
            "Fade" => Self::Fade,
            "Dissolve" => Self::Dissolve,
            "ZoomPush" => Self::ZoomPush,
            _ => Self::Instant,
        }
    }

    /// Maps the richer runtime preset catalog onto the Studio canvas effects.
    pub fn from_preset(s: &str) -> Self {
        match s {
            "window_push" | "timeline_slide" | "shutter_left" | "port_hole_left" => Self::SlideLeft,
            "window_pop" | "shutter_right" | "port_hole_right" => Self::SlideRight,
            "modal_present" | "shutter_up" | "port_hole_up" => Self::SlideUp,
            "modal_dismiss" | "shutter_down" | "port_hole_down" => Self::SlideDown,
            "fade" => Self::Fade,
            "round_flip_to_launcher" | "round_flip_from_launcher" => Self::ZoomPush,
            "none" | "instant" => Self::Instant,
            other => Self::from_code(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScreenTransition {
    pub target_screen_idx: usize,
    pub progress: f32,
    pub duration: f32,
    pub style: TransitionStyle,
    pub easing: String,
}

impl ScreenTransition {
    pub fn visual_progress(&self) -> f32 {
        let t = self.progress.clamp(0.0, 1.0);
        match self.easing.as_str() {
            "in_sine" => 1.0 - (t * core::f32::consts::FRAC_PI_2).cos(),
            "out_sine" => (t * core::f32::consts::FRAC_PI_2).sin(),
            "out_cubic" => 1.0 - (1.0 - t).powi(3),
            "out_back" => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            "moook" => t * t * (3.0 - 2.0 * t),
            "linear" => t,
            _ => -((core::f32::consts::PI * t).cos() - 1.0) / 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveDrag {
    None,
    ResizeColDivider { col_idx: usize },
    ResizeRowDivider { row_idx: usize },
    MoveWidget { widget_idx: usize },
    ResizeWidgetSpan { widget_idx: usize },
}
