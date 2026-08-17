//! 2D simulated LCD canvas widget renderer with display theme shaders and active touch feedback.

use crate::types::DisplayTheme;
use core::f32::consts::PI;
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};
use embedded_gui_codegen::WidgetDef;

/// Theme color mapping token palette.
pub struct ThemePalette {
    pub display_bg: Color32,
    pub card_bg: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub success: Color32,
    pub danger: Color32,
}

impl ThemePalette {
    pub fn for_theme(theme: DisplayTheme) -> Self {
        match theme {
            DisplayTheme::DarkTft => Self {
                display_bg: Color32::from_rgb(18, 20, 24),
                card_bg: Color32::from_rgb(30, 33, 40),
                border: Color32::from_rgb(55, 62, 75),
                text_primary: Color32::from_rgb(230, 235, 245),
                text_dim: Color32::from_rgb(140, 150, 165),
                accent: Color32::from_rgb(45, 110, 220),
                success: Color32::from_rgb(40, 190, 110),
                danger: Color32::from_rgb(220, 50, 50),
            },
            DisplayTheme::LightTft => Self {
                display_bg: Color32::from_rgb(240, 244, 248),
                card_bg: Color32::from_rgb(255, 255, 255),
                border: Color32::from_rgb(205, 215, 225),
                text_primary: Color32::from_rgb(20, 25, 35),
                text_dim: Color32::from_rgb(90, 100, 115),
                accent: Color32::from_rgb(25, 95, 210),
                success: Color32::from_rgb(30, 160, 90),
                danger: Color32::from_rgb(210, 40, 40),
            },
            DisplayTheme::AmberPhosphor => Self {
                display_bg: Color32::from_rgb(15, 10, 4),
                card_bg: Color32::from_rgb(30, 20, 6),
                border: Color32::from_rgb(140, 95, 20),
                text_primary: Color32::from_rgb(255, 180, 40),
                text_dim: Color32::from_rgb(180, 125, 25),
                accent: Color32::from_rgb(255, 160, 20),
                success: Color32::from_rgb(255, 195, 50),
                danger: Color32::from_rgb(255, 90, 20),
            },
            DisplayTheme::EmeraldGreen => Self {
                display_bg: Color32::from_rgb(4, 15, 8),
                card_bg: Color32::from_rgb(8, 30, 15),
                border: Color32::from_rgb(25, 120, 55),
                text_primary: Color32::from_rgb(50, 255, 120),
                text_dim: Color32::from_rgb(35, 175, 80),
                accent: Color32::from_rgb(40, 230, 100),
                success: Color32::from_rgb(80, 255, 140),
                danger: Color32::from_rgb(255, 140, 40),
            },
            DisplayTheme::MonochromeOled => Self {
                display_bg: Color32::BLACK,
                card_bg: Color32::from_rgb(16, 16, 16),
                border: Color32::WHITE,
                text_primary: Color32::WHITE,
                text_dim: Color32::from_rgb(180, 180, 180),
                accent: Color32::WHITE,
                success: Color32::WHITE,
                danger: Color32::WHITE,
            },
        }
    }
}

