//! Embedded GUI Studio
//! Cross-platform interactive designer, visual inspector, live animation previewer, and Rust code generator for embedded-gui.

use core::f32::consts::PI;
use eframe::egui::{
    self, Color32, CornerRadius, DragValue, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2,
};
use embedded_gui_codegen::{
    GridPlacementDef, GridTrackDef, ScreenDef, WidgetDef, generate_rust_code, parse_kdl_screen,
    serialize_kdl_screen,
};

const SAMPLE_THERMOSTAT: &str = r#"screen id="Thermostat" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="36px 1fr 48px" gap=6 padding=8 {
        status_bar id="status" time="14:32" col=0 row=0 col_span=2
        scale id="temp_gauge" mode="radial" min=15.0 max=35.0 value=22.5 major_ticks=4 col=0 row=1
        slider id="target_slider" min=10 max=40 value=23 col=1 row=1
        button id="btn_heat" text="Heat Mode" style="accent" col=0 row=2
        toggle id="power_switch" label="Power" checked=true col=1 row=2
    }
}
"#;

const SAMPLE_DASHBOARD: &str = r#"screen id="SensorDashboard" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="32px 1fr 1fr" gap=4 padding=6 {
        label id="header" text="ENVIRONMENTAL TELEMETRY" style="bold" col=0 row=0 col_span=2
        panel id="pnl_temp" style="card" col=0 row=1
        panel id="pnl_hum" style="card" col=1 row=1
        progress id="battery" value=0.85 col=0 row=2
        button id="btn_sync" text="Sync Telemetry" col=1 row=2
    }
}
"#;

const SAMPLE_WAVEFORM: &str = r#"screen id="ScopeScreen" width=320 height=240 theme="dark" {
    grid cols="1fr 80px" rows="30px 1fr 40px" gap=4 padding=4 {
        label id="title" text="DSO-X 2-CHANNEL OSCILLOSCOPE" col=0 row=0 col_span=2
        plotter id="wave_view" mode="sine" col=0 row=1
        roller id="v_div" selected=1 col=1 row=1 {
            option "100mV"
            option "500mV"
            option "1V"
            option "5V"
        }
        button id="btn_run" text="RUN/STOP" style="accent" col=0 row=2
        button id="btn_single" text="SINGLE" col=1 row=2
    }
}
"#;

const SAMPLE_MOTION_KITCHEN_SINK: &str = r#"screen id="MotionShowcase" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="32px 1fr 1fr" gap=6 padding=6 {
        status_bar id="clock" time="12:00" col=0 row=0 col_span=2
        plotter id="live_scope" mode="sine" col=0 row=1
        busy_wheel id="spinner" active=true col=1 row=1
        progress id="pulse_bar" value=0.5 col=0 row=2
        scale id="dyn_gauge" mode="radial" min=0.0 max=100.0 value=50.0 major_ticks=5 col=1 row=2
    }
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StudioTab {
    VisualPreview,
    RustCodegen,
    AstHierarchy,
}

struct EmbeddedGuiStudio {
    kdl_source: String,
    parsed_screen: Result<ScreenDef, String>,
    generated_rust: String,
    active_tab: StudioTab,
    preview_zoom: f32,
    copied_toast_timer: f32,

    // Selection & Inspector
    selected_widget_idx: Option<usize>,

    // Animation playback state
    is_playing: bool,
    timeline_time: f32,
    playback_speed: f32,
    loop_duration: f32,
}

impl Default for EmbeddedGuiStudio {
    fn default() -> Self {
        let mut app = Self {
            kdl_source: SAMPLE_WAVEFORM.to_string(),
            parsed_screen: Err("Not parsed".to_string()),
            generated_rust: String::new(),
            active_tab: StudioTab::VisualPreview,
            preview_zoom: 1.5,
            copied_toast_timer: 0.0,
            selected_widget_idx: None,
            is_playing: true,
            timeline_time: 0.0,
            playback_speed: 1.0,
            loop_duration: 4.0,
        };
        app.recompile();
        app
    }
}

