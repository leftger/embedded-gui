//! Tests for embedded-dsp integration in embedded-gui.

#[cfg(feature = "embedded-dsp")]
#[test]
fn test_dsp_touch_input_filter() {
    use embedded_gui::TouchInputFilter;

    let mut filter = TouchInputFilter::new(0.1);

    // Initial point
    let (s_x1, _s_y1) = filter.filter(100.0, 200.0);
    assert!((s_x1 - 100.0).abs() < 50.0);

    // Sudden noise spike (e.g. 500, 500) — low-pass filter damps it
    let (s_x2, s_y2) = filter.filter(500.0, 500.0);
    assert!(s_x2 < 400.0, "low-pass filter damps sudden spike");
    assert!(s_y2 < 400.0, "low-pass filter damps sudden spike");
}

#[cfg(feature = "embedded-dsp")]
#[test]
fn test_dsp_spectrum_analyzer_widget() {
    use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
    use embedded_gui::{Framebuffer, Rect, RenderCtx, SpectrumAnalyzerWidget};

    let mut fb = Framebuffer::<1600>::new(40, 40);
    fb.clear_color(Rgb565::BLACK);

    let samples: [f32; 16] = [
        0.1, 0.5, 0.9, 0.2, 0.4, 0.8, 0.3, 0.6, 0.2, 0.7, 0.5, 0.1, 0.3, 0.6, 0.8, 0.4,
    ];
    let analyzer = SpectrumAnalyzerWidget::new(Rect::new(0, 0, 40, 40), &samples);

    let mut ctx = RenderCtx::compositing(&mut fb, Rect::new(0, 0, 40, 40));
    analyzer.draw(&mut ctx).unwrap();
}
