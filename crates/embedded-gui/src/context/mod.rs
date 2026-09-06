pub mod builders;
pub mod core_impl;
pub mod input;
pub mod mutators;
pub mod present;
pub mod render;
pub mod types;

pub use builders::WidgetBuilder;
#[allow(unused_imports)]
pub(crate) use input::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use crate::GuiContext;
    use crate::geometry::Rect;
    use crate::input::{InputEvent, PointerButton, UiEvent};

    #[test]
    fn test_context_initialization() {
        let viewport = Rect::new(0, 0, 320, 240);
        let ctx: GuiContext<32, 16, 16> = GuiContext::new(viewport);
        assert_eq!(ctx.viewport(), viewport);
        assert_eq!(ctx.widgets().len(), 0);
        assert_eq!(ctx.focus(), None);
    }

    #[test]
    fn test_widget_addition_and_focus() {
        let viewport = Rect::new(0, 0, 320, 240);
        let mut ctx: GuiContext<32, 16, 16> = GuiContext::new(viewport);

        let btn1 = ctx
            .add_themed_button(Rect::new(10, 10, 80, 30), "Button 1")
            .unwrap();
        let btn2 = ctx
            .add_themed_button(Rect::new(10, 50, 80, 30), "Button 2")
            .unwrap();

        assert_eq!(ctx.widgets().len(), 2);
        // Automatically focuses first focusable widget
        assert_eq!(ctx.focus(), Some(btn1));

        ctx.set_focus(Some(btn2)).unwrap();
        assert_eq!(ctx.focus(), Some(btn2));
    }

    #[test]
    fn test_state_mutations() {
        let viewport = Rect::new(0, 0, 320, 240);
        let mut ctx: GuiContext<32, 16, 16> = GuiContext::new(viewport);

        let progress_id = ctx
            .add_themed_progress_bar(Rect::new(10, 10, 100, 10), 0.5)
            .unwrap();
        ctx.set_progress(progress_id, 0.75).unwrap();

        let toggle_id = ctx
            .add_themed_toggle(Rect::new(10, 30, 40, 20), "Toggle", false)
            .unwrap();
        ctx.set_toggle(toggle_id, true).unwrap();

        ctx.set_enabled(progress_id, false).unwrap();
        assert!(!ctx.effective_enabled(progress_id));
    }

    #[test]
    fn test_input_handling() {
        let viewport = Rect::new(0, 0, 320, 240);
        let mut ctx: GuiContext<32, 16, 16> = GuiContext::new(viewport);

        let btn = ctx
            .add_themed_button(Rect::new(10, 10, 80, 30), "Press")
            .unwrap();

        // Pointer down inside button
        ctx.handle_input(InputEvent::Pointer {
            x: 15,
            y: 15,
            state: crate::input::PointerState::Pressed,
            button: PointerButton::Primary,
        })
        .unwrap();

        // Pointer up emits click event
        ctx.handle_input(InputEvent::Pointer {
            x: 15,
            y: 15,
            state: crate::input::PointerState::Released,
            button: PointerButton::Primary,
        })
        .unwrap();

        let events = &ctx.events;
        assert!(
            events
                .iter()
                .any(|e| matches!(e, UiEvent::Clicked(id) if id == &btn))
        );
    }

    #[test]
    fn test_render_dirty_buffered() {
        use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
        let viewport = Rect::new(0, 0, 320, 240);
        let mut ctx: GuiContext<32, 16, 16> = GuiContext::new(viewport);
        let btn = ctx
            .add_themed_button(Rect::new(10, 10, 80, 30), "Buffered")
            .unwrap();

        let mut display = crate::test_buffer::TestBuffer::new(320, 240);
        let mut scratch = [Rgb565::BLACK; 256];

        // Mark button dirty
        ctx.dirty.add(Rect::new(10, 10, 80, 30)).unwrap();
        ctx.render_dirty_buffered(&mut display, &mut scratch)
            .unwrap();

        assert!(ctx.node(btn).is_some());
    }

    #[test]
    fn test_spatial_focus_navigation_2d() {
        use crate::input::NavDirection;
        let viewport = Rect::new(0, 0, 320, 240);
        let mut ctx: GuiContext<32, 16, 16> = GuiContext::new(viewport);

        // 2x2 grid:
        // [btn_tl (0,0)]    [btn_tr (100,0)]
        // [btn_bl (0,50)]   [btn_br (100,50)]
        let btn_tl = ctx
            .add_themed_button(Rect::new(0, 0, 50, 30), "TL")
            .unwrap();
        let btn_tr = ctx
            .add_themed_button(Rect::new(100, 0, 50, 30), "TR")
            .unwrap();
        let btn_bl = ctx
            .add_themed_button(Rect::new(0, 50, 50, 30), "BL")
            .unwrap();
        let btn_br = ctx
            .add_themed_button(Rect::new(100, 50, 50, 30), "BR")
            .unwrap();

        assert_eq!(ctx.focus(), Some(btn_tl));

        // Moving Down should go to btn_bl, NOT btn_tr!
        ctx.handle_input(InputEvent::Down).unwrap();
        assert_eq!(ctx.focus(), Some(btn_bl));

        // Moving Right should go to btn_br
        ctx.handle_input(InputEvent::Right).unwrap();
        assert_eq!(ctx.focus(), Some(btn_br));

        // Moving Up should go to btn_tr
        ctx.handle_input(InputEvent::Up).unwrap();
        assert_eq!(ctx.focus(), Some(btn_tr));

        // Moving Left should go back to btn_tl
        ctx.handle_input(InputEvent::Left).unwrap();
        assert_eq!(ctx.focus(), Some(btn_tl));

        // Explicit method call works too:
        assert!(ctx.move_focus_direction(NavDirection::Down).unwrap());
        assert_eq!(ctx.focus(), Some(btn_bl));
    }
}
