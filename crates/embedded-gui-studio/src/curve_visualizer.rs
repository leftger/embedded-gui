//! Interactive Animation Easing Curve visualizer and graph plotter.

use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Stroke, StrokeKind, Vec2};
use embedded_gui::motion::timing::{EasingCurve, evaluate_easing};

/// Renders a live visual graph of the selected easing curve with an active animated tracer head.
pub fn render_curve_graph(ui: &mut egui::Ui, curve: EasingCurve, norm_t: f32, size: Vec2) {
    let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
    let rect = response.rect;

    // Background and frame
    painter.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(18, 20, 26));
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0f32, Color32::from_rgb(45, 52, 65)),
        StrokeKind::Inside,
    );

    // Padding inside graph
    let graph_rect = rect.shrink2(Vec2::new(12.0, 10.0));

    // Diagonal reference guide (Linear baseline)
    painter.line_segment(
        [
            Pos2::new(graph_rect.min.x, graph_rect.max.y),
            Pos2::new(graph_rect.max.x, graph_rect.min.y),
        ],
        Stroke::new(1.0f32, Color32::from_rgba_unmultiplied(80, 90, 110, 80)),
    );

    // Sample and plot curve points
    let steps = 60;
    let mut pts = Vec::with_capacity(steps);
    for i in 0..=steps {
        let x_norm = i as f32 / steps as f32;
        let y_val = evaluate_easing(curve, x_norm);
        let px = graph_rect.min.x + x_norm * graph_rect.width();
        let py = graph_rect.max.y - y_val * graph_rect.height();
        pts.push(Pos2::new(px, py));
    }

    // Draw curve spline line
    for w in pts.windows(2) {
        painter.line_segment(
            [w[0], w[1]],
            Stroke::new(2.0f32, Color32::from_rgb(60, 180, 255)),
        );
    }

    // Current animated tracer bead
    let cur_y_val = evaluate_easing(curve, norm_t.clamp(0.0, 1.0));
    let bead_x = graph_rect.min.x + norm_t.clamp(0.0, 1.0) * graph_rect.width();
    let bead_y = graph_rect.max.y - cur_y_val * graph_rect.height();
    let bead_pos = Pos2::new(bead_x, bead_y);

    // Vertical playhead line
    painter.line_segment(
        [
            Pos2::new(bead_x, graph_rect.min.y),
            Pos2::new(bead_x, graph_rect.max.y),
        ],
        Stroke::new(1.0f32, Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
    );

    // Glowing tracer head
    painter.circle_filled(bead_pos, 5.5, Color32::from_rgb(255, 200, 60));
    painter.circle_stroke(bead_pos, 5.5, Stroke::new(1.5f32, Color32::WHITE));

    // Curve name label overlay
    painter.text(
        Pos2::new(graph_rect.min.x + 4.0, graph_rect.min.y + 2.0),
        egui::Align2::LEFT_TOP,
        format!("{:?} ({:.0}%)", curve, cur_y_val * 100.0),
        FontId::proportional(9.5),
        Color32::from_rgb(170, 200, 240),
    );
}
