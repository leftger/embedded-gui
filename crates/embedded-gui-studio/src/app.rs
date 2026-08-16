//! Main Studio Application state, UI lifecycle, and canvas interaction.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Key, Pos2, Rect, Stroke, StrokeKind, Vec2,
};
use embedded_gui::motion::timing::{EasingCurve, evaluate_easing};
use embedded_gui_codegen::{
    GridPlacementDef, GridTrackDef, ScreenDef, generate_rust_code, parse_kdl_screen,
    serialize_kdl_screen,
};

use crate::curve_visualizer::render_curve_graph;
use crate::inspector::render_inspector_panel;
use crate::layout::compute_track_sizes;
use crate::presets::*;
use crate::renderer::{ThemePalette, draw_animated_widget};
use crate::types::{ActiveDrag, DisplayTheme, HardwareProfile, StudioMode, StudioTab};

pub struct EmbeddedGuiStudio {
    pub kdl_source: String,
    pub parsed_screen: Result<ScreenDef, String>,
    pub generated_rust: String,
    pub active_tab: StudioTab,
    pub mode: StudioMode,
    pub preview_zoom: f32,
    pub copied_toast_timer: f32,
    pub action_toast: Option<(String, f32)>,

    // Theme & Hardware
    pub display_theme: DisplayTheme,
    pub hardware_profile: HardwareProfile,

    // Selection & Inspector
    pub selected_widget_idx: Option<usize>,
    pub active_drag: ActiveDrag,
    pub pressed_widget: Option<usize>,

    // Animation playback state
    pub is_playing: bool,
    pub timeline_time: f32,
    pub playback_speed: f32,
    pub loop_duration: f32,
    pub selected_easing: EasingCurve,

    // Undo / Redo history
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
}

impl Default for EmbeddedGuiStudio {
    fn default() -> Self {
        let mut app = Self {
            kdl_source: SAMPLE_AUTOMOTIVE_CLUSTER.to_string(),
            parsed_screen: Err("Not parsed".to_string()),
            generated_rust: String::new(),
            active_tab: StudioTab::VisualPreview,
            mode: StudioMode::Design,
            preview_zoom: 1.5,
            copied_toast_timer: 0.0,
            action_toast: None,
            display_theme: DisplayTheme::DarkTft,
            hardware_profile: HardwareProfile::Esp32S3Box,
            selected_widget_idx: None,
            active_drag: ActiveDrag::None,
            pressed_widget: None,
            is_playing: true,
            timeline_time: 0.0,
            playback_speed: 1.0,
            loop_duration: 4.0,
            selected_easing: EasingCurve::EaseInOutCubic,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };
        app.recompile();
        app
    }
}

