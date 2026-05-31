use core::convert::Infallible;

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_gui::prelude::*;

struct MockTarget {
    pixels: heapless::Vec<(i32, i32, Rgb565), 4096>,
    size: Size,
}

impl MockTarget {
    fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: heapless::Vec::new(),
            size: Size::new(width, height),
        }
    }
}

impl DrawTarget for MockTarget {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            let _ = self.pixels.push((point.x, point.y, color));
        }
        Ok(())
    }
}

impl OriginDimensions for MockTarget {
    fn size(&self) -> Size {
        self.size
    }
}

#[test]
fn renders_label_and_progress_bar() {
    let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 64, 32));
    gui.add_label(Rect::new(2, 2, 40, 8), "OK", Style::label())
        .unwrap();
    gui.add_progress_bar(Rect::new(2, 14, 20, 6), 0.5, Style::progress())
        .unwrap();

    let mut target = MockTarget::new(64, 32);
    gui.render(&mut target).unwrap();

    assert!(!target.pixels.is_empty());
    assert!(
        target
            .pixels
            .iter()
            .any(|&(_, _, color)| color == Rgb565::new(0, 50, 18))
    );
}

#[test]
fn focus_moves_between_buttons() {
    let mut gui = GuiContext::<4, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let first = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    let second = gui
        .add_button(Rect::new(0, 12, 30, 10), "TWO", Style::button())
        .unwrap();

    assert_eq!(gui.focus(), Some(first));
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.focus(), Some(second));
    gui.handle_input(InputEvent::Up).unwrap();
    assert_eq!(gui.focus(), Some(first));
}

#[test]
fn widget_flags_control_focus_rendering_and_pointer_hits() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 80, 40));
    let first = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    let second = gui
        .add_button(Rect::new(0, 12, 30, 10), "TWO", Style::button())
        .unwrap();
    let label = gui
        .add_label(Rect::new(40, 0, 30, 10), "LBL", Style::label())
        .unwrap();

    assert_eq!(gui.focus(), Some(first));
    gui.set_disabled(first, true).unwrap();
    assert_eq!(gui.focus(), Some(second));
    assert!(gui.has_flag(first, WidgetFlags::DISABLED).unwrap());

    while gui.pop_event().is_some() {}
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert!(gui.pop_event().is_none());

    gui.set_clickable(label, true).unwrap();
    while gui.pop_event().is_some() {}
    gui.handle_input(InputEvent::Pointer {
        x: 42,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert_eq!(gui.focus(), Some(second));
    assert_eq!(gui.pop_event(), Some(UiEvent::Pressed(label)));
    assert_eq!(gui.pop_event(), Some(UiEvent::PointerPressed(label)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Clicked(label)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Activate(label)));
}

#[test]
fn hidden_parent_invalidates_child_focus_and_rendering() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 80, 40));
    let panel = gui
        .add_panel(Rect::new(0, 0, 40, 20), Style::panel())
        .unwrap();
    let child = gui
        .add_button(Rect::new(2, 2, 30, 10), "C", Style::button())
        .unwrap();
    gui.add_child(panel, child).unwrap();
    gui.set_focus(Some(child)).unwrap();

    gui.set_hidden(panel, true).unwrap();

    assert_eq!(gui.focus(), None);
    let mut target = MockTarget::new(80, 40);
    gui.render(&mut target).unwrap();
    assert!(target.pixels.is_empty());
}

#[test]
fn focused_menu_changes_selection() {
    static ITEMS: [&str; 3] = ["PLAY", "OPTS", "QUIT"];
    let mut gui = GuiContext::<4, 8, 8>::new(Rect::new(0, 0, 96, 48));
    let menu = gui
        .add_menu(Rect::new(0, 0, 60, 30), &ITEMS, 0, Style::panel())
        .unwrap();

    assert_eq!(gui.focus(), Some(menu));
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.menu_selected(menu), Some(1));
    gui.handle_input(InputEvent::Up).unwrap();
    assert_eq!(gui.menu_selected(menu), Some(0));
}

#[test]
fn dirty_tracker_merges_overlapping_regions() {
    let mut dirty = DirtyTracker::<4>::new();
    dirty.add(Rect::new(0, 0, 10, 10)).unwrap();
    dirty.add(Rect::new(5, 5, 10, 10)).unwrap();

    assert_eq!(dirty.as_slice().len(), 1);
    assert_eq!(dirty.bounding_rect(), Some(Rect::new(0, 0, 15, 15)));
}

#[test]
fn linear_layout_arranges_columns() {
    let layout = LinearLayout::column().with_gap(2);
    let mut out = [Rect::empty(); 3];
    let count = layout.arrange(Rect::new(0, 0, 30, 34), 3, &mut out);

    assert_eq!(count, 3);
    assert_eq!(out[0], Rect::new(0, 0, 30, 10));
    assert_eq!(out[1], Rect::new(0, 12, 30, 10));
    assert_eq!(out[2], Rect::new(0, 24, 30, 10));
}

