use crate::ast::*;
use core::fmt::Write as _;

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
                let mut extra_props = String::new();
                if let Some(c) = bg {
                    extra_props.push_str(&format!(" bg=\"{}\"", c));
                }
                if let Some(c) = fg {
                    extra_props.push_str(&format!(" fg=\"{}\"", c));
                }
                if let Some(c) = text_color {
                    extra_props.push_str(&format!(" text_color=\"{}\"", c));
                }
                if let Some(c) = pressed_bg {
                    extra_props.push_str(&format!(" pressed_bg=\"{}\"", c));
                }
                if let Some(c) = pressed_fg {
                    extra_props.push_str(&format!(" pressed_fg=\"{}\"", c));
                }
                if let Some(c) = pressed_text {
                    extra_props.push_str(&format!(" pressed_text=\"{}\"", c));
                }
                if let Some(c) = disabled_bg {
                    extra_props.push_str(&format!(" disabled_bg=\"{}\"", c));
                }
                if let Some(c) = disabled_fg {
                    extra_props.push_str(&format!(" disabled_fg=\"{}\"", c));
                }
                if let Some(c) = disabled_text {
                    extra_props.push_str(&format!(" disabled_text=\"{}\"", c));
                }
                if let Some(c) = focused_bg {
                    extra_props.push_str(&format!(" focused_bg=\"{}\"", c));
                }
                if let Some(c) = focused_fg {
                    extra_props.push_str(&format!(" focused_fg=\"{}\"", c));
                }
                if let Some(c) = focused_text {
                    extra_props.push_str(&format!(" focused_text=\"{}\"", c));
                }
                if let Some(c) = border_color {
                    extra_props.push_str(&format!(" border_color=\"{}\"", c));
                }
                if let Some(r) = corner_radius {
                    extra_props.push_str(&format!(" corner_radius={}", r));
                }
                let _ = writeln!(
                    &mut out,
                    "        button{} text=\"{}\"{}{}{}{} col={} row={}",
                    id_attr, text, style_attr, click_attr, extra_props, span_attrs, p.col, p.row
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