impl EmbeddedGuiStudio {
    fn recompile(&mut self) {
        match parse_kdl_screen(&self.kdl_source) {
            Ok(screen) => {
                self.generated_rust = generate_rust_code(&screen);
                self.parsed_screen = Ok(screen);
            }
            Err(err) => {
                self.parsed_screen = Err(err.to_string());
            }
        }
    }

    /// Syncs inspector modifications back into the KDL source and Rust code.
    fn sync_from_screen(&mut self, screen: &ScreenDef) {
        self.kdl_source = serialize_kdl_screen(screen);
        self.generated_rust = generate_rust_code(screen);
        self.parsed_screen = Ok(screen.clone());
    }

    fn render_visual_preview(&mut self, ui: &mut egui::Ui, screen: &ScreenDef) {
        // Toolbar: Zoom & Playback Controls
        ui.horizontal(|ui| {
            ui.label(format!(
                "Screen: {} ({}×{} px)",
                screen.id, screen.width, screen.height
            ));
            ui.separator();

            // Zoom controls
            ui.label("Zoom:");
            ui.selectable_value(&mut self.preview_zoom, 1.0, "1x");
            ui.selectable_value(&mut self.preview_zoom, 1.5, "1.5x");
            ui.selectable_value(&mut self.preview_zoom, 2.0, "2x");

            ui.separator();

            // Animation Play/Pause
            let play_label = if self.is_playing {
                "⏸ Pause"
            } else {
                "▶ Play"
            };
            if ui.button(play_label).clicked() {
                self.is_playing = !self.is_playing;
            }

            if ui.button("↺ Reset").clicked() {
                self.timeline_time = 0.0;
            }

            // Time Scrubber
            ui.label("Time:");
            ui.add(
                egui::Slider::new(&mut self.timeline_time, 0.0..=self.loop_duration)
                    .show_value(true)
                    .custom_formatter(|n, _| format!("{:.2}s", n)),
            );

            // Speed dropdown/buttons
            ui.label("Speed:");
            ui.selectable_value(&mut self.playback_speed, 0.5, "0.5x");
            ui.selectable_value(&mut self.playback_speed, 1.0, "1.0x");
            ui.selectable_value(&mut self.playback_speed, 2.0, "2.0x");
        });
        ui.separator();

        let screen_w = screen.width as f32 * self.preview_zoom;
        let screen_h = screen.height as f32 * self.preview_zoom;

        egui::ScrollArea::both().show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                Vec2::new(screen_w + 32.0, screen_h + 32.0),
                egui::Sense::click(),
            );
            let origin = response.rect.min + Vec2::new(16.0, 16.0);
            let display_rect = Rect::from_min_size(origin, Vec2::new(screen_w, screen_h));

