//! `embedded-gui-codegen`: KDL Markup Parser and Rust Code Generator for `embedded-gui`
//!
//! Enables UI designers and non-technical domain experts to author declarative GUI screens
//! in KDL and compile them into deterministic, zero-allocation (`no_std`) Rust code.

pub mod assets;

use core::fmt::Write as _;
use kdl::{KdlDocument, KdlNode, KdlValue};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("KDL parse error: {0}")]
    Parse(#[from] kdl::KdlError),
    #[error("Missing required attribute '{0}' on node '{1}'")]
    MissingAttribute(&'static str, String),
    #[error("Invalid attribute value for '{0}': {1}")]
    InvalidValue(String, String),
    #[error("Invalid track specification: {0}")]
    InvalidTrack(String),
    #[error("Unknown widget or container node: '{0}'")]
    UnknownNode(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridTrackDef {
    Px(u32),
    Fr(u8),
    Auto,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridPlacementDef {
    pub col: usize,
    pub row: usize,
    pub col_span: usize,
    pub row_span: usize,
    /// Optional motion applied to this widget by the generated app and Studio preview.
    pub animation: Option<WidgetAnimationDef>,
}

impl Default for GridPlacementDef {
    fn default() -> Self {
        Self {
            col: 0,
            row: 0,
            col_span: 1,
            row_span: 1,
            animation: None,
        }
    }
}

/// Declarative animation attached to a widget node in KDL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WidgetAnimationDef {
    /// Named preset, for example `fade_in_up`, `slide_in_left`, or `pulse`.
    pub preset: String,
    /// `screen_enter`, `screen_exit`, `click`, or `loop`.
    pub trigger: String,
    pub duration_ms: u32,
    pub delay_ms: u32,
    /// An [`embedded_gui::animation::Easing`] variant in snake case.
    pub easing: String,
    /// Number of plays. `0` means repeat forever.
    pub repeat: u16,
}

impl Default for WidgetAnimationDef {
    fn default() -> Self {
        Self {
            preset: "fade_in_up".into(),
            trigger: "screen_enter".into(),
            duration_ms: 400,
            delay_ms: 0,
            easing: "out_cubic".into(),
            repeat: 1,
        }
    }
}

/// Default transition used when this screen is navigated to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenTransitionDef {
    /// A named `TransitionPreset`, stored in snake case.
    pub preset: String,
    pub duration_ms: u32,
    pub easing: String,
    pub origin: String,
}

impl Default for ScreenTransitionDef {
    fn default() -> Self {
        Self {
            preset: "window_push".into(),
            duration_ms: 300,
            easing: "in_out_sine".into(),
            origin: "center".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PathVerbDef {
    MoveTo(i32, i32),
    LineTo(i32, i32),
    QuadTo(i32, i32, i32, i32),
    CubicTo(i32, i32, i32, i32, i32, i32),
    Close,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WidgetDef {
    Label {
        id: Option<String>,
        text: String,
        style: Option<String>,
        /// Name of a `font` declared on the screen, e.g. a BDF import.
        font: Option<String>,
    },
    Button {
        id: Option<String>,
        text: String,
        on_click: Option<String>,
        style: Option<String>,
    },
    Toggle {
        id: Option<String>,
        label: String,
        checked: bool,
    },
    Checkbox {
        id: Option<String>,
        label: String,
        checked: bool,
    },
    Slider {
        id: Option<String>,
        min: i32,
        max: i32,
        value: i32,
    },
    Dropdown {
        id: Option<String>,
        options: Vec<String>,
        selected: usize,
    },
    Roller {
        id: Option<String>,
        options: Vec<String>,
        selected: usize,
    },
    Scale {
        id: Option<String>,
        mode: String,
        min: f32,
        max: f32,
        value: f32,
        major_ticks: u8,
        minor_ticks: u8,
    },
    Spinbox {
        id: Option<String>,
        min: i32,
        max: i32,
        value: i32,
        digits: u8,
        decimals: u8,
    },
    Table {
        id: Option<String>,
        headers: Option<Vec<String>>,
        rows: Vec<Vec<String>>,
    },
    ProgressBar {
        id: Option<String>,
        value: f32,
    },
    SweepingArc {
        id: Option<String>,
        start_angle: i16,
        end_angle: i16,
    },
    BusyWheel {
        id: Option<String>,
        active: bool,
    },
    Plotter {
        id: Option<String>,
        mode: String,
    },
    StatusBar {
        id: Option<String>,
        time: String,
    },
    TimePicker {
        id: Option<String>,
        hour: u8,
        minute: u8,
        is_12h: bool,
        is_pm: bool,
    },
    NumberPicker {
        id: Option<String>,
        min: i32,
        max: i32,
        value: i32,
        unit: String,
    },
    Dialog {
        id: Option<String>,
        title: String,
        message: String,
        dialog_type: String,
    },
    ContentIndicator {
        id: Option<String>,
        count: u8,
        active: u8,
    },
    CrumbsIndicator {
        id: Option<String>,
        count: u8,
        active: u8,
    },
    Panel {
        id: Option<String>,
        style: Option<String>,
    },
    Image {
        id: Option<String>,
        source: String,
        fit: String,
        mode: String,
        tint: Option<String>,
    },
    /// Wrap-around scrolling list with slot falloff, edge fade, and chrome masks.
    Carousel {
        id: Option<String>,
        items: Vec<String>,
        selected: usize,
        item_step: u16,
        visible: u8,
        shift: i16,
        mask_top: u16,
        mask_bottom: u16,
        fade: bool,
        indicator: bool,
        pulse: u8,
        style: Option<String>,
        font: Option<String>,
    },
    /// Stacked 1bpp bitmap parts, each independently toggled and tinted.
    CompositeIcon {
        id: Option<String>,
        parts: Vec<IconPartDef>,
        scale: u8,
        align: String,
        tint: Option<String>,
        threshold: u8,
        invert: bool,
    },
    /// A mesh rendered through embedded-3dgfx inside the widget rect.
    Mesh3d {
        id: Option<String>,
        source: String,
        shading: String,
        color: Option<String>,
        scale: f32,
        roll: f32,
        pitch: f32,
        yaw: f32,
        camera_distance: f32,
        fov: f32,
    },
    Spacer,
    VectorPath {
        id: Option<String>,
        stroke_width: u8,
        verbs: Vec<PathVerbDef>,
    },
    RectShape {
        id: Option<String>,
        radius: u8,
        stroke_width: u8,
        fill_color: Option<String>,
        stroke_color: Option<String>,
    },
    LineShape {
        id: Option<String>,
        stroke_width: u8,
        color: Option<String>,
    },
    CircleShape {
        id: Option<String>,
        radius: u16,
        stroke_width: u8,
        fill_color: Option<String>,
        stroke_color: Option<String>,
    },
}

/// One layer of a [`WidgetDef::CompositeIcon`].
#[derive(Clone, Debug, PartialEq)]
pub struct IconPartDef {
    pub source: String,
    pub dx: i32,
    pub dy: i32,
    pub visible: bool,
    pub tint: Option<String>,
}

/// A font imported by a screen, resolved to bitmap data by a project-aware
/// caller such as `include_gui!`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontAssetDef {
    /// Name referenced by `font=` on widgets.
    pub name: String,
    pub source: String,
    /// Characters to embed; empty means the font's full range.
    pub chars: String,
}

/// RGB565 image data resolved by a project-aware caller such as `include_gui!`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageAssetDef {
    pub source: String,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u16>,
}

impl WidgetDef {
    pub fn id(&self) -> Option<&str> {
        match self {
            WidgetDef::Label { id, .. }
            | WidgetDef::Button { id, .. }
            | WidgetDef::Toggle { id, .. }
            | WidgetDef::Checkbox { id, .. }
            | WidgetDef::Slider { id, .. }
            | WidgetDef::Dropdown { id, .. }
            | WidgetDef::Roller { id, .. }
            | WidgetDef::Scale { id, .. }
            | WidgetDef::Spinbox { id, .. }
            | WidgetDef::Table { id, .. }
            | WidgetDef::ProgressBar { id, .. }
            | WidgetDef::SweepingArc { id, .. }
            | WidgetDef::BusyWheel { id, .. }
            | WidgetDef::Plotter { id, .. }
            | WidgetDef::StatusBar { id, .. }
            | WidgetDef::TimePicker { id, .. }
            | WidgetDef::NumberPicker { id, .. }
            | WidgetDef::Dialog { id, .. }
            | WidgetDef::ContentIndicator { id, .. }
            | WidgetDef::CrumbsIndicator { id, .. }
            | WidgetDef::Panel { id, .. }
            | WidgetDef::Image { id, .. }
            | WidgetDef::Carousel { id, .. }
            | WidgetDef::CompositeIcon { id, .. }
            | WidgetDef::Mesh3d { id, .. }
            | WidgetDef::VectorPath { id, .. }
            | WidgetDef::RectShape { id, .. }
            | WidgetDef::LineShape { id, .. }
            | WidgetDef::CircleShape { id, .. } => id.as_deref(),
            WidgetDef::Spacer => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridLayoutDef {
    pub id: Option<String>,
    pub cols: Vec<GridTrackDef>,
    pub rows: Vec<GridTrackDef>,
    pub gap: u16,
    pub padding: u16,
    pub children: Vec<(GridPlacementDef, WidgetDef)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenDef {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub theme: Option<String>,
    /// Transition used by project navigation when this screen is the destination.
    pub transition: Option<ScreenTransitionDef>,
    /// Fonts imported by this screen, referenced by `font=` on widgets.
    pub fonts: Vec<FontAssetDef>,
    pub grid: GridLayoutDef,
}

/// Parses track strings like `"140px 1fr 2fr auto"` or `"140px, 1fr, 48px"`.
pub fn parse_tracks(spec: &str) -> Result<Vec<GridTrackDef>, CodegenError> {
    let mut tracks = Vec::new();
    let tokens = spec.split(|c: char| c.is_whitespace() || c == ',');
    for token in tokens {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(px_str) = t.strip_suffix("px") {
            let px = px_str
                .parse::<u32>()
                .map_err(|_| CodegenError::InvalidTrack(format!("Invalid pixel track: '{}'", t)))?;
            tracks.push(GridTrackDef::Px(px));
        } else if let Some(fr_str) = t.strip_suffix("fr") {
            let fr = fr_str.parse::<u8>().map_err(|_| {
                CodegenError::InvalidTrack(format!("Invalid fractional track: '{}'", t))
            })?;
            tracks.push(GridTrackDef::Fr(fr));
        } else if t.eq_ignore_ascii_case("auto") {
            tracks.push(GridTrackDef::Auto);
        } else if let Ok(px) = t.parse::<u32>() {
            tracks.push(GridTrackDef::Px(px));
        } else {
            return Err(CodegenError::InvalidTrack(format!(
                "Unrecognized track format: '{}'",
                t
            )));
        }
    }
    if tracks.is_empty() {
        tracks.push(GridTrackDef::Fr(1));
    }
    Ok(tracks)
}

fn entry_to_str(e: &kdl::KdlEntry) -> Option<&str> {
    match e.value() {
        KdlValue::String(s) | KdlValue::RawString(s) => Some(s.as_str()),
        _ => None,
    }
}

fn get_string_prop<'a>(node: &'a KdlNode, name: &str) -> Option<&'a str> {
    node.get(name).and_then(entry_to_str)
}

fn get_i64_prop(node: &KdlNode, name: &str) -> Option<i64> {
    node.get(name).and_then(|e| match e.value() {
        KdlValue::Base10(i) | KdlValue::Base2(i) | KdlValue::Base8(i) | KdlValue::Base16(i) => {
            Some(*i)
        }
        _ => None,
    })
}

fn get_f64_prop(node: &KdlNode, name: &str) -> Option<f64> {
    node.get(name).and_then(|e| match e.value() {
        KdlValue::Base10Float(f) => Some(*f),
        KdlValue::Base10(i) => Some(*i as f64),
        _ => None,
    })
}

fn get_bool_prop(node: &KdlNode, name: &str) -> Option<bool> {
    node.get(name).and_then(|e| match e.value() {
        KdlValue::Bool(b) => Some(*b),
        _ => None,
    })
}

/// Parses an SVG path definition string (e.g. `M 10 20 C 15 25 30 40 50 20 Z` or `M0,0 L20,20 Z`) into PathVerbDefs.
pub fn parse_svg_path_d(d: &str) -> Vec<PathVerbDef> {
    let mut verbs = Vec::new();
    let mut cur_x = 0i32;
    let mut cur_y = 0i32;
    let mut chars = d.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() || c == ',' {
            chars.next();
            continue;
        }

        if c.is_ascii_alphabetic() {
            let cmd = chars.next().unwrap();
            let is_relative = cmd.is_ascii_lowercase();

            // Extract numeric coordinates following the command
            let mut numbers = Vec::new();
            while let Some(&next_c) = chars.peek() {
                if next_c.is_whitespace() || next_c == ',' {
                    chars.next();
                    continue;
                }
                if next_c.is_ascii_digit() || next_c == '-' || next_c == '+' || next_c == '.' {
                    let mut num_str = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc.is_ascii_digit() || nc == '-' || nc == '+' || nc == '.' {
                            num_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if let Ok(val) = num_str.parse::<f32>() {
                        numbers.push(val.round() as i32);
                    }
                } else {
                    break;
                }
            }

            match cmd.to_ascii_uppercase() {
                'M' => {
                    let mut idx = 0;
                    while idx + 1 < numbers.len() {
                        let nx = if is_relative {
                            cur_x + numbers[idx]
                        } else {
                            numbers[idx]
                        };
                        let ny = if is_relative {
                            cur_y + numbers[idx + 1]
                        } else {
                            numbers[idx + 1]
                        };
                        cur_x = nx;
                        cur_y = ny;
                        if idx == 0 {
                            verbs.push(PathVerbDef::MoveTo(cur_x, cur_y));
                        } else {
                            verbs.push(PathVerbDef::LineTo(cur_x, cur_y));
                        }
                        idx += 2;
                    }
                }
                'L' => {
                    let mut idx = 0;
                    while idx + 1 < numbers.len() {
                        let nx = if is_relative {
                            cur_x + numbers[idx]
                        } else {
                            numbers[idx]
                        };
                        let ny = if is_relative {
                            cur_y + numbers[idx + 1]
                        } else {
                            numbers[idx + 1]
                        };
                        cur_x = nx;
                        cur_y = ny;
                        verbs.push(PathVerbDef::LineTo(cur_x, cur_y));
                        idx += 2;
                    }
                }
                'H' => {
                    for x in numbers {
                        let nx = if is_relative { cur_x + x } else { x };
                        cur_x = nx;
                        verbs.push(PathVerbDef::LineTo(cur_x, cur_y));
                    }
                }
                'V' => {
                    for y in numbers {
                        let ny = if is_relative { cur_y + y } else { y };
                        cur_y = ny;
                        verbs.push(PathVerbDef::LineTo(cur_x, cur_y));
                    }
                }
                'Q' => {
                    let mut idx = 0;
                    while idx + 3 < numbers.len() {
                        let cx = if is_relative {
                            cur_x + numbers[idx]
                        } else {
                            numbers[idx]
                        };
                        let cy = if is_relative {
                            cur_y + numbers[idx + 1]
                        } else {
                            numbers[idx + 1]
                        };
                        let ex = if is_relative {
                            cur_x + numbers[idx + 2]
                        } else {
                            numbers[idx + 2]
                        };
                        let ey = if is_relative {
                            cur_y + numbers[idx + 3]
                        } else {
                            numbers[idx + 3]
                        };
                        verbs.push(PathVerbDef::QuadTo(cx, cy, ex, ey));
                        cur_x = ex;
                        cur_y = ey;
                        idx += 4;
                    }
                }
                'C' => {
                    let mut idx = 0;
                    while idx + 5 < numbers.len() {
                        let c1x = if is_relative {
                            cur_x + numbers[idx]
                        } else {
                            numbers[idx]
                        };
                        let c1y = if is_relative {
                            cur_y + numbers[idx + 1]
                        } else {
                            numbers[idx + 1]
                        };
                        let c2x = if is_relative {
                            cur_x + numbers[idx + 2]
                        } else {
                            numbers[idx + 2]
                        };
                        let c2y = if is_relative {
                            cur_y + numbers[idx + 3]
                        } else {
                            numbers[idx + 3]
                        };
                        let ex = if is_relative {
                            cur_x + numbers[idx + 4]
                        } else {
                            numbers[idx + 4]
                        };
                        let ey = if is_relative {
                            cur_y + numbers[idx + 5]
                        } else {
                            numbers[idx + 5]
                        };
                        verbs.push(PathVerbDef::CubicTo(c1x, c1y, c2x, c2y, ex, ey));
                        cur_x = ex;
                        cur_y = ey;
                        idx += 6;
                    }
                }
                'Z' => {
                    verbs.push(PathVerbDef::Close);
                }
                _ => {}
            }
        } else {
            chars.next();
        }
    }

    verbs
}

fn parse_path_verbs(node: &KdlNode) -> Vec<PathVerbDef> {
    if let Some(d_str) = get_string_prop(node, "d").or_else(|| get_string_prop(node, "data")) {
        return parse_svg_path_d(d_str);
    }

    let mut verbs = Vec::new();
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            let args: Vec<i32> = child
                .entries()
                .iter()
                .filter_map(|e| match e.value() {
                    KdlValue::Base10(i) => Some(*i as i32),
                    KdlValue::Base10Float(f) => Some(*f as i32),
                    _ => None,
                })
                .collect();

            match name {
                "move_to" | "move" if args.len() >= 2 => {
                    verbs.push(PathVerbDef::MoveTo(args[0], args[1]));
                }
                "line_to" | "line" if args.len() >= 2 => {
                    verbs.push(PathVerbDef::LineTo(args[0], args[1]));
                }
                "quad_to" | "quad" if args.len() >= 4 => {
                    verbs.push(PathVerbDef::QuadTo(args[0], args[1], args[2], args[3]));
                }
                "cubic_to" | "cubic" if args.len() >= 6 => {
                    verbs.push(PathVerbDef::CubicTo(
                        args[0], args[1], args[2], args[3], args[4], args[5],
                    ));
                }
                "close" => {
                    verbs.push(PathVerbDef::Close);
                }
                _ => {}
            }
        }
    }
    verbs
}

/// Parses a single widget KDL node.
pub fn parse_widget(node: &KdlNode) -> Result<(GridPlacementDef, WidgetDef), CodegenError> {
    let tag = node.name().value();
    let col = get_i64_prop(node, "col").unwrap_or(0).max(0) as usize;
    let row = get_i64_prop(node, "row").unwrap_or(0).max(0) as usize;
    let col_span = get_i64_prop(node, "col_span")
        .or_else(|| get_i64_prop(node, "colSpan"))
        .unwrap_or(1)
        .max(1) as usize;
    let row_span = get_i64_prop(node, "row_span")
        .or_else(|| get_i64_prop(node, "rowSpan"))
        .unwrap_or(1)
        .max(1) as usize;
    let animation = get_string_prop(node, "animation").map(|preset| WidgetAnimationDef {
        preset: preset.to_string(),
        trigger: get_string_prop(node, "animation_trigger")
            .unwrap_or("screen_enter")
            .to_string(),
        duration_ms: get_i64_prop(node, "animation_duration")
            .unwrap_or(400)
            .clamp(1, u32::MAX as i64) as u32,
        delay_ms: get_i64_prop(node, "animation_delay")
            .unwrap_or(0)
            .clamp(0, u32::MAX as i64) as u32,
        easing: get_string_prop(node, "animation_easing")
            .unwrap_or("out_cubic")
            .to_string(),
        repeat: get_i64_prop(node, "animation_repeat")
            .unwrap_or(1)
            .clamp(0, u16::MAX as i64) as u16,
    });
    let placement = GridPlacementDef {
        col,
        row,
        col_span,
        row_span,
        animation,
    };

    let id = get_string_prop(node, "id").map(|s| s.to_string());
    let style = get_string_prop(node, "style").map(|s| s.to_string());

    let widget = match tag {
        "label" | "banner" => {
            let text = get_string_prop(node, "text")
                .or_else(|| node.entries().first().and_then(entry_to_str))
                .unwrap_or("")
                .to_string();
            let font = get_string_prop(node, "font").map(|s| s.to_string());
            WidgetDef::Label {
                id,
                text,
                style,
                font,
            }
        }
        "button" => {
            let text = get_string_prop(node, "text")
                .or_else(|| node.entries().first().and_then(entry_to_str))
                .unwrap_or("Button")
                .to_string();
            let on_click = get_string_prop(node, "on_click")
                .or_else(|| get_string_prop(node, "onClick"))
                .map(|s| s.to_string());
            WidgetDef::Button {
                id,
                text,
                on_click,
                style,
            }
        }
        "toggle" => {
            let label = get_string_prop(node, "label")
                .or_else(|| get_string_prop(node, "text"))
                .unwrap_or("")
                .to_string();
            let checked = get_bool_prop(node, "checked").unwrap_or(false);
            WidgetDef::Toggle { id, label, checked }
        }
        "checkbox" => {
            let label = get_string_prop(node, "label")
                .or_else(|| get_string_prop(node, "text"))
                .unwrap_or("")
                .to_string();
            let checked = get_bool_prop(node, "checked").unwrap_or(false);
            WidgetDef::Checkbox { id, label, checked }
        }
        "slider" => {
            let min = get_i64_prop(node, "min").unwrap_or(0) as i32;
            let max = get_i64_prop(node, "max").unwrap_or(100) as i32;
            let value = get_i64_prop(node, "value").unwrap_or(min as i64) as i32;
            WidgetDef::Slider {
                id,
                min,
                max,
                value,
            }
        }
        "dropdown" => {
            let mut options = Vec::new();
            if let Some(children) = node.children() {
                for child in children.nodes() {
                    if child.name().value() == "option" {
                        if let Some(opt_text) = child.entries().first().and_then(entry_to_str) {
                            options.push(opt_text.to_string());
                        }
                    }
                }
            }
            let selected = get_i64_prop(node, "selected").unwrap_or(0).max(0) as usize;
            WidgetDef::Dropdown {
                id,
                options,
                selected,
            }
        }
        "roller" => {
            let mut options = Vec::new();
            if let Some(children) = node.children() {
                for child in children.nodes() {
                    if child.name().value() == "option" {
                        if let Some(opt_text) = child.entries().first().and_then(entry_to_str) {
                            options.push(opt_text.to_string());
                        }
                    }
                }
            }
            let selected = get_i64_prop(node, "selected").unwrap_or(0).max(0) as usize;
            WidgetDef::Roller {
                id,
                options,
                selected,
            }
        }
        "scale" | "gauge" => {
            let mode = get_string_prop(node, "mode")
                .unwrap_or("radial")
                .to_string();
            let min = get_f64_prop(node, "min").unwrap_or(0.0) as f32;
            let max = get_f64_prop(node, "max").unwrap_or(100.0) as f32;
            let value = get_f64_prop(node, "value").unwrap_or(min as f64) as f32;
            let major_ticks = get_i64_prop(node, "major_ticks").unwrap_or(5).max(1) as u8;
            let minor_ticks = get_i64_prop(node, "minor_ticks").unwrap_or(2).max(1) as u8;
            WidgetDef::Scale {
                id,
                mode,
                min,
                max,
                value,
                major_ticks,
                minor_ticks,
            }
        }
        "spinbox" => {
            let min = get_i64_prop(node, "min").unwrap_or(0) as i32;
            let max = get_i64_prop(node, "max").unwrap_or(9999) as i32;
            let value = get_i64_prop(node, "value").unwrap_or(min as i64) as i32;
            let digits = get_i64_prop(node, "digits").unwrap_or(4).max(1) as u8;
            let decimals = get_i64_prop(node, "decimals").unwrap_or(0) as u8;
            WidgetDef::Spinbox {
                id,
                min,
                max,
                value,
                digits,
                decimals,
            }
        }
        "progress" | "progress_bar" => {
            let value = get_f64_prop(node, "value").unwrap_or(0.0) as f32;
            WidgetDef::ProgressBar { id, value }
        }
        "sweeping_arc" | "arc" => {
            let start_angle = get_i64_prop(node, "start_angle").unwrap_or(0) as i16;
            let end_angle = get_i64_prop(node, "end_angle").unwrap_or(270) as i16;
            WidgetDef::SweepingArc {
                id,
                start_angle,
                end_angle,
            }
        }
        "busy_wheel" | "spinner" => {
            let active = get_bool_prop(node, "active").unwrap_or(true);
            WidgetDef::BusyWheel { id, active }
        }
        "plotter" | "chart" => {
            let mode = get_string_prop(node, "mode").unwrap_or("line").to_string();
            WidgetDef::Plotter { id, mode }
        }
        "status_bar" => {
            let time = get_string_prop(node, "time").unwrap_or("12:00").to_string();
            WidgetDef::StatusBar { id, time }
        }
        "time_picker" => {
            let hour = get_i64_prop(node, "hour").unwrap_or(12).clamp(0, 23) as u8;
            let minute = get_i64_prop(node, "minute").unwrap_or(0).clamp(0, 59) as u8;
            let is_12h = get_bool_prop(node, "is_12h").unwrap_or(true);
            let is_pm = get_bool_prop(node, "is_pm").unwrap_or(false);
            WidgetDef::TimePicker {
                id,
                hour,
                minute,
                is_12h,
                is_pm,
            }
        }
        "number_picker" => {
            let min = get_i64_prop(node, "min").unwrap_or(0) as i32;
            let max = get_i64_prop(node, "max").unwrap_or(100) as i32;
            let value = get_i64_prop(node, "value").unwrap_or(min as i64) as i32;
            let unit = get_string_prop(node, "unit").unwrap_or("").to_string();
            WidgetDef::NumberPicker {
                id,
                min,
                max,
                value,
                unit,
            }
        }
        "dialog" => {
            let title = get_string_prop(node, "title")
                .unwrap_or("Alert")
                .to_string();
            let message = get_string_prop(node, "message").unwrap_or("").to_string();
            let dialog_type = get_string_prop(node, "type").unwrap_or("info").to_string();
            WidgetDef::Dialog {
                id,
                title,
                message,
                dialog_type,
            }
        }
        "content_indicator" => {
            let count = get_i64_prop(node, "count").unwrap_or(3).max(1) as u8;
            let active = get_i64_prop(node, "active").unwrap_or(0) as u8;
            WidgetDef::ContentIndicator { id, count, active }
        }
        "crumbs" | "crumbs_indicator" => {
            let count = get_i64_prop(node, "count").unwrap_or(3).max(1) as u8;
            let active = get_i64_prop(node, "active").unwrap_or(0) as u8;
            WidgetDef::CrumbsIndicator { id, count, active }
        }
        "panel" | "card" => WidgetDef::Panel { id, style },
        "image" | "bitmap" => {
            let source = get_string_prop(node, "src")
                .or_else(|| get_string_prop(node, "source"))
                .ok_or_else(|| CodegenError::MissingAttribute("image", "src".into()))?
                .to_string();
            let fit = get_string_prop(node, "fit")
                .unwrap_or("stretch")
                .to_string();
            let mode = get_string_prop(node, "mode").unwrap_or("color").to_string();
            let tint = get_string_prop(node, "tint").map(str::to_string);
            WidgetDef::Image {
                id,
                source,
                fit,
                mode,
                tint,
            }
        }
        "carousel" => {
            let mut items = Vec::new();
            if let Some(children) = node.children() {
                for child in children.nodes() {
                    if matches!(child.name().value(), "option" | "item") {
                        if let Some(text) = child.entries().first().and_then(entry_to_str) {
                            items.push(text.to_string());
                        }
                    }
                }
            }
            WidgetDef::Carousel {
                id,
                items,
                selected: get_i64_prop(node, "selected").unwrap_or(0).max(0) as usize,
                item_step: get_i64_prop(node, "item_step").unwrap_or(16).clamp(1, 255) as u16,
                visible: get_i64_prop(node, "visible").unwrap_or(7).clamp(1, 31) as u8,
                shift: get_i64_prop(node, "shift").unwrap_or(0).clamp(-4096, 4096) as i16,
                mask_top: get_i64_prop(node, "mask_top").unwrap_or(0).clamp(0, 4096) as u16,
                mask_bottom: get_i64_prop(node, "mask_bottom")
                    .unwrap_or(0)
                    .clamp(0, 4096) as u16,
                fade: get_bool_prop(node, "fade").unwrap_or(true),
                indicator: get_bool_prop(node, "indicator").unwrap_or(false),
                pulse: get_i64_prop(node, "pulse").unwrap_or(255).clamp(0, 255) as u8,
                style,
                font: get_string_prop(node, "font").map(str::to_string),
            }
        }
        "icon" | "composite_icon" => {
            let mut parts = Vec::new();
            if let Some(children) = node.children() {
                for child in children.nodes() {
                    if child.name().value() != "part" {
                        continue;
                    }
                    let source = get_string_prop(child, "src")
                        .or_else(|| get_string_prop(child, "source"))
                        .or_else(|| child.entries().first().and_then(entry_to_str))
                        .ok_or_else(|| CodegenError::MissingAttribute("part", "src".into()))?
                        .to_string();
                    parts.push(IconPartDef {
                        source,
                        dx: get_i64_prop(child, "x").unwrap_or(0) as i32,
                        dy: get_i64_prop(child, "y").unwrap_or(0) as i32,
                        visible: get_bool_prop(child, "visible").unwrap_or(true),
                        tint: get_string_prop(child, "tint").map(str::to_string),
                    });
                }
            }
            if parts.is_empty() {
                return Err(CodegenError::MissingAttribute(
                    "icon",
                    "at least one 'part' child".into(),
                ));
            }
            WidgetDef::CompositeIcon {
                id,
                parts,
                scale: get_i64_prop(node, "scale").unwrap_or(1).clamp(1, 16) as u8,
                align: get_string_prop(node, "align")
                    .unwrap_or("center")
                    .to_string(),
                tint: get_string_prop(node, "tint").map(str::to_string),
                threshold: get_i64_prop(node, "threshold").unwrap_or(128).clamp(0, 255) as u8,
                invert: get_bool_prop(node, "invert").unwrap_or(false),
            }
        }
        "mesh" | "mesh3d" => {
            let source = get_string_prop(node, "src")
                .or_else(|| get_string_prop(node, "source"))
                .ok_or_else(|| CodegenError::MissingAttribute("mesh", "src".into()))?
                .to_string();
            WidgetDef::Mesh3d {
                id,
                source,
                shading: get_string_prop(node, "shading")
                    .unwrap_or("solid")
                    .to_string(),
                color: get_string_prop(node, "color").map(str::to_string),
                scale: get_f64_prop(node, "scale").unwrap_or(1.0) as f32,
                roll: get_f64_prop(node, "roll").unwrap_or(0.0) as f32,
                pitch: get_f64_prop(node, "pitch").unwrap_or(0.0) as f32,
                yaw: get_f64_prop(node, "yaw").unwrap_or(0.0) as f32,
                camera_distance: get_f64_prop(node, "camera_distance").unwrap_or(4.0) as f32,
                fov: get_f64_prop(node, "fov").unwrap_or(1.5707964) as f32,
            }
        }
        "spacer" => WidgetDef::Spacer,
        "vector_path" | "path" => {
            let stroke_width = get_i64_prop(node, "stroke_width").unwrap_or(2).max(1) as u8;
            let verbs = parse_path_verbs(node);
            WidgetDef::VectorPath {
                id,
                stroke_width,
                verbs,
            }
        }
        "rect" | "rectangle" => {
            let radius = get_i64_prop(node, "radius").unwrap_or(0).max(0) as u8;
            let stroke_width = get_i64_prop(node, "stroke_width").unwrap_or(1).max(0) as u8;
            let fill_color = get_string_prop(node, "fill").map(|s| s.to_string());
            let stroke_color = get_string_prop(node, "stroke").map(|s| s.to_string());
            WidgetDef::RectShape {
                id,
                radius,
                stroke_width,
                fill_color,
                stroke_color,
            }
        }
        "line" => {
            let stroke_width = get_i64_prop(node, "stroke_width").unwrap_or(1).max(1) as u8;
            let color = get_string_prop(node, "color").map(|s| s.to_string());
            WidgetDef::LineShape {
                id,
                stroke_width,
                color,
            }
        }
        "circle" => {
            let radius = get_i64_prop(node, "radius").unwrap_or(10).max(1) as u16;
            let stroke_width = get_i64_prop(node, "stroke_width").unwrap_or(1).max(0) as u8;
            let fill_color = get_string_prop(node, "fill").map(|s| s.to_string());
            let stroke_color = get_string_prop(node, "stroke").map(|s| s.to_string());
            WidgetDef::CircleShape {
                id,
                radius,
                stroke_width,
                fill_color,
                stroke_color,
            }
        }
        "table" => {
            let mut headers = None;
            let mut rows = Vec::new();
            if let Some(children) = node.children() {
                for child in children.nodes() {
                    match child.name().value() {
                        "headers" | "header" => {
                            let cols: Vec<String> = child
                                .entries()
                                .iter()
                                .filter_map(entry_to_str)
                                .map(|s| s.to_string())
                                .collect();
                            if !cols.is_empty() {
                                headers = Some(cols);
                            }
                        }
                        "row" => {
                            let cols: Vec<String> = child
                                .entries()
                                .iter()
                                .filter_map(entry_to_str)
                                .map(|s| s.to_string())
                                .collect();
                            rows.push(cols);
                        }
                        _ => {}
                    }
                }
            }
            WidgetDef::Table { id, headers, rows }
        }
        other => return Err(CodegenError::UnknownNode(other.to_string())),
    };

    Ok((placement, widget))
}

/// Parses a complete KDL document containing a `screen` definition.
pub fn parse_kdl_screen(kdl_source: &str) -> Result<ScreenDef, CodegenError> {
    let doc: KdlDocument = kdl_source.parse()?;
    let screen_node = doc
        .nodes()
        .iter()
        .find(|n| n.name().value() == "screen")
        .ok_or_else(|| {
            CodegenError::MissingAttribute(
                "screen",
                "Document must contain a root 'screen' node".into(),
            )
        })?;

    let id = get_string_prop(screen_node, "id")
        .or_else(|| screen_node.entries().first().and_then(entry_to_str))
        .unwrap_or("MainScreen")
        .to_string();
    let width = get_i64_prop(screen_node, "width").unwrap_or(320).max(1) as u32;
    let height = get_i64_prop(screen_node, "height").unwrap_or(240).max(1) as u32;
    let theme = get_string_prop(screen_node, "theme").map(|s| s.to_string());
    let transition = get_string_prop(screen_node, "transition").map(|preset| ScreenTransitionDef {
        preset: preset.to_string(),
        duration_ms: get_i64_prop(screen_node, "transition_duration")
            .unwrap_or(300)
            .clamp(1, u32::MAX as i64) as u32,
        easing: get_string_prop(screen_node, "transition_easing")
            .unwrap_or("in_out_sine")
            .to_string(),
        origin: get_string_prop(screen_node, "transition_origin")
            .unwrap_or("center")
            .to_string(),
    });

    let mut fonts = Vec::new();
    if let Some(children) = screen_node.children() {
        for child in children.nodes() {
            if child.name().value() != "font" {
                continue;
            }
            let name = get_string_prop(child, "id")
                .or_else(|| get_string_prop(child, "name"))
                .or_else(|| child.entries().first().and_then(entry_to_str))
                .ok_or_else(|| CodegenError::MissingAttribute("font", "id".into()))?
                .to_string();
            let source = get_string_prop(child, "src")
                .or_else(|| get_string_prop(child, "source"))
                .ok_or_else(|| CodegenError::MissingAttribute("font", "src".into()))?
                .to_string();
            fonts.push(FontAssetDef {
                name,
                source,
                chars: get_string_prop(child, "chars").unwrap_or("").to_string(),
            });
        }
    }

    let grid_node = screen_node
        .children()
        .and_then(|c| c.nodes().iter().find(|n| n.name().value() == "grid"))
        .ok_or_else(|| {
            CodegenError::MissingAttribute("grid", "screen node must contain a 'grid' child".into())
        })?;

    let cols_str = get_string_prop(grid_node, "cols").unwrap_or("1fr");
    let rows_str = get_string_prop(grid_node, "rows").unwrap_or("1fr");
    let col_tracks = parse_tracks(cols_str)?;
    let row_tracks = parse_tracks(rows_str)?;
    let gap = get_i64_prop(grid_node, "gap").unwrap_or(4).max(0) as u16;
    let padding = get_i64_prop(grid_node, "padding").unwrap_or(4).max(0) as u16;

    let mut children = Vec::new();
    if let Some(grid_children) = grid_node.children() {
        for child in grid_children.nodes() {
            let (placement, widget) = parse_widget(child)?;
            children.push((placement, widget));
        }
    }

    let grid = GridLayoutDef {
        id: get_string_prop(grid_node, "id").map(|s| s.to_string()),
        cols: col_tracks,
        rows: row_tracks,
        gap,
        padding,
        children,
    };

    Ok(ScreenDef {
        id,
        width,
        height,
        theme,
        transition,
        fonts,
        grid,
    })
}

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' {
            out.push('_');
        } else {
            out.push(c);
        }
    }
    out
}

fn style_expr(style: Option<&str>, default: &str) -> String {
    match style {
        None => default.to_string(),
        Some(s) if s.starts_with("Style::") => s.to_string(),
        Some("default") | Some("label") | Some("bold") | Some("dim") => "Style::label()".into(),
        Some("body") | Some("body-accent") | Some("body-success") | Some("body-danger")
        | Some("body-dim") | Some("menu") => {
            "{ let mut style = Style::label(); style.font = FontId::Scaled6x10; style }".into()
        }
        Some("hint") | Some("hint-dim") | Some("hint-accent") => {
            "{ let mut style = Style::label(); style.font = FontId::Medium4x7; style }".into()
        }
        Some("button") => "Style::button()".into(),
        Some("panel") | Some("card") => "Style::panel()".into(),
        Some("progress") => "Style::progress()".into(),
        // Semantic tokens are palette concerns in Studio; firmware codegen maps
        // them onto the closest stock Style constructor.
        Some("accent") | Some("success") | Some("danger") | Some("inverted") => {
            "Style::label()".into()
        }
        Some(_) => default.to_string(),
    }
}

/// Generates zero-allocation `#![no_std]` Rust code from a parsed `ScreenDef`.
pub fn generate_rust_code(screen: &ScreenDef) -> String {
    let mut out = String::new();

    let screen_name = &screen.id;
    let widget_struct_name = format!("{}Widgets", screen_name);
    let app_struct_name = format!("{}App", screen_name);
    let num_cols = screen.grid.cols.len();
    let num_rows = screen.grid.rows.len();
    let total_nodes = screen.grid.children.len();

    // File header
    let _ = writeln!(
        &mut out,
        "// Auto-generated by embedded-gui-codegen. DO NOT EDIT."
    );
    let _ = writeln!(&mut out, "use embedded_gui::prelude::*;");
    let _ = writeln!(&mut out);

    // Struct containing all named widget IDs
    let _ = writeln!(
        &mut out,
        "/// Strongly-typed widget IDs for {}.",
        screen_name
    );
    let _ = writeln!(&mut out, "#[derive(Clone, Copy, Debug)]");
    let _ = writeln!(&mut out, "pub struct {} {{", widget_struct_name);
    for (_, w) in &screen.grid.children {
        if let Some(id) = w.id() {
            let field_name = to_snake_case(id);
            let _ = writeln!(&mut out, "    pub {}: WidgetId,", field_name);
        }
    }
    let _ = writeln!(&mut out, "}}");
    let _ = writeln!(&mut out);

    // App struct
    let _ = writeln!(&mut out, "pub struct {} {{", app_struct_name);
    let _ = writeln!(&mut out, "    pub widgets: {},", widget_struct_name);
    let _ = writeln!(&mut out, "}}");
    let _ = writeln!(&mut out);

    // Impl block with build method
    let _ = writeln!(&mut out, "impl {} {{", app_struct_name);
    let _ = writeln!(&mut out, "    pub const WIDTH: u32 = {};", screen.width);
    let _ = writeln!(&mut out, "    pub const HEIGHT: u32 = {};", screen.height);
    if let Some(transition) = &screen.transition {
        let _ = writeln!(
            &mut out,
            "    pub const TRANSITION: ScreenTransitionSpec = ScreenTransitionSpec {{ effect: {}, duration_ms: {}, origin: {}, easing: {} }};",
            transition_effect_expr(&transition.preset),
            transition.duration_ms,
            transition_origin_expr(&transition.origin),
            easing_expr(&transition.easing),
        );
    }
    let _ = writeln!(
        &mut out,
        "    pub const NODE_COUNT: usize = {};",
        total_nodes
    );
    let _ = writeln!(&mut out);
    let _ = writeln!(
        &mut out,
        "    pub fn build<'a, const N: usize, const E: usize, const D: usize>("
    );
    let _ = writeln!(&mut out, "        gui: &mut GuiContext<'a, N, E, D>,");
    let _ = writeln!(&mut out, "    ) -> Result<Self, GuiError> {{");

    // Track arrays
    let _ = writeln!(&mut out, "        let col_tracks = [");
    for t in &screen.grid.cols {
        match t {
            GridTrackDef::Px(px) => {
                let _ = writeln!(&mut out, "            GridTrack::Px({}),", px);
            }
            GridTrackDef::Fr(fr) => {
                let _ = writeln!(&mut out, "            GridTrack::Fr({}),", fr);
            }
            GridTrackDef::Auto => {
                let _ = writeln!(&mut out, "            GridTrack::Auto,");
            }
        }
    }
    let _ = writeln!(&mut out, "        ];");

    let _ = writeln!(&mut out, "        let row_tracks = [");
    for t in &screen.grid.rows {
        match t {
            GridTrackDef::Px(px) => {
                let _ = writeln!(&mut out, "            GridTrack::Px({}),", px);
            }
            GridTrackDef::Fr(fr) => {
                let _ = writeln!(&mut out, "            GridTrack::Fr({}),", fr);
            }
            GridTrackDef::Auto => {
                let _ = writeln!(&mut out, "            GridTrack::Auto,");
            }
        }
    }
    let _ = writeln!(&mut out, "        ];");

    let _ = writeln!(
        &mut out,
        "        let grid = GridLayout::<{}, {}>::new(col_tracks, row_tracks)",
        num_cols, num_rows
    );
    let _ = writeln!(&mut out, "            .with_gap({})", screen.grid.gap);
    let _ = writeln!(
        &mut out,
        "            .with_padding(EdgeInsets::all({}));",
        screen.grid.padding
    );
    let _ = writeln!(&mut out);

    // Placements
    let _ = writeln!(&mut out, "        let placements = [");
    for (p, _) in &screen.grid.children {
        if p.col_span > 1 || p.row_span > 1 {
            let _ = writeln!(
                &mut out,
                "            GridPlacement::span({}, {}, {}, {}),",
                p.col, p.row, p.col_span, p.row_span
            );
        } else {
            let _ = writeln!(
                &mut out,
                "            GridPlacement::cell({}, {}),",
                p.col, p.row
            );
        }
    }
    let _ = writeln!(&mut out, "        ];");
    let _ = writeln!(
        &mut out,
        "        let mut cells = [Rect::empty(); {}];",
        total_nodes
    );
    let _ = writeln!(
        &mut out,
        "        grid.arrange_cells(gui.viewport(), &placements, &mut cells);"
    );
    let _ = writeln!(&mut out);

    // Widget builder instantiations
    let mut named_vars = Vec::new();
    for (idx, (_, w)) in screen.grid.children.iter().enumerate() {
        let var_name = if let Some(id) = w.id() {
            let name = to_snake_case(id);
            named_vars.push(name.clone());
            name
        } else {
            format!("_w{}", idx)
        };

        match w {
            WidgetDef::Label { text, style, .. } => {
                let st = style_expr(style.as_deref(), "Style::default()");
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_label(cells[{}], \"{}\", {})?;",
                    var_name, idx, text, st
                );
            }
            WidgetDef::Button { text, style, .. } => {
                let st = style_expr(style.as_deref(), "Style::button()");
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_button(cells[{}], \"{}\", {})?;",
                    var_name, idx, text, st
                );
            }
            WidgetDef::Toggle { label, checked, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_toggle(cells[{}], \"{}\", {}, Style::default())?;",
                    var_name, idx, label, checked
                );
            }
            WidgetDef::Checkbox { label, checked, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_checkbox(cells[{}], \"{}\", {}, Style::default())?;",
                    var_name, idx, label, checked
                );
            }
            WidgetDef::Slider {
                min, max, value, ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_slider(cells[{}], {}, {}, {}, Style::default())?;",
                    var_name, idx, min, max, value
                );
            }
            WidgetDef::Dropdown {
                options, selected, ..
            } => {
                let opts_joined = options
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_dropdown(cells[{}], &[{}], {}, Style::panel())?;",
                    var_name, idx, opts_joined, selected
                );
            }
            WidgetDef::Roller {
                options, selected, ..
            } => {
                let opts_joined = options
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_roller(cells[{}], &[{}], {}, Style::panel())?;",
                    var_name, idx, opts_joined, selected
                );
            }
            WidgetDef::Scale {
                mode,
                min,
                max,
                value,
                ..
            } => {
                let scale_mode = if mode.eq_ignore_ascii_case("radial") {
                    "ScaleMode::Radial"
                } else if mode.eq_ignore_ascii_case("linear_vertical") {
                    "ScaleMode::LinearVertical"
                } else {
                    "ScaleMode::LinearHorizontal"
                };
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_scale(cells[{}], {}, {:.1}, {:.1}, {:.1}, Style::panel())?;",
                    var_name, idx, scale_mode, min, max, value
                );
            }
            WidgetDef::Spinbox {
                min, max, value, ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_spinbox(cells[{}], {}, {}, {}, Style::panel())?;",
                    var_name, idx, min, max, value
                );
            }
            WidgetDef::ProgressBar { value, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_progress_bar(cells[{}], {:.2}, Style::default())?;",
                    var_name, idx, value
                );
            }
            WidgetDef::SweepingArc {
                start_angle,
                end_angle,
                ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_sweeping_arc(cells[{}], {}, {}, Style::panel())?;",
                    var_name, idx, start_angle, end_angle
                );
            }
            WidgetDef::BusyWheel { active, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_busy_wheel(cells[{}], {}, Style::panel())?;",
                    var_name, idx, active
                );
            }
            WidgetDef::Plotter { mode, .. } => {
                let chart_mode = if mode.eq_ignore_ascii_case("bar") {
                    "ChartMode::Bar"
                } else {
                    "ChartMode::Line"
                };
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_plotter(cells[{}], {}, Style::panel())?;",
                    var_name, idx, chart_mode
                );
            }
            WidgetDef::StatusBar { time, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_status_bar(cells[{}], \"{}\", Style::panel())?;",
                    var_name, idx, time
                );
            }
            WidgetDef::TimePicker {
                hour,
                minute,
                is_12h,
                is_pm,
                ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_time_picker(cells[{}], {}, {}, {}, {}, Style::panel())?;",
                    var_name, idx, hour, minute, is_12h, is_pm
                );
            }
            WidgetDef::NumberPicker {
                min,
                max,
                value,
                unit,
                ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_number_picker(cells[{}], {}, {}, {}, \"{}\", Style::panel())?;",
                    var_name, idx, min, max, value, unit
                );
            }
            WidgetDef::Dialog { title, message, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_dialog(cells[{}], \"{}\", \"{}\", Style::panel())?;",
                    var_name, idx, title, message
                );
            }
            WidgetDef::ContentIndicator { count, active, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_content_indicator(cells[{}], {}, {}, Style::panel())?;",
                    var_name, idx, count, active
                );
            }
            WidgetDef::CrumbsIndicator { count, active, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_crumbs_indicator(cells[{}], {}, {}, Style::panel())?;",
                    var_name, idx, count, active
                );
            }
            WidgetDef::Panel { style, .. } => {
                let st = style_expr(style.as_deref(), "Style::panel()");
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_panel(cells[{}], {})?;",
                    var_name, idx, st
                );
            }
            WidgetDef::Image {
                source, fit, mode, ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_spacer(cells[{}])?; // image src={:?} fit={:?} mode={:?}; asset data is supplied by include_gui!",
                    var_name, idx, source, fit, mode
                );
            }
            WidgetDef::Spacer => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_spacer(cells[{}])?;",
                    var_name, idx
                );
            }
            WidgetDef::VectorPath {
                stroke_width,
                verbs,
                ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let mut _path_{} = VectorPath::<{}>::new();",
                    var_name,
                    verbs.len().max(1)
                );
                for v in verbs {
                    match v {
                        PathVerbDef::MoveTo(x, y) => {
                            let _ = writeln!(
                                &mut out,
                                "        _path_{}.push(PathVerb::MoveTo(Point::new({}, {})));",
                                var_name, x, y
                            );
                        }
                        PathVerbDef::LineTo(x, y) => {
                            let _ = writeln!(
                                &mut out,
                                "        _path_{}.push(PathVerb::LineTo(Point::new({}, {})));",
                                var_name, x, y
                            );
                        }
                        PathVerbDef::QuadTo(cx, cy, x, y) => {
                            let _ = writeln!(
                                &mut out,
                                "        _path_{}.push(PathVerb::QuadTo(Point::new({}, {}), Point::new({}, {})));",
                                var_name, cx, cy, x, y
                            );
                        }
                        PathVerbDef::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                            let _ = writeln!(
                                &mut out,
                                "        _path_{}.push(PathVerb::CubicTo(Point::new({}, {}), Point::new({}, {}), Point::new({}, {})));",
                                var_name, c1x, c1y, c2x, c2y, x, y
                            );
                        }
                        PathVerbDef::Close => {
                            let _ = writeln!(
                                &mut out,
                                "        _path_{}.push(PathVerb::Close);",
                                var_name
                            );
                        }
                    }
                }
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_panel(cells[{}], Style::panel())?;",
                    var_name, idx
                );
                let _ = writeln!(
                    &mut out,
                    "        let _ = {}; // stroke_width: {}",
                    var_name, stroke_width
                );
            }
            WidgetDef::RectShape {
                radius,
                stroke_width,
                ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_panel(cells[{}], Style::panel())?; // rect r={} sw={}",
                    var_name, idx, radius, stroke_width
                );
            }
            WidgetDef::LineShape { stroke_width, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_spacer(cells[{}])?; // line sw={}",
                    var_name, idx, stroke_width
                );
            }
            WidgetDef::CircleShape {
                radius,
                stroke_width,
                ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_panel(cells[{}], Style::panel())?; // circle r={} sw={}",
                    var_name, idx, radius, stroke_width
                );
            }
            WidgetDef::Table { .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_table(cells[{}], &[], Style::panel())?;",
                    var_name, idx
                );
            }
            WidgetDef::Carousel {
                items,
                selected,
                style,
                ..
            } => {
                let joined = items
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ");
                let st = style_expr(style.as_deref(), "Style::label()");
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_carousel(cells[{}], &[{}], {}, {}, {})?;",
                    var_name,
                    idx,
                    joined,
                    selected,
                    carousel_spec_expr(w),
                    st
                );
            }
            WidgetDef::CompositeIcon { parts, .. } => {
                let sources = parts
                    .iter()
                    .map(|part| part.source.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_spacer(cells[{}])?; // icon parts=[{}]; asset data is supplied by include_gui!",
                    var_name, idx, sources
                );
            }
            WidgetDef::Mesh3d { source, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_spacer(cells[{}])?; // mesh src={:?}; geometry is supplied by include_gui!",
                    var_name, idx, source
                );
            }
        }
    }

    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "        Ok(Self {{");
    let _ = writeln!(&mut out, "            widgets: {} {{", widget_struct_name);
    for name in &named_vars {
        let _ = writeln!(&mut out, "                {},", name);
    }
    let _ = writeln!(&mut out, "            }},");
    let _ = writeln!(&mut out, "        }})");
    let _ = writeln!(&mut out, "    }}");
    let _ = writeln!(&mut out);
    write_animation_method(
        &mut out,
        screen,
        "screen_enter",
        "start_screen_enter_animations",
    );
    write_animation_method(
        &mut out,
        screen,
        "screen_exit",
        "start_screen_exit_animations",
    );
    write_animation_method(&mut out, screen, "click", "start_click_animations");
    write_animation_method(&mut out, screen, "loop", "start_loop_animations");

    let _ = writeln!(
        &mut out,
        "    pub fn apply_theme<'a, const N: usize, const E: usize, const D: usize>("
    );
    let _ = writeln!(&mut out, "        &self,");
    let _ = writeln!(&mut out, "        _gui: &mut GuiContext<'a, N, E, D>,");
    let _ = writeln!(&mut out, "        _theme: Theme,");
    let _ = writeln!(&mut out, "    ) {{");
    let _ = writeln!(&mut out, "    }}");
    let _ = writeln!(&mut out);
    let _ = writeln!(
        &mut out,
        "    pub fn set_language<'a, const N: usize, const E: usize, const D: usize>("
    );
    let _ = writeln!(&mut out, "        &self,");
    let _ = writeln!(&mut out, "        _gui: &mut GuiContext<'a, N, E, D>,");
    let _ = writeln!(&mut out, "        _table: &TranslationTable,");
    let _ = writeln!(&mut out, "    ) {{");
    let _ = writeln!(&mut out, "    }}");
    let _ = writeln!(&mut out, "}}");

    out
}