#[test]
fn screen_stack_applies_commands() {
    let main = ScreenId::new(1);
    let settings = ScreenId::new(2);
    let hud = ScreenId::new(3);
    let mut stack = ScreenStack::<4>::with_root(main).unwrap();

    stack.apply(ScreenCommand::Push(settings)).unwrap();
    assert_eq!(stack.current(), Some(settings));
    stack.apply(ScreenCommand::Replace(hud)).unwrap();
    assert_eq!(stack.as_slice(), &[main, hud]);
    stack.apply(ScreenCommand::Pop).unwrap();
    assert_eq!(stack.current(), Some(main));
}

#[test]
fn screen_stack_emits_lifecycle_events() {
    let main = ScreenId::new(1);
    let settings = ScreenId::new(2);
    let hud = ScreenId::new(3);
    let mut events = heapless::Vec::<ScreenLifecycleEvent, 8>::new();
    let mut stack = ScreenStack::<4>::with_root_lifecycle(main, &mut events).unwrap();

    assert_eq!(events.as_slice(), &[ScreenLifecycleEvent::Mount(main)]);
    events.clear();

    let transition = stack
        .apply_lifecycle(ScreenCommand::Push(settings), &mut events)
        .unwrap();
    assert_eq!(
        transition,
        ScreenTransition {
            from: Some(main),
            to: Some(settings),
            command: ScreenCommand::Push(settings),
        }
    );
    assert_eq!(
        events.as_slice(),
        &[
            ScreenLifecycleEvent::Pause(main),
            ScreenLifecycleEvent::Mount(settings)
        ]
    );
    events.clear();

    stack
        .apply_lifecycle(ScreenCommand::Replace(hud), &mut events)
        .unwrap();
    assert_eq!(
        events.as_slice(),
        &[
            ScreenLifecycleEvent::Unmount(settings),
            ScreenLifecycleEvent::Mount(hud)
        ]
    );
    events.clear();

    stack
        .apply_lifecycle(ScreenCommand::Pop, &mut events)
        .unwrap();
    assert_eq!(
        events.as_slice(),
        &[
            ScreenLifecycleEvent::Unmount(hud),
            ScreenLifecycleEvent::Resume(main)
        ]
    );
}

#[test]
fn text_alignment_draws_inside_rect() {
    let mut target = MockTarget::new(32, 16);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 32, 16));
    ctx.draw_text_in(
        Rect::new(0, 0, 20, 8),
        "A",
        TextStyle {
            color: Rgb565::WHITE,
            font: FontId::Tiny3x5,
            opacity: 255,
            align: TextAlign::Center,
            vertical_align: VerticalAlign::Top,
            wrap: TextWrap::None,
            line_spacing: 0,
        },
    )
    .unwrap();

    assert!(target.pixels.iter().any(|&(x, y, _)| x == 9 && y == 0));
}

#[test]
fn font_metrics_scale_with_selected_font() {
    let small = RenderCtx::<MockTarget>::text_metrics_with_font("AB", FontId::Tiny3x5);
    let medium = RenderCtx::<MockTarget>::text_metrics_with_font("AB", FontId::Medium4x7);
    let large = RenderCtx::<MockTarget>::text_metrics_with_font("AB", FontId::Scaled6x10);
    assert!(medium.width > small.width);
    assert!(medium.height > small.height);
    assert!(large.width > medium.width);
    assert!(large.height > medium.height);
}

#[test]
fn draw_text_with_large_font_draws_taller_pixels() {
    let mut target = TestBuffer::new(32, 24);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 32, 24));
    ctx.draw_text_in_with_font(
        Rect::new(0, 0, 32, 24),
        "A",
        TextStyle::new(Rgb565::WHITE),
        FontId::Scaled6x10,
    )
    .unwrap();

    assert_eq!(target.pixel_at(0, 0), Some(Rgb565::BLACK));
    assert_eq!(target.pixel_at(2, 0), Some(Rgb565::WHITE));
}

#[test]
fn rounded_rect_clips_corners() {
    let mut target = TestBuffer::new(16, 16);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 16, 16));
    ctx.fill_rounded_rect(Rect::new(2, 2, 10, 10), 4, Rgb565::RED)
        .unwrap();

    assert_eq!(target.pixel_at(2, 2), Some(Rgb565::BLACK));
    assert_eq!(target.pixel_at(11, 2), Some(Rgb565::BLACK));
    assert_eq!(target.pixel_at(2, 11), Some(Rgb565::BLACK));
    assert_eq!(target.pixel_at(11, 11), Some(Rgb565::BLACK));
    assert_eq!(target.pixel_at(7, 7), Some(Rgb565::RED));
}

#[test]
fn rounded_rect_gradient_interpolates_across_axis() {
    let mut target = TestBuffer::new(12, 12);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 12, 12));
    ctx.fill_rounded_rect_gradient_alpha(
        Rect::new(1, 1, 10, 10),
        0,
        LinearGradient::vertical(Rgb565::RED, Rgb565::BLUE),
        255,
    )
    .unwrap();

    let top = target.pixel_at(6, 1).unwrap();
    let bottom = target.pixel_at(6, 10).unwrap();
    assert!(top.r() > bottom.r());
    assert!(bottom.b() > top.b());
}

#[test]
fn low_quality_gradient_collapses_to_flat_fill() {
    let mut target = TestBuffer::new(12, 12);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 12, 12));
    ctx.set_quality(RenderQuality::Low);
    ctx.fill_rounded_rect_gradient_alpha(
        Rect::new(1, 1, 10, 10),
        0,
        LinearGradient::vertical(Rgb565::RED, Rgb565::BLUE),
        255,
    )
    .unwrap();

    assert_eq!(target.pixel_at(6, 1), target.pixel_at(6, 10));
}

