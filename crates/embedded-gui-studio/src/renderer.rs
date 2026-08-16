//! 2D simulated LCD canvas widget renderer.

use core::f32::consts::PI;
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};
use embedded_gui_codegen::WidgetDef;

/// Draws an animated 2D preview representation of an embedded-gui widget.
pub fn draw_animated_widget(painter: &egui::Painter, rect: Rect, widget: &WidgetDef, time: f32) {
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
            } else if style.as_deref() == Some("danger") {
                Color32::from_rgb(220, 45, 45)
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
                } else if mode == "triangle" {
                    let saw = (phase / PI).fract();
                    let tri = if saw < 0.5 {
                        saw * 4.0 - 1.0
                    } else {
                        3.0 - saw * 4.0
                    };
                    center_y - tri * amp * 0.7
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
