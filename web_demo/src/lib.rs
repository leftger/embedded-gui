//! Multi-screen animated WebAssembly showcase.
//!
//! Each category is its own 320x240 `GuiContext` screen. Navigating between
//! screens renders the outgoing and incoming contexts through
//! `render_transition_pair`, giving the browser demo the same
//! PushMoook/slide/fade/shutter/flip/wipe transitions that firmware can use.

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
const SCREEN_COUNT: usize = 6;
const TRANSITION_MS: u32 = 420;

static ITEMS: [&str; 5] = ["HOME", "MEDIA", "MAPS", "SETTINGS", "POWER"];
static CAROUSEL_ITEMS: [&str; 7] = ["ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN"];
static ROWS: [&[&str]; 2] = [&["1", "2"], &["3", "4"]];
static KEYS: [char; 12] = ['1', '2', '3', '4', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
static VALUES: [f32; 8] = [1.0, 2.0, 4.0, 3.0, 5.0, 8.0, 6.0, 7.0];
static TITLES: [&str; 3] = ["Card 1", "Card 2", "Card 3"];
static ACTIONS: [&str; 2] = ["OK", "Cancel"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScreenKind {
    Controls,
    Lists,
    Data,
    Text,
    Motion,
    Overlays,
}

impl ScreenKind {
    const ALL: [ScreenKind; SCREEN_COUNT] = [
        ScreenKind::Controls,
        ScreenKind::Lists,
        ScreenKind::Data,
        ScreenKind::Text,
        ScreenKind::Motion,
        ScreenKind::Overlays,
    ];

    fn from_index(index: usize) -> Self {
        Self::ALL[index % SCREEN_COUNT]
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|&kind| kind == self).unwrap_or(0)
    }

    fn title(self) -> &'static str {
        match self {
            ScreenKind::Controls => "CONTROLS",
            ScreenKind::Lists => "LISTS & MENUS",
            ScreenKind::Data => "DATA & GAUGES",
            ScreenKind::Text => "TEXT & INPUT",
            ScreenKind::Motion => "MOTION",
            ScreenKind::Overlays => "OVERLAYS & FEEDBACK",
        }
    }
}

struct ScreenIds {
    prev: WidgetId,
    next: WidgetId,
    button: Option<WidgetId>,
    toggle: Option<WidgetId>,
    slider: Option<WidgetId>,
    clicks_label: Option<WidgetId>,
    slider_label: Option<WidgetId>,
    progress: Option<WidgetId>,
    tabs: Option<WidgetId>,
    dropdown: Option<WidgetId>,
    roller: Option<WidgetId>,
    gauge: Option<WidgetId>,
    arc_gauge: Option<WidgetId>,
    carousel: Option<WidgetId>,
    state_surface: Option<WidgetId>,
    heads_up: Option<WidgetId>,
}

struct TransitionState {
    active: ActiveScreenTransition,
    elapsed_ms: u32,
    duration_ms: u32,
}

struct App {
    gui: Box<GuiContext<'static, 256, 128, 64>>,
    outgoing: Option<Box<GuiContext<'static, 256, 128, 64>>>,
    display: WebSimulatorDisplay<Rgb565>,
    canvas: HtmlCanvasElement,
    screen: ScreenKind,
    ids: ScreenIds,
    transition: Option<TransitionState>,
    animator: WidgetAnimator<32, 32>,
    motion_phase: f32,
    autoplay_ms: u32,
    pointer_down: bool,
    clicks: i32,
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

        let (gui, ids) = build_screen(ScreenKind::Controls);