#[test]
fn expanded_widgets_update_values() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 96, 64));
    let toggle = gui
        .add_toggle(Rect::new(0, 0, 50, 10), "ON", false, Style::button())
        .unwrap();
    let checkbox = gui
        .add_checkbox(Rect::new(0, 12, 50, 10), "CHK", false, Style::button())
        .unwrap();
    let slider = gui
        .add_slider(Rect::new(0, 24, 50, 10), 0.0, 0.0, 1.0, Style::button())
        .unwrap();

    gui.set_focus(Some(toggle)).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.toggle_value(toggle), Some(true));
    while gui.pop_event().is_some() {}

    gui.set_focus(Some(checkbox)).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.checked_value(checkbox), Some(true));
    while gui.pop_event().is_some() {}

    gui.set_focus(Some(slider)).unwrap();
    gui.handle_input(InputEvent::Right).unwrap();
    assert!(gui.slider_value(slider).unwrap() > 0.0);
}

#[test]
fn programmatic_setters_are_dirty_only() {
    let mut gui = GuiContext::<8, 4, 8>::new(Rect::new(0, 0, 96, 64));
    let progress = gui
        .add_progress_bar(Rect::new(0, 0, 50, 10), 0.0, Style::progress())
        .unwrap();
    let value = gui
        .add_value_label(Rect::new(0, 12, 50, 10), "V", 0, Style::panel())
        .unwrap();
    while gui.pop_event().is_some() {}
    gui.clear_dirty();

    gui.set_progress(progress, 0.5).unwrap();
    gui.set_value_label(value, 42).unwrap();

    assert!(gui.pop_event().is_none());
    assert!(!gui.dirty_regions().is_empty());
}

#[test]
fn focus_groups_limit_navigation() {
    let mut gui = GuiContext::<4, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let first = gui
        .add_button(Rect::new(0, 0, 30, 10), "A", Style::button())
        .unwrap();
    let second = gui
        .add_button(Rect::new(0, 12, 30, 10), "B", Style::button())
        .unwrap();
    let group = FocusGroupId::new(2);
    gui.set_focus_group(second, group).unwrap();
    gui.set_active_focus_group(Some(group));

    assert_eq!(gui.focus(), Some(second));
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.focus(), Some(second));
    assert_ne!(gui.focus(), Some(first));
}

#[test]
fn layout_items_mix_fixed_and_fill() {
    let layout = LinearLayout::row().with_gap(2);
    let items = [
        LayoutItem::fixed(10),
        LayoutItem::fill(),
        LayoutItem::percent(25),
    ];
    let mut out = [Rect::empty(); 3];

    let count = layout.arrange_items(Rect::new(0, 0, 100, 20), &items, &mut out);

    assert_eq!(count, 3);
    assert_eq!(out[0], Rect::new(0, 0, 10, 20));
    assert_eq!(out[2].w, 24);
    assert!(out[1].w > out[0].w);
}

#[test]
fn constraint_layout_supports_ratio_min_max_and_weighted_fill() {
    let layout = LinearLayout::row().with_gap(2);
    let items = [
        LayoutItem::length(10),
        LayoutItem::ratio(1, 2),
        LayoutItem::min(20),
        LayoutItem::max(15),
        LayoutItem::fill_weight(2),
    ];
    let mut out = [Rect::empty(); 5];

    let count = layout.arrange_items(Rect::new(0, 0, 120, 16), &items, &mut out);

    assert_eq!(count, 5);
    assert_eq!(out[0].w, 10);
    assert_eq!(out[1].w, 56);
    assert_eq!(out[2].w, 20);
    assert_eq!(out[3].w, 15);
    assert_eq!(out[4].w, 11);
}

#[test]
fn flex_layout_grow_distributes_extra_space_by_weight() {
    let layout = LinearLayout::row().with_gap(2);
    let items = [
        LayoutItem::length(10).with_grow(1),
        LayoutItem::length(10).with_grow(3),
    ];
    let mut out = [Rect::empty(); 2];
    layout.arrange_items_flex(Rect::new(0, 0, 40, 10), &items, &mut out, true, false);
    assert!(out[1].w > out[0].w);
}

#[test]
fn flex_layout_shrink_reduces_overflow_by_weight() {
    let layout = LinearLayout::row().with_gap(0);
    let items = [
        LayoutItem::length(30).with_shrink(3),
        LayoutItem::length(30).with_shrink(1),
    ];
    let mut out = [Rect::empty(); 2];
    layout.arrange_items_flex(Rect::new(0, 0, 40, 10), &items, &mut out, false, true);
    assert!(out[0].w < out[1].w);
}

#[test]
fn flex_presets_and_item_helpers_work() {
    let layout = LinearLayout::flex_row().with_gap(2);
    let items = [LayoutItem::rigid(10), LayoutItem::flex(10)];
    let mut out = [Rect::empty(); 2];
    layout.arrange_items_flex(Rect::new(0, 0, 40, 12), &items, &mut out, true, true);

    assert_eq!(out[0].w, 10);
    assert!(out[1].w > out[0].w);
}