            // Background canvas click deselects
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if !display_rect.contains(pos) {
                        self.selected_widget_idx = None;
                    }
                }
            }

            // Bezel / Hardware Chassis
            let bezel_rect = display_rect.expand(6.0);
            painter.rect_filled(
                bezel_rect,
                CornerRadius::same(8),
                Color32::from_rgb(30, 32, 38),
            );
            painter.rect_stroke(
                bezel_rect,
                CornerRadius::same(8),
                Stroke::new(2.0f32, Color32::from_rgb(70, 75, 85)),
                StrokeKind::Outside,
            );

            // Display LCD background
            painter.rect_filled(
                display_rect,
                CornerRadius::same(2),
                Color32::from_rgb(18, 20, 24),
            );

            // Compute grid layout pixel bounds
            let pad = (screen.grid.padding as f32) * self.preview_zoom;
            let gap = (screen.grid.gap as f32) * self.preview_zoom;
            let inner_rect = display_rect.shrink(pad);

            let cols = &screen.grid.cols;
            let rows = &screen.grid.rows;
            let col_widths = compute_track_sizes(cols, inner_rect.width(), gap);
            let row_heights = compute_track_sizes(rows, inner_rect.height(), gap);

            // Calculate track starting offsets
            let mut col_xs = Vec::with_capacity(col_widths.len());
            let mut cur_x = inner_rect.min.x;
            for w in &col_widths {
                col_xs.push(cur_x);
                cur_x += *w + gap;
            }

            let mut row_ys = Vec::with_capacity(row_heights.len());
            let mut cur_y = inner_rect.min.y;
            for h in &row_heights {
                row_ys.push(cur_y);
                cur_y += *h + gap;
            }

            let t = self.timeline_time;
            let pointer_pos = ui.input(|i| i.pointer.interact_pos());
            let mut newly_selected = None;

            // Draw each widget with live animation time passed in
            for (idx, (placement, widget)) in screen.grid.children.iter().enumerate() {
                let c = placement.col.min(col_xs.len().saturating_sub(1));
                let r = placement.row.min(row_ys.len().saturating_sub(1));
                let c_span = placement.col_span.max(1);
                let r_span = placement.row_span.max(1);

                let x0 = col_xs.get(c).copied().unwrap_or(inner_rect.min.x);
                let y0 = row_ys.get(r).copied().unwrap_or(inner_rect.min.y);

                let mut w = 0.0;
                for i in 0..c_span {
                    if let Some(cw) = col_widths.get(c + i) {
                        w += *cw;
                        if i + 1 < c_span {
                            w += gap;
                        }
                    }
                }

                let mut h = 0.0;
                for i in 0..r_span {
                    if let Some(rh) = row_heights.get(r + i) {
                        h += *rh;
                        if i + 1 < r_span {
                            h += gap;
                        }
                    }
                }

                let widget_rect = Rect::from_min_size(Pos2::new(x0, y0), Vec2::new(w, h));

                // Click-to-select detection
                if response.clicked() {
                    if let Some(pos) = pointer_pos {
                        if widget_rect.contains(pos) {
                            newly_selected = Some(idx);
                        }
                    }
                }

                // Draw widget representation
                draw_animated_widget(&painter, widget_rect, widget, t);

                // Selection highlight & bounding box handles
                if self.selected_widget_idx == Some(idx) {
                    let select_stroke = Stroke::new(2.0f32, Color32::from_rgb(60, 160, 255));
                    painter.rect_stroke(
                        widget_rect.expand(2.0),
                        CornerRadius::same(4),
                        select_stroke,
                        StrokeKind::Outside,
                    );

                    // Draw 4 corner handles
                    let handle_size = 5.0;
                    let corners = [
                        widget_rect.left_top(),
                        widget_rect.right_top(),
                        widget_rect.left_bottom(),
                        widget_rect.right_bottom(),
                    ];
                    for corner in corners {
                        let h_rect = Rect::from_center_size(corner, Vec2::splat(handle_size));
                        painter.rect_filled(
                            h_rect,
                            CornerRadius::same(1),
                            Color32::from_rgb(60, 160, 255),
                        );
                    }

                    // Floating selection badge
                    let badge_text = format!(
                        "🎯 {} [c:{}, r:{}]",
                        widget.id().unwrap_or("widget"),
                        placement.col,
                        placement.row
                    );
                    let badge_pos = Pos2::new(widget_rect.min.x, widget_rect.min.y - 14.0);
                    painter.rect_filled(
                        Rect::from_min_size(badge_pos, Vec2::new(120.0, 14.0)),
                        CornerRadius::same(3),
                        Color32::from_rgb(30, 80, 180),
                    );
                    painter.text(
                        Pos2::new(badge_pos.x + 4.0, badge_pos.y + 2.0),
                        egui::Align2::LEFT_TOP,
                        badge_text,
                        FontId::proportional(9.0),
                        Color32::WHITE,
                    );
                }
            }

            if let Some(sel) = newly_selected {
                self.selected_widget_idx = Some(sel);
            }
        });
    }

    /// Renders the visual property inspector sidebar for the selected widget or screen.
    fn render_inspector_panel(&mut self, ui: &mut egui::Ui) {
        let mut screen = match &self.parsed_screen {
            Ok(s) => s.clone(),
            Err(_) => {
                ui.label("Fix KDL syntax errors to use the Inspector.");
                return;
            }
        };

        let mut modified = false;

        if let Some(idx) = self.selected_widget_idx {
            if idx < screen.grid.children.len() {
                let (placement, widget) = &mut screen.grid.children[idx];

                ui.horizontal(|ui| {
                    ui.heading("🔍 Widget Inspector");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✕ Deselect").clicked() {
                            self.selected_widget_idx = None;
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
                                    .selectable_value(mode, "line".to_string(), "Line")
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
                                    DragValue::new(&mut sel)
                                        .range(0..=options.len().saturating_sub(1)),
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
                    self.selected_widget_idx = None;
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
                    self.selected_widget_idx = Some(screen.grid.children.len() - 1);
                    modified = true;
                }
            } else {
                self.selected_widget_idx = None;
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
                        self.selected_widget_idx = Some(screen.grid.children.len() - 1);
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
                        self.selected_widget_idx = Some(screen.grid.children.len() - 1);
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
                        self.selected_widget_idx = Some(screen.grid.children.len() - 1);
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
                        self.selected_widget_idx = Some(screen.grid.children.len() - 1);
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
                        self.selected_widget_idx = Some(screen.grid.children.len() - 1);
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
                        self.selected_widget_idx = Some(screen.grid.children.len() - 1);
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
                        self.selected_widget_idx = Some(screen.grid.children.len() - 1);
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
                        self.selected_widget_idx = Some(screen.grid.children.len() - 1);
                        modified = true;
                    }
                    ui.end_row();
                });
        }

        if modified {
            self.sync_from_screen(&screen);
        }
    }
}

