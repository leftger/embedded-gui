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
    Slider {
        id: Option<String>,
        min: i32,
        max: i32,
        value: i32,
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
}

impl WidgetDef {
    pub fn id(&self) -> Option<&str> {
        match self {
            WidgetDef::Label { id, .. }
            | WidgetDef::Button { id, .. }
            | WidgetDef::Toggle { id, .. }
            | WidgetDef::Slider { id, .. }
            | WidgetDef::Scale { id, .. }
            | WidgetDef::Spinbox { id, .. }
            | WidgetDef::Table { id, .. }
            | WidgetDef::ProgressBar { id, .. } => id.as_deref(),
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

fn entry_to_str<'a>(e: &'a kdl::KdlEntry) -> Option<&'a str> {
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
    let _ = writeln!(&mut out, "");

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
    let _ = writeln!(&mut out, "");

    // App struct
    let _ = writeln!(&mut out, "pub struct {} {{", app_struct_name);
    let _ = writeln!(&mut out, "    pub widgets: {},", widget_struct_name);
    let _ = writeln!(&mut out, "}}");
    let _ = writeln!(&mut out, "");

    // Impl block with build method
    let _ = writeln!(&mut out, "impl {} {{", app_struct_name);
    let _ = writeln!(&mut out, "    pub const WIDTH: u32 = {};", screen.width);
    let _ = writeln!(&mut out, "    pub const HEIGHT: u32 = {};", screen.height);
    let _ = writeln!(
        &mut out,
        "    pub const NODE_COUNT: usize = {};",
        total_nodes
    );
    let _ = writeln!(&mut out, "");
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
    let _ = writeln!(&mut out, "");

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
    let _ = writeln!(&mut out, "");

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
                let st = style.as_deref().unwrap_or("Style::default()");
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_label(cells[{}], \"{}\", {})?;",
                    var_name, idx, text, st
                );
            }
            WidgetDef::Button { text, style, .. } => {
                let st = style.as_deref().unwrap_or("Style::button()");
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
            WidgetDef::Slider {
                min, max, value, ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_slider(cells[{}], {}, {}, {}, Style::default())?;",
                    var_name, idx, min, max, value
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
                min,
                max,
                value,
                digits,
                decimals,
                ..
            } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_spinbox(cells[{}], {}, {}, {}, Style::panel())?;",
                    var_name, idx, min, max, value
                );
                if *digits != 4 || *decimals != 0 {
                    // Optional attribute adjustments
                }
            }
            WidgetDef::ProgressBar { value, .. } => {
                let _ = writeln!(
                    &mut out,
                    "        let {} = gui.add_progress_bar(cells[{}], {:.2}, Style::default())?;",
                    var_name, idx, value
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

    let _ = writeln!(&mut out, "");
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
    fn test_compile_kdl_screen_to_rust() {
        let kdl = r#"
screen id="Thermostat" width=320 height=240 theme="dark" {
    grid cols="140px 1fr" rows="24px 1fr 48px" gap=6 padding=8 {
        banner col=0 row=0 col_span=2 text="Smart Thermostat"
        spinbox id="TempSetpoint" col=0 row=1 min=100 max=350 value=215 digits=4 decimals=1
        scale id="RoomGauge" col=1 row=1 mode="radial" min=10.0 max=40.0 value=22.5
        button id="FanBtn" col=0 row=2 text="TOGGLE FAN"
        toggle id="EcoMode" col=1 row=2 checked=true
    }
}
"#;
        let rust_code = compile_kdl_to_rust(kdl).unwrap();
        assert!(rust_code.contains("pub struct ThermostatWidgets {"));
        assert!(rust_code.contains("pub temp_setpoint: WidgetId,"));
        assert!(rust_code.contains("pub room_gauge: WidgetId,"));
        assert!(rust_code.contains("pub fan_btn: WidgetId,"));
        assert!(rust_code.contains("pub eco_mode: WidgetId,"));
        assert!(rust_code.contains("pub struct ThermostatApp {"));
        assert!(rust_code.contains("pub const NODE_COUNT: usize = 5;"));
        assert!(rust_code.contains("GridLayout::<2, 3>::new"));
    }
}