#[test]
fn constraint_layout_documents_overflow_and_cross_axis_rules() {
    let overflow = [
        LayoutItem::length(40),
        LayoutItem::min(30),
        LayoutItem::fill(),
    ];
    let mut out = [Rect::empty(); 3];
    let count =
        LinearLayout::row()
            .with_gap(2)
            .arrange_items(Rect::new(0, 0, 50, 20), &overflow, &mut out);

    assert_eq!(count, 3);
    assert_eq!(out[0].w, 40);
    assert_eq!(out[1].w, 30);
    assert_eq!(out[2].w, 0);
    assert!(out[1].right() > 50);

    let cross = [LayoutItem::length(10).with_cross(Constraint::percent(50))];
    let mut out = [Rect::empty(); 1];
    LinearLayout {
        axis: Axis::Horizontal,
        gap: 0,
        padding: EdgeInsets::all(0),
        cross_align: Align::Center,
    }
    .arrange_items(Rect::new(0, 0, 40, 20), &cross, &mut out);

    assert_eq!(out[0], Rect::new(0, 5, 10, 10));
}

#[test]
fn intrinsic_layout_sizes_text_widgets_by_content() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 120, 40));
    let short = gui
        .add_label(Rect::new(0, 0, 1, 1), "A", Style::label())
        .unwrap();
    let long = gui
        .add_button(Rect::new(0, 0, 1, 1), "LONG", Style::button())
        .unwrap();
    let ids = [short, long];

    let count = gui
        .apply_layout_intrinsic(
            LinearLayout::row().with_gap(2),
            Rect::new(0, 0, 120, 20),
            &ids,
        )
        .unwrap();
    assert_eq!(count, 2);

    let short_rect = gui.widgets().iter().find(|w| w.id == short).unwrap().rect;
    let long_rect = gui.widgets().iter().find(|w| w.id == long).unwrap().rect;
    assert!(long_rect.w > short_rect.w);
}

#[test]
fn intrinsic_layout_can_preserve_cross_axis_size() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 120, 60));
    let a = gui
        .add_button(Rect::new(0, 0, 1, 1), "A", Style::button())
        .unwrap();
    let b = gui
        .add_button(Rect::new(0, 0, 1, 1), "BBBB", Style::button())
        .unwrap();
    let ids = [a, b];

    gui.apply_layout_intrinsic_with_cross(
        LinearLayout {
            axis: Axis::Horizontal,
            gap: 2,
            padding: EdgeInsets::all(0),
            cross_align: Align::Start,
        },
        Rect::new(0, 0, 120, 60),
        &ids,
        true,
    )
    .unwrap();

    let a_rect = gui.widgets().iter().find(|w| w.id == a).unwrap().rect;
    let b_rect = gui.widgets().iter().find(|w| w.id == b).unwrap().rect;
    assert!(a_rect.h < 60);
    assert_eq!(a_rect.h, b_rect.h);
}

#[test]
fn block_calculates_inner_area_and_renders_title() {
    let block = Block::styled(Style::panel()).title("HUD");

    assert_eq!(
        block.inner(Rect::new(0, 0, 30, 20)),
        Rect::new(3, 3, 24, 14)
    );
    assert_eq!(
        block.title_area(Rect::new(0, 0, 30, 20)),
        Some(Rect::new(3, 0, 24, 7))
    );
    assert_eq!(
        block.content_area(Rect::new(0, 0, 30, 20)),
        Rect::new(3, 12, 24, 5)
    );

    let mut target = TestBuffer::new(40, 24);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 40, 24));
    block.render(Rect::new(0, 0, 30, 20), &mut ctx).unwrap();

    target.assert_non_empty_rect(Rect::new(0, 0, 30, 20));
    assert!(target.count_color(Style::panel().border.color) > 0);
}

#[test]
fn block_shadow_draws_outside_body() {
    let mut style = Style::panel();
    style.shadow = Some(Shadow {
        color: Rgb565::BLUE,
        opacity: 255,
        offset_x: 2,
        offset_y: 2,
        spread: 1,
    });
    style.background = None;
    style.gradient = None;
    style.border = Border::none();
    let block = Block::styled(style);

    let mut target = TestBuffer::new(24, 24);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 24, 24));
    block.render(Rect::new(2, 2, 10, 10), &mut ctx).unwrap();

    assert_eq!(target.pixel_at(12, 12), Some(Rgb565::BLUE));
}

#[test]
fn shadow_quality_profile_controls_shadow_passes() {
    let style = Style {
        background: None,
        gradient: None,
        font: FontId::Tiny3x5,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::CYAN,
        opacity: 255,
        corner_radius: 2,
        shadow: Some(Shadow {
            color: Rgb565::GREEN,
            opacity: 255,
            offset_x: 1,
            offset_y: 1,
            spread: 3,
        }),
        border: Border::none(),
        padding: EdgeInsets::all(0),
    };
    let block = Block::styled(style);

    let mut low_target = TestBuffer::new(24, 24);
    let mut low_ctx = RenderCtx::new(&mut low_target, Rect::new(0, 0, 24, 24));
    low_ctx.set_quality(RenderQuality::Low);
    block.render(Rect::new(4, 4, 10, 10), &mut low_ctx).unwrap();
    assert_eq!(low_target.pixel_at(10, 10), Some(Rgb565::BLACK));

    let mut medium_target = TestBuffer::new(24, 24);
    let mut medium_ctx = RenderCtx::new(&mut medium_target, Rect::new(0, 0, 24, 24));
    medium_ctx.set_quality(RenderQuality::Medium);
    block
        .render(Rect::new(4, 4, 10, 10), &mut medium_ctx)
        .unwrap();
    assert_eq!(medium_target.pixel_at(10, 10), Some(Rgb565::GREEN));
}

