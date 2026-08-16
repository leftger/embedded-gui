use embedded_graphics_core::{draw_target::DrawTarget, geometry::Point, pixelcolor::Rgb565};
use heapless::Vec;

use crate::{
    geometry::Rect,
    image::{ImageFit, ImageRef},
    render::{RenderCtx, TextStyle},
    style::{Border, LinearGradient, Shadow},
};

/// Discrete atomic 2D drawing task generated during UI traversal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawTask<'a> {
    Fill {
        rect: Rect,
        color: Rgb565,
        radius: u8,
        opacity: u8,
    },
    Gradient {
        rect: Rect,
        gradient: LinearGradient,
        radius: u8,
        opacity: u8,
    },
    Border {
        rect: Rect,
        border: Border,
        radius: u8,
    },
    Label {
        rect: Rect,
        text: &'a str,
        style: TextStyle,
    },
    Image {
        rect: Rect,
        image: ImageRef<'a>,
        tint: Option<Rgb565>,
    },
    Arc {
        center: Point,
        radius: u16,
        start_angle: i16,
        end_angle: i16,
        stroke_width: u8,
        color: Rgb565,
    },
    Line {
        start: Point,
        end: Point,
        color: Rgb565,
        width: u8,
    },
    BoxShadow {
        rect: Rect,
        shadow: Shadow,
        radius: u8,
    },
}

impl<'a> DrawTask<'a> {
    /// Returns the bounding box of the draw task.
    pub fn bounds(&self) -> Rect {
        match self {
            DrawTask::Fill { rect, .. } => *rect,
            DrawTask::Gradient { rect, .. } => *rect,
            DrawTask::Border { rect, border, .. } => {
                let w = border.width as i32;
                Rect::new(
                    rect.x - w,
                    rect.y - w,
                    rect.w + (w as u32 * 2),
                    rect.h + (w as u32 * 2),
                )
            }
            DrawTask::Label { rect, .. } => *rect,
            DrawTask::Image { rect, .. } => *rect,
            DrawTask::Arc {
                center,
                radius,
                stroke_width,
                ..
            } => {
                let r = *radius as i32 + (*stroke_width as i32 / 2) + 1;
                Rect::new(center.x - r, center.y - r, (r * 2) as u32, (r * 2) as u32)
            }
            DrawTask::Line {
                start, end, width, ..
            } => {
                let min_x = start.x.min(end.x) - (*width as i32 / 2);
                let min_y = start.y.min(end.y) - (*width as i32 / 2);
                let max_x = start.x.max(end.x) + (*width as i32 / 2);
                let max_y = start.y.max(end.y) + (*width as i32 / 2);
                Rect::new(
                    min_x,
                    min_y,
                    (max_x - min_x).max(1) as u32,
                    (max_y - min_y).max(1) as u32,
                )
            }
            DrawTask::BoxShadow { rect, shadow, .. } => {
                let s = shadow.spread as i32;
                Rect::new(
                    rect.x + shadow.offset_x as i32 - s,
                    rect.y + shadow.offset_y as i32 - s,
                    (rect.w as i32 + s * 2).max(1) as u32,
                    (rect.h as i32 + s * 2).max(1) as u32,
                )
            }
        }
    }
}

/// A fixed-capacity queue of draw tasks (strictly zero-allocation).
#[derive(Debug, Clone)]
pub struct DrawTaskQueue<'a, const CAPACITY: usize> {
    tasks: Vec<DrawTask<'a>, CAPACITY>,
}

