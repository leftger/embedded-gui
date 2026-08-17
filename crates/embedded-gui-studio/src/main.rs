//! Embedded GUI Studio
//! Cross-platform interactive designer, visual inspector, live animation previewer, and Rust code generator for embedded-gui.

mod app;
mod assets;
mod bridge;
mod curve_visualizer;
mod exporter;
mod inspector;
mod layout;
mod presets;
mod renderer;
mod syntax;
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
