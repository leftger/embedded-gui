//! Interactive kitchen-sink WebAssembly demo.
//!
//! The canvas is a 320x240 "device viewport" into a much taller
//! `GuiContext` workspace. The same RGB565 widget tree and input contract used
//! by firmware is rendered with a scroll offset, so a mouse wheel or drag can
//! browse through many widget categories without leaving the embedded screen
//! paradigm.

use std::cell::RefCell;
use std::rc::Rc;

use embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor, WebColors},
};
use embedded_graphics_web_simulator::display::WebSimulatorDisplay;
use embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder;
use embedded_gui::prelude::*;
use embedded_gui::{PropertyKey, PropertyValue};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, KeyboardEvent, PointerEvent, WheelEvent};

const W: u32 = 320;
const H: u32 = 240;
/// Tall virtual workspace behind the 320x240 device viewport.
const CONTENT_H: u32 = 2400;

static ITEMS: [&str; 5] = ["HOME", "MEDIA", "MAPS", "SETTINGS", "POWER"];
static CAROUSEL_ITEMS: [&str; 7] = ["ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN"];
static ROWS: [&[&str]; 2] = [&["1", "2"], &["3", "4"]];
static KEYS: [char; 12] = ['1', '2', '3', '4', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
static VALUES: [f32; 8] = [1.0, 2.0, 4.0, 3.0, 5.0, 8.0, 6.0, 7.0];
static FEED_ITEMS: [&str; 3] = ["Feed 1", "Feed 2", "Feed 3"];
static TITLES: [&str; 3] = ["Card 1", "Card 2", "Card 3"];
static ACTIONS: [&str; 2] = ["OK", "Cancel"];
static SUGGESTIONS: [&str; 2] = ["alpha", "beta"];
static CELLS: [MenuCell; 3] = [
    MenuCell::new("One").with_subtitle("first"),
    MenuCell::new("Two"),
    MenuCell::new("Three").with_enabled(false),
];
static IMAGE_PX: [u16; 16] = [0xFFFF; 16];

struct Ids {
    button: WidgetId,
    toggle: WidgetId,
    slider: WidgetId,
    clicks_label: WidgetId,
    slider_label: WidgetId,
}

struct App {
    gui: GuiContext<'static, 256, 128, 64>,
    display: WebSimulatorDisplay<Rgb565>,
    canvas: HtmlCanvasElement,
    scroll: i32,
    content_h: u32,
    ids: Ids,
    pointer_down: bool,
    drag_scrolling: bool,
    pointer_start_y: i32,
    scroll_start: i32,
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

        let mut gui = GuiContext::<256, 128, 64>::new(Rect::new(0, 0, W, CONTENT_H));
        let (content_h, ids) = build_ui(&mut gui);

        Ok(Self {
            gui,
            display,
            canvas,
            scroll: 0,
            content_h,
            ids,
            pointer_down: false,
            drag_scrolling: false,
            pointer_start_y: 0,
            scroll_start: 0,
            clicks: 0,
            toggle_on: false,
        })
    }

    fn max_scroll(&self) -> i32 {
        (self.content_h as i32 - H as i32).max(0)
    }

    fn screen_to_content(&self, x: i32, y: i32) -> (i32, i32) {
        (x, y + self.scroll)
    }

    fn pointer_pos(&self, client_x: i32, client_y: i32) -> (i32, i32) {
        let rect = self.canvas.get_bounding_client_rect();
        let x = ((client_x as f64 - rect.left()) / rect.width() * W as f64) as i32;
        let y = ((client_y as f64 - rect.top()) / rect.height() * H as f64) as i32;
        self.screen_to_content(x.clamp(0, W as i32 - 1), y.clamp(0, H as i32 - 1))
    }

    fn handle_input(&mut self, event: InputEvent) {
        let _ = self.gui.handle_input(event);
        let _ = self.gui.tick_input(1);
        self.drain_events();
        self.redraw();
    }

    fn scroll_by(&mut self, delta: i32) {
        self.scroll = (self.scroll + delta).clamp(0, self.max_scroll());
        self.redraw();
    }

    fn drain_events(&mut self) {
        while let Some(event) = self.gui.pop_event() {
            self.on_ui_event(event);
        }
    }

    fn on_ui_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::Activate(id) if id == self.ids.button => {
                self.clicks = self.clicks.saturating_add(1);
                let _ = self.gui.set_value_label(self.ids.clicks_label, self.clicks);
            }
            UiEvent::ValueChanged(id) if id == self.ids.toggle => {
                self.toggle_on = !self.toggle_on;
            }
            UiEvent::ValueChanged(id) if id == self.ids.slider => {
                let value = match self.gui.get_widget_property(id, PropertyKey::Value) {
                    Some(PropertyValue::Float(value)) => value,
                    _ => 0.0,
                };
                let _ = self
                    .gui
                    .set_value_label(self.ids.slider_label, (value * 100.0).round() as i32);
            }
            _ => {}
        }
    }

    fn redraw(&mut self) {
        let _ = self.display.clear(Rgb565::BLACK);
        let _ = self.gui.render_with_offset_opacity_and_clip(
            &mut self.display,
            0,
            -self.scroll,
            255,
            Rect::new(0, 0, W, H),
        );
        let _ = self.display.flush();
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let app = Rc::new(RefCell::new(App::new()?));
    app.borrow_mut().redraw();

    // Pointer input: click/drag widgets or drag to scroll the virtual workspace.
    {
        let down_app = Rc::clone(&app);
        let pointer_down =
            Closure::<dyn FnMut(PointerEvent)>::wrap(Box::new(move |event: PointerEvent| {
                if event.button() == 0 {
                    let mut app = down_app.borrow_mut();
                    let (x, y) = app.pointer_pos(event.client_x(), event.client_y());
                    app.pointer_down = true;
                    app.drag_scrolling = false;
                    app.pointer_start_y = event.client_y();
                    app.scroll_start = app.scroll;
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
                if !app.pointer_down {
                    return;
                }
                let dy = event.client_y() - app.pointer_start_y;
                if app.drag_scrolling || dy.abs() > 6 {
                    app.drag_scrolling = true;
                    app.scroll = (app.scroll_start + dy).clamp(0, app.max_scroll());
                    app.redraw();
                } else {
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
                    app.drag_scrolling = false;
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

    // Wheel input scrolls the virtual workspace.
    {
        let wheel_app = Rc::clone(&app);
        let wheel = Closure::<dyn FnMut(WheelEvent)>::wrap(Box::new(move |event: WheelEvent| {
            event.prevent_default();
            wheel_app.borrow_mut().scroll_by(event.delta_y() as i32);
        }));
        let canvas = &app.borrow().canvas.clone();
        canvas
            .add_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref())
            .map_err(js_error)?;
        wheel.forget();
    }

    // Keyboard input: spatial navigation, select/back, page scroll.
    {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let key_app = Rc::clone(&app);
        let keydown =
            Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(move |event: KeyboardEvent| {
                let key = event.key();
                match key.as_str() {
                    "PageDown" => {
                        event.prevent_default();
                        key_app.borrow_mut().scroll_by(48);
                        return;
                    }
                    "PageUp" => {
                        event.prevent_default();
                        key_app.borrow_mut().scroll_by(-48);
                        return;
                    }
                    _ => {}
                }
                let Some(input) = keyboard_input(key.as_str()) else {
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

macro_rules! add {
    ($gui:expr, $y:expr, $h:expr, $f:expr) => {{
        let rect = Rect::new(8, $y, W - 16, $h);
        let id = $f(rect).unwrap();
        $y += $h as i32 + 4;
        id
    }};
}

#[allow(clippy::too_many_lines)]
fn build_ui(gui: &mut GuiContext<'static, 256, 128, 64>) -> (u32, Ids) {
    let mut y = 4i32;

    // ── Basic controls ────────────────────────────────────────────────────
    section(gui, &mut y, "BASIC CONTROLS");
    let button = add!(gui, y, 22, |r| gui.add_button(
        r,
        "CLICK ME",
        Style::button()
    ));
    let clicks_label = add!(gui, y, 14, |r| {
        gui.add_value_label(r, "CLICKS", 0, Style::panel())
    });
    let toggle = add!(gui, y, 16, |r| {
        gui.add_toggle(r, "ENABLE", false, Style::button())
    });
    let _ = add!(gui, y, 16, |r| {
        gui.add_checkbox(r, "CHECKBOX", true, Style::button())
    });
    let _ = add!(gui, y, 14, |r| {
        gui.add_progress_bar(r, 0.68, Style::progress())
    });
    let slider = add!(gui, y, 18, |r| {
        gui.add_slider(r, 0.5, 0.0, 1.0, Style::progress())
    });
    let slider_label = add!(gui, y, 14, |r| {
        gui.add_value_label(r, "SLIDER", 50, Style::panel())
    });
    let _ = add!(gui, y, 18, |r| {
        gui.add_icon_button(r, '>', "ICON BUTTON", Style::button())
    });

    // ── Lists & menus ─────────────────────────────────────────────────────
    section(gui, &mut y, "LISTS & MENUS");
    let _ = add!(gui, y, 60, |r| {
        gui.add_list(r, &ITEMS, 0, 4, Style::panel())
    });
    let _ = add!(gui, y, 60, |r| {
        gui.add_circular_list(r, &ITEMS, 0, 4, Style::panel())
    });
    let _ = add!(gui, y, 18, |r| {
        gui.add_tabs(r, &ITEMS, 0, Style::button())
    });
    let _ = add!(gui, y, 24, |r| {
        gui.add_dropdown(r, &ITEMS, 0, Style::panel())
    });
    let _ = add!(gui, y, 60, |r| {
        gui.add_roller(r, &ITEMS, 0, Style::panel())
    });
    let _ = add!(gui, y, 70, |r| {
        gui.add_menu(r, &ITEMS, 0, Style::panel())
    });
    let _ = add!(gui, y, 70, |r| {
        gui.add_rich_menu(r, &CELLS, 0, 3, Style::panel())
    });
    let _ = add!(gui, y, 70, |r| {
        gui.add_feed_timeline(r, &FEED_ITEMS, 0, 3, false, Style::panel())
    });

    // ── Data visualizations ───────────────────────────────────────────────
    section(gui, &mut y, "DATA & GAUGES");
    let _ = add!(gui, y, 26, |r| {
        gui.add_meter(r, 6.5, 0.0, 10.0, Style::panel())
    });
    let _ = add!(gui, y, 70, |r| {
        gui.add_arc_gauge(r, 6.5, 0.0, 10.0, 180, 270, 4, false, Style::panel())
    });
    let _ = add!(gui, y, 70, |r| {
        gui.add_gauge(r, 7.0, 0.0, 10.0, Style::panel())
    });
    let _ = add!(gui, y, 44, |r| {
        gui.add_sweeping_arc(
            r,
            0.5,
            true,
            16,
            2,
            2,
            Rgb565::CSS_BLACK,
            Rgb565::CSS_CYAN,
            Rgb565::CSS_WHITE,
            Style::panel(),
        )
    });
    let _ = add!(gui, y, 70, |r| {
        gui.add_gauge_needle(r, 7.0, 0.0, 10.0, 180, 270, Style::panel())
    });
    let _ = add!(gui, y, 46, |r| {
        gui.add_chart(r, &VALUES, 0.0, 10.0, Style::panel())
    });
    let _ = add!(gui, y, 46, |r| {
        gui.add_plotter(r, &VALUES, 0, 0.0, 10.0, Style::panel())
    });
    let _ = add!(gui, y, 70, |r| {
        gui.add_radial_scale(r, 0.0, 10.0, 5.0, Style::panel())
    });
    let _ = add!(gui, y, 24, |r| {
        gui.add_linear_scale(r, 0.0, 10.0, 5.0, Style::panel())
    });
    let _ = add!(gui, y, 70, |r| {
        gui.add_dial(r, 6.0, 0.0, 10.0, Style::panel())
    });

    // ── Text, input & tables ──────────────────────────────────────────────
    section(gui, &mut y, "TEXT & INPUT");
    let _ = add!(gui, y, 24, |r| {
        gui.add_spinbox(r, 0, 99, 42, Style::panel())
    });
    let _ = add!(gui, y, 40, |r| {
        gui.add_textarea(r, "Edit me", "placeholder", Style::panel())
    });
    let _ = add!(gui, y, 44, |r| {
        gui.add_keyboard(r, &KEYS, 6, None, Style::panel())
    });
    let _ = add!(gui, y, 50, |r| { gui.add_table(r, &ROWS, Style::panel()) });
    let _ = add!(gui, y, 40, |r| {
        gui.add_autocomplete_widget(r, &SUGGESTIONS, Style::panel())
    });

    // ── Motion, surfaces & overlays ───────────────────────────────────────
    section(gui, &mut y, "MOTION & OVERLAYS");
    let _ = add!(gui, y, 30, |r| { gui.add_spinner(r, 0.5, Style::panel()) });
    let _ = add!(gui, y, 60, |r| {
        gui.add_carousel(
            r,
            &CAROUSEL_ITEMS,
            2,
            CarouselSpec::new(10, 5),
            Style::panel(),
        )
    });
    let image = ImageRef::new(4, 4, &IMAGE_PX);
    let _ = add!(gui, y, 40, |r| {
        gui.add_image(r, image, ImageFit::Stretch, Style::panel())
    });
    let _ = add!(gui, y, 40, |r| {
        gui.add_peek_reveal(r, image, "Peek", "Subtitle", Style::panel())
    });
    let _ = add!(gui, y, 40, |r| {
        gui.add_glance_tile(r, 'g', "Glance", "Sub", Style::panel())
    });
    let _ = add!(gui, y, 50, |r| {
        gui.add_card_deck(r, &TITLES, 0, Style::panel())
    });
    let _ = add!(gui, y, 50, |r| {
        gui.add_state_surface(
            r,
            SurfaceState::Loading,
            "STATE SURFACE",
            "Loading...",
            None,
            Style::panel(),
        )
    });
    let _ = add!(gui, y, 26, |r| {
        gui.add_heads_up_banner(
            r,
            NotificationLevel::Warning,
            "HEADS UP",
            500,
            Style::panel(),
        )
    });
    let _ = add!(gui, y, 50, |r| {
        gui.add_notification_action_sheet(
            r,
            NotificationLevel::Info,
            "NOTIFICATION",
            "Body",
            &ACTIONS,
            0,
            false,
            Style::panel(),
        )
    });
    let _ = add!(gui, y, 24, |r| {
        gui.add_toast(r, "TOAST", 1000, Style::panel())
    });
    let _ = add!(gui, y, 40, |r| {
        gui.add_dialog(r, "DIALOG", "Body", Style::panel())
    });

    (
        y as u32 + 2,
        Ids {
            button,
            toggle,
            slider,
            clicks_label,
            slider_label,
        },
    )
}

fn section(gui: &mut GuiContext<'static, 256, 128, 64>, y: &mut i32, text: &'static str) {
    gui.add_label(Rect::new(8, *y, 304, 12), text, Style::label())
        .unwrap();
    *y += 16;
}
