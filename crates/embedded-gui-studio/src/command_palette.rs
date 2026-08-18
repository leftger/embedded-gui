//! Quick-Action Command Palette (Ctrl+K / Cmd+K) for Studio.
//! Provides a fast fuzzy search launcher to switch screens, insert widgets, toggle themes,
//! and trigger studio actions.

use crate::app::EmbeddedGuiStudio;
use crate::types::{DisplayTheme, StudioMode};
use eframe::egui::{self, Color32, Key, RichText};

pub enum ActionType {
    SwitchScreen(usize),
    SwitchTheme(DisplayTheme),
    InsertWidget(&'static str),
    ToggleMode,
    ToggleRulers,
}

pub fn render_command_palette(app: &mut EmbeddedGuiStudio, ctx: &egui::Context) {
    if !app.command_palette_open {
        return;
    }

    let mut close = false;
    let mut selected_action: Option<ActionType> = None;

    egui::Window::new("🔍 Command Palette")
        .collapsible(false)
        .resizable(false)
        .fixed_size([480.0, 320.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, -100.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(18.0));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut app.command_query)
                        .hint_text("Type a command or screen name... (ESC to close)")
                        .desired_width(400.0),
                );
                response.request_focus();

                if ui.button("✕").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                    close = true;
                }
            });

            ui.separator();

            let query = app.command_query.to_lowercase();

            egui::ScrollArea::vertical()
                .max_height(240.0)
                .show(ui, |ui| {
                    // Screen jumping actions
                    for (i, (name, _)) in app.project_screens.iter().enumerate() {
                        let full_label = format!("Jump to Screen: {}", name);
                        if query.is_empty() || full_label.to_lowercase().contains(&query) {
                            ui.horizontal(|ui| {
                                ui.label("📱");
                                if ui.selectable_label(false, &full_label).clicked() {
                                    selected_action = Some(ActionType::SwitchScreen(i));
                                }
                            });
                        }
                    }

                    // Theme switcher actions
                    let themes = [
                        ("Theme: Dark TFT", DisplayTheme::DarkTft, "🌙"),
                        ("Theme: Light TFT", DisplayTheme::LightTft, "☀️"),
                        ("Theme: Amber CRT", DisplayTheme::AmberPhosphor, "📟"),
                        ("Theme: Emerald Matrix", DisplayTheme::EmeraldGreen, "🟢"),
                        ("Theme: Monochrome OLED", DisplayTheme::MonochromeOled, "⚪"),
                    ];
                    for (tname, theme, icon) in themes {
                        if query.is_empty() || tname.to_lowercase().contains(&query) {
                            ui.horizontal(|ui| {
                                ui.label(icon);
                                if ui.selectable_label(false, tname).clicked() {
                                    selected_action = Some(ActionType::SwitchTheme(theme));
                                }
                            });
                        }
                    }

                    // Widget insertion actions
                    let widgets = [
                        ("Insert: Button", "        button text=\"NEW BTN\"\n", "🔘"),
                        ("Insert: Toggle", "        toggle checked=true\n", "☑"),
                        (
                            "Insert: Slider",
                            "        slider min=0 max=100 value=50\n",
                            "🎚",
                        ),
                        (
                            "Insert: Spinbox",
                            "        spinbox min=0 max=100 value=25\n",
                            "🔢",
                        ),
                        (
                            "Insert: Radial Scale",
                            "        scale mode=\"radial\" min=0 max=100 value=75\n",
                            "🧭",
                        ),
                        ("Insert: Progress Bar", "        progress value=60\n", "📊"),
                        ("Insert: Banner", "        banner text=\"TITLE\"\n", "🏷"),
                    ];
                    for (wname, snippet, icon) in widgets {
                        if query.is_empty() || wname.to_lowercase().contains(&query) {
                            ui.horizontal(|ui| {
                                ui.label(icon);
                                if ui.selectable_label(false, wname).clicked() {
                                    selected_action = Some(ActionType::InsertWidget(snippet));
                                }
                            });
                        }
                    }

                    // Mode toggle actions
                    if query.is_empty() || "toggle mode design interactive".contains(&query) {
                        ui.horizontal(|ui| {
                            ui.label("🎮");
                            let mode_text = match app.mode {
                                StudioMode::Design => "Switch to Live Interactive Mode",
                                StudioMode::Interactive => "Switch to Design Mode",
                            };
                            if ui.selectable_label(false, mode_text).clicked() {
                                selected_action = Some(ActionType::ToggleMode);
                            }
                        });
                    }

                    if query.is_empty() || "toggle rulers canvas".contains(&query) {
                        ui.horizontal(|ui| {
                            ui.label("📏");
                            let ruler_text = if app.show_rulers {
                                "Hide Canvas Pixel Rulers"
                            } else {
                                "Show Canvas Pixel Rulers"
                            };
                            if ui.selectable_label(false, ruler_text).clicked() {
                                selected_action = Some(ActionType::ToggleRulers);
                            }
                        });
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Tip: Press Ctrl+K / Cmd+K anytime to open")
                        .weak()
                        .size(11.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(Color32::from_rgb(120, 140, 160), "Esc to dismiss");
                });
            });
        });

    if let Some(act) = selected_action {
        match act {
            ActionType::SwitchScreen(idx) => app.switch_to_screen(idx),
            ActionType::SwitchTheme(theme) => {
                app.display_theme = theme;
                if app.live_stream {
                    app.push_live_frame();
                }
            }
            ActionType::InsertWidget(snippet) => app.insert_widget_snippet(snippet),
            ActionType::ToggleMode => {
                app.mode = match app.mode {
                    StudioMode::Design => StudioMode::Interactive,
                    StudioMode::Interactive => StudioMode::Design,
                };
            }
            ActionType::ToggleRulers => app.show_rulers = !app.show_rulers,
        }
        close = true;
    }

    if close {
        app.command_palette_open = false;
        app.command_query.clear();
    }
}
