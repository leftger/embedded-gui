//! Main Studio Application state, UI lifecycle, and canvas interaction.

use eframe::egui::{
    self, Color32, ColorImage, CornerRadius, FontId, Key, Pos2, Rect, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Vec2,
};
use embedded_graphics_core::pixelcolor::RgbColor;
use embedded_gui::motion::timing::{EasingCurve, evaluate_easing};
use embedded_gui_codegen::{
    GridPlacementDef, GridTrackDef, ScreenDef, WidgetDef, generate_rust_code, parse_kdl_screen,
    serialize_kdl_screen,
};

use crate::curve_visualizer::render_curve_graph;
use crate::inspector::render_inspector_panel;
use crate::presets::*;

/// How long a tapped widget keeps its accent ring after activation. Long enough
/// to register a brief board tap, short enough not to linger.
const PRESS_FLASH_SECS: f32 = 0.25;
use crate::types::{
    ActiveDrag, DisplayTheme, HardwareProfile, ScreenTransition, StudioMode, StudioTab,
    TransitionStyle,
};

pub struct EmbeddedGuiStudio {
    pub kdl_source: String,
    pub parsed_screen: Result<ScreenDef, String>,
    pub generated_rust: String,
    pub active_tab: StudioTab,
    pub mode: StudioMode,
    pub preview_zoom: f32,
    /// Texture containing the same RGB565 framebuffer submitted to USB.
    pub preview_texture: Option<TextureHandle>,
    /// Set when the screen size differs from the attached panel, as
    /// `(screen_w, screen_h, panel_w, panel_h)`.
    pub device_size_warning: Option<(u32, u32, u16, u16)>,
    /// Tracks the handshake edge so the first fitted frame is sent once the
    /// agent reports its panel size.
    pub device_handshake_seen: bool,
    pub copied_toast_timer: f32,
    pub action_toast: Option<(String, f32)>,

    // Multi-Screen Project Management & Live Transitions
    pub project_screens: Vec<(String, String)>,
    pub active_screen_idx: usize,
    pub transition_state: Option<ScreenTransition>,
    /// Root directory of the open on-disk project, if any.
    pub project_root: Option<std::path::PathBuf>,
    pub project_name: String,
    /// Relative screen paths from `project.kdl`, parallel to `project_screens`.
    pub project_screen_files: Vec<String>,

    // Hardware Bridge
    pub hardware_bridge: crate::bridge::HardwareBridge,

    // USB display agent link (RGB565 streaming over native USB bulk)
    pub device_link: Option<crate::device_link::DeviceLink>,
    pub device_ports: Vec<String>,
    pub selected_port: Option<String>,
    pub live_stream: bool,

    // Theme & Hardware
    pub display_theme: DisplayTheme,
    pub hardware_profile: HardwareProfile,

    // Selection & Inspector
    pub selected_widget_idx: Option<usize>,
    pub active_drag: ActiveDrag,
    pub pressed_widget: Option<usize>,

    // On-glass touch reported by the display agent, in panel framebuffer
    // coordinates. `None` when no finger is down. Drives Live Interactive
    // alongside the mouse.
    pub board_touch: Option<(u16, u16)>,
    /// True for the single frame a board touch transitions to pressed, so
    /// edge-triggered widgets (buttons, toggles) fire exactly once per tap.
    pub board_touch_pressed_edge: bool,
    board_touch_was_pressed: bool,

    /// Widget index flashing with transient press feedback, plus seconds left.
    /// Set when a widget is activated (mouse or board) so a tap is visible on
    /// the canvas and the streamed panel even after the finger lifts.
    pub interaction_flash: Option<(usize, f32)>,

    // Animation playback state
    pub is_playing: bool,
    pub timeline_time: f32,
    pub playback_speed: f32,
    pub loop_duration: f32,
    pub selected_easing: EasingCurve,
    /// Accumulates UI time so USB animation submission is capped independently
    /// of the monitor/egui repaint rate.
    pub animation_stream_accumulator: f32,

    // Simulated Signals & Reactive State
    pub mock_playground: crate::playground::MockPlaygroundState,
    pub preview_visual_state: Option<embedded_gui::style::VisualState>,

    // UI/UX Enhancements: Command Palette, Rulers, Layers
    pub command_palette_open: bool,
    pub command_query: String,
    pub show_rulers: bool,
    pub cursor_screen_coords: Option<(i32, i32)>,

    // Undo / Redo history
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
}

impl Default for EmbeddedGuiStudio {
    fn default() -> Self {
        let mut app = Self::new_offline();
        app.autoconnect_display();
        app
    }
}

impl EmbeddedGuiStudio {
    /// Builds the studio without touching USB, so tests can exercise the
    /// screen/target logic without a board attached.
    pub fn new_offline() -> Self {
        let project_screens = vec![
            (
                "AutoCluster".to_string(),
                SAMPLE_AUTOMOTIVE_CLUSTER.to_string(),
            ),
            ("HvacClimate".to_string(), SAMPLE_HVAC_CLIMATE.to_string()),
            (
                "PatientMonitor".to_string(),
                SAMPLE_PATIENT_MONITOR.to_string(),
            ),
            (
                "CncController".to_string(),
                SAMPLE_CNC_CONTROLLER.to_string(),
            ),
            (
                "FitnessTracker".to_string(),
                SAMPLE_SMARTWATCH_FITNESS.to_string(),
            ),
        ];
        let initial_kdl = project_screens[0].1.clone();

        let mut app = Self {
            kdl_source: initial_kdl,
            parsed_screen: Err("Not parsed".to_string()),
            generated_rust: String::new(),
            active_tab: StudioTab::VisualPreview,
            mode: StudioMode::Design,
            preview_zoom: 1.5,
            preview_texture: None,
            device_size_warning: None,
            device_handshake_seen: false,
            copied_toast_timer: 0.0,
            action_toast: None,
            project_screens,
            active_screen_idx: 0,
            transition_state: None,
            project_root: None,
            project_name: "Untitled".to_string(),
            project_screen_files: Vec::new(),
            hardware_bridge: crate::bridge::HardwareBridge::new(9080),
            device_link: None,
            device_ports: Vec::new(),
            selected_port: None,
            live_stream: true,
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
            board_touch: None,
            board_touch_pressed_edge: false,
            board_touch_was_pressed: false,
            interaction_flash: None,
            animation_stream_accumulator: 0.0,
            mock_playground: crate::playground::MockPlaygroundState::default(),
            preview_visual_state: None,
            command_palette_open: false,
            command_query: String::new(),
            show_rulers: true,
            cursor_screen_coords: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };
        app.reparse();
        app.select_target_for_screen();
        app
    }

    pub fn switch_to_screen(&mut self, idx: usize) {
        if idx < self.project_screens.len() {
            if self.active_screen_idx < self.project_screens.len() {
                self.project_screens[self.active_screen_idx].1 = self.kdl_source.clone();
            }
            self.active_screen_idx = idx;
            let source = self.project_screens[idx].1.clone();
            self.load_kdl_source(source);
        }
    }
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

    pub fn insert_widget_snippet(&mut self, snippet: &str) {
        self.push_undo_snapshot();
        if let Some(last_brace_idx) = self.kdl_source.rfind('}') {
            let mut new_kdl = self.kdl_source[..last_brace_idx].to_string();
            new_kdl.push_str(snippet);
            new_kdl.push_str(&self.kdl_source[last_brace_idx..]);
            self.load_kdl_source(new_kdl);
        } else {
            let mut new_kdl = self.kdl_source.clone();
            new_kdl.push_str(snippet);
            self.load_kdl_source(new_kdl);
        }
    }

    pub fn align_selected_widget(&mut self, col: usize, row: Option<usize>) {
        if let Ok(mut screen) = self.parsed_screen.clone() {
            if let Some(sel_idx) = self.selected_widget_idx {
                if let Some((p, _)) = screen.grid.children.get_mut(sel_idx) {
                    p.col = col;
                    if let Some(r) = row {
                        p.row = r;
                    }
                    self.sync_from_screen(&screen);
                }
            }
        }
    }

    pub fn align_selected_widget_row(&mut self, row: usize) {
        if let Ok(mut screen) = self.parsed_screen.clone() {
            if let Some(sel_idx) = self.selected_widget_idx {
                if let Some((p, _)) = screen.grid.children.get_mut(sel_idx) {
                    p.row = row;
                    self.sync_from_screen(&screen);
                }
            }
        }
    }

    /// Renders the current screen to RGB565 and streams it to the connected
    /// display agent (dirty tiles only after the first frame). Disconnects on
    /// a transport error.
    pub fn push_live_frame(&mut self) {
        self.push_live_frame_inner(false);
    }

    /// Renders and streams the current screen, repainting every tile even where
    /// pixels are unchanged. Backs the manual **Push Frame** action.
    pub fn push_live_frame_full(&mut self) {
        self.push_live_frame_inner(true);
    }

    fn push_live_frame_inner(&mut self, force_full: bool) {
        let screen = match &self.parsed_screen {
            Ok(s) => s.clone(),
            Err(_) => return,
        };
        let Some(link) = self.device_link.as_ref() else {
            return;
        };
        // Rendering is host-side work; transmission happens on the link thread,
        // so a stalled device never blocks the UI.
        let mut frame = crate::live_render::render_screen_at_with_assets(
            &screen,
            self.display_theme,
            self.animation_phase(),
            self.active_highlight(),
            self.project_root.as_deref(),
        );
        // The agent advertises its panel size during the handshake. Fitting here
        // keeps an oversized screen centered instead of losing its right and
        // bottom edges to rectangles the panel cannot address.
        if let Some((fb_w, fb_h)) = link.framebuffer_size() {
            if (fb_w, fb_h) != (frame.width, frame.height) {
                let bg = crate::live_render::RenderedFrame::background_for(self.display_theme);
                frame = frame.fit_to(fb_w, fb_h, bg);
                self.device_size_warning = Some((screen.width, screen.height, fb_w, fb_h));
            } else {
                self.device_size_warning = None;
            }
        }
        if force_full {
            link.submit_full(frame);
        } else {
            link.submit(frame);
        }

        if let Some(err) = link.take_error() {
            self.device_link = None;
            self.action_toast = Some((format!("USB stream error: {err}"), 3.0));
        }
    }

    pub fn recompile(&mut self) {
        let previous_size = self
            .parsed_screen
            .as_ref()
            .ok()
            .map(|screen| (screen.width, screen.height));
        self.reparse();
        let current_size = self
            .parsed_screen
            .as_ref()
            .ok()
            .map(|screen| (screen.width, screen.height));
        if previous_size.is_some() && current_size != previous_size {
            self.hardware_profile = HardwareProfile::Custom;
        }
        self.stream_if_live();
    }

