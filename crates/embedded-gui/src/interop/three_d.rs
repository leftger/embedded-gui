//! High-level 3D + 2D interop pipeline, spatial anchors, and UI-to-texture rendering.

use core::fmt::Debug;
use embedded_3dgfx::{
    camera::Ray,
    command_buffer::{CommandBuffer, RenderCommand},
    engine::K3dengine,
    mesh::{K3dMesh, RenderMode},
    primitive::DrawPrimitive,
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
#[allow(unused_imports)]
use nalgebra::ComplexField;
use nalgebra::{Point2, Point3, Vector3};

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
    pub zbuffer: &'a mut [u32],
    pub commands: CommandBuffer<256>,
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

    /// Clear all pending render and gizmo commands.
    pub fn clear_commands(&mut self) {
        self.commands.clear();
    }

    /// Record 3D meshes into the pipeline command buffer.
    pub fn record_scene<'m>(&mut self, meshes: impl IntoIterator<Item = &'m K3dMesh<'m>>) {
        let mut meshes_iter = meshes.into_iter().peekable();
        if meshes_iter.peek().is_some() {
            let existing: heapless::Vec<RenderCommand, 64> =
                self.commands.iter().cloned().collect();
            self.engine
                .record(meshes_iter, &mut self.commands, None)
                .ok();
            for cmd in existing {
                let _ = self.commands.push(cmd);
            }
        }
    }

    /// Render 3D meshes and queued gizmos to `target` using Z-buffering.
    pub fn render_scene<'m, D>(
        &mut self,
        target: &mut D,
        meshes: impl IntoIterator<Item = &'m K3dMesh<'m>>,
    ) where
        D: DrawTarget<Color = Rgb565> + OriginDimensions,
        D::Error: Debug,
    {
        self.record_scene(meshes);

        let vp = self.gui.viewport();
        let mut frame = FrameCtx {
            zbuffer: self.zbuffer,
            width: vp.w as usize,
            height: vp.h as usize,
        };
        self.engine
            .execute(target, &mut frame, &self.commands, None)
            .ok();
        self.commands.clear();
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
        self.record_scene(meshes);

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
        self.commands.clear();

        Ok(())
    }

    /// Check whether a 3D world position is occluded by geometry in the depth buffer.
    ///
    /// Returns `true` if an existing rendered mesh is closer to the camera than `world_pos`.
    /// Returns `false` if `world_pos` is visible / in front of the geometry.
    pub fn is_world_point_occluded(&self, world_pos: Point3<f32>) -> bool {
        let arr = [world_pos.x, world_pos.y, world_pos.z];
        if let Some(screen_pt) = self
            .engine
            .transform_point(&arr, self.engine.camera.vp_matrix)
        {
            let width = self.gui.viewport().w as usize;
            let height = self.gui.viewport().h as usize;
            let x = screen_pt.x;
            let y = screen_pt.y;
            if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
                let idx = y as usize * width + x as usize;
                if idx < self.zbuffer.len() {
                    let depth = embedded_3dgfx::to_zdepth((screen_pt.z as u32) << 16);
                    return depth > self.zbuffer[idx].saturating_add(embedded_3dgfx::DEPTH_EPSILON);
                }
            }
        }
        true
    }

    /// Render a solid color billboard quad at `billboard.position` with depth testing against the Z-buffer.
    pub fn render_billboard_quad<D>(
        &mut self,
        target: &mut D,
        billboard: &Billboard3d,
        color: Rgb565,
    ) -> Result<bool, D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let arr = [
            billboard.position.x,
            billboard.position.y,
            billboard.position.z,
        ];
        let Some(pt) = self
            .engine
            .transform_point(&arr, self.engine.camera.vp_matrix)
        else {
            return Ok(false);
        };

        let vp = self.gui.viewport();
        let width = vp.w as usize;
        let height = vp.h as usize;
        let z_depth = embedded_3dgfx::to_zdepth((pt.z as u32) << 16);

        let half_w = (billboard.size.width / 2) as i32;
        let half_h = (billboard.size.height / 2) as i32;
        let origin_x = pt.x + billboard.offset.x - half_w;
        let origin_y = pt.y + billboard.offset.y - half_h;

        let x0 = origin_x.max(0);
        let y0 = origin_y.max(0);
        let x1 = (origin_x + billboard.size.width as i32).min(width as i32);
        let y1 = (origin_y + billboard.size.height as i32).min(height as i32);

        if x0 >= x1 || y0 >= y1 {
            return Ok(false);
        }

        let mut drawn_any = false;
        for y in y0..y1 {
            let row_idx = y as usize * width;
            for x in x0..x1 {
                let idx = row_idx + x as usize;
                if idx < self.zbuffer.len()
                    && z_depth <= self.zbuffer[idx].saturating_add(embedded_3dgfx::DEPTH_EPSILON)
                {
                    if billboard.depth_write {
                        self.zbuffer[idx] = z_depth;
                    }
                    target.draw_iter(core::iter::once(Pixel(Point::new(x, y), color)))?;
                    drawn_any = true;
                }
            }
        }

        Ok(drawn_any)
    }

    /// Render a 2D pixel buffer (e.g. from an off-screen GUI widget or icon) as a 3D world-space billboard
    /// with per-pixel depth testing against the Z-buffer.
    pub fn render_billboard_buffer<D>(
        &mut self,
        target: &mut D,
        billboard: &Billboard3d,
        buffer: &[Rgb565],
        transparent_color: Option<Rgb565>,
    ) -> Result<bool, D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let arr = [
            billboard.position.x,
            billboard.position.y,
            billboard.position.z,
        ];
        let Some(pt) = self
            .engine
            .transform_point(&arr, self.engine.camera.vp_matrix)
        else {
            return Ok(false);
        };

        let vp = self.gui.viewport();
        let width = vp.w as usize;
        let height = vp.h as usize;
        let z_depth = embedded_3dgfx::to_zdepth((pt.z as u32) << 16);

        let buf_w = billboard.size.width as usize;
        let buf_h = billboard.size.height as usize;
        if buffer.len() < buf_w * buf_h {
            return Ok(false);
        }

        let half_w = (billboard.size.width / 2) as i32;
        let half_h = (billboard.size.height / 2) as i32;
        let origin_x = pt.x + billboard.offset.x - half_w;
        let origin_y = pt.y + billboard.offset.y - half_h;

        let mut drawn_any = false;
        for by in 0..buf_h {
            let y = origin_y + by as i32;
            if y < 0 || y as usize >= height {
                continue;
            }
            let row_idx = y as usize * width;
            for bx in 0..buf_w {
                let x = origin_x + bx as i32;
                if x < 0 || x as usize >= width {
                    continue;
                }
                let color = buffer[by * buf_w + bx];
                if let Some(trans) = transparent_color {
                    if color == trans {
                        continue;
                    }
                }
                let idx = row_idx + x as usize;
                if idx < self.zbuffer.len()
                    && z_depth <= self.zbuffer[idx].saturating_add(embedded_3dgfx::DEPTH_EPSILON)
                {
                    if billboard.depth_write {
                        self.zbuffer[idx] = z_depth;
                    }
                    target.draw_iter(core::iter::once(Pixel(Point::new(x, y), color)))?;
                    drawn_any = true;
                }
            }
        }

        Ok(drawn_any)
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

