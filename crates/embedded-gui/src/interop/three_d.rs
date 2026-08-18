//! High-level 3D + 2D interop pipeline, spatial anchors, and UI-to-texture rendering.

use core::fmt::Debug;
use embedded_3dgfx::{
    K3dengine, Ray,
    command_buffer::CommandBuffer,
    mesh::{K3dMesh, RenderMode},
    renderer::FrameCtx,
};
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::Rgb565,
    primitives::Rectangle,
};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics_framebuf::backends::FrameBufferBackend;
use nalgebra::Point3;

use crate::{
    context::GuiContext,
    geometry::Rect,
    input::{InputEvent, PointerButton, PointerState, UiEvent},
    widget::WidgetId,
};

/// High-level combined 3D scene + 2D GUI overlay rendering pipeline.
pub struct Gui3dPipeline<
    'a,
    const MAX_NODES: usize,
    const MAX_HANDLERS: usize,
    const MAX_ACTIVE: usize,
> {
    /// 3D rendering engine instance.
    pub engine: K3dengine,
    /// 2D GUI context instance.
    pub gui: GuiContext<'a, MAX_NODES, MAX_HANDLERS, MAX_ACTIVE>,
    zbuffer: &'a mut [u32],
    commands: CommandBuffer<64>,
}

impl<'a, const MAX_NODES: usize, const MAX_HANDLERS: usize, const MAX_ACTIVE: usize>
    Gui3dPipeline<'a, MAX_NODES, MAX_HANDLERS, MAX_ACTIVE>
{
    /// Create a new pipeline given screen dimensions and a Z-buffer slice.
    pub fn new(width: usize, height: usize, zbuffer: &'a mut [u32]) -> Self {
        Self {
            engine: K3dengine::new(width as u16, height as u16),
            gui: GuiContext::new(Rect::new(0, 0, width as u32, height as u32)),
            zbuffer,
            commands: CommandBuffer::new(),
        }
    }

    /// Render a frame containing 3D meshes followed by dirty 2D GUI widgets onto `target`.
    pub fn render_frame<'m, D>(
        &mut self,
        target: &mut D,
        meshes: impl IntoIterator<Item = &'m K3dMesh<'m>>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565> + OriginDimensions,
        D::Error: Debug,
    {
        self.commands.clear();
        self.engine.record(meshes, &mut self.commands, None).ok();

        let vp = self.gui.viewport();
        let mut frame = FrameCtx {
            zbuffer: self.zbuffer,
            width: vp.w as usize,
            height: vp.h as usize,
        };
        self.engine
            .execute(target, &mut frame, &self.commands, None)
            .ok();

        self.gui.render(target)?;
        self.gui.clear_dirty();

        Ok(())
    }
}

/// Re-exported so generated code and app code can describe geometry without
/// depending on `embedded-3dgfx` directly.
pub use embedded_3dgfx::mesh::Geometry;

/// Shading used by [`MeshPanel`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeshShading {
    Points,
    Lines,
    #[default]
    Solid,
    /// Flat-shaded against a fixed light direction; needs per-face normals.
    Lit,
}

/// A single mesh framed inside a GUI rect: the 3D counterpart of an image
/// widget, used for spinning logos and other decorative geometry.
///
/// The camera looks at the origin from `camera_distance` along +Z, so a mesh
/// centered on the origin stays framed no matter how it is rotated.
pub struct MeshPanel<'a> {
    pub geometry: Geometry<'a>,
    pub color: Rgb565,
    pub shading: MeshShading,
    pub scale: f32,
    /// Roll, pitch, yaw in radians. Animate these for spin or coin-flip motion.
    pub attitude: (f32, f32, f32),
    pub camera_distance: f32,
    /// Vertical field of view in radians.
    pub fov: f32,
    pub light_dir: [f32; 3],
}

impl<'a> MeshPanel<'a> {
    pub fn new(geometry: Geometry<'a>, color: Rgb565) -> Self {
        Self {
            geometry,
            color,
            shading: MeshShading::Solid,
            scale: 1.0,
            attitude: (0.0, 0.0, 0.0),
            camera_distance: 4.0,
            fov: core::f32::consts::FRAC_PI_2,
            light_dir: [0.0, 0.0, -1.0],
        }
    }
}

