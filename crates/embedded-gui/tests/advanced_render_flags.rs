//! Render chart/plotter/scale widgets with all decoration and style flags.

use embedded_gui::prelude::*;

mod common;
use common::MockTarget;

#[test]
fn renders_chart_and_plotter_with_decorations() {
    static VALUES: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

    let mut gui = GuiContext::<16, 8, 8>::new(Rect::new(0, 0, 200, 200));
    let mut y = 0u32;
    let mut rect = |height: u32| {
        let r = Rect::new(0, y as i32, 200, height);
        y += height + 2;
        r
    };

    let line = gui
        .add_chart(rect(60), &VALUES, 0.0, 1.0, Style::panel())
        .unwrap();
    gui.set_chart_style(line, 3, true, true).unwrap();
    gui.set_chart_decoration(line, ChartMode::Bars, true, true, true)
        .unwrap();

    let bars = gui
        .add_chart(rect(60), &VALUES, 0.0, 1.0, Style::panel())
        .unwrap();
    gui.set_chart_style(bars, 2, false, false).unwrap();
    gui.set_chart_decoration(bars, ChartMode::Line, false, false, false)
        .unwrap();

    let plotter = gui
        .add_plotter(rect(50), &VALUES, 4, 0.0, 1.0, Style::panel())
        .unwrap();
    gui.set_plotter_style(plotter, 2).unwrap();
    gui.set_plotter_decoration(plotter, true, true).unwrap();

    gui.add_scale(rect(60), ScaleMode::Radial, 0.0, 10.0, 5.0, Style::panel())
        .unwrap();
    gui.add_scale(
        rect(40),
        ScaleMode::LinearHorizontal,
        0.0,
        10.0,
        5.0,
        Style::panel(),
    )
    .unwrap();
    gui.add_scale(
        rect(80),
        ScaleMode::LinearVertical,
        0.0,
        10.0,
        5.0,
        Style::panel(),
    )
    .unwrap();

    let mut target = MockTarget::new(200, 200);
    gui.render(&mut target).unwrap();
    assert!(!target.pixels.is_empty());
}
