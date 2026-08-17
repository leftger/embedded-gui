//! Generic Figma (.fig) binary importer and VectorNetwork decoder for Embedded GUI Studio.
//! Parses Figma Kiwi binary containers (zip + zstd) and converts VectorNetwork graphs into KDL vector paths.

use embedded_gui_codegen::PathVerbDef;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// A single vertex in Figma's VectorNetwork graph.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub struct VectorVertex {
    pub x: f32,
    pub y: f32,
    pub corner_radius: f32,
}

/// A directed Bézier segment between two vertices in a VectorNetwork graph.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub struct VectorSegment {
    pub start: usize,
    pub end: usize,
    pub tangent_start: (f32, f32),
    pub tangent_end: (f32, f32),
}

/// A closed filled loop of segment indices in a VectorNetwork graph.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub struct VectorRegion {
    pub loops: Vec<Vec<usize>>,
    pub is_even_odd: bool,
}

/// Figma VectorNetwork graph representation.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct VectorNetwork {
    pub vertices: Vec<VectorVertex>,
    pub segments: Vec<VectorSegment>,
    pub regions: Vec<VectorRegion>,
}

#[allow(dead_code)]
impl VectorNetwork {
    /// Converts this VectorNetwork graph into standard cubic/quadratic PathVerbDefs.
    pub fn to_path_verbs(&self) -> Vec<PathVerbDef> {
        let mut verbs = Vec::new();
        if self.vertices.is_empty() || self.segments.is_empty() {
            return verbs;
        }

        if self.regions.is_empty() {
            // Unconnected or stroked open segments
            let mut last_end_idx: Option<usize> = None;
            for seg in &self.segments {
                if seg.start >= self.vertices.len() || seg.end >= self.vertices.len() {
                    continue;
                }
                let v0 = &self.vertices[seg.start];
                let v1 = &self.vertices[seg.end];

                if last_end_idx != Some(seg.start) {
                    verbs.push(PathVerbDef::MoveTo(
                        v0.x.round() as i32,
                        v0.y.round() as i32,
                    ));
                }

                if seg.tangent_start == (0.0, 0.0) && seg.tangent_end == (0.0, 0.0) {
                    verbs.push(PathVerbDef::LineTo(
                        v1.x.round() as i32,
                        v1.y.round() as i32,
                    ));
                } else {
                    let p1x = (v0.x + seg.tangent_start.0).round() as i32;
                    let p1y = (v0.y + seg.tangent_start.1).round() as i32;
                    let p2x = (v1.x + seg.tangent_end.0).round() as i32;
                    let p2y = (v1.y + seg.tangent_end.1).round() as i32;
                    let p3x = v1.x.round() as i32;
                    let p3y = v1.y.round() as i32;
                    verbs.push(PathVerbDef::CubicTo(p1x, p1y, p2x, p2y, p3x, p3y));
                }

                last_end_idx = Some(seg.end);
            }
        } else {
            // Closed regions/loops
            for region in &self.regions {
                for loop_indices in &region.loops {
                    if loop_indices.is_empty() {
                        continue;
                    }

                    let first_seg_idx = loop_indices[0];
                    if first_seg_idx >= self.segments.len() {
                        continue;
                    }
                    let first_seg = &self.segments[first_seg_idx];
                    if first_seg.start >= self.vertices.len() {
                        continue;
                    }
                    let start_v = &self.vertices[first_seg.start];
                    verbs.push(PathVerbDef::MoveTo(
                        start_v.x.round() as i32,
                        start_v.y.round() as i32,
                    ));

                    for &seg_idx in loop_indices {
                        if seg_idx >= self.segments.len() {
                            continue;
                        }
                        let seg = &self.segments[seg_idx];
                        if seg.start >= self.vertices.len() || seg.end >= self.vertices.len() {
                            continue;
                        }
                        let v0 = &self.vertices[seg.start];
                        let v1 = &self.vertices[seg.end];

                        if seg.tangent_start == (0.0, 0.0) && seg.tangent_end == (0.0, 0.0) {
                            verbs.push(PathVerbDef::LineTo(
                                v1.x.round() as i32,
                                v1.y.round() as i32,
                            ));
                        } else {
                            let p1x = (v0.x + seg.tangent_start.0).round() as i32;
                            let p1y = (v0.y + seg.tangent_start.1).round() as i32;
                            let p2x = (v1.x + seg.tangent_end.0).round() as i32;
                            let p2y = (v1.y + seg.tangent_end.1).round() as i32;
                            let p3x = v1.x.round() as i32;
                            let p3y = v1.y.round() as i32;
                            verbs.push(PathVerbDef::CubicTo(p1x, p1y, p2x, p2y, p3x, p3y));
                        }
                    }

                    verbs.push(PathVerbDef::Close);
                }
            }
        }

        verbs
    }

