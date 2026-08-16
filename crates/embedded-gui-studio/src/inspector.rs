//! Visual inspector sidebar panel for properties, tracks, and screen settings.

use eframe::egui::{self, DragValue};
use embedded_gui_codegen::{GridPlacementDef, ScreenDef, WidgetDef};

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
                WidgetDef::Label { id, text, style } => {
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
                    ui.horizontal(|ui| {
                        ui.label("On Click:");
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
            if ui.add(DragValue::new(&mut w).range(128..=1920)).changed() {
                screen.width = w.max(32) as u32;
                modified = true;
            }
            ui.label("Height:");
            let mut h = screen.height as i32;
            if ui.add(DragValue::new(&mut h).range(64..=1080)).changed() {
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

        // Quick Add Widget Section
        ui.label(egui::RichText::new("➕ Insert Widget").strong());
        egui::Grid::new("insert_widget_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                if ui.button("🔘 Button").clicked() {
                    screen.grid.children.push((
                        GridPlacementDef::default(),
                        WidgetDef::Button {
                            id: Some("new_btn".into()),
                            text: "Button".into(),
                            on_click: None,
                            style: Some("accent".into()),
                        },
                    ));
                    *selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
                if ui.button("🏷 Label").clicked() {
                    screen.grid.children.push((
                        GridPlacementDef::default(),
                        WidgetDef::Label {
                            id: Some("new_label".into()),
                            text: "New Label".into(),
                            style: None,
                        },
                    ));
                    *selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
                ui.end_row();

                if ui.button("🎚 Slider").clicked() {
                    screen.grid.children.push((
                        GridPlacementDef::default(),
                        WidgetDef::Slider {
                            id: Some("new_slider".into()),
                            min: 0,
                            max: 100,
                            value: 50,
                        },
                    ));
                    *selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
                if ui.button("⏻ Toggle").clicked() {
                    screen.grid.children.push((
                        GridPlacementDef::default(),
                        WidgetDef::Toggle {
                            id: Some("new_toggle".into()),
                            label: "Power".into(),
                            checked: true,
                        },
                    ));
                    *selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
                ui.end_row();

                if ui.button("⏱ Gauge / Scale").clicked() {
                    screen.grid.children.push((
                        GridPlacementDef::default(),
                        WidgetDef::Scale {
                            id: Some("new_gauge".into()),
                            mode: "radial".into(),
                            min: 0.0,
                            max: 100.0,
                            value: 25.0,
                            major_ticks: 5,
                            minor_ticks: 2,
                        },
                    ));
                    *selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
                if ui.button("📊 Progress Bar").clicked() {
                    screen.grid.children.push((
                        GridPlacementDef::default(),
                        WidgetDef::ProgressBar {
                            id: Some("new_progress".into()),
                            value: 0.75,
                        },
                    ));
                    *selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
                ui.end_row();

                if ui.button("📈 Scope Plotter").clicked() {
                    screen.grid.children.push((
                        GridPlacementDef::default(),
                        WidgetDef::Plotter {
                            id: Some("new_plotter".into()),
                            mode: "sine".into(),
                        },
                    ));
                    *selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
                if ui.button("⚙️ Busy Spinner").clicked() {
                    screen.grid.children.push((
                        GridPlacementDef::default(),
                        WidgetDef::BusyWheel {
                            id: Some("new_spinner".into()),
                            active: true,
                        },
                    ));
                    *selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
                ui.end_row();
            });
    }

    modified
}