fn compute_track_sizes(tracks: &[GridTrackDef], available_size: f32, gap: f32) -> Vec<f32> {
    let n = tracks.len();
    if n == 0 {
        return vec![available_size];
    }
    let total_gap = gap * (n.saturating_sub(1) as f32);
    let net_space = (available_size - total_gap).max(0.0);

    let mut fixed_sum = 0.0;
    let mut total_fr = 0u32;

    for t in tracks {
        match t {
            GridTrackDef::Px(px) => fixed_sum += *px as f32,
            GridTrackDef::Fr(fr) => total_fr += *fr as u32,
            GridTrackDef::Auto => fixed_sum += 32.0,
        }
    }

    let remaining = (net_space - fixed_sum).max(0.0);
    let fr_unit = if total_fr > 0 {
        remaining / (total_fr as f32)
    } else {
        0.0
    };

    tracks
        .iter()
        .map(|t| match t {
            GridTrackDef::Px(px) => *px as f32,
            GridTrackDef::Fr(fr) => (*fr as f32) * fr_unit,
            GridTrackDef::Auto => 32.0,
        })
        .collect()
}

fn draw_animated_widget(painter: &egui::Painter, rect: Rect, widget: &WidgetDef, time: f32) {
    match widget {
        WidgetDef::Label { text, .. } => {
            painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(26, 28, 34));
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                FontId::proportional(12.0),
                Color32::from_rgb(220, 225, 235),
            );
        }
        WidgetDef::Button { text, style, .. } => {
            let bg = if style.as_deref() == Some("accent") {
                Color32::from_rgb(45, 110, 220)
            } else {
                Color32::from_rgb(50, 56, 68)
            };
            painter.rect_filled(rect, CornerRadius::same(4), bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, Color32::from_rgb(90, 100, 120)),
                StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("🔘 {}", text),
                FontId::proportional(12.0),
                Color32::WHITE,
            );
        }
        WidgetDef::Toggle { label, checked, .. } => {
            painter.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(32, 35, 42));
            let check_icon = if *checked { " [ON]" } else { " [OFF]" };
            let text_color = if *checked {
                Color32::from_rgb(80, 220, 120)
            } else {
                Color32::from_rgb(160, 165, 175)
            };
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("⏻ {}{}", label, check_icon),
                FontId::proportional(11.0),
                text_color,
            );
        }
        WidgetDef::Checkbox { label, checked, .. } => {
            painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(30, 33, 40));
            let mark = if *checked { "☑" } else { "☐" };
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{} {}", mark, label),
                FontId::proportional(11.0),
                Color32::from_rgb(200, 205, 215),
            );
        }
        WidgetDef::Slider {
            min, max, value, ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(28, 30, 36));
            let pct = if max > min {
                ((*value - *min) as f32 / (*max - *min) as f32).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let track_h = 6.0;
            let track_y = rect.center().y - track_h / 2.0;
            let track_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + 8.0, track_y),
                Vec2::new(rect.width() - 16.0, track_h),
            );
            painter.rect_filled(
                track_rect,
                CornerRadius::same(3),
                Color32::from_rgb(50, 55, 65),
            );

            let fill_w = track_rect.width() * pct;
            let fill_rect = Rect::from_min_size(track_rect.min, Vec2::new(fill_w, track_h));
            painter.rect_filled(
                fill_rect,
                CornerRadius::same(3),
                Color32::from_rgb(60, 140, 240),
            );

            painter.text(
                Pos2::new(rect.center().x, rect.min.y + 7.0),
                egui::Align2::CENTER_CENTER,
                format!("Slider: {}", value),
                FontId::proportional(10.0),
                Color32::from_rgb(180, 185, 195),
            );
        }
        WidgetDef::ProgressBar { value, .. } => {
            painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(24, 26, 32));
            let track_rect = rect.shrink2(Vec2::new(6.0, 6.0));
            painter.rect_filled(
                track_rect,
                CornerRadius::same(3),
                Color32::from_rgb(45, 50, 60),
            );

            let animated_val = if *value > 0.0 {
                *value
            } else {
                0.5 + 0.45 * (time * 2.0).sin()
            };
            let fill_w = track_rect.width() * animated_val.clamp(0.0, 1.0);
            let fill_rect =
                Rect::from_min_size(track_rect.min, Vec2::new(fill_w, track_rect.height()));
            painter.rect_filled(
                fill_rect,
                CornerRadius::same(3),
                Color32::from_rgb(40, 180, 120),
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{:.0}%", animated_val * 100.0),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
        WidgetDef::Scale {
            mode,
            min,
            max,
            value,
            ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(26, 30, 38));
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, Color32::from_rgb(60, 70, 85)),
                StrokeKind::Inside,
            );

            let dynamic_val = *value + ((*max - *min) * 0.2 * (time * 1.8).sin());
            let clamped_val = dynamic_val.clamp(*min, *max);

            painter.text(
                Pos2::new(rect.center().x, rect.center().y - 8.0),
                egui::Align2::CENTER_CENTER,
                format!("⏱ Scale ({})", mode),
                FontId::proportional(11.0),
                Color32::from_rgb(140, 180, 240),
            );
            painter.text(
                Pos2::new(rect.center().x, rect.center().y + 8.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.1} [{:.0}..{:.0}]", clamped_val, min, max),
                FontId::proportional(12.0),
                Color32::from_rgb(230, 235, 245),
            );
        }
        WidgetDef::BusyWheel { active, .. } => {
            painter.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(20, 24, 32));
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, Color32::from_rgb(45, 52, 68)),
                StrokeKind::Inside,
            );

            let center = rect.center();
            let radius = (rect.width().min(rect.height()) / 2.0 - 8.0).max(6.0);
            let num_dots = 8;
            let rotation_offset = if *active { time * 5.0 } else { 0.0 };

            for i in 0..num_dots {
                let angle = rotation_offset + (i as f32 * 2.0 * PI / num_dots as f32);
                let dot_pos = center + Vec2::new(angle.cos() * radius, angle.sin() * radius);
                let alpha = (i as f32 + 1.0) / (num_dots as f32);
                let dot_radius = 2.0 + alpha * 2.0;
                let color = Color32::from_rgba_unmultiplied(60, 160, 255, (alpha * 255.0) as u8);
                painter.circle_filled(dot_pos, dot_radius, color);
            }

            painter.text(
                Pos2::new(center.x, rect.max.y - 8.0),
                egui::Align2::CENTER_CENTER,
                "Busy Spinner",
                FontId::proportional(9.0),
                Color32::from_rgb(140, 150, 165),
            );
        }
        WidgetDef::SweepingArc {
            start_angle,
            end_angle,
            ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(20, 24, 32));
            let center = rect.center();
            let radius = (rect.width().min(rect.height()) / 2.0 - 10.0).max(8.0);

            let pulse = (time * 2.5).sin() * 0.5 + 0.5;
            let sweep_end = *start_angle as f32 + (*end_angle - *start_angle) as f32 * pulse;

            painter.circle_stroke(
                center,
                radius,
                Stroke::new(2.0f32, Color32::from_rgb(50, 60, 75)),
            );

            let mut a = *start_angle as f32;
            while a <= sweep_end {
                let rad = a.to_radians();
                let p = center + Vec2::new(rad.cos() * radius, rad.sin() * radius);
                painter.circle_filled(p, 2.5, Color32::from_rgb(80, 200, 255));
                a += 15.0;
            }

            painter.text(
                Pos2::new(center.x, center.y),
                egui::Align2::CENTER_CENTER,
                format!("{:.0}°", sweep_end),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
        WidgetDef::Plotter { mode, .. } => {
            painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(10, 14, 20));
            painter.rect_stroke(
                rect,
                CornerRadius::same(3),
                Stroke::new(1.0f32, Color32::from_rgb(35, 45, 60)),
                StrokeKind::Inside,
            );

            let grid_color = Color32::from_rgb(25, 35, 45);
            let num_v = 6;
            for i in 1..num_v {
                let x = rect.min.x + (rect.width() * i as f32 / num_v as f32);
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    Stroke::new(0.8f32, grid_color),
                );
            }
            let num_h = 4;
            for i in 1..num_h {
                let y = rect.min.y + (rect.height() * i as f32 / num_h as f32);
                painter.line_segment(
                    [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                    Stroke::new(0.8f32, grid_color),
                );
            }

            let center_y = rect.center().y;
            let amp = rect.height() * 0.35;
            let steps = (rect.width() as usize).max(20);
            let mut points = Vec::with_capacity(steps);

            for i in 0..steps {
                let x = rect.min.x + i as f32;
                let phase = (i as f32 * 0.08) - (time * 6.0);
                let y = if mode == "square" {
                    if phase.sin() >= 0.0 {
                        center_y - amp
                    } else {
                        center_y + amp
                    }
                } else {
                    center_y - (phase.sin() * amp * 0.8 + (phase * 2.0).sin() * amp * 0.2)
                };
                points.push(Pos2::new(x, y));
            }

            for win in points.windows(2) {
                painter.line_segment(
                    [win[0], win[1]],
                    Stroke::new(1.8f32, Color32::from_rgb(60, 230, 180)),
                );
            }

            painter.text(
                Pos2::new(rect.min.x + 6.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                "CH1: 1.00kHz (Live Scope)",
                FontId::proportional(9.0),
                Color32::from_rgb(60, 230, 180),
            );
        }
        WidgetDef::StatusBar { time: time_str, .. } => {
            painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(35, 38, 48));
            painter.text(
                Pos2::new(rect.min.x + 8.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                "📶  🔋 98%",
                FontId::proportional(10.0),
                Color32::from_rgb(180, 190, 200),
            );

            let colon = if (time * 2.0).fract() > 0.5 { ":" } else { " " };
            let live_time = format!("12{}34", colon);
            let display_time = if time_str.is_empty() {
                &live_time
            } else {
                time_str
            };

            painter.text(
                Pos2::new(rect.max.x - 8.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                display_time,
                FontId::proportional(11.0),
                Color32::from_rgb(220, 225, 235),
            );
        }
        WidgetDef::Panel { style, .. } => {
            let bg = if style.as_deref() == Some("card") {
                Color32::from_rgb(32, 36, 44)
            } else {
                Color32::from_rgb(25, 28, 35)
            };
            painter.rect_filled(rect, CornerRadius::same(4), bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, Color32::from_rgb(55, 60, 72)),
                StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Card Panel",
                FontId::proportional(11.0),
                Color32::from_rgb(120, 130, 145),
            );
        }
        WidgetDef::Dropdown {
            options, selected, ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(38, 42, 52));
            painter.rect_stroke(
                rect,
                CornerRadius::same(3),
                Stroke::new(1.0f32, Color32::from_rgb(70, 80, 95)),
                StrokeKind::Inside,
            );
            let item = options
                .get(*selected)
                .map(|s| s.as_str())
                .unwrap_or("(none)");
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("▼ {}", item),
                FontId::proportional(11.0),
                Color32::from_rgb(210, 215, 225),
            );
        }
        WidgetDef::Roller {
            options, selected, ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(25, 28, 35));
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, Color32::from_rgb(60, 68, 82)),
                StrokeKind::Inside,
            );

            if !options.is_empty() {
                let len = options.len();
                let sel = (*selected).min(len.saturating_sub(1));
                let prev_idx = (sel + len - 1) % len;
                let next_idx = (sel + 1) % len;

                let row_h = rect.height() / 3.0;

                // Previous option (faded top row)
                let prev_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), row_h));
                painter.text(
                    prev_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &options[prev_idx],
                    FontId::proportional(9.5),
                    Color32::from_rgb(110, 118, 130),
                );

                // Active / selected option (highlighted center row)
                let cur_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 2.0, rect.min.y + row_h),
                    Vec2::new(rect.width() - 4.0, row_h),
                );
                painter.rect_filled(
                    cur_rect,
                    CornerRadius::same(2),
                    Color32::from_rgb(45, 105, 205),
                );
                painter.text(
                    cur_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &options[sel],
                    FontId::proportional(11.0),
                    Color32::WHITE,
                );

                // Next option (faded bottom row)
                let next_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, rect.min.y + 2.0 * row_h),
                    Vec2::new(rect.width(), row_h),
                );
                painter.text(
                    next_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &options[next_idx],
                    FontId::proportional(9.5),
                    Color32::from_rgb(110, 118, 130),
                );
            }
        }
        WidgetDef::Spacer => {}
        _ => {
            painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(28, 30, 36));
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                widget.id().unwrap_or("widget"),
                FontId::proportional(10.0),
                Color32::from_rgb(160, 170, 180),
            );
        }
    }
}