#[test]
fn text_model_draws_styled_spans() {
    let red = TextStyle::new(Rgb565::RED);
    let green = TextStyle::new(Rgb565::GREEN);
    let spans = [Span::styled("A", red), Span::styled("B", green)];
    let lines = [Line::from_spans(&spans).aligned(TextAlign::Center)];
    let text = Text::from_lines(&lines);
    let mut target = TestBuffer::new(24, 12);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 24, 12));

    ctx.draw_text_model_in(Rect::new(0, 0, 24, 12), text)
        .unwrap();

    assert!(target.count_color(Rgb565::RED) > 0);
    assert!(target.count_color(Rgb565::GREEN) > 0);
}

#[test]
fn text_model_reports_metrics_and_wraps_spans() {
    let spans = [
        Span::styled("ABCD", TextStyle::new(Rgb565::RED)),
        Span::styled("EF\nGH", TextStyle::new(Rgb565::GREEN)),
    ];
    let lines = [Line::from_spans(&spans)];
    let text = Text::from_lines(&lines)
        .wrapped(TextWrap::Character)
        .line_spacing(0);

    assert_eq!(lines[0].visual_line_count(12, TextWrap::Character), 3);
    assert_eq!(
        text.metrics(12),
        TextMetrics {
            width: 12,
            height: 18
        }
    );

    let mut target = TestBuffer::new(12, 24);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 12, 24));
    ctx.draw_text_model_in(Rect::new(0, 0, 12, 24), text)
        .unwrap();

    assert!(target.count_color(Rgb565::RED) > 0);
    assert!(target.count_color(Rgb565::GREEN) > 0);
}

#[test]
fn widget_state_helpers_clamp_and_advance() {
    let mut list = ListState::new(0, 0, 2);
    assert!(list.set_selected(3, 4));
    assert_eq!(list, ListState::new(3, 2, 2));
    assert!(list.next(4));
    assert_eq!(list.selected, 0);

    let mut tabs = TabsState::new(0);
    assert!(tabs.previous(3));
    assert_eq!(tabs.selected, 2);

    let mut scroll = ScrollState::new(0, 20);
    assert!(scroll.scroll_by(99));
    assert_eq!(scroll.offset_y, 20);

    let mut slider = SliderState::new(0.5, 0.0, 1.0);
    assert!(slider.set_value(2.0));
    assert_eq!(slider.value, 1.0);
}

#[test]
fn stateful_widget_trait_renders_with_external_state() {
    struct Bar;

    impl StatefulWidget<SliderState> for Bar {
        fn render_stateful<D>(
            &self,
            area: Rect,
            state: &mut SliderState,
            ctx: &mut RenderCtx<'_, D>,
        ) -> Result<(), D::Error>
        where
            D: DrawTarget<Color = Rgb565>,
        {
            let width = (area.w as f32 * state.value).round() as u32;
            ctx.fill_rect(Rect::new(area.x, area.y, width, area.h), Rgb565::CYAN)
        }
    }

    let mut state = SliderState::new(0.5, 0.0, 1.0);
    let mut target = TestBuffer::new(10, 4);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 10, 4));
    Bar.render_stateful(Rect::new(0, 0, 10, 4), &mut state, &mut ctx)
        .unwrap();

    assert_eq!(target.count_color(Rgb565::CYAN), 20);
}

#[test]
fn test_buffer_tracks_pixels_and_digest() {
    let mut target = TestBuffer::new(8, 8);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 8, 8));
    ctx.fill_rect(Rect::new(2, 2, 3, 3), Rgb565::BLUE).unwrap();

    assert_eq!(target.pixel_at(2, 2), Some(Rgb565::BLUE));
    assert_eq!(target.count_color(Rgb565::BLUE), 9);
    assert_ne!(target.digest(), 0);
}

#[test]
fn test_buffer_diffs_changed_rows_and_bounding_region() {
    let previous = TestBuffer::new(8, 8);
    let mut next = previous.clone();
    let mut ctx = RenderCtx::new(&mut next, Rect::new(0, 0, 8, 8));
    ctx.fill_rect(Rect::new(2, 2, 3, 2), Rgb565::BLUE).unwrap();

    assert_eq!(
        next.diff_bounding_region(&previous),
        Some(PresentRegion::new(2, 2, 3, 2))
    );
    let rows: heapless::Vec<PresentRegion, 4> = next.diff_regions(&previous);
    assert_eq!(
        rows.as_slice(),
        &[
            PresentRegion::new(2, 2, 3, 1),
            PresentRegion::new(2, 3, 3, 1)
        ]
    );
    let fallback: heapless::Vec<PresentRegion, 1> = next.diff_regions(&previous);
    assert_eq!(fallback.as_slice(), &[PresentRegion::new(2, 2, 3, 2)]);
}