        let mut app = Self {
            gui: Box::new(gui),
            outgoing: None,
            display,
            canvas,
            screen: ScreenKind::Controls,
            ids,
            transition: None,
            animator: WidgetAnimator::new(),
            motion_phase: 0.0,
            autoplay_ms: 0,
            pointer_down: false,
            clicks: 0,
        };
        start_screen_animations(ScreenKind::Controls, &app.ids, &mut app.animator);
        Ok(app)
    }

    fn pointer_pos(&self, client_x: i32, client_y: i32) -> (i32, i32) {
        let rect = self.canvas.get_bounding_client_rect();
        let x = ((client_x as f64 - rect.left()) / rect.width() * W as f64) as i32;
        let y = ((client_y as f64 - rect.top()) / rect.height() * H as f64) as i32;
        (x.clamp(0, W as i32 - 1), y.clamp(0, H as i32 - 1))
    }

    fn handle_input(&mut self, event: InputEvent) {
        self.autoplay_ms = 0;
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
            UiEvent::Activate(id) if id == self.ids.prev => self.navigate(-1),
            UiEvent::Activate(id) if id == self.ids.next => self.navigate(1),
            UiEvent::Activate(id) if self.ids.button == Some(id) => {
                self.clicks = self.clicks.saturating_add(1);
                if let Some(label) = self.ids.clicks_label {
                    let _ = self.gui.set_value_label(label, self.clicks);
                }
            }
            UiEvent::ValueChanged(id) if self.ids.toggle == Some(id) => {
                // The toggle widget mutates its own checked state; nothing else to do.
            }
            UiEvent::ValueChanged(id) if self.ids.slider == Some(id) => {
                let value = match self.gui.get_widget_property(id, PropertyKey::Value) {
                    Some(PropertyValue::Float(value)) => value,
                    _ => 0.0,
                };
                if let Some(label) = self.ids.slider_label {
                    let _ = self
                        .gui
                        .set_value_label(label, (value * 100.0).round() as i32);
                }
            }
            _ => {}
        }
    }

    fn navigate(&mut self, direction: i8) {
        if self.transition.is_some() {
            return;
        }
        let next_index = (self.screen.index() as isize + direction as isize)
            .rem_euclid(SCREEN_COUNT as isize) as usize;
        let next_screen = ScreenKind::from_index(next_index);
        let (new_gui, new_ids) = build_screen(next_screen);
        let old_gui = core::mem::replace(&mut self.gui, Box::new(new_gui));
        self.ids = new_ids;
        self.screen = next_screen;
        self.outgoing = Some(old_gui);
        self.animator = WidgetAnimator::new();
        start_screen_animations(next_screen, &self.ids, &mut self.animator);
        self.transition = Some(TransitionState {
            active: ActiveScreenTransition {
                from: None,
                to: None,
                effect: transition_effect(direction, next_index),
                origin: ScreenTransitionOrigin::Center,
                progress: 0.0,
            },
            elapsed_ms: 0,
            duration_ms: TRANSITION_MS,
        });
        self.redraw();
    }

    fn tick(&mut self, dt_ms: u32) {
        if self.transition.is_some() {
            let mut transition_done = false;
            if let Some(transition) = &mut self.transition {
                transition.elapsed_ms = transition.elapsed_ms.saturating_add(dt_ms);
                transition.active.progress =
                    (transition.elapsed_ms as f32 / transition.duration_ms as f32).clamp(0.0, 1.0);
                transition_done = transition.elapsed_ms >= transition.duration_ms;
            }
            if transition_done {
                self.outgoing = None;
                self.transition = None;
            }
            self.redraw();
            return;
        }

        self.motion_phase += dt_ms as f32;
        self.autoplay_ms = self.autoplay_ms.saturating_add(dt_ms);
        if self.autoplay_ms >= 7000 {
            self.autoplay_ms = 0;
            self.navigate(1);
            self.redraw();
            return;
        }
        let _ = self.animator.tick(dt_ms, &mut self.gui);
        self.tick_screen_motion(dt_ms);
        self.redraw();
    }

    fn tick_screen_motion(&mut self, dt_ms: u32) {
        let phase = self.motion_phase;
        match self.screen {
            ScreenKind::Controls => {
                if let Some(progress) = self.ids.progress {
                    let value = 0.5 + 0.45 * (phase * 0.004).sin();
                    let _ = self.gui.set_progress(progress, value);
                }
                if let Some(tabs) = self.ids.tabs {
                    let index = ((phase * 0.0012) as usize) % ITEMS.len();
                    let _ = self.gui.set_tab_selected(tabs, index);
                }
            }
            ScreenKind::Lists => {
                if let Some(dropdown) = self.ids.dropdown {
                    let index = ((phase * 0.001) as usize) % ITEMS.len();
                    let _ = self.gui.set_dropdown_selected(dropdown, index);
                }
                if let Some(roller) = self.ids.roller {
                    let index = ((phase * 0.0008) as usize) % ITEMS.len();
                    let _ = self.gui.set_roller_selected(roller, index);
                }
            }
            ScreenKind::Data => {
                if let Some(gauge) = self.ids.gauge {
                    let value = 5.5 + 4.0 * (phase * 0.0035).sin();
                    let _ = self.gui.set_gauge_value(gauge, value);
                }
                if let Some(arc_gauge) = self.ids.arc_gauge {
                    let value = 5.5 + 4.0 * (phase * 0.0028).sin();
                    let _ = self.gui.set_gauge_value(arc_gauge, value);
                }
            }
            ScreenKind::Text => {}
            ScreenKind::Motion => {
                if let Some(carousel) = self.ids.carousel {
                    let shift = ((phase * 0.006).sin() * 7.0) as i16;
                    let _ = self.gui.set_carousel_shift(carousel, shift);
                }
            }
            ScreenKind::Overlays => {
                if let Some(state) = self.ids.state_surface {
                    let _ = self.gui.tick_state_surface(state, dt_ms, 1.0);
                }
                if let Some(heads_up) = self.ids.heads_up {
                    let _ = self.gui.tick_heads_up(heads_up, dt_ms);
                }
            }
        }
    }

    fn redraw(&mut self) {
        let _ = self.display.clear(Rgb565::BLACK);
        if let Some(transition) = &self.transition {
            if let Some(outgoing) = &self.outgoing {
                let _ = render_transition_pair(
                    &mut self.display,
                    outgoing,
                    &self.gui,
                    transition.active,
                    W,
                    H,
                );
            }
        } else {
            let _ = self.gui.render(&mut self.display);
        }
        let _ = self.display.flush();
    }
}