impl eframe::App for EmbeddedGuiStudio {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle Toast Timer
        if self.copied_toast_timer > 0.0 {
            self.copied_toast_timer -= ctx.input(|i| i.stable_dt);
        }

        // Advance animation timeline clock and request next frame repaint
        if self.is_playing {
            let dt = ctx.input(|i| i.stable_dt);
            self.timeline_time += dt * self.playback_speed;
            if self.timeline_time > self.loop_duration {
                self.timeline_time %= self.loop_duration;
            }
            ctx.request_repaint();
        }

        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(egui::RichText::new("⚡ Embedded GUI Studio").strong());
                ui.separator();

                ui.menu_button("📄 Presets", |ui| {
                    if ui.button("📈 Live Oscilloscope").clicked() {
                        self.kdl_source = SAMPLE_WAVEFORM.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("✨ Motion Kitchen Sink").clicked() {
                        self.kdl_source = SAMPLE_MOTION_KITCHEN_SINK.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("🌡 Smart Thermostat").clicked() {
                        self.kdl_source = SAMPLE_THERMOSTAT.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("📊 Sensor Dashboard").clicked() {
                        self.kdl_source = SAMPLE_DASHBOARD.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                });

                if ui.button("📋 Copy Rust Code").clicked() {
                    ctx.copy_text(self.generated_rust.clone());
                    self.copied_toast_timer = 2.0;
                }

                if self.copied_toast_timer > 0.0 {
                    ui.colored_label(Color32::from_rgb(80, 220, 120), "✓ Copied to clipboard!");
                }

                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| match &self.parsed_screen {
                        Ok(screen) => {
                            ui.colored_label(
                                Color32::from_rgb(80, 220, 120),
                                format!("✓ Valid ({} nodes)", screen.grid.children.len()),
                            );
                        }
                        Err(_) => {
                            ui.colored_label(Color32::from_rgb(255, 100, 100), "✗ Syntax Error");
                        }
                    },
                );
            });
        });

        // Left Panel: KDL Code Editor
        egui::SidePanel::left("editor_panel")
            .min_width(340.0)
            .default_width(380.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("KDL Screen Definition");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Clear").clicked() {
                            self.kdl_source.clear();
                            self.selected_widget_idx = None;
                            self.recompile();
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 70.0)
                    .show(ui, |ui| {
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut self.kdl_source)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(24)
                                .desired_width(f32::INFINITY)
                                .lock_focus(true),
                        );
                        if response.changed() {
                            self.recompile();
                        }
                    });

                if let Err(err) = &self.parsed_screen {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(255, 90, 90), format!("⚠️ {}", err));
                }
            });

        // Right Panel: Visual Property Inspector
        egui::SidePanel::right("inspector_panel")
            .min_width(260.0)
            .default_width(300.0)
            .show(ctx, |ui| {
                self.render_inspector_panel(ui);
            });

        // Center Panel: Tabs (Visual Preview / Rust Codegen / AST)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.active_tab,
                    StudioTab::VisualPreview,
                    "🖥 Visual Preview",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    StudioTab::RustCodegen,
                    "🦀 Generated Rust",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    StudioTab::AstHierarchy,
                    "🌲 AST Inspector",
                );
            });
            ui.separator();

            match self.active_tab {
                StudioTab::VisualPreview => {
                    if let Ok(screen) = self.parsed_screen.clone() {
                        self.render_visual_preview(ui, &screen);
                    } else {
                        ui.label("Fix KDL syntax errors to display preview.");
                    }
                }
                StudioTab::RustCodegen => {
                    egui::ScrollArea::both().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.generated_rust.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY),
                        );
                    });
                }
                StudioTab::AstHierarchy => {
                    if let Ok(screen) = &self.parsed_screen {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.heading(format!("Screen: {}", screen.id));
                            ui.label(format!("Dimensions: {}x{}", screen.width, screen.height));
                            ui.label(format!("Cols: {:?}", screen.grid.cols));
                            ui.label(format!("Rows: {:?}", screen.grid.rows));
                            ui.label(format!(
                                "Gap: {}, Padding: {}",
                                screen.grid.gap, screen.grid.padding
                            ));
                            ui.separator();
                            ui.heading("Widget Placements:");
                            for (idx, (p, w)) in screen.grid.children.iter().enumerate() {
                                let label_str = format!(
                                    "{} • [c:{}, r:{}, span:{}x{}] {:?}",
                                    if self.selected_widget_idx == Some(idx) {
                                        "👉"
                                    } else {
                                        " "
                                    },
                                    p.col,
                                    p.row,
                                    p.col_span,
                                    p.row_span,
                                    w
                                );
                                if ui
                                    .selectable_label(
                                        self.selected_widget_idx == Some(idx),
                                        label_str,
                                    )
                                    .clicked()
                                {
                                    self.selected_widget_idx = Some(idx);
                                }
                            }
                        });
                    } else {
                        ui.label("No AST available.");
                    }
                }
            }
        });
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
        Box::new(|_cc| Ok(Box::new(EmbeddedGuiStudio::default()))),
    )
}
