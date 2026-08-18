//! `embedded-gui-codegen`: KDL Markup Parser and Rust Code Generator for `embedded-gui`
//!
//! Enables UI designers and non-technical domain experts to author declarative GUI screens
//! in KDL and compile them into deterministic, zero-allocation (`no_std`) Rust code.

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
}

impl Default for GridPlacementDef {
    fn default() -> Self {
        Self {
            col: 0,
            row: 0,
            col_span: 1,
            row_span: 1,
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
    let placement = GridPlacementDef {
        col,
        row,
        col_span,
        row_span,
    };

    let id = get_string_prop(node, "id").map(|s| s.to_string());
    let style = get_string_prop(node, "style").map(|s| s.to_string());

    let widget = match tag {
        "label" | "banner" => {
            let text = get_string_prop(node, "text")
                .or_else(|| node.entries().first().and_then(entry_to_str))
                .unwrap_or("")
                .to_string();
            WidgetDef::Label { id, text, style }
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
    let _ = writeln!(&mut out, "}}");

    out
}

/// Serializes a `ScreenDef` back into clean, formatted KDL markup.
pub fn serialize_kdl_screen(screen: &ScreenDef) -> String {
    let mut out = String::new();
    let theme_attr = match &screen.theme {
        Some(t) => format!(" theme=\"{}\"", t),
        None => String::new(),
    };
    let _ = writeln!(
        &mut out,
        "screen id=\"{}\" width={} height={}{} {{",
        screen.id, screen.width, screen.height, theme_attr
    );

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
            s
        };

        match w {
            WidgetDef::Label { id, text, style } => {
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
                    "        label{} text=\"{}\"{}{} col={} row={}",
                    id_attr, text, style_attr, span_attrs, p.col, p.row
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
}
