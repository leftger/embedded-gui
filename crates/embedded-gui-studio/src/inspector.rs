//! Visual inspector sidebar panel for properties, tracks, and screen settings.

use eframe::egui::{self, DragValue};
use embedded_gui_codegen::{GridPlacementDef, ScreenDef, WidgetDef};

fn push_new_widget(
    screen: &mut ScreenDef,
    widget: WidgetDef,
    selected_widget_idx: &mut Option<usize>,
) {
    let max_cols = screen.grid.cols.len().max(1);
    let (next_col, next_row) = if let Some((last_p, _)) = screen.grid.children.last() {
        let nc = (last_p.col + last_p.col_span) % max_cols;
        let nr = last_p.row + if nc == 0 { last_p.row_span } else { 0 };
        (nc, nr)
    } else {
        (0, 0)
    };
    let placement = GridPlacementDef {
        col: next_col,
        row: next_row,
        col_span: 1,
        row_span: 1,
    };
    screen.grid.children.push((placement, widget));
    *selected_widget_idx = Some(screen.grid.children.len() - 1);
}

fn push_vector_asset(
    screen: &mut ScreenDef,
    name: &str,
    d: &str,
    selected_widget_idx: &mut Option<usize>,
) {
    let verbs = embedded_gui_codegen::parse_svg_path_d(d);
    push_new_widget(
        screen,
        WidgetDef::VectorPath {
            id: Some(name.to_lowercase()),
            stroke_width: 1,
            verbs,
        },
        selected_widget_idx,
    );
}

