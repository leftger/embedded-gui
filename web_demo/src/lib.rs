//! Interactive WebAssembly demo: render an `embedded-gui` screen into an HTML
//! canvas and feed browser pointer/keyboard events into the same input pipeline
//! used by firmware.
//!
//! This demonstrates the full loop the community asked for:
//! one retained widget tree + input contract, running on MCU firmware *and* in
//! the browser with no UI rewrite.

use std::cell::RefCell;
use std::rc::Rc;

use embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_web_simulator::display::WebSimulatorDisplay;
use embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder;
use embedded_gui::prelude::*;
use embedded_gui::{PropertyKey, PropertyValue};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, KeyboardEvent, PointerEvent};

const W: u32 = 320;
const H: u32 = 240;

struct App {
    gui: GuiContext<'static, 48, 32, 24>,
    display: WebSimulatorDisplay<Rgb565>,
    canvas: HtmlCanvasElement,
    button: WidgetId,
    toggle: WidgetId,
    slider: WidgetId,
    progress: WidgetId,
    clicks_label: WidgetId,
    slider_label: WidgetId,
    status_label: WidgetId,
    pointer_down: bool,
    clicks: i32,
    toggle_on: bool,
}

impl App {
    fn new() -> Result<Self, JsValue> {
        let settings = OutputSettingsBuilder::new().scale(2).build();
        let display = WebSimulatorDisplay::<Rgb565>::new((W, H), &settings, None);

        let document = web_sys::window()
            .ok_or_else(|| JsValue::from_str("no window"))?
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let canvas: HtmlCanvasElement = document
            .query_selector("canvas")
            .map_err(js_error)?
            .ok_or_else(|| JsValue::from_str("canvas not found"))?
            .dyn_into()
            .map_err(js_error)?;

        let mut gui = GuiContext::<48, 32, 24>::new(Rect::new(0, 0, W, H));

        gui.add_panel(Rect::new(8, 8, W - 16, H - 16), Style::panel())
            .map_err(js_error)?;
        gui.add_label(
            Rect::new(24, 20, 272, 14),
            "embedded-gui interactive browser demo",
            Style::label(),
        )
        .map_err(js_error)?;

        let button = gui
            .add_button(Rect::new(24, 48, 140, 32), "CLICK ME", Style::button())
            .map_err(js_error)?;
        let toggle = gui
            .add_toggle(Rect::new(24, 96, 150, 16), "ENABLE", false, Style::button())
            .map_err(js_error)?;
        let slider = gui
            .add_slider(
                Rect::new(24, 130, 272, 18),
                0.5,
                0.0,
                1.0,
                Style::progress(),
            )
            .map_err(js_error)?;

        let progress = gui
            .add_progress_bar(Rect::new(24, 168, 272, 12), 0.0, Style::progress())
            .map_err(js_error)?;
        let clicks_label = gui
            .add_value_label(Rect::new(24, 196, 100, 16), "CLICKS", 0, Style::panel())
            .map_err(js_error)?;
        let slider_label = gui
            .add_value_label(Rect::new(150, 196, 146, 16), "SLIDER", 50, Style::panel())
            .map_err(js_error)?;
        let status_label = gui
            .add_label(
                Rect::new(24, 218, 272, 12),
                "Pointer: click/toggle/slider  Keyboard: arrows+Enter",
                Style::label(),
            )
            .map_err(js_error)?;

        Ok(Self {
            gui,
            display,
            canvas,
            button,
            toggle,
            slider,
            progress,
            clicks_label,
            slider_label,
            status_label,
            pointer_down: false,
            clicks: 0,
            toggle_on: false,
        })
    }

    fn pointer_pos(&self, client_x: i32, client_y: i32) -> (i32, i32) {
        let rect = self.canvas.get_bounding_client_rect();
        let x = ((client_x as f64 - rect.left()) / rect.width() * W as f64) as i32;
        let y = ((client_y as f64 - rect.top()) / rect.height() * H as f64) as i32;
        (x.clamp(0, W as i32 - 1), y.clamp(0, H as i32 - 1))
    }

    fn handle_input(&mut self, event: InputEvent) {
        let _ = self.gui.handle_input(event);
        let _ = self.gui.tick_input(1);
        self.drain_events();
        self.redraw();
    }

    fn drain_events(&mut self) {
        while let Some(event) = self.gui.pop_event() {
            self.on_ui_event(event);
        }
    }