fn easing_expr(easing: &str) -> &'static str {
    match easing {
        "in_sine" => "Easing::InSine",
        "out_sine" => "Easing::OutSine",
        "in_out_sine" => "Easing::InOutSine",
        "out_cubic" => "Easing::OutCubic",
        "out_back" => "Easing::OutBack",
        "out_bounce" => "Easing::OutBounce",
        "moook" => "Easing::Moook",
        _ => "Easing::Linear",
    }
}

fn transition_effect_expr(preset: &str) -> &'static str {
    match preset {
        "window_push" => "ScreenTransitionEffect::PushMoook",
        "window_pop" => "ScreenTransitionEffect::PopMoook",
        "fade" => "ScreenTransitionEffect::Fade",
        "timeline_slide" => "ScreenTransitionEffect::SlideLeft",
        "modal_present" => "ScreenTransitionEffect::ModalSlideUp",
        "modal_dismiss" => "ScreenTransitionEffect::ModalSlideDown",
        "shutter_left" => "ScreenTransitionEffect::ShutterLeft",
        "shutter_right" => "ScreenTransitionEffect::ShutterRight",
        "shutter_up" => "ScreenTransitionEffect::ShutterUp",
        "shutter_down" => "ScreenTransitionEffect::ShutterDown",
        "port_hole_left" => "ScreenTransitionEffect::PortHoleLeft",
        "port_hole_right" => "ScreenTransitionEffect::PortHoleRight",
        "port_hole_up" => "ScreenTransitionEffect::PortHoleUp",
        "port_hole_down" => "ScreenTransitionEffect::PortHoleDown",
        "round_flip_to_launcher" => "ScreenTransitionEffect::RoundFlipRight",
        "round_flip_from_launcher" => "ScreenTransitionEffect::RoundFlipLeft",
        _ => "ScreenTransitionEffect::None",
    }
}

