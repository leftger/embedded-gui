use embedded_3dgfx::{
    K3dengine,
    command_buffer::CommandBuffer,
    mesh::{Geometry, K3dMesh, RenderMode},
    renderer::FrameCtx,
};
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
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

    let mut engine = K3dengine::new(W as u16, H as u16);
    engine.camera.set_position(Point3::new(0.0, 0.0, 3.0));

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

    let mut commands = CommandBuffer::<64>::new();
    let mut zbuffer = [u32::MAX; W * H];
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, W as u32, H as u32));
    gui.add_panel(Rect::new(4, 4, 88, 22), Style::panel())
        .unwrap();
    gui.add_label(Rect::new(8, 8, 76, 8), "3D + GUI", Style::label())
        .unwrap();
    let progress = gui
        .add_progress_bar(Rect::new(8, 18, 72, 5), 0.5, Style::progress())
        .unwrap();
    gui.clear_dirty();

    let mut angle = 0.0f32;
    let mut pulse = Tween::new(0.1, 1.0, 1400, Easing::Smoothstep);
    'running: loop {
        display.clear(Rgb565::BLACK).unwrap();
        commands.clear();

        angle += 0.03;
        if pulse.tick(16) {
            pulse.reset();
        }
        gui.set_progress(progress, pulse.value()).unwrap();
        let _partial_present_hint = gui.dirty_regions().first().copied();

        mesh.set_attitude(0.0, angle, 0.0);
        engine
            .record(core::iter::once(&mesh), &mut commands, None)
            .unwrap();
        let mut frame = FrameCtx {
            zbuffer: &mut zbuffer,
            width: W,
            height: H,
        };
        engine
            .execute(&mut display, &mut frame, &commands, None)
            .unwrap();
        gui.render(&mut display).unwrap();
        gui.clear_dirty();

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
