//! Precise Draw Commands (PDC) vector graphics format and renderer.
//!
//! Compact, binary-efficient vector format supporting arbitrary paths, circles,
//! and 13.3 fixed-point subpixel precision coordinates.

use embedded_graphics_core::{draw_target::DrawTarget, geometry::Point, pixelcolor::Rgb565};
use heapless::Vec;

use crate::{
    geometry::Rect,
    render::{RenderCtx, StrokeStyle},
};

/// Type of Precise Draw Command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdcCommandType {
    /// Arbitrary line/polygon path with standard integer coordinates.
    Path,
    /// Circle with center point and radius.
    Circle,
    /// Arbitrary path with 13.3 fixed-point subpixel precision (1/8th pixel).
    PrecisePath,
}

/// Error indicating capacity limit exceeded during PDC construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PdcError;

/// A 13.3 fixed-point coordinate point (1 unit = 1/8th pixel).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PdcPrecisePoint {
    pub x_fixed: i16,
    pub y_fixed: i16,
}

impl PdcPrecisePoint {
    pub const fn from_subpixels(x_fixed: i16, y_fixed: i16) -> Self {
        Self { x_fixed, y_fixed }
    }

    pub const fn from_pixels(x: i16, y: i16) -> Self {
        Self {
            x_fixed: x << 3,
            y_fixed: y << 3,
        }
    }

    pub const fn to_pixel_point(self) -> Point {
        Point::new((self.x_fixed >> 3) as i32, (self.y_fixed >> 3) as i32)
    }

    pub fn to_f32_point(self) -> (f32, f32) {
        (self.x_fixed as f32 / 8.0, self.y_fixed as f32 / 8.0)
    }
}

/// A single Precise Draw Command encoding stroke, fill, and geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct PdcCommand<const MAX_POINTS: usize = 16> {
    pub command_type: PdcCommandType,
    pub stroke_color: Option<Rgb565>,
    pub fill_color: Option<Rgb565>,
    pub stroke_width: u8,
    pub radius: u16,
    pub is_closed: bool,
    pub points: Vec<PdcPrecisePoint, MAX_POINTS>,
}

impl<const MAX_POINTS: usize> PdcCommand<MAX_POINTS> {
    pub const fn new(command_type: PdcCommandType) -> Self {
        Self {
            command_type,
            stroke_color: None,
            fill_color: None,
            stroke_width: 1,
            radius: 0,
            is_closed: false,
            points: Vec::new(),
        }
    }

    pub fn circle(
        center: Point,
        radius: u16,
        stroke: Option<Rgb565>,
        fill: Option<Rgb565>,
        stroke_width: u8,
    ) -> Self {
        let mut cmd = Self::new(PdcCommandType::Circle);
        cmd.stroke_color = stroke;
        cmd.fill_color = fill;
        cmd.stroke_width = stroke_width;
        cmd.radius = radius;
        let p = PdcPrecisePoint::from_pixels(center.x as i16, center.y as i16);
        let _ = cmd.points.push(p);
        cmd
    }

    pub fn add_point(&mut self, pt: Point) -> Result<(), PdcError> {
        self.points
            .push(PdcPrecisePoint::from_pixels(pt.x as i16, pt.y as i16))
            .map_err(|_| PdcError)
    }

    pub fn add_subpixel_point(&mut self, x_fixed: i16, y_fixed: i16) -> Result<(), PdcError> {
        self.points
            .push(PdcPrecisePoint::from_subpixels(x_fixed, y_fixed))
            .map_err(|_| PdcError)
    }

