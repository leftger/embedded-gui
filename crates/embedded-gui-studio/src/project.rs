//! On-disk multi-screen KDL projects.
//!
//! A project is a directory containing a `project.kdl` manifest and the screen
//! files it references (usually under `screens/`). Studio loads the whole set
//! into tabs and can write it back in place.

use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{DisplayTheme, HardwareProfile};

/// Manifest filename expected at the project root.
pub const MANIFEST_NAME: &str = "project.kdl";

/// Parsed project ready to load into Studio.
#[derive(Debug, Clone)]
pub struct GuiProject {
    pub root: PathBuf,
    pub name: String,
    pub hardware_profile: HardwareProfile,
    pub theme: Option<DisplayTheme>,
    /// `(tab_name, kdl_source)` in manifest order.
    pub screens: Vec<(String, String)>,
    /// Relative paths from `project.kdl`, parallel to `screens`.
    pub screen_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectError(pub String);

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl GuiProject {
    /// Loads a project from a directory or a `project.kdl` path.
    pub fn load(path: &Path) -> Result<Self, ProjectError> {
        let (root, manifest_path) = resolve_project_paths(path)?;
        let manifest = fs::read_to_string(&manifest_path).map_err(|e| {
            ProjectError(format!("failed to read {}: {e}", manifest_path.display()))
        })?;
        let meta = parse_manifest(&manifest)?;

        let mut screens = Vec::with_capacity(meta.screens.len());
        let mut screen_files = Vec::with_capacity(meta.screens.len());
        for entry in &meta.screens {
            let screen_path = root.join(&entry.file);
            let source = fs::read_to_string(&screen_path).map_err(|e| {
                ProjectError(format!(
                    "failed to read screen '{}': {e}",
                    screen_path.display()
                ))
            })?;
            screens.push((entry.id.clone(), source));
            screen_files.push(entry.file.clone());
        }
        if screens.is_empty() {
            return Err(ProjectError(
                "project.kdl must list at least one screen".into(),
            ));
        }

        Ok(Self {
            root,
            name: meta.name,
            hardware_profile: meta.hardware_profile,
            theme: meta.theme,
            screens,
            screen_files,
        })
    }

    /// Writes `project.kdl` and every screen file under `root`.
    ///
    /// Screen files are written to `screens/{snake_id}.kdl` unless an existing
    /// relative path is supplied via `screen_files` (same order as `screens`).
    pub fn save(
        root: &Path,
        name: &str,
        hardware_profile: HardwareProfile,
        theme: DisplayTheme,
        screens: &[(String, String)],
        screen_files: Option<&[String]>,
    ) -> Result<PathBuf, ProjectError> {
        if screens.is_empty() {
            return Err(ProjectError("cannot save an empty project".into()));
        }
        fs::create_dir_all(root.join("screens"))
            .map_err(|e| ProjectError(format!("failed to create screens/: {e}")))?;

        let mut file_rels = Vec::with_capacity(screens.len());
        for (i, (id, source)) in screens.iter().enumerate() {
            let rel = screen_files
                .and_then(|files| files.get(i).cloned())
                .unwrap_or_else(|| format!("screens/{}.kdl", to_snake_case(id)));
            let abs = root.join(&rel);
            if let Some(parent) = abs.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    ProjectError(format!("failed to create {}: {e}", parent.display()))
                })?;
            }
            fs::write(&abs, source)
                .map_err(|e| ProjectError(format!("failed to write {}: {e}", abs.display())))?;
            file_rels.push(rel);
        }

        let manifest = format_manifest(name, hardware_profile, theme, screens, &file_rels);
        let manifest_path = root.join(MANIFEST_NAME);
        fs::write(&manifest_path, manifest).map_err(|e| {
            ProjectError(format!("failed to write {}: {e}", manifest_path.display()))
        })?;
        Ok(manifest_path)
    }
}

#[derive(Debug)]
struct ManifestScreen {
    id: String,
    file: String,
}

#[derive(Debug)]
struct ManifestMeta {
    name: String,
    hardware_profile: HardwareProfile,
    theme: Option<DisplayTheme>,
    screens: Vec<ManifestScreen>,
}

