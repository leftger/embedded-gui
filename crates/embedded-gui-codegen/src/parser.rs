use crate::ast::*;
use kdl::{KdlDocument, KdlNode};

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
    e.value().as_string()
}

fn get_string_prop<'a>(node: &'a KdlNode, name: &str) -> Option<&'a str> {
    node.get(name).and_then(|v| v.as_string())
}

fn get_i64_prop(node: &KdlNode, name: &str) -> Option<i64> {
    node.get(name)
        .and_then(|v| v.as_integer())
        .map(|i| i as i64)
}

fn get_f64_prop(node: &KdlNode, name: &str) -> Option<f64> {
    node.get(name)
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
}

fn get_bool_prop(node: &KdlNode, name: &str) -> Option<bool> {
    node.get(name).and_then(|v| v.as_bool())
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
                .filter_map(|e| {
                    e.value()
                        .as_integer()
                        .map(|i| i as i32)
                        .or_else(|| e.value().as_float().map(|f| f as i32))
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
            let bg = get_string_prop(node, "bg")
                .or_else(|| get_string_prop(node, "background"))
                .map(|s| s.to_string());
            let fg = get_string_prop(node, "fg")
                .or_else(|| get_string_prop(node, "foreground"))
                .map(|s| s.to_string());
            let text_color = get_string_prop(node, "text_color")
                .or_else(|| get_string_prop(node, "textColor"))
                .or_else(|| get_string_prop(node, "color"))
                .map(|s| s.to_string());
            let pressed_bg = get_string_prop(node, "pressed_bg")
                .or_else(|| get_string_prop(node, "pressedBg"))
                .map(|s| s.to_string());
            let pressed_fg = get_string_prop(node, "pressed_fg")
                .or_else(|| get_string_prop(node, "pressedFg"))
                .map(|s| s.to_string());
            let pressed_text = get_string_prop(node, "pressed_text")
                .or_else(|| get_string_prop(node, "pressedText"))
                .or_else(|| get_string_prop(node, "pressedColor"))
                .map(|s| s.to_string());
            let disabled_bg = get_string_prop(node, "disabled_bg")
                .or_else(|| get_string_prop(node, "disabledBg"))
                .map(|s| s.to_string());
            let disabled_fg = get_string_prop(node, "disabled_fg")
                .or_else(|| get_string_prop(node, "disabledFg"))
                .map(|s| s.to_string());
            let disabled_text = get_string_prop(node, "disabled_text")
                .or_else(|| get_string_prop(node, "disabledText"))
                .or_else(|| get_string_prop(node, "disabledColor"))
                .map(|s| s.to_string());
            let focused_bg = get_string_prop(node, "focused_bg")
                .or_else(|| get_string_prop(node, "focusedBg"))
                .map(|s| s.to_string());
            let focused_fg = get_string_prop(node, "focused_fg")
                .or_else(|| get_string_prop(node, "focusedFg"))
                .map(|s| s.to_string());
            let focused_text = get_string_prop(node, "focused_text")
                .or_else(|| get_string_prop(node, "focusedText"))
                .or_else(|| get_string_prop(node, "focusedColor"))
                .map(|s| s.to_string());
            let border_color = get_string_prop(node, "border_color")
                .or_else(|| get_string_prop(node, "borderColor"))
                .or_else(|| get_string_prop(node, "border"))
                .map(|s| s.to_string());
            let corner_radius = get_i64_prop(node, "corner_radius")
                .or_else(|| get_i64_prop(node, "cornerRadius"))
                .or_else(|| get_i64_prop(node, "radius"))
                .map(|v| v as u8);

            WidgetDef::Button {
                id,
                text,
                on_click,
                style,
                bg,
                fg,
                text_color,
                pressed_bg,
                pressed_fg,
                pressed_text,
                disabled_bg,
                disabled_fg,
                disabled_text,
                focused_bg,
                focused_fg,
                focused_text,
                border_color,
                corner_radius,
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

    let container_node = screen_node
        .children()
        .and_then(|c| {
            c.nodes().iter().find(|n| {
                let name = n.name().value();
                name == "grid" || name == "column" || name == "row" || name == "stack"
            })
        })
        .ok_or_else(|| {
            CodegenError::MissingAttribute(
                "layout",
                "screen node must contain a 'grid', 'column', 'row', or 'stack' child".into(),
            )
        })?;

    let container_type = container_node.name().value();
    let gap = get_i64_prop(container_node, "gap").unwrap_or(4).max(0) as u16;
    let padding = get_i64_prop(container_node, "padding").unwrap_or(4).max(0) as u16;

    let mut children = Vec::new();
    if let Some(c_children) = container_node.children() {
        for (idx, child) in c_children.nodes().iter().enumerate() {
            let (mut placement, widget) = parse_widget(child)?;
            match container_type {
                "column" => {
                    if get_i64_prop(child, "row").is_none() {
                        placement.row = idx;
                    }
                    if get_i64_prop(child, "col").is_none() {
                        placement.col = 0;
                    }
                }
                "row" => {
                    if get_i64_prop(child, "col").is_none() {
                        placement.col = idx;
                    }
                    if get_i64_prop(child, "row").is_none() {
                        placement.row = 0;
                    }
                }
                "stack" => {
                    if get_i64_prop(child, "col").is_none() {
                        placement.col = 0;
                    }
                    if get_i64_prop(child, "row").is_none() {
                        placement.row = 0;
                    }
                }
                _ => {}
            }
            children.push((placement, widget));
        }
    }

    let (col_tracks, row_tracks) = match container_type {
        "column" => {
            let cols_str = get_string_prop(container_node, "cols").unwrap_or("1fr");
            let cols = parse_tracks(cols_str)?;
            let rows = if let Some(rows_str) = get_string_prop(container_node, "rows") {
                parse_tracks(rows_str)?
            } else {
                vec![GridTrackDef::Fr(1); children.len().max(1)]
            };
            (cols, rows)
        }
        "row" => {
            let rows_str = get_string_prop(container_node, "rows").unwrap_or("1fr");
            let rows = parse_tracks(rows_str)?;
            let cols = if let Some(cols_str) = get_string_prop(container_node, "cols") {
                parse_tracks(cols_str)?
            } else {
                vec![GridTrackDef::Fr(1); children.len().max(1)]
            };
            (cols, rows)
        }
        _ => {
            let cols_str = get_string_prop(container_node, "cols").unwrap_or("1fr");
            let rows_str = get_string_prop(container_node, "rows").unwrap_or("1fr");
            (parse_tracks(cols_str)?, parse_tracks(rows_str)?)
        }
    };

    let grid = GridLayoutDef {
        id: get_string_prop(container_node, "id").map(|s| s.to_string()),
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

pub(crate) fn to_snake_case(s: &str) -> String {
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

pub fn parse_hex_to_rgb565(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim().trim_start_matches('#').trim_start_matches("0x");
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some((r >> 3, g >> 2, b >> 3))
    } else if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).ok()?;
        let g = u8::from_str_radix(&s[1..2], 16).ok()?;
        let b = u8::from_str_radix(&s[2..3], 16).ok()?;
        Some(((r * 17) >> 3, ((g * 17) >> 2), ((b * 17) >> 3)))
    } else {
        None
    }
}

pub fn parse_color_name_rgb565(s: &str) -> Option<(u8, u8, u8)> {
    let norm = s.to_ascii_lowercase();
    let norm = norm.replace(['-', '_', ' '], "");
    match norm.as_str() {
        "aliceblue" => Some((29, 61, 31)),
        "antiquewhite" => Some((30, 58, 26)),
        "aqua" => Some((0, 63, 31)),
        "aquamarine" => Some((15, 63, 26)),
        "azure" => Some((29, 63, 31)),
        "beige" => Some((30, 61, 27)),
        "bisque" => Some((31, 56, 24)),
        "black" => Some((0, 0, 0)),
        "blanchedalmond" => Some((31, 58, 25)),
        "blue" => Some((0, 0, 31)),
        "blueviolet" => Some((17, 11, 27)),
        "brown" => Some((20, 10, 5)),
        "burlywood" => Some((27, 45, 16)),
        "cadetblue" => Some((12, 39, 19)),
        "chartreuse" => Some((15, 63, 0)),
        "chocolate" => Some((26, 26, 4)),
        "coral" => Some((31, 31, 10)),
        "cornflowerblue" => Some((12, 37, 29)),
        "cornsilk" => Some((31, 61, 27)),
        "crimson" => Some((27, 5, 7)),
        "cyan" => Some((0, 63, 31)),
        "darkblue" => Some((0, 0, 17)),
        "darkcyan" => Some((0, 34, 17)),
        "darkgoldenrod" => Some((22, 33, 1)),
        "darkgray" => Some((21, 42, 21)),
        "darkgreen" => Some((0, 25, 0)),
        "darkgrey" => Some((21, 42, 21)),
        "darkkhaki" => Some((23, 45, 13)),
        "darkmagenta" => Some((17, 0, 17)),
        "darkolivegreen" => Some((10, 26, 6)),
        "darkorange" => Some((31, 35, 0)),
        "darkorchid" => Some((19, 12, 25)),
        "darkred" => Some((17, 0, 0)),
        "darksalmon" => Some((28, 37, 15)),
        "darkseagreen" => Some((17, 46, 17)),
        "darkslateblue" => Some((9, 15, 17)),
        "darkslategray" => Some((6, 20, 10)),
        "darkslategrey" => Some((6, 20, 10)),
        "darkturquoise" => Some((0, 51, 25)),
        "darkviolet" => Some((18, 0, 26)),
        "deeppink" => Some((31, 5, 18)),
        "deepskyblue" => Some((0, 47, 31)),
        "dimgray" => Some((13, 26, 13)),
        "dimgrey" => Some((13, 26, 13)),
        "dodgerblue" => Some((4, 36, 31)),
        "firebrick" => Some((22, 8, 4)),
        "floralwhite" => Some((31, 62, 29)),
        "forestgreen" => Some((4, 34, 4)),
        "fuchsia" => Some((31, 0, 31)),
        "gainsboro" => Some((27, 54, 27)),
        "ghostwhite" => Some((30, 61, 31)),
        "gold" => Some((31, 53, 0)),
        "goldenrod" => Some((27, 41, 4)),
        "gray" => Some((16, 32, 16)),
        "green" => Some((0, 32, 0)),
        "greenyellow" => Some((21, 63, 6)),
        "grey" => Some((16, 32, 16)),
        "honeydew" => Some((29, 63, 29)),
        "hotpink" => Some((31, 26, 22)),
        "indianred" => Some((25, 23, 11)),
        "indigo" => Some((9, 0, 16)),
        "ivory" => Some((31, 63, 29)),
        "khaki" => Some((29, 57, 17)),
        "lavender" => Some((28, 57, 30)),
        "lavenderblush" => Some((31, 59, 30)),
        "lawngreen" => Some((15, 62, 0)),
        "lemonchiffon" => Some((31, 62, 25)),
        "lightblue" => Some((21, 53, 28)),
        "lightcoral" => Some((29, 32, 16)),
        "lightcyan" => Some((27, 63, 31)),
        "lightgoldenrodyellow" => Some((30, 62, 26)),
        "lightgray" => Some((26, 52, 26)),
        "lightgreen" => Some((18, 59, 18)),
        "lightgrey" => Some((26, 52, 26)),
        "lightpink" => Some((31, 45, 23)),
        "lightsalmon" => Some((31, 40, 15)),
        "lightseagreen" => Some((4, 44, 21)),
        "lightskyblue" => Some((16, 51, 30)),
        "lightslategray" => Some((14, 34, 19)),
        "lightslategrey" => Some((14, 34, 19)),
        "lightsteelblue" => Some((21, 48, 27)),
        "lightyellow" => Some((31, 63, 27)),
        "lime" => Some((0, 63, 0)),
        "limegreen" => Some((6, 51, 6)),
        "linen" => Some((30, 59, 28)),
        "magenta" => Some((31, 0, 31)),
        "maroon" => Some((16, 0, 0)),
        "mediumaquamarine" => Some((12, 51, 21)),
        "mediumblue" => Some((0, 0, 25)),
        "mediumorchid" => Some((23, 21, 26)),
        "mediumpurple" => Some((18, 28, 27)),
        "mediumseagreen" => Some((7, 44, 14)),
        "mediumslateblue" => Some((15, 26, 29)),
        "mediumspringgreen" => Some((0, 62, 19)),
        "mediumturquoise" => Some((9, 52, 25)),
        "mediumvioletred" => Some((24, 5, 16)),
        "midnightblue" => Some((3, 6, 14)),
        "mintcream" => Some((30, 63, 30)),
        "mistyrose" => Some((31, 56, 27)),
        "moccasin" => Some((31, 56, 22)),
        "navajowhite" => Some((31, 55, 21)),
        "navy" => Some((0, 0, 16)),
        "oldlace" => Some((31, 61, 28)),
        "olive" => Some((16, 32, 0)),
        "olivedrab" => Some((13, 35, 4)),
        "orange" => Some((31, 41, 0)),
        "orangered" => Some((31, 17, 0)),
        "orchid" => Some((27, 28, 26)),
        "palegoldenrod" => Some((29, 57, 21)),
        "palegreen" => Some((18, 62, 18)),
        "paleturquoise" => Some((21, 59, 29)),
        "palevioletred" => Some((27, 28, 18)),
        "papayawhip" => Some((31, 59, 26)),
        "peachpuff" => Some((31, 54, 22)),
        "peru" => Some((25, 33, 8)),
        "pink" => Some((31, 47, 25)),
        "plum" => Some((27, 40, 27)),
        "powderblue" => Some((21, 55, 28)),
        "purple" => Some((16, 0, 16)),
        "red" => Some((31, 0, 0)),
        "rosybrown" => Some((23, 35, 17)),
        "royalblue" => Some((8, 26, 27)),
        "saddlebrown" => Some((17, 17, 2)),
        "salmon" => Some((30, 32, 14)),
        "sandybrown" => Some((30, 41, 12)),
        "seagreen" => Some((6, 34, 11)),
        "seashell" => Some((31, 61, 29)),
        "sienna" => Some((19, 20, 5)),
        "silver" => Some((23, 47, 23)),
        "skyblue" => Some((16, 51, 29)),
        "slateblue" => Some((13, 22, 25)),
        "slategray" => Some((14, 32, 18)),
        "slategrey" => Some((14, 32, 18)),
        "snow" => Some((31, 62, 30)),
        "springgreen" => Some((0, 63, 15)),
        "steelblue" => Some((9, 32, 22)),
        "tan" => Some((26, 44, 17)),
        "teal" => Some((0, 32, 16)),
        "thistle" => Some((26, 47, 26)),
        "tomato" => Some((31, 24, 9)),
        "turquoise" => Some((8, 55, 25)),
        "violet" => Some((29, 32, 29)),
        "wheat" => Some((30, 55, 22)),
        "white" => Some((31, 63, 31)),
        "whitesmoke" => Some((30, 61, 30)),
        "yellow" => Some((31, 63, 0)),
        "yellowgreen" => Some((19, 51, 6)),
        _ => None,
    }
}

pub fn color_to_rust_expr(s: &str) -> String {
    let s = s.trim();
    if s.starts_with("Rgb565::") {
        s.to_string()
    } else if let Some((r, g, b)) = parse_hex_to_rgb565(s) {
        format!("Rgb565::new({}, {}, {})", r, g, b)
    } else if let Some((r, g, b)) = parse_color_name_rgb565(s) {
        format!("Rgb565::new({}, {}, {})", r, g, b)
    } else {
        match s.to_lowercase().as_str() {
            "white" => "Rgb565::WHITE".into(),
            "black" => "Rgb565::BLACK".into(),
            "red" => "Rgb565::RED".into(),
            "green" => "Rgb565::GREEN".into(),
            "blue" => "Rgb565::BLUE".into(),
            "yellow" => "Rgb565::YELLOW".into(),
            "cyan" => "Rgb565::CYAN".into(),
            "magenta" => "Rgb565::MAGENTA".into(),
            _ => "Rgb565::WHITE".into(),
        }
    }
}

pub(crate) fn style_expr(style: Option<&str>, default: &str) -> String {
    match style {
        None => default.to_string(),
        Some(s)
            if s.starts_with("Style::")
                || s.starts_with("WidgetStyle::")
                || s.starts_with('{')
                || s.starts_with("crate::") =>
        {
            s.to_string()
        }
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
        Some(s) if s.starts_with('#') || s.starts_with("0x") => {
            format!("WidgetStyle::button().with_bg({})", color_to_rust_expr(s))
        }
        Some(s) => s.to_string(),
    }
}