fn transition_origin_expr(origin: &str) -> &'static str {
    match origin {
        "top_left" => "ScreenTransitionOrigin::TopLeft",
        "top" => "ScreenTransitionOrigin::Top",
        "top_right" => "ScreenTransitionOrigin::TopRight",
        "left" => "ScreenTransitionOrigin::Left",
        "right" => "ScreenTransitionOrigin::Right",
        "bottom_left" => "ScreenTransitionOrigin::BottomLeft",
        "bottom" => "ScreenTransitionOrigin::Bottom",
        "bottom_right" => "ScreenTransitionOrigin::BottomRight",
        _ => "ScreenTransitionOrigin::Center",
    }
}

fn animation_expr(from: &str, to: &str, animation: &WidgetAnimationDef, ping_pong: bool) -> String {
    let mut expr = format!(
        "Animation::new(({from}) as f32, ({to}) as f32, {}, {}).with_delay({})",
        animation.duration_ms,
        easing_expr(&animation.easing),
        animation.delay_ms
    );
    if ping_pong || animation.repeat != 1 {
        let mode = if ping_pong {
            "RepeatMode::PingPong"
        } else {
            "RepeatMode::Loop"
        };
        let count = if animation.repeat == 0 {
            "None".to_string()
        } else {
            format!("Some({})", animation.repeat.saturating_sub(1))
        };
        let _ = write!(
            &mut expr,
            ".with_repeat_mode({mode}).with_repeat_count({count})"
        );
    }
    expr
}