/// Configuration for a 3D world-space billboard with depth testing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Billboard3d {
    /// World position of the billboard center.
    pub position: Point3<f32>,
    /// Display size on screen in pixels (width, height).
    pub size: Size,
    /// Pixel offset from the projected center (e.g. to anchor at bottom instead of center).
    pub offset: Point,
    /// Whether to write the billboard's depth to the Z-buffer.
    pub depth_write: bool,
}

impl Billboard3d {
    /// Create a new billboard at `position` with the given screen `size`.
    pub fn new(position: Point3<f32>, size: Size) -> Self {
        Self {
            position,
            size,
            offset: Point::zero(),
            depth_write: true,
        }
    }

    /// Set a 2D pixel offset relative to the projected screen position.
    pub fn with_offset(mut self, offset: Point) -> Self {
        self.offset = offset;
        self
    }

    /// Enable or disable Z-buffer depth writing for the billboard.
    pub fn with_depth_write(mut self, depth_write: bool) -> Self {
        self.depth_write = depth_write;
        self
    }
}

/// Trait for immediate-mode 3D debug gizmo drawing into a 3D pipeline.
pub trait Gui3dGizmos {
    /// Draw a line segment between two 3D world coordinates.
    fn draw_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Rgb565) -> bool;

    /// Draw a 3D ray starting at `origin` along `direction` for `length` units.
    fn draw_ray(
        &mut self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
        length: f32,
        color: Rgb565,
    ) -> bool;

    /// Draw an axis-aligned wireframe bounding box between `min` and `max`.
    fn draw_wireframe_box(&mut self, min: Point3<f32>, max: Point3<f32>, color: Rgb565) -> bool;

    /// Draw an axis-aligned wireframe cube centered at `center` with side length `size`.
    fn draw_wireframe_cube(&mut self, center: Point3<f32>, size: f32, color: Rgb565) -> bool;

    /// Draw RGB 3D coordinate axes (+X red, +Y green, +Z blue) starting from `origin`.
    fn draw_axes(&mut self, origin: Point3<f32>, length: f32) -> bool;

    /// Draw a ground grid on the XZ plane centered at `center`.
    fn draw_grid(
        &mut self,
        center: Point3<f32>,
        cell_size: f32,
        half_count: i32,
        grid_color: Rgb565,
        axis_color: Option<Rgb565>,
    ) -> usize;
}

