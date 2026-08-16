use embedded_3dgfx::mesh::{Geometry, K3dMesh, RenderMode};
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use embedded_gui::interop::three_d::Gui3dPipeline;
use embedded_gui::prelude::*;
use nalgebra::Point3;

const W: usize = 160;
const H: usize = 96;

static VERTS: [[f32; 3]; 4] = [
    [-0.8, -0.5, 0.0],
    [0.8, -0.5, 0.0],
    [0.8, 0.5, 0.0],
    [-0.8, 0.5, 0.0],
];
static LINES: [[usize; 2]; 4] = [[0, 1], [1, 2], [2, 3], [3, 0]];

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(W as u32, H as u32));
    let settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("embedded-3dgfx + embedded-gui overlay", &settings);

    let mut zbuffer = [u32::MAX; W * H];
    let mut pipeline = Gui3dPipeline::<8, 8, 8>::new(W, H, &mut zbuffer);
    pipeline
        .engine
        .camera
        .set_position(Point3::new(0.0, 0.0, 3.0));

    let geometry = Geometry {
        vertices: &VERTS,
        faces: &[],
        colors: &[],
        lines: &LINES,
        normals: &[],
        vertex_normals: &[],
        uvs: &[],
        texture_id: None,
    };
    let mut mesh = K3dMesh::new(geometry);
    mesh.set_render_mode(RenderMode::Lines);
    mesh.set_color(Rgb565::new(0, 48, 31));

    pipeline
        .gui
        .add_panel(Rect::new(4, 4, 88, 22), Style::panel())
        .unwrap();
    pipeline
        .gui
        .add_label(Rect::new(8, 8, 76, 8), "3D + GUI", Style::label())
        .unwrap();
    let progress = pipeline
        .gui
        .add_progress_bar(Rect::new(8, 18, 72, 5), 0.5, Style::progress())
        .unwrap();
    pipeline.gui.clear_dirty();

    let mut angle = 0.0f32;
    let mut pulse = Tween::new(0.1, 1.0, 1400, Easing::Smoothstep);
    'running: loop {
        display.clear(Rgb565::BLACK).unwrap();

        angle += 0.03;
        if pulse.tick(16) {
            pulse.reset();
        }
        pipeline.gui.set_progress(progress, pulse.value()).unwrap();

        mesh.set_attitude(0.0, angle, 0.0);
        pipeline.render_frame(&mut display, [&mesh]).unwrap();

        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::Escape,
                    ..
                } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
