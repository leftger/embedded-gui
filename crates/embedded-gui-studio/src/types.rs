//! Studio types, hardware profiles, display themes, screen transitions, and drag state definitions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioTab {
    VisualPreview,
    RustCodegen,
    AstHierarchy,
    AssetBrowser,
    ScreenFlow,
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
    Esp32S3Box,
    Stm32H7Capacitive,
    RoundWearableWatch,
    Waveshare43,
    Ssd1306Oled,
}

impl HardwareProfile {
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            Self::Custom => None,
            Self::Esp32S3Box => Some((320, 240)),
            Self::Stm32H7Capacitive => Some((480, 272)),
            Self::RoundWearableWatch => Some((240, 240)),
            Self::Waveshare43 => Some((800, 480)),
            Self::Ssd1306Oled => Some((128, 64)),
        }
    }

    pub fn bpp(&self) -> u32 {
        match self {
            Self::Ssd1306Oled => 1,
            _ => 16,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Custom => "Custom Screen",
            Self::Esp32S3Box => "ESP32-S3 Box (320×240 IPS)",
            Self::Stm32H7Capacitive => "STM32-H7 Touch (480×272 TFT)",
            Self::RoundWearableWatch => "Round Watch (240×240 GC9A01)",
            Self::Waveshare43 => "Waveshare 4.3\" (800×480 RGB565)",
            Self::Ssd1306Oled => "SSD1306 OLED (128×64 Mono 1-bit)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionStyle {
    SlideLeft,
    SlideRight,
    Fade,
    Instant,
}

impl TransitionStyle {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SlideLeft => "Slide Left (300ms)",
            Self::SlideRight => "Slide Right (300ms)",
            Self::Fade => "Fade (200ms)",
            Self::Instant => "Instant",
        }
    }

    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            Self::SlideLeft => "SlideLeft",
            Self::SlideRight => "SlideRight",
            Self::Fade => "Fade",
            Self::Instant => "Instant",
        }
    }

    pub fn from_code(s: &str) -> Self {
        match s {
            "SlideLeft" => Self::SlideLeft,
            "SlideRight" => Self::SlideRight,
            "Fade" => Self::Fade,
            _ => Self::Instant,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScreenTransition {
    pub target_screen_idx: usize,
    pub progress: f32,
    pub duration: f32,
    pub style: TransitionStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveDrag {
    None,
    ResizeColDivider { col_idx: usize },
    ResizeRowDivider { row_idx: usize },
    MoveWidget { widget_idx: usize },
    ResizeWidgetSpan { widget_idx: usize },
}