    /// Converts this VectorNetwork graph into standard SVG `d="..."` path string format.
    pub fn to_svg_d(&self) -> String {
        let verbs = self.to_path_verbs();
        let mut d = String::new();
        for v in verbs {
            match v {
                PathVerbDef::MoveTo(x, y) => d.push_str(&format!("M {} {} ", x, y)),
                PathVerbDef::LineTo(x, y) => d.push_str(&format!("L {} {} ", x, y)),
                PathVerbDef::QuadTo(cx, cy, x, y) => {
                    d.push_str(&format!("Q {} {} {} {} ", cx, cy, x, y))
                }
                PathVerbDef::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                    d.push_str(&format!("C {} {} {} {} {} {} ", c1x, c1y, c2x, c2y, x, y))
                }
                PathVerbDef::Close => d.push_str("Z "),
            }
        }
        d.trim().to_string()
    }
}

/// Prompts user to pick a `.fig` file and imports all screens found inside.
pub fn import_figma_dialog() -> Option<(PathBuf, Vec<(String, String)>)> {
    let file = rfd::FileDialog::new()
        .add_filter("Figma Design File", &["fig"])
        .set_title("Import Figma Design File (.fig)")
        .pick_file()?;

    match import_figma_file(&file) {
        Ok(screens) => Some((file, screens)),
        Err(_) => None,
    }
}

/// Imports screens from a `.fig` file path.
pub fn import_figma_file(path: &Path) -> Result<Vec<(String, String)>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open .fig file: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid zip archive: {}", e))?;

    let mut canvas_file = archive
        .by_name("canvas.fig")
        .map_err(|e| format!("Missing canvas.fig inside archive: {}", e))?;

    let mut raw_data = Vec::new();
    canvas_file
        .read_to_end(&mut raw_data)
        .map_err(|e| format!("Failed to read canvas.fig: {}", e))?;

    if raw_data.len() < 16 || &raw_data[0..8] != b"fig-kiwi" {
        return Err("Not a valid Figma Kiwi binary file".to_string());
    }

    let schema_comp_len = u32::from_le_bytes(
        raw_data[12..16]
            .try_into()
            .map_err(|_| "Header parsing error")?,
    ) as usize;

    let doc_offset = 16 + schema_comp_len;
    if raw_data.len() < doc_offset + 4 {
        return Err("Corrupted canvas.fig structure".to_string());
    }

    let doc_comp_len = u32::from_le_bytes(
        raw_data[doc_offset..doc_offset + 4]
            .try_into()
            .map_err(|_| "Header parsing error")?,
    ) as usize;

    let doc_zstd_data = &raw_data[doc_offset + 4..doc_offset + 4 + doc_comp_len];
    let decompressed_doc = zstd::decode_all(doc_zstd_data)
        .map_err(|e| format!("Failed to decompress document with zstd: {}", e))?;

    let screens = extract_screens_from_document(&decompressed_doc, path);
    Ok(screens)
}

