use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics_core::{draw_target::DrawTarget, geometry::Size, pixelcolor::RgbColor};
use embedded_gui::prelude::*;

struct DummyDisplay {
    pixels: [Rgb565; 320 * 240],
}

impl DummyDisplay {
    fn new() -> Self {
        Self {
            pixels: [Rgb565::BLACK; 320 * 240],
        }
    }
}

impl embedded_graphics_core::geometry::OriginDimensions for DummyDisplay {
    fn size(&self) -> Size {
        Size::new(320, 240)
    }
}

impl DrawTarget for DummyDisplay {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        for embedded_graphics_core::Pixel(pt, color) in pixels {
            if pt.x >= 0 && pt.x < 320 && pt.y >= 0 && pt.y < 240 {
                let idx = (pt.y as usize) * 320 + (pt.x as usize);
                self.pixels[idx] = color;
            }
        }
        Ok(())
    }
}

#[test]
fn test_reactive_signals_and_callbacks() {
    let mut signal = Signal::<i32, 4>::new(42);
    assert_eq!(signal.get(), 42);
    assert_eq!(signal.version(), 0);
    assert!(!signal.is_dirty());

    let widget_a = WidgetId::new(10);
    let widget_b = WidgetId::new(20);

    assert!(signal.subscribe(widget_a));
    assert!(signal.subscribe(widget_b));
    assert_eq!(signal.subscribers(), &[widget_a, widget_b]);

    // Mutate signal value
    assert!(signal.set(100));
    assert_eq!(signal.get(), 100);
    assert_eq!(signal.version(), 1);
    assert!(signal.is_dirty());

    // Setting same value returns false
    assert!(!signal.set(100));
    assert_eq!(signal.version(), 1);

    signal.clear_dirty();
    assert!(!signal.is_dirty());

    // Callbacks
    static mut CALLBACK_CALLED: bool = false;
    let slot = CallbackSlot::<u32>::new(|val| {
        if val == 999 {
            unsafe {
                CALLBACK_CALLED = true;
            }
        }
    });

    assert!(slot.is_bound());
    slot.emit(999);
    unsafe {
        assert!(CALLBACK_CALLED);
    }
}

#[test]
fn test_state_machine_transitions_and_lerp() {
    let mut sm = WidgetStateMachine::new(VisualState::Normal);
    assert_eq!(sm.current(), VisualState::Normal);
    assert!(!sm.is_animating());

    // Start transition to Pressed over 10 ticks
    assert!(sm.transition_to(VisualState::Pressed, 10));
    assert!(sm.is_animating());
    assert_eq!(sm.target(), VisualState::Pressed);
    assert_eq!(sm.progress(), 0.0);

    // Initial lerp
    assert!((sm.lerp_scalar(10.0, 20.0) - 10.0).abs() < 1e-4);

    // Halfway tick
    sm.tick(5);
    assert!(sm.is_animating());
    assert!((sm.progress() - 0.5).abs() < 1e-4);
    assert!((sm.lerp_scalar(10.0, 20.0) - 15.0).abs() < 1e-4);

    // Finish tick
    sm.tick(5);
    assert!(!sm.is_animating());
    assert_eq!(sm.current(), VisualState::Pressed);
    assert!((sm.progress() - 1.0).abs() < 1e-4);
    assert!((sm.lerp_scalar(10.0, 20.0) - 20.0).abs() < 1e-4);
}

#[test]
fn test_slice_model_and_repeater_widget() {
    let items = ["Engine 1", "Engine 2", "Auxiliary Power", "Cabin Temp"];
    let model = SliceModel::new(&items);

    assert_eq!(model.row_count(), 4);
    assert_eq!(model.row_data(0), Some("Engine 1"));
    assert_eq!(model.row_data(3), Some("Cabin Temp"));
    assert_eq!(model.row_data(4), None);

    let mut repeater = RepeaterWidget::<4>::new(20);
    repeater.total_count = model.row_count();

    assert_eq!(repeater.selected, 0);
    repeater.bump_selection(1);
    assert_eq!(repeater.selected, 1);

    repeater.bump_selection(-1);
    assert_eq!(repeater.selected, 0);
}

#[test]
fn test_line_buffer_renderer_mcu_stream() {
    let mut line_buf = LineBufferRenderer::<{ 320 * 8 }>::new(320, 8);
    assert_eq!(line_buf.width(), 320);
    assert_eq!(line_buf.lines(), 8);

    let mut display = DummyDisplay::new();
    let viewport = Rect::new(0, 0, 320, 64);

    let result = line_buf.render_stream(
        &mut display,
        viewport,
        Some(Rgb565::RED),
        |_target, _slice_rect| {
            // Line slice rendering hook
            Ok(())
        },
    );

    assert!(result.is_ok());

    // Verify display pixels were filled with RED within the 64 lines
    assert_eq!(display.pixels[0], Rgb565::RED);
    assert_eq!(display.pixels[320 * 63 + 319], Rgb565::RED);
    assert_eq!(display.pixels[320 * 64], Rgb565::BLACK);
}
