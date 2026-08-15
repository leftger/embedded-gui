#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::primitives::Rectangle;
use embedded_gui::prelude::*;
use embedded_gui::{PropertyKey, PropertyValue};
use panic_halt as _;

struct DummyDisplay;
impl OriginDimensions for DummyDisplay {
    fn size(&self) -> Size {
        Size::new(240, 240)
    }
}
impl DrawTarget for DummyDisplay {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        Ok(())
    }

    fn fill_solid(&mut self, _area: &Rectangle, _color: Self::Color) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let mut display = DummyDisplay;
    let mut gui = GuiContext::<32, 16, 16>::new(Rect::new(0, 0, 240, 240));

    let panel = gui
        .add_panel(Rect::new(0, 0, 240, 240), Style::panel())
        .unwrap();
    let label = gui
        .add_label(Rect::new(10, 10, 100, 20), "Status: Running", Style::label())
        .unwrap();
    let btn = gui
        .add_button(Rect::new(10, 40, 80, 30), "Action", Style::button())
        .unwrap();
    let progress = gui
        .add_progress_bar(Rect::new(10, 80, 150, 12), 0.65, Style::progress())
        .unwrap();
    let slider = gui
        .add_slider(Rect::new(10, 100, 150, 20), 0.0, 100.0, 50.0, Style::progress())
        .unwrap();

    let _ = gui.add_child(panel, label);
    let _ = gui.add_child(panel, btn);
    let _ = gui.add_child(panel, progress);
    let _ = gui.add_child(panel, slider);

    let layout = LinearLayout::column().with_gap(4);
    let mut out = [Rect::empty(); 4];
    layout.arrange(Rect::new(0, 0, 240, 240), 4, &mut out);

    let mut t: f32 = 0.0;
    loop {
        t = (t + 0.01) % 1.0;
        let _ = gui.set_widget_property(progress, PropertyKey::Value, PropertyValue::Float(t));
        let _ = gui.render(&mut display);
        cortex_m::asm::nop();
    }
}