impl<'a, const MAX_NODES: usize, const MAX_HANDLERS: usize, const MAX_ACTIVE: usize> Gui3dGizmos
    for Gui3dPipeline<'a, MAX_NODES, MAX_HANDLERS, MAX_ACTIVE>
{
    fn draw_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Rgb565) -> bool {
        let p0 = self
            .engine
            .transform_point(&[start.x, start.y, start.z], self.engine.camera.vp_matrix);
        let p1 = self
            .engine
            .transform_point(&[end.x, end.y, end.z], self.engine.camera.vp_matrix);
        if let (Some(s0), Some(s1)) = (p0, p1) {
            let prim =
                DrawPrimitive::Line([Point2::new(s0.x, s0.y), Point2::new(s1.x, s1.y)], color);
            self.commands.push(RenderCommand::Draw(prim)).is_ok()
        } else {
            false
        }
    }

    fn draw_ray(
        &mut self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
        length: f32,
        color: Rgb565,
    ) -> bool {
        let dot = direction.dot(&direction);
        let dir = if dot > 1e-6 {
            let len = ComplexField::sqrt(dot);
            direction / len
        } else {
            direction
        };
        let end = origin + dir * length;
        self.draw_line(origin, end, color)
    }

    fn draw_wireframe_box(&mut self, min: Point3<f32>, max: Point3<f32>, color: Rgb565) -> bool {
        let corners = [
            Point3::new(min.x, min.y, min.z),
            Point3::new(max.x, min.y, min.z),
            Point3::new(max.x, max.y, min.z),
            Point3::new(min.x, max.y, min.z),
            Point3::new(min.x, min.y, max.z),
            Point3::new(max.x, min.y, max.z),
            Point3::new(max.x, max.y, max.z),
            Point3::new(min.x, max.y, max.z),
        ];
        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        let mut any_drawn = false;
        for &(a, b) in &edges {
            if self.draw_line(corners[a], corners[b], color) {
                any_drawn = true;
            }
        }
        any_drawn
    }

    fn draw_wireframe_cube(&mut self, center: Point3<f32>, size: f32, color: Rgb565) -> bool {
        let half = size * 0.5;
        let min = Point3::new(center.x - half, center.y - half, center.z - half);
        let max = Point3::new(center.x + half, center.y + half, center.z + half);
        self.draw_wireframe_box(min, max, color)
    }

    fn draw_axes(&mut self, origin: Point3<f32>, length: f32) -> bool {
        let red = Rgb565::new(31, 0, 0);
        let green = Rgb565::new(0, 63, 0);
        let blue = Rgb565::new(0, 0, 31);

        let px = Point3::new(origin.x + length, origin.y, origin.z);
        let py = Point3::new(origin.x, origin.y + length, origin.z);
        let pz = Point3::new(origin.x, origin.y, origin.z + length);

        let d1 = self.draw_line(origin, px, red);
        let d2 = self.draw_line(origin, py, green);
        let d3 = self.draw_line(origin, pz, blue);
        d1 || d2 || d3
    }

    fn draw_grid(
        &mut self,
        center: Point3<f32>,
        cell_size: f32,
        half_count: i32,
        grid_color: Rgb565,
        axis_color: Option<Rgb565>,
    ) -> usize {
        let extent = half_count as f32 * cell_size;
        let min_x = center.x - extent;
        let max_x = center.x + extent;
        let min_z = center.z - extent;
        let max_z = center.z + extent;
        let y = center.y;
        let mut count = 0;

        for i in -half_count..=half_count {
            let x = center.x + i as f32 * cell_size;
            let color = if i == 0 {
                axis_color.unwrap_or(grid_color)
            } else {
                grid_color
            };
            if self.draw_line(Point3::new(x, y, min_z), Point3::new(x, y, max_z), color) {
                count += 1;
            }
        }

        for j in -half_count..=half_count {
            let z = center.z + j as f32 * cell_size;
            let color = if j == 0 {
                axis_color.unwrap_or(grid_color)
            } else {
                grid_color
            };
            if self.draw_line(Point3::new(min_x, y, z), Point3::new(max_x, y, z), color) {
                count += 1;
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::pixelcolor::RgbColor;

    struct MockTarget {
        pixels: [(Point, Rgb565); 1024],
        count: usize,
    }

    impl MockTarget {
        fn new() -> Self {
            Self {
                pixels: [(Point::zero(), Rgb565::BLACK); 1024],
                count: 0,
            }
        }
    }

    impl OriginDimensions for MockTarget {
        fn size(&self) -> Size {
            Size::new(64, 64)
        }
    }

    impl DrawTarget for MockTarget {
        type Color = Rgb565;
        type Error = core::convert::Infallible;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            for Pixel(pt, c) in pixels {
                if self.count < self.pixels.len() {
                    self.pixels[self.count] = (pt, c);
                    self.count += 1;
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_gui3d_gizmos_recording() {
        let mut zbuffer = [0u32; 64 * 64];
        let mut pipeline: Gui3dPipeline<'_, 16, 8, 4> = Gui3dPipeline::new(64, 64, &mut zbuffer);

        pipeline
            .engine
            .camera
            .set_position(Point3::new(0.0, 0.0, 5.0));
        pipeline.engine.camera.set_target(Point3::origin());

        // Draw axes, cube, and ray gizmos
        assert!(pipeline.draw_axes(Point3::origin(), 1.0));
        assert!(pipeline.draw_wireframe_cube(Point3::origin(), 1.0, Rgb565::WHITE));
        assert!(pipeline.draw_ray(
            Point3::origin(),
            Vector3::new(0.0, 1.0, 0.0),
            2.0,
            Rgb565::YELLOW
        ));
        assert_eq!(pipeline.commands.len(), 16); // 3 axes + 12 cube edges + 1 ray

        let mut target = MockTarget::new();
        pipeline.render_scene(&mut target, []);
        assert!(target.count > 0);
        assert_eq!(pipeline.commands.len(), 0);
    }

    #[test]
    fn test_gui3d_billboard_depth_testing() {
        let mut zbuffer = [u32::MAX; 64 * 64];
        let mut pipeline: Gui3dPipeline<'_, 16, 8, 4> = Gui3dPipeline::new(64, 64, &mut zbuffer);

        pipeline
            .engine
            .camera
            .set_position(Point3::new(0.0, 0.0, 5.0));
        pipeline.engine.camera.set_target(Point3::origin());

        let billboard = Billboard3d::new(Point3::new(0.0, 0.0, 0.0), Size::new(4, 4));

        let mut target = MockTarget::new();
        let drawn = pipeline
            .render_billboard_quad(&mut target, &billboard, Rgb565::GREEN)
            .unwrap();
        assert!(drawn);
        assert_eq!(target.count, 16); // 4x4 pixels drawn

        // Center pixel now has written depth
        let center_idx = 32 * 64 + 32;
        let written_depth = pipeline.zbuffer[center_idx];
        assert!(written_depth < u32::MAX);

        // Billboard behind existing depth should be occluded
        let behind_billboard = Billboard3d::new(Point3::new(0.0, 0.0, -2.0), Size::new(4, 4));
        assert!(pipeline.is_world_point_occluded(behind_billboard.position));

        let mut target2 = MockTarget::new();
        let drawn_behind = pipeline
            .render_billboard_quad(&mut target2, &behind_billboard, Rgb565::RED)
            .unwrap();
        assert!(!drawn_behind);
        assert_eq!(target2.count, 0); // Completely occluded!
    }

    #[test]
    fn test_billboard_buffer_and_mesh_panel_errors() {
        let mut zbuffer = [u32::MAX; 64 * 64];
        let mut pipeline: Gui3dPipeline<'_, 16, 8, 4> = Gui3dPipeline::new(64, 64, &mut zbuffer);
        pipeline
            .engine
            .camera
            .set_position(Point3::new(0.0, 0.0, 5.0));
        pipeline.engine.camera.set_target(Point3::origin());

        let billboard = Billboard3d::new(Point3::new(0.0, 0.0, 0.0), Size::new(2, 2))
            .with_offset(Point::new(1, 1))
            .with_depth_write(false);
        assert_eq!(billboard.offset, Point::new(1, 1));
        assert!(!billboard.depth_write);

        let mut pixels = [Rgb565::GREEN; 4];
        pixels[0] = Rgb565::BLACK;
        let mut target = MockTarget::new();
        let drawn = pipeline
            .render_billboard_buffer(&mut target, &billboard, &pixels, Some(Rgb565::BLACK))
            .unwrap();
        assert!(drawn);

        let too_small = pipeline
            .render_billboard_buffer(&mut target, &billboard, &[Rgb565::GREEN; 1], None)
            .unwrap();
        assert!(!too_small);

        let mut target2 = MockTarget::new();
        static VERTICES: [[f32; 3]; 1] = [[0.0, 0.0, 0.0]];
        let geometry = Geometry {
            vertices: &VERTICES,
            faces: &[],
            colors: &[],
            lines: &[],
            normals: &[],
            vertex_normals: &[],
            uvs: &[],
            texture_id: None,
        };
        let panel = MeshPanel::new(geometry, Rgb565::WHITE);
        assert!(render_mesh_panel(&mut target2, Rect::new(0, 0, 0, 0), &panel, &mut []).is_ok());

        let mut small_zb = [0u32; 2];
        assert_eq!(
            render_mesh_panel(&mut target2, Rect::new(0, 0, 4, 4), &panel, &mut small_zb)
                .unwrap_err(),
            MeshPanelError::ZBufferTooSmall { needed: 16, got: 2 }
        );

        for shading in [
            MeshShading::Points,
            MeshShading::Lines,
            MeshShading::Solid,
            MeshShading::Lit,
        ] {
            let mut panel = MeshPanel::new(geometry, Rgb565::WHITE);
            panel.shading = shading;
            let mut zb = [0u32; 16];
            let _ = render_mesh_panel(&mut target2, Rect::new(0, 0, 4, 4), &panel, &mut zb);
        }
    }

    #[test]
    fn test_dispatch_pointer_and_texture_rendering() {
        let mut zbuffer = [0u32; 16 * 16];
        let mut pipeline: Gui3dPipeline<'_, 8, 4, 2> = Gui3dPipeline::new(16, 16, &mut zbuffer);
        pipeline
            .engine
            .camera
            .set_position(Point3::new(0.0, 0.0, 5.0));
        pipeline.engine.camera.set_target(Point3::origin());

        let scene = dispatch_pointer_input(
            &mut pipeline.gui,
            &pipeline.engine,
            8,
            8,
            PointerState::Pressed,
            PointerButton::Primary,
        );
        assert!(matches!(scene, InputResult::ScenePick(_)));

        let ignored = dispatch_pointer_input(
            &mut pipeline.gui,
            &pipeline.engine,
            8,
            8,
            PointerState::Released,
            PointerButton::Primary,
        );
        assert!(matches!(ignored, InputResult::Ignored));

        let mut texture = [Rgb565::BLACK; 4];
        assert!(pipeline.gui.render_to_texture(&mut texture, 2, 2).is_ok());
        assert!(pipeline.gui.render_to_texture(&mut texture, 3, 2).is_err());
    }

    #[test]
    fn test_more_gizmos_grid_and_wireframe_box() {
        let mut zbuffer = [0u32; 64 * 64];
        let mut pipeline: Gui3dPipeline<'_, 16, 8, 4> = Gui3dPipeline::new(64, 64, &mut zbuffer);
        pipeline
            .engine
            .camera
            .set_position(Point3::new(0.0, 0.0, 5.0));
        pipeline.engine.camera.set_target(Point3::origin());

        assert!(pipeline.draw_wireframe_box(
            Point3::new(-1.0, -1.0, -1.0),
            Point3::new(1.0, 1.0, 1.0),
            Rgb565::WHITE
        ));

        let count = pipeline.draw_grid(
            Point3::origin(),
            1.0,
            2,
            Rgb565::new(20, 20, 20),
            Some(Rgb565::WHITE),
        );
        assert_eq!(count, 10);

        pipeline.clear_commands();
        let hidden = pipeline.draw_line(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 101.0),
            Rgb565::WHITE,
        );
        assert!(!hidden);
    }
}