    fn on_ui_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::Activate(id) if id == self.button => {
                self.clicks = self.clicks.saturating_add(1);
                let _ = self.gui.set_value_label(self.clicks_label, self.clicks);
                let _ = self
                    .gui
                    .set_progress(self.progress, (self.clicks as f32 / 10.0).min(1.0));
                self.set_status("BUTTON CLICKED");
            }
            UiEvent::ValueChanged(id) if id == self.toggle => {
                self.toggle_on = !self.toggle_on;
                self.set_status(if self.toggle_on {
                    "TOGGLE ON"
                } else {
                    "TOGGLE OFF"
                });
            }
            UiEvent::ValueChanged(id) if id == self.slider => {
                let value = match self.gui.get_widget_property(id, PropertyKey::Value) {
                    Some(PropertyValue::Float(value)) => value,
                    _ => 0.0,
                };
                let _ = self
                    .gui
                    .set_value_label(self.slider_label, (value * 100.0).round() as i32);
            }
            _ => {}
        }
    }

    fn set_status(&mut self, text: &'static str) {
        let _ = self.gui.set_widget_property(
            self.status_label,
            PropertyKey::Text,
            PropertyValue::Str(text),
        );
    }

    fn redraw(&mut self) {
        let _ = self.display.clear(Rgb565::BLACK);
        let _ = self.gui.render(&mut self.display);
        let _ = self.display.flush();
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let app = Rc::new(RefCell::new(App::new()?));
    app.borrow_mut().redraw();

    // Pointer input: primary button press/drag/release on the canvas.
    {
        let down_app = Rc::clone(&app);
        let pointer_down =
            Closure::<dyn FnMut(PointerEvent)>::wrap(Box::new(move |event: PointerEvent| {
                if event.button() == 0 {
                    let mut app = down_app.borrow_mut();
                    let (x, y) = app.pointer_pos(event.client_x(), event.client_y());
                    app.pointer_down = true;
                    app.handle_input(InputEvent::Pointer {
                        x,
                        y,
                        state: PointerState::Pressed,
                        button: PointerButton::Primary,
                    });
                }
            }));

        let move_app = Rc::clone(&app);
        let pointer_move =
            Closure::<dyn FnMut(PointerEvent)>::wrap(Box::new(move |event: PointerEvent| {
                let mut app = move_app.borrow_mut();
                if app.pointer_down {
                    let (x, y) = app.pointer_pos(event.client_x(), event.client_y());
                    app.handle_input(InputEvent::Pointer {
                        x,
                        y,
                        state: PointerState::Moved,
                        button: PointerButton::Primary,
                    });
                }
            }));

        let up_app = Rc::clone(&app);
        let pointer_up =
            Closure::<dyn FnMut(PointerEvent)>::wrap(Box::new(move |event: PointerEvent| {
                if event.button() == 0 {
                    let mut app = up_app.borrow_mut();
                    app.pointer_down = false;
                    let (x, y) = app.pointer_pos(event.client_x(), event.client_y());
                    app.handle_input(InputEvent::Pointer {
                        x,
                        y,
                        state: PointerState::Released,
                        button: PointerButton::Primary,
                    });
                }
            }));

        let canvas = &app.borrow().canvas.clone();
        canvas
            .add_event_listener_with_callback("pointerdown", pointer_down.as_ref().unchecked_ref())
            .map_err(js_error)?;
        canvas
            .add_event_listener_with_callback("pointermove", pointer_move.as_ref().unchecked_ref())
            .map_err(js_error)?;
        canvas
            .add_event_listener_with_callback("pointerup", pointer_up.as_ref().unchecked_ref())
            .map_err(js_error)?;
        canvas
            .add_event_listener_with_callback("pointercancel", pointer_up.as_ref().unchecked_ref())
            .map_err(js_error)?;

        pointer_down.forget();
        pointer_move.forget();
        pointer_up.forget();
    }

    // Keyboard input: spatial navigation + select/back mapped to InputEvent.
    {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let key_app = Rc::clone(&app);
        let keydown =
            Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(move |event: KeyboardEvent| {
                let Some(input) = keyboard_input(event.key().as_str()) else {
                    return;
                };
                event.prevent_default();
                key_app.borrow_mut().handle_input(input);
            }));
        window
            .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
            .map_err(js_error)?;
        keydown.forget();
    }

    Ok(())
}

fn keyboard_input(key: &str) -> Option<InputEvent> {
    match key {
        "ArrowUp" => Some(InputEvent::Up),
        "ArrowDown" => Some(InputEvent::Down),
        "ArrowLeft" => Some(InputEvent::Left),
        "ArrowRight" => Some(InputEvent::Right),
        "Enter" | " " => Some(InputEvent::Select),
        "Escape" | "Backspace" => Some(InputEvent::Back),
        _ => None,
    }
}

fn js_error(error: impl core::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}
