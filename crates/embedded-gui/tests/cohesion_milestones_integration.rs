use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_gui::{
    CustomCanvas, CustomWidget, EventContext, EventPolicy, Framebuffer, LanguageId, Rect, Render,
    ScreenId, Signal, StandardGuiContext, Theme, TranslationEntry, TranslationTable, UiEvent,
    ViewContext, WidgetStyle, include_gui, include_project,
};

// Test M5: include_project! on the studio demo project (generates ScreenTag, AppNavigator, status, menu, counter modules)
include_project!("../embedded-gui-studio/examples/ssd1357-demo/project.kdl");

// Test M3: include_gui! on a generated screen with Screen and Render
include_gui!("examples/ui/smart_thermostat.kdl");

// A sample custom widget implementing the new safe CustomWidget trait (M4)
struct AnalogMeterWidget {
    value: f32,
}

impl CustomWidget for AnalogMeterWidget {
    fn render(
        &self,
        canvas: &mut dyn CustomCanvas,
        rect: Rect,
        _style: &WidgetStyle,
        _state: embedded_gui::VisualState,
    ) -> Result<(), GuiError> {
        // Draw meter background
        canvas.fill_rect(rect, Rgb565::BLACK)?;
        canvas.draw_rect(rect, Rgb565::WHITE)?;

        // Draw indicator needle line
        let cx = rect.x + (rect.w as i32) / 2;
        let cy = rect.bottom() - 2;
        let nx = cx + ((self.value - 0.5) * (rect.w as f32 * 0.8)) as i32;
        let ny = rect.y + 4;
        canvas.draw_line((cx, cy), (nx, ny), Rgb565::RED)?;
        Ok(())
    }

    fn handle_event(&self, event: &UiEvent, _ctx: &mut EventContext) -> EventPolicy {
        if matches!(event, UiEvent::Clicked(_)) {
            EventPolicy::Stop
        } else {
            EventPolicy::Continue
        }
    }

    fn focusable(&self) -> bool {
        true
    }
}

static TRANSLATION_ENTRIES: [TranslationEntry<'static>; 2] = [
    TranslationEntry::new("Fan", &["Fan", "Ventilador", "Lüfter"]),
    TranslationEntry::new("Save", &["Save", "Guardar", "Speichern"]),
];

#[test]
fn test_m1_dynamic_theming_and_i18n() {
    let table = TranslationTable::new(&TRANSLATION_ENTRIES);
    let mut gui = StandardGuiContext::new(Rect::new(0, 0, 320, 240));

    gui.set_translation_table(&table);
    assert_eq!(gui.translate("Fan"), "Fan");

    // Switch language to Spanish
    gui.set_language(LanguageId::ES).unwrap();
    assert_eq!(gui.translate("Fan"), "Ventilador");

    // Build screen
    let app = SmartThermostatApp::build(&mut gui).unwrap();

    // Verify apply_theme updates theme
    app.apply_theme(&mut gui, Theme::light()).unwrap();
    assert_eq!(gui.theme(), Theme::light());

    // Switch language to Spanish via app
    app.set_language(&mut gui, LanguageId::ES).unwrap();
    assert_eq!(gui.active_language(), LanguageId::ES);

    // Verify set_text helper
    gui.set_text(app.widgets.fan_btn, "Custom Fan").unwrap();
}

#[test]
fn test_m2_reactive_signals() {
    let mut gui = StandardGuiContext::new(Rect::new(0, 0, 320, 240));
    let lbl = gui
        .add_label(Rect::new(10, 10, 80, 24), "Initial", WidgetStyle::label())
        .unwrap();

    let mut temp_signal = Signal::<i32, 4>::new(21);
    gui.bind_signal(lbl, &mut temp_signal).unwrap();

    // Initially not dirty
    assert!(!gui.poll_signal(&mut temp_signal).unwrap());

    // Update signal
    temp_signal.update(|v| *v = 24);
    assert!(temp_signal.is_dirty());

    // Polling auto-invalidates subscribed widget dirty rects
    let polled = gui.poll_signal(&mut temp_signal).unwrap();
    assert!(polled);
    assert!(!temp_signal.is_dirty());
    assert_eq!(temp_signal.get(), 24);

    // ViewContext signal binding
    let mut cx = ViewContext::new(&mut gui, Rect::new(0, 0, 320, 240));
    cx.bind_signal(lbl, &mut temp_signal).unwrap();
}

#[test]
fn test_m3_layout_and_component_convergence() {
    let mut gui = StandardGuiContext::new(Rect::new(0, 0, 320, 240));
    let app = SmartThermostatApp::build(&mut gui).unwrap();

    // Inherent screen_id check
    assert_eq!(app.screen_id(), ScreenId::new(6356));

    // Screen trait implementation check
    assert_eq!(
        <SmartThermostatApp as Screen<'_, 64, 32, 16>>::id(&app),
        ScreenId::new(6356)
    );

    // Render trait implementation check into a fresh ViewContext
    let mut gui2 = StandardGuiContext::new(Rect::new(0, 0, 320, 240));
    let mut cx = ViewContext::new(&mut gui2, Rect::new(0, 0, 320, 240));
    let root_id = app.render(&mut cx).unwrap();
    assert_eq!(root_id, WidgetId(2));
    assert!(gui2.widgets().len() >= 5);
}

#[test]
fn test_m4_custom_widget_extension() {
    let meter = AnalogMeterWidget { value: 0.75 };
    let mut gui = StandardGuiContext::new(Rect::new(0, 0, 320, 240));
    let meter_id = gui
        .add_custom_widget(Rect::new(10, 10, 100, 60), &meter, WidgetStyle::panel())
        .unwrap();

    assert!(gui.absolute_rect(meter_id).is_some());

    // Render to framebuffer
    let mut fb = Framebuffer::<{ 320 * 240 }>::new(320, 240);
    fb.clear_color(Rgb565::BLACK);
    gui.render(&mut fb).unwrap();

    // Verify pixels were drawn
    let colored = fb.pixels().iter().filter(|&&c| c != Rgb565::BLACK).count();
    assert!(colored > 50, "Custom widget should draw onto canvas");
}

#[test]
fn test_m5_include_project_navigation() {
    let mut nav = AppNavigator::default();
    assert_eq!(nav.current(), ScreenTag::Status);
    assert_eq!(nav.previous(), None);

    // Switch screen
    assert!(nav.switch_to(ScreenTag::Menu));
    assert_eq!(nav.current(), ScreenTag::Menu);
    assert_eq!(nav.previous(), Some(ScreenTag::Status));
    assert_eq!(nav.transition_count(), 1);

    // Switch back
    assert!(nav.back());
    assert_eq!(nav.current(), ScreenTag::Status);
    assert_eq!(nav.previous(), Some(ScreenTag::Menu));

    // Verify submodules generated from project.kdl
    let mut gui = StandardGuiContext::new(Rect::new(0, 0, 96, 64));
    let _status_app = status::StatusApp::build(&mut gui).unwrap();
    let _menu_app = menu::MenuApp::build(&mut gui).unwrap();
    let _counter_app = counter::CounterApp::build(&mut gui).unwrap();
}
