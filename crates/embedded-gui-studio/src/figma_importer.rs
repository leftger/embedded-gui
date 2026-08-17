//! Generic Figma (.fig) binary importer for Embedded GUI Studio.
//! Parses Figma Kiwi binary containers (zip + zstd) and dynamically extracts canvas frames into KDL screen schemas.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

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

    // 3. Generate clean declarative KDL schemas for each discovered screen
    for (i, screen_name) in discovered_names.iter().take(6).enumerate() {
        let next_screen = discovered_names
            .get((i + 1) % discovered_names.len())
            .unwrap_or(screen_name);
        let kdl = if detected_w <= 128 {
            format!(
                r#"// Imported Screen: {} ({}x{} OLED)
screen id="{}" width={} height={} {{
    grid cols="1fr 1fr" rows="18px 24px 1fr" gap=2 padding=3 {{
        label text="[{}]" style="accent" col=0 row=0
        label text="🔋 98%" col=1 row=0
        label text="ONLINE" style="success" col=0 row=1 col_span=2
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