/// Draws an animated 2D preview representation of an embedded-gui widget.
pub fn draw_animated_widget(
    painter: &egui::Painter,
    rect: Rect,
    widget: &WidgetDef,
    time: f32,
    theme: DisplayTheme,
    is_pressed: bool,
) {
    let p = ThemePalette::for_theme(theme);

    match widget {
        WidgetDef::Label { text, .. } => {
            painter.rect_filled(rect, CornerRadius::same(3), p.card_bg);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                FontId::proportional(12.0),
                p.text_primary,
            );
        }
        WidgetDef::Button { text, style, .. } => {
            let mut bg = if is_pressed {
                Color32::from_rgb(70, 150, 255)
            } else if style.as_deref() == Some("accent") {
                p.accent
            } else if style.as_deref() == Some("danger") {
                p.danger
            } else {
                p.card_bg
            };

            if is_pressed {
                bg = Color32::from_rgb(90, 170, 255);
            }

            let btn_rect = if is_pressed { rect.shrink(1.5) } else { rect };
            painter.rect_filled(btn_rect, CornerRadius::same(4), bg);
            painter.rect_stroke(
                btn_rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, p.border),
                StrokeKind::Inside,
            );
            painter.text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("🔘 {}", text),
                FontId::proportional(12.0),
                if is_pressed {
                    Color32::WHITE
                } else {
                    p.text_primary
                },
            );
        }
        WidgetDef::Toggle { label, checked, .. } => {
            painter.rect_filled(rect, CornerRadius::same(4), p.card_bg);
            let check_icon = if *checked { " [ON]" } else { " [OFF]" };
            let text_color = if *checked { p.success } else { p.text_dim };
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("⏻ {}{}", label, check_icon),
                FontId::proportional(11.0),
                text_color,
            );
        }
        WidgetDef::Checkbox { label, checked, .. } => {
            painter.rect_filled(rect, CornerRadius::same(3), p.card_bg);
            let mark = if *checked { "☑" } else { "☐" };
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{} {}", mark, label),
                FontId::proportional(11.0),
                p.text_primary,
            );
        }
        WidgetDef::Slider {
            min, max, value, ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(3), p.card_bg);
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
            painter.rect_filled(fill_rect, CornerRadius::same(3), p.accent);

            // Slider thumb handle knob
            let thumb_x = track_rect.min.x + fill_w;
            painter.circle_filled(Pos2::new(thumb_x, rect.center().y), 5.0, Color32::WHITE);

            painter.text(
                Pos2::new(rect.center().x, rect.min.y + 7.0),
                egui::Align2::CENTER_CENTER,
                format!("Slider: {}", value),
                FontId::proportional(10.0),
                p.text_dim,
            );
        }
        WidgetDef::ProgressBar { value, .. } => {
            painter.rect_filled(rect, CornerRadius::same(3), p.card_bg);
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
            painter.rect_filled(fill_rect, CornerRadius::same(3), p.success);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{:.0}%", animated_val * 100.0),
                FontId::proportional(11.0),
                p.text_primary,
            );
        }
        WidgetDef::Scale {
            mode,
            min,
            max,
            value,
            ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(4), p.card_bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, p.border),
                StrokeKind::Inside,
            );

            let dynamic_val = *value + ((*max - *min) * 0.15 * (time * 1.8).sin());
            let clamped_val = dynamic_val.clamp(*min, *max);

            painter.text(
                Pos2::new(rect.center().x, rect.center().y - 8.0),
                egui::Align2::CENTER_CENTER,
                format!("⏱ Scale ({})", mode),
                FontId::proportional(11.0),
                p.accent,
            );
            painter.text(
                Pos2::new(rect.center().x, rect.center().y + 8.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.1} [{:.0}..{:.0}]", clamped_val, min, max),
                FontId::proportional(12.0),
                p.text_primary,
            );
        }
        WidgetDef::BusyWheel { active, .. } => {
            painter.rect_filled(rect, CornerRadius::same(4), p.card_bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, p.border),
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
                let color = Color32::from_rgba_unmultiplied(
                    p.accent.r(),
                    p.accent.g(),
                    p.accent.b(),
                    (alpha * 255.0) as u8,
                );
                painter.circle_filled(dot_pos, dot_radius, color);
            }

            painter.text(
                Pos2::new(center.x, rect.max.y - 8.0),
                egui::Align2::CENTER_CENTER,
                "Busy Spinner",
                FontId::proportional(9.0),
                p.text_dim,
            );
        }
        WidgetDef::SweepingArc {
            start_angle,
            end_angle,
            ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(4), p.card_bg);
            let center = rect.center();
            let radius = (rect.width().min(rect.height()) / 2.0 - 10.0).max(8.0);

            let pulse = (time * 2.5).sin() * 0.5 + 0.5;
            let sweep_end = *start_angle as f32 + (*end_angle - *start_angle) as f32 * pulse;

            painter.circle_stroke(center, radius, Stroke::new(2.0f32, p.border));

            let mut a = *start_angle as f32;
            while a <= sweep_end {
                let rad = a.to_radians();
                let pt = center + Vec2::new(rad.cos() * radius, rad.sin() * radius);
                painter.circle_filled(pt, 2.5, p.accent);
                a += 15.0;
            }

            painter.text(
                Pos2::new(center.x, center.y),
                egui::Align2::CENTER_CENTER,
                format!("{:.0}°", sweep_end),
                FontId::proportional(11.0),
                p.text_primary,
            );
        }
        WidgetDef::Plotter { mode, .. } => {
            painter.rect_filled(rect, CornerRadius::same(3), p.display_bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(3),
                Stroke::new(1.0f32, p.border),
                StrokeKind::Inside,
            );

            let num_v = 6;
            for i in 1..num_v {
                let x = rect.min.x + (rect.width() * i as f32 / num_v as f32);
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    Stroke::new(0.8f32, p.border),
                );
            }
            let num_h = 4;
            for i in 1..num_h {
                let y = rect.min.y + (rect.height() * i as f32 / num_h as f32);
                painter.line_segment(
                    [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                    Stroke::new(0.8f32, p.border),
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
                painter.line_segment([win[0], win[1]], Stroke::new(1.8f32, p.success));
            }

            painter.text(
                Pos2::new(rect.min.x + 6.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                "CH1: 1.00kHz (Live Scope)",
                FontId::proportional(9.0),
                p.success,
            );
        }
        WidgetDef::StatusBar { time: time_str, .. } => {
            painter.rect_filled(rect, CornerRadius::ZERO, p.card_bg);
            painter.text(
                Pos2::new(rect.min.x + 8.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                "📶  🔋 98%",
                FontId::proportional(10.0),
                p.text_dim,
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
                p.text_primary,
            );
        }
        WidgetDef::Panel { style, .. } => {
            let bg = if style.as_deref() == Some("card") {
                p.card_bg
            } else {
                p.display_bg
            };
            painter.rect_filled(rect, CornerRadius::same(4), bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, p.border),
                StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Card Panel",
                FontId::proportional(11.0),
                p.text_dim,
            );
        }
        WidgetDef::Dropdown {
            options, selected, ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(3), p.card_bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(3),
                Stroke::new(1.0f32, p.border),
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
                p.text_primary,
            );
        }
        WidgetDef::Roller {
            options, selected, ..
        } => {
            painter.rect_filled(rect, CornerRadius::same(4), p.display_bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0f32, p.border),
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
                    p.text_dim,
                );

                // Active / selected option (highlighted center row)
                let cur_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 2.0, rect.min.y + row_h),
                    Vec2::new(rect.width() - 4.0, row_h),
                );
                painter.rect_filled(cur_rect, CornerRadius::same(2), p.accent);
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
                    p.text_dim,
                );
            }
        }
        WidgetDef::Spacer => {}
        WidgetDef::RectShape {
            radius,
            stroke_width,
            fill_color,
            stroke_color,
            ..
        } => {
            let fill = fill_color
                .as_deref()
                .and_then(parse_color_hex)
                .unwrap_or(p.card_bg);
            let stroke_c = stroke_color
                .as_deref()
                .and_then(parse_color_hex)
                .unwrap_or(p.border);
            let cr = CornerRadius::same(*radius);
            if *stroke_width > 0 {
                painter.rect(
                    rect,
                    cr,
                    fill,
                    Stroke::new(*stroke_width as f32, stroke_c),
                    StrokeKind::Inside,
                );
            } else {
                painter.rect_filled(rect, cr, fill);
            }
        }
        WidgetDef::LineShape {
            stroke_width,
            color,
            ..
        } => {
            let col = color
                .as_deref()
                .and_then(parse_color_hex)
                .unwrap_or(p.border);
            painter.line_segment(
                [
                    Pos2::new(rect.min.x, rect.center().y),
                    Pos2::new(rect.max.x, rect.center().y),
                ],
                Stroke::new(*stroke_width as f32, col),
            );
        }
        WidgetDef::CircleShape {
            stroke_width,
            fill_color,
            stroke_color,
            ..
        } => {
            let fill = fill_color
                .as_deref()
                .and_then(parse_color_hex)
                .unwrap_or(p.card_bg);
            let stroke_c = stroke_color
                .as_deref()
                .and_then(parse_color_hex)
                .unwrap_or(p.border);
            let r = rect.width().min(rect.height()) / 2.0;
            if *stroke_width > 0 {
                painter.circle(
                    rect.center(),
                    r,
                    fill,
                    Stroke::new(*stroke_width as f32, stroke_c),
                );
            } else {
                painter.circle_filled(rect.center(), r, fill);
            }
        }
        WidgetDef::VectorPath {
            stroke_width,
            verbs,
            ..
        } => {
            let stroke = Stroke::new(*stroke_width as f32, p.accent);
            let mut current_pos = rect.min;
            for v in verbs {
                match v {
                    embedded_gui_codegen::PathVerbDef::MoveTo(x, y) => {
                        current_pos = Pos2::new(rect.min.x + *x as f32, rect.min.y + *y as f32);
                    }
                    embedded_gui_codegen::PathVerbDef::LineTo(x, y) => {
                        let next_pos = Pos2::new(rect.min.x + *x as f32, rect.min.y + *y as f32);
                        painter.line_segment([current_pos, next_pos], stroke);
                        current_pos = next_pos;
                    }
                    embedded_gui_codegen::PathVerbDef::QuadTo(cx, cy, x, y) => {
                        let cp = Pos2::new(rect.min.x + *cx as f32, rect.min.y + *cy as f32);
                        let ep = Pos2::new(rect.min.x + *x as f32, rect.min.y + *y as f32);
                        painter.add(egui::Shape::QuadraticBezier(
                            egui::epaint::QuadraticBezierShape::from_points_stroke(
                                [current_pos, cp, ep],
                                false,
                                Color32::TRANSPARENT,
                                stroke,
                            ),
                        ));
                        current_pos = ep;
                    }
                    embedded_gui_codegen::PathVerbDef::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                        let c1 = Pos2::new(rect.min.x + *c1x as f32, rect.min.y + *c1y as f32);
                        let c2 = Pos2::new(rect.min.x + *c2x as f32, rect.min.y + *c2y as f32);
                        let ep = Pos2::new(rect.min.x + *x as f32, rect.min.y + *y as f32);
                        painter.add(egui::Shape::CubicBezier(
                            egui::epaint::CubicBezierShape::from_points_stroke(
                                [current_pos, c1, c2, ep],
                                false,
                                Color32::TRANSPARENT,
                                stroke,
                            ),
                        ));
                        current_pos = ep;
                    }
                    embedded_gui_codegen::PathVerbDef::Close => {}
                }
            }
        }
        _ => {
            painter.rect_filled(rect, CornerRadius::same(3), p.card_bg);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                widget.id().unwrap_or("widget"),
                FontId::proportional(10.0),
                p.text_dim,
            );
        }
    }
}

fn parse_color_hex(hex: &str) -> Option<Color32> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else {
        None
    }
}
