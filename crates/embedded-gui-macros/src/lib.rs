//! Procedural macros for compiling KDL UI layouts into `no_std` Rust code for `embedded-gui`.

use proc_macro::TokenStream;
use std::path::{Component, Path, PathBuf};
use syn::{LitStr, parse_macro_input};

/// Compiles an external KDL UI markup file into typed, zero-allocation Rust code.
///
/// # Example
/// ```ignore
/// embedded_gui_macros::include_gui!("ui/main_screen.kdl");
/// ```
#[proc_macro]
pub fn include_gui(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let rel_path = lit_str.value();

    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            return syn::Error::new(lit_str.span(), "Failed to determine CARGO_MANIFEST_DIR")
                .to_compile_error()
                .into();
        }
    };

    let full_path = manifest_dir.join(&rel_path);
    let kdl_source = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(err) => {
            return syn::Error::new(
                lit_str.span(),
                format!(
                    "Failed to read KDL file at '{}': {}",
                    full_path.display(),
                    err
                ),
            )
            .to_compile_error()
            .into();
        }
    };

    let screen = match embedded_gui_codegen::parse_kdl_screen(&kdl_source) {
        Ok(screen) => screen,
        Err(err) => {
            return syn::Error::new(lit_str.span(), format!("KDL GUI Codegen Error: {}", err))
                .to_compile_error()
                .into();
        }
    };
    let project_root = find_project_root(&full_path);
    let assets = match load_project_assets(&screen, &project_root) {
        Ok(assets) => assets,
        Err(message) => {
            return syn::Error::new(lit_str.span(), message)
                .to_compile_error()
                .into();
        }
    };

    match Ok::<_, embedded_gui_codegen::CodegenError>(
        embedded_gui_codegen::generate_rust_code_with_assets(&screen, &assets),
    ) {
        Ok(rust_code) => match rust_code.parse::<proc_macro2::TokenStream>() {
            Ok(tokens) => tokens.into(),
            Err(err) => syn::Error::new(
                lit_str.span(),
                format!("Failed to tokenize generated code: {}", err),
            )
            .to_compile_error()
            .into(),
        },
        Err(err) => syn::Error::new(lit_str.span(), format!("KDL GUI Codegen Error: {}", err))
            .to_compile_error()
            .into(),
    }
}

fn find_project_root(screen_path: &Path) -> PathBuf {
    let parent = screen_path.parent().unwrap_or_else(|| Path::new("."));
    parent
        .ancestors()
        .find(|dir| dir.join("project.kdl").is_file())
        .unwrap_or(parent)
        .to_path_buf()
}

fn load_image_assets(
    screen: &embedded_gui_codegen::ScreenDef,
    project_root: &Path,
) -> Result<Vec<embedded_gui_codegen::ImageAssetDef>, String> {
    let mut assets = Vec::new();
    for (_, widget) in &screen.grid.children {
        let embedded_gui_codegen::WidgetDef::Image {
            source, mode, tint, ..
        } = widget
        else {
            continue;
        };
        let relative = Path::new(source);
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| matches!(part, Component::ParentDir | Component::RootDir))
        {
            return Err(format!(
                "Image asset path must stay inside the project: {source}"
            ));
        }
        let path = project_root.join(relative);
        let image = image::open(&path)
            .map_err(|err| format!("Failed to decode image asset '{}': {err}", path.display()))?
            .into_rgba8();
        let (width, height) = image.dimensions();
        let tint = tint
            .as_deref()
            .map(parse_color)
            .transpose()?
            .unwrap_or(0xFFFF);
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for rgba in image.into_raw().chunks_exact(4) {
            let alpha = u16::from(rgba[3]);
            let pixel = if mode == "mask" || mode == "mono" {
                let luminance =
                    (u16::from(rgba[0]) * 77 + u16::from(rgba[1]) * 150 + u16::from(rgba[2]) * 29)
                        >> 8;
                if alpha > 0 && luminance < 128 {
                    tint
                } else {
                    0
                }
            } else {
                let r = u16::from(rgba[0]) * alpha / 255;
                let g = u16::from(rgba[1]) * alpha / 255;
                let b = u16::from(rgba[2]) * alpha / 255;
                ((r * 31 / 255) << 11) | ((g * 63 / 255) << 5) | (b * 31 / 255)
            };
            pixels.push(pixel);
        }
        assets.push(embedded_gui_codegen::ImageAssetDef {
            source: source.clone(),
            width,
            height,
            pixels,
        });
    }
    Ok(assets)
}

/// Resolves an asset path against the project root, rejecting anything that
/// escapes it so a KDL file can never pull in arbitrary host files.
fn project_path(project_root: &Path, source: &str) -> Result<PathBuf, String> {
    let relative = Path::new(source);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::RootDir))
    {
        return Err(format!("Asset path must stay inside the project: {source}"));
    }
    Ok(project_root.join(relative))
}

