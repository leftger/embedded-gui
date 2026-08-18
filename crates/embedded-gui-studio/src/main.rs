//! Embedded GUI Studio
//! Cross-platform interactive designer, visual inspector, live animation previewer, and Rust code generator for embedded-gui.

mod app;
mod assets;
mod bridge;
mod curve_visualizer;
mod device_link;
mod exporter;
mod figma_importer;
mod inspector;
mod layout;
mod live_render;
mod presets;
mod syntax;
mod theme;
mod types;

use app::EmbeddedGuiStudio;
use eframe::egui;

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
        Box::new(|_cc| Ok(Box::new(EmbeddedGuiStudio::default()))),
    )
}
