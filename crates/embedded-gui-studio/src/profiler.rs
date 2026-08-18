//! Embedded Memory and Asset Profiler for hardware targets.
//! Computes static SRAM and Flash budgets for framebuffers, BDF fonts, 3D meshes,
//! and widget instances against microcontroller limits.

use eframe::egui::{self, Color32, ProgressBar, RichText};
use embedded_gui_codegen::ScreenDef;

use crate::types::HardwareProfile;

/// Detailed breakdown of memory consumption for an authored screen.
#[derive(Debug, Clone, Default)]
pub struct MemoryProfile {
    pub framebuffer_sram_bytes: usize,
    pub line_buffer_sram_bytes: usize,
    pub font_glyphs_flash_bytes: usize,
    pub mesh_geometry_flash_bytes: usize,
    pub composite_icons_flash_bytes: usize,
    pub widget_instances_ram_bytes: usize,
    pub total_estimated_sram_bytes: usize,
    pub total_estimated_flash_bytes: usize,
}

pub fn analyze_screen_memory(screen: &ScreenDef, profile: &HardwareProfile) -> MemoryProfile {
    let bpp = profile.bpp();
    let width = screen.width as usize;
    let height = screen.height as usize;

    let framebuffer_sram_bytes = if bpp == 1 {
        (width * height).div_ceil(8)
    } else {
        width * height * 2 // RGB565 = 2 bytes per pixel
    };

    let line_buffer_sram_bytes = if bpp == 1 {
        (width * 8).div_ceil(8)
    } else {
        width * 8 * 2 // 8-line partial slice
    };

    let mut font_glyphs_flash_bytes = 0;
    let mut mesh_geometry_flash_bytes = 0;
    let mut composite_icons_flash_bytes = 0;
    let mut widget_count = 0;

    for (_placement, widget) in &screen.grid.children {
        widget_count += 1;
        match widget {
            embedded_gui_codegen::WidgetDef::Mesh3d { source, .. } => {
                // Approximate 32 bytes per vertex + index buffers for embedded meshes
                if !source.is_empty() {
                    mesh_geometry_flash_bytes += 4096; // typical low-poly mesh size
                }
            }
            embedded_gui_codegen::WidgetDef::CompositeIcon { parts, .. } => {
                composite_icons_flash_bytes += parts.len() * 16;
            }
            embedded_gui_codegen::WidgetDef::Label { text, .. } => {
                font_glyphs_flash_bytes += text.len() * 16;
            }
            _ => {}
        }
    }

    // Estimated 48 bytes per active widget instance struct
    let widget_instances_ram_bytes = widget_count * 48;

    let total_estimated_sram_bytes = framebuffer_sram_bytes + widget_instances_ram_bytes;
    let total_estimated_flash_bytes = font_glyphs_flash_bytes
        + mesh_geometry_flash_bytes
        + composite_icons_flash_bytes
        + (widget_count * 128);

    MemoryProfile {
        framebuffer_sram_bytes,
        line_buffer_sram_bytes,
        font_glyphs_flash_bytes,
        mesh_geometry_flash_bytes,
        composite_icons_flash_bytes,
        widget_instances_ram_bytes,
        total_estimated_sram_bytes,
        total_estimated_flash_bytes,
    }
}