fn resolve_project_paths(path: &Path) -> Result<(PathBuf, PathBuf), ProjectError> {
    if path.is_file() {
        let root = path
            .parent()
            .ok_or_else(|| ProjectError("invalid project path".into()))?
            .to_path_buf();
        Ok((root, path.to_path_buf()))
    } else if path.is_dir() {
        let manifest = path.join(MANIFEST_NAME);
        if !manifest.is_file() {
            return Err(ProjectError(format!(
                "no {MANIFEST_NAME} in {}",
                path.display()
            )));
        }
        Ok((path.to_path_buf(), manifest))
    } else {
        Err(ProjectError(format!("path not found: {}", path.display())))
    }
}

fn parse_manifest(src: &str) -> Result<ManifestMeta, ProjectError> {
    let doc: kdl::KdlDocument = src
        .parse()
        .map_err(|e| ProjectError(format!("invalid project.kdl: {e}")))?;
    let root = doc
        .nodes()
        .iter()
        .find(|n| n.name().value() == "project")
        .ok_or_else(|| ProjectError("project.kdl must contain a top-level project node".into()))?;

    let name = entry_str(root, "name").unwrap_or_else(|| "Untitled".to_string());
    let hardware_profile = resolve_profile(root);
    let theme = entry_str(root, "theme").and_then(|t| parse_theme(&t));

    let mut screens = Vec::new();
    if let Some(children) = root.children() {
        for node in children.nodes() {
            if node.name().value() != "screen" {
                continue;
            }
            let id = entry_str(node, "id")
                .ok_or_else(|| ProjectError("each project screen needs id=\"...\"".into()))?;
            let file = entry_str(node, "file")
                .ok_or_else(|| ProjectError(format!("screen id=\"{id}\" needs file=\"...\"")))?;
            screens.push(ManifestScreen { id, file });
        }
    }
    Ok(ManifestMeta {
        name,
        hardware_profile,
        theme,
        screens,
    })
}

fn resolve_profile(node: &kdl::KdlNode) -> HardwareProfile {
    if let Some(panel) = entry_str(node, "panel") {
        return HardwareProfile::from_panel_slug(&panel).unwrap_or(HardwareProfile::Custom);
    }
    match (entry_u32(node, "width"), entry_u32(node, "height")) {
        (Some(width), Some(height)) => HardwareProfile::from_dimensions(width, height),
        _ => HardwareProfile::Custom,
    }
}

fn format_manifest(
    name: &str,
    profile: HardwareProfile,
    theme: DisplayTheme,
    screens: &[(String, String)],
    files: &[String],
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "project name={:?} theme={:?} ",
        name,
        theme_slug(theme)
    ));
    match profile {
        HardwareProfile::Custom => {}
        HardwareProfile::Detected { width, height } => {
            out.push_str(&format!("width={width} height={height} "));
        }
        other => {
            if let Some(slug) = other.panel_slug() {
                out.push_str(&format!("panel={slug:?} "));
            } else if let Some((w, h)) = other.dimensions() {
                out.push_str(&format!("width={w} height={h} "));
            }
        }
    }
    out.push_str("{\n");
    for (i, (id, _)) in screens.iter().enumerate() {
        let file = &files[i];
        out.push_str(&format!("    screen id={id:?} file={file:?}\n"));
    }
    out.push_str("}\n");
    out
}

fn entry_str(node: &kdl::KdlNode, key: &str) -> Option<String> {
    node.get(key).and_then(|e| match e.value() {
        kdl::KdlValue::String(s) | kdl::KdlValue::RawString(s) => Some(s.to_string()),
        _ => None,
    })
}

fn entry_u32(node: &kdl::KdlNode, key: &str) -> Option<u32> {
    node.get(key).and_then(|e| match e.value() {
        kdl::KdlValue::Base10(i)
        | kdl::KdlValue::Base2(i)
        | kdl::KdlValue::Base8(i)
        | kdl::KdlValue::Base16(i) => u32::try_from(*i).ok(),
        kdl::KdlValue::String(s) | kdl::KdlValue::RawString(s) => s.parse().ok(),
        _ => None,
    })
}

fn parse_theme(s: &str) -> Option<DisplayTheme> {
    match s.to_ascii_lowercase().as_str() {
        "dark" | "dark_tft" => Some(DisplayTheme::DarkTft),
        "light" | "light_tft" => Some(DisplayTheme::LightTft),
        "amber" | "amber_phosphor" => Some(DisplayTheme::AmberPhosphor),
        "emerald" | "emerald_green" => Some(DisplayTheme::EmeraldGreen),
        "mono" | "monochrome" | "monochrome_oled" => Some(DisplayTheme::MonochromeOled),
        "soft_ui" | "soft" => Some(DisplayTheme::SoftUi),
        _ => None,
    }
}

