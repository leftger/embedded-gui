//! Minimal WebAssembly demo: render an `embedded-gui` screen into an HTML
//! canvas using the same RGB565 `GuiContext` pipeline used on embedded
//! targets. This is the "same UI in the browser" bridge recommended in the
//! LVGL community discussion: the retained widget tree and renderer stay
//! firmware-compatible while the target is a canvas instead of a panel.

use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_web_simulator::display::WebSimulatorDisplay;
use embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder;
use embedded_gui::prelude::*;
use wasm_bindgen::prelude::*;

const W: u32 = 320;
const H: u32 = 240;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let settings = OutputSettingsBuilder::new().scale(2).build();
    let mut display = WebSimulatorDisplay::<Rgb565>::new((W, H), &settings, None);

    // The same fixed-capacity context type a no_std firmware uses.
    let mut gui = GuiContext::<32, 16, 16>::new(Rect::new(0, 0, W, H));

    gui.add_panel(Rect::new(8, 8, W - 16, H - 16), Style::panel())
        .map_err(js_error)?;
    gui.add_label(
        Rect::new(24, 24, 240, 16),
        "embedded-gui in the browser",
        Style::label(),
    )
    .map_err(js_error)?;
    gui.add_button(Rect::new(24, 56, 120, 32), "SAME WIDGETS", Style::button())
        .map_err(js_error)?;
    gui.add_progress_bar(Rect::new(24, 112, 272, 14), 0.68, Style::progress())
        .map_err(js_error)?;
    gui.add_slider(
        Rect::new(24, 144, 272, 20),
        0.42,
        0.0,
        1.0,
        Style::progress(),
    )
    .map_err(js_error)?;
    gui.add_label(
        Rect::new(24, 196, 272, 12),
        "No_std core, firmware-compatible renderer",
        Style::label(),
    )
    .map_err(js_error)?;

    gui.render(&mut display).map_err(js_error)?;
    display.flush()?;
    Ok(())
}

fn js_error(error: impl core::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}