impl<'a, const CAPACITY: usize> Default for DrawTaskQueue<'a, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const CAPACITY: usize> DrawTaskQueue<'a, CAPACITY> {
    pub const fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn push(&mut self, task: DrawTask<'a>) -> Result<(), DrawTask<'a>> {
        self.tasks.push(task)
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn as_slice(&self) -> &[DrawTask<'a>] {
        &self.tasks
    }
}

/// A pluggable hardware or software rendering unit.
pub trait DrawUnit<D: DrawTarget<Color = Rgb565>> {
    /// Returns whether this draw unit can accelerate or handle the given task.
    fn can_handle(&self, task: &DrawTask) -> bool;

    /// Executes the task on the target buffer.
    fn execute(&mut self, task: &DrawTask, target: &mut D) -> Result<(), D::Error>;
}

/// Default software draw unit implementing rasterization via [`RenderCtx`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SoftwareDrawUnit;

impl<D: DrawTarget<Color = Rgb565>> DrawUnit<D> for SoftwareDrawUnit {
    fn can_handle(&self, _task: &DrawTask) -> bool {
        true
    }

    fn execute(&mut self, task: &DrawTask, target: &mut D) -> Result<(), D::Error> {
        let b = task.bounds();
        let mut ctx = RenderCtx::new(target, b);
        match task {
            DrawTask::Fill {
                rect,
                color,
                radius,
                opacity,
            } => {
                if *opacity == 0 {
                    return Ok(());
                }
                if *radius == 0 && *opacity == 255 {
                    ctx.fill_rect(*rect, *color)?;
                } else {
                    ctx.fill_rounded_rect_alpha(*rect, *radius, *color, *opacity)?;
                }
            }
            DrawTask::Gradient {
                rect,
                gradient,
                radius,
                opacity,
            } => {
                if *opacity == 0 {
                    return Ok(());
                }
                ctx.fill_rounded_rect_gradient_alpha(*rect, *radius, *gradient, *opacity)?;
            }
            DrawTask::Border {
                rect,
                border,
                radius,
            } => {
                if border.width > 0 {
                    if *radius == 0 {
                        ctx.stroke_rect(*rect, *border)?;
                    } else {
                        ctx.stroke_rounded_rect(*rect, *radius, *border)?;
                    }
                }
            }
            DrawTask::Label { rect, text, style } => {
                ctx.draw_text_in(*rect, text, *style)?;
            }
            DrawTask::Image {
                rect,
                image,
                tint: _,
            } => {
                ctx.draw_image(*rect, *image, ImageFit::Stretch)?;
            }
            DrawTask::Arc {
                center,
                radius,
                start_angle,
                end_angle,
                stroke_width: _,
                color,
            } => {
                ctx.stroke_arc(
                    center.x,
                    center.y,
                    *radius as u32,
                    *start_angle as i32,
                    *end_angle as i32,
                    *color,
                )?;
            }
            DrawTask::Line {
                start,
                end,
                color,
                width: _,
            } => {
                ctx.draw_line(start.x, start.y, end.x, end.y, *color)?;
            }
            DrawTask::BoxShadow {
                rect,
                shadow,
                radius: _,
            } => {
                if shadow.opacity > 0 {
                    let s = shadow.spread as i32;
                    let shadow_rect = Rect::new(
                        rect.x + shadow.offset_x as i32 - s,
                        rect.y + shadow.offset_y as i32 - s,
                        (rect.w as i32 + s * 2).max(1) as u32,
                        (rect.h as i32 + s * 2).max(1) as u32,
                    );
                    ctx.fill_rect_alpha(shadow_rect, shadow.color, shadow.opacity)?;
                }
            }
        }
        Ok(())
    }
}

/// Dispatches a queue of draw tasks through a list of draw units with a fallback unit.
pub fn dispatch_draw_tasks<D: DrawTarget<Color = Rgb565>, const CAP: usize>(
    queue: &DrawTaskQueue<'_, CAP>,
    target: &mut D,
    units: &mut [&mut dyn DrawUnit<D>],
    fallback: &mut SoftwareDrawUnit,
) -> Result<(), D::Error> {
    for task in queue.as_slice() {
        let mut handled = false;
        for unit in units.iter_mut() {
            if unit.can_handle(task) {
                unit.execute(task, target)?;
                handled = true;
                break;
            }
        }
        if !handled {
            fallback.execute(task, target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::Framebuffer;
    use embedded_graphics_core::pixelcolor::RgbColor;

    #[test]
    fn test_draw_task_bounds() {
        let fill = DrawTask::Fill {
            rect: Rect::new(10, 20, 30, 40),
            color: Rgb565::RED,
            radius: 0,
            opacity: 255,
        };
        assert_eq!(fill.bounds(), Rect::new(10, 20, 30, 40));

        let border = DrawTask::Border {
            rect: Rect::new(10, 20, 30, 40),
            border: Border::one(Rgb565::WHITE),
            radius: 0,
        };
        assert_eq!(border.bounds(), Rect::new(9, 19, 32, 42));
    }

    #[test]
    fn test_draw_task_queue_operations() {
        let mut queue = DrawTaskQueue::<4>::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        let task1 = DrawTask::Fill {
            rect: Rect::new(0, 0, 10, 10),
            color: Rgb565::RED,
            radius: 0,
            opacity: 255,
        };
        let task2 = DrawTask::Fill {
            rect: Rect::new(10, 10, 10, 10),
            color: Rgb565::BLUE,
            radius: 0,
            opacity: 255,
        };

        assert!(queue.push(task1).is_ok());
        assert!(queue.push(task2).is_ok());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.as_slice().len(), 2);

        queue.clear();
        assert!(queue.is_empty());
    }

    struct MockHwFillUnit {
        fill_count: usize,
    }

    impl<D: DrawTarget<Color = Rgb565>> DrawUnit<D> for MockHwFillUnit {
        fn can_handle(&self, task: &DrawTask) -> bool {
            matches!(task, DrawTask::Fill { .. })
        }

        fn execute(&mut self, task: &DrawTask, target: &mut D) -> Result<(), D::Error> {
            if let DrawTask::Fill { rect, color, .. } = task {
                self.fill_count += 1;
                let mut ctx = RenderCtx::new(target, *rect);
                ctx.fill_rect(*rect, *color)?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_draw_unit_dispatch() {
        let mut fb = Framebuffer::<400>::new(20, 20);
        let mut queue = DrawTaskQueue::<4>::new();

        queue
            .push(DrawTask::Fill {
                rect: Rect::new(0, 0, 5, 5),
                color: Rgb565::RED,
                radius: 0,
                opacity: 255,
            })
            .unwrap();

        let mut hw_unit = MockHwFillUnit { fill_count: 0 };
        let mut fallback = SoftwareDrawUnit;

        let mut units: [&mut dyn DrawUnit<Framebuffer<400>>; 1] = [&mut hw_unit];
        dispatch_draw_tasks(&queue, &mut fb, &mut units, &mut fallback).unwrap();

        assert_eq!(hw_unit.fill_count, 1);
        assert_eq!(fb.pixels()[0], Rgb565::RED);
    }
}