pub fn render_profiler_panel(ui: &mut egui::Ui, screen: &ScreenDef, profile: &HardwareProfile) {
    let mem = analyze_screen_memory(screen, profile);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("⚡ Embedded Memory & Resource Profiler");
        ui.label(
            RichText::new(
                "Real-time SRAM and Flash budget estimation for bare-metal microcontrollers.",
            )
            .weak(),
        );
        ui.separator();

        // Top metric summary cards
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Total SRAM Budget").strong());
                    ui.heading(format!("{:.1} KB", mem.total_estimated_sram_bytes as f32 / 1024.0));
                    ui.label(format!("Full Framebuffer: {} B", mem.framebuffer_sram_bytes));
                    ui.label(format!("Line Buffer (8 lines): {} B", mem.line_buffer_sram_bytes));
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Total Flash / ROM Budget").strong());
                    ui.heading(format!("{:.1} KB", mem.total_estimated_flash_bytes as f32 / 1024.0));
                    ui.label(format!("Fonts & Bitmaps: {} B", mem.font_glyphs_flash_bytes));
                    ui.label(format!("3D Meshes & Icons: {} B", mem.mesh_geometry_flash_bytes + mem.composite_icons_flash_bytes));
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Active UI Nodes").strong());
                    ui.heading(format!("{}", screen.grid.children.len()));
                    ui.label(format!("Grid Tracks: {}c × {}r", screen.grid.cols.len(), screen.grid.rows.len()));
                    ui.label(format!("Widget State RAM: {} B", mem.widget_instances_ram_bytes));
                });
            });
        });

        ui.add_space(16.0);
        ui.heading("📊 Target Microcontroller SRAM Limits");
        ui.separator();

        let targets = [
            ("ARM Cortex-M0+ (16 KB SRAM)", 16 * 1024),
            ("ARM Cortex-M4 (64 KB SRAM)", 64 * 1024),
            ("ESP32-S3 (512 KB SRAM)", 512 * 1024),
            ("STM32-H7 (1024 KB SRAM)", 1024 * 1024),
        ];

        for (name, max_sram) in targets {
            let ratio = (mem.total_estimated_sram_bytes as f32 / max_sram as f32).clamp(0.0, 1.0);
            let pct = ratio * 100.0;
            let bar_color = if pct > 90.0 {
                Color32::from_rgb(230, 70, 70)
            } else if pct > 60.0 {
                Color32::from_rgb(230, 180, 50)
            } else {
                Color32::from_rgb(70, 200, 120)
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(name).strong());
                ui.label(format!("{:.1}% ({}/{} KB)", pct, mem.total_estimated_sram_bytes / 1024, max_sram / 1024));
            });

            let progress = ProgressBar::new(ratio)
                .fill(bar_color)
                .animate(false);
            ui.add(progress);
            ui.add_space(6.0);
        }

        ui.add_space(16.0);
        ui.heading("💡 Optimization Recommendations");
        ui.separator();

        if mem.total_estimated_sram_bytes > 32 * 1024 {
            ui.colored_label(
                Color32::from_rgb(240, 180, 60),
                "⚠️ Full framebuffer exceeds 32 KB. On resource-constrained Cortex-M0/M4 chips, enable the LineBufferRenderer streaming backend to drop SRAM usage to < 2 KB.",
            );
        } else {
            ui.colored_label(
                Color32::from_rgb(80, 220, 120),
                "✓ Memory footprint fits comfortably within typical embedded microcontroller SRAM boundaries.",
            );
        }

        if mem.mesh_geometry_flash_bytes > 0 {
            ui.label("• 3D mesh assets are compiled to static byte slices in Flash and streamed via caller-owned Z-buffers.");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_gui_codegen::parse_kdl_screen;

    #[test]
    fn test_memory_profile_calculation() {
        let kdl = "screen id=\"Test\" width=320 height=240 {\n    grid cols=\"1fr\" rows=\"1fr\" {\n        label text=\"Sample\"\n    }\n}\n";
        let screen = parse_kdl_screen(kdl).unwrap();
        let profile = analyze_screen_memory(&screen, &HardwareProfile::Esp32S3Box);

        // 320 * 240 * 2 = 153,600 bytes
        assert_eq!(profile.framebuffer_sram_bytes, 320 * 240 * 2);
        // Line buffer for 320 * 8 * 2 = 5,120 bytes
        assert_eq!(profile.line_buffer_sram_bytes, 320 * 8 * 2);
        assert!(profile.total_estimated_sram_bytes > 153_600);
    }
}