/// Clips and translates draws into a sub-rect, so the 3D rasterizer can render
/// at panel-local coordinates while writing into the shared framebuffer.
struct OffsetTarget<'d, D> {
    inner: &'d mut D,
    origin: Point,
    size: Size,
}

impl<D> OriginDimensions for OffsetTarget<'_, D> {
    fn size(&self) -> Size {
        self.size
    }
}

impl<D> DrawTarget for OffsetTarget<'_, D>
where
    D: DrawTarget<Color = Rgb565>,
{
    type Color = Rgb565;
    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let w = self.size.width as i32;
        let h = self.size.height as i32;
        let origin = self.origin;
        self.inner.draw_iter(pixels.into_iter().filter_map(
            move |Pixel(point, color)| match point {
                p if p.x >= 0 && p.y >= 0 && p.x < w && p.y < h => {
                    Some(Pixel(Point::new(p.x + origin.x, p.y + origin.y), color))
                }
                _ => None,
            },
        ))
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let clipped = area.intersection(&Rectangle::new(Point::zero(), self.size));
        if clipped.size.width == 0 || clipped.size.height == 0 {
            return Ok(());
        }
        self.inner.fill_solid(
            &Rectangle::new(
                Point::new(
                    clipped.top_left.x + self.origin.x,
                    clipped.top_left.y + self.origin.y,
                ),
                clipped.size,
            ),
            color,
        )
    }
}

/// Renders `panel` into `rect` of `target`.
///
/// `zbuffer` must hold at least `rect.w * rect.h` entries; it is cleared by the
/// engine on each call, so one scratch buffer can be shared by every panel.
pub fn render_mesh_panel<D>(
    target: &mut D,
    rect: Rect,
    panel: &MeshPanel<'_>,
    zbuffer: &mut [u32],
) -> Result<(), MeshPanelError>
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: Debug,
{
    if rect.is_empty() {
        return Ok(());
    }
    let pixels = rect.w as usize * rect.h as usize;
    if zbuffer.len() < pixels {
        return Err(MeshPanelError::ZBufferTooSmall {
            needed: pixels,
            got: zbuffer.len(),
        });
    }

    let mut engine = K3dengine::new(rect.w as u16, rect.h as u16);
    engine
        .camera
        .set_position(nalgebra::Point3::new(0.0, 0.0, panel.camera_distance));
    engine.camera.set_target(nalgebra::Point3::origin());
    engine.camera.set_fovy(panel.fov);

    let mut mesh = K3dMesh::new(panel.geometry);
    mesh.set_color(panel.color);
    mesh.set_scale(panel.scale);
    let (roll, pitch, yaw) = panel.attitude;
    mesh.set_attitude(roll, pitch, yaw);
    mesh.set_render_mode(match panel.shading {
        MeshShading::Points => RenderMode::Points,
        MeshShading::Lines => RenderMode::Lines,
        MeshShading::Solid => RenderMode::Solid,
        MeshShading::Lit => RenderMode::SolidLightDir(nalgebra::Vector3::new(
            panel.light_dir[0],
            panel.light_dir[1],
            panel.light_dir[2],
        )),
    });

    let mut commands = CommandBuffer::<512>::new();
    engine
        .record([&mesh], &mut commands, None)
        .map_err(|_| MeshPanelError::RecordFailed)?;

    let mut frame = FrameCtx {
        zbuffer: &mut zbuffer[..pixels],
        width: rect.w as usize,
        height: rect.h as usize,
    };
    let mut view = OffsetTarget {
        inner: target,
        origin: Point::new(rect.x, rect.y),
        size: Size::new(rect.w, rect.h),
    };
    engine
        .execute(&mut view, &mut frame, &commands, None)
        .map_err(|_| MeshPanelError::ExecuteFailed)?;
    Ok(())
}

/// Why a [`render_mesh_panel`] call could not draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshPanelError {
    ZBufferTooSmall {
        needed: usize,
        got: usize,
    },
    /// Geometry exceeded the command buffer budget.
    RecordFailed,
    ExecuteFailed,
}

