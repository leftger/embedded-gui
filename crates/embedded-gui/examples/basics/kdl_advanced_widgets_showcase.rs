//! Compiles the carousel, imported-font, composite-icon, and 3D-mesh KDL nodes
//! into firmware-shaped Rust and renders one frame of each.
//!
//! The KDL and its art live in the Studio demo project, so this example proves
//! that what a designer sees in the preview is exactly what `include_gui!`
//! bakes into flash.

use embedded_gui::interop::three_d::render_mesh_panel;
use embedded_gui::prelude::*;

mod counter {
    use embedded_gui::prelude::*;
    embedded_gui::include_gui!("../embedded-gui-studio/examples/ssd1357-demo/screens/counter.kdl");
}

mod menu {
    use embedded_gui::prelude::*;
    embedded_gui::include_gui!("../embedded-gui-studio/examples/ssd1357-demo/screens/menu.kdl");
}

const WIDTH: u32 = 96;
const HEIGHT: u32 = 64;

fn lit_pixels(buffer: &TestBuffer) -> usize {
    let black = Rgb565::new(0, 0, 0);
    (0..HEIGHT as i32)
        .flat_map(|y| (0..WIDTH as i32).map(move |x| (x, y)))
        .filter(|(x, y)| buffer.pixel_at(*x, *y).is_some_and(|px| px != black))
        .count()
}

fn main() {
    let mut gui = GuiContext::<64, 16, 64>::new(Rect::new(0, 0, WIDTH, HEIGHT));
    let app = counter::CounterApp::build(&mut gui).expect("counter screen fits its budgets");

    let mut buffer = TestBuffer::new(WIDTH, HEIGHT);
    gui.render(&mut buffer).unwrap();

    // The mesh needs a Z-buffer, so it is drawn by the app rather than the
    // widget tree; its rect comes from the spacer the node reserved.
    let rect = gui
        .absolute_rect(app.widgets.gem)
        .expect("the mesh node reserved a rect");
    let mut zbuffer = vec![0u32; (rect.w * rect.h) as usize];
    render_mesh_panel(&mut buffer, rect, &counter::gem_mesh_panel(), &mut zbuffer)
        .expect("mesh fits the command budget");

    println!(
        "counter screen: {} lit pixels, mesh rect {}x{}",
        lit_pixels(&buffer),
        rect.w,
        rect.h
    );

    let mut menu_gui = GuiContext::<64, 16, 64>::new(Rect::new(0, 0, WIDTH, HEIGHT));
    let _menu = menu::MenuApp::build(&mut menu_gui).expect("menu screen fits its budgets");
    let mut menu_buffer = TestBuffer::new(WIDTH, HEIGHT);
    menu_gui.render(&mut menu_buffer).unwrap();
    println!("menu screen: {} lit pixels", lit_pixels(&menu_buffer));
}