#[test]
fn tween_reaches_target() {
    let mut tween = Tween::new(0.0, 10.0, 100, Easing::Linear);
    assert!(!tween.tick(50));
    assert_eq!(tween.value(), 5.0);
    assert!(tween.tick(50));
    assert_eq!(tween.value(), 10.0);
}

#[test]
fn render_digest_stays_stable() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    gui.add_panel(Rect::new(0, 0, 40, 20), Style::panel())
        .unwrap();
    gui.add_button(Rect::new(2, 2, 30, 10), "GO", Style::button())
        .unwrap();
    gui.add_slider(Rect::new(2, 14, 30, 6), 0.5, 0.0, 1.0, Style::progress())
        .unwrap();

    let mut target = MockTarget::new(64, 32);
    gui.render(&mut target).unwrap();
    let digest = target.pixels.iter().fold(0u64, |acc, &(x, y, c)| {
        acc.wrapping_mul(16_777_619)
            ^ x as u64
            ^ ((y as u64) << 16)
            ^ ((c.r() as u64) << 32)
            ^ ((c.g() as u64) << 40)
            ^ ((c.b() as u64) << 48)
    });

    assert_eq!(digest, 4_248_994_834_688_788_393);
}

#[test]
fn parent_relative_widgets_have_absolute_rects() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 100, 80));
    let panel = gui
        .add_panel(Rect::new(10, 12, 50, 40), Style::panel())
        .unwrap();
    let child = gui
        .add_label(Rect::new(3, 4, 20, 8), "REL", Style::label())
        .unwrap();

    gui.add_child(panel, child).unwrap();

    assert_eq!(gui.absolute_rect(child), Some(Rect::new(13, 16, 20, 8)));
}

#[test]
fn event_path_reports_capture_target_and_bubble_order() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 80, 40));
    let root = gui
        .add_panel(Rect::new(0, 0, 60, 30), Style::panel())
        .unwrap();
    let child = gui
        .add_panel(Rect::new(2, 2, 40, 20), Style::panel())
        .unwrap();
    let target = gui
        .add_button(Rect::new(3, 3, 20, 10), "T", Style::button())
        .unwrap();
    gui.add_child(root, child).unwrap();
    gui.add_child(child, target).unwrap();

    let mut path = heapless::Vec::<EventContext, 8>::new();
    let count = gui.event_path(target, &mut path).unwrap();

    assert_eq!(count, 5);
    assert_eq!(
        path.as_slice(),
        &[
            EventContext {
                target,
                current: root,
                phase: EventPhase::Capture,
            },
            EventContext {
                target,
                current: child,
                phase: EventPhase::Capture,
            },
            EventContext {
                target,
                current: target,
                phase: EventPhase::Target,
            },
            EventContext {
                target,
                current: child,
                phase: EventPhase::Bubble,
            },
            EventContext {
                target,
                current: root,
                phase: EventPhase::Bubble,
            },
        ]
    );
}

#[test]
fn widget_event_path_includes_kind_and_phase_order() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 80, 40));
    let root = gui
        .add_panel(Rect::new(0, 0, 60, 30), Style::panel())
        .unwrap();
    let target = gui
        .add_button(Rect::new(3, 3, 20, 10), "T", Style::button())
        .unwrap();
    gui.add_child(root, target).unwrap();

    let mut path = heapless::Vec::<WidgetEvent, 8>::new();
    let count = gui
        .widget_event_path(target, WidgetEventKind::Pressed, &mut path)
        .unwrap();

    assert_eq!(count, 3);
    assert_eq!(
        path.as_slice(),
        &[
            WidgetEvent {
                target,
                current: root,
                phase: EventPhase::Capture,
                kind: WidgetEventKind::Pressed,
            },
            WidgetEvent {
                target,
                current: target,
                phase: EventPhase::Target,
                kind: WidgetEventKind::Pressed,
            },
            WidgetEvent {
                target,
                current: root,
                phase: EventPhase::Bubble,
                kind: WidgetEventKind::Pressed,
            },
        ]
    );
}

#[test]
fn widget_event_path_skips_bubble_when_flag_removed() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 80, 40));
    let root = gui
        .add_panel(Rect::new(0, 0, 60, 30), Style::panel())
        .unwrap();
    let target = gui
        .add_button(Rect::new(3, 3, 20, 10), "T", Style::button())
        .unwrap();
    gui.add_child(root, target).unwrap();
    gui.remove_flag(target, WidgetFlags::EVENT_BUBBLE).unwrap();

    let mut path = heapless::Vec::<WidgetEvent, 8>::new();
    let count = gui
        .widget_event_path(target, WidgetEventKind::Pressed, &mut path)
        .unwrap();

    assert_eq!(count, 2);
    assert_eq!(
        path.as_slice(),
        &[
            WidgetEvent {
                target,
                current: root,
                phase: EventPhase::Capture,
                kind: WidgetEventKind::Pressed,
            },
            WidgetEvent {
                target,
                current: target,
                phase: EventPhase::Target,
                kind: WidgetEventKind::Pressed,
            },
        ]
    );
}