fn write_binding(
    out: &mut String,
    widget: &str,
    property: &str,
    from: &str,
    to: &str,
    animation: &WidgetAnimationDef,
    ping_pong: bool,
) {
    let expr = animation_expr(from, to, animation, ping_pong);
    let _ = writeln!(
        out,
        "            animator.bind_property_with_policy(self.widgets.{widget}, AnimatedProperty::{property}, {expr}, AnimationConflictPolicy::Replace)?;"
    );
}

fn write_animation_method(out: &mut String, screen: &ScreenDef, trigger: &str, method: &str) {
    let configured: Vec<_> = screen
        .grid
        .children
        .iter()
        .filter_map(|(placement, widget)| {
            let animation = placement.animation.as_ref()?;
            (animation.trigger == trigger)
                .then(|| widget.id().map(|id| (to_snake_case(id), animation)))
                .flatten()
        })
        .collect();
    if configured.is_empty() {
        return;
    }

    let _ = writeln!(
        out,
        "    pub fn {method}<'a, const N: usize, const E: usize, const D: usize, const TRACKS: usize, const BINDINGS: usize>("
    );
    let _ = writeln!(out, "        &self,");
    let _ = writeln!(out, "        gui: &GuiContext<'a, N, E, D>,");
    let _ = writeln!(
        out,
        "        animator: &mut WidgetAnimator<TRACKS, BINDINGS>,"
    );
    let _ = writeln!(out, "    ) -> Result<(), WidgetAnimationError> {{");

    for (widget, animation) in configured {
        let _ = writeln!(
            out,
            "        if let Some(node) = gui.widgets().iter().find(|node| node.id == self.widgets.{widget}) {{"
        );
        let _ = writeln!(out, "            let base = node.rect;");
        let reverse = trigger == "screen_exit";
        match animation.preset.as_str() {
            "fade_in" => {
                let (from, to) = if reverse { ("255", "0") } else { ("0", "255") };
                write_binding(out, &widget, "Opacity", from, to, animation, false);
            }
            "fade_in_up" => {
                let (from_y, to_y, from_alpha, to_alpha) = if reverse {
                    ("base.y", "base.y + 24", "255", "0")
                } else {
                    ("base.y + 24", "base.y", "0", "255")
                };
                write_binding(out, &widget, "WidgetY", from_y, to_y, animation, false);
                write_binding(
                    out, &widget, "Opacity", from_alpha, to_alpha, animation, false,
                );
            }
            "slide_in_left" => {
                let (from, to) = if reverse {
                    ("base.x", "base.x - base.w as i32")
                } else {
                    ("base.x - base.w as i32", "base.x")
                };
                write_binding(out, &widget, "WidgetX", from, to, animation, false);
            }
            "slide_in_right" => {
                let (from, to) = if reverse {
                    ("base.x", "base.x + base.w as i32")
                } else {
                    ("base.x + base.w as i32", "base.x")
                };
                write_binding(out, &widget, "WidgetX", from, to, animation, false);
            }
            "slide_in_up" => {
                let (from, to) = if reverse {
                    ("base.y", "base.y + base.h as i32")
                } else {
                    ("base.y + base.h as i32", "base.y")
                };
                write_binding(out, &widget, "WidgetY", from, to, animation, false);
            }
            "slide_in_down" => {
                let (from, to) = if reverse {
                    ("base.y", "base.y - base.h as i32")
                } else {
                    ("base.y - base.h as i32", "base.y")
                };
                write_binding(out, &widget, "WidgetY", from, to, animation, false);
            }
            "zoom_in" => {
                let (from_w, to_w, from_h, to_h) = if reverse {
                    ("base.w", "1", "base.h", "1")
                } else {
                    ("1", "base.w", "1", "base.h")
                };
                write_binding(out, &widget, "WidgetWidth", from_w, to_w, animation, false);
                write_binding(out, &widget, "WidgetHeight", from_h, to_h, animation, false);
            }
            "pulse" | "breathe" => {
                write_binding(out, &widget, "Opacity", "150", "255", animation, true);
            }
            "shake" => {
                write_binding(
                    out,
                    &widget,
                    "WidgetX",
                    "base.x - 5",
                    "base.x + 5",
                    animation,
                    true,
                );
            }
            _ => {}
        }
        let _ = writeln!(out, "        }}");
    }
    let _ = writeln!(out, "        Ok(())");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
}