    /// Adds a quadratic Bézier curve segment flattened into subpixel points.
    pub fn add_bezier_quad(
        &mut self,
        p0: Point,
        p1: Point,
        p2: Point,
        steps: usize,
    ) -> Result<(), PdcError> {
        let n = steps.max(2);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let one_minus_t = 1.0 - t;
            let x = one_minus_t * one_minus_t * p0.x as f32
                + 2.0 * one_minus_t * t * p1.x as f32
                + t * t * p2.x as f32;
            let y = one_minus_t * one_minus_t * p0.y as f32
                + 2.0 * one_minus_t * t * p1.y as f32
                + t * t * p2.y as f32;
            let x_scaled = x * 8.0;
            let y_scaled = y * 8.0;
            let x_sub = (x_scaled + if x_scaled >= 0.0 { 0.5 } else { -0.5 }) as i16;
            let y_sub = (y_scaled + if y_scaled >= 0.0 { 0.5 } else { -0.5 }) as i16;
            self.add_subpixel_point(x_sub, y_sub)?;
        }
        Ok(())
    }

    /// Adds a cubic Bézier curve segment flattened into subpixel points.
    pub fn add_bezier_cubic(
        &mut self,
        p0: Point,
        p1: Point,
        p2: Point,
        p3: Point,
        steps: usize,
    ) -> Result<(), PdcError> {
        let n = steps.max(3);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let one_minus_t = 1.0 - t;
            let t_sq = t * t;
            let omt_sq = one_minus_t * one_minus_t;
            let x = omt_sq * one_minus_t * p0.x as f32
                + 3.0 * omt_sq * t * p1.x as f32
                + 3.0 * one_minus_t * t_sq * p2.x as f32
                + t_sq * t * p3.x as f32;
            let y = omt_sq * one_minus_t * p0.y as f32
                + 3.0 * omt_sq * t * p1.y as f32
                + 3.0 * one_minus_t * t_sq * p2.y as f32
                + t_sq * t * p3.y as f32;
            let x_scaled = x * 8.0;
            let y_scaled = y * 8.0;
            let x_sub = (x_scaled + if x_scaled >= 0.0 { 0.5 } else { -0.5 }) as i16;
            let y_sub = (y_scaled + if y_scaled >= 0.0 { 0.5 } else { -0.5 }) as i16;
            self.add_subpixel_point(x_sub, y_sub)?;
        }
        Ok(())
    }

    /// Renders this vector command into the provided [`RenderCtx`].
    pub fn render<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, offset: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        match self.command_type {
            PdcCommandType::Circle => {
                if let Some(center_pt) = self.points.first() {
                    let p = center_pt.to_pixel_point();
                    let cx = p.x + offset.x;
                    let cy = p.y + offset.y;
                    let r = self.radius as u32;

                    if let Some(fill) = self.fill_color {
                        ctx.fill_circle(cx, cy, r, fill)?;
                    }
                    if let Some(stroke) = self.stroke_color {
                        if self.stroke_width > 0 {
                            ctx.stroke_circle(cx, cy, r, stroke)?;
                        }
                    }
                }
            }
            PdcCommandType::Path | PdcCommandType::PrecisePath => {
                if self.points.len() < 2 {
                    return Ok(());
                }

                // Render stroke lines connecting points
                if let Some(stroke) = self.stroke_color {
                    let mut prev = self.points[0].to_pixel_point();
                    prev.x += offset.x;
                    prev.y += offset.y;

                    let mut i = 1;
                    while i < self.points.len() {
                        let mut curr = self.points[i].to_pixel_point();
                        curr.x += offset.x;
                        curr.y += offset.y;

                        if self.stroke_width <= 1 {
                            ctx.draw_line(prev.x, prev.y, curr.x, curr.y, stroke)?;
                        } else {
                            ctx.draw_line_styled(
                                prev.x,
                                prev.y,
                                curr.x,
                                curr.y,
                                StrokeStyle::new(stroke).with_width(self.stroke_width),
                            )?;
                        }
                        prev = curr;
                        i += 1;
                    }

                    if self.is_closed && self.points.len() >= 3 {
                        let mut first = self.points[0].to_pixel_point();
                        first.x += offset.x;
                        first.y += offset.y;
                        if self.stroke_width <= 1 {
                            ctx.draw_line(prev.x, prev.y, first.x, first.y, stroke)?;
                        } else {
                            ctx.draw_line_styled(
                                prev.x,
                                prev.y,
                                first.x,
                                first.y,
                                StrokeStyle::new(stroke).with_width(self.stroke_width),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// A composite Precise Draw Command Image (PDC vector asset).
#[derive(Clone, Debug, PartialEq)]
pub struct PdcImage<const MAX_COMMANDS: usize = 8, const MAX_POINTS_PER_CMD: usize = 16> {
    pub viewbox: Rect,
    pub commands: Vec<PdcCommand<MAX_POINTS_PER_CMD>, MAX_COMMANDS>,
}

impl<const MAX_COMMANDS: usize, const MAX_POINTS_PER_CMD: usize> Default
    for PdcImage<MAX_COMMANDS, MAX_POINTS_PER_CMD>
{
    fn default() -> Self {
        Self::new(Rect::new(0, 0, 0, 0))
    }
}

impl<const MAX_COMMANDS: usize, const MAX_POINTS_PER_CMD: usize>
    PdcImage<MAX_COMMANDS, MAX_POINTS_PER_CMD>
{
    pub const fn new(viewbox: Rect) -> Self {
        Self {
            viewbox,
            commands: Vec::new(),
        }
    }

    pub fn push_command(&mut self, cmd: PdcCommand<MAX_POINTS_PER_CMD>) -> Result<(), PdcError> {
        self.commands.push(cmd).map_err(|_| PdcError)
    }

    /// Renders the entire vector image at the given target origin.
    pub fn draw<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, origin: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        for cmd in &self.commands {
            cmd.render(ctx, origin)?;
        }
        Ok(())
    }
}

/// Pebble-inspired PDCS (Precise Draw Command Sequence) vector animation reel.
///
/// Cycles through keyframed vector frames with per-frame durations, supporting
/// looping, pausing, rewinding, and zero-allocation frame rendering.
#[derive(Clone, Debug)]
pub struct PdcsReel<
    const MAX_FRAMES: usize = 8,
    const MAX_CMDS: usize = 8,
    const MAX_POINTS: usize = 16,
> {
    pub frames: Vec<PdcImage<MAX_CMDS, MAX_POINTS>, MAX_FRAMES>,
    pub durations_ms: [u32; MAX_FRAMES],
    pub loop_count: u32,
    pub completed_loops: u32,
    pub current_frame: usize,
    pub frame_elapsed_ms: u32,
    pub is_playing: bool,
}

impl<const MAX_FRAMES: usize, const MAX_CMDS: usize, const MAX_POINTS: usize> Default
    for PdcsReel<MAX_FRAMES, MAX_CMDS, MAX_POINTS>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_FRAMES: usize, const MAX_CMDS: usize, const MAX_POINTS: usize>
    PdcsReel<MAX_FRAMES, MAX_CMDS, MAX_POINTS>
{
    pub const fn new() -> Self {
        Self {
            frames: Vec::new(),
            durations_ms: [100; MAX_FRAMES],
            loop_count: 0,
            completed_loops: 0,
            current_frame: 0,
            frame_elapsed_ms: 0,
            is_playing: true,
        }
    }

    pub fn add_frame(
        &mut self,
        frame: PdcImage<MAX_CMDS, MAX_POINTS>,
        duration_ms: u32,
    ) -> Result<(), PdcError> {
        let idx = self.frames.len();
        if idx < MAX_FRAMES {
            self.durations_ms[idx] = duration_ms.max(1);
            self.frames.push(frame).map_err(|_| PdcError)
        } else {
            Err(PdcError)
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn rewind(&mut self) {
        self.current_frame = 0;
        self.frame_elapsed_ms = 0;
        self.completed_loops = 0;
    }

    pub fn is_finished(&self) -> bool {
        self.loop_count > 0 && self.completed_loops >= self.loop_count
    }

    /// Advance sequence playback by `dt_ms` milliseconds.
    pub fn update(&mut self, dt_ms: u32) {
        if !self.is_playing || self.frames.is_empty() || self.is_finished() {
            return;
        }

        self.frame_elapsed_ms += dt_ms;
        let mut dur = self.durations_ms[self.current_frame];
        while self.frame_elapsed_ms >= dur {
            self.frame_elapsed_ms -= dur;
            self.current_frame += 1;
            if self.current_frame >= self.frames.len() {
                self.current_frame = 0;
                self.completed_loops += 1;
                if self.is_finished() {
                    self.is_playing = false;
                    self.current_frame = self.frames.len() - 1;
                    break;
                }
            }
            dur = self.durations_ms[self.current_frame];
        }
    }

    /// Current active vector frame image.
    pub fn current_image(&self) -> Option<&PdcImage<MAX_CMDS, MAX_POINTS>> {
        self.frames.get(self.current_frame)
    }

    /// Renders current animated vector frame to target.
    pub fn draw<D, C>(&self, ctx: &mut RenderCtx<'_, D, C>, origin: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        C: crate::render::Compositor<D>,
    {
        if let Some(img) = self.current_image() {
            img.draw(ctx, origin)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;
    use embedded_graphics_core::pixelcolor::RgbColor;

    #[test]
    fn test_pdc_precise_points() {
        let pt = PdcPrecisePoint::from_pixels(10, 20);
        assert_eq!(pt.x_fixed, 80);
        assert_eq!(pt.y_fixed, 160);
        assert_eq!(pt.to_pixel_point(), Point::new(10, 20));

        let sub_pt = PdcPrecisePoint::from_subpixels(84, 164);
        assert_eq!(sub_pt.to_pixel_point(), Point::new(10, 20));
        let (fx, fy) = sub_pt.to_f32_point();
        assert!((fx - 10.5).abs() < 0.001);
        assert!((fy - 20.5).abs() < 0.001);
    }

    #[test]
    fn test_pdc_image_render() {
        let mut img = PdcImage::<4, 8>::new(Rect::new(0, 0, 20, 20));
        let circle = PdcCommand::circle(Point::new(10, 10), 4, Some(Rgb565::RED), None, 1);
        assert!(img.push_command(circle).is_ok());

        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        img.draw(&mut ctx, Point::new(0, 0)).unwrap();
    }

    #[test]
    fn test_pdcs_reel_playback() {
        let mut reel = PdcsReel::<4, 2, 4>::new();
        let frame1 = PdcImage::new(Rect::new(0, 0, 10, 10));
        let frame2 = PdcImage::new(Rect::new(0, 0, 10, 10));

        assert!(reel.add_frame(frame1, 50).is_ok());
        assert!(reel.add_frame(frame2, 50).is_ok());
        assert_eq!(reel.current_frame, 0);

        reel.update(30);
        assert_eq!(reel.current_frame, 0);

        reel.update(25);
        assert_eq!(reel.current_frame, 1);

        reel.update(50);
        // Loops back to frame 0
        assert_eq!(reel.current_frame, 0);
        assert_eq!(reel.completed_loops, 1);
    }

    #[test]
    fn test_pdc_command_paths_and_beziers() {
        let mut path = PdcCommand::<8>::new(PdcCommandType::Path);
        assert!(path.add_point(Point::new(0, 0)).is_ok());
        assert!(path.add_point(Point::new(10, 0)).is_ok());
        assert!(path.add_point(Point::new(10, 10)).is_ok());
        path.is_closed = true;
        path.stroke_color = Some(Rgb565::RED);
        path.stroke_width = 2;

        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        path.render(&mut ctx, Point::new(0, 0)).unwrap();

        let mut overflow = PdcCommand::<2>::new(PdcCommandType::Path);
        overflow.add_point(Point::new(0, 0)).unwrap();
        overflow.add_point(Point::new(1, 1)).unwrap();
        assert!(overflow.add_point(Point::new(2, 2)).is_err());

        let mut quad = PdcCommand::<16>::new(PdcCommandType::PrecisePath);
        quad.add_bezier_quad(Point::new(0, 0), Point::new(5, 10), Point::new(10, 0), 4)
            .unwrap();
        quad.add_bezier_cubic(
            Point::new(10, 0),
            Point::new(15, 10),
            Point::new(5, 10),
            Point::new(0, 0),
            4,
        )
        .unwrap();
        quad.stroke_color = Some(Rgb565::WHITE);
        quad.is_closed = true;
        let mut fb2 = Framebuffer::<400>::new(20, 20);
        let mut ctx2 = RenderCtx::new(&mut fb2, Rect::new(0, 0, 20, 20));
        quad.render(&mut ctx2, Point::new(0, 0)).unwrap();

        let mut fb3 = Framebuffer::<400>::new(20, 20);
        let mut ctx3 = RenderCtx::new(&mut fb3, Rect::new(0, 0, 20, 20));
        PdcCommand::<4>::new(PdcCommandType::Path)
            .render(&mut ctx3, Point::new(0, 0))
            .unwrap();
    }

    #[test]
    fn test_pdc_image_default_overflow_and_reel_controls() {
        let mut img = PdcImage::<1, 2>::default();
        assert_eq!(img.viewbox, Rect::new(0, 0, 0, 0));
        let cmd = PdcCommand::<2>::new(PdcCommandType::Path);
        img.push_command(cmd).unwrap();
        assert!(
            img.push_command(PdcCommand::<2>::new(PdcCommandType::Path))
                .is_err()
        );

        let mut reel = PdcsReel::<1, 1, 2>::default();
        reel.pause();
        assert!(!reel.is_playing);
        reel.update(100);
        reel.play();
        assert!(reel.is_playing);
        reel.add_frame(PdcImage::new(Rect::new(0, 0, 1, 1)), 10)
            .unwrap();
        reel.update(10);
        assert!(reel.completed_loops >= 1);

        reel.rewind();
        assert_eq!(reel.current_frame, 0);
        assert_eq!(reel.completed_loops, 0);

        let mut finite = PdcsReel::<2, 1, 2>::new();
        finite.loop_count = 1;
        finite
            .add_frame(PdcImage::new(Rect::new(0, 0, 1, 1)), 10)
            .unwrap();
        finite.update(10);
        assert!(finite.is_finished());
        finite.play();
        finite.update(100);
        assert_eq!(finite.current_frame, 0);
        // A finished reel ignores updates even if play() is called again.
        assert!(finite.is_playing);

        let mut full = PdcsReel::<1, 1, 2>::new();
        full.add_frame(PdcImage::new(Rect::new(0, 0, 1, 1)), 10)
            .unwrap();
        assert!(
            full.add_frame(PdcImage::new(Rect::new(0, 0, 1, 1)), 10)
                .is_err()
        );

        assert!(reel.current_image().is_some());
        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut ctx = RenderCtx::new(&mut fb, Rect::new(0, 0, 20, 20));
        reel.draw(&mut ctx, Point::new(0, 0)).unwrap();
    }
}