#[test]
fn dispatch_widget_event_stops_when_handler_requests_stop() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 80, 40));
    let root = gui
        .add_panel(Rect::new(0, 0, 60, 30), Style::panel())
        .unwrap();
    let child = gui
        .add_panel(Rect::new(1, 1, 40, 20), Style::panel())
        .unwrap();
    let target = gui
        .add_button(Rect::new(3, 3, 20, 10), "T", Style::button())
        .unwrap();
    gui.add_child(root, child).unwrap();
    gui.add_child(child, target).unwrap();

    let mut path = heapless::Vec::<WidgetEvent, 8>::new();
    let mut seen = heapless::Vec::<WidgetEvent, 8>::new();
    gui.dispatch_widget_event(target, WidgetEventKind::Pressed, &mut path, |event| {
        let _ = seen.push(event);
        if event.current == child && event.phase == EventPhase::Capture {
            EventPolicy::Stop
        } else {
            EventPolicy::Continue
        }
    })
    .unwrap();

    assert_eq!(
        seen.as_slice(),
        &[
            WidgetEvent {
                target,
                current: root,
                phase: EventPhase::Capture,
                kind: WidgetEventKind::Pressed,
            },
            WidgetEvent {
                target,
                current: child,
                phase: EventPhase::Capture,
                kind: WidgetEventKind::Pressed,
            },
        ]
    );
}

#[test]
fn dispatch_widget_event_stops_when_registered_policy_matches() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 80, 40));
    let root = gui
        .add_panel(Rect::new(0, 0, 60, 30), Style::panel())
        .unwrap();
    let child = gui
        .add_panel(Rect::new(1, 1, 40, 20), Style::panel())
        .unwrap();
    let target = gui
        .add_button(Rect::new(3, 3, 20, 10), "T", Style::button())
        .unwrap();
    gui.add_child(root, child).unwrap();
    gui.add_child(child, target).unwrap();
    gui.set_dispatch_policy(
        child,
        WidgetDispatchPolicy::stop(WidgetEventFilter::POINTER, EventPhaseMask::CAPTURE),
    )
    .unwrap();

    let mut path = heapless::Vec::<WidgetEvent, 8>::new();
    let mut seen = heapless::Vec::<WidgetEvent, 8>::new();
    gui.dispatch_widget_event(target, WidgetEventKind::Pressed, &mut path, |event| {
        let _ = seen.push(event);
        EventPolicy::Continue
    })
    .unwrap();

    assert_eq!(
        seen.as_slice(),
        &[
            WidgetEvent {
                target,
                current: root,
                phase: EventPhase::Capture,
                kind: WidgetEventKind::Pressed,
            },
            WidgetEvent {
                target,
                current: child,
                phase: EventPhase::Capture,
                kind: WidgetEventKind::Pressed,
            },
        ]
    );
}

#[test]
fn scrollable_capture_stops_pointer_propagation_by_default() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 80, 40));
    let root = gui
        .add_panel(Rect::new(0, 0, 60, 30), Style::panel())
        .unwrap();
    let scroll = gui
        .add_scroll_view(Rect::new(1, 1, 40, 20), 0, 80, Style::panel())
        .unwrap();
    let target = gui
        .add_button(Rect::new(3, 3, 20, 10), "T", Style::button())
        .unwrap();
    gui.add_child(root, scroll).unwrap();
    gui.add_child(scroll, target).unwrap();

    let mut path = heapless::Vec::<WidgetEvent, 8>::new();
    let mut seen = heapless::Vec::<WidgetEvent, 8>::new();
    gui.dispatch_widget_event(target, WidgetEventKind::Pressed, &mut path, |event| {
        let _ = seen.push(event);
        EventPolicy::Continue
    })
    .unwrap();

    assert_eq!(
        seen.as_slice(),
        &[
            WidgetEvent {
                target,
                current: root,
                phase: EventPhase::Capture,
                kind: WidgetEventKind::Pressed,
            },
            WidgetEvent {
                target,
                current: scroll,
                phase: EventPhase::Capture,
                kind: WidgetEventKind::Pressed,
            },
        ]
    );
}

#[test]
fn app_event_queue_reports_overflow() {
    let mut gui = GuiContext::<1, 1, 1>::new(Rect::new(0, 0, 16, 16));

    gui.handle_input(InputEvent::Back).unwrap();
    assert_eq!(
        gui.handle_input(InputEvent::Back),
        Err(GuiError::EventsFull)
    );
    assert_eq!(gui.pop_event(), Some(UiEvent::Back));
}