fn theme_slug(theme: DisplayTheme) -> &'static str {
    match theme {
        DisplayTheme::DarkTft => "dark",
        DisplayTheme::LightTft => "light",
        DisplayTheme::AmberPhosphor => "amber",
        DisplayTheme::EmeraldGreen => "emerald",
        DisplayTheme::MonochromeOled => "mono",
        DisplayTheme::SoftUi => "soft_ui",
    }
}

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else if c == '-' || c == ' ' {
            out.push('_');
        } else {
            out.push(c);
        }
    }
    if out.is_empty() { "screen".into() } else { out }
}

/// File picker that loads a project by selecting its `project.kdl` manifest.
///
/// A file picker is used rather than a folder picker because macOS Finder does
/// not let you select a file inside a folder-only dialog. [`GuiProject::load`]
/// accepts either the manifest file or its directory.
pub fn open_project_dialog() -> Option<Result<GuiProject, ProjectError>> {
    let file = rfd::FileDialog::new()
        .set_title("Open KDL Project (select project.kdl)")
        .add_filter("KDL Project Manifest", &["kdl"])
        .set_file_name(MANIFEST_NAME)
        .pick_file()?;
    Some(GuiProject::load(&file))
}

/// Folder picker that saves the current Studio tabs as a project.
pub fn save_project_dialog(
    name: &str,
    hardware_profile: HardwareProfile,
    theme: DisplayTheme,
    screens: &[(String, String)],
) -> Option<Result<PathBuf, ProjectError>> {
    let folder = rfd::FileDialog::new()
        .set_title("Save KDL Project Folder")
        .pick_folder()?;
    Some(GuiProject::save(
        &folder,
        name,
        hardware_profile,
        theme,
        screens,
        None,
    ))
}

/// Saves into an existing project root, preserving relative screen paths when possible.
pub fn save_project_to(
    root: &Path,
    name: &str,
    hardware_profile: HardwareProfile,
    theme: DisplayTheme,
    screens: &[(String, String)],
    screen_files: Option<&[String]>,
) -> Result<PathBuf, ProjectError> {
    GuiProject::save(root, name, hardware_profile, theme, screens, screen_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn roundtrip_project_with_ssd1357_panel() {
        let dir = tempfile_dir();
        let screens = vec![
            (
                "Status".to_string(),
                r#"screen id="Status" width=96 height=64 {
    grid cols="1fr" rows="1fr" gap=0 padding=0 {
        label text="FUNC TEST" style="accent" col=0 row=0
    }
}
"#
                .to_string(),
            ),
            (
                "Menu".to_string(),
                r#"screen id="Menu" width=96 height=64 {
    grid cols="1fr" rows="14px 1fr 12px" gap=0 padding=0 {
        label text="SETTINGS" style="accent" col=0 row=0
        label text="Item" col=0 row=1
        label text="hints" style="dim" col=0 row=2
    }
}
"#
                .to_string(),
            ),
        ];
        GuiProject::save(
            &dir,
            "ssd1357-demo",
            HardwareProfile::Ssd1357,
            DisplayTheme::DarkTft,
            &screens,
            None,
        )
        .unwrap();

        let loaded = GuiProject::load(&dir).unwrap();
        assert_eq!(loaded.name, "ssd1357-demo");
        assert_eq!(loaded.hardware_profile, HardwareProfile::Ssd1357);
        assert_eq!(loaded.theme, Some(DisplayTheme::DarkTft));
        assert_eq!(loaded.screens.len(), 2);
        assert_eq!(loaded.screens[0].0, "Status");
        assert!(loaded.screens[0].1.contains("FUNC TEST"));
        assert_eq!(loaded.screens[1].0, "Menu");
    }

    #[test]
    fn panel_slug_maps_known_controllers() {
        assert_eq!(
            HardwareProfile::from_panel_slug("ssd1357"),
            Some(HardwareProfile::Ssd1357)
        );
        assert_eq!(
            HardwareProfile::from_panel_slug("SSD1306"),
            Some(HardwareProfile::Ssd1306Oled)
        );
    }

    fn tempfile_dir() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "embedded-gui-studio-project-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        let mut marker = fs::File::create(path.join(".keep")).unwrap();
        let _ = write!(marker, "x");
        path
    }
}
