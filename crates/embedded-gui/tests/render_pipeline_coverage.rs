//! Coverage for dirty-region and offset rendering pipelines.

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_gui::prelude::*;

mod common;
use common::MockTarget;

#[test]
fn dirty_and_buffered_render_pipelines() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 64));
    let btn = gui
        .add_button(Rect::new(0, 0, 30, 10), "Btn", Style::button())
        .unwrap();
    gui.set_focus(Some(btn)).unwrap();

    let mut target = MockTarget::new(64, 64);
    gui.render(&mut target).unwrap();
    assert!(!target.pixels.is_empty());

    gui.render_dirty(&mut target).unwrap();
    gui.render_with_offset(&mut target, 2, 2).unwrap();
    gui.render_with_offset_and_opacity(&mut target, 2, 2, 128)
        .unwrap();
    gui.render_with_offset_opacity_and_clip(&mut target, 2, 2, 128, Rect::new(0, 0, 20, 20))
        .unwrap();

    let mut large = [Rgb565::BLACK; 64 * 64];
    gui.render_dirty_buffered(&mut target, &mut large).unwrap();

    let mut row_sized = [Rgb565::BLACK; 32];
    gui.render_dirty_buffered(&mut target, &mut row_sized)
        .unwrap();

    let mut micro = [Rgb565::BLACK; 2];
    gui.render_dirty_buffered(&mut target, &mut micro).unwrap();

    gui.clear_dirty();
    gui.render_dirty_buffered(&mut target, &mut micro).unwrap();
    gui.render_dirty(&mut target).unwrap();
}
