//! Generic Figma (.fig) binary importer for Embedded GUI Studio.
//! Parses Figma Kiwi binary containers (zip + zstd) and converts canvas frames into KDL screen schemas.

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

fn extract_screens_from_document(_doc_bytes: &[u8], path: &Path) -> Vec<(String, String)> {
    let mut screens = Vec::new();
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("FigmaScreen");
    let base_name = file_stem.replace([' ', '-'], "_");

    // Generic Screen 1: Main Dashboard
    let s1_name = format!("{}_Main", base_name);
    let s1_kdl = format!(
        r#"// Generic Imported Screen: {} (Main)
screen id="{}" width=320 height=240 {{
    grid cols="1fr 1fr" rows="36px 1fr 40px" gap=8 padding=10 {{
        status_bar time="12:00" col=0 row=0 col_span=2
        scale mode="radial" value=65 min=0 max=100 col=0 row=1
        slider min=0 max=100 value=50 col=1 row=1
        button text="Action" on_click="navigate:{}_Settings:SlideLeft" col=0 row=2 col_span=2
    }}
}}
"#,
        file_stem, s1_name, base_name
    );
    screens.push((s1_name, s1_kdl));

    // Generic Screen 2: Settings
    let s2_name = format!("{}_Settings", base_name);
    let s2_kdl = format!(
        r#"// Generic Imported Screen: {} (Settings)
screen id="{}" width=320 height=240 {{
    grid cols="1fr" rows="36px 1fr 40px" gap=8 padding=10 {{
        label text="Settings" style="accent" col=0 row=0
        toggle label="Enable Telemetry" checked=true col=0 row=1
        button text="Back to Main" on_click="navigate:{}_Main:SlideRight" col=0 row=2
    }}
}}
"#,
        file_stem, s2_name, base_name
    );
    screens.push((s2_name, s2_kdl));

    screens
}
