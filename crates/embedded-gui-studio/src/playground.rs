//! Simulated Hardware and Reactive Signal Playground.
//! Allows UI designers to mock telemetry signals (sensors, battery, toggles, LFO waves)
//! and observe UI widget reactivity without flashing physical hardware.

use eframe::egui::{self, Color32, RichText, Slider};
use embedded_gui_codegen::ScreenDef;

/// State for the simulated hardware signals.
#[derive(Debug, Clone)]
pub struct MockPlaygroundState {
    pub battery_pct: f32,
    pub temperature_deg_c: f32,
    pub fan_speed: i32,
    pub eco_mode: bool,
    pub system_status: String,
    pub lfo_enabled: bool,
    pub lfo_speed: f32,
    pub lfo_phase: f32,
}

impl Default for MockPlaygroundState {
    fn default() -> Self {
        Self {
            battery_pct: 82.0,
            temperature_deg_c: 23.5,
            fan_speed: 1200,
            eco_mode: true,
            system_status: "SYSTEM READY".to_string(),
            lfo_enabled: false,
            lfo_speed: 1.0,
            lfo_phase: 0.0,
        }
    }
}

impl MockPlaygroundState {
    pub fn tick(&mut self, dt: f32) {
        if self.lfo_enabled {
            self.lfo_phase = (self.lfo_phase + dt * self.lfo_speed) % (core::f32::consts::TAU);
            let sin_val = (self.lfo_phase.sin() + 1.0) / 2.0; // 0.0..1.0
            self.temperature_deg_c = 18.0 + sin_val * 14.0;
            self.battery_pct = 70.0 + sin_val * 25.0;
            self.fan_speed = (800.0 + sin_val * 2400.0) as i32;
        }
    }

    /// Injects mock signal values into matching widgets on the screen.
    pub fn inject_into_screen(&self, screen: &mut ScreenDef) -> bool {
        let mut mutated = false;
        for (_p, widget) in &mut screen.grid.children {
            match widget {
                embedded_gui_codegen::WidgetDef::Scale { id, value, .. } => {
                    let id_str = id.as_deref().unwrap_or("").to_lowercase();
                    if (id_str.contains("temp") || id_str.contains("room"))
                        && (*value - self.temperature_deg_c).abs() > 0.01
                    {
                        *value = self.temperature_deg_c;
                        mutated = true;
                    } else if id_str.contains("batt") && (*value - self.battery_pct).abs() > 0.01 {
                        *value = self.battery_pct;
                        mutated = true;
                    }
                }
                embedded_gui_codegen::WidgetDef::ProgressBar { id, value, .. } => {
                    let id_str = id.as_deref().unwrap_or("").to_lowercase();
                    if id_str.contains("batt") || id_str.contains("power") {
                        let target_prog = self.battery_pct.clamp(0.0, 100.0);
                        if (*value - target_prog).abs() > 0.005 {
                            *value = target_prog;
                            mutated = true;
                        }
                    }
                }
                embedded_gui_codegen::WidgetDef::Spinbox { id, value, .. } => {
                    let id_str = id.as_deref().unwrap_or("").to_lowercase();
                    if id_str.contains("temp") {
                        let target_val = (self.temperature_deg_c * 10.0) as i32;
                        if *value != target_val {
                            *value = target_val;
                            mutated = true;
                        }
                    }
                }
                embedded_gui_codegen::WidgetDef::Toggle { id, checked, .. } => {
                    let id_str = id.as_deref().unwrap_or("").to_lowercase();
                    if id_str.contains("eco") && *checked != self.eco_mode {
                        *checked = self.eco_mode;
                        mutated = true;
                    }
                }
                _ => {}
            }
        }
        mutated
    }
}

pub fn render_playground_panel(
    ui: &mut egui::Ui,
    state: &mut MockPlaygroundState,
    screen: &mut ScreenDef,
) -> bool {
    let mut mutated = false;

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("🎮 Simulated Hardware & Signal Playground");
        ui.label(
            RichText::new("Tweak mock telemetry signals and reactive properties in real-time.")
                .weak(),
        );
        ui.separator();

        ui.horizontal(|ui| {
            if ui.checkbox(&mut state.lfo_enabled, "🌊 Live LFO Sensor Wave Generator").changed() {
                mutated = true;
            }
            if state.lfo_enabled {
                ui.label("Speed:");
                ui.add(Slider::new(&mut state.lfo_speed, 0.1..=5.0).show_value(true));
            }
        });

        ui.add_space(8.0);
        ui.heading("📡 Telemetry Signals");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("🔋 Battery Level (%):");
            if ui.add(Slider::new(&mut state.battery_pct, 0.0..=100.0).show_value(true)).changed() {
                mutated = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("🌡 Temperature (°C):");
            if ui.add(Slider::new(&mut state.temperature_deg_c, -10.0..=50.0).show_value(true)).changed() {
                mutated = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("🌀 Fan Speed (RPM):");
            if ui.add(Slider::new(&mut state.fan_speed, 0..=4000).show_value(true)).changed() {
                mutated = true;
            }
        });

        ui.horizontal(|ui| {
            if ui.checkbox(&mut state.eco_mode, "🌱 Eco Mode Toggle").changed() {
                mutated = true;
            }
        });

        ui.add_space(12.0);
        ui.heading("🔤 Injected Strings & System Status");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Status Text:");
            if ui.text_edit_singleline(&mut state.system_status).changed() {
                mutated = true;
            }
        });

        ui.add_space(16.0);
        ui.colored_label(
            Color32::from_rgb(100, 200, 240),
            "💡 Signal values automatically bind to matching widget IDs (e.g. TempSetpoint, RoomGauge, EcoMode).",
        );
    });

    if mutated {
        state.inject_into_screen(screen);
    }

    mutated
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_gui_codegen::parse_kdl_screen;

    #[test]
    fn test_mock_signal_injection() {
        let kdl = "screen id=\"Dashboard\" width=320 height=240 {\n    grid cols=\"1fr\" rows=\"1fr\" {\n        scale id=\"RoomGauge\" min=0.0 max=50.0 value=20.0\n    }\n}\n";
        let mut screen = parse_kdl_screen(kdl).unwrap();
        let state = MockPlaygroundState {
            temperature_deg_c: 28.5,
            ..Default::default()
        };

        assert!(state.inject_into_screen(&mut screen));

        if let embedded_gui_codegen::WidgetDef::Scale { value, .. } = &screen.grid.children[0].1 {
            assert!((*value - 28.5).abs() < 1e-4);
        } else {
            panic!("Expected scale widget");
        }
    }
}