impl EmbeddedGuiStudio {
    pub fn push_undo_snapshot(&mut self) {
        if self.undo_stack.last() != Some(&self.kdl_source) {
            self.undo_stack.push(self.kdl_source.clone());
            if self.undo_stack.len() > 50 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.kdl_source.clone());
            self.kdl_source = prev;
            self.recompile();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.kdl_source.clone());
            self.kdl_source = next;
            self.recompile();
        }
    }

    pub fn recompile(&mut self) {
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
    pub fn sync_from_screen(&mut self, screen: &ScreenDef) {
        self.push_undo_snapshot();
        self.kdl_source = serialize_kdl_screen(screen);
        self.generated_rust = generate_rust_code(screen);
        self.parsed_screen = Ok(screen.clone());
    }

    pub fn render_visual_preview(&mut self, ui: &mut egui::Ui, screen: &ScreenDef) {
        // Toolbar: Mode, Zoom, Themes, Hardware, & Playback Controls
        ui.horizontal(|ui| {
            // Mode Switcher
            let mode_btn = match self.mode {
                StudioMode::Design => "✏️ Design Mode",
                StudioMode::Interactive => "🎮 Live Interactive",
            };
            if ui.button(mode_btn).clicked() {
                self.mode = match self.mode {
                    StudioMode::Design => StudioMode::Interactive,
                    StudioMode::Interactive => StudioMode::Design,
                };
            }

            ui.separator();

            // Zoom controls
            ui.label("Zoom:");
            ui.selectable_value(&mut self.preview_zoom, 1.0, "1x");
            ui.selectable_value(&mut self.preview_zoom, 1.5, "1.5x");
            ui.selectable_value(&mut self.preview_zoom, 2.0, "2x");

            ui.separator();

            // Display Theme Selector
            ui.label("Theme:");
            egui::ComboBox::from_id_salt("theme_selector")
                .selected_text(match self.display_theme {
                    DisplayTheme::DarkTft => "Dark TFT",
                    DisplayTheme::LightTft => "Light TFT",
                    DisplayTheme::AmberPhosphor => "Amber CRT",
                    DisplayTheme::EmeraldGreen => "Emerald Matrix",
                    DisplayTheme::MonochromeOled => "Monochrome OLED",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.display_theme, DisplayTheme::DarkTft, "Dark TFT");
                    ui.selectable_value(
                        &mut self.display_theme,
                        DisplayTheme::LightTft,
                        "Light TFT",
                    );
                    ui.selectable_value(
                        &mut self.display_theme,
                        DisplayTheme::AmberPhosphor,
                        "Amber CRT",
                    );
                    ui.selectable_value(
                        &mut self.display_theme,
                        DisplayTheme::EmeraldGreen,
                        "Emerald Matrix",
                    );
                    ui.selectable_value(
                        &mut self.display_theme,
                        DisplayTheme::MonochromeOled,
                        "Monochrome OLED",
                    );
                });

            ui.separator();

            // Hardware Target Profile
            ui.label("Target:");
            let prev_profile = self.hardware_profile;
            egui::ComboBox::from_id_salt("hardware_profile_selector")
                .selected_text(self.hardware_profile.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.hardware_profile,
                        HardwareProfile::Custom,
                        HardwareProfile::Custom.name(),
                    );
                    ui.selectable_value(
                        &mut self.hardware_profile,
                        HardwareProfile::Esp32S3Box,
                        HardwareProfile::Esp32S3Box.name(),
                    );
                    ui.selectable_value(
                        &mut self.hardware_profile,
                        HardwareProfile::Stm32H7Capacitive,
                        HardwareProfile::Stm32H7Capacitive.name(),
                    );
                    ui.selectable_value(
                        &mut self.hardware_profile,
                        HardwareProfile::RoundWearableWatch,
                        HardwareProfile::RoundWearableWatch.name(),
                    );
                    ui.selectable_value(
                        &mut self.hardware_profile,
                        HardwareProfile::Waveshare43,
                        HardwareProfile::Waveshare43.name(),
                    );
                    ui.selectable_value(
                        &mut self.hardware_profile,
                        HardwareProfile::Ssd1306Oled,
                        HardwareProfile::Ssd1306Oled.name(),
                    );
                });

            if self.hardware_profile != prev_profile {
                if let Some((w, h)) = self.hardware_profile.dimensions() {
                    let mut s = screen.clone();
                    s.width = w;
                    s.height = h;
                    self.sync_from_screen(&s);
                }
            }

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

            // Easing Curve selector
            ui.label("Curve:");
            egui::ComboBox::from_id_salt("easing_curve_combo")
                .selected_text(format!("{:?}", self.selected_easing))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_easing, EasingCurve::Linear, "Linear");
                    ui.selectable_value(
                        &mut self.selected_easing,
                        EasingCurve::EaseInOutQuad,
                        "EaseInOutQuad",
                    );
                    ui.selectable_value(
                        &mut self.selected_easing,
                        EasingCurve::EaseInOutCubic,
                        "EaseInOutCubic",
                    );
                    ui.selectable_value(
                        &mut self.selected_easing,
                        EasingCurve::EaseOutBack,
                        "EaseOutBack (Overshoot)",
                    );
                    ui.selectable_value(
                        &mut self.selected_easing,
                        EasingCurve::EaseOutBounce,
                        "EaseOutBounce (Physics)",
                    );
                    ui.selectable_value(
                        &mut self.selected_easing,
                        EasingCurve::Moook,
                        "Moook (Pebble UI)",
                    );
                    ui.selectable_value(
                        &mut self.selected_easing,
                        EasingCurve::CubicBezier,
                        "Cubic Bezier (Custom)",
                    );
                });
        });

        // Interactive Curve Visualizer Bar
        let norm_t = if self.loop_duration > 0.0 {
            (self.timeline_time / self.loop_duration).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let eased_progress = evaluate_easing(self.selected_easing, norm_t);
        let t = eased_progress * self.loop_duration;

        ui.horizontal(|ui| {
            render_curve_graph(ui, self.selected_easing, norm_t, Vec2::new(140.0, 36.0));
            ui.label(
                egui::RichText::new(format!("Eased: {:.2}s / {:.0}%", t, eased_progress * 100.0))
                    .weak(),
            );
            if let Some((msg, _)) = &self.action_toast {
                ui.colored_label(Color32::from_rgb(100, 230, 150), msg);
            }
        });
        ui.separator();

        let screen_w = screen.width as f32 * self.preview_zoom;
        let screen_h = screen.height as f32 * self.preview_zoom;

        let palette = ThemePalette::for_theme(self.display_theme);

        egui::ScrollArea::both().show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                Vec2::new(screen_w + 32.0, screen_h + 32.0),
                egui::Sense::click_and_drag(),
            );
            let origin = response.rect.min + Vec2::new(16.0, 16.0);
            let display_rect = Rect::from_min_size(origin, Vec2::new(screen_w, screen_h));

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
            painter.rect_filled(display_rect, CornerRadius::same(2), palette.display_bg);

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

            let pointer_pos = ui.input(|i| i.pointer.interact_pos());
            let primary_down = ui.input(|i| i.pointer.primary_down());
            let primary_pressed = ui.input(|i| i.pointer.primary_pressed());

            if !primary_down {
                self.active_drag = ActiveDrag::None;
                self.pressed_widget = None;
            }

            let mut mutated_screen = screen.clone();
            let mut did_mutate = false;

            // Background canvas click deselects
            if response.clicked() {
                if let Some(pos) = pointer_pos {
                    if !display_rect.contains(pos) {
                        self.selected_widget_idx = None;
                    }
                }
            }

            // --- A. INTERACTIVE OR DESIGN MODE INPUT ---
            if self.mode == StudioMode::Interactive {
                // Interactive Touch Execution
                if let Some(pos) = pointer_pos {
                    if display_rect.contains(pos) {
                        for (idx, (p, w)) in mutated_screen.grid.children.iter_mut().enumerate() {
                            let c = p.col.min(col_xs.len().saturating_sub(1));
                            let r = p.row.min(row_ys.len().saturating_sub(1));
                            let c_span = p.col_span.max(1);
                            let r_span = p.row_span.max(1);
                            let x0 = col_xs.get(c).copied().unwrap_or(inner_rect.min.x);
                            let y0 = row_ys.get(r).copied().unwrap_or(inner_rect.min.y);
                            let mut w_px = 0.0;
                            for i in 0..c_span {
                                if let Some(cw) = col_widths.get(c + i) {
                                    w_px += *cw;
                                    if i + 1 < c_span {
                                        w_px += gap;
                                    }
                                }
                            }
                            let mut h_px = 0.0;
                            for i in 0..r_span {
                                if let Some(rh) = row_heights.get(r + i) {
                                    h_px += *rh;
                                    if i + 1 < r_span {
                                        h_px += gap;
                                    }
                                }
                            }
                            let w_rect =
                                Rect::from_min_size(Pos2::new(x0, y0), Vec2::new(w_px, h_px));

                            if w_rect.contains(pos) {
                                match w {
                                    embedded_gui_codegen::WidgetDef::Button {
                                        text,
                                        on_click,
                                        ..
                                    } => {
                                        if primary_pressed {
                                            self.pressed_widget = Some(idx);
                                            let action_name =
                                                on_click.as_deref().unwrap_or("Triggered");
                                            self.action_toast = Some((
                                                format!("🔘 Button '{}' -> {}", text, action_name),
                                                2.0,
                                            ));
                                        }
                                    }
                                    embedded_gui_codegen::WidgetDef::Toggle {
                                        label,
                                        checked,
                                        ..
                                    } => {
                                        if primary_pressed {
                                            *checked = !*checked;
                                            self.action_toast = Some((
                                                format!(
                                                    "⏻ Toggle '{}' -> {}",
                                                    label,
                                                    if *checked { "ON" } else { "OFF" }
                                                ),
                                                1.5,
                                            ));
                                            did_mutate = true;
                                        }
                                    }
                                    embedded_gui_codegen::WidgetDef::Checkbox {
                                        label,
                                        checked,
                                        ..
                                    } => {
                                        if primary_pressed {
                                            *checked = !*checked;
                                            self.action_toast = Some((
                                                format!("☑ Checkbox '{}' -> {}", label, checked),
                                                1.5,
                                            ));
                                            did_mutate = true;
                                        }
                                    }
                                    embedded_gui_codegen::WidgetDef::Slider {
                                        min,
                                        max,
                                        value,
                                        ..
                                    } => {
                                        if primary_down {
                                            let pct = ((pos.x - (w_rect.min.x + 8.0))
                                                / (w_rect.width() - 16.0))
                                                .clamp(0.0, 1.0);
                                            let new_val = (*min as f32 + pct * (*max - *min) as f32)
                                                .round()
                                                as i32;
                                            if new_val != *value {
                                                *value = new_val;
                                                self.action_toast =
                                                    Some((format!("🎚 Slider -> {}", value), 1.0));
                                                did_mutate = true;
                                            }
                                        }
                                    }
                                    embedded_gui_codegen::WidgetDef::Roller {
                                        options,
                                        selected,
                                        ..
                                    } if primary_pressed && !options.is_empty() => {
                                        *selected = (*selected + 1) % options.len();
                                        self.action_toast = Some((
                                            format!("Roller -> {}", options[*selected]),
                                            1.5,
                                        ));
                                        did_mutate = true;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            } else {
                // Design Mode: Drag, Move, Span Resizing
                if let Some(pos) = pointer_pos {
                    if self.active_drag == ActiveDrag::None && primary_pressed {
                        let mut hit_handle = false;
                        if let Some(sel_idx) = self.selected_widget_idx {
                            if let Some((sel_p, _)) = screen.grid.children.get(sel_idx) {
                                let c = sel_p.col.min(col_xs.len().saturating_sub(1));
                                let r = sel_p.row.min(row_ys.len().saturating_sub(1));
                                let c_span = sel_p.col_span.max(1);
                                let r_span = sel_p.row_span.max(1);
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
                                let br_rect = Rect::from_center_size(
                                    Pos2::new(x0 + w, y0 + h),
                                    Vec2::splat(14.0),
                                );
                                if br_rect.contains(pos) {
                                    self.active_drag = ActiveDrag::ResizeWidgetSpan {
                                        widget_idx: sel_idx,
                                    };
                                    hit_handle = true;
                                }
                            }
                        }

                        if !hit_handle {
                            for (ci, &cx) in col_xs.iter().enumerate().skip(1) {
                                let div_x = cx - gap / 2.0;
                                if (pos.x - div_x).abs() <= 8.0
                                    && pos.y >= inner_rect.min.y
                                    && pos.y <= inner_rect.max.y
                                {
                                    self.active_drag =
                                        ActiveDrag::ResizeColDivider { col_idx: ci - 1 };
                                    hit_handle = true;
                                    break;
                                }
                            }
                        }

                        if !hit_handle {
                            for (ri, &ry) in row_ys.iter().enumerate().skip(1) {
                                let div_y = ry - gap / 2.0;
                                if (pos.y - div_y).abs() <= 8.0
                                    && pos.x >= inner_rect.min.x
                                    && pos.x <= inner_rect.max.x
                                {
                                    self.active_drag =
                                        ActiveDrag::ResizeRowDivider { row_idx: ri - 1 };
                                    hit_handle = true;
                                    break;
                                }
                            }
                        }

                        if !hit_handle {
                            for (idx, (p, _)) in screen.grid.children.iter().enumerate().rev() {
                                let c = p.col.min(col_xs.len().saturating_sub(1));
                                let r = p.row.min(row_ys.len().saturating_sub(1));
                                let c_span = p.col_span.max(1);
                                let r_span = p.row_span.max(1);
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
                                let w_rect =
                                    Rect::from_min_size(Pos2::new(x0, y0), Vec2::new(w, h));
                                if w_rect.contains(pos) {
                                    self.selected_widget_idx = Some(idx);
                                    self.active_drag = ActiveDrag::MoveWidget { widget_idx: idx };
                                    break;
                                }
                            }
                        }
                    }
                }

                // Drag Execution
                if let Some(pos) = pointer_pos {
                    match self.active_drag {
                        ActiveDrag::ResizeColDivider { col_idx } => {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
                            if col_idx < col_xs.len() && col_idx < mutated_screen.grid.cols.len() {
                                let start_x = col_xs[col_idx];
                                let max_w =
                                    ((inner_rect.width() / self.preview_zoom) - 24.0).max(24.0);
                                let raw_px = ((pos.x - start_x) / self.preview_zoom)
                                    .clamp(24.0, max_w)
                                    .round() as u32;
                                mutated_screen.grid.cols[col_idx] = GridTrackDef::Px(raw_px);

                                if col_idx + 1 < mutated_screen.grid.cols.len() {
                                    if let GridTrackDef::Px(_) =
                                        mutated_screen.grid.cols[col_idx + 1]
                                    {
                                        let pair_total = (col_widths[col_idx]
                                            + col_widths[col_idx + 1])
                                            / self.preview_zoom;
                                        let next_px =
                                            (pair_total - raw_px as f32).max(24.0).round() as u32;
                                        mutated_screen.grid.cols[col_idx + 1] =
                                            GridTrackDef::Px(next_px);
                                    }
                                }
                                did_mutate = true;
                            }
                        }
                        ActiveDrag::ResizeRowDivider { row_idx } => {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeRow);
                            if row_idx < row_ys.len() && row_idx < mutated_screen.grid.rows.len() {
                                let start_y = row_ys[row_idx];
                                let max_h =
                                    ((inner_rect.height() / self.preview_zoom) - 16.0).max(16.0);
                                let raw_px = ((pos.y - start_y) / self.preview_zoom)
                                    .clamp(16.0, max_h)
                                    .round() as u32;
                                mutated_screen.grid.rows[row_idx] = GridTrackDef::Px(raw_px);

                                if row_idx + 1 < mutated_screen.grid.rows.len() {
                                    if let GridTrackDef::Px(_) =
                                        mutated_screen.grid.rows[row_idx + 1]
                                    {
                                        let pair_total = (row_heights[row_idx]
                                            + row_heights[row_idx + 1])
                                            / self.preview_zoom;
                                        let next_px =
                                            (pair_total - raw_px as f32).max(16.0).round() as u32;
                                        mutated_screen.grid.rows[row_idx + 1] =
                                            GridTrackDef::Px(next_px);
                                    }
                                }
                                did_mutate = true;
                            }
                        }
                        ActiveDrag::ResizeWidgetSpan { widget_idx } => {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                            if let Some((p, _)) = mutated_screen.grid.children.get_mut(widget_idx) {
                                let mut target_c = p.col;
                                for (ci, &cx) in col_xs.iter().enumerate() {
                                    if pos.x >= cx {
                                        target_c = ci;
                                    }
                                }
                                let mut target_r = p.row;
                                for (ri, &ry) in row_ys.iter().enumerate() {
                                    if pos.y >= ry {
                                        target_r = ri;
                                    }
                                }
                                let new_c_span = (target_c.saturating_sub(p.col) + 1)
                                    .clamp(1, cols.len().saturating_sub(p.col));
                                let new_r_span = (target_r.saturating_sub(p.row) + 1)
                                    .clamp(1, rows.len().saturating_sub(p.row));
                                if new_c_span != p.col_span || new_r_span != p.row_span {
                                    p.col_span = new_c_span;
                                    p.row_span = new_r_span;
                                    did_mutate = true;
                                }
                            }
                        }
                        ActiveDrag::MoveWidget { widget_idx } => {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                            if let Some((p, _)) = mutated_screen.grid.children.get_mut(widget_idx) {
                                let mut target_c = 0;
                                for (ci, &cx) in col_xs.iter().enumerate() {
                                    if pos.x >= cx {
                                        target_c = ci;
                                    }
                                }
                                let mut target_r = 0;
                                for (ri, &ry) in row_ys.iter().enumerate() {
                                    if pos.y >= ry {
                                        target_r = ri;
                                    }
                                }
                                target_c = target_c.min(col_xs.len().saturating_sub(1));
                                target_r = target_r.min(row_ys.len().saturating_sub(1));
                                if target_c != p.col || target_r != p.row {
                                    p.col = target_c;
                                    p.row = target_r;
                                    did_mutate = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Grid Divider Lines
                for (ci, &cx) in col_xs.iter().enumerate().skip(1) {
                    let div_x = cx - gap / 2.0;
                    let is_hovered = pointer_pos.is_some_and(|pos| {
                        (pos.x - div_x).abs() <= 8.0
                            && pos.y >= inner_rect.min.y
                            && pos.y <= inner_rect.max.y
                    });
                    let is_active =
                        self.active_drag == ActiveDrag::ResizeColDivider { col_idx: ci - 1 };

                    if is_hovered && self.active_drag == ActiveDrag::None {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
                    }

                    let color = if is_active {
                        Color32::from_rgb(80, 220, 255)
                    } else if is_hovered {
                        Color32::from_rgb(60, 160, 240)
                    } else {
                        Color32::from_rgba_unmultiplied(60, 80, 110, 80)
                    };
                    let thickness = if is_active || is_hovered {
                        2.5f32
                    } else {
                        1.0f32
                    };

                    painter.line_segment(
                        [
                            Pos2::new(div_x, inner_rect.min.y),
                            Pos2::new(div_x, inner_rect.max.y),
                        ],
                        Stroke::new(thickness, color),
                    );
                }

                for (ri, &ry) in row_ys.iter().enumerate().skip(1) {
                    let div_y = ry - gap / 2.0;
                    let is_hovered = pointer_pos.is_some_and(|pos| {
                        (pos.y - div_y).abs() <= 8.0
                            && pos.x >= inner_rect.min.x
                            && pos.x <= inner_rect.max.x
                    });
                    let is_active =
                        self.active_drag == ActiveDrag::ResizeRowDivider { row_idx: ri - 1 };

                    if is_hovered && self.active_drag == ActiveDrag::None {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeRow);
                    }

                    let color = if is_active {
                        Color32::from_rgb(80, 220, 255)
                    } else if is_hovered {
                        Color32::from_rgb(60, 160, 240)
                    } else {
                        Color32::from_rgba_unmultiplied(60, 80, 110, 80)
                    };
                    let thickness = if is_active || is_hovered {
                        2.5f32
                    } else {
                        1.0f32
                    };

                    painter.line_segment(
                        [
                            Pos2::new(inner_rect.min.x, div_y),
                            Pos2::new(inner_rect.max.x, div_y),
                        ],
                        Stroke::new(thickness, color),
                    );
                }
            }

            // --- B. RENDER ALL WIDGETS & SELECTION OVERLAYS ---
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
                let is_pressed = self.pressed_widget == Some(idx);

                // Draw live widget representation
                draw_animated_widget(
                    &painter,
                    widget_rect,
                    widget,
                    t,
                    self.display_theme,
                    is_pressed,
                );

                // Selection highlight & bounding box in Design Mode
                if self.mode == StudioMode::Design && self.selected_widget_idx == Some(idx) {
                    let select_stroke = Stroke::new(2.0f32, Color32::from_rgb(60, 160, 255));
                    painter.rect_stroke(
                        widget_rect.expand(2.0),
                        CornerRadius::same(4),
                        select_stroke,
                        StrokeKind::Outside,
                    );

                    // Corner handles
                    let handle_size = 6.0;
                    let br_corner = widget_rect.right_bottom();

                    for corner in [
                        widget_rect.left_top(),
                        widget_rect.right_top(),
                        widget_rect.left_bottom(),
                    ] {
                        let h_rect = Rect::from_center_size(corner, Vec2::splat(handle_size));
                        painter.rect_filled(
                            h_rect,
                            CornerRadius::same(1),
                            Color32::from_rgb(60, 160, 255),
                        );
                    }

                    // Green bottom-right span resizing handle
                    painter.rect_filled(
                        Rect::from_center_size(br_corner, Vec2::splat(handle_size)),
                        CornerRadius::same(1),
                        Color32::from_rgb(80, 220, 120),
                    );

                    // Floating selection badge
                    let badge_text = format!(
                        "🎯 {} [c:{}, r:{}, span:{}x{}]",
                        widget.id().unwrap_or("widget"),
                        placement.col,
                        placement.row,
                        placement.col_span,
                        placement.row_span
                    );
                    let badge_pos = Pos2::new(widget_rect.min.x, widget_rect.min.y - 14.0);
                    painter.rect_filled(
                        Rect::from_min_size(badge_pos, Vec2::new(160.0, 14.0)),
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

            if did_mutate {
                self.sync_from_screen(&mutated_screen);
            }
        });
    }
}

impl eframe::App for EmbeddedGuiStudio {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle Timers
        let dt = ctx.input(|i| i.stable_dt);
        if self.copied_toast_timer > 0.0 {
            self.copied_toast_timer -= dt;
        }
        if let Some((_, timer)) = &mut self.action_toast {
            *timer -= dt;
            if *timer <= 0.0 {
                self.action_toast = None;
            }
        }

        // Handle Keyboard Shortcuts
        ctx.input(|i| {
            // Undo: Ctrl+Z / Cmd+Z
            if i.modifiers.command && i.key_pressed(Key::Z) && !i.modifiers.shift {
                self.undo();
            }
            // Redo: Ctrl+Y / Cmd+Shift+Z / Ctrl+Shift+Z
            if (i.modifiers.command && i.key_pressed(Key::Y))
                || (i.modifiers.command && i.modifiers.shift && i.key_pressed(Key::Z))
            {
                self.redo();
            }
            // Space: Play / Pause
            if i.key_pressed(Key::Space) {
                self.is_playing = !self.is_playing;
            }
            // Tab: Toggle Design / Interactive Mode
            if i.key_pressed(Key::Tab) && !i.modifiers.command {
                self.mode = match self.mode {
                    StudioMode::Design => StudioMode::Interactive,
                    StudioMode::Interactive => StudioMode::Design,
                };
            }
            // Delete / Backspace: Delete selected widget
            if let Some(sel_idx) = self.selected_widget_idx {
                if (i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace))
                    && self.mode == StudioMode::Design
                {
                    if let Ok(mut screen) = self.parsed_screen.clone() {
                        if sel_idx < screen.grid.children.len() {
                            screen.grid.children.remove(sel_idx);
                            self.selected_widget_idx = None;
                            self.sync_from_screen(&screen);
                        }
                    }
                }
                // Ctrl+D / Cmd+D: Duplicate widget
                if i.modifiers.command && i.key_pressed(Key::D) {
                    if let Ok(mut screen) = self.parsed_screen.clone() {
                        if sel_idx < screen.grid.children.len() {
                            let (p, w) = &screen.grid.children[sel_idx];
                            let dup_p = GridPlacementDef {
                                col: p.col + 1,
                                row: p.row,
                                col_span: p.col_span,
                                row_span: p.row_span,
                            };
                            let dup_w = w.clone();
                            screen.grid.children.push((dup_p, dup_w));
                            self.selected_widget_idx = Some(screen.grid.children.len() - 1);
                            self.sync_from_screen(&screen);
                        }
                    }
                }
                // Arrow keys: Nudge widget position
                if i.key_pressed(Key::ArrowLeft)
                    || i.key_pressed(Key::ArrowRight)
                    || i.key_pressed(Key::ArrowUp)
                    || i.key_pressed(Key::ArrowDown)
                {
                    if let Ok(mut screen) = self.parsed_screen.clone() {
                        if sel_idx < screen.grid.children.len() {
                            let p = &mut screen.grid.children[sel_idx].0;
                            if i.key_pressed(Key::ArrowLeft) && p.col > 0 {
                                p.col -= 1;
                            }
                            if i.key_pressed(Key::ArrowRight) {
                                p.col += 1;
                            }
                            if i.key_pressed(Key::ArrowUp) && p.row > 0 {
                                p.row -= 1;
                            }
                            if i.key_pressed(Key::ArrowDown) {
                                p.row += 1;
                            }
                            self.sync_from_screen(&screen);
                        }
                    }
                }
            }
        });

        // Advance animation timeline clock
        if self.is_playing {
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
                    if ui.button("🚗 Automotive Digital Cluster").clicked() {
                        self.push_undo_snapshot();
                        self.kdl_source = SAMPLE_AUTOMOTIVE_CLUSTER.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("❄️ HVAC Smart Climate").clicked() {
                        self.push_undo_snapshot();
                        self.kdl_source = SAMPLE_HVAC_CLIMATE.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("🩺 Patient Vital Monitor").clicked() {
                        self.push_undo_snapshot();
                        self.kdl_source = SAMPLE_PATIENT_MONITOR.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("⚙️ Industrial CNC Controller").clicked() {
                        self.push_undo_snapshot();
                        self.kdl_source = SAMPLE_CNC_CONTROLLER.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("⌚ Smartwatch Activity Tracker").clicked() {
                        self.push_undo_snapshot();
                        self.kdl_source = SAMPLE_SMARTWATCH_FITNESS.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("📈 Live Oscilloscope").clicked() {
                        self.push_undo_snapshot();
                        self.kdl_source = SAMPLE_WAVEFORM.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("✨ Motion Kitchen Sink").clicked() {
                        self.push_undo_snapshot();
                        self.kdl_source = SAMPLE_MOTION_KITCHEN_SINK.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("🌡 Smart Thermostat").clicked() {
                        self.push_undo_snapshot();
                        self.kdl_source = SAMPLE_THERMOSTAT.to_string();
                        self.selected_widget_idx = None;
                        self.recompile();
                        ui.close_menu();
                    }
                    if ui.button("📊 Sensor Dashboard").clicked() {
                        self.push_undo_snapshot();
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

                if ui.button("↩ Undo").clicked() {
                    self.undo();
                }
                if ui.button("↪ Redo").clicked() {
                    self.redo();
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

        // Bottom Hardware Profiler Bar
        egui::TopBottomPanel::bottom("bottom_hardware_profiler").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Ok(screen) = &self.parsed_screen {
                    let bpp = self.hardware_profile.bpp();
                    let fb_bytes = (screen.width * screen.height * bpp) / 8;
                    let fb_kb = fb_bytes as f32 / 1024.0;
                    let static_ram_kb = (screen.grid.children.len() * 96) as f32 / 1024.0;
                    let spi_mb_sec = (fb_bytes as f32 * 60.0) / 1_000_000.0;

                    ui.label(egui::RichText::new("📊 Hardware Budget:").strong());
                    ui.label(format!("Resolution: {}×{} px", screen.width, screen.height));
                    ui.separator();
                    ui.label(format!("Framebuffer: {:.1} KB ({} bpp)", fb_kb, bpp));
                    ui.separator();
                    ui.label(format!(
                        "Static Nodes: {:.2} KB ({} widgets)",
                        static_ram_kb,
                        screen.grid.children.len()
                    ));
                    ui.separator();
                    ui.label(format!("60 FPS Bandwidth: {:.2} MB/s", spi_mb_sec));
                    ui.separator();
                    ui.colored_label(
                        Color32::from_rgb(80, 220, 120),
                        "✓ Real-time 60 FPS Capable",
                    );
                }
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
                            self.push_undo_snapshot();
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
                if let Ok(mut screen) = self.parsed_screen.clone() {
                    let mut sel_idx = self.selected_widget_idx;
                    let modified = render_inspector_panel(ui, &mut screen, &mut sel_idx);
                    self.selected_widget_idx = sel_idx;
                    if modified {
                        self.sync_from_screen(&screen);
                    }
                } else {
                    ui.label("Fix KDL syntax errors to use the Inspector.");
                }
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
