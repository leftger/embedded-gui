//! Font and Vector / PDC Asset Browser and Glyph Inspector.

use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Stroke, StrokeKind, Vec2};

pub struct FontDef {
    pub name: &'static str,
    pub dimensions: &'static str,
    pub bpp: u8,
    pub size_bytes: usize,
    pub sample_text: &'static str,
}

pub const BUILTIN_FONTS: &[FontDef] = &[
    FontDef {
        name: "Font6x8 (Monospaced Micro)",
        dimensions: "6×8 px",
        bpp: 1,
        size_bytes: 768,
        sample_text: "CPU: 240MHz  RAM: 512KB  TEMP: 42.5°C",
    },
    FontDef {
        name: "Font8x16 (Standard Monospaced)",
        dimensions: "8×16 px",
        bpp: 1,
        size_bytes: 2048,
        sample_text: "VOLTS: 3.31V  CURR: 120mA  FREQ: 60Hz",
    },
    FontDef {
        name: "Font12x16 (Medium Bold Display)",
        dimensions: "12×16 px",
        bpp: 1,
        size_bytes: 3072,
        sample_text: "72 BPM  120/80 mmHg  99% SpO2",
    },
    FontDef {
        name: "ProFont18 (High Legibility Numeric)",
        dimensions: "14×24 px",
        bpp: 1,
        size_bytes: 4608,
        sample_text: "SPEED: 68 MPH  RPM: 4,500",
    },
];

pub struct PdcIconDef {
    pub name: &'static str,
    pub category: &'static str,
    pub icon_char: &'static str,
    pub kdl_snippet: &'static str,
}

pub const BUILTIN_ICONS: &[PdcIconDef] = &[
    PdcIconDef {
        name: "Battery Status",
        category: "Power",
        icon_char: "🔋",
        kdl_snippet: "label text=\"🔋 98%\"",
    },
    PdcIconDef {
        name: "Power Lightning",
        category: "Power",
        icon_char: "⚡",
        kdl_snippet: "button text=\"⚡ PWR\"",
    },
    PdcIconDef {
        name: "Wireless Signal",
        category: "Connectivity",
        icon_char: "📶",
        kdl_snippet: "status_bar time=\"12:00\"",
    },
    PdcIconDef {
        name: "Bluetooth Mesh",
        category: "Connectivity",
        icon_char: "ᛒ",
        kdl_snippet: "label text=\"ᛒ CONNECTED\"",
    },
    PdcIconDef {
        name: "ECG Heart Rate",
        category: "Medical",
        icon_char: "❤️",
        kdl_snippet: "label text=\"❤️ 72 BPM\"",
    },
    PdcIconDef {
        name: "Temperature",
        category: "Sensors",
        icon_char: "🌡",
        kdl_snippet: "scale mode=\"radial\" value=22.5",
    },
    PdcIconDef {
        name: "Speedometer",
        category: "Industrial",
        icon_char: "⏱",
        kdl_snippet: "scale mode=\"radial\" min=0 max=160",
    },
    PdcIconDef {
        name: "Gear Settings",
        category: "System",
        icon_char: "⚙️",
        kdl_snippet: "button text=\"⚙️ SETUP\"",
    },
    PdcIconDef {
        name: "Warning Shield",
        category: "System",
        icon_char: "⚠️",
        kdl_snippet: "button text=\"⚠️ ALARM\" style=\"danger\"",
    },
    PdcIconDef {
        name: "Play Trigger",
        category: "Media",
        icon_char: "▶",
        kdl_snippet: "button text=\"▶ START\" style=\"accent\"",
    },
    PdcIconDef {
        name: "Pause Trigger",
        category: "Media",
        icon_char: "⏸",
        kdl_snippet: "button text=\"⏸ PAUSE\"",
    },
    PdcIconDef {
        name: "Stop / Emergency",
        category: "Industrial",
        icon_char: "⏹",
        kdl_snippet: "button text=\"⏹ ESTOP\" style=\"danger\"",
    },
];

pub fn render_asset_browser(ui: &mut egui::Ui, copied_toast: &mut Option<(String, f32)>) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("🔤 Embedded Fonts & Glyph Inspector");
        ui.label(egui::RichText::new("Pre-compiled binary bitmap fonts optimized for zero-allocation no_std execution.").weak());
        ui.separator();

        for font in BUILTIN_FONTS {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(font.name).strong());
                    ui.separator();
                    ui.label(format!("Size: {} ({} bpp)", font.dimensions, font.bpp));
                    ui.separator();
                    ui.label(format!("Flash ROM: ~{} B", font.size_bytes));
                });

                // Simulated LCD Font Raster Preview Card
                let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 34.0), egui::Sense::hover());
                let rect = response.rect;
                painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(18, 22, 28));
                painter.rect_stroke(rect, CornerRadius::same(3), Stroke::new(1.0f32, Color32::from_rgb(45, 55, 70)), StrokeKind::Inside);
                painter.text(
                    Pos2::new(rect.min.x + 8.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    font.sample_text,
                    FontId::monospace(12.5),
                    Color32::from_rgb(80, 220, 160),
                );
            });
            ui.add_space(4.0);
        }

        ui.add_space(12.0);
        ui.heading("🎨 PDC Vector Icons Library");
        ui.label(egui::RichText::new("PDC vector graphics and symbol tokens for industrial, medical, and consumer devices.").weak());
        ui.separator();

        egui::Grid::new("icons_grid")
            .num_columns(4)
            .spacing([12.0, 12.0])
            .show(ui, |ui| {
                for (i, icon) in BUILTIN_ICONS.iter().enumerate() {
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.set_width(120.0);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new(icon.icon_char).size(26.0));
                                ui.label(egui::RichText::new(icon.name).strong().size(10.5));
                                ui.label(egui::RichText::new(icon.category).weak().size(9.0));

                                if ui.small_button("📋 Copy KDL").clicked() {
                                    ui.ctx().copy_text(icon.kdl_snippet.to_string());
                                    *copied_toast = Some((format!("Copied '{}'", icon.name), 2.0));
                                }
                            });
                        });
                    });

                    if (i + 1) % 4 == 0 {
                        ui.end_row();
                    }
                }
            });
    });
}