#[test]
fn select_emits_pressed_clicked_activate_and_focus_events() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Select).unwrap();

    assert_eq!(gui.pop_event(), Some(UiEvent::Defocused(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Focused(button)));
    assert_eq!(
        gui.pop_event(),
        Some(UiEvent::FocusChanged {
            old: Some(button),
            new: Some(button),
        })
    );
    assert_eq!(gui.pop_event(), Some(UiEvent::Pressed(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Clicked(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Activate(button)));
}

#[test]
fn pointer_release_emits_release_events() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();

    assert_eq!(gui.pop_event(), Some(UiEvent::Released(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::PointerReleased(button)));
}

#[test]
fn event_filter_limits_targeted_app_events() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.set_event_filter(button, UiEventFilter::ACTIVATE)
        .unwrap();
    gui.handle_input(InputEvent::Select).unwrap();

    assert_eq!(gui.pop_event(), Some(UiEvent::Pressed(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Clicked(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Activate(button)));
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn event_filter_defaults_to_all_when_cleared() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_event_filter(button, UiEventFilter::ACTIVATE)
        .unwrap();
    gui.clear_event_filter(button).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::Defocused(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Focused(button)));
    assert!(gui.pop_event().is_some());
}

#[test]
fn parent_visibility_hides_children() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let panel = gui
        .add_panel(Rect::new(0, 0, 40, 20), Style::panel())
        .unwrap();
    let child = gui
        .add_label(Rect::new(2, 2, 20, 8), "HIDE", Style::label())
        .unwrap();
    gui.add_child(panel, child).unwrap();
    gui.set_visible(panel, false).unwrap();

    let mut target = MockTarget::new(64, 32);
    gui.render(&mut target).unwrap();

    assert!(target.pixels.is_empty());
    assert!(!gui.dirty_regions().is_empty());
}

#[test]
fn present_regions_follow_dirty_regions() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let value = gui
        .add_value_label(Rect::new(4, 5, 20, 8), "V", 1, Style::panel())
        .unwrap();
    gui.clear_dirty();

    gui.set_value_label(value, 2).unwrap();

    let regions: heapless::Vec<PresentRegion, 4> = gui.present_regions().collect();
    assert_eq!(regions.as_slice(), &[PresentRegion::new(4, 5, 20, 8)]);
    assert_eq!(
        gui.bounding_present_region(),
        Some(PresentRegion::new(4, 5, 20, 8))
    );
}

#[test]
fn themed_builders_use_context_theme() {
    let mut gui = GuiContext::<4, 4, 4>::new(Rect::new(0, 0, 64, 32));
    let mut theme = Theme::dark();
    theme.button = Style {
        background: Some(Rgb565::new(1, 2, 3)),
        gradient: None,
        font: FontId::Tiny3x5,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::new(4, 5, 6),
        opacity: 255,
        corner_radius: 0,
        shadow: None,
        border: Border::one(Rgb565::new(7, 8, 9)),
        padding: EdgeInsets::all(1),
    };
    gui.set_theme(theme).unwrap();

    let button = gui.add_themed_button(Rect::new(0, 0, 20, 10), "T").unwrap();

    assert_eq!(
        gui.widgets()
            .iter()
            .find(|w| w.id == button)
            .unwrap()
            .style
            .normal,
        theme.button
    );
}

#[test]
fn style_class_overrides_widget_style_at_render() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let label = gui
        .add_label(Rect::new(2, 2, 40, 12), "A", Style::label())
        .unwrap();
    let class = StyleClassId::new(1);
    let mut class_style = Style::label();
    class_style.text = Rgb565::RED;
    gui.set_style_class(class, class_style).unwrap();
    gui.set_widget_style_class(label, Some(class)).unwrap();

    let mut target = MockTarget::new(64, 32);
    gui.render(&mut target).unwrap();
    assert!(target.pixels.iter().any(|&(_, _, c)| c == Rgb565::RED));
}

#[test]
fn style_class_state_override_applies_only_in_state() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(2, 2, 40, 12), "A", Style::button())
        .unwrap();
    let class = StyleClassId::new(2);
    gui.set_widget_style_class(button, Some(class)).unwrap();

    let focused_bg = Rgb565::new(13, 1, 29);
    let mut focused_style = Style::button();
    focused_style.background = Some(focused_bg);
    focused_style.gradient = None;
    focused_style.shadow = None;
    focused_style.border = Border::none();
    gui.set_style_class_state(class, VisualState::Focused, focused_style)
        .unwrap();

    gui.set_focus(None).unwrap();
    let mut normal_target = MockTarget::new(64, 32);
    gui.render(&mut normal_target).unwrap();
    let normal_count = normal_target
        .pixels
        .iter()
        .filter(|&&(_, _, c)| c == focused_bg)
        .count();

    gui.set_focus(Some(button)).unwrap();
    let mut focused_target = MockTarget::new(64, 32);
    gui.render(&mut focused_target).unwrap();
    let focused_count = focused_target
        .pixels
        .iter()
        .filter(|&&(_, _, c)| c == focused_bg)
        .count();
    assert!(focused_count > normal_count);
}

#[test]
fn new_widgets_handle_input() {
    static TABS: [&str; 3] = ["A", "B", "C"];
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 96, 64));
    let tabs = gui
        .add_tabs(Rect::new(0, 0, 60, 10), &TABS, 0, Style::button())
        .unwrap();
    let scroll = gui
        .add_scroll_view(Rect::new(0, 12, 40, 20), 0, 100, Style::panel())
        .unwrap();
    let meter = gui
        .add_meter(Rect::new(44, 12, 40, 20), 0.0, 0.0, 1.0, Style::progress())
        .unwrap();

    gui.set_focus(Some(tabs)).unwrap();
    gui.handle_input(InputEvent::Right).unwrap();
    assert_eq!(gui.tab_selected(tabs), Some(1));

    gui.set_focus(Some(scroll)).unwrap();
    gui.handle_input(InputEvent::Down).unwrap();
    assert!(gui.scroll_offset(scroll).unwrap() > 0);

    gui.set_meter_value(meter, 0.75).unwrap();
    let mut target = MockTarget::new(96, 64);
    gui.render(&mut target).unwrap();
    assert!(!target.pixels.is_empty());
}