/// Renders a [`WidgetDef::Carousel`]'s parameters as a `CarouselSpec` literal.
fn carousel_spec_expr(widget: &WidgetDef) -> String {
    let WidgetDef::Carousel {
        item_step,
        visible,
        shift,
        mask_top,
        mask_bottom,
        fade,
        indicator,
        pulse,
        ..
    } = widget
    else {
        return "CarouselSpec::default()".to_string();
    };
    format!(
        "CarouselSpec {{ item_step: {item_step}, visible_slots: {visible}, shift: {shift}, \
mask_top: {mask_top}, mask_bottom: {mask_bottom}, fade_edges: {fade}, indicator: {indicator}, \
indicator_pulse: {pulse}, ..CarouselSpec::default() }}"
    )
}

/// A 1bpp icon layer resolved by a project-aware caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconAssetDef {
    pub source: String,
    pub bitmap: assets::MonoBitmapData,
}

/// A screen font resolved to embeddable bitmap data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontBinaryDef {
    pub name: String,
    pub font: assets::BitmapFontData,
}

/// Mesh geometry resolved by a project-aware caller.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshAssetDef {
    pub source: String,
    pub mesh: assets::MeshData,
}

/// Everything `include_gui!` resolved off disk for one screen.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectAssets {
    pub images: Vec<ImageAssetDef>,
    pub icons: Vec<IconAssetDef>,
    pub fonts: Vec<FontBinaryDef>,
    pub meshes: Vec<MeshAssetDef>,
}

/// Generates Rust code with project image nodes backed by static RGB565 arrays.
///
/// Plain [`generate_rust_code`] intentionally leaves image nodes as spacers
/// because a standalone KDL string has no filesystem base. `include_gui!`
/// resolves those files and calls this variant.
pub fn generate_rust_code_with_image_assets(
    screen: &ScreenDef,
    assets: &[ImageAssetDef],
) -> String {
    generate_rust_code_with_assets(
        screen,
        &ProjectAssets {
            images: assets.to_vec(),
            ..ProjectAssets::default()
        },
    )
}