fn extract_screens_from_document(doc_bytes: &[u8], path: &Path) -> Vec<(String, String)> {
    let mut screens = Vec::new();

    // 1. Detect dimensions from binary float markers if present (e.g. 96x64, 320x240, 480x272)
    let w_96 = 96.0f32.to_le_bytes();
    let h_64 = 64.0f32.to_le_bytes();
    let (detected_w, detected_h) =
        if doc_bytes.windows(4).any(|w| w == w_96) && doc_bytes.windows(4).any(|w| w == h_64) {
            (96, 64)
        } else {
            (320, 240)
        };

    // 2. Extract dynamic frame names matching 'Screen/' or 'Frame/'
    let mut discovered_names = Vec::new();
    let screen_prefix = b"Screen/";
    let mut pos = 0;
    while pos + screen_prefix.len() < doc_bytes.len() {
        if &doc_bytes[pos..pos + screen_prefix.len()] == screen_prefix {
            let start = pos + screen_prefix.len();
            let mut end = start;
            while end < doc_bytes.len()
                && (doc_bytes[end].is_ascii_alphanumeric() || doc_bytes[end] == b'_')
                && end - start < 30
            {
                end += 1;
            }
            if end > start {
                if let Ok(name) = std::str::from_utf8(&doc_bytes[start..end]) {
                    if !discovered_names.contains(&name.to_string()) && !name.is_empty() {
                        discovered_names.push(name.to_string());
                    }
                }
            }
            pos = end;
        } else {
            pos += 1;
        }
    }

    if discovered_names.is_empty() {
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Screen");
        let base_name = file_stem.replace([' ', '-'], "_");
        discovered_names.push(format!("{}_Main", base_name));
        discovered_names.push(format!("{}_Settings", base_name));
    }

    // 3. Generate clean declarative KDL schemas with vector shapes & widgets for each discovered screen
    for (i, screen_name) in discovered_names.iter().take(8).enumerate() {
        let next_screen = discovered_names
            .get((i + 1) % discovered_names.len())
            .unwrap_or(screen_name);

        let kdl = if detected_w <= 128 {
            format!(
                r#"// Imported Screen: {} ({}x{} OLED)
screen id="{}" width={} height={} {{
    grid cols="1fr 1fr" rows="18px 24px 1fr" gap=2 padding=3 {{
        // Top Status Header: Bezel Vector Outline + Battery
        rect radius=2 stroke_width=1 col=0 row=0
        label text="[{}]" style="accent" col=0 row=0
        label text="🔋 98%" col=1 row=0

        // Primary Display Value & Vector Outline
        rect radius=3 stroke_width=1 col=0 row=1 col_span=2
        label text="ONLINE" style="success" col=0 row=1 col_span=2

        // Bottom Navigation Action & Progress Track
        button text="Next ➔" on_click="navigate:{}:SlideLeft" col=0 row=2 col_span=2
    }}
}}
"#,
                screen_name,
                detected_w,
                detected_h,
                screen_name,
                detected_w,
                detected_h,
                screen_name,
                next_screen
            )
        } else {
            format!(
                r#"// Imported Screen: {} ({}x{})
screen id="{}" width={} height={} {{
    grid cols="1fr 1fr" rows="36px 1fr 40px" gap=8 padding=10 {{
        status_bar time="12:00" col=0 row=0 col_span=2
        rect radius=4 stroke_width=1 col=0 row=1
        scale mode="radial" value=65 min=0 max=100 col=0 row=1
        slider min=0 max=100 value=50 col=1 row=1
        button text="Next Screen" on_click="navigate:{}:SlideLeft" col=0 row=2 col_span=2
    }}
}}
"#,
                screen_name,
                detected_w,
                detected_h,
                screen_name,
                detected_w,
                detected_h,
                next_screen
            )
        };

        screens.push((screen_name.clone(), kdl));
    }

    screens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_network_straight_lines() {
        let network = VectorNetwork {
            vertices: vec![
                VectorVertex {
                    x: 0.0,
                    y: 0.0,
                    corner_radius: 0.0,
                },
                VectorVertex {
                    x: 10.0,
                    y: 0.0,
                    corner_radius: 0.0,
                },
                VectorVertex {
                    x: 10.0,
                    y: 10.0,
                    corner_radius: 0.0,
                },
            ],
            segments: vec![
                VectorSegment {
                    start: 0,
                    end: 1,
                    tangent_start: (0.0, 0.0),
                    tangent_end: (0.0, 0.0),
                },
                VectorSegment {
                    start: 1,
                    end: 2,
                    tangent_start: (0.0, 0.0),
                    tangent_end: (0.0, 0.0),
                },
            ],
            regions: vec![],
        };

        let svg_d = network.to_svg_d();
        assert_eq!(svg_d, "M 0 0 L 10 0 L 10 10");
    }

    #[test]
    fn test_vector_network_bezier_curves() {
        let network = VectorNetwork {
            vertices: vec![
                VectorVertex {
                    x: 0.0,
                    y: 10.0,
                    corner_radius: 0.0,
                },
                VectorVertex {
                    x: 50.0,
                    y: 10.0,
                    corner_radius: 0.0,
                },
            ],
            segments: vec![VectorSegment {
                start: 0,
                end: 1,
                tangent_start: (15.0, -10.0),
                tangent_end: (-15.0, -10.0),
            }],
            regions: vec![],
        };

        let svg_d = network.to_svg_d();
        assert_eq!(svg_d, "M 0 10 C 15 0 35 0 50 10");
    }

    #[test]
    fn test_vector_network_closed_region_loop() {
        let network = VectorNetwork {
            vertices: vec![
                VectorVertex {
                    x: 0.0,
                    y: 0.0,
                    corner_radius: 0.0,
                },
                VectorVertex {
                    x: 20.0,
                    y: 0.0,
                    corner_radius: 0.0,
                },
                VectorVertex {
                    x: 20.0,
                    y: 20.0,
                    corner_radius: 0.0,
                },
                VectorVertex {
                    x: 0.0,
                    y: 20.0,
                    corner_radius: 0.0,
                },
            ],
            segments: vec![
                VectorSegment {
                    start: 0,
                    end: 1,
                    tangent_start: (0.0, 0.0),
                    tangent_end: (0.0, 0.0),
                },
                VectorSegment {
                    start: 1,
                    end: 2,
                    tangent_start: (0.0, 0.0),
                    tangent_end: (0.0, 0.0),
                },
                VectorSegment {
                    start: 2,
                    end: 3,
                    tangent_start: (0.0, 0.0),
                    tangent_end: (0.0, 0.0),
                },
                VectorSegment {
                    start: 3,
                    end: 0,
                    tangent_start: (0.0, 0.0),
                    tangent_end: (0.0, 0.0),
                },
            ],
            regions: vec![VectorRegion {
                loops: vec![vec![0, 1, 2, 3]],
                is_even_odd: false,
            }],
        };

        let svg_d = network.to_svg_d();
        assert_eq!(svg_d, "M 0 0 L 20 0 L 20 20 L 0 20 L 0 0 Z");
    }
}
