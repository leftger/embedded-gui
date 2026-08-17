//! Custom lightweight KDL syntax highlighter for egui multiline editor.

use eframe::egui::{self, Color32, FontId, TextFormat};

/// Formats KDL source code with syntax highlighting into an `egui::text::LayoutJob`.
pub fn highlight_kdl(ctx: &egui::Context, text: &str) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let default_font = FontId::monospace(12.5);

    let is_dark = ctx.style().visuals.dark_mode;

    let col_comment = if is_dark {
        Color32::from_rgb(110, 120, 135)
    } else {
        Color32::from_rgb(130, 140, 150)
    };
    let col_tag = if is_dark {
        Color32::from_rgb(255, 120, 150)
    } else {
        Color32::from_rgb(210, 40, 80)
    };
    let col_prop_key = if is_dark {
        Color32::from_rgb(120, 200, 255)
    } else {
        Color32::from_rgb(20, 120, 220)
    };
    let col_string = if is_dark {
        Color32::from_rgb(140, 225, 140)
    } else {
        Color32::from_rgb(35, 145, 55)
    };
    let col_number = if is_dark {
        Color32::from_rgb(255, 195, 100)
    } else {
        Color32::from_rgb(200, 120, 20)
    };
    let col_bracket = if is_dark {
        Color32::from_rgb(200, 210, 225)
    } else {
        Color32::from_rgb(60, 70, 85)
    };
    let col_text = if is_dark {
        Color32::from_rgb(220, 225, 235)
    } else {
        Color32::from_rgb(30, 35, 45)
    };

    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            job.append(
                line,
                0.0,
                TextFormat {
                    font_id: default_font.clone(),
                    color: col_comment,
                    ..Default::default()
                },
            );
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: default_font.clone(),
                    color: col_comment,
                    ..Default::default()
                },
            );
            continue;
        }

        let mut in_string = false;
        let mut string_buf = String::new();
        let mut word_buf = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if in_string {
                string_buf.push(ch);
                if ch == '"' && (i == 0 || chars[i - 1] != '\\') {
                    in_string = false;
                    job.append(
                        &string_buf,
                        0.0,
                        TextFormat {
                            font_id: default_font.clone(),
                            color: col_string,
                            ..Default::default()
                        },
                    );
                    string_buf.clear();
                }
                i += 1;
                continue;
            }

            if ch == '"' {
                // Flush previous word
                if !word_buf.is_empty() {
                    let color =
                        classify_word(&word_buf, col_tag, col_prop_key, col_number, col_text);
                    job.append(
                        &word_buf,
                        0.0,
                        TextFormat {
                            font_id: default_font.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    word_buf.clear();
                }
                in_string = true;
                string_buf.push(ch);
                i += 1;
                continue;
            }

            if ch == '{' || ch == '}' || ch == ';' || ch == '(' || ch == ')' {
                if !word_buf.is_empty() {
                    let color =
                        classify_word(&word_buf, col_tag, col_prop_key, col_number, col_text);
                    job.append(
                        &word_buf,
                        0.0,
                        TextFormat {
                            font_id: default_font.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    word_buf.clear();
                }
                job.append(
                    &ch.to_string(),
                    0.0,
                    TextFormat {
                        font_id: default_font.clone(),
                        color: col_bracket,
                        ..Default::default()
                    },
                );
                i += 1;
                continue;
            }

            if ch == '=' {
                if !word_buf.is_empty() {
                    job.append(
                        &word_buf,
                        0.0,
                        TextFormat {
                            font_id: default_font.clone(),
                            color: col_prop_key,
                            ..Default::default()
                        },
                    );
                    word_buf.clear();
                }
                job.append(
                    "=",
                    0.0,
                    TextFormat {
                        font_id: default_font.clone(),
                        color: col_bracket,
                        ..Default::default()
                    },
                );
                i += 1;
                continue;
            }

            if ch.is_whitespace() {
                if !word_buf.is_empty() {
                    let color =
                        classify_word(&word_buf, col_tag, col_prop_key, col_number, col_text);
                    job.append(
                        &word_buf,
                        0.0,
                        TextFormat {
                            font_id: default_font.clone(),
                            color,
                            ..Default::default()
                        },
                    );
                    word_buf.clear();
                }
                job.append(
                    &ch.to_string(),
                    0.0,
                    TextFormat {
                        font_id: default_font.clone(),
                        color: col_text,
                        ..Default::default()
                    },
                );
                i += 1;
                continue;
            }

            word_buf.push(ch);
            i += 1;
        }

        if in_string && !string_buf.is_empty() {
            job.append(
                &string_buf,
                0.0,
                TextFormat {
                    font_id: default_font.clone(),
                    color: col_string,
                    ..Default::default()
                },
            );
        } else if !word_buf.is_empty() {
            let color = classify_word(&word_buf, col_tag, col_prop_key, col_number, col_text);
            job.append(
                &word_buf,
                0.0,
                TextFormat {
                    font_id: default_font.clone(),
                    color,
                    ..Default::default()
                },
            );
        }

        job.append(
            "\n",
            0.0,
            TextFormat {
                font_id: default_font.clone(),
                color: col_text,
                ..Default::default()
            },
        );
    }

    job
}

fn classify_word(
    word: &str,
    tag_col: Color32,
    _key_col: Color32,
    num_col: Color32,
    text_col: Color32,
) -> Color32 {
    match word {
        "screen" | "grid" | "label" | "button" | "toggle" | "checkbox" | "slider" | "progress"
        | "scale" | "plotter" | "busy_wheel" | "status_bar" | "panel" | "roller" | "dropdown"
        | "sweeping_arc" | "spacer" | "option" => tag_col,
        "true" | "false" => tag_col,
        s if s
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+') =>
        {
            num_col
        }
        s if s.ends_with("px") || s.ends_with("fr") || s.ends_with("%") => num_col,
        _ => text_col,
    }
}
