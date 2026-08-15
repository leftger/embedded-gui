#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_gui::prelude::*;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut gui = GuiContext::<16, 16, 16>::new(Rect::new(0, 0, 128, 64));
    let _ = gui.add_label(Rect::new(0, 0, 100, 20), "Size Check", Style::label());

    loop {
        cortex_m::asm::nop();
    }
}