    fn reparse(&mut self) {
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
        self.stream_if_live();
    }

    /// Replaces the editor contents with a newly loaded screen. The KDL canvas
    /// is authoritative: exact known dimensions select their matching target,
    /// while non-standard dimensions select Custom.
    pub fn load_kdl_source(&mut self, source: String) {
        self.push_undo_snapshot();
        self.kdl_source = source;
        self.selected_widget_idx = None;
        self.reparse();
        self.select_target_for_screen();
        self.stream_if_live();
    }

    /// Replaces the in-memory tab set with an on-disk KDL project.
    pub fn load_project(&mut self, project: crate::project::GuiProject) {
        self.push_undo_snapshot();
        self.project_root = Some(project.root);
        self.project_name = project.name;
        self.project_screen_files = project.screen_files;
        self.project_screens = project.screens;
        self.active_screen_idx = 0;
        self.selected_widget_idx = None;
        if let Some(theme) = project.theme {
            self.display_theme = theme;
        }
        self.hardware_profile = project.hardware_profile;
        let source = self.project_screens[0].1.clone();
        self.kdl_source = source;
        self.reparse();
        self.select_target_for_screen();
        self.stream_if_live();
    }

    /// Flushes the active editor into `project_screens` and writes the project.
    pub fn save_project_to_disk(&mut self) -> Result<std::path::PathBuf, String> {
        if self.active_screen_idx < self.project_screens.len() {
            self.project_screens[self.active_screen_idx].1 = self.kdl_source.clone();
        }
        let files = if self.project_screen_files.len() == self.project_screens.len() {
            Some(self.project_screen_files.as_slice())
        } else {
            None
        };
        if let Some(root) = self.project_root.clone() {
            crate::project::save_project_to(
                &root,
                &self.project_name,
                self.hardware_profile,
                self.display_theme,
                &self.project_screens,
                files,
            )
            .map_err(|e| e.0)
        } else {
            crate::project::save_project_dialog(
                &self.project_name,
                self.hardware_profile,
                self.display_theme,
                &self.project_screens,
            )
            .ok_or_else(|| "Save cancelled".to_string())?
            .map_err(|e| e.0)
            .inspect(|path| {
                self.project_root = path.parent().map(|p| p.to_path_buf());
            })
        }
    }

    /// Copies a file into the open project under `assets/<subdir>`, returning
    /// the copy's path and the project-relative path KDL should reference.
    ///
    /// Existing names are never overwritten: a `-2`, `-3`, ... suffix is added
    /// so re-importing a revised asset can't silently replace one still in use
    /// by another screen.
    fn copy_into_assets(
        &self,
        source: &std::path::Path,
        subdir: &str,
    ) -> Result<(std::path::PathBuf, String), String> {
        let root = self
            .project_root
            .clone()
            .ok_or_else(|| "Save or open a project before importing assets".to_string())?;
        let file_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "The selected file has an invalid filename".to_string())?;
        let assets_dir = if subdir.is_empty() {
            root.join("assets")
        } else {
            root.join("assets").join(subdir)
        };
        std::fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Could not create {}: {e}", assets_dir.display()))?;

        let mut destination = assets_dir.join(file_name);
        if destination.exists() && destination != source {
            let stem = source
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("asset");
            let extension = source
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("bin");
            for suffix in 2.. {
                let candidate = assets_dir.join(format!("{stem}-{suffix}.{extension}"));
                if !candidate.exists() {
                    destination = candidate;
                    break;
                }
            }
        }
        if destination != source {
            std::fs::copy(source, &destination).map_err(|e| {
                format!(
                    "Could not copy {} to {}: {e}",
                    source.display(),
                    destination.display()
                )
            })?;
        }

