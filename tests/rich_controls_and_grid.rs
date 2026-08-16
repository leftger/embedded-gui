use embedded_graphics_core::{
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor, WebColors},
};
use embedded_gui::{
    EdgeInsets, Framebuffer, GridLayout, GridPlacement, GridTrack, GuiContext, PathVerb,
    PropertyKey, PropertyValue, Rect, RenderCtx, ScaleWidget, SpinboxWidget, StrokeStyle, Style,
    TableWidget, VectorPath,
};

#[test]
fn test_grid_layout_fractional_and_fixed_tracks() {
    let grid = GridLayout::<4, 3>::new(
        [
            GridTrack::Px(40),
            GridTrack::Fr(1),
            GridTrack::Fr(2),
            GridTrack::Auto,
        ],
        [GridTrack::Px(20), GridTrack::Fr(1), GridTrack::Px(30)],
    )
    .with_gap(4)
    .with_padding(EdgeInsets::all(8));

    let container = Rect::new(0, 0, 320, 240);
    let placements = [
        GridPlacement::cell(0, 0),       // Top-left fixed 40x20
        GridPlacement::span(1, 0, 3, 1), // Top row spanning 3 cols
        GridPlacement::cell(0, 1),       // Middle left
        GridPlacement::span(1, 1, 2, 2), // 2x2 area
    ];
    let mut out = [Rect::empty(); 4];

    let count = grid.arrange_cells(container, &placements, &mut out);
    assert_eq!(count, 4);

    assert_eq!(out[0].x, 8);
    assert_eq!(out[0].y, 8);
    assert_eq!(out[0].w, 40);
    assert_eq!(out[0].h, 20);

    // Placement 1 starts at col 1 (8 + 40 + 4 = 52)
    assert_eq!(out[1].x, 52);
    assert_eq!(out[1].y, 8);
    assert_eq!(out[1].h, 20);
}

#[test]
fn test_vector_path_and_bezier_curves() {
    let mut fb = Framebuffer::<{ 100 * 100 }>::new(100, 100);
    fb.clear_color(Rgb565::BLACK);

    let mut path = VectorPath::<16>::new();
    path.move_to(Point::new(10, 10))
        .line_to(Point::new(40, 10))
        .quad_to(Point::new(70, 30), Point::new(40, 60))
        .cubic_to(Point::new(30, 80), Point::new(60, 90), Point::new(80, 80))
        .close();

    assert_eq!(path.len(), 5);
    assert_eq!(path.verbs()[0], PathVerb::MoveTo(Point::new(10, 10)));
    assert_eq!(path.verbs()[4], PathVerb::Close);

    {
        let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 100, 100));
        let style = StrokeStyle::new(Rgb565::CSS_CYAN).with_width(2);
        ctx.draw_vector_path(&path, style).unwrap();
    }

    // Verify that curve pixels were rendered into the framebuffer
    let rendered_pixels = fb.pixels().iter().filter(|&&c| c != Rgb565::BLACK).count();
    assert!(rendered_pixels > 50);
}

#[test]
fn test_spinbox_digit_navigation_and_precision() {
    let mut spinbox = SpinboxWidget::new(0, 9999, 1250)
        .with_digits(4)
        .with_decimals(2);

    let mut text = heapless::String::<16>::new();
    spinbox.format_text(&mut text);
    assert_eq!(text.as_str(), "12.50");

    // Focused on least significant digit (index 0 -> 0.01s place)
    spinbox.increment();
    assert_eq!(spinbox.value, 1251);

    // Move to 10s place (index 1 -> 0.1s place)
    spinbox.prev_digit();
    spinbox.increment();
    assert_eq!(spinbox.value, 1261);

    // Move to 100s place (index 2 -> 1.0s place)
    spinbox.prev_digit();
    spinbox.decrement();
    assert_eq!(spinbox.value, 1161);

    spinbox.format_text(&mut text);
    assert_eq!(text.as_str(), "11.61");
}

#[test]
fn test_table_widget_and_2d_grid_navigation() {
    let data: &[&[&str]] = &[
        &["CPU", "35%", "Normal"],
        &["MEM", "62%", "Normal"],
        &["GPU", "88%", "High"],
    ];
    let headers: &[&str] = &["Metric", "Usage", "Status"];

    let mut table = TableWidget::new(data)
        .with_headers(headers)
        .with_selection(0, 0);

    assert_eq!(table.selected, Some((0, 0)));

    // Navigate right
    table.move_cursor(0, 1);
    assert_eq!(table.selected, Some((0, 1)));

    // Navigate down
    table.move_cursor(1, 0);
    assert_eq!(table.selected, Some((1, 1)));

    // Render table to framebuffer
    let mut fb = Framebuffer::<{ 120 * 80 }>::new(120, 80);
    let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 120, 80));
    table
        .render(
            &mut ctx,
            Rect::new(0, 0, 120, 80),
            Style::default().into(),
            embedded_gui::VisualState::Normal,
        )
        .unwrap();

    let non_black = fb.pixels().iter().filter(|&&c| c != Rgb565::BLACK).count();
    assert!(non_black > 0);
}

#[test]
fn test_scale_widget_render_and_properties() {
    let mut scale = ScaleWidget::new(0.0, 100.0, 45.0)
        .with_ticks(4, 2)
        .with_angles(180, 0)
        .with_needle(true, Rgb565::CSS_RED);

    let mut fb = Framebuffer::<{ 100 * 100 }>::new(100, 100);
    let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 100, 100));
    scale
        .render(
            &mut ctx,
            Rect::new(0, 0, 100, 100),
            Style::default().into(),
            embedded_gui::VisualState::Normal,
        )
        .unwrap();

    // Verify property mutation
    let _ = embedded_gui::widget::Widget::set_property(
        &mut scale,
        PropertyKey::Value,
        PropertyValue::Float(75.0),
    );
    assert_eq!(scale.value, 75.0);

    // Linear horizontal scale
    let lin_scale = ScaleWidget::linear_horizontal(0.0, 50.0, 25.0);
    lin_scale
        .render(
            &mut ctx,
            Rect::new(0, 0, 100, 40),
            Style::default().into(),
            embedded_gui::VisualState::Normal,
        )
        .unwrap();
}

#[test]
fn test_gui_context_new_widgets_integration() {
    let mut gui = GuiContext::<16, 4, 8>::new(Rect::new(0, 0, 240, 240));

    let table_rows: &[&[&str]] = &[&["A1", "B1"], &["A2", "B2"]];
    let table_id = gui
        .add_table(Rect::new(10, 10, 100, 60), table_rows, Style::default())
        .unwrap();
    assert!(table_id.0 < 16);

    let scale_id = gui
        .add_radial_scale(
            Rect::new(10, 80, 80, 80),
            0.0,
            100.0,
            50.0,
            Style::default(),
        )
        .unwrap();
    assert!(scale_id.0 < 16);

    let spinbox_id = gui
        .add_spinbox(Rect::new(10, 170, 90, 25), 0, 999, 120, Style::default())
        .unwrap();
    assert!(spinbox_id.0 < 16);
}