/// Generates Rust code with every project asset resolved: images, 1bpp icon
/// parts, imported fonts, and mesh geometry.
pub fn generate_rust_code_with_assets(screen: &ScreenDef, assets: &ProjectAssets) -> String {
    let mut out = generate_rust_code(screen);
    let mut declarations = String::new();

    for font in &assets.fonts {
        let const_name = font_const_name(&font.name);
        let data = &font.font;
        let _ = writeln!(
            &mut declarations,
            "static {}_GLYPHS: [u8; {}] = [",
            const_name,
            data.glyphs.len()
        );
        for chunk in data.glyphs.chunks(16) {
            let values = chunk
                .iter()
                .map(|byte| format!("0x{byte:02X}"))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(&mut declarations, "    {},", values);
        }
        let _ = writeln!(&mut declarations, "];");
        let _ = writeln!(
            &mut declarations,
            "static {const}: BitmapFont = BitmapFont {{ width: {w}, height: {h}, advance: {adv}, \
        line_height: {lh}, first_char: {first}, bytes_per_row: {bpr}, glyphs: &{const}_GLYPHS }};\n",
            const = const_name,
            w = data.width,
            h = data.height,
            adv = data.advance,
            lh = data.line_height,
            first = data.first_char,
            bpr = data.bytes_per_row,
        );
    }

    // Fonts referenced by labels and carousels replace the default style font.
    for (idx, (_, widget)) in screen.grid.children.iter().enumerate() {
        let (font_name, var_name) = match widget {
            WidgetDef::Label { font: Some(f), .. } | WidgetDef::Carousel { font: Some(f), .. } => (
                f,
                widget
                    .id()
                    .map(to_snake_case)
                    .unwrap_or_else(|| format!("_w{}", idx)),
            ),
            _ => continue,
        };
        if !assets.fonts.iter().any(|f| f.name == *font_name) {
            continue;
        }
        let font_expr = format!(
            "{{ let mut style = Style::label(); style.font = FontId::Bitmap(&{}); style }}",
            font_const_name(font_name)
        );
        let prefix = format!("        let {} = gui.add_", var_name);
        out = out
            .lines()
            .map(|line| {
                if !line.starts_with(&prefix) {
                    return line.to_string();
                }
                // The style is always the final argument of the builder call.
                match (line.rfind(", "), line.rfind(")?;")) {
                    (Some(start), Some(end)) if start < end => {
                        format!("{}, {}{}", &line[..start], font_expr, &line[end..])
                    }
                    _ => line.to_string(),
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        out.push('\n');
    }

    for (idx, (_, widget)) in screen.grid.children.iter().enumerate() {
        let var_name = widget
            .id()
            .map(to_snake_case)
            .unwrap_or_else(|| format!("widget_{}", idx));
        match widget {
            WidgetDef::CompositeIcon {
                id,
                parts,
                scale,
                align,
                tint,
                ..
            } => {
                let mut part_exprs = Vec::new();
                for (part_idx, part) in parts.iter().enumerate() {
                    let Some(asset) = assets.icons.iter().find(|a| a.source == part.source) else {
                        continue;
                    };
                    let bits_name = format!("__ICON_{}_{}_BITS", idx, part_idx);
                    let _ = writeln!(
                        &mut declarations,
                        "static {}: [u8; {}] = [",
                        bits_name,
                        asset.bitmap.bits.len()
                    );
                    for chunk in asset.bitmap.bits.chunks(16) {
                        let values = chunk
                            .iter()
                            .map(|byte| format!("0x{byte:02X}"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let _ = writeln!(&mut declarations, "    {},", values);
                    }
                    let _ = writeln!(&mut declarations, "];\n");
                    let tint_expr = match part.tint.as_deref().or(tint.as_deref()) {
                        Some(color) => format!("Some({})", rgb565_expr(color)),
                        None => "None".to_string(),
                    };
                    part_exprs.push(format!(
                        "    IconPart {{ bitmap: MonoBitmap {{ width: {w}, height: {h}, bits: &{bits} }}, \
dx: {dx}, dy: {dy}, visible: {vis}, tint: {tint} }},",
                        w = asset.bitmap.width,
                        h = asset.bitmap.height,
                        bits = bits_name,
                        dx = part.dx,
                        dy = part.dy,
                        vis = part.visible,
                        tint = tint_expr,
                    ));
                }
                if part_exprs.is_empty() {
                    continue;
                }
                let parts_name = format!("__ICON_{}_PARTS", idx);
                // `pub` so firmware can copy into a mutable buffer and flip
                // `visible`/`tint` without rebuilding the screen. Prefer the
                // widget id when present so call sites read as `WEAPON_PARTS`.
                let pub_alias = id.as_ref().map(|name| {
                    let upper = name
                        .chars()
                        .map(|c| {
                            if c.is_ascii_alphanumeric() {
                                c.to_ascii_uppercase()
                            } else {
                                '_'
                            }
                        })
                        .collect::<String>();
                    format!("{upper}_PARTS")
                });
                let _ = writeln!(
                    &mut declarations,
                    "pub static {}: [IconPart<'static>; {}] = [\n{}\n];\n",
                    parts_name,
                    part_exprs.len(),
                    part_exprs.join("\n")
                );
                if let Some(alias) = pub_alias.as_ref() {
                    if alias != &parts_name {
                        let _ = writeln!(
                            &mut declarations,
                            "/// Seed for a firmware-owned mutable copy of the `{id}` icon parts.\n\
pub static {alias}: [IconPart<'static>; {n}] = {parts};\n",
                            id = id.as_deref().unwrap_or("icon"),
                            n = part_exprs.len(),
                            parts = parts_name,
                        );
                    }
                }

                let sources = parts
                    .iter()
                    .map(|part| part.source.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let needle = format!(
                    "        let {} = gui.add_spacer(cells[{}])?; // icon parts=[{}]; asset data is supplied by include_gui!",
                    var_name, idx, sources
                );
                let align_expr = if align == "top_left" {
                    "IconAlign::TopLeft"
                } else {
                    "IconAlign::Center"
                };
                let replacement = format!(
                    "        let {} = gui.add_composite_icon(cells[{}], &{}, CompositeIconSpec {{ scale: {}, align: {}, paper: None }}, Style::default())?;",
                    var_name, idx, parts_name, scale, align_expr
                );
                out = out.replace(&needle, &replacement);
            }
            WidgetDef::Mesh3d {
                source,
                shading,
                color,
                scale,
                roll,
                pitch,
                yaw,
                camera_distance,
                fov,
                ..
            } => {
                let Some(asset) = assets.meshes.iter().find(|a| a.source == *source) else {
                    continue;
                };
                let mesh = &asset.mesh;
                let base = format!("__MESH_{}", idx);
                let _ = writeln!(
                    &mut declarations,
                    "static {}_VERTICES: [[f32; 3]; {}] = [",
                    base,
                    mesh.vertices.len()
                );
                for v in &mesh.vertices {
                    let _ = writeln!(
                        &mut declarations,
                        "    [{:?}, {:?}, {:?}],",
                        v[0], v[1], v[2]
                    );
                }
                let _ = writeln!(&mut declarations, "];");
                let _ = writeln!(
                    &mut declarations,
                    "static {}_FACES: [[usize; 3]; {}] = [",
                    base,
                    mesh.faces.len()
                );
                for f in &mesh.faces {
                    let _ = writeln!(&mut declarations, "    [{}, {}, {}],", f[0], f[1], f[2]);
                }
                let _ = writeln!(&mut declarations, "];");
                let _ = writeln!(
                    &mut declarations,
                    "static {}_NORMALS: [[f32; 3]; {}] = [",
                    base,
                    mesh.normals.len()
                );
                for n in &mesh.normals {
                    let _ = writeln!(
                        &mut declarations,
                        "    [{:?}, {:?}, {:?}],",
                        n[0], n[1], n[2]
                    );
                }
                let _ = writeln!(&mut declarations, "];\n");

                let shading_expr = match shading.as_str() {
                    "points" => "MeshShading::Points",
                    "lines" | "wireframe" => "MeshShading::Lines",
                    "lit" => "MeshShading::Lit",
                    _ => "MeshShading::Solid",
                };
                let color_expr = match color.as_deref() {
                    Some(c) => rgb565_expr(c),
                    None => "Rgb565::new(31, 63, 31)".to_string(),
                };
                // The panel is a free function rather than a widget: the 3D
                // rasterizer needs a Z-buffer the caller owns.
                let _ = writeln!(
                    &mut declarations,
                    "/// Mesh panel for the `{source}` node. Render it with\n\
/// `embedded_gui::interop::three_d::render_mesh_panel` using the rect of the\n\
/// `{var_name}` widget; requires embedded-gui's `embedded-3dgfx` feature.\n\
pub fn {var_name}_mesh_panel() -> MeshPanel<'static> {{\n\
    let mut panel = MeshPanel::new(\n\
        Geometry {{\n\
            vertices: &{base}_VERTICES,\n\
            faces: &{base}_FACES,\n\
            normals: &{base}_NORMALS,\n\
            ..Geometry::default()\n\
        }},\n\
        {color_expr},\n\
    );\n\
    panel.shading = {shading_expr};\n\
    panel.scale = {scale:?};\n\
    panel.attitude = ({roll:?}, {pitch:?}, {yaw:?});\n\
    panel.camera_distance = {camera_distance:?};\n\
    panel.fov = {fov:?};\n\
    panel\n\
}}\n"
                );
            }
            _ => {}
        }
    }

    for (idx, (_, widget)) in screen.grid.children.iter().enumerate() {
        let WidgetDef::Image {
            source, fit, mode, ..
        } = widget
        else {
            continue;
        };
        let Some(asset) = assets.images.iter().find(|asset| asset.source == *source) else {
            continue;
        };
        let var_name = widget
            .id()
            .map(to_snake_case)
            .unwrap_or_else(|| format!("widget_{}", idx));
        let const_name = format!("__IMAGE_ASSET_{}_PIXELS", idx);
        let _ = writeln!(&mut declarations, "const {}: &[u16] = &[", const_name);
        for chunk in asset.pixels.chunks(12) {
            let values = chunk
                .iter()
                .map(|pixel| format!("0x{pixel:04X}"))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(&mut declarations, "    {},", values);
        }
        let _ = writeln!(&mut declarations, "];\n");

        let needle = format!(
            "        let {} = gui.add_spacer(cells[{}])?; // image src={:?} fit={:?} mode={:?}; asset data is supplied by include_gui!",
            var_name, idx, source, fit, mode
        );
        let fit_expr = if fit == "center" {
            "ImageFit::Center"
        } else {
            "ImageFit::Stretch"
        };
        let replacement = format!(
            "        let {} = gui.add_image(cells[{}], ImageRef::new({}, {}, {}), {}, Style::default())?;",
            var_name, idx, asset.width, asset.height, const_name, fit_expr
        );
        out = out.replace(&needle, &replacement);
    }

    if !assets.meshes.is_empty() {
        declarations.insert_str(
            0,
            "use embedded_gui::interop::three_d::{Geometry, MeshPanel, MeshShading};\n\n",
        );
    }

    if !declarations.is_empty() {
        let marker = "use embedded_gui::prelude::*;\n";
        out = out.replacen(marker, &format!("{marker}\n{declarations}"), 1);
    }
    out
}

/// Rust-cases a font name into the identifier of its generated static.
fn font_const_name(name: &str) -> String {
    let mut out = String::from("__FONT_");
    for ch in name.chars() {
        out.push(if ch.is_ascii_alphanumeric() {
            ch.to_ascii_uppercase()
        } else {
            '_'
        });
    }
    out
}

/// Renders a color token (`#RRGGBB`, `#RGB`, or a palette name) as an
/// `Rgb565::new(..)` literal.
fn rgb565_expr(token: &str) -> String {
    let (r, g, b) = match token.trim().trim_start_matches('#') {
        hex if hex.len() == 6 && token.trim_start().starts_with('#') => {
            let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            (byte(0), byte(2), byte(4))
        }
        hex if hex.len() == 3 && token.trim_start().starts_with('#') => {
            let nib = |i: usize| {
                let v = u8::from_str_radix(&hex[i..i + 1], 16).unwrap_or(0);
                v * 17
            };
            (nib(0), nib(1), nib(2))
        }
        "black" => (0, 0, 0),
        "red" | "danger" => (255, 0, 0),
        "green" | "success" => (0, 255, 0),
        "blue" => (0, 0, 255),
        "yellow" | "warning" => (255, 255, 0),
        "cyan" | "accent" => (0, 255, 255),
        "magenta" => (255, 0, 255),
        _ => (255, 255, 255),
    };
    format!(
        "Rgb565::new({}, {}, {})",
        r as u16 * 31 / 255,
        g as u16 * 63 / 255,
        b as u16 * 31 / 255
    )
}

/// Serializes a `ScreenDef` back into clean, formatted KDL markup.
pub fn serialize_kdl_screen(screen: &ScreenDef) -> String {
    let mut out = String::new();
    let theme_attr = match &screen.theme {
        Some(t) => format!(" theme=\"{}\"", t),
        None => String::new(),
    };
    let transition_attr = screen
        .transition
        .as_ref()
        .map(|transition| {
            format!(
                " transition=\"{}\" transition_duration={} transition_easing=\"{}\" transition_origin=\"{}\"",
                transition.preset,
                transition.duration_ms,
                transition.easing,
                transition.origin
            )
        })
        .unwrap_or_default();
    let _ = writeln!(
        &mut out,
        "screen id=\"{}\" width={} height={}{}{} {{",
        screen.id, screen.width, screen.height, theme_attr, transition_attr
    );

    for font in &screen.fonts {
        let chars_attr = if font.chars.is_empty() {
            String::new()
        } else {
            format!(" chars=\"{}\"", font.chars)
        };
        let _ = writeln!(
            &mut out,
            "    font id=\"{}\" src=\"{}\"{}",
            font.name, font.source, chars_attr
        );
    }

    let cols_str: Vec<String> = screen
        .grid
        .cols
        .iter()
        .map(|t| match t {
            GridTrackDef::Px(px) => format!("{}px", px),
            GridTrackDef::Fr(fr) => format!("{}fr", fr),
            GridTrackDef::Auto => "auto".to_string(),
        })
        .collect();

    let rows_str: Vec<String> = screen
        .grid
        .rows
        .iter()
        .map(|t| match t {
            GridTrackDef::Px(px) => format!("{}px", px),
            GridTrackDef::Fr(fr) => format!("{}fr", fr),
            GridTrackDef::Auto => "auto".to_string(),
        })
        .collect();

    let grid_id_attr = match &screen.grid.id {
        Some(id) => format!(" id=\"{}\"", id),
        None => String::new(),
    };

    let _ = writeln!(
        &mut out,
        "    grid{} cols=\"{}\" rows=\"{}\" gap={} padding={} {{",
        grid_id_attr,
        cols_str.join(" "),
        rows_str.join(" "),
        screen.grid.gap,
        screen.grid.padding
    );

    for (p, w) in &screen.grid.children {
        let span_attrs = {
            let mut s = String::new();
            if p.col_span > 1 {
                let _ = write!(&mut s, " col_span={}", p.col_span);
            }
            if p.row_span > 1 {
                let _ = write!(&mut s, " row_span={}", p.row_span);
            }
            if let Some(animation) = &p.animation {
                let _ = write!(
                    &mut s,
                    " animation=\"{}\" animation_trigger=\"{}\" animation_duration={} animation_delay={} animation_easing=\"{}\" animation_repeat={}",
                    animation.preset,
                    animation.trigger,
                    animation.duration_ms,
                    animation.delay_ms,
                    animation.easing,
                    animation.repeat
                );
            }
            s
        };

        match w {
            WidgetDef::Label {
                id,
                text,
                style,
                font,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let style_attr = style
                    .as_ref()
                    .map(|s| format!(" style=\"{}\"", s))
                    .unwrap_or_default();
                let font_attr = font
                    .as_ref()
                    .map(|s| format!(" font=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        label{} text=\"{}\"{}{}{} col={} row={}",
                    id_attr, text, style_attr, font_attr, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Button {
                id,
                text,
                on_click,
                style,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let style_attr = style
                    .as_ref()
                    .map(|s| format!(" style=\"{}\"", s))
                    .unwrap_or_default();
                let click_attr = on_click
                    .as_ref()
                    .map(|s| format!(" on_click=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        button{} text=\"{}\"{}{}{} col={} row={}",
                    id_attr, text, style_attr, click_attr, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Toggle { id, label, checked } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        toggle{} label=\"{}\" checked={}{} col={} row={}",
                    id_attr, label, checked, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Checkbox { id, label, checked } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        checkbox{} label=\"{}\" checked={}{} col={} row={}",
                    id_attr, label, checked, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Slider {
                id,
                min,
                max,
                value,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        slider{} min={} max={} value={}{} col={} row={}",
                    id_attr, min, max, value, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Dropdown {
                id,
                options,
                selected,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        dropdown{} selected={}{} col={} row={} {{",
                    id_attr, selected, span_attrs, p.col, p.row
                );
                for opt in options {
                    let _ = writeln!(&mut out, "            option \"{}\"", opt);
                }
                let _ = writeln!(&mut out, "        }}");
            }
            WidgetDef::Roller {
                id,
                options,
                selected,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        roller{} selected={}{} col={} row={} {{",
                    id_attr, selected, span_attrs, p.col, p.row
                );
                for opt in options {
                    let _ = writeln!(&mut out, "            option \"{}\"", opt);
                }
                let _ = writeln!(&mut out, "        }}");
            }
            WidgetDef::Scale {
                id,
                mode,
                min,
                max,
                value,
                major_ticks,
                minor_ticks,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        scale{} mode=\"{}\" min={:.1} max={:.1} value={:.1} major_ticks={} minor_ticks={}{} col={} row={}",
                    id_attr,
                    mode,
                    min,
                    max,
                    value,
                    major_ticks,
                    minor_ticks,
                    span_attrs,
                    p.col,
                    p.row
                );
            }
            WidgetDef::Spinbox {
                id,
                min,
                max,
                value,
                digits,
                decimals,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        spinbox{} min={} max={} value={} digits={} decimals={}{} col={} row={}",
                    id_attr, min, max, value, digits, decimals, span_attrs, p.col, p.row
                );
            }
            WidgetDef::ProgressBar { id, value } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        progress{} value={:.2}{} col={} row={}",
                    id_attr, value, span_attrs, p.col, p.row
                );
            }
            WidgetDef::SweepingArc {
                id,
                start_angle,
                end_angle,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        sweeping_arc{} start_angle={} end_angle={}{} col={} row={}",
                    id_attr, start_angle, end_angle, span_attrs, p.col, p.row
                );
            }
            WidgetDef::BusyWheel { id, active } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        busy_wheel{} active={}{} col={} row={}",
                    id_attr, active, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Plotter { id, mode } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        plotter{} mode=\"{}\"{} col={} row={}",
                    id_attr, mode, span_attrs, p.col, p.row
                );
            }
            WidgetDef::StatusBar { id, time } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        status_bar{} time=\"{}\"{} col={} row={}",
                    id_attr, time, span_attrs, p.col, p.row
                );
            }
            WidgetDef::TimePicker {
                id,
                hour,
                minute,
                is_12h,
                is_pm,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        time_picker{} hour={} minute={} is_12h={} is_pm={}{} col={} row={}",
                    id_attr, hour, minute, is_12h, is_pm, span_attrs, p.col, p.row
                );
            }
            WidgetDef::NumberPicker {
                id,
                min,
                max,
                value,
                unit,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        number_picker{} min={} max={} value={} unit=\"{}\"{} col={} row={}",
                    id_attr, min, max, value, unit, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Dialog {
                id,
                title,
                message,
                dialog_type,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        dialog{} title=\"{}\" message=\"{}\" type=\"{}\"{} col={} row={}",
                    id_attr, title, message, dialog_type, span_attrs, p.col, p.row
                );
            }
            WidgetDef::ContentIndicator { id, count, active } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        content_indicator{} count={} active={}{} col={} row={}",
                    id_attr, count, active, span_attrs, p.col, p.row
                );
            }
            WidgetDef::CrumbsIndicator { id, count, active } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        crumbs{} count={} active={}{} col={} row={}",
                    id_attr, count, active, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Panel { id, style } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let style_attr = style
                    .as_ref()
                    .map(|s| format!(" style=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        panel{}{}{} col={} row={}",
                    id_attr, style_attr, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Image {
                id,
                source,
                fit,
                mode,
                tint,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let tint_attr = tint
                    .as_ref()
                    .map(|s| format!(" tint=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        image{} src=\"{}\" fit=\"{}\" mode=\"{}\"{}{} col={} row={}",
                    id_attr, source, fit, mode, tint_attr, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Carousel {
                id,
                items,
                selected,
                item_step,
                visible,
                shift,
                mask_top,
                mask_bottom,
                fade,
                indicator,
                pulse,
                style,
                font,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let style_attr = style
                    .as_ref()
                    .map(|s| format!(" style=\"{}\"", s))
                    .unwrap_or_default();
                let font_attr = font
                    .as_ref()
                    .map(|s| format!(" font=\"{}\"", s))
                    .unwrap_or_default();
                let mut optional = String::new();
                if *shift != 0 {
                    let _ = write!(&mut optional, " shift={}", shift);
                }
                if *mask_top != 0 {
                    let _ = write!(&mut optional, " mask_top={}", mask_top);
                }
                if *mask_bottom != 0 {
                    let _ = write!(&mut optional, " mask_bottom={}", mask_bottom);
                }
                if !*fade {
                    let _ = write!(&mut optional, " fade=false");
                }
                if *indicator {
                    let _ = write!(&mut optional, " indicator=true pulse={}", pulse);
                }
                let _ = writeln!(
                    &mut out,
                    "        carousel{} selected={} item_step={} visible={}{}{}{}{} col={} row={} {{",
                    id_attr,
                    selected,
                    item_step,
                    visible,
                    optional,
                    style_attr,
                    font_attr,
                    span_attrs,
                    p.col,
                    p.row
                );
                for item in items {
                    let _ = writeln!(&mut out, "            option \"{}\"", item);
                }
                let _ = writeln!(&mut out, "        }}");
            }
            WidgetDef::CompositeIcon {
                id,
                parts,
                scale,
                align,
                tint,
                threshold,
                invert,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let tint_attr = tint
                    .as_ref()
                    .map(|s| format!(" tint=\"{}\"", s))
                    .unwrap_or_default();
                let mut optional = String::new();
                if *threshold != 128 {
                    let _ = write!(&mut optional, " threshold={}", threshold);
                }
                if *invert {
                    let _ = write!(&mut optional, " invert=true");
                }
                let _ = writeln!(
                    &mut out,
                    "        icon{} scale={} align=\"{}\"{}{}{} col={} row={} {{",
                    id_attr, scale, align, tint_attr, optional, span_attrs, p.col, p.row
                );
                for part in parts {
                    let part_tint = part
                        .tint
                        .as_ref()
                        .map(|s| format!(" tint=\"{}\"", s))
                        .unwrap_or_default();
                    let visible = if part.visible {
                        String::new()
                    } else {
                        " visible=false".to_string()
                    };
                    let _ = writeln!(
                        &mut out,
                        "            part src=\"{}\" x={} y={}{}{}",
                        part.source, part.dx, part.dy, visible, part_tint
                    );
                }
                let _ = writeln!(&mut out, "        }}");
            }
            WidgetDef::Mesh3d {
                id,
                source,
                shading,
                color,
                scale,
                roll,
                pitch,
                yaw,
                camera_distance,
                fov,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let color_attr = color
                    .as_ref()
                    .map(|s| format!(" color=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        mesh{} src=\"{}\" shading=\"{}\"{} scale={} roll={} pitch={} yaw={} camera_distance={} fov={}{} col={} row={}",
                    id_attr,
                    source,
                    shading,
                    color_attr,
                    scale,
                    roll,
                    pitch,
                    yaw,
                    camera_distance,
                    fov,
                    span_attrs,
                    p.col,
                    p.row
                );
            }
            WidgetDef::Spacer => {
                let _ = writeln!(
                    &mut out,
                    "        spacer{} col={} row={}",
                    span_attrs, p.col, p.row
                );
            }
            WidgetDef::VectorPath {
                id,
                stroke_width,
                verbs,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        vector_path{} stroke_width={}{} col={} row={} {{",
                    id_attr, stroke_width, span_attrs, p.col, p.row
                );
                for v in verbs {
                    match v {
                        PathVerbDef::MoveTo(x, y) => {
                            let _ = writeln!(&mut out, "            move_to {} {}", x, y);
                        }
                        PathVerbDef::LineTo(x, y) => {
                            let _ = writeln!(&mut out, "            line_to {} {}", x, y);
                        }
                        PathVerbDef::QuadTo(cx, cy, x, y) => {
                            let _ =
                                writeln!(&mut out, "            quad_to {} {} {} {}", cx, cy, x, y);
                        }
                        PathVerbDef::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                            let _ = writeln!(
                                &mut out,
                                "            cubic_to {} {} {} {} {} {}",
                                c1x, c1y, c2x, c2y, x, y
                            );
                        }
                        PathVerbDef::Close => {
                            let _ = writeln!(&mut out, "            close");
                        }
                    }
                }
                let _ = writeln!(&mut out, "        }}");
            }
            WidgetDef::RectShape {
                id,
                radius,
                stroke_width,
                fill_color,
                stroke_color,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let fill_attr = fill_color
                    .as_ref()
                    .map(|s| format!(" fill=\"{}\"", s))
                    .unwrap_or_default();
                let stroke_attr = stroke_color
                    .as_ref()
                    .map(|s| format!(" stroke=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        rect{} radius={} stroke_width={}{}{}{} col={} row={}",
                    id_attr, radius, stroke_width, fill_attr, stroke_attr, span_attrs, p.col, p.row
                );
            }
            WidgetDef::LineShape {
                id,
                stroke_width,
                color,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let col_attr = color
                    .as_ref()
                    .map(|s| format!(" color=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        line{} stroke_width={}{}{} col={} row={}",
                    id_attr, stroke_width, col_attr, span_attrs, p.col, p.row
                );
            }
            WidgetDef::CircleShape {
                id,
                radius,
                stroke_width,
                fill_color,
                stroke_color,
            } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let fill_attr = fill_color
                    .as_ref()
                    .map(|s| format!(" fill=\"{}\"", s))
                    .unwrap_or_default();
                let stroke_attr = stroke_color
                    .as_ref()
                    .map(|s| format!(" stroke=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        circle{} radius={} stroke_width={}{}{}{} col={} row={}",
                    id_attr, radius, stroke_width, fill_attr, stroke_attr, span_attrs, p.col, p.row
                );
            }
            WidgetDef::Table { id, headers, rows } => {
                let id_attr = id
                    .as_ref()
                    .map(|s| format!(" id=\"{}\"", s))
                    .unwrap_or_default();
                let _ = writeln!(
                    &mut out,
                    "        table{}{} col={} row={} {{",
                    id_attr, span_attrs, p.col, p.row
                );
                if let Some(h) = headers {
                    let quoted: Vec<String> = h.iter().map(|s| format!("\"{}\"", s)).collect();
                    let _ = writeln!(&mut out, "            headers {}", quoted.join(" "));
                }
                for r in rows {
                    let quoted: Vec<String> = r.iter().map(|s| format!("\"{}\"", s)).collect();
                    let _ = writeln!(&mut out, "            row {}", quoted.join(" "));
                }
                let _ = writeln!(&mut out, "        }}");
            }
        }
    }

    let _ = writeln!(&mut out, "    }}");
    let _ = writeln!(&mut out, "}}");
    out
}

/// Convenience function: parses KDL and returns the complete generated Rust code string.
pub fn compile_kdl_to_rust(kdl_source: &str) -> Result<String, CodegenError> {
    let screen = parse_kdl_screen(kdl_source)?;
    Ok(generate_rust_code(&screen))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tracks() {
        let tracks = parse_tracks("140px, 1fr 2fr auto").unwrap();
        assert_eq!(
            tracks,
            vec![
                GridTrackDef::Px(140),
                GridTrackDef::Fr(1),
                GridTrackDef::Fr(2),
                GridTrackDef::Auto,
            ]
        );
    }

    #[test]
    fn test_compile_full_widget_suite_to_rust() {
        let kdl = r#"
screen id="FullSuite" width=320 height=240 theme="dark" {
    grid cols="100px 1fr 100px" rows="24px 1fr 1fr 40px" gap=4 padding=6 {
        status_bar id="StatusBar" col=0 row=0 col_span=3 time="10:42"
        banner col=0 row=1 text="Smart Climate"
        spinbox id="Temp" col=1 row=1 min=100 max=350 value=215
        scale id="Tach" col=2 row=1 mode="radial" min=0.0 max=120.0 value=65.0
        
        dropdown id="ModeSelect" col=0 row=2 {
            option "Auto"
            option "Cool"
            option "Heat"
        }
        number_picker id="BpmPicker" col=1 row=2 min=40 max=200 value=135 unit="BPM"
        sweeping_arc id="Sweep" col=2 row=2 start_angle=0 end_angle=180
        
        button id="SaveBtn" col=0 row=3 text="SAVE"
        toggle id="EcoTog" col=1 row=3 label="ECO" checked=true
        checkbox id="SyncChk" col=2 row=3 label="SYNC" checked=false
    }
}
"#;
        let rust_code = compile_kdl_to_rust(kdl).unwrap();
        assert!(rust_code.contains("pub struct FullSuiteWidgets {"));
        assert!(rust_code.contains("pub status_bar: WidgetId,"));
        assert!(rust_code.contains("pub temp: WidgetId,"));
        assert!(rust_code.contains("pub tach: WidgetId,"));
        assert!(rust_code.contains("pub mode_select: WidgetId,"));
        assert!(rust_code.contains("pub bpm_picker: WidgetId,"));
        assert!(rust_code.contains("pub sweep: WidgetId,"));
        assert!(rust_code.contains("pub save_btn: WidgetId,"));
        assert!(rust_code.contains("pub eco_tog: WidgetId,"));
        assert!(rust_code.contains("pub sync_chk: WidgetId,"));
        assert!(rust_code.contains("pub struct FullSuiteApp {"));
        assert!(rust_code.contains("pub const NODE_COUNT: usize = 10;"));
    }

    #[test]
    fn test_serialize_kdl_roundtrip() {
        let original_kdl = r#"screen id="Thermostat" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="36px 1fr 48px" gap=6 padding=8 {
        status_bar id="status" time="14:32" col=0 row=0 col_span=2
        scale id="temp_gauge" mode="radial" min=15.0 max=35.0 value=22.5 major_ticks=4 minor_ticks=2 col=0 row=1
        slider id="target_slider" min=10 max=40 value=23 col=1 row=1
        button id="btn_heat" text="Heat Mode" style="accent" col=0 row=2
        toggle id="power_switch" label="Power" checked=true col=1 row=2
    }
}
"#;
        let screen = parse_kdl_screen(original_kdl).unwrap();
        let serialized = serialize_kdl_screen(&screen);
        let parsed_again = parse_kdl_screen(&serialized).unwrap();
        assert_eq!(screen, parsed_again);
    }

    #[test]
    fn roundtrips_widget_animation_and_screen_transition() {
        let kdl = r#"screen id="Motion" width=320 height=240 transition="window_push" transition_duration=420 transition_easing="moook" transition_origin="right" {
    grid cols="1fr" rows="1fr" gap=0 padding=0 {
        button id="launch" text="Launch" animation="fade_in_up" animation_trigger="screen_enter" animation_duration=500 animation_delay=80 animation_easing="out_back" animation_repeat=2 col=0 row=0
    }
}"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        let transition = screen.transition.as_ref().unwrap();
        assert_eq!(transition.preset, "window_push");
        assert_eq!(transition.duration_ms, 420);

        let animation = screen.grid.children[0].0.animation.as_ref().unwrap();
        assert_eq!(animation.preset, "fade_in_up");
        assert_eq!(animation.delay_ms, 80);
        assert_eq!(animation.repeat, 2);

        let reparsed = parse_kdl_screen(&serialize_kdl_screen(&screen)).unwrap();
        assert_eq!(screen, reparsed);

        let generated = generate_rust_code(&screen);
        assert!(generated.contains("pub const TRANSITION: ScreenTransitionSpec"));
        assert!(generated.contains("ScreenTransitionEffect::PushMoook"));
        assert!(generated.contains("ScreenTransitionOrigin::Right"));
        assert!(generated.contains("start_screen_enter_animations"));
        assert!(generated.contains("AnimatedProperty::WidgetY"));
        assert!(generated.contains("Easing::OutBack"));
    }

    #[test]
    fn roundtrips_carousel_icon_mesh_and_fonts() {
        let kdl = r#"screen id="Advanced" width=96 height=64 {
    font id="sevenseg" src="assets/fonts/sevenseg30.bdf" chars="0123456789"
    grid cols="1fr 26px" rows="12px 1fr" gap=0 padding=0 {
        label id="count" text="042" font="sevenseg" col=0 row=0
        carousel id="items" selected=1 item_step=16 visible=7 mask_top=14 mask_bottom=12 indicator=true pulse=96 style="body" col=0 row=1 {
            option "ONE"
            option "TWO"
        }
        icon id="battery" scale=2 align="top_left" tint="success" threshold=100 invert=true col=1 row=1 {
            part src="assets/icons/shell.bmp" x=0 y=0
            part src="assets/icons/bolt.bmp" x=3 y=1 visible=false tint="accent"
        }
        mesh id="gem" src="assets/meshes/gem.obj" shading="lit" color="accent" scale=1.5 roll=0.4 pitch=0.7 yaw=0 camera_distance=3.5 fov=1.2 col=1 row=0
    }
}
"#;
        let screen = parse_kdl_screen(kdl).unwrap();
        assert_eq!(screen.fonts.len(), 1);
        assert_eq!(screen.fonts[0].chars, "0123456789");

        let reparsed = parse_kdl_screen(&serialize_kdl_screen(&screen)).unwrap();
        assert_eq!(screen, reparsed);
    }

    #[test]
    fn generates_carousel_builder_calls() {
        let kdl = r#"screen id="Menu" width=96 height=64 {
    grid cols="1fr" rows="1fr" gap=0 padding=0 {
        carousel id="items" selected=2 item_step=16 visible=7 mask_top=14 indicator=true pulse=96 col=0 row=0 {
            option "ONE"
            option "TWO"
            option "THREE"
        }
    }
}"#;
        let rust_code = compile_kdl_to_rust(kdl).unwrap();
        assert!(
            rust_code.contains("gui.add_carousel(cells[0], &[\"ONE\", \"TWO\", \"THREE\"], 2,")
        );
        assert!(rust_code.contains("item_step: 16"));
        assert!(rust_code.contains("mask_top: 14"));
        assert!(rust_code.contains("indicator: true"));
        assert!(rust_code.contains("indicator_pulse: 96"));
    }

    #[test]
    fn embeds_fonts_icons_and_meshes_into_generated_code() {
        let kdl = r##"screen id="Advanced" width=96 height=64 {
    font id="sevenseg" src="assets/fonts/sevenseg30.bdf"
    grid cols="1fr" rows="1fr 1fr 1fr" gap=0 padding=0 {
        label id="count" text="42" font="sevenseg" col=0 row=0
        icon id="battery" scale=2 col=0 row=1 {
            part src="assets/icons/shell.bmp" x=1 y=2
        }
        mesh id="gem" src="assets/meshes/gem.obj" shading="lit" color="#00FF00" col=0 row=2
    }
}"##;
        let screen = parse_kdl_screen(kdl).unwrap();
        let assets = ProjectAssets {
            fonts: vec![FontBinaryDef {
                name: "sevenseg".into(),
                font: assets::BitmapFontData {
                    width: 8,
                    height: 8,
                    advance: 8,
                    line_height: 8,
                    first_char: b'0',
                    bytes_per_row: 1,
                    glyphs: vec![0xFF; 16],
                },
            }],
            icons: vec![IconAssetDef {
                source: "assets/icons/shell.bmp".into(),
                bitmap: assets::MonoBitmapData {
                    width: 8,
                    height: 2,
                    bits: vec![0b1010_1010, 0b0101_0101],
                },
            }],
            meshes: vec![MeshAssetDef {
                source: "assets/meshes/gem.obj".into(),
                mesh: assets::MeshData {
                    vertices: vec![[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                    faces: vec![[0, 1, 2]],
                    normals: vec![[0.0, 0.0, 1.0]],
                },
            }],
            ..ProjectAssets::default()
        };

        let rust_code = generate_rust_code_with_assets(&screen, &assets);
        assert!(rust_code.contains("static __FONT_SEVENSEG: BitmapFont = BitmapFont {"));
        assert!(rust_code.contains("FontId::Bitmap(&__FONT_SEVENSEG)"));
        assert!(rust_code.contains("pub static __ICON_1_PARTS: [IconPart<'static>; 1]"));
        assert!(rust_code.contains("pub static BATTERY_PARTS: [IconPart<'static>; 1]"));
        assert!(rust_code.contains("gui.add_composite_icon(cells[1], &__ICON_1_PARTS"));
        assert!(rust_code.contains("scale: 2"));
        assert!(rust_code.contains("pub fn gem_mesh_panel() -> MeshPanel<'static>"));
        assert!(rust_code.contains("panel.shading = MeshShading::Lit;"));
        assert!(rust_code.contains("Rgb565::new(0, 63, 0)"));
    }

    #[test]
    fn test_svg_path_d_parsing_and_codegen() {
        let svg_d = "M 10 20 L 30 40 Q 50 60 70 80 C 10 20 30 40 50 60 Z";
        let verbs = parse_svg_path_d(svg_d);
        assert_eq!(verbs.len(), 5);
        assert_eq!(verbs[0], PathVerbDef::MoveTo(10, 20));
        assert_eq!(verbs[1], PathVerbDef::LineTo(30, 40));
        assert_eq!(verbs[2], PathVerbDef::QuadTo(50, 60, 70, 80));
        assert_eq!(verbs[3], PathVerbDef::CubicTo(10, 20, 30, 40, 50, 60));
        assert_eq!(verbs[4], PathVerbDef::Close);

        let kdl = r#"screen id="VectorApp" width=320 height=240 {
    grid cols="1fr" rows="1fr" {
        path id="my_curve" d="M 0 10 C 20 0, 40 40, 60 10 Z" stroke_width=2 col=0 row=0
    }
}"#;
        let rust_code = compile_kdl_to_rust(kdl).unwrap();
        assert!(rust_code.contains("pub struct VectorAppWidgets {"));
        assert!(rust_code.contains("pub my_curve: WidgetId,"));
        assert!(rust_code.contains("let mut _path_my_curve = VectorPath::<3>::new();"));
    }

    #[test]
    fn image_assets_round_trip_and_generate_static_rgb565() {
        let kdl = r##"screen id="Assets" width=96 height=64 {
    grid cols="1fr" rows="1fr" {
        image id="logo" src="assets/logo.png" fit="center" mode="mask" tint="#00FFFF" col=0 row=0
    }
}"##;
        let screen = parse_kdl_screen(kdl).unwrap();
        assert_eq!(
            parse_kdl_screen(&serialize_kdl_screen(&screen)).unwrap(),
            screen
        );

        let generated = generate_rust_code_with_image_assets(
            &screen,
            &[ImageAssetDef {
                source: "assets/logo.png".into(),
                width: 2,
                height: 1,
                pixels: vec![0x0000, 0xFFFF],
            }],
        );
        assert!(generated.contains("const __IMAGE_ASSET_0_PIXELS: &[u16]"));
        assert!(generated.contains("ImageRef::new(2, 1, __IMAGE_ASSET_0_PIXELS)"));
        assert!(generated.contains("ImageFit::Center"));
    }
}