        let name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(file_name);
        let relative = if subdir.is_empty() {
            format!("assets/{name}")
        } else {
            format!("assets/{subdir}/{name}")
        };
        Ok((destination, relative))
    }

    /// Copies an image into the open project's `assets/` directory and inserts
    /// an image node behind the current screen's other widgets.
    fn import_image_asset(&mut self) -> Result<std::path::PathBuf, String> {
        let source = rfd::FileDialog::new()
            .set_title("Import PNG, JPEG, or BMP")
            .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
            .pick_file()
            .ok_or_else(|| "Import cancelled".to_string())?;
        let (destination, relative) = self.copy_into_assets(&source, "")?;

        let mut screen = self.parsed_screen.as_ref().map_err(Clone::clone)?.clone();
        screen.grid.children.insert(
            0,
            (
                GridPlacementDef::default(),
                WidgetDef::Image {
                    id: Some("image".into()),
                    source: relative,
                    fit: "center".into(),
                    mode: "color".into(),
                    tint: None,
                },
            ),
        );
        self.selected_widget_idx = Some(0);
        self.sync_from_screen(&screen);
        Ok(destination)
    }

    /// Imports a BDF font and declares it on the active screen, ready for
    /// `font="…"` on labels and carousels.
    fn import_font_asset(&mut self) -> Result<std::path::PathBuf, String> {
        let source = rfd::FileDialog::new()
            .set_title("Import BDF bitmap font")
            .add_filter("BDF font", &["bdf"])
            .pick_file()
            .ok_or_else(|| "Import cancelled".to_string())?;

        // Fail before copying if the font can't be parsed, so a broken import
        // doesn't leave a stray file in the project.
        let text = std::fs::read_to_string(&source)
            .map_err(|e| format!("Could not read {}: {e}", source.display()))?;
        embedded_gui_codegen::assets::parse_bdf(&text, None)
            .map_err(|e| format!("{}: {e}", source.display()))?;

        let (destination, relative) = self.copy_into_assets(&source, "fonts")?;
        let name = destination
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("font")
            .to_string();

        let mut screen = self.parsed_screen.as_ref().map_err(Clone::clone)?.clone();
        if let Some(existing) = screen.fonts.iter_mut().find(|font| font.name == name) {
            existing.source = relative;
        } else {
            screen.fonts.push(embedded_gui_codegen::FontAssetDef {
                name,
                source: relative,
                chars: String::new(),
            });
        }
        self.sync_from_screen(&screen);
        Ok(destination)
    }

    /// Imports 1-bit icon art. With a composite icon selected the art becomes
    /// another layer of it; otherwise a new icon widget is created.
    fn import_icon_asset(&mut self) -> Result<std::path::PathBuf, String> {
        let source = rfd::FileDialog::new()
            .set_title("Import icon art (PNG, BMP)")
            .add_filter("Image", &["png", "bmp", "jpg", "jpeg"])
            .pick_file()
            .ok_or_else(|| "Import cancelled".to_string())?;
        let (destination, relative) = self.copy_into_assets(&source, "icons")?;

        let mut screen = self.parsed_screen.as_ref().map_err(Clone::clone)?.clone();
        let part = embedded_gui_codegen::IconPartDef {
            source: relative,
            dx: 0,
            dy: 0,
            visible: true,
            tint: None,
        };
        let selected = self
            .selected_widget_idx
            .filter(|idx| *idx < screen.grid.children.len());
        match selected.map(|idx| &mut screen.grid.children[idx].1) {
            Some(WidgetDef::CompositeIcon { parts, .. }) => parts.push(part),
            _ => {
                screen.grid.children.insert(
                    0,
                    (
                        GridPlacementDef::default(),
                        WidgetDef::CompositeIcon {
                            id: Some("icon".into()),
                            parts: vec![part],
                            scale: 1,
                            align: "center".into(),
                            tint: Some("accent".into()),
                            threshold: 128,
                            invert: false,
                        },
                    ),
                );
                self.selected_widget_idx = Some(0);
            }
        }
        self.sync_from_screen(&screen);
        Ok(destination)
    }

    /// Imports an OBJ or STL mesh and inserts a 3D node for it.
    fn import_mesh_asset(&mut self) -> Result<std::path::PathBuf, String> {
        let source = rfd::FileDialog::new()
            .set_title("Import 3D mesh")
            .add_filter("OBJ or STL mesh", &["obj", "stl"])
            .pick_file()
            .ok_or_else(|| "Import cancelled".to_string())?;

        let bytes = std::fs::read(&source)
            .map_err(|e| format!("Could not read {}: {e}", source.display()))?;
        let name = source.file_name().unwrap_or_default().to_string_lossy();
        embedded_gui_codegen::assets::parse_mesh(&name, &bytes)
            .map_err(|e| format!("{}: {e}", source.display()))?;

        let (destination, relative) = self.copy_into_assets(&source, "meshes")?;
        let mut screen = self.parsed_screen.as_ref().map_err(Clone::clone)?.clone();
        screen.grid.children.insert(
            0,
            (
                GridPlacementDef::default(),
                WidgetDef::Mesh3d {
                    id: Some("mesh".into()),
                    source: relative,
                    shading: "lit".into(),
                    color: Some("accent".into()),
                    scale: 1.0,
                    roll: 0.0,
                    pitch: 0.0,
                    yaw: 0.0,
                    camera_distance: 4.0,
                    fov: 1.5707964,
                },
            ),
        );
        self.selected_widget_idx = Some(0);
        self.sync_from_screen(&screen);
        Ok(destination)
    }

    /// Scans for display agents at startup and attaches to one if it is
    /// unambiguous.
    ///
    /// Connecting only reaches the handshake; the panel size arrives later, so
    /// the target is adopted in [`EmbeddedGuiStudio::adopt_detected_panel`].
    /// With several agents present the choice is left to the user rather than
    /// guessing which board they meant.
    pub fn autoconnect_display(&mut self) {
        self.device_ports = crate::device_link::list_devices();
        let [only_device] = self.device_ports.as_slice() else {
            return;
        };
        let device_id = only_device.clone();
        self.selected_port = Some(device_id.clone());
        match crate::device_link::DeviceLink::connect(&device_id) {
            Ok(link) => {
                self.device_link = Some(link);
                self.action_toast = Some((format!("Found display {device_id}"), 2.5));
            }
            Err(e) => self.action_toast = Some((e, 3.0)),
        }
    }

    /// Records the attached panel as the active target without changing the
    /// authored KDL canvas. Streaming fits the frame to the panel separately.
    pub fn adopt_detected_panel(&mut self) {
        let Some((fb_w, fb_h)) = self
            .device_link
            .as_ref()
            .and_then(|link| link.framebuffer_size())
        else {
            return;
        };
        let detected = HardwareProfile::Detected {
            width: fb_w as u32,
            height: fb_h as u32,
        };
        if self.hardware_profile == detected {
            return;
        }
        self.hardware_profile = detected;
        self.action_toast = Some((format!("Target set to panel {fb_w}x{fb_h}"), 2.5));
    }

    /// Selects the profile matching the authored screen dimensions, or Custom
    /// when the dimensions do not correspond to a canned target.
    fn select_target_for_screen(&mut self) {
        if let Ok(screen) = &self.parsed_screen {
            self.hardware_profile = HardwareProfile::from_dimensions(screen.width, screen.height);
        }
    }

    /// Explicitly resizes the active screen after the user selects a target.
    pub fn apply_hardware_profile(&mut self) -> bool {
        let Some((w, h)) = self.hardware_profile.dimensions() else {
            return false;
        };
        let Ok(screen) = &self.parsed_screen else {
            return false;
        };
        if screen.width == w && screen.height == h {
            return false;
        }
        let mut resized = screen.clone();
        resized.width = w;
        resized.height = h;
        self.sync_from_screen(&resized);
        true
    }

    /// Streams the current screen whenever Live is enabled and a board is
    /// attached. Every mutation path funnels through here so canvas drags,
    /// inspector edits, and screen switches reach the panel without a manual
    /// **Push Frame**.
    pub fn stream_if_live(&mut self) {
        if self.live_stream && self.device_link.is_some() {
            self.push_live_frame();
        }
    }

    /// Pulls any touch samples the agent reported and folds them into the
    /// current board-touch state. The firmware only emits on press, move, or
    /// release, so a held-still finger produces no samples and the last known
    /// position persists.
    pub fn drain_board_touches(&mut self) {
        self.board_touch_pressed_edge = false;
        let Some(link) = self.device_link.as_ref() else {
            self.board_touch = None;
            self.board_touch_was_pressed = false;
            return;
        };
        let samples = link.take_touches();
        let Some(last) = samples.last().copied() else {
            return;
        };
        if last.pressed {
            self.board_touch = Some((last.x, last.y));
            if !self.board_touch_was_pressed {
                self.board_touch_pressed_edge = true;
            }
            self.board_touch_was_pressed = true;
        } else {
            self.board_touch = None;
            self.board_touch_was_pressed = false;
        }
    }

    /// Maps the current board touch (panel framebuffer space) into a canvas
    /// position inside `display_rect`. Assumes the active screen matches the
    /// panel size, which the auto-detect path enforces; otherwise the mapping
    /// scales proportionally.
    fn board_canvas_pos(&self, display_rect: Rect, screen: &ScreenDef) -> Option<Pos2> {
        let (px, py) = self.board_touch?;
        let sx = display_rect.width() / screen.width.max(1) as f32;
        let sy = display_rect.height() / screen.height.max(1) as f32;
        Some(Pos2::new(
            display_rect.min.x + px as f32 * sx,
            display_rect.min.y + py as f32 * sy,
        ))
    }

    /// Widget index that should render with transient press feedback: whatever
    /// is held right now, or the last-tapped widget while its flash decays.
    fn active_highlight(&self) -> Option<usize> {
        if self.mode != StudioMode::Interactive {
            return None;
        }
        self.pressed_widget
            .or(self.interaction_flash.map(|(idx, _)| idx))
    }

    fn animation_phase(&self) -> f32 {
        if self.loop_duration <= f32::EPSILON {
            0.0
        } else {
            (self.timeline_time / self.loop_duration).rem_euclid(1.0)
        }
    }

    /// Inserts a new widget into the active screen layout and synchronizes KDL source.
    pub fn insert_widget(&mut self, widget: WidgetDef) {
        if let Ok(mut screen) = self.parsed_screen.clone() {
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
                animation: None,
            };
            screen.grid.children.push((placement, widget));
            self.selected_widget_idx = Some(screen.grid.children.len() - 1);
            self.sync_from_screen(&screen);
            self.action_toast = Some(("✓ Widget inserted".to_string(), 2.0));
        }
    }

    /// Inserts a named vector asset as an SVG path widget onto the canvas.
    pub fn insert_vector_asset(&mut self, name: &str, d: &str) {
        let verbs = embedded_gui_codegen::parse_svg_path_d(d);
        self.insert_widget(WidgetDef::VectorPath {
            id: Some(name.to_lowercase()),
            stroke_width: 1,
            verbs,
        });
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
            let prev_theme = self.display_theme;
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
            if self.display_theme != prev_theme && self.live_stream {
                self.push_live_frame();
            }

            ui.separator();

            // Hardware Target Profile
            ui.label("Target:");
            let prev_profile = self.hardware_profile;
            egui::ComboBox::from_id_salt("hardware_profile_selector")
                .selected_text(self.hardware_profile.name())
                .show_ui(ui, |ui| {
                    // Only offered once an agent has reported its panel size.
                    if let HardwareProfile::Detected { width, height } = self.hardware_profile {
                        let detected = HardwareProfile::Detected { width, height };
                        ui.selectable_value(&mut self.hardware_profile, detected, detected.name());
                    }
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
                    ui.selectable_value(
                        &mut self.hardware_profile,
                        HardwareProfile::Ssd1357,
                        HardwareProfile::Ssd1357.name(),
                    );
                });

            if self.hardware_profile != prev_profile {
                self.apply_hardware_profile();
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

            ui.separator();

            // Visual State Preview Selector
            ui.label("State:");
            egui::ComboBox::from_id_salt("state_preview_combo")
                .selected_text(match self.preview_visual_state {
                    None => "Auto State",
                    Some(embedded_gui::style::VisualState::Normal) => "Normal",
                    Some(embedded_gui::style::VisualState::Pressed) => "Pressed",
                    Some(embedded_gui::style::VisualState::Focused) => "Focused",
                    Some(embedded_gui::style::VisualState::Disabled) => "Disabled",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.preview_visual_state, None, "Auto State");
                    ui.selectable_value(
                        &mut self.preview_visual_state,
                        Some(embedded_gui::style::VisualState::Normal),
                        "Normal",
                    );
                    ui.selectable_value(
                        &mut self.preview_visual_state,
                        Some(embedded_gui::style::VisualState::Pressed),
                        "Pressed",
                    );
                    ui.selectable_value(
                        &mut self.preview_visual_state,
                        Some(embedded_gui::style::VisualState::Focused),
                        "Focused",
                    );
                    ui.selectable_value(
                        &mut self.preview_visual_state,
                        Some(embedded_gui::style::VisualState::Disabled),
                        "Disabled",
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
        // 📱 Persistent Multi-Screen Tabs Bar
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📱 Screens:").strong());
            let mut to_switch = None;
            let mut to_remove = None;
            for (i, (name, _)) in self.project_screens.iter().enumerate() {
                let is_active = i == self.active_screen_idx;
                let tab_text = match name.as_str() {
                    "AutoCluster" => "🚗 AutoCluster",
                    "HvacClimate" => "❄️ HvacClimate",
                    "PatientMonitor" => "🩺 PatientMonitor",
                    "CncController" => "⚙️ CncController",
                    "FitnessTracker" => "⌚ FitnessTracker",
                    _ => name.as_str(),
                };
                if ui.selectable_label(is_active, tab_text).clicked() {
                    to_switch = Some(i);
                }
                if self.project_screens.len() > 1 && is_active && ui.small_button("✕").clicked() {
                    to_remove = Some(i);
                }
            }
            if let Some(i) = to_switch {
                self.switch_to_screen(i);
            }
            if let Some(i) = to_remove {
                self.project_screens.remove(i);
                self.active_screen_idx = self
                    .active_screen_idx
                    .min(self.project_screens.len().saturating_sub(1));
                let restored = self.project_screens[self.active_screen_idx].1.clone();
                self.load_kdl_source(restored);
            }
            if ui.button("➕ New Screen").clicked() {
                let count = self.project_screens.len() + 1;
                let new_name = format!("Screen{}", count);
                let new_kdl = format!(
                    "screen id=\"{}\" width=320 height=240 {{\n    grid cols=\"1fr\" rows=\"1fr\" gap=8 padding=8 {{\n        label text=\"Hello {}\"\n    }}\n}}\n",
                    new_name, new_name
                );
                self.project_screens.push((new_name, new_kdl));
                self.switch_to_screen(self.project_screens.len() - 1);
            }
        });
        ui.separator();

        // 🧭 Breadcrumb Navigation Bar
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🧭").size(11.0));
            if ui.small_button(format!("📱 {}", screen.id)).clicked() {
                self.selected_widget_idx = None;
            }
            ui.label(egui::RichText::new("❯").weak());
            ui.label(
                egui::RichText::new(format!(
                    "Grid ({}×{})",
                    screen.grid.cols.len(),
                    screen.grid.rows.len()
                ))
                .weak(),
            );
            if let Some(sel_idx) = self.selected_widget_idx {
                if let Some((placement, widget)) = screen.grid.children.get(sel_idx) {
                    ui.label(egui::RichText::new("❯").weak());
                    ui.colored_label(
                        Color32::from_rgb(80, 200, 255),
                        format!(
                            "{} (c:{}, r:{})",
                            widget.id().unwrap_or("widget"),
                            placement.col,
                            placement.row
                        ),
                    );
                }
            }
        });
        ui.separator();

        // 🧰 Quick-Insert Widget Palette
        if self.mode == StudioMode::Design {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("🧰 Insert:").strong());
                let mut snippet_to_insert = None;
                if ui.button("🔘 Button").clicked() {
                    snippet_to_insert = Some("        button text=\"NEW BTN\"\n");
                }
                if ui.button("☑ Toggle").clicked() {
                    snippet_to_insert = Some("        toggle checked=true\n");
                }
                if ui.button("🎚 Slider").clicked() {
                    snippet_to_insert = Some("        slider min=0 max=100 value=50\n");
                }
                if ui.button("🔢 Spinbox").clicked() {
                    snippet_to_insert = Some("        spinbox min=0 max=100 value=25\n");
                }
                if ui.button("🧭 Radial Scale").clicked() {
                    snippet_to_insert =
                        Some("        scale mode=\"radial\" min=0 max=100 value=75\n");
                }
                if ui.button("📊 Progress").clicked() {
                    snippet_to_insert = Some("        progress value=60\n");
                }
                if ui.button("🎠 Carousel").clicked() {
                    snippet_to_insert = Some("        carousel count=5 selected=0\n");
                }
                if ui.button("🏷 Banner").clicked() {
                    snippet_to_insert = Some("        banner text=\"HEADER TITLE\"\n");
                }

                if let Some(snippet) = snippet_to_insert {
                    self.insert_widget_snippet(snippet);
                }
            });
            ui.separator();

            // 📐 Alignment & Distribution Toolbar
            if let Some(_sel_idx) = self.selected_widget_idx {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📐 Align Selected:").strong());
                    if ui.button("◀ Left").clicked() {
                        self.align_selected_widget(0, None);
                    }
                    if ui.button("⏺ Center").clicked() {
                        let center_col = screen.grid.cols.len().saturating_sub(1) / 2;
                        self.align_selected_widget(center_col, None);
                    }
                    if ui.button("▶ Right").clicked() {
                        let right_col = screen.grid.cols.len().saturating_sub(1);
                        self.align_selected_widget(right_col, None);
                    }
                    if ui.button("▲ Top").clicked() {
                        self.align_selected_widget_row(0);
                    }
                    if ui.button("▼ Bottom").clicked() {
                        let bottom_row = screen.grid.rows.len().saturating_sub(1);
                        self.align_selected_widget_row(bottom_row);
                    }
                });
                ui.separator();
            }
        }

        let screen_w = screen.width as f32 * self.preview_zoom;
        let screen_h = screen.height as f32 * self.preview_zoom;

        // Render through embedded-gui itself. This is the same RGB565 path used
        // for USB, so the canvas reflects the actual firmware-facing pixels.
        let preview_frame = crate::live_render::render_screen_at_with_assets(
            screen,
            self.display_theme,
            self.animation_phase(),
            self.active_highlight(),
            self.project_root.as_deref(),
        );
        let mut preview_rgb = Vec::with_capacity(preview_frame.pixels.len() * 3);
        for pixel in &preview_frame.pixels {
            preview_rgb.extend_from_slice(&[
                (u16::from(pixel.r()) * 255 / 31) as u8,
                (u16::from(pixel.g()) * 255 / 63) as u8,
                (u16::from(pixel.b()) * 255 / 31) as u8,
            ]);
        }
        let preview_image = ColorImage::from_rgb(
            [preview_frame.width as usize, preview_frame.height as usize],
            &preview_rgb,
        );
        match &mut self.preview_texture {
            Some(texture) => texture.set(preview_image, TextureOptions::NEAREST),
            None => {
                self.preview_texture = Some(ui.ctx().load_texture(
                    "embedded-gui-rgb565-preview",
                    preview_image,
                    TextureOptions::NEAREST,
                ));
            }
        }
        let preview_texture_id = self.preview_texture.as_ref().map(TextureHandle::id);

        egui::ScrollArea::both().show(ui, |ui| {
            let margin_left = 42.0;
            let margin_top = 32.0;
            let margin_right = 42.0;
            let margin_bottom = 40.0;

            let (response, painter) = ui.allocate_painter(
                Vec2::new(
                    screen_w + margin_left + margin_right,
                    screen_h + margin_top + margin_bottom,
                ),
                egui::Sense::click_and_drag(),
            );
            let mut canvas_offset = Vec2::ZERO;
            let mut transition_scale = 1.0;
            let mut transition_alpha = 255;
            if let Some(trans) = &self.transition_state {
                let progress = trans.visual_progress();
                match trans.style {
                    TransitionStyle::SlideLeft => {
                        canvas_offset.x = -screen_w * progress;
                    }
                    TransitionStyle::SlideRight => {
                        canvas_offset.x = screen_w * progress;
                    }
                    TransitionStyle::SlideUp => {
                        canvas_offset.y = -screen_h * progress;
                    }
                    TransitionStyle::SlideDown => {
                        canvas_offset.y = screen_h * progress;
                    }
                    TransitionStyle::ZoomPush => {
                        transition_scale = (1.0 - progress * 0.25).max(0.1);
                        let shrink = screen_w * (1.0 - transition_scale) / 2.0;
                        canvas_offset += Vec2::new(shrink, shrink);
                    }
                    TransitionStyle::Fade | TransitionStyle::Dissolve => {
                        transition_alpha = ((1.0 - progress) * 255.0) as u8;
                    }
                    _ => {}
                }
            }
            let origin = response.rect.min + Vec2::new(margin_left, margin_top) + canvas_offset;
            let display_rect = Rect::from_min_size(
                origin,
                Vec2::new(screen_w * transition_scale, screen_h * transition_scale),
            );

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

            // Display the exact RGB565 framebuffer sent to the board. Nearest
            // filtering preserves the embedded display's pixel boundaries.
            if let Some(texture_id) = preview_texture_id {
                painter.image(
                    texture_id,
                    display_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::from_white_alpha(transition_alpha),
                );
            }

            // Compute grid layout pixel bounds
            let pad = (screen.grid.padding as f32) * self.preview_zoom;
            let gap = (screen.grid.gap as f32) * self.preview_zoom;
            let inner_rect = display_rect.shrink(pad);

            let cols = &screen.grid.cols;
            let rows = &screen.grid.rows;

            // Scale the renderer's own track math into canvas space. Recomputing
            // it against the zoomed viewport would misplace `px` and `auto`
            // tracks, whose sizes are absolute and do not scale with zoom.
            let geometry = crate::live_render::grid_geometry(screen);
            let zoom = self.preview_zoom;
            let col_widths: Vec<f32> = geometry.col_sizes.iter().map(|w| w * zoom).collect();
            let row_heights: Vec<f32> = geometry.row_sizes.iter().map(|h| h * zoom).collect();
            let col_xs: Vec<f32> = geometry
                .col_starts
                .iter()
                .map(|x| display_rect.min.x + x * zoom)
                .collect();
            let row_ys: Vec<f32> = geometry
                .row_starts
                .iter()
                .map(|y| display_rect.min.y + y * zoom)
                .collect();

            let mouse_pos = ui.input(|i| i.pointer.interact_pos());
            let mouse_down = ui.input(|i| i.pointer.primary_down());
            let mouse_pressed = ui.input(|i| i.pointer.primary_pressed());

            // In Live Interactive, on-glass touches drive the same hit-testing
            // as the mouse: the board acts as a second pointer. Design mode
            // ignores the board so editing is never disturbed by a stray touch.
            let board_pos = if self.mode == StudioMode::Interactive {
                self.board_canvas_pos(display_rect, screen)
            } else {
                None
            };
            let pointer_pos = board_pos.or(mouse_pos);
            let primary_down = mouse_down || board_pos.is_some();
            let primary_pressed =
                mouse_pressed || (board_pos.is_some() && self.board_touch_pressed_edge);

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
                                // Transient press feedback for any widget under
                                // the pointer, so mouse and board taps both
                                // flash the touched cell.
                                if primary_down {
                                    self.pressed_widget = Some(idx);
                                }
                                if primary_pressed {
                                    self.interaction_flash = Some((idx, PRESS_FLASH_SECS));
                                }
                                match w {
                                    embedded_gui_codegen::WidgetDef::Button {
                                        text,
                                        on_click,
                                        ..
                                    } => {
                                        if primary_pressed {
                                            self.pressed_widget = Some(idx);
                                            if let Some(act) = on_click {
                                                if act.starts_with("navigate:") {
                                                    let parts: Vec<&str> = act.split(':').collect();
                                                    if let Some(target_name) = parts.get(1) {
                                                        if let Some(target_idx) = self.project_screens.iter().position(|(n, _)| n == *target_name) {
                                                            let target_transition = self.project_screens
                                                                .get(target_idx)
                                                                .and_then(|(_, source)| parse_kdl_screen(source).ok())
                                                                .and_then(|screen| screen.transition);
                                                            let (trans_style, duration, easing) = if let Some(code) = parts.get(2) {
                                                                let style = TransitionStyle::from_code(code);
                                                                let seconds = if style == TransitionStyle::Fade {
                                                                    0.2
                                                                } else if style == TransitionStyle::Instant {
                                                                    0.001
                                                                } else {
                                                                    0.3
                                                                };
                                                                (style, seconds, "in_out_sine".to_string())
                                                            } else if let Some(spec) = target_transition {
                                                                (
                                                                    TransitionStyle::from_preset(&spec.preset),
                                                                    spec.duration_ms as f32 / 1000.0,
                                                                    spec.easing,
                                                                )
                                                            } else {
                                                                (TransitionStyle::SlideLeft, 0.3, "in_out_sine".to_string())
                                                            };
                                                            self.transition_state = Some(ScreenTransition {
                                                                target_screen_idx: target_idx,
                                                                progress: 0.0,
                                                                duration,
                                                                style: trans_style,
                                                                easing,
                                                            });
                                                            self.action_toast = Some((format!("🔀 Navigating to '{}' ({})", target_name, trans_style.name()), 2.0));
                                                        } else {
                                                            self.action_toast = Some((format!("🔘 Button '{}' -> Target '{}' not found", text, target_name), 2.0));
                                                        }
                                                    }
                                                } else {
                                                    self.action_toast = Some((format!("🔘 Button '{}' -> {}", text, act), 2.0));
                                                }
                                            } else {
                                                self.action_toast = Some((format!("🔘 Button '{}' pressed", text), 2.0));
                                            }
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

            // --- B. EDITOR SELECTION OVERLAYS ---
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
                    let badge_font = FontId::proportional(9.5);
                    let badge_galley = painter.layout_no_wrap(
                        badge_text,
                        badge_font,
                        Color32::WHITE,
                    );
                    let badge_w = (badge_galley.size().x + 12.0).max(100.0);
                    let badge_h = 16.0;
                    let badge_y = if widget_rect.min.y - 18.0 < display_rect.min.y {
                        widget_rect.min.y + 4.0
                    } else {
                        widget_rect.min.y - 18.0
                    };
                    let badge_pos = Pos2::new(
                        widget_rect.min.x.clamp(display_rect.min.x, (display_rect.max.x - badge_w).max(display_rect.min.x)),
                        badge_y,
                    );
                    let badge_rect = Rect::from_min_size(badge_pos, Vec2::new(badge_w, badge_h));
                    painter.rect_filled(
                        badge_rect,
                        CornerRadius::same(3),
                        Color32::from_rgb(30, 80, 180),
                    );
                    painter.galley(
                        Pos2::new(badge_pos.x + 6.0, badge_pos.y + 2.0),
                        badge_galley,
                        Color32::WHITE,
                    );
                }
            }

            // Show where the physical panel is being touched, so the operator
            // can correlate on-glass taps with the live canvas.
            if let Some(pos) = board_pos {
                painter.circle_stroke(
                    pos,
                    12.0,
                    Stroke::new(2.0_f32, Color32::from_rgb(80, 220, 255)),
                );
                painter.circle_filled(pos, 3.0, Color32::from_rgb(80, 220, 255));
            }

            // 📏 Canvas Pixel Rulers & Coordinates Crosshair HUD
            if self.show_rulers {
                // Top ruler ticks & coordinates
                let mut x_tick = 0.0;
                while x_tick <= screen_w {
                    let mark_x = display_rect.min.x + x_tick;
                    let is_major = (x_tick / self.preview_zoom).round() as i32 % 50 == 0;
                    let tick_len = if is_major { 8.0 } else { 4.0 };
                    painter.line_segment(
                        [
                            Pos2::new(mark_x, display_rect.min.y - tick_len),
                            Pos2::new(mark_x, display_rect.min.y),
                        ],
                        Stroke::new(1.0_f32, Color32::from_rgb(100, 110, 130)),
                    );
                    if is_major {
                        painter.text(
                            Pos2::new(mark_x, display_rect.min.y - 10.0),
                            egui::Align2::CENTER_BOTTOM,
                            format!("{:.0}", x_tick / self.preview_zoom),
                            FontId::proportional(8.5),
                            Color32::from_rgb(140, 155, 175),
                        );
                    }
                    x_tick += 10.0 * self.preview_zoom;
                }

                // Left ruler ticks & coordinates
                let mut y_tick = 0.0;
                while y_tick <= screen_h {
                    let mark_y = display_rect.min.y + y_tick;
                    let is_major = (y_tick / self.preview_zoom).round() as i32 % 50 == 0;
                    let tick_len = if is_major { 8.0 } else { 4.0 };
                    painter.line_segment(
                        [
                            Pos2::new(display_rect.min.x - tick_len, mark_y),
                            Pos2::new(display_rect.min.x, mark_y),
                        ],
                        Stroke::new(1.0_f32, Color32::from_rgb(100, 110, 130)),
                    );
                    if is_major {
                        painter.text(
                            Pos2::new(display_rect.min.x - 10.0, mark_y),
                            egui::Align2::RIGHT_CENTER,
                            format!("{:.0}", y_tick / self.preview_zoom),
                            FontId::proportional(8.5),
                            Color32::from_rgb(140, 155, 175),
                        );
                    }
                    y_tick += 10.0 * self.preview_zoom;
                }
            }

            // HUD Coordinate crosshair badge
            if let Some(pos) = pointer_pos {
                if display_rect.contains(pos) {
                    let px_x = ((pos.x - display_rect.min.x) / self.preview_zoom)
                        .clamp(0.0, screen.width.saturating_sub(1) as f32) as i32;
                    let px_y = ((pos.y - display_rect.min.y) / self.preview_zoom)
                        .clamp(0.0, screen.height.saturating_sub(1) as f32) as i32;
                    self.cursor_screen_coords = Some((px_x, px_y));

                    let hud_text = format!("📍 X:{} Y:{}", px_x, px_y);
                    let hud_font = FontId::monospace(9.5);
                    let text_galley = painter.layout_no_wrap(
                        hud_text,
                        hud_font,
                        Color32::from_rgb(120, 220, 160),
                    );
                    let badge_w = text_galley.size().x + 16.0;
                    let badge_h = 20.0;
                    let badge_x = (display_rect.max.x - badge_w).max(display_rect.min.x);
                    let badge_y = display_rect.max.y + 8.0;
                    let hud_rect = Rect::from_min_size(
                        Pos2::new(badge_x, badge_y),
                        Vec2::new(badge_w, badge_h),
                    );

                    painter.rect_filled(
                        hud_rect,
                        CornerRadius::same(4),
                        Color32::from_rgb(20, 24, 30),
                    );
                    painter.rect_stroke(
                        hud_rect,
                        CornerRadius::same(4),
                        Stroke::new(1.0_f32, Color32::from_rgb(50, 60, 75)),
                        StrokeKind::Inside,
                    );
                    painter.galley(
                        Pos2::new(hud_rect.min.x + 8.0, hud_rect.min.y + 4.0),
                        text_galley,
                        Color32::from_rgb(120, 220, 160),
                    );
                } else {
                    self.cursor_screen_coords = None;
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
        // The agent reports its panel size asynchronously, so the frame sent at
        // connect time cannot be fitted yet. Resend once the handshake lands.
        let handshaked = self
            .device_link
            .as_ref()
            .is_some_and(|link| link.is_handshaked());
        if handshaked && !self.device_handshake_seen {
            self.device_handshake_seen = true;
            self.adopt_detected_panel();
            self.push_live_frame_full();
        } else if !handshaked {
            self.device_handshake_seen = false;
        }

        // Fold in any on-glass touches the agent reported this frame so Live
        // Interactive reacts to the board, then keep repainting while a finger
        // is held (held-still touches send no new samples).
        self.drain_board_touches();
        if self.board_touch.is_some() {
            ctx.request_repaint();
        }

        // Handle Timers
        let dt = ctx.input(|i| i.stable_dt);
        self.mock_playground.tick(dt);
        if self.mock_playground.lfo_enabled {
            ctx.request_repaint();
        }
        if self.copied_toast_timer > 0.0 {
            self.copied_toast_timer -= dt;
        }
        if let Some((_, timer)) = &mut self.action_toast {
            *timer -= dt;
            if *timer <= 0.0 {
                self.action_toast = None;
            }
        }
        if let Some((_, timer)) = &mut self.interaction_flash {
            *timer -= dt;
            if *timer <= 0.0 {
                self.interaction_flash = None;
            }
            // Keep repainting and restreaming so the ring appears and then
            // clears on the board even without further input. The final tick
            // (now None) streams the frame that erases the ring.
            ctx.request_repaint();
            self.stream_if_live();
        }

        // Advance Screen Transition animation
        if let Some(mut trans) = self.transition_state.take() {
            trans.progress += dt / trans.duration.max(0.001);
            if trans.progress >= 1.0 {
                self.switch_to_screen(trans.target_screen_idx);
            } else {
                self.transition_state = Some(trans);
                ctx.request_repaint();
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
            // Command Palette: Ctrl+K / Cmd+K
            if i.modifiers.command && i.key_pressed(Key::K) {
                self.command_palette_open = !self.command_palette_open;
            }
            // File Shortcuts
            if i.modifiers.command && i.key_pressed(Key::O) {
                if let Some((path, content)) = crate::exporter::open_kdl_file_dialog() {
                    self.load_kdl_source(content);
                    self.action_toast = Some((
                        format!("Opened {:?}", path.file_name().unwrap_or_default()),
                        2.0,
                    ));
                }
            }
            if i.modifiers.command && i.key_pressed(Key::S) {
                let def_name = self
                    .parsed_screen
                    .as_ref()
                    .map(|s| s.id.as_str())
                    .unwrap_or("screen");
                if let Some(path) =
                    crate::exporter::save_kdl_file_dialog(&self.kdl_source, def_name)
                {
                    self.action_toast = Some((
                        format!("Saved to {:?}", path.file_name().unwrap_or_default()),
                        2.0,
                    ));
                }
            }
            if i.modifiers.command && i.key_pressed(Key::I) {
                if let Some((path, screens)) = crate::figma_importer::import_figma_dialog() {
                    let count = screens.len();
                    self.project_screens.extend(screens);
                    self.switch_to_screen(self.project_screens.len() - count);
                    self.action_toast = Some((
                        format!(
                            "✓ Imported {} screen(s) from {:?}",
                            count,
                            path.file_name().unwrap_or_default()
                        ),
                        3.0,
                    ));
                }
            }
            if i.modifiers.command && i.key_pressed(Key::E) {
                if let Ok(screen) = &self.parsed_screen {
                    match crate::exporter::export_standalone_crate_dialog(
                        screen,
                        &self.kdl_source,
                        &self.generated_rust,
                    ) {
                        Ok(path) => {
                            self.action_toast = Some((
                                format!(
                                    "✓ Crate exported to {:?}",
                                    path.file_name().unwrap_or_default()
                                ),
                                3.0,
                            ));
                        }
                        Err(err) => {
                            self.action_toast = Some((format!("Export: {}", err), 2.5));
                        }
                    }
                }
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
                                animation: p.animation.clone(),
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
            self.animation_stream_accumulator += dt;
            const STREAM_PERIOD: f32 = 1.0 / 30.0;
            let has_animated_pixels = self
                .parsed_screen
                .as_ref()
                .is_ok_and(crate::live_render::has_animated_content);
            if self.animation_stream_accumulator >= STREAM_PERIOD
                && self.live_stream
                && self.device_link.is_some()
                && has_animated_pixels
            {
                // The link's single latest-frame slot coalesces if SPI is
                // slower than this producer, preventing animation latency.
                self.animation_stream_accumulator %= STREAM_PERIOD;
                self.push_live_frame();
            }
            ctx.request_repaint();
        }

        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(egui::RichText::new("⚡ Embedded GUI Studio").strong());
                ui.separator();

                ui.menu_button("📁 File", |ui| {
                    if ui.button("📂 Open Project (project.kdl)…").clicked() {
                        if let Some(result) = crate::project::open_project_dialog() {
                            match result {
                                Ok(project) => {
                                    let name = project.name.clone();
                                    let n = project.screens.len();
                                    self.load_project(project);
                                    self.action_toast = Some((
                                        format!("Opened project '{name}' ({n} screens)"),
                                        2.5,
                                    ));
                                }
                                Err(err) => {
                                    self.action_toast =
                                        Some((format!("Open project: {err}"), 3.0));
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("💾 Save Project").clicked() {
                        match self.save_project_to_disk() {
                            Ok(path) => {
                                self.action_toast = Some((
                                    format!("Saved project to {}", path.display()),
                                    2.5,
                                ));
                            }
                            Err(err) => {
                                self.action_toast = Some((format!("Save project: {err}"), 3.0));
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("💾 Save Project As…").clicked() {
                        self.project_root = None;
                        match self.save_project_to_disk() {
                            Ok(path) => {
                                self.action_toast = Some((
                                    format!("Saved project to {}", path.display()),
                                    2.5,
                                ));
                            }
                            Err(err) => {
                                self.action_toast = Some((format!("Save project: {err}"), 3.0));
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("🖼 Import Image Asset…").clicked() {
                        match self.import_image_asset() {
                            Ok(path) => {
                                self.action_toast = Some((
                                    format!(
                                        "Imported {} and added it to the active screen",
                                        path.file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("image")
                                    ),
                                    3.0,
                                ));
                            }
                            Err(err) if err != "Import cancelled" => {
                                self.action_toast = Some((format!("Import image: {err}"), 3.0));
                            }
                            Err(_) => {}
                        }
                        ui.close_menu();
                    }
                    if ui.button("🔤 Import Font (BDF)…").clicked() {
                        match self.import_font_asset() {
                            Ok(path) => {
                                self.action_toast = Some((
                                    format!(
                                        "Imported {}; reference it with font=\"{}\"",
                                        path.file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("font"),
                                        path.file_stem()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("font")
                                    ),
                                    4.0,
                                ));
                            }
                            Err(err) if err != "Import cancelled" => {
                                self.action_toast = Some((format!("Import font: {err}"), 4.0));
                            }
                            Err(_) => {}
                        }
                        ui.close_menu();
                    }
                    if ui.button("🧩 Import Icon Art (1-bit)…").clicked() {
                        match self.import_icon_asset() {
                            Ok(path) => {
                                self.action_toast = Some((
                                    format!(
                                        "Imported {} as an icon layer",
                                        path.file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("icon")
                                    ),
                                    3.0,
                                ));
                            }
                            Err(err) if err != "Import cancelled" => {
                                self.action_toast = Some((format!("Import icon: {err}"), 3.0));
                            }
                            Err(_) => {}
                        }
                        ui.close_menu();
                    }
                    if ui.button("🧊 Import Mesh (OBJ)…").clicked() {
                        match self.import_mesh_asset() {
                            Ok(path) => {
                                self.action_toast = Some((
                                    format!(
                                        "Imported {} as a 3D node",
                                        path.file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("mesh")
                                    ),
                                    3.0,
                                ));
                            }
                            Err(err) if err != "Import cancelled" => {
                                self.action_toast = Some((format!("Import mesh: {err}"), 3.0));
                            }
                            Err(_) => {}
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("📂 Open .kdl File... (Ctrl+O)").clicked() {
                        if let Some((path, content)) = crate::exporter::open_kdl_file_dialog() {
                            self.load_kdl_source(content);
                            self.action_toast = Some((
                                format!("Opened {:?}", path.file_name().unwrap_or_default()),
                                2.0,
                            ));
                        }
                        ui.close_menu();
                    }
                    if ui.button("💾 Save .kdl File... (Ctrl+S)").clicked() {
                        let def_name = self
                            .parsed_screen
                            .as_ref()
                            .map(|s| s.id.as_str())
                            .unwrap_or("screen");
                        if let Some(path) =
                            crate::exporter::save_kdl_file_dialog(&self.kdl_source, def_name)
                        {
                            self.action_toast = Some((
                                format!("Saved to {:?}", path.file_name().unwrap_or_default()),
                                2.0,
                            ));
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🎨 Import Figma (.fig)... (Ctrl+I)").clicked() {
                        if let Some((path, screens)) = crate::figma_importer::import_figma_dialog()
                        {
                            let count = screens.len();
                            self.project_screens.extend(screens);
                            self.switch_to_screen(self.project_screens.len() - count);
                            self.action_toast = Some((
                                format!(
                                    "✓ Imported {} screen(s) from {:?}",
                                    count,
                                    path.file_name().unwrap_or_default()
                                ),
                                3.0,
                            ));
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button("📦 Export Standalone Crate... (Ctrl+E)")
                        .clicked()
                    {
                        if let Ok(screen) = &self.parsed_screen {
                            match crate::exporter::export_standalone_crate_dialog(
                                screen,
                                &self.kdl_source,
                                &self.generated_rust,
                            ) {
                                Ok(path) => {
                                    self.action_toast = Some((
                                        format!(
                                            "✓ Crate exported to {:?}",
                                            path.file_name().unwrap_or_default()
                                        ),
                                        3.0,
                                    ));
                                }
                                Err(err) => {
                                    self.action_toast = Some((format!("Export: {}", err), 2.5));
                                }
                            }
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("➕ Insert Widget", |ui| {
                    ui.label(egui::RichText::new("🔘 Controls & Inputs").strong());
                    if ui.button("🔘 Button (Action & Navigate)").clicked() {
                        self.insert_widget(WidgetDef::Button {
                            id: None,
                            text: "CLICK ME".to_string(),
                            on_click: None,
                            style: None,
                        });
                        ui.close_menu();
                    }
                    if ui.button("🔲 Toggle Switch").clicked() {
                        self.insert_widget(WidgetDef::Toggle {
                            id: None,
                            label: "POWER".to_string(),
                            checked: true,
                        });
                        ui.close_menu();
                    }
                    if ui.button("☑ Checkbox").clicked() {
                        self.insert_widget(WidgetDef::Checkbox {
                            id: None,
                            label: "ENABLE".to_string(),
                            checked: false,
                        });
                        ui.close_menu();
                    }
                    if ui.button("🎚 Linear Slider").clicked() {
                        self.insert_widget(WidgetDef::Slider {
                            id: None,
                            min: 0,
                            max: 100,
                            value: 50,
                        });
                        ui.close_menu();
                    }
                    if ui.button("🔢 Spinbox (Precision Digit)").clicked() {
                        self.insert_widget(WidgetDef::Spinbox {
                            id: None,
                            min: 0,
                            max: 999,
                            value: 120,
                            digits: 3,
                            decimals: 1,
                        });
                        ui.close_menu();
                    }
                    if ui.button("🔢 Number Picker (Unit Scroll)").clicked() {
                        self.insert_widget(WidgetDef::NumberPicker {
                            id: None,
                            min: 40,
                            max: 220,
                            value: 135,
                            unit: "BPM".to_string(),
                        });
                        ui.close_menu();
                    }
                    if ui.button("🕒 Time Picker (HH:MM)").clicked() {
                        self.insert_widget(WidgetDef::TimePicker {
                            id: None,
                            hour: 12,
                            minute: 30,
                            is_12h: true,
                            is_pm: true,
                        });
                        ui.close_menu();
                    }
                    if ui.button("📋 Dropdown Menu").clicked() {
                        self.insert_widget(WidgetDef::Dropdown {
                            id: None,
                            options: vec!["Auto".to_string(), "Cool".to_string(), "Heat".to_string()],
                            selected: 0,
                        });
                        ui.close_menu();
                    }
                    if ui.button("🎡 Rotary Roller Wheel").clicked() {
                        self.insert_widget(WidgetDef::Roller {
                            id: None,
                            options: vec!["Low".to_string(), "Med".to_string(), "High".to_string(), "Turbo".to_string()],
                            selected: 1,
                        });
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("📝 Text & Containers").strong());
                    if ui.button("📝 Label (Primary Text)").clicked() {
                        self.insert_widget(WidgetDef::Label {
                            id: None,
                            text: "SYSTEM ONLINE".to_string(),
                            style: None,
                            font: None,
                        });
                        ui.close_menu();
                    }
                    if ui.button("✨ Inverted XOR Status Label").clicked() {
                        self.insert_widget(WidgetDef::Label {
                            id: None,
                            text: "[ ACTIVE ]".to_string(),
                            style: Some("inverted".to_string()),
                            font: None,
                        });
                        ui.close_menu();
                    }
                    if ui.button("📱 Header Status Bar").clicked() {
                        self.insert_widget(WidgetDef::StatusBar {
                            id: None,
                            time: "10:42".to_string(),
                        });
                        ui.close_menu();
                    }
                    if ui.button("💬 Confirmation Dialog").clicked() {
                        self.insert_widget(WidgetDef::Dialog {
                            id: None,
                            title: "Confirm".to_string(),
                            message: "Apply settings now?".to_string(),
                            dialog_type: "confirm".to_string(),
                        });
                        ui.close_menu();
                    }
                    if ui.button("📊 Data Table Grid").clicked() {
                        self.insert_widget(WidgetDef::Table {
                            id: None,
                            headers: Some(vec!["SENSOR".to_string(), "VAL".to_string()]),
                            rows: vec![
                                vec!["Core Temp".to_string(), "42°C".to_string()],
                                vec!["Voltage".to_string(), "3.3V".to_string()],
                            ],
                        });
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("📊 Gauges & Waveforms").strong());
                    if ui.button("📈 Progress Bar").clicked() {
                        self.insert_widget(WidgetDef::ProgressBar {
                            id: None,
                            value: 0.65,
                        });
                        ui.close_menu();
                    }
                    if ui.button("⏱ Radial Tachometer Scale").clicked() {
                        self.insert_widget(WidgetDef::Scale {
                            id: None,
                            mode: "radial".to_string(),
                            min: 0.0,
                            max: 120.0,
                            value: 65.0,
                            major_ticks: 6,
                            minor_ticks: 2,
                        });
                        ui.close_menu();
                    }
                    if ui.button("📐 Sweeping Arc Dial").clicked() {
                        self.insert_widget(WidgetDef::SweepingArc {
                            id: None,
                            start_angle: 0,
                            end_angle: 180,
                        });
                        ui.close_menu();
                    }
                    if ui.button("🌀 Animated Busy Spinner").clicked() {
                        self.insert_widget(WidgetDef::BusyWheel {
                            id: None,
                            active: true,
                        });
                        ui.close_menu();
                    }
                    if ui.button("📉 Oscilloscope Waveform Plotter").clicked() {
                        self.insert_widget(WidgetDef::Plotter {
                            id: None,
                            mode: "waveform".to_string(),
                        });
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("📐 Vector Shapes & Splines").strong());
                    if ui.button("🔲 Bezel Rectangle (Border & Fill)").clicked() {
                        self.insert_widget(WidgetDef::RectShape {
                            id: None,
                            radius: 2,
                            stroke_width: 1,
                            fill_color: Some("#000000".to_string()),
                            stroke_color: Some("#FFFFFF".to_string()),
                        });
                        ui.close_menu();
                    }
                    if ui.button("⚪ Vector Circle").clicked() {
                        self.insert_widget(WidgetDef::CircleShape {
                            id: None,
                            radius: 12,
                            stroke_width: 1,
                            fill_color: None,
                            stroke_color: Some("#FFFFFF".to_string()),
                        });
                        ui.close_menu();
                    }
                    if ui.button("➖ Divider Line").clicked() {
                        self.insert_widget(WidgetDef::LineShape {
                            id: None,
                            stroke_width: 1,
                            color: Some("#FFFFFF".to_string()),
                        });
                        ui.close_menu();
                    }
                    if ui.button("✒️ SVG Bézier Curve Path").clicked() {
                        self.insert_vector_asset("curve", "M 0 10 C 20 0, 40 40, 60 10");
                        ui.close_menu();
                    }
                });

                ui.menu_button("🎨 Vector Assets", |ui| {
                    ui.label(egui::RichText::new("🔋 Power & Battery").strong());
                    if ui.button("🔋 Battery 100% (Full)").clicked() {
                        self.insert_vector_asset("batt_full", "M 0 0 L 14 0 L 14 6 L 0 6 Z M 14 2 L 15 2 L 15 4 L 14 4 Z M 2 2 L 12 2 L 12 4 L 2 4 Z");
                        ui.close_menu();
                    }
                    if ui.button("🪫 Battery 20% (Low)").clicked() {
                        self.insert_vector_asset("batt_low", "M 0 0 L 14 0 L 14 6 L 0 6 Z M 14 2 L 15 2 L 15 4 L 14 4 Z M 2 2 L 4 2 L 4 4 L 2 4 Z");
                        ui.close_menu();
                    }
                    if ui.button("⚡ Lightning Bolt (Charging)").clicked() {
                        self.insert_vector_asset("bolt", "M 6 0 L 1 7 L 5 7 L 3 13 L 10 5 L 5 5 Z");
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("📶 Connectivity").strong());
                    if ui.button("📡 Bluetooth 5.2 Icon").clicked() {
                        self.insert_vector_asset("bluetooth", "M 4 1 L 8 5 L 5 8 L 5 0 L 8 3 L 4 7");
                        ui.close_menu();
                    }
                    if ui.button("📶 Cellular Signal Bars").clicked() {
                        self.insert_vector_asset("signal", "M 1 6 L 3 6 L 3 8 L 1 8 Z M 5 4 L 7 4 L 7 8 L 5 8 Z M 9 2 L 11 2 L 11 8 L 9 8 Z");
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("🛡️ Badges & Status").strong());
                    if ui.button("⚠️ Warning Triangle").clicked() {
                        self.insert_vector_asset("warning", "M 6 1 L 12 11 L 0 11 Z M 6 4 L 6 7 M 6 9 L 6 10");
                        ui.close_menu();
                    }
                    if ui.button("🛡️ Security Shield").clicked() {
                        self.insert_vector_asset("shield", "M 0 2 L 6 0 L 12 2 L 12 7 C 12 10, 6 13, 6 13 C 6 13, 0 10, 0 7 Z");
                        ui.close_menu();
                    }
                    if ui.button("❤️ Heart Pulse (Health)").clicked() {
                        self.insert_vector_asset("heart", "M 6 2 C 4 0, 0 1, 0 4 C 0 8, 6 11, 6 11 C 6 11, 12 8, 12 4 C 12 1, 8 0, 6 2 Z");
                        ui.close_menu();
                    }
                    if ui.button("🎯 Target Crosshair").clicked() {
                        self.insert_vector_asset("crosshair", "M 6 0 L 6 3 M 6 9 L 6 12 M 0 6 L 3 6 M 9 6 L 12 6 M 6 2 C 8.2 2, 10 3.8, 10 6 C 10 8.2, 8.2 10, 6 10 C 3.8 10, 2 8.2, 2 6 C 2 3.8, 3.8 2, 6 2 Z");
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("⚙️ Navigation & Tools").strong());
                    if ui.button("⏱ Timer Clock Dial").clicked() {
                        self.insert_vector_asset("timer", "M 6 0 C 9.3 0, 12 2.7, 12 6 C 12 9.3, 9.3 12, 6 12 C 2.7 12, 0 9.3, 0 6 C 0 2.7, 2.7 0, 6 0 Z M 6 2 L 6 6 L 9 6");
                        ui.close_menu();
                    }
                    if ui.button("⚙️ Settings Gear").clicked() {
                        self.insert_vector_asset("gear", "M 5 0 L 7 0 L 7 2 L 9 3 L 11 1 L 12 2 L 10 4 L 11 6 L 13 6 L 13 8 L 11 8 L 10 10 L 12 12 L 11 13 L 9 11 L 7 12 L 7 14 L 5 14 L 5 12 L 3 11 L 1 13 L 0 12 L 2 10 L 1 8 L 0 8 L 0 6 L 2 6 L 1 4 L 2 2 L 4 3 L 5 2 Z");
                        ui.close_menu();
                    }
                    if ui.button("▶ Play Arrow").clicked() {
                        self.insert_vector_asset("play", "M 2 1 L 11 6 L 2 11 Z");
                        ui.close_menu();
                    }
                    if ui.button("⏸ Pause Double Bar").clicked() {
                        self.insert_vector_asset("pause", "M 2 1 L 5 1 L 5 11 L 2 11 Z M 7 1 L 10 1 L 10 11 L 7 11 Z");
                        ui.close_menu();
                    }
                });

                ui.menu_button("✨ Easing Curves", |ui| {
                    ui.label(egui::RichText::new("✨ Motion Easing Solver").strong());
                    let curves = [
                        (EasingCurve::Linear, "Linear (Constant Rate)"),
                        (EasingCurve::EaseInQuad, "EaseInQuad (Gentle Start)"),
                        (EasingCurve::EaseOutQuad, "EaseOutQuad (Gentle Decel)"),
                        (EasingCurve::EaseInOutQuad, "EaseInOutQuad (Smooth S-Curve)"),
                        (EasingCurve::EaseInCubic, "EaseInCubic (Accelerating)"),
                        (EasingCurve::EaseOutCubic, "EaseOutCubic (Decelerating)"),
                        (EasingCurve::EaseInOutCubic, "EaseInOutCubic (Natural Motion)"),
                        (EasingCurve::EaseOutBack, "EaseOutBack (Overshoot & Settle)"),
                        (EasingCurve::EaseOutBounce, "EaseOutBounce (Falling Ball Settle)"),
                        (EasingCurve::Moook, "Moook (Pebble Rubber-Band Physics)"),
                        (EasingCurve::CubicBezier, "CubicBezier (Custom Spline Parameterized)"),
                    ];
                    for (curve, name) in curves {
                        if ui.selectable_label(self.selected_easing == curve, name).clicked() {
                            self.selected_easing = curve;
                            self.action_toast = Some((format!("Active Easing: {:?}", curve), 2.0));
                            ui.close_menu();
                        }
                    }
                });

                ui.menu_button("📄 Presets", |ui| {
                    if ui.button("📟 SSD1306 OLED (128×64 Mono)").clicked() {
                        self.display_theme = DisplayTheme::MonochromeOled;
                        self.hardware_profile = HardwareProfile::Ssd1306Oled;
                        self.load_kdl_source(SAMPLE_SSD1306_OLED.to_string());
                        ui.close_menu();
                    }
                    if ui.button("📟 SSD1357 OLED (96×64 RGB)").clicked() {
                        self.display_theme = DisplayTheme::DarkTft;
                        self.hardware_profile = HardwareProfile::Ssd1357;
                        self.load_kdl_source(SAMPLE_SSD1357.to_string());
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🚗 Automotive Digital Cluster").clicked() {
                        self.load_kdl_source(SAMPLE_AUTOMOTIVE_CLUSTER.to_string());
                        ui.close_menu();
                    }
                    if ui.button("❄️ HVAC Smart Climate").clicked() {
                        self.load_kdl_source(SAMPLE_HVAC_CLIMATE.to_string());
                        ui.close_menu();
                    }
                    if ui.button("🩺 Patient Vital Monitor").clicked() {
                        self.load_kdl_source(SAMPLE_PATIENT_MONITOR.to_string());
                        ui.close_menu();
                    }
                    if ui.button("⚙️ Industrial CNC Controller").clicked() {
                        self.load_kdl_source(SAMPLE_CNC_CONTROLLER.to_string());
                        ui.close_menu();
                    }
                    if ui.button("⌚ Smartwatch Activity Tracker").clicked() {
                        self.load_kdl_source(SAMPLE_SMARTWATCH_FITNESS.to_string());
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("📈 Live Oscilloscope").clicked() {
                        self.load_kdl_source(SAMPLE_WAVEFORM.to_string());
                        ui.close_menu();
                    }
                    if ui.button("✨ Motion Kitchen Sink").clicked() {
                        self.load_kdl_source(SAMPLE_MOTION_KITCHEN_SINK.to_string());
                        ui.close_menu();
                    }
                    if ui.button("🌡 Smart Thermostat").clicked() {
                        self.load_kdl_source(SAMPLE_THERMOSTAT.to_string());
                        ui.close_menu();
                    }
                    if ui.button("📊 Sensor Dashboard").clicked() {
                        self.load_kdl_source(SAMPLE_DASHBOARD.to_string());
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

        // Bottom Hardware Profiler Bar & Silicon Bridge
        egui::TopBottomPanel::bottom("bottom_hardware_profiler").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Ok(screen) = &self.parsed_screen {
                    let bpp = self.hardware_profile.bpp();
                    let fb_bytes = (screen.width * screen.height * bpp) / 8;
                    let fb_kb = fb_bytes as f32 / 1024.0;
                    let static_ram_kb = (screen.grid.children.len() * 96) as f32 / 1024.0;
                    let spi_mb_sec = (fb_bytes as f32 * 60.0) / 1_000_000.0;

                    ui.label(egui::RichText::new("📊 Budget:").strong());
                    ui.label(format!("FB: {:.1}KB", fb_kb));
                    ui.separator();
                    ui.label(format!("Nodes: {:.2}KB", static_ram_kb));
                    ui.separator();
                    ui.label(format!("SPI 60FPS: {:.2}MB/s", spi_mb_sec));
                    ui.separator();

                    if let Some((cx, cy)) = self.cursor_screen_coords {
                        ui.label(
                            egui::RichText::new(format!("📍 Cursor: X:{cx} Y:{cy}"))
                                .color(Color32::from_rgb(120, 220, 160))
                                .monospace(),
                        );
                    } else {
                        ui.label(egui::RichText::new("📍 Cursor: --:--").weak().monospace());
                    }
                    ui.separator();

                    // Hardware Bridge controls
                    if self.hardware_bridge.is_running {
                        let count = self
                            .hardware_bridge
                            .client_count
                            .lock()
                            .map(|c| *c)
                            .unwrap_or(0);
                        ui.colored_label(
                            Color32::from_rgb(80, 220, 120),
                            format!("🔌 Bridge (9080): {} dev", count),
                        );
                        if ui.button("⚡ Hot Reload").clicked() {
                            let sent = self.hardware_bridge.broadcast_kdl(&self.kdl_source);
                            self.action_toast =
                                Some((format!("Hot-reloaded {} device(s)", sent), 2.0));
                        }
                    } else if ui.button("🔌 Start Hardware Bridge").clicked()
                        && self.hardware_bridge.start().is_ok()
                    {
                        self.action_toast =
                            Some(("Bridge listening on 127.0.0.1:9080".into(), 2.0));
                    }

                    ui.separator();

                    // USB Display Agent controls (native 512-byte USB bulk)
                    // Surface worker-thread failures without blocking the UI.
                    if let Some(link) = self.device_link.as_ref() {
                        let err = link.take_error();
                        if err.is_some() || !link.is_alive() {
                            let msg = err.unwrap_or_else(|| "device disconnected".to_string());
                            self.device_link = None;
                            self.action_toast = Some((format!("USB link lost: {msg}"), 3.0));
                        }
                    }

                    let connection = self
                        .device_link
                        .as_ref()
                        .map(|l| (l.device_id().to_string(), l.is_handshaked()));
                    match connection {
                        Some((name, handshaked)) => {
                            let color = if handshaked {
                                Color32::from_rgb(80, 220, 120)
                            } else {
                                Color32::from_rgb(230, 180, 60)
                            };
                            let suffix = if handshaked { "" } else { " (no ACK)" };
                            ui.colored_label(color, format!("🖥 USB: {}{}", name, suffix));
                            if ui.button("⏏ Disconnect").clicked() {
                                self.device_link = None;
                            } else if ui.button("⚡ Push Frame").clicked() {
                                self.push_live_frame_full();
                            }
                            ui.checkbox(&mut self.live_stream, "Live");
                            if let Some((sw, sh, pw, ph)) = self.device_size_warning {
                                ui.colored_label(
                                    Color32::from_rgb(230, 180, 60),
                                    format!("⚠ screen {sw}x{sh} ≠ panel {pw}x{ph} (centered)"),
                                );
                            }
                        }
                        None => {
                            if self.device_ports.is_empty() {
                                self.device_ports = crate::device_link::list_devices();
                            }
                            let selected_text = self
                                .selected_port
                                .clone()
                                .unwrap_or_else(|| "Select USB device".to_string());
                            let ports = self.device_ports.clone();
                            egui::ComboBox::from_id_salt("usb_port_selector")
                                .selected_text(selected_text)
                                .show_ui(ui, |ui| {
                                    for p in &ports {
                                        ui.selectable_value(
                                            &mut self.selected_port,
                                            Some(p.clone()),
                                            p,
                                        );
                                    }
                                });
                            if ui.small_button("🔄").clicked() {
                                self.device_ports = crate::device_link::list_devices();
                            }
                            if ui.button("🔌 Connect USB").clicked() {
                                if let Some(device_id) = self.selected_port.clone() {
                                    match crate::device_link::DeviceLink::connect(&device_id) {
                                        Ok(link) => {
                                            self.device_link = Some(link);
                                            self.push_live_frame();
                                            self.action_toast =
                                                Some((format!("Connected {}", device_id), 2.0));
                                        }
                                        Err(e) => {
                                            self.action_toast = Some((e, 3.0));
                                        }
                                    }
                                } else {
                                    self.action_toast =
                                        Some(("Select a USB display agent first".into(), 2.0));
                                }
                            }
                        }
                    }
                }
            });
        });

        // Left Panel: KDL Code Editor with Syntax Highlighting
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

                let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                    let mut job = crate::syntax::highlight_kdl(ui.ctx(), string);
                    job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(job))
                };

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 70.0)
                    .show(ui, |ui| {
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut self.kdl_source)
                                .layouter(&mut layouter)
                                .desired_rows(24)
                                .desired_width(f32::INFINITY)
                                .lock_focus(true),
                        );
                        if response.changed() {
                            self.recompile();
                            if self.hardware_bridge.is_running {
                                self.hardware_bridge.broadcast_kdl(&self.kdl_source);
                            }
                            if self.live_stream && self.device_link.is_some() {
                                self.push_live_frame();
                            }
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
                    let previous_size = (screen.width, screen.height);
                    let mut sel_idx = self.selected_widget_idx;
                    let available_screens: Vec<String> = self
                        .project_screens
                        .iter()
                        .map(|(name, _)| name.clone())
                        .collect();
                    let modified =
                        render_inspector_panel(ui, &mut screen, &mut sel_idx, &available_screens);
                    self.selected_widget_idx = sel_idx;
                    if modified {
                        if (screen.width, screen.height) != previous_size {
                            self.hardware_profile = HardwareProfile::Custom;
                        }
                        self.sync_from_screen(&screen);
                    }
                } else {
                    ui.label("Fix KDL syntax errors to use the Inspector.");
                }
            });

        // Center Panel: Tabs (Visual Preview / Rust Codegen / AST / Assets / Flow)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.active_tab,
                    StudioTab::VisualPreview,
                    "🖥 Visual Preview",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    StudioTab::ScreenFlow,
                    "🗺 Screen Flow",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    StudioTab::AssetBrowser,
                    "🔤 Fonts & Assets",
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
                ui.selectable_value(
                    &mut self.active_tab,
                    StudioTab::Profiler,
                    "⚡ Resource Profiler",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    StudioTab::SignalPlayground,
                    "🎮 Signal Playground",
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
                StudioTab::ScreenFlow => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("🗺️ Multi-Screen Flow & Navigation State Machine");
                        ui.label(egui::RichText::new("Interactive navigation routing and transition pipeline for multi-screen embedded applications. Click any screen card to select it.").weak());
                        ui.separator();

                        let mut to_switch = None;

                        // Visual Interactive Screen Nodes Grid
                        egui::Grid::new("screen_flow_grid")
                            .num_columns(3)
                            .spacing([16.0, 16.0])
                            .show(ui, |ui| {
                                for (i, (name, kdl)) in self.project_screens.iter().enumerate() {
                                    let is_active = i == self.active_screen_idx;
                                    let icon = match name.as_str() {
                                        "AutoCluster" => "🚗",
                                        "HvacClimate" => "❄️",
                                        "PatientMonitor" => "🩺",
                                        "CncController" => "⚙️",
                                        "FitnessTracker" => "⌚",
                                        _ => "📱",
                                    };

                                    let parsed = parse_kdl_screen(kdl);
                                    let widget_count = parsed.as_ref().map(|s| s.grid.children.len()).unwrap_or(0);
                                    let (w, h) = parsed.as_ref().map(|s| (s.width, s.height)).unwrap_or((320, 240));

                                    ui.group(|ui| {
                                        ui.set_width(180.0);
                                        ui.vertical_centered(|ui| {
                                            ui.label(egui::RichText::new(icon).size(32.0));
                                            ui.label(egui::RichText::new(name).strong().size(13.0));
                                            ui.label(format!("{}×{} px • {} widgets", w, h, widget_count));

                                            if is_active {
                                                ui.colored_label(Color32::from_rgb(80, 220, 120), "● Active Screen");
                                            } else if ui.button("Select Screen").clicked() {
                                                to_switch = Some(i);
                                            }
                                        });
                                    });

                                    if (i + 1) % 3 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });

                        if let Some(idx) = to_switch {
                            self.switch_to_screen(idx);
                        }

                        ui.add_space(16.0);
                        ui.heading("🔗 Connected Screen Transitions");
                        ui.separator();

                        if let Ok(screen) = &self.parsed_screen {
                            let mut nav_buttons = Vec::new();
                            for (p, w) in &screen.grid.children {
                                if let embedded_gui_codegen::WidgetDef::Button { text, on_click: Some(act), .. } = w {
                                    if act.starts_with("navigate:") {
                                        let parts: Vec<&str> = act.split(':').collect();
                                        let target = parts.get(1).copied().unwrap_or("?");
                                        let effect = parts.get(2).copied().unwrap_or("SlideLeft");
                                        nav_buttons.push((text.as_str(), target, effect, p.col, p.row));
                                    }
                                }
                            }

                            if nav_buttons.is_empty() {
                                ui.label(egui::RichText::new("No navigation buttons defined on this screen yet. Select a Button in Design Mode and set its Action Trigger to 'Navigate to Screen'!").weak());
                            } else {
                                for (btn_text, target, effect, c, r) in nav_buttons {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("🔘 Button '{}' (c:{}, r:{})", btn_text, c, r));
                                            ui.colored_label(Color32::from_rgb(80, 220, 120), "➔");
                                            ui.label(egui::RichText::new(target).strong());
                                            ui.label(format!("via {}", effect));
                                        });
                                    });
                                }
                            }
                        }
                    });
                }
                StudioTab::AssetBrowser => {
                    crate::assets::render_asset_browser(ui, &mut self.action_toast);
                }
                StudioTab::RustCodegen => {
                    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        let mut job = crate::syntax::highlight_rust(ui.ctx(), string);
                        job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(job))
                    };
                    egui::ScrollArea::both().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.generated_rust.as_str())
                                .layouter(&mut layouter)
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
                StudioTab::Profiler => {
                    if let Ok(screen) = &self.parsed_screen {
                        crate::profiler::render_profiler_panel(ui, screen, &self.hardware_profile);
                    } else {
                        ui.label("Fix KDL syntax errors to analyze memory profile.");
                    }
                }
                StudioTab::SignalPlayground => {
                    if let Ok(mut screen) = self.parsed_screen.clone() {
                        let mutated = crate::playground::render_playground_panel(
                            ui,
                            &mut self.mock_playground,
                            &mut screen,
                        );
                        if mutated {
                            self.sync_from_screen(&screen);
                        }
                    } else {
                        ui.label("Fix KDL syntax errors to use Signal Playground.");
                    }
                }
            }
        });

        crate::command_palette::render_command_palette(self, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size(app: &EmbeddedGuiStudio) -> (u32, u32) {
        let screen = app.parsed_screen.as_ref().expect("screen parses");
        (screen.width, screen.height)
    }

    #[test]
    fn startup_selects_the_target_matching_the_first_screen() {
        let app = EmbeddedGuiStudio::new_offline();
        assert_eq!(
            app.hardware_profile,
            HardwareProfile::from_dimensions(size(&app).0, size(&app).1)
        );
    }

    #[test]
    fn returning_to_a_screen_restores_its_authored_target() {
        let mut app = EmbeddedGuiStudio::new_offline();
        app.switch_to_screen(1);
        app.switch_to_screen(0);
        assert_eq!(size(&app), (480, 272));
        assert_eq!(app.hardware_profile, HardwareProfile::Stm32H7Capacitive);
    }

    #[test]
    fn loading_a_preset_selects_its_matching_target() {
        let mut app = EmbeddedGuiStudio::new_offline();
        app.hardware_profile = HardwareProfile::Ssd1306Oled;
        app.load_kdl_source(SAMPLE_AUTOMOTIVE_CLUSTER.to_string());
        assert_eq!(size(&app), (480, 272));
        assert_eq!(app.hardware_profile, HardwareProfile::Stm32H7Capacitive);
    }

    #[test]
    fn loading_non_standard_dimensions_selects_custom() {
        let mut app = EmbeddedGuiStudio::new_offline();
        let source = SAMPLE_SSD1357.replace("width=96", "width=97");
        app.load_kdl_source(source);
        assert_eq!(app.hardware_profile, HardwareProfile::Custom);
        assert_eq!(size(&app), (97, 64));
    }

    #[test]
    fn opening_a_96x64_screen_selects_the_ssd1357_target() {
        let mut app = EmbeddedGuiStudio::new_offline();
        app.load_kdl_source(SAMPLE_SSD1357.to_string());
        assert_eq!(app.hardware_profile, HardwareProfile::Ssd1357);
        assert_eq!(size(&app), (96, 64));
    }

    #[test]
    fn editing_kdl_dimensions_reverts_the_target_to_custom() {
        let mut app = EmbeddedGuiStudio::new_offline();
        app.load_kdl_source(SAMPLE_SSD1357.to_string());
        assert_eq!(app.hardware_profile, HardwareProfile::Ssd1357);

        app.kdl_source = app.kdl_source.replace("width=96", "width=95");
        app.recompile();

        assert_eq!(app.hardware_profile, HardwareProfile::Custom);
        assert_eq!(size(&app), (95, 64));
    }
}
