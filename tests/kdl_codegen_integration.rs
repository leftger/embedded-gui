use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_gui::{Framebuffer, GuiContext, Rect};
use embedded_gui_macros::include_gui;

// 1. Compile the external KDL screen into typed Rust code at compile time
include_gui!("examples/ui/smart_thermostat.kdl");

#[test]
fn test_generated_kdl_screen_initialization_and_render() {
    let mut gui = GuiContext::<32, 8, 8>::new(Rect::new(
        0,
        0,
        SmartThermostatApp::WIDTH,
        SmartThermostatApp::HEIGHT,
    ));

    // 2. Build the entire UI using the auto-generated struct
    let app = SmartThermostatApp::build(&mut gui).expect("Failed to build generated KDL screen");

    // 3. Verify all typed widget IDs were generated and mapped
    assert!(app.widgets.temp_setpoint.0 < 32);
    assert!(app.widgets.room_gauge.0 < 32);
    assert!(app.widgets.fan_btn.0 < 32);
    assert!(app.widgets.eco_mode.0 < 32);

    // 4. Render to framebuffer and verify non-empty render
    let mut fb = Framebuffer::<{ 320 * 240 }>::new(320, 240);
    fb.clear_color(Rgb565::BLACK);
    gui.render(&mut fb).expect("Failed to render generated GUI");

    let rendered_pixels = fb.pixels().iter().filter(|&&c| c != Rgb565::BLACK).count();
    assert!(
        rendered_pixels > 500,
        "Expected rendered screen to contain UI pixels"
    );
}