/// Renders the visual property inspector sidebar for the selected widget or screen.
pub fn render_inspector_panel(
    ui: &mut egui::Ui,
    screen: &mut ScreenDef,
    selected_widget_idx: &mut Option<usize>,
) -> bool {
    let mut modified = false;

    if let Some(idx) = *selected_widget_idx {
        if idx < screen.grid.children.len() {
            let (placement, widget) = &mut screen.grid.children[idx];

            ui.horizontal(|ui| {
                ui.heading("🔍 Widget Inspector");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕ Deselect").clicked() {
                        *selected_widget_idx = None;
                    }
                });
            });
            ui.separator();

            // 1. Grid Placement Section
            ui.label(egui::RichText::new("📍 Grid Placement").strong());
            ui.horizontal(|ui| {
                ui.label("Col:");
                let mut col = placement.col as i32;
                if ui.add(DragValue::new(&mut col).range(0..=16)).changed() {
                    placement.col = col.max(0) as usize;
                    modified = true;
                }
                ui.label("Row:");
                let mut row = placement.row as i32;
                if ui.add(DragValue::new(&mut row).range(0..=16)).changed() {
                    placement.row = row.max(0) as usize;
                    modified = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Col Span:");
                let mut c_span = placement.col_span as i32;
                if ui.add(DragValue::new(&mut c_span).range(1..=8)).changed() {
                    placement.col_span = c_span.max(1) as usize;
                    modified = true;
                }
                ui.label("Row Span:");
                let mut r_span = placement.row_span as i32;
                if ui.add(DragValue::new(&mut r_span).range(1..=8)).changed() {
                    placement.row_span = r_span.max(1) as usize;
                    modified = true;
                }
            });

            ui.separator();

            // 2. Widget Specific Properties
            ui.label(egui::RichText::new("⚙️ Properties").strong());

            match widget {
                WidgetDef::Label {
                    id,
                    text,
                    style,
                    font,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Text:");
                        if ui.text_edit_singleline(text).changed() {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Style:");
                        let mut style_str = style.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut style_str).changed() {
                            *style = if style_str.trim().is_empty() {
                                None
                            } else {
                                Some(style_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Font:")
                            .on_hover_text("Name of a font imported on this screen");
                        let mut font_str = font.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut font_str).changed() {
                            *font = if font_str.trim().is_empty() {
                                None
                            } else {
                                Some(font_str)
                            };
                            modified = true;
                        }
                    });
                }
                WidgetDef::Button {
                    id,
                    text,
                    on_click,
                    style,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Text:");
                        if ui.text_edit_singleline(text).changed() {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Style:");
                        let mut style_str = style.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut style_str).changed() {
                            *style = if style_str.trim().is_empty() {
                                None
                            } else {
                                Some(style_str)
                            };
                            modified = true;
                        }
                    });

                    // 1-Click Action Trigger & Screen Navigation Selector
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("⚡ Action Trigger").strong());
                        let cur_action = on_click.clone().unwrap_or_default();
                        let is_nav = cur_action.starts_with("navigate:");

                        let mut action_type = if cur_action.is_empty() {
                            "None"
                        } else if is_nav {
                            "Navigate to Screen"
                        } else {
                            "Custom Action"
                        };

                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt("btn_action_type")
                                .selected_text(action_type)
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(&mut action_type, "None", "None")
                                        .clicked()
                                    {
                                        *on_click = None;
                                        modified = true;
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut action_type,
                                            "Navigate to Screen",
                                            "🔀 Navigate to Screen",
                                        )
                                        .clicked()
                                    {
                                        *on_click = Some("navigate:HvacClimate:SlideLeft".into());
                                        modified = true;
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut action_type,
                                            "Custom Action",
                                            "⚙ Custom Handler",
                                        )
                                        .clicked()
                                    {
                                        *on_click = Some("on_button_click".into());
                                        modified = true;
                                    }
                                });
                        });

                        if is_nav {
                            let parts: Vec<&str> = cur_action.split(':').collect();
                            let target_screen = parts.get(1).copied().unwrap_or("HvacClimate");
                            let trans_code = parts.get(2).copied().unwrap_or("SlideLeft");

                            let known_targets = [
                                "AutoCluster",
                                "HvacClimate",
                                "PatientMonitor",
                                "CncController",
                                "FitnessTracker",
                            ];
                            let mut selected_target = target_screen.to_string();

                            ui.horizontal(|ui| {
                                ui.label("Target:");
                                egui::ComboBox::from_id_salt("btn_nav_target")
                                    .selected_text(&selected_target)
                                    .show_ui(ui, |ui| {
                                        for t in known_targets {
                                            if ui
                                                .selectable_value(
                                                    &mut selected_target,
                                                    t.to_string(),
                                                    t,
                                                )
                                                .clicked()
                                            {
                                                *on_click = Some(format!(
                                                    "navigate:{}:{}",
                                                    selected_target, trans_code
                                                ));
                                                modified = true;
                                            }
                                        }
                                    });
                            });

                            let transitions = [
                                ("SlideLeft", "➡️ Slide Left (300ms)"),
                                ("SlideRight", "⬅️ Slide Right (300ms)"),
                                ("Fade", "✨ Fade (200ms)"),
                                ("Instant", "⚡ Instant"),
                            ];
                            let mut selected_trans = trans_code.to_string();

                            ui.horizontal(|ui| {
                                ui.label("Effect:");
                                egui::ComboBox::from_id_salt("btn_nav_trans")
                                    .selected_text(
                                        transitions
                                            .iter()
                                            .find(|(c, _)| *c == trans_code)
                                            .map(|(_, n)| *n)
                                            .unwrap_or("Slide Left"),
                                    )
                                    .show_ui(ui, |ui| {
                                        for (c, label) in transitions {
                                            if ui
                                                .selectable_value(
                                                    &mut selected_trans,
                                                    c.to_string(),
                                                    label,
                                                )
                                                .clicked()
                                            {
                                                *on_click = Some(format!(
                                                    "navigate:{}:{}",
                                                    target_screen, selected_trans
                                                ));
                                                modified = true;
                                            }
                                        }
                                    });
                            });
                        } else if action_type == "Custom Action" {
                            ui.horizontal(|ui| {
                                ui.label("Handler:");
                                let mut click_str = on_click.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut click_str).changed() {
                                    *on_click = if click_str.trim().is_empty() {
                                        None
                                    } else {
                                        Some(click_str)
                                    };
                                    modified = true;
                                }
                            });
                        }
                    });
                }
                WidgetDef::Toggle { id, label, checked }
                | WidgetDef::Checkbox { id, label, checked } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Label:");
                        if ui.text_edit_singleline(label).changed() {
                            modified = true;
                        }
                    });
                    if ui.checkbox(checked, "Checked").changed() {
                        modified = true;
                    }
                }
                WidgetDef::Slider {
                    id,
                    min,
                    max,
                    value,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Min:");
                        if ui.add(DragValue::new(min)).changed() {
                            modified = true;
                        }
                        ui.label("Max:");
                        if ui.add(DragValue::new(max)).changed() {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        if ui.add(egui::Slider::new(value, *min..=*max)).changed() {
                            modified = true;
                        }
                    });
                }
                WidgetDef::ProgressBar { id, value } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Progress:");
                        if ui.add(egui::Slider::new(value, 0.0..=1.0)).changed() {
                            modified = true;
                        }
                    });
                }
                WidgetDef::Scale {
                    id,
                    mode,
                    min,
                    max,
                    value,
                    major_ticks,
                    ..
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        ui.selectable_value(mode, "radial".to_string(), "Radial");
                        ui.selectable_value(mode, "linear".to_string(), "Linear");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Min:");
                        if ui.add(DragValue::new(min)).changed() {
                            modified = true;
                        }
                        ui.label("Max:");
                        if ui.add(DragValue::new(max)).changed() {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        if ui.add(DragValue::new(value)).changed() {
                            modified = true;
                        }
                        ui.label("Ticks:");
                        if ui.add(DragValue::new(major_ticks).range(1..=12)).changed() {
                            modified = true;
                        }
                    });
                }
                WidgetDef::Plotter { id, mode } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        if ui
                            .selectable_value(mode, "sine".to_string(), "Sine")
                            .clicked()
                            || ui
                                .selectable_value(mode, "square".to_string(), "Square")
                                .clicked()
                            || ui
                                .selectable_value(mode, "triangle".to_string(), "Triangle")
                                .clicked()
                        {
                            modified = true;
                        }
                    });
                }
                WidgetDef::BusyWheel { id, active } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    if ui.checkbox(active, "Active / Spinning").changed() {
                        modified = true;
                    }
                }
                WidgetDef::StatusBar { id, time } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        if ui.text_edit_singleline(time).changed() {
                            modified = true;
                        }
                    });
                }
                WidgetDef::Image {
                    id,
                    source,
                    fit,
                    mode,
                    tint,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = (!id_str.trim().is_empty()).then_some(id_str);
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Project path:");
                        if ui.text_edit_singleline(source).changed() {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Fit:");
                        if ui
                            .selectable_value(fit, "center".into(), "Center")
                            .changed()
                            || ui
                                .selectable_value(fit, "stretch".into(), "Stretch")
                                .changed()
                        {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        if ui.selectable_value(mode, "color".into(), "Color").changed()
                            || ui
                                .selectable_value(mode, "mask".into(), "1-bit Mask")
                                .changed()
                        {
                            modified = true;
                        }
                    });
                    if mode == "mask" {
                        ui.horizontal(|ui| {
                            ui.label("Tint:");
                            let mut tint_str = tint.clone().unwrap_or_else(|| "accent".into());
                            if ui.text_edit_singleline(&mut tint_str).changed() {
                                *tint = (!tint_str.trim().is_empty()).then_some(tint_str);
                                modified = true;
                            }
                        });
                    }
                }
                WidgetDef::Carousel {
                    id,
                    items,
                    selected,
                    item_step,
                    visible,
                    shift,
                    mask_top,
                    mask_bottom,
                    fade,
                    indicator,
                    pulse,
                    style,
                    font,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = (!id_str.trim().is_empty()).then_some(id_str);
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Selected:");
                        let max = items.len().saturating_sub(1) as i32;
                        let mut sel = *selected as i32;
                        if ui
                            .add(egui::Slider::new(&mut sel, 0..=max.max(0)))
                            .changed()
                        {
                            *selected = sel.max(0) as usize;
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Item step (px):");
                        let mut step = *item_step as i32;
                        if ui.add(egui::Slider::new(&mut step, 4..=64)).changed() {
                            *item_step = step as u16;
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Visible slots:");
                        let mut slots = *visible as i32;
                        if ui.add(egui::Slider::new(&mut slots, 1..=15)).changed() {
                            *visible = slots as u8;
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Scroll shift (px):")
                            .on_hover_text("In-flight offset; animate this to scroll the list");
                        let mut value = *shift as i32;
                        let range = *item_step as i32;
                        if ui
                            .add(egui::Slider::new(&mut value, -range..=range))
                            .changed()
                        {
                            *shift = value as i16;
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Chrome masks:");
                        let mut top = *mask_top as i32;
                        let mut bottom = *mask_bottom as i32;
                        if ui
                            .add(egui::DragValue::new(&mut top).prefix("top "))
                            .changed()
                        {
                            *mask_top = top.max(0) as u16;
                            modified = true;
                        }
                        if ui
                            .add(egui::DragValue::new(&mut bottom).prefix("bottom "))
                            .changed()
                        {
                            *mask_bottom = bottom.max(0) as u16;
                            modified = true;
                        }
                    });
                    if ui.checkbox(fade, "Fade edges").changed() {
                        modified = true;
                    }
                    if ui.checkbox(indicator, "Selection indicator").changed() {
                        modified = true;
                    }
                    if *indicator {
                        ui.horizontal(|ui| {
                            ui.label("Pulse:");
                            let mut value = *pulse as i32;
                            if ui.add(egui::Slider::new(&mut value, 0..=255)).changed() {
                                *pulse = value as u8;
                                modified = true;
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.label("Style:");
                        let mut style_str = style.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut style_str).changed() {
                            *style = (!style_str.trim().is_empty()).then_some(style_str);
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Font:");
                        let mut font_str = font.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut font_str).changed() {
                            *font = (!font_str.trim().is_empty()).then_some(font_str);
                            modified = true;
                        }
                    });
                    ui.separator();
                    ui.label("Items:");
                    let mut remove = None;
                    for (idx, item) in items.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.text_edit_singleline(item).changed() {
                                modified = true;
                            }
                            if ui.small_button("🗑").clicked() {
                                remove = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = remove {
                        items.remove(idx);
                        *selected = (*selected).min(items.len().saturating_sub(1));
                        modified = true;
                    }
                    if ui.button("➕ Add item").clicked() {
                        items.push(format!("ITEM {}", items.len() + 1));
                        modified = true;
                    }
                }
                WidgetDef::CompositeIcon {
                    id,
                    parts,
                    scale,
                    align,
                    tint,
                    threshold,
                    invert,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = (!id_str.trim().is_empty()).then_some(id_str);
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        let mut value = *scale as i32;
                        if ui.add(egui::Slider::new(&mut value, 1..=8)).changed() {
                            *scale = value as u8;
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Align:");
                        if ui
                            .selectable_value(align, "center".into(), "Center")
                            .changed()
                            || ui
                                .selectable_value(align, "top_left".into(), "Top left")
                                .changed()
                        {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Ink:");
                        let mut tint_str = tint.clone().unwrap_or_else(|| "accent".into());
                        if ui.text_edit_singleline(&mut tint_str).changed() {
                            *tint = (!tint_str.trim().is_empty()).then_some(tint_str);
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Threshold:").on_hover_text(
                            "Luminance below this becomes ink when the art is imported",
                        );
                        let mut value = *threshold as i32;
                        if ui.add(egui::Slider::new(&mut value, 0..=255)).changed() {
                            *threshold = value as u8;
                            modified = true;
                        }
                    });
                    if ui
                        .checkbox(invert, "Invert (art is light-on-dark)")
                        .changed()
                    {
                        modified = true;
                    }
                    ui.separator();
                    ui.label("Parts (stacked in order):");
                    let mut remove = None;
                    let removable = parts.len() > 1;
                    for (idx, part) in parts.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("src:");
                                if ui.text_edit_singleline(&mut part.source).changed() {
                                    modified = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("offset:");
                                if ui
                                    .add(egui::DragValue::new(&mut part.dx).prefix("x "))
                                    .changed()
                                    || ui
                                        .add(egui::DragValue::new(&mut part.dy).prefix("y "))
                                        .changed()
                                {
                                    modified = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut part.visible, "Visible").changed() {
                                    modified = true;
                                }
                                let mut part_tint = part.tint.clone().unwrap_or_default();
                                ui.label("tint:");
                                if ui.text_edit_singleline(&mut part_tint).changed() {
                                    part.tint = (!part_tint.trim().is_empty()).then_some(part_tint);
                                    modified = true;
                                }
                                if removable && ui.small_button("🗑").clicked() {
                                    remove = Some(idx);
                                }
                            });
                        });
                    }
                    if let Some(idx) = remove {
                        parts.remove(idx);
                        modified = true;
                    }
                    if ui.button("➕ Add part").clicked() {
                        parts.push(embedded_gui_codegen::IconPartDef {
                            source: "assets/icons/part.bmp".into(),
                            dx: 0,
                            dy: 0,
                            visible: true,
                            tint: None,
                        });
                        modified = true;
                    }
                }
                WidgetDef::Mesh3d {
                    id,
                    source,
                    shading,
                    color,
                    scale,
                    roll,
                    pitch,
                    yaw,
                    camera_distance,
                    fov,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = (!id_str.trim().is_empty()).then_some(id_str);
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Project path (.obj):");
                        if ui.text_edit_singleline(source).changed() {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Shading:");
                        for (value, label) in [
                            ("solid", "Solid"),
                            ("lit", "Lit"),
                            ("lines", "Wireframe"),
                            ("points", "Points"),
                        ] {
                            if ui.selectable_value(shading, value.into(), label).changed() {
                                modified = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color_str = color.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut color_str).changed() {
                            *color = (!color_str.trim().is_empty()).then_some(color_str);
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        if ui
                            .add(egui::Slider::new(scale, 0.1..=8.0).logarithmic(true))
                            .changed()
                        {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Rotation (rad):");
                        if ui
                            .add(egui::DragValue::new(roll).prefix("roll ").speed(0.05))
                            .changed()
                            || ui
                                .add(egui::DragValue::new(pitch).prefix("pitch ").speed(0.05))
                                .changed()
                            || ui
                                .add(egui::DragValue::new(yaw).prefix("yaw ").speed(0.05))
                                .changed()
                        {
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Camera:");
                        if ui
                            .add(
                                egui::DragValue::new(camera_distance)
                                    .prefix("dist ")
                                    .speed(0.1),
                            )
                            .changed()
                            || ui
                                .add(egui::DragValue::new(fov).prefix("fov ").speed(0.02))
                                .changed()
                        {
                            modified = true;
                        }
                    });
                }
                WidgetDef::Roller {
                    id,
                    options,
                    selected,
                }
                | WidgetDef::Dropdown {
                    id,
                    options,
                    selected,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        let mut id_str = id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            *id = if id_str.trim().is_empty() {
                                None
                            } else {
                                Some(id_str)
                            };
                            modified = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Selected:");
                        let mut sel = *selected as i32;
                        if ui
                            .add(
                                DragValue::new(&mut sel).range(0..=options.len().saturating_sub(1)),
                            )
                            .changed()
                        {
                            *selected = sel.max(0) as usize;
                            modified = true;
                        }
                    });
                    ui.label("Options:");
                    let mut remove_idx = None;
                    for (i, opt) in options.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.text_edit_singleline(opt).changed() {
                                modified = true;
                            }
                            if ui.small_button("🗑").clicked() {
                                remove_idx = Some(i);
                            }
                        });
                    }
                    if let Some(i) = remove_idx {
                        if options.len() > 1 {
                            options.remove(i);
                            *selected = (*selected).min(options.len() - 1);
                            modified = true;
                        }
                    }
                    if ui.button("➕ Add Option").clicked() {
                        options.push(format!("Option {}", options.len() + 1));
                        modified = true;
                    }
                }
                _ => {
                    ui.label(format!("Widget: {:?}", widget));
                }
            }

            ui.separator();

            // 3. Widget Actions
            let mut should_delete = false;
            let mut should_duplicate = false;

            ui.horizontal(|ui| {
                if ui.button("🗑 Delete Widget").clicked() {
                    should_delete = true;
                }

                if ui.button("➕ Duplicate").clicked() {
                    should_duplicate = true;
                }
            });

            if should_delete {
                screen.grid.children.remove(idx);
                *selected_widget_idx = None;
                modified = true;
            } else if should_duplicate {
                let (p, w) = &screen.grid.children[idx];
                let dup_placement = GridPlacementDef {
                    col: p.col + 1,
                    row: p.row,
                    col_span: p.col_span,
                    row_span: p.row_span,
                };
                let dup_widget = w.clone();
                screen.grid.children.push((dup_placement, dup_widget));
                *selected_widget_idx = Some(screen.grid.children.len() - 1);
                modified = true;
            }
        } else {
            *selected_widget_idx = None;
        }
    } else {
        // Screen & Global Grid Inspector
        ui.heading("📐 Screen & Grid");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Screen ID:");
            if ui.text_edit_singleline(&mut screen.id).changed() {
                modified = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Width:");
            let mut w = screen.width as i32;
            if ui.add(DragValue::new(&mut w).range(32..=1920)).changed() {
                screen.width = w.max(32) as u32;
                modified = true;
            }
            ui.label("Height:");
            let mut h = screen.height as i32;
            if ui.add(DragValue::new(&mut h).range(32..=1080)).changed() {
                screen.height = h.max(32) as u32;
                modified = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Gap:");
            let mut gap = screen.grid.gap as i32;
            if ui.add(DragValue::new(&mut gap).range(0..=32)).changed() {
                screen.grid.gap = gap.max(0) as u16;
                modified = true;
            }
            ui.label("Padding:");
            let mut pad = screen.grid.padding as i32;
            if ui.add(DragValue::new(&mut pad).range(0..=48)).changed() {
                screen.grid.padding = pad.max(0) as u16;
                modified = true;
            }
        });

        ui.separator();

        ui.separator();

        // Comprehensive Categorized Accordions & Dropdown Palettes
        egui::CollapsingHeader::new(egui::RichText::new("🔘 Controls & Inputs").strong())
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("controls_input_grid")
                    .num_columns(2)
                    .spacing([6.0, 6.0])
                    .show(ui, |ui| {
                        if ui.button("🔘 Button").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Button {
                                    id: Some("btn".into()),
                                    text: "Click".into(),
                                    on_click: None,
                                    style: Some("accent".into()),
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("🔲 Toggle").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Toggle {
                                    id: Some("toggle".into()),
                                    label: "Power".into(),
                                    checked: true,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("☑ Checkbox").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Checkbox {
                                    id: Some("chk".into()),
                                    label: "Enable".into(),
                                    checked: false,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("🎚 Slider").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Slider {
                                    id: Some("slider".into()),
                                    min: 0,
                                    max: 100,
                                    value: 50,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("🔢 Spinbox").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Spinbox {
                                    id: Some("spin".into()),
                                    min: 0,
                                    max: 999,
                                    value: 120,
                                    digits: 3,
                                    decimals: 1,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("🔢 NumPicker").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::NumberPicker {
                                    id: Some("numpick".into()),
                                    min: 40,
                                    max: 220,
                                    value: 135,
                                    unit: "BPM".into(),
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("🕒 TimePicker").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::TimePicker {
                                    id: Some("time".into()),
                                    hour: 12,
                                    minute: 30,
                                    is_12h: true,
                                    is_pm: true,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("📋 Dropdown").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Dropdown {
                                    id: Some("drop".into()),
                                    options: vec!["Auto".into(), "Cool".into(), "Heat".into()],
                                    selected: 0,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("🎡 Roller Wheel").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Roller {
                                    id: Some("roller".into()),
                                    options: vec!["Low".into(), "Med".into(), "High".into()],
                                    selected: 1,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                    });
            });

        egui::CollapsingHeader::new(egui::RichText::new("📝 Text & Containers").strong())
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("text_containers_grid")
                    .num_columns(2)
                    .spacing([6.0, 6.0])
                    .show(ui, |ui| {
                        if ui.button("📝 Label").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Label {
                                    id: Some("label".into()),
                                    text: "System Ready".into(),
                                    style: None,
                                    font: None,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("✨ XOR Inverted").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Label {
                                    id: Some("badge".into()),
                                    text: "[ ACTIVE ]".into(),
                                    style: Some("inverted".into()),
                                    font: None,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("📱 StatusBar").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::StatusBar {
                                    id: Some("status".into()),
                                    time: "10:42".into(),
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("💬 Dialog").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Dialog {
                                    id: Some("dlg".into()),
                                    title: "Confirm".into(),
                                    message: "Apply settings?".into(),
                                    dialog_type: "confirm".into(),
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("📊 Table Grid").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Table {
                                    id: Some("table".into()),
                                    headers: Some(vec!["SENSOR".into(), "VAL".into()]),
                                    rows: vec![
                                        vec!["Temp".into(), "42°C".into()],
                                        vec!["Volt".into(), "3.3V".into()],
                                    ],
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                    });
            });

        egui::CollapsingHeader::new(egui::RichText::new("📊 Gauges & Telemetry").strong())
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("gauges_telemetry_grid")
                    .num_columns(2)
                    .spacing([6.0, 6.0])
                    .show(ui, |ui| {
                        if ui.button("📈 ProgressBar").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::ProgressBar {
                                    id: Some("progress".into()),
                                    value: 0.65,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("⏱ Tachometer").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Scale {
                                    id: Some("scale".into()),
                                    mode: "radial".into(),
                                    min: 0.0,
                                    max: 100.0,
                                    value: 65.0,
                                    major_ticks: 5,
                                    minor_ticks: 2,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("📐 Sweeping Arc").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::SweepingArc {
                                    id: Some("arc".into()),
                                    start_angle: 0,
                                    end_angle: 180,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("🌀 Spinner").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::BusyWheel {
                                    id: Some("spinner".into()),
                                    active: true,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("📉 Scope Plotter").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Plotter {
                                    id: Some("plotter".into()),
                                    mode: "waveform".into(),
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                    });
            });

        egui::CollapsingHeader::new(egui::RichText::new("📐 Vector Shapes").strong())
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("vector_shapes_grid")
                    .num_columns(2)
                    .spacing([6.0, 6.0])
                    .show(ui, |ui| {
                        if ui.button("🔲 Bezel Rect").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::RectShape {
                                    id: Some("bezel".into()),
                                    radius: 2,
                                    stroke_width: 1,
                                    fill_color: Some("#000000".into()),
                                    stroke_color: Some("#FFFFFF".into()),
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("⚪ Circle").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::CircleShape {
                                    id: Some("circle".into()),
                                    radius: 12,
                                    stroke_width: 1,
                                    fill_color: None,
                                    stroke_color: Some("#FFFFFF".into()),
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();

                        if ui.button("➖ Line Divider").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::LineShape {
                                    id: Some("line".into()),
                                    stroke_width: 1,
                                    color: Some("#FFFFFF".into()),
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("✒️ Bézier Curve").clicked() {
                            push_vector_asset(
                                screen,
                                "spline",
                                "M 0 10 C 20 0, 40 40, 60 10",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                        if ui.button("🖼 Project Image").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Image {
                                    id: Some("image".into()),
                                    source: "assets/image.png".into(),
                                    fit: "center".into(),
                                    mode: "color".into(),
                                    tint: None,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                        if ui.button("🎠 Carousel").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Carousel {
                                    id: Some("carousel".into()),
                                    items: vec![
                                        "DEFAULT".into(),
                                        "STEALTH".into(),
                                        "BRIGHTNESS".into(),
                                    ],
                                    selected: 0,
                                    item_step: 16,
                                    visible: 7,
                                    shift: 0,
                                    mask_top: 0,
                                    mask_bottom: 0,
                                    fade: true,
                                    indicator: true,
                                    pulse: 96,
                                    style: None,
                                    font: None,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.button("🧩 Composite Icon").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::CompositeIcon {
                                    id: Some("icon".into()),
                                    parts: vec![embedded_gui_codegen::IconPartDef {
                                        source: "assets/icons/part.bmp".into(),
                                        dx: 0,
                                        dy: 0,
                                        visible: true,
                                        tint: None,
                                    }],
                                    scale: 1,
                                    align: "center".into(),
                                    tint: Some("accent".into()),
                                    threshold: 128,
                                    invert: false,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                        if ui.button("🧊 3D Mesh").clicked() {
                            push_new_widget(
                                screen,
                                WidgetDef::Mesh3d {
                                    id: Some("logo".into()),
                                    source: "assets/logo.obj".into(),
                                    shading: "lit".into(),
                                    color: Some("accent".into()),
                                    scale: 1.0,
                                    roll: 0.0,
                                    pitch: 0.0,
                                    yaw: 0.0,
                                    camera_distance: 4.0,
                                    fov: 1.5707964,
                                },
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                    });
            });

        egui::CollapsingHeader::new(egui::RichText::new("🎨 Vector Asset Library").strong())
            .default_open(false)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("🔋 Power & Battery").small());
                egui::Grid::new("asset_power_grid")
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        if ui.small_button("🔋 Batt Full").clicked() {
                            push_vector_asset(
                                screen,
                                "batt_full",
                                "M 0 0 L 14 0 L 14 6 L 0 6 Z M 14 2 L 15 2 L 15 4 L 14 4 Z M 2 2 L 12 2 L 12 4 L 2 4 Z",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.small_button("🪫 Batt Low").clicked() {
                            push_vector_asset(
                                screen,
                                "batt_low",
                                "M 0 0 L 14 0 L 14 6 L 0 6 Z M 14 2 L 15 2 L 15 4 L 14 4 Z M 2 2 L 4 2 L 4 4 L 2 4 Z",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                        if ui.small_button("⚡ Bolt (Charge)").clicked() {
                            push_vector_asset(
                                screen,
                                "bolt",
                                "M 6 0 L 1 7 L 5 7 L 3 13 L 10 5 L 5 5 Z",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                    });

                ui.separator();
                ui.label(egui::RichText::new("📶 Connectivity & Badges").small());
                egui::Grid::new("asset_badges_grid")
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        if ui.small_button("📡 Bluetooth").clicked() {
                            push_vector_asset(
                                screen,
                                "bluetooth",
                                "M 4 1 L 8 5 L 5 8 L 5 0 L 8 3 L 4 7",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.small_button("📶 Signal Bars").clicked() {
                            push_vector_asset(
                                screen,
                                "signal",
                                "M 1 6 L 3 6 L 3 8 L 1 8 Z M 5 4 L 7 4 L 7 8 L 5 8 Z M 9 2 L 11 2 L 11 8 L 9 8 Z",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                        if ui.small_button("⚠️ Warning").clicked() {
                            push_vector_asset(
                                screen,
                                "warning",
                                "M 6 1 L 12 11 L 0 11 Z M 6 4 L 6 7 M 6 9 L 6 10",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.small_button("🛡️ Shield").clicked() {
                            push_vector_asset(
                                screen,
                                "shield",
                                "M 0 2 L 6 0 L 12 2 L 12 7 C 12 10, 6 13, 6 13 C 6 13, 0 10, 0 7 Z",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                        if ui.small_button("❤️ Heart / BPM").clicked() {
                            push_vector_asset(
                                screen,
                                "heart",
                                "M 6 2 C 4 0, 0 1, 0 4 C 0 8, 6 11, 6 11 C 6 11, 12 8, 12 4 C 12 1, 8 0, 6 2 Z",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.small_button("🎯 Crosshair").clicked() {
                            push_vector_asset(
                                screen,
                                "crosshair",
                                "M 6 0 L 6 3 M 6 9 L 6 12 M 0 6 L 3 6 M 9 6 L 12 6 M 6 2 C 8.2 2, 10 3.8, 10 6 C 10 8.2, 8.2 10, 6 10 C 3.8 10, 2 8.2, 2 6 C 2 3.8, 3.8 2, 6 2 Z",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                    });

                ui.separator();
                ui.label(egui::RichText::new("⏱️ Tools & Media").small());
                egui::Grid::new("asset_tools_grid")
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        if ui.small_button("⏱ Clock Dial").clicked() {
                            push_vector_asset(
                                screen,
                                "timer",
                                "M 6 0 C 9.3 0, 12 2.7, 12 6 C 12 9.3, 9.3 12, 6 12 C 2.7 12, 0 9.3, 0 6 C 0 2.7, 2.7 0, 6 0 Z M 6 2 L 6 6 L 9 6",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        if ui.small_button("⚙️ Settings Gear").clicked() {
                            push_vector_asset(
                                screen,
                                "gear",
                                "M 5 0 L 7 0 L 7 2 L 9 3 L 11 1 L 12 2 L 10 4 L 11 6 L 13 6 L 13 8 L 11 8 L 10 10 L 12 12 L 11 13 L 9 11 L 7 12 L 7 14 L 5 14 L 5 12 L 3 11 L 1 13 L 0 12 L 2 10 L 1 8 L 0 8 L 0 6 L 2 6 L 1 4 L 2 2 L 4 3 L 5 2 Z",
                                selected_widget_idx,
                            );
                            modified = true;
                        }
                        ui.end_row();
                        if ui.small_button("▶ Play").clicked() {
                            push_vector_asset(screen, "play", "M 2 1 L 11 6 L 2 11 Z", selected_widget_idx);
                            modified = true;
                        }
                        if ui.small_button("⏸ Pause").clicked() {
                            push_vector_asset(screen, "pause", "M 2 1 L 5 1 L 5 11 L 2 11 Z M 7 1 L 10 1 L 10 11 L 7 11 Z", selected_widget_idx);
                            modified = true;
                        }
                        ui.end_row();
                    });
            });
    }

    modified
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_gui_codegen::parse_kdl_screen;

    /// `DragValue::range` clamps the existing value even without interaction,
    /// so a range floor above a supported panel size silently rewrites the KDL.
    #[test]
    fn showing_the_screen_inspector_leaves_a_96x64_canvas_untouched() {
        let screen = parse_kdl_screen(
            r#"screen id="Home" width=96 height=64 {
                grid cols="1fr" rows="1fr" gap=0 padding=0 {
                    label text="X" col=0 row=0
                }
            }"#,
        )
        .expect("screen parses");
        // `__run_test_ui` takes an `Fn`, so the mutable state lives in cells.
        let screen = std::cell::RefCell::new(screen);
        let selected = std::cell::RefCell::new(None);
        let modified = std::cell::Cell::new(true);

        egui::__run_test_ui(|ui| {
            modified.set(render_inspector_panel(
                ui,
                &mut screen.borrow_mut(),
                &mut selected.borrow_mut(),
            ));
        });

        let screen = screen.into_inner();
        assert!(
            !modified.get(),
            "merely showing the inspector must not edit"
        );
        assert_eq!((screen.width, screen.height), (96, 64));
    }
}
