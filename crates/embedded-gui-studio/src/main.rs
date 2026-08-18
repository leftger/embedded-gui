//! Embedded GUI Studio
//! Cross-platform interactive designer, visual inspector, live animation previewer, and Rust code generator for embedded-gui.

mod app;
mod assets;
mod bridge;
mod command_palette;
mod curve_visualizer;
mod device_link;
mod exporter;
mod figma_importer;
mod inspector;
mod layout;
mod live_render;
mod playground;
mod presets;
mod profiler;
mod project;
mod syntax;
mod theme;
mod types;

use app::EmbeddedGuiStudio;
use eframe::egui;

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidate_paths = [
        // Linux (Noto, DejaVu, FreeFonts)
        "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansMath-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
        // macOS
        "/System/Library/Fonts/Apple Symbols.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode.ttf",
        "/System/Library/Fonts/SFNS.ttf",
        // Windows
        "C:\\Windows\\Fonts\\seguisym.ttf",
        "C:\\Windows\\Fonts\\seguiemj.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];

    let mut added_any = false;
    for (i, path) in candidate_paths.iter().enumerate() {
        if let Ok(bytes) = std::fs::read(path) {
            let font_key = format!("fallback_symbol_font_{i}");
            fonts.font_data.insert(
                font_key.clone(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.push(font_key.clone());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.push(font_key);
            }
            added_any = true;
        }
    }

    if added_any {
        ctx.set_fonts(fonts);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 780.0])
            .with_min_inner_size([800.0, 520.0])
            .with_title("Embedded GUI Studio - Visual Inspector & KDL Codegen"),
        ..Default::default()
    };

    eframe::run_native(
        "Embedded GUI Studio",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(EmbeddedGuiStudio::default()))
        }),
    )
}
