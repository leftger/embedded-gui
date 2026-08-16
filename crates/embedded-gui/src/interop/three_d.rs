//! High-level 3D + 2D interop pipeline, spatial anchors, and UI-to-texture rendering.

use core::fmt::Debug;
use embedded_3dgfx::{
    K3dengine, Ray, command_buffer::CommandBuffer, mesh::K3dMesh, renderer::FrameCtx,
};
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point},
    pixelcolor::Rgb565,
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