/// Extension trait for spatial 3D world anchors on `GuiContext`.
pub trait WorldAnchorExt {
    /// Anchor a widget to a 3D world coordinate using camera projection.
    fn anchor_widget_to_world(
        &mut self,
        widget_id: WidgetId,
        world_pos: Point3<f32>,
        engine: &K3dengine,
        offset: Point,
    ) -> bool;
}

impl<'a, const MAX_NODES: usize, const MAX_HANDLERS: usize, const MAX_ACTIVE: usize> WorldAnchorExt
    for GuiContext<'a, MAX_NODES, MAX_HANDLERS, MAX_ACTIVE>
{
    fn anchor_widget_to_world(
        &mut self,
        widget_id: WidgetId,
        world_pos: Point3<f32>,
        engine: &K3dengine,
        offset: Point,
    ) -> bool {
        if let Some(screen_pt) = engine.project_point(world_pos) {
            let pos = Point::new(screen_pt.x + offset.x, screen_pt.y + offset.y);
            if let Some(rect) = self.absolute_rect(widget_id) {
                let new_rect = Rect::new(pos.x, pos.y, rect.w, rect.h);
                if let Some(node) = self.node_mut(widget_id) {
                    node.rect = new_rect;
                    let _ = self.dirty.add(new_rect);
                    return true;
                }
            }
        }
        false
    }
}

/// Result of dispatching an input event in a 3D+GUI application.
#[derive(Clone, Copy, Debug)]
pub enum InputResult {
    /// Handled by 2D GUI.
    GuiHandled(UiEvent),
    /// Unhandled by GUI; converted into a 3D pick ray.
    ScenePick(Ray),
    /// Event ignored / unhandled.
    Ignored,
}

/// Router for dispatching pointer events to 2D GUI first, then 3D picking rays.
pub fn dispatch_pointer_input<
    'a,
    const MAX_NODES: usize,
    const MAX_HANDLERS: usize,
    const MAX_ACTIVE: usize,
>(
    gui: &mut GuiContext<'a, MAX_NODES, MAX_HANDLERS, MAX_ACTIVE>,
    engine: &K3dengine,
    x: i32,
    y: i32,
    state: PointerState,
    button: PointerButton,
) -> InputResult {
    let evt = InputEvent::Pointer {
        x,
        y,
        state,
        button,
    };
    let _ = gui.handle_input(evt);
    if let Some(ui_evt) = gui.pop_event() {
        InputResult::GuiHandled(ui_evt)
    } else if state == PointerState::Pressed {
        let vp = gui.viewport();
        let ray = Ray::from_screen_point(
            Point::new(x, y),
            &engine.camera,
            vp.w as usize,
            vp.h as usize,
        );
        InputResult::ScenePick(ray)
    } else {
        InputResult::Ignored
    }
}

struct SliceBackend<'b>(&'b mut [Rgb565]);

impl FrameBufferBackend for SliceBackend<'_> {
    type Color = Rgb565;
    fn set(&mut self, index: usize, color: Rgb565) {
        self.0[index] = color;
    }
    fn get(&self, index: usize) -> Rgb565 {
        self.0[index]
    }
    fn nr_elements(&self) -> usize {
        self.0.len()
    }
}

/// Extension trait for rendering GUI trees directly into textures for 3D meshes.
#[allow(clippy::result_unit_err)]
pub trait RenderToTextureExt {
    /// Render the GUI context into a slice buffer to be used as a 3D texture.
    fn render_to_texture(
        &mut self,
        buffer: &mut [Rgb565],
        width: usize,
        height: usize,
    ) -> Result<(), ()>;
}

impl<'a, const MAX_NODES: usize, const MAX_HANDLERS: usize, const MAX_ACTIVE: usize>
    RenderToTextureExt for GuiContext<'a, MAX_NODES, MAX_HANDLERS, MAX_ACTIVE>
{
    #[allow(clippy::result_unit_err)]
    fn render_to_texture(
        &mut self,
        buffer: &mut [Rgb565],
        width: usize,
        height: usize,
    ) -> Result<(), ()> {
        if buffer.len() != width * height {
            return Err(());
        }
        let mut fb = FrameBuf::new(SliceBackend(buffer), width, height);
        self.render(&mut fb).map_err(|_| ())?;
        Ok(())
    }
}