/// Loads every off-disk asset a screen refers to: images, icon parts, fonts,
/// and meshes.
fn load_project_assets(
    screen: &embedded_gui_codegen::ScreenDef,
    project_root: &Path,
) -> Result<embedded_gui_codegen::ProjectAssets, String> {
    use embedded_gui_codegen::{FontBinaryDef, IconAssetDef, MeshAssetDef, WidgetDef, assets};

    let mut out = embedded_gui_codegen::ProjectAssets {
        images: load_image_assets(screen, project_root)?,
        ..Default::default()
    };

    for font in &screen.fonts {
        let path = project_path(project_root, &font.source)?;
        let source = std::fs::read_to_string(&path)
            .map_err(|err| format!("Failed to read font '{}': {err}", path.display()))?;
        let chars = (!font.chars.is_empty()).then_some(font.chars.as_str());
        let data = assets::parse_bdf(&source, chars)
            .map_err(|err| format!("Font '{}': {err}", path.display()))?;
        out.fonts.push(FontBinaryDef {
            name: font.name.clone(),
            font: data,
        });
    }

    for (_, widget) in &screen.grid.children {
        match widget {
            WidgetDef::CompositeIcon {
                parts,
                threshold,
                invert,
                ..
            } => {
                for part in parts {
                    if out.icons.iter().any(|icon| icon.source == part.source) {
                        continue;
                    }
                    let path = project_path(project_root, &part.source)?;
                    let decoded = image::open(&path)
                        .map_err(|err| {
                            format!("Failed to decode icon '{}': {err}", path.display())
                        })?
                        .into_rgba8();
                    let (width, height) = decoded.dimensions();
                    out.icons.push(IconAssetDef {
                        source: part.source.clone(),
                        bitmap: assets::mono_from_rgba(
                            width,
                            height,
                            &decoded.into_raw(),
                            *threshold,
                            *invert,
                        ),
                    });
                }
            }
            WidgetDef::Mesh3d { source, .. } => {
                if out.meshes.iter().any(|mesh| mesh.source == *source) {
                    continue;
                }
                let path = project_path(project_root, source)?;
                let bytes = std::fs::read(&path)
                    .map_err(|err| format!("Failed to read mesh '{}': {err}", path.display()))?;
                let mut mesh = assets::parse_mesh(source, &bytes)
                    .map_err(|err| format!("Mesh '{}': {err}", path.display()))?;
                mesh.normalize();
                out.meshes.push(MeshAssetDef {
                    source: source.clone(),
                    mesh,
                });
            }
            _ => {}
        }
    }

    Ok(out)
}

fn parse_color(value: &str) -> Result<u16, String> {
    let (r, g, b) = match value {
        "accent" => (0, 255, 255),
        "success" => (0, 255, 0),
        "danger" => (255, 0, 0),
        "primary" | "white" => (255, 255, 255),
        "black" | "background" => (0, 0, 0),
        hex if hex.len() == 7 && hex.starts_with('#') => {
            let r = u8::from_str_radix(&hex[1..3], 16)
                .map_err(|_| format!("Invalid image tint: {value}"))?;
            let g = u8::from_str_radix(&hex[3..5], 16)
                .map_err(|_| format!("Invalid image tint: {value}"))?;
            let b = u8::from_str_radix(&hex[5..7], 16)
                .map_err(|_| format!("Invalid image tint: {value}"))?;
            (r, g, b)
        }
        _ => return Err(format!("Unknown image tint: {value}")),
    };
    Ok(((u16::from(r) * 31 / 255) << 11)
        | ((u16::from(g) * 63 / 255) << 5)
        | (u16::from(b) * 31 / 255))
}

/// Compiles an inline KDL string literal into typed Rust code.
///
/// # Example
/// ```ignore
/// embedded_gui_macros::gui_kdl!(r#"
/// screen id="Thermostat" width=320 height=240 {
///     grid cols="140px 1fr" rows="24px 1fr 48px" gap=6 {
///         banner col=0 row=0 col_span=2 text="Smart Thermostat"
///         spinbox id="Temp" col=0 row=1 min=100 max=350 value=215
///         scale id="Gauge" col=1 row=1 mode="radial" min=10.0 max=40.0 value=22.5
///     }
/// }
/// "#);
/// ```
#[proc_macro]
pub fn gui_kdl(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let kdl_source = lit_str.value();

    match embedded_gui_codegen::compile_kdl_to_rust(&kdl_source) {
        Ok(rust_code) => match rust_code.parse::<proc_macro2::TokenStream>() {
            Ok(tokens) => tokens.into(),
            Err(err) => syn::Error::new(
                lit_str.span(),
                format!("Failed to tokenize generated code: {}", err),
            )
            .to_compile_error()
            .into(),
        },
        Err(err) => syn::Error::new(lit_str.span(), format!("KDL GUI Codegen Error: {}", err))
            .to_compile_error()
            .into(),
    }
}