fn start_screen_animations(
    kind: ScreenKind,
    ids: &ScreenIds,
    animator: &mut WidgetAnimator<32, 32>,
) {
    match kind {
        ScreenKind::Controls => {
            if let Some(progress) = ids.progress {
                let _ = animator.ping_pong_progress(progress, 0.15, 0.95, 1600, Easing::InOutSine);
            }
            if let Some(tabs) = ids.tabs {
                let animation = Animation::new(0.0, (ITEMS.len() - 1) as f32, 2400, Easing::Linear)
                    .with_repeat_mode(RepeatMode::Loop)
                    .with_repeat_count(None);
                let _ = animator.bind_property(tabs, AnimatedProperty::TabSelected, animation);
            }
        }
        ScreenKind::Lists => {
            if let Some(dropdown) = ids.dropdown {
                let animation = Animation::new(0.0, (ITEMS.len() - 1) as f32, 2600, Easing::Linear)
                    .with_repeat_mode(RepeatMode::Loop)
                    .with_repeat_count(None);
                let _ =
                    animator.bind_property(dropdown, AnimatedProperty::DropdownSelected, animation);
            }
            if let Some(roller) = ids.roller {
                let animation = Animation::new(0.0, (ITEMS.len() - 1) as f32, 2800, Easing::Linear)
                    .with_repeat_mode(RepeatMode::Loop)
                    .with_repeat_count(None);
                let _ = animator.bind_property(roller, AnimatedProperty::RollerSelected, animation);
            }
        }
        ScreenKind::Data => {
            let animation = Animation::new(2.0, 9.0, 1600, Easing::InOutSine)
                .with_repeat_mode(RepeatMode::PingPong)
                .with_repeat_count(None);
            if let Some(gauge) = ids.gauge {
                let _ = animator.bind_property(gauge, AnimatedProperty::GaugeValue, animation);
            }
            let arc_animation = Animation::new(2.0, 9.0, 1900, Easing::InOutSine)
                .with_repeat_mode(RepeatMode::PingPong)
                .with_repeat_count(None);
            if let Some(arc_gauge) = ids.arc_gauge {
                let _ =
                    animator.bind_property(arc_gauge, AnimatedProperty::GaugeValue, arc_animation);
            }
        }
        ScreenKind::Text => {}
        ScreenKind::Motion => {}
        ScreenKind::Overlays => {}
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let app = Rc::new(RefCell::new(App::new()?));
    app.borrow_mut().redraw();

    // Pointer input.
    {
        let down_app = Rc::clone(&app);
        let pointer_down =
            Closure::<dyn FnMut(PointerEvent)>::wrap(Box::new(move |event: PointerEvent| {
                if event.button() == 0 {
                    let mut app = down_app.borrow_mut();
                    app.pointer_down = true;
                    let (x, y) = app.pointer_pos(event.client_x(), event.client_y());
                    app.handle_input(InputEvent::Pointer {
                        x,
                        y,
                        state: PointerState::Pressed,
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
            .add_event_listener_with_callback("pointerup", pointer_up.as_ref().unchecked_ref())
            .map_err(js_error)?;
        canvas
            .add_event_listener_with_callback("pointercancel", pointer_up.as_ref().unchecked_ref())
            .map_err(js_error)?;

        pointer_down.forget();
        pointer_up.forget();
    }

    // Keyboard input.
    {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let key_app = Rc::clone(&app);
        let keydown =
            Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(move |event: KeyboardEvent| {
                let key = event.key();
                match key.as_str() {
                    "ArrowRight" if event.shift_key() => {
                        event.prevent_default();
                        key_app.borrow_mut().navigate(1);
                        return;
                    }
                    "ArrowLeft" if event.shift_key() => {
                        event.prevent_default();
                        key_app.borrow_mut().navigate(-1);
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

    // Animation loop for screen transitions and in-widget motion.
    {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let loop_window = window.clone();
        let holder: &'static RefCell<Option<Closure<dyn FnMut()>>> =
            Box::leak(Box::new(RefCell::new(None)));
        let tick_app = Rc::clone(&app);
        let mut last_frame_ms = 0.0f64;
        let frame = Closure::<dyn FnMut()>::wrap(Box::new(move || {
            let now = js_sys::Date::now();
            let dt = if last_frame_ms == 0.0 {
                16.0
            } else {
                (now - last_frame_ms).clamp(8.0, 80.0)
            };
            last_frame_ms = now;
            tick_app.borrow_mut().tick(dt as u32);
            if let Some(callback) = holder.borrow().as_ref() {
                let _ = loop_window.request_animation_frame(callback.as_ref().unchecked_ref());
            }
        }));
        *holder.borrow_mut() = Some(frame);
        let callback = holder.borrow();
        let callback_ref = callback
            .as_ref()
            .ok_or_else(|| JsValue::from_str("no loop"))?;
        window
            .request_animation_frame(callback_ref.as_ref().unchecked_ref())
            .map_err(js_error)?;
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

fn transition_effect(direction: i8, index: usize) -> ScreenTransitionEffect {
    const FORWARD: [ScreenTransitionEffect; 7] = [
        ScreenTransitionEffect::PushMoook,
        ScreenTransitionEffect::SlideLeft,
        ScreenTransitionEffect::ShutterRight,
        ScreenTransitionEffect::RoundFlipRight,
        ScreenTransitionEffect::PortHoleRight,
        ScreenTransitionEffect::WipeRight,
        ScreenTransitionEffect::CircularReveal,
    ];
    const BACKWARD: [ScreenTransitionEffect; 7] = [
        ScreenTransitionEffect::PopMoook,
        ScreenTransitionEffect::SlideRight,
        ScreenTransitionEffect::ShutterLeft,
        ScreenTransitionEffect::RoundFlipLeft,
        ScreenTransitionEffect::PortHoleLeft,
        ScreenTransitionEffect::WipeLeft,
        ScreenTransitionEffect::Fade,
    ];
    let effects = if direction > 0 { FORWARD } else { BACKWARD };
    effects[index % effects.len()]
}

fn js_error(error: impl core::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}

fn build_screen(kind: ScreenKind) -> (GuiContext<'static, 256, 128, 64>, ScreenIds) {
    let mut gui = GuiContext::<256, 128, 64>::new(Rect::new(0, 0, W, H));
    let (prev, next) = add_chrome(&mut gui, kind.title(), kind.index());
    let ids = match kind {
        ScreenKind::Controls => add_controls(&mut gui, prev, next),
        ScreenKind::Lists => add_lists(&mut gui, prev, next),
        ScreenKind::Data => add_data(&mut gui, prev, next),
        ScreenKind::Text => add_text(&mut gui, prev, next),
        ScreenKind::Motion => add_motion(&mut gui, prev, next),
        ScreenKind::Overlays => add_overlays(&mut gui, prev, next),
    };
    (gui, ids)
}

fn add_chrome(
    gui: &mut GuiContext<'static, 256, 128, 64>,
    title: &'static str,
    index: usize,
) -> (WidgetId, WidgetId) {
    gui.add_label(Rect::new(12, 8, 200, 12), title, Style::label())
        .unwrap();
    gui.add_value_label(
        Rect::new(220, 7, 88, 12),
        "SCREEN",
        index as i32 + 1,
        Style::panel(),
    )
    .unwrap();
    let prev = gui
        .add_button(Rect::new(12, 212, 84, 22), "PREV", Style::button())
        .unwrap();
    let next = gui
        .add_button(Rect::new(224, 212, 84, 22), "NEXT", Style::button())
        .unwrap();
    (prev, next)
}

fn add_controls(
    gui: &mut GuiContext<'static, 256, 128, 64>,
    prev: WidgetId,
    next: WidgetId,
) -> ScreenIds {
    gui.add_label(
        Rect::new(12, 28, 296, 10),
        "BUTTON / TOGGLE / CHECKBOX / SLIDER",
        Style::label(),
    )
    .unwrap();

    let button = gui
        .add_button(Rect::new(12, 42, 136, 22), "CLICK ME", Style::button())
        .unwrap();
    let toggle = gui
        .add_toggle(
            Rect::new(164, 44, 144, 16),
            "ENABLE",
            false,
            Style::button(),
        )
        .unwrap();
    let checkbox = gui
        .add_checkbox(Rect::new(12, 70, 136, 16), "CHECK", true, Style::button())
        .unwrap();
    let _icon = gui
        .add_icon_button(Rect::new(164, 68, 144, 20), '>', "ICON", Style::button())
        .unwrap();
    let progress = gui
        .add_progress_bar(Rect::new(12, 96, 296, 12), 0.68, Style::progress())
        .unwrap();
    let slider = gui
        .add_slider(
            Rect::new(12, 116, 296, 16),
            0.5,
            0.0,
            1.0,
            Style::progress(),
        )
        .unwrap();
    let clicks_label = gui
        .add_value_label(Rect::new(12, 142, 130, 14), "CLICKS", 0, Style::panel())
        .unwrap();
    let slider_label = gui
        .add_value_label(Rect::new(164, 142, 144, 14), "SLIDER", 50, Style::panel())
        .unwrap();
    let tabs = gui
        .add_tabs(Rect::new(12, 168, 296, 16), &ITEMS, 0, Style::button())
        .unwrap();

    let _ = checkbox;
    let _ = toggle;

    ScreenIds {
        prev,
        next,
        button: Some(button),
        toggle: Some(toggle),
        slider: Some(slider),
        clicks_label: Some(clicks_label),
        slider_label: Some(slider_label),
        progress: Some(progress),
        tabs: Some(tabs),
        dropdown: None,
        roller: None,
        gauge: None,
        arc_gauge: None,
        carousel: None,
        state_surface: None,
        heads_up: None,
    }
}

fn add_lists(
    gui: &mut GuiContext<'static, 256, 128, 64>,
    prev: WidgetId,
    next: WidgetId,
) -> ScreenIds {
    gui.add_label(
        Rect::new(12, 28, 296, 10),
        "MENU / LIST / DROPDOWN / ROLLER",
        Style::label(),
    )
    .unwrap();

    let _menu = gui
        .add_menu(Rect::new(12, 42, 140, 82), &ITEMS, 0, Style::panel())
        .unwrap();
    let _list = gui
        .add_list(Rect::new(168, 42, 140, 82), &ITEMS, 0, 5, Style::panel())
        .unwrap();
    let dropdown = gui
        .add_dropdown(Rect::new(12, 132, 140, 22), &ITEMS, 0, Style::panel())
        .unwrap();
    let roller = gui
        .add_roller(Rect::new(168, 130, 140, 62), &ITEMS, 0, Style::panel())
        .unwrap();

    ScreenIds {
        prev,
        next,
        button: None,
        toggle: None,
        slider: None,
        clicks_label: None,
        slider_label: None,
        progress: None,
        tabs: None,
        dropdown: Some(dropdown),
        roller: Some(roller),
        gauge: None,
        arc_gauge: None,
        carousel: None,
        state_surface: None,
        heads_up: None,
    }
}

fn add_data(
    gui: &mut GuiContext<'static, 256, 128, 64>,
    prev: WidgetId,
    next: WidgetId,
) -> ScreenIds {
    gui.add_label(
        Rect::new(12, 28, 296, 10),
        "CHART / PLOTTER / ARC / GAUGE",
        Style::label(),
    )
    .unwrap();

    let _chart = gui
        .add_chart(
            Rect::new(12, 42, 296, 44),
            &VALUES,
            0.0,
            10.0,
            Style::panel(),
        )
        .unwrap();
    let _plotter = gui
        .add_plotter(
            Rect::new(12, 92, 296, 36),
            &VALUES,
            0,
            0.0,
            10.0,
            Style::panel(),
        )
        .unwrap();
    let arc_gauge = gui
        .add_arc_gauge(
            Rect::new(12, 136, 140, 66),
            6.5,
            0.0,
            10.0,
            180,
            270,
            4,
            false,
            Style::panel(),
        )
        .unwrap();
    let gauge = gui
        .add_gauge(Rect::new(168, 136, 140, 66), 7.0, 0.0, 10.0, Style::panel())
        .unwrap();

    ScreenIds {
        prev,
        next,
        button: None,
        toggle: None,
        slider: None,
        clicks_label: None,
        slider_label: None,
        progress: None,
        tabs: None,
        dropdown: None,
        roller: None,
        gauge: Some(gauge),
        arc_gauge: Some(arc_gauge),
        carousel: None,
        state_surface: None,
        heads_up: None,
    }
}

fn add_text(
    gui: &mut GuiContext<'static, 256, 128, 64>,
    prev: WidgetId,
    next: WidgetId,
) -> ScreenIds {
    gui.add_label(
        Rect::new(12, 28, 296, 10),
        "TEXTAREA / TABLE / KEYBOARD",
        Style::label(),
    )
    .unwrap();

    let _textarea = gui
        .add_textarea(
            Rect::new(12, 38, 296, 42),
            "Edit me",
            "placeholder",
            Style::panel(),
        )
        .unwrap();
    let _table = gui
        .add_table(Rect::new(12, 88, 296, 56), &ROWS, Style::panel())
        .unwrap();
    let _keyboard = gui
        .add_keyboard(Rect::new(12, 152, 296, 50), &KEYS, 6, None, Style::button())
        .unwrap();

    ScreenIds {
        prev,
        next,
        button: None,
        toggle: None,
        slider: None,
        clicks_label: None,
        slider_label: None,
        progress: None,
        tabs: None,
        dropdown: None,
        roller: None,
        gauge: None,
        arc_gauge: None,
        carousel: None,
        state_surface: None,
        heads_up: None,
    }
}

fn add_motion(
    gui: &mut GuiContext<'static, 256, 128, 64>,
    prev: WidgetId,
    next: WidgetId,
) -> ScreenIds {
    gui.add_label(
        Rect::new(12, 28, 296, 10),
        "CAROUSEL / CARD DECK",
        Style::label(),
    )
    .unwrap();

    let carousel = gui
        .add_carousel(
            Rect::new(12, 38, 296, 66),
            &CAROUSEL_ITEMS,
            2,
            CarouselSpec {
                fade_edges: false,
                indicator: true,
                ..CarouselSpec::new(12, 5)
            },
            Style::panel(),
        )
        .unwrap();
    let _card_deck = gui
        .add_card_deck(Rect::new(12, 114, 296, 54), &TITLES, 0, Style::panel())
        .unwrap();
    add_motion_hint(gui);

    ScreenIds {
        prev,
        next,
        button: None,
        toggle: None,
        slider: None,
        clicks_label: None,
        slider_label: None,
        progress: None,
        tabs: None,
        dropdown: None,
        roller: None,
        gauge: None,
        arc_gauge: None,
        carousel: Some(carousel),
        state_surface: None,
        heads_up: None,
    }
}

fn add_motion_hint(gui: &mut GuiContext<'static, 256, 128, 64>) {
    gui.add_label(
        Rect::new(12, 182, 296, 10),
        "CAROUSEL DRIFTS WHILE THE DECK CYCLES",
        Style::label(),
    )
    .unwrap();
}

fn add_overlays(
    gui: &mut GuiContext<'static, 256, 128, 64>,
    prev: WidgetId,
    next: WidgetId,
) -> ScreenIds {
    gui.add_label(
        Rect::new(12, 28, 296, 10),
        "STATE / NOTIFICATION / HEADS-UP",
        Style::label(),
    )
    .unwrap();

    let state_surface = gui
        .add_state_surface(
            Rect::new(12, 38, 296, 52),
            SurfaceState::Error,
            "SYNC",
            "Could not load",
            Some("Retry"),
            Style::panel(),
        )
        .unwrap();
    let heads_up = gui
        .add_heads_up_banner(
            Rect::new(12, 98, 296, 26),
            NotificationLevel::Warning,
            "HEADS UP",
            60_000,
            Style::panel(),
        )
        .unwrap();
    let _sheet = gui
        .add_notification_action_sheet(
            Rect::new(12, 132, 296, 64),
            NotificationLevel::Info,
            "NOTIFICATION",
            "Body",
            &ACTIONS,
            0,
            true,
            Style::panel(),
        )
        .unwrap();

    ScreenIds {
        prev,
        next,
        button: None,
        toggle: None,
        slider: None,
        clicks_label: None,
        slider_label: None,
        progress: None,
        tabs: None,
        dropdown: None,
        roller: None,
        gauge: None,
        arc_gauge: None,
        carousel: None,
        state_surface: Some(state_surface),
        heads_up: Some(heads_up),
    }
}
