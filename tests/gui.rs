use core::convert::Infallible;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
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
            overflow: TextOverflow::Clip,
            overflow_policy: TextOverflowPolicy::Global(TextOverflow::Clip),
            kerning: false,
            max_lines: None,
            ellipsis: EllipsisMode::ThreeDots,
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
fn nested_apply_layout_positions_children_relative_to_parent() {
    let mut gui = GuiContext::<12, 16, 12>::new(Rect::new(0, 0, 120, 80));
    let parent = gui
        .add_panel(Rect::new(10, 12, 60, 30), Style::panel())
        .unwrap();
    let a = gui
        .add_button(Rect::new(0, 0, 1, 1), "A", Style::button())
        .unwrap();
    let b = gui
        .add_button(Rect::new(0, 0, 1, 1), "B", Style::button())
        .unwrap();
    gui.add_child(parent, a).unwrap();
    gui.add_child(parent, b).unwrap();

    gui.apply_layout(
        LinearLayout::row().with_gap(2),
        Rect::new(1, 2, 40, 10),
        &[a, b],
    )
    .unwrap();

    let a_abs = gui.absolute_rect(a).unwrap();
    let b_abs = gui.absolute_rect(b).unwrap();
    assert_eq!(a_abs, Rect::new(11, 14, 19, 10));
    assert_eq!(b_abs, Rect::new(32, 14, 19, 10));
}

#[test]
fn nested_layout_respects_parent_clip_children_for_overflowing_child() {
    let mut gui = GuiContext::<12, 16, 12>::new(Rect::new(0, 0, 80, 40));
    let parent = gui
        .add_panel(Rect::new(6, 6, 20, 12), Style::panel())
        .unwrap();
    let child = gui
        .add_label(Rect::new(0, 0, 1, 1), "OVERFLOW LABEL", Style::label())
        .unwrap();
    gui.add_child(parent, child).unwrap();

    gui.apply_layout(LinearLayout::row(), Rect::new(0, 0, 40, 8), &[child])
        .unwrap();

    let mut target = MockTarget::new(80, 40);
    gui.render(&mut target).unwrap();
    let leaked_pixels = target
        .pixels
        .iter()
        .any(|&(x, y, _)| x > 26 && y >= 6 && y <= 18);
    assert!(!leaked_pixels);
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
fn animation_delay_repeat_and_ping_pong_work() {
    let mut anim = Animation::new(0.0, 100.0, 100, Easing::Linear)
        .with_delay(50)
        .with_repeat_mode(RepeatMode::PingPong)
        .with_repeat_count(Some(3));

    assert_eq!(anim.value(), 0.0);
    assert_eq!(anim.tick(40), AnimationState::Running);
    assert_eq!(anim.value(), 0.0);

    anim.tick(60);
    assert!(anim.value() > 40.0 && anim.value() < 60.0);

    anim.tick(100);
    assert!(anim.value() < 60.0);
    assert!(!anim.is_done());

    anim.tick(200);
    assert!(anim.is_done());
}

#[test]
fn animation_reverse_starts_from_target() {
    let mut anim = Animation::new(0.0, 10.0, 100, Easing::Linear);
    anim.set_reversed(true);
    assert_eq!(anim.value(), 10.0);
    anim.tick(50);
    assert_eq!(anim.value(), 5.0);
}

#[test]
fn animation_supports_custom_curve_callbacks() {
    fn hold_then_pop(t: f32) -> f32 {
        if t < 0.5 { 0.0 } else { 1.2 }
    }

    let mut anim = Animation::new(0.0, 10.0, 100, Easing::Linear).with_custom_curve(hold_then_pop);
    anim.tick(40);
    assert_eq!(anim.value(), 0.0);
    anim.tick(20);
    assert_eq!(anim.value(), 12.0);

    anim.clear_custom_curve();
    anim.reset();
    anim.tick(50);
    assert_eq!(anim.value(), 5.0);
}

#[test]
fn animation_supports_custom_interpolator_callbacks() {
    fn midpoint_bias(from: f32, to: f32, t: f32) -> f32 {
        if t < 0.5 { from } else { to }
    }

    let mut anim =
        Animation::new(10.0, 30.0, 100, Easing::Linear).with_custom_interpolator(midpoint_bias);
    anim.tick(49);
    assert_eq!(anim.value(), 10.0);
    anim.tick(1);
    assert_eq!(anim.value(), 30.0);

    anim.clear_custom_interpolator();
    anim.reset();
    anim.tick(50);
    assert_eq!(anim.value(), 20.0);
}

#[test]
fn animation_per_track_handlers_fire_for_start_and_stop() {
    static STARTED: AtomicUsize = AtomicUsize::new(0);
    static STOPPED_FINISH: AtomicUsize = AtomicUsize::new(0);
    static STOPPED_CANCEL: AtomicUsize = AtomicUsize::new(0);

    fn on_started() {
        STARTED.fetch_add(1, Ordering::Relaxed);
    }
    fn on_stopped(finished: bool) {
        if finished {
            STOPPED_FINISH.fetch_add(1, Ordering::Relaxed);
        } else {
            STOPPED_CANCEL.fetch_add(1, Ordering::Relaxed);
        }
    }

    STARTED.store(0, Ordering::Relaxed);
    STOPPED_FINISH.store(0, Ordering::Relaxed);
    STOPPED_CANCEL.store(0, Ordering::Relaxed);

    let mut anim = Animation::new(0.0, 1.0, 20, Easing::Linear).with_delay(10);
    anim.set_handlers(AnimationHandlers {
        on_started: Some(on_started),
        on_stopped: Some(on_stopped),
    });
    anim.tick(9);
    assert_eq!(STARTED.load(Ordering::Relaxed), 0);
    anim.tick(1);
    assert_eq!(STARTED.load(Ordering::Relaxed), 1);
    anim.tick(20);
    assert_eq!(STOPPED_FINISH.load(Ordering::Relaxed), 1);

    let mut manager = AnimationManager::<2>::new();
    let mut cancel_anim = Animation::new(0.0, 1.0, 100, Easing::Linear);
    cancel_anim.set_handlers(AnimationHandlers {
        on_started: None,
        on_stopped: Some(on_stopped),
    });
    let id = manager.start(cancel_anim).unwrap();
    assert!(manager.stop(id));
    assert_eq!(STOPPED_CANCEL.load(Ordering::Relaxed), 1);
}

#[test]
fn speed_helper_calculates_duration() {
    let ms = Animation::duration_from_speed(120.0, 60.0);
    assert_eq!(ms, 2000);
}

#[test]
fn animation_total_duration_includes_delay_and_repeat_count() {
    let once = Animation::new(0.0, 1.0, 100, Easing::Linear).with_delay(20);
    assert_eq!(once.total_duration_ms(false, false), Some(100));
    assert_eq!(once.total_duration_ms(true, false), Some(120));
    assert_eq!(once.total_duration_ms(true, true), Some(120));

    let finite = Animation::new(0.0, 1.0, 100, Easing::Linear)
        .with_delay(10)
        .with_repeat_mode(RepeatMode::Loop)
        .with_repeat_count(Some(3));
    assert_eq!(finite.total_duration_ms(true, true), Some(330));

    let infinite = Animation::new(0.0, 1.0, 100, Easing::Linear)
        .with_repeat_mode(RepeatMode::Loop)
        .with_repeat_count(None);
    assert_eq!(infinite.total_duration_ms(true, true), None);
}

#[test]
fn animation_manager_runs_multiple_tracks_and_reclaims_slots() {
    let mut manager = AnimationManager::<2>::new();
    let a = manager
        .start(Animation::new(0.0, 10.0, 100, Easing::Linear))
        .unwrap();
    let b = manager
        .start(
            Animation::new(0.0, 1.0, 100, Easing::Linear)
                .with_repeat_mode(RepeatMode::Loop)
                .with_repeat_count(Some(4)),
        )
        .unwrap();

    assert_eq!(
        manager.start(Animation::new(0.0, 5.0, 100, Easing::Linear)),
        Err(AnimationError::Full)
    );
    assert_eq!(manager.active_count(), 2);

    manager.tick(100);
    assert!(manager.value(a).is_none());
    assert!(manager.value(b).is_some());
    assert_eq!(manager.active_count(), 1);

    manager.tick(400);
    assert!(manager.value(b).is_none());
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn animation_manager_can_stop_track_early() {
    let mut manager = AnimationManager::<2>::new();
    let id = manager
        .start(Animation::new(0.0, 10.0, 200, Easing::Linear))
        .unwrap();
    assert!(manager.stop(id));
    assert!(manager.value(id).is_none());
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn animation_manager_callbacks_pause_and_seek_work() {
    static STARTED: AtomicUsize = AtomicUsize::new(0);
    static REPEATED: AtomicUsize = AtomicUsize::new(0);
    static COMPLETED_FINISHED: AtomicUsize = AtomicUsize::new(0);
    static COMPLETED_STOPPED: AtomicUsize = AtomicUsize::new(0);

    fn on_start(_: AnimationId) {
        STARTED.fetch_add(1, Ordering::Relaxed);
    }
    fn on_repeat(_: AnimationId, iteration: u16) {
        REPEATED.fetch_add(1, Ordering::Relaxed);
        let _ = iteration;
    }
    fn on_complete(_: AnimationId, finished: bool) {
        if finished {
            COMPLETED_FINISHED.fetch_add(1, Ordering::Relaxed);
        } else {
            COMPLETED_STOPPED.fetch_add(1, Ordering::Relaxed);
        }
    }

    STARTED.store(0, Ordering::Relaxed);
    REPEATED.store(0, Ordering::Relaxed);
    COMPLETED_FINISHED.store(0, Ordering::Relaxed);
    COMPLETED_STOPPED.store(0, Ordering::Relaxed);

    let mut manager = AnimationManager::<3>::new();
    manager.set_callbacks(AnimationManagerCallbacks {
        on_start: Some(on_start),
        on_repeat: Some(on_repeat),
        on_complete: Some(on_complete),
    });
    let id = manager
        .start(
            Animation::new(0.0, 1.0, 10, Easing::Linear)
                .with_repeat_mode(RepeatMode::Loop)
                .with_repeat_count(Some(3)),
        )
        .unwrap();
    assert_eq!(STARTED.load(Ordering::Relaxed), 1);

    manager.set_paused(true);
    manager.tick(10);
    let paused_value = manager.value(id).unwrap();
    manager.tick(10);
    assert_eq!(manager.value(id).unwrap(), paused_value);

    manager.set_paused(false);
    assert!(!manager.is_paused());
    assert!(manager.seek(id, 25));
    assert!(manager.seek_stepped(id, 30, 2));
    manager.tick(5);
    assert_eq!(COMPLETED_FINISHED.load(Ordering::Relaxed), 1);

    let stop_id = manager
        .start(Animation::new(0.0, 1.0, 100, Easing::Linear))
        .unwrap();
    assert!(manager.stop(stop_id));
    assert_eq!(COMPLETED_STOPPED.load(Ordering::Relaxed), 1);
}

#[test]
fn animation_manager_seek_stepped_advances_to_target_elapsed() {
    let mut manager = AnimationManager::<2>::new();
    let id = manager
        .start(Animation::new(0.0, 100.0, 100, Easing::Linear))
        .unwrap();
    assert!(manager.seek_stepped(id, 75, 7));
    let value = manager.value(id).unwrap();
    assert!(value >= 74.0 && value <= 76.0);
}

#[test]
fn animation_manager_replay_stepped_emits_samples() {
    let mut manager = AnimationManager::<2>::new();
    let id = manager
        .start(Animation::new(0.0, 100.0, 100, Easing::Linear))
        .unwrap();
    let mut samples = heapless::Vec::<f32, 32>::new();
    assert!(manager.replay_stepped(id, 60, 10, |v| {
        let _ = samples.push(v);
    }));
    assert_eq!(samples.len(), 6);
    assert!(samples[samples.len() - 1] >= 59.0 && samples[samples.len() - 1] <= 61.0);
}

#[test]
fn widget_animator_updates_progress_and_meter_values() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 96, 64));
    let progress = gui
        .add_progress_bar(Rect::new(0, 0, 50, 10), 0.0, Style::progress())
        .unwrap();
    let meter = gui
        .add_meter(Rect::new(0, 12, 20, 20), 0.0, 0.0, 1.0, Style::progress())
        .unwrap();
    gui.clear_dirty();

    let mut animator = WidgetAnimator::<4, 4>::new();
    animator
        .animate_progress(progress, 0.0, 1.0, 100, Easing::Linear)
        .unwrap();
    animator
        .animate_meter(meter, 0.0, 1.0, 100, Easing::Linear)
        .unwrap();

    animator.tick(50, &mut gui).unwrap();
    assert_eq!(gui.pop_event(), None);
    assert!(!gui.dirty_regions().is_empty());
    gui.clear_dirty();

    animator.tick(50, &mut gui).unwrap();
    assert_eq!(animator.active_count(), 0);
}

#[test]
fn widget_animator_reports_binding_overflow() {
    let mut gui = GuiContext::<4, 4, 4>::new(Rect::new(0, 0, 32, 16));
    let progress = gui
        .add_progress_bar(Rect::new(0, 0, 20, 8), 0.0, Style::progress())
        .unwrap();

    let mut animator = WidgetAnimator::<4, 1>::new();
    animator
        .bind_property_with_policy(
            progress,
            AnimatedProperty::Progress,
            Animation::new(0.0, 1.0, 100, Easing::Linear),
            AnimationConflictPolicy::Queue,
        )
        .unwrap();
    assert_eq!(
        animator.bind_property_with_policy(
            progress,
            AnimatedProperty::Progress,
            Animation::new(0.0, 1.0, 100, Easing::Linear),
            AnimationConflictPolicy::Queue,
        ),
        Err(WidgetAnimationError::BindingsFull)
    );
}

#[test]
fn widget_animator_ignore_policy_reports_conflict_ignored() {
    let mut gui = GuiContext::<4, 4, 4>::new(Rect::new(0, 0, 32, 16));
    let progress = gui
        .add_progress_bar(Rect::new(0, 0, 20, 8), 0.0, Style::progress())
        .unwrap();
    let mut animator = WidgetAnimator::<4, 4>::new();
    animator
        .bind_property_with_policy(
            progress,
            AnimatedProperty::Progress,
            Animation::new(0.0, 1.0, 100, Easing::Linear),
            AnimationConflictPolicy::Ignore,
        )
        .unwrap();
    assert_eq!(
        animator.bind_property_with_policy(
            progress,
            AnimatedProperty::Progress,
            Animation::new(0.0, 1.0, 100, Easing::Linear),
            AnimationConflictPolicy::Ignore,
        ),
        Err(WidgetAnimationError::ConflictIgnored)
    );
}

#[test]
fn widget_animator_convenience_builders_work() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let panel = gui
        .add_panel(Rect::new(1, 1, 20, 10), Style::panel())
        .unwrap();
    let slider = gui
        .add_slider(Rect::new(1, 14, 20, 8), 0.0, 0.0, 1.0, Style::button())
        .unwrap();
    let scroll = gui
        .add_scroll_view(Rect::new(24, 1, 20, 20), 0, 100, Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<8, 8>::new();

    animator
        .animate_widget_x(panel, 1, 5, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_widget_height(panel, 10, 14, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_opacity(panel, 255, 64, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_slider_value(slider, 0.0, 1.0, 20, Easing::Linear)
        .unwrap();
    animator
        .animate_scroll_offset_y(scroll, 0, 16, 20, Easing::Linear)
        .unwrap();
    animator.tick(20, &mut gui).unwrap();

    assert_eq!(gui.absolute_rect(panel).unwrap().x, 5);
    assert_eq!(gui.absolute_rect(panel).unwrap().h, 14);
    assert!(gui.slider_value(slider).unwrap() > 0.9);
    assert_eq!(gui.scroll_offset(scroll).unwrap(), 16);
}

#[test]
fn widget_animator_policy_aware_builders_apply_conflict_rules() {
    let mut gui = GuiContext::<4, 4, 4>::new(Rect::new(0, 0, 48, 20));
    let panel = gui
        .add_panel(Rect::new(0, 0, 20, 8), Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<4, 4>::new();
    animator
        .animate_widget_x_with_policy(
            panel,
            0,
            10,
            100,
            Easing::Linear,
            AnimationConflictPolicy::Ignore,
        )
        .unwrap();
    assert_eq!(
        animator.animate_widget_x_with_policy(
            panel,
            0,
            12,
            100,
            Easing::Linear,
            AnimationConflictPolicy::Ignore
        ),
        Err(WidgetAnimationError::ConflictIgnored)
    );
}

#[test]
fn widget_animator_extended_property_builders_work() {
    static TABS: [&str; 3] = ["A", "B", "C"];
    static ITEMS: [&str; 4] = ["ONE", "TWO", "THREE", "FOUR"];
    let mut gui = GuiContext::<20, 20, 20>::new(Rect::new(0, 0, 140, 80));
    let tabs = gui
        .add_tabs(Rect::new(0, 0, 60, 12), &TABS, 0, Style::button())
        .unwrap();
    let dropdown = gui
        .add_dropdown(Rect::new(62, 0, 56, 12), &ITEMS, 0, Style::button())
        .unwrap();
    let roller = gui
        .add_roller(Rect::new(0, 14, 40, 24), &ITEMS, 0, Style::button())
        .unwrap();
    let gauge = gui
        .add_gauge(Rect::new(42, 14, 24, 24), 0.0, 0.0, 1.0, Style::progress())
        .unwrap();
    let spinner = gui
        .add_spinner(Rect::new(68, 14, 16, 16), 0.0, Style::progress())
        .unwrap();
    let progress = gui
        .add_progress_bar(Rect::new(0, 42, 84, 8), 0.0, Style::progress())
        .unwrap();
    let card = gui
        .add_panel(Rect::new(86, 14, 30, 20), Style::panel())
        .unwrap();

    let mut animator = WidgetAnimator::<24, 24>::new();
    animator
        .animate_tab_selected(tabs, 0, 2, 40, Easing::Linear)
        .unwrap();
    animator
        .animate_dropdown_selected(dropdown, 0, 3, 40, Easing::Linear)
        .unwrap();
    animator
        .animate_roller_selected(roller, 0, 2, 40, Easing::Linear)
        .unwrap();
    animator
        .animate_gauge_value(gauge, 0.0, 1.0, 40, Easing::Linear)
        .unwrap();
    animator
        .animate_spinner_phase(spinner, 0.0, 1.0, 40, Easing::Linear)
        .unwrap();
    animator
        .ping_pong_progress(progress, 0.0, 1.0, 20, Easing::InOutSine)
        .unwrap();
    animator
        .pulse_opacity(card, 32, 200, 20, Easing::InOutSine)
        .unwrap();

    animator.tick(20, &mut gui).unwrap();
    assert_eq!(gui.tab_selected(tabs), Some(1));
    assert!(gui.dropdown_selected(dropdown).is_some());
    assert!(gui.roller_selected(roller).is_some());

    animator.tick(20, &mut gui).unwrap();
    assert_eq!(gui.tab_selected(tabs), Some(2));
    assert_eq!(gui.dropdown_selected(dropdown), Some(3));
    assert_eq!(gui.roller_selected(roller), Some(2));
}

#[test]
fn widget_animator_stagger_path_and_presets_work() {
    let mut gui = GuiContext::<16, 32, 16>::new(Rect::new(0, 0, 120, 80));
    let a = gui
        .add_panel(Rect::new(0, 0, 16, 10), Style::panel())
        .unwrap();
    let b = gui
        .add_panel(Rect::new(0, 12, 16, 10), Style::panel())
        .unwrap();
    let c = gui
        .add_panel(Rect::new(0, 24, 16, 10), Style::panel())
        .unwrap();
    let focus = gui
        .add_panel(Rect::new(10, 40, 20, 12), Style::panel())
        .unwrap();

    let mut animator = WidgetAnimator::<32, 32>::new();
    assert_eq!(
        animator
            .stagger_widget_x(&[a, b, c], 0, 20, 30, 10, Easing::Linear)
            .unwrap(),
        3
    );

    let path = [
        PathPoint::new(10.0, 40.0),
        PathPoint::new(20.0, 46.0),
        PathPoint::new(34.0, 42.0),
    ];
    animator
        .animate_widget_path(focus, &path, 60, Easing::InOutSine)
        .unwrap();
    animator.preset_fade_in_up(focus, 46, 40, 40).unwrap();
    animator.preset_attention_shake(focus, 34, 2, 40).unwrap();
    animator
        .animate_corner_radius(focus, 0, 4, 40, Easing::Linear)
        .unwrap();
    animator
        .animate_accent_color(
            focus,
            Rgb565::new(0, 20, 10),
            Rgb565::new(20, 55, 5),
            40,
            Easing::Linear,
        )
        .unwrap();

    animator.tick(10, &mut gui).unwrap();
    assert_eq!(gui.absolute_rect(a).unwrap().x, 7);
    assert_eq!(gui.absolute_rect(b).unwrap().x, 0);
    animator.tick(10, &mut gui).unwrap();
    assert!(gui.absolute_rect(b).unwrap().x > 0);
    animator.tick(40, &mut gui).unwrap();
    let focus_node = gui.widgets().iter().find(|w| w.id == focus).unwrap();
    assert!(focus_node.style.normal.corner_radius >= 3);
}

#[test]
fn animation_presets_namespace_helpers_work() {
    let mut gui = GuiContext::<16, 32, 16>::new(Rect::new(0, 0, 120, 80));
    let a = gui
        .add_panel(Rect::new(4, 4, 18, 10), Style::panel())
        .unwrap();
    let b = gui
        .add_panel(Rect::new(4, 18, 18, 10), Style::panel())
        .unwrap();
    let c = gui
        .add_panel(Rect::new(4, 32, 18, 10), Style::panel())
        .unwrap();
    let focus = gui
        .add_panel(Rect::new(40, 20, 22, 12), Style::panel())
        .unwrap();

    let mut animator = WidgetAnimator::<32, 32>::new();
    assert_eq!(
        presets::orchestrate_stagger_x(&mut animator, &[a, b, c], 4, 24, 40, 10).unwrap(),
        3
    );
    presets::entrance_fade_in_up(&mut animator, focus, 30, 20, 40).unwrap();
    presets::attention_shake(&mut animator, focus, 40, 2, 40).unwrap();
    presets::style_breathe(&mut animator, focus, 96, 200, 0, 4, 40).unwrap();
    presets::style_accent_cycle(
        &mut animator,
        focus,
        Rgb565::new(0, 20, 10),
        Rgb565::new(20, 55, 5),
        40,
    )
    .unwrap();
    presets::path_float_loop(&mut animator, focus, 40, 20, 2, 40).unwrap();

    animator.tick(10, &mut gui).unwrap();
    assert!(gui.absolute_rect(a).unwrap().x > 4);
    animator.tick(40, &mut gui).unwrap();
    let node = gui.widgets().iter().find(|w| w.id == focus).unwrap();
    assert!(node.style.normal.corner_radius >= 3);
}

#[test]
fn animation_edge_cases_cover_delay_reverse_and_id_wrap() {
    let mut anim = Animation::new(0.0, 1.0, 0, Easing::Linear)
        .with_delay(20)
        .with_repeat_mode(RepeatMode::Loop)
        .with_repeat_count(Some(2));
    anim.set_reversed(true);
    assert_eq!(anim.value(), 1.0);
    anim.tick(20);
    assert!(anim.value() <= 1.0);

    let mut manager = AnimationManager::<1>::new();
    manager.set_next_id_for_test(u16::MAX);
    let first = manager
        .start(Animation::new(0.0, 1.0, 1, Easing::Linear))
        .unwrap();
    assert_eq!(first.raw(), u16::MAX);
    manager.tick(1);
    let second = manager
        .start(Animation::new(0.0, 1.0, 1, Easing::Linear))
        .unwrap();
    assert_eq!(second.raw(), 1);
}

#[test]
fn widget_animator_animates_geometry_and_style_properties() {
    let mut gui = GuiContext::<8, 8, 16>::new(Rect::new(0, 0, 96, 64));
    let slider = gui
        .add_slider(Rect::new(1, 1, 20, 8), 0.0, 0.0, 1.0, Style::button())
        .unwrap();
    let scroll = gui
        .add_scroll_view(Rect::new(0, 12, 30, 20), 0, 120, Style::panel())
        .unwrap();
    let panel = gui
        .add_panel(Rect::new(4, 40, 20, 10), Style::panel())
        .unwrap();
    gui.clear_dirty();

    let mut animator = WidgetAnimator::<8, 8>::new();
    animator
        .bind_property(
            slider,
            AnimatedProperty::SliderValue,
            Animation::new(0.0, 1.0, 100, Easing::Linear),
        )
        .unwrap();
    animator
        .bind_property(
            scroll,
            AnimatedProperty::ScrollOffsetY,
            Animation::new(0.0, 50.0, 100, Easing::Linear),
        )
        .unwrap();
    animator
        .bind_property(
            panel,
            AnimatedProperty::WidgetX,
            Animation::new(4.0, 10.0, 100, Easing::Linear),
        )
        .unwrap();
    animator
        .bind_property(
            panel,
            AnimatedProperty::Opacity,
            Animation::new(255.0, 64.0, 100, Easing::Linear),
        )
        .unwrap();
    animator.tick(100, &mut gui).unwrap();

    assert!(gui.slider_value(slider).unwrap() > 0.9);
    assert!(gui.scroll_offset(scroll).unwrap() >= 49);
    assert_eq!(gui.absolute_rect(panel).unwrap().x, 10);
}

#[test]
fn timeline_sequence_and_group_work() {
    let mut seq = AnimationSequence::<4>::new();
    seq.push_delay(10).unwrap();
    seq.push_animation(Animation::new(0.0, 1.0, 20, Easing::Linear))
        .unwrap();
    let mut player = SequencePlayer::<2, 4>::new(seq);
    player.tick(10).unwrap();
    player.tick(1).unwrap();
    assert!(player.active_value().is_some());

    let mut group = AnimationGroup::<2>::new();
    group
        .push(Animation::new(0.0, 1.0, 10, Easing::Linear))
        .unwrap();
    group
        .push(Animation::new(1.0, 0.0, 10, Easing::Linear))
        .unwrap();
    let mut manager = AnimationManager::<2>::new();
    let ids = group.start(&mut manager).unwrap();
    assert!(ids[0].is_some());
    assert!(ids[1].is_some());
}

#[test]
fn render_primitives_and_word_wrap_and_image_compile_path() {
    let mut target = TestBuffer::new(32, 32);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 32, 32));
    ctx.draw_line(0, 0, 10, 10, Rgb565::WHITE).unwrap();
    ctx.stroke_circle(16, 16, 5, Rgb565::RED).unwrap();
    ctx.fill_circle(6, 20, 3, Rgb565::GREEN).unwrap();
    ctx.stroke_arc(16, 16, 8, 0, 90, Rgb565::BLUE).unwrap();
    let poly = [Point::new(20, 20), Point::new(28, 20), Point::new(24, 28)];
    ctx.fill_polygon(&poly, Rgb565::CYAN).unwrap();
    let img = ImageRef::new(2, 2, &[0xFFFF, 0xF800, 0x07E0, 0x001F]);
    ctx.draw_image(Rect::new(0, 24, 4, 4), img, ImageFit::Stretch)
        .unwrap();
    ctx.draw_text_in(
        Rect::new(0, 0, 12, 16),
        "HELLO WORLD",
        TextStyle::new(Rgb565::WHITE).with_wrap(TextWrap::Word),
    )
    .unwrap();
    assert!(target.count_color(Rgb565::WHITE) > 0);
}

#[test]
fn screen_transition_runner_tracks_progress() {
    let mut stack = ScreenStack::<4>::with_root(ScreenId::new(1)).unwrap();
    let mut events = heapless::Vec::<ScreenLifecycleEvent, 8>::new();
    let mut runner = ScreenTransitionRunner::new();
    runner
        .apply(
            &mut stack,
            ScreenCommand::Push(ScreenId::new(2)),
            ScreenTransitionSpec::fade(100),
            &mut events,
        )
        .unwrap();
    runner.tick(50);
    let active = runner.active().unwrap();
    assert!(active.opacity_u8() > 0);
    assert!(active.slide_offset_x(100) == 0);
    let sample = active.sample(100, 80);
    assert!(sample.incoming_opacity > 0);
    let zoom = ActiveScreenTransition {
        from: Some(ScreenId::new(1)),
        to: Some(ScreenId::new(2)),
        effect: ScreenTransitionEffect::Zoom,
        origin: ScreenTransitionOrigin::Center,
        progress: 0.6,
    };
    let zoom_sample = zoom.sample(100, 80);
    assert!(zoom_sample.incoming_opacity > zoom_sample.outgoing_opacity / 2);
    let circular = ActiveScreenTransition {
        from: Some(ScreenId::new(1)),
        to: Some(ScreenId::new(2)),
        effect: ScreenTransitionEffect::CircularReveal,
        origin: ScreenTransitionOrigin::TopLeft,
        progress: 0.4,
    };
    let circular_sample = circular.sample(100, 80);
    assert!(circular_sample.incoming_clip.is_some());
    runner.tick(100);
    assert!(runner.active().is_none());
}

#[test]
fn context_can_render_with_offset_and_opacity() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 32, 16));
    gui.add_panel(Rect::new(1, 1, 10, 8), Style::panel())
        .unwrap();
    let mut target = TestBuffer::new(32, 16);
    gui.render_with_offset_and_opacity(&mut target, 4, 0, 128)
        .unwrap();
    assert!(target.digest() != 0);
}

#[test]
fn transition_compositor_renders_outgoing_and_incoming_contexts() {
    let mut outgoing = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 32, 16));
    let mut incoming = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 32, 16));
    outgoing
        .add_panel(Rect::new(1, 1, 10, 8), Style::panel())
        .unwrap();
    incoming
        .add_panel(Rect::new(12, 2, 10, 8), Style::button())
        .unwrap();
    let mut target = TestBuffer::new(32, 16);

    render_transition_pair(
        &mut target,
        &outgoing,
        &incoming,
        ActiveScreenTransition {
            from: Some(ScreenId::new(1)),
            to: Some(ScreenId::new(2)),
            effect: ScreenTransitionEffect::Fade,
            origin: ScreenTransitionOrigin::Center,
            progress: 0.5,
        },
        32,
        16,
    )
    .unwrap();

    assert!(target.digest() != 0);
}

#[test]
fn transition_wipe_and_origin_variants_produce_clips() {
    let wipe = ActiveScreenTransition {
        from: Some(ScreenId::new(1)),
        to: Some(ScreenId::new(2)),
        effect: ScreenTransitionEffect::WipeRight,
        origin: ScreenTransitionOrigin::Center,
        progress: 0.5,
    };
    let sample = wipe.sample(100, 80);
    assert!(sample.incoming_clip.is_some());

    let reveal =
        ScreenTransitionSpec::circular_reveal(300).with_origin(ScreenTransitionOrigin::BottomRight);
    assert_eq!(reveal.origin, ScreenTransitionOrigin::BottomRight);
    let eased = ScreenTransitionSpec::wipe_down(280).with_easing(Easing::OutBack);
    assert_eq!(eased.easing, Easing::OutBack);
}

#[test]
fn keyframe_track_advances_and_finishes() {
    let mut track = KeyframeTrack::<4>::new();
    track
        .push(Keyframe {
            value: 0.5,
            duration_ms: 20,
            easing: Easing::Linear,
        })
        .unwrap();
    track
        .push(Keyframe {
            value: 1.0,
            duration_ms: 20,
            easing: Easing::Linear,
        })
        .unwrap();
    track.reset(0.0);
    track.tick(10).unwrap();
    assert!(track.value().unwrap() > 0.0);
    track.tick(10).unwrap();
    track.tick(10).unwrap();
    assert!(track.value().unwrap() >= 0.5);
    track.tick(10).unwrap();
    assert!(track.is_done());
    assert!(track.value().unwrap() >= 1.0);
}

#[test]
fn keyframe_track_callbacks_fire_per_segment() {
    static STARTS: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
    static ENDS: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
    fn on_start(_idx: usize, _from: f32, _to: f32) {
        STARTS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }
    fn on_end(_idx: usize, _value: f32) {
        ENDS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    let mut track = KeyframeTrack::<3>::new();
    track
        .push(Keyframe {
            value: 0.5,
            duration_ms: 10,
            easing: Easing::Linear,
        })
        .unwrap();
    track
        .push(Keyframe {
            value: 1.0,
            duration_ms: 10,
            easing: Easing::Linear,
        })
        .unwrap();
    track.set_callbacks(KeyframeTrackCallbacks {
        on_segment_start: Some(on_start),
        on_segment_complete: Some(on_end),
    });
    track.reset(0.0);
    track.tick(10).unwrap();
    track.tick(10).unwrap();

    assert_eq!(STARTS.load(core::sync::atomic::Ordering::Relaxed), 2);
    assert_eq!(ENDS.load(core::sync::atomic::Ordering::Relaxed), 2);
    STARTS.store(0, core::sync::atomic::Ordering::Relaxed);
    ENDS.store(0, core::sync::atomic::Ordering::Relaxed);
}

#[test]
fn widget_animator_introspection_and_stop_helpers_work() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let panel = gui
        .add_panel(Rect::new(1, 1, 20, 10), Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<8, 8>::new();
    animator
        .animate_widget_x(panel, 1, 8, 100, Easing::Linear)
        .unwrap();
    animator
        .animate_opacity(panel, 255, 64, 100, Easing::Linear)
        .unwrap();
    assert!(animator.is_animating_widget(panel));
    assert!(animator.is_animating_widget_property(panel, AnimatedProperty::WidgetX));

    let mut handles = heapless::Vec::<AnimationId, 8>::new();
    assert_eq!(animator.handles_for_widget(panel, &mut handles), 2);
    assert_eq!(
        animator.stop_widget_property(panel, AnimatedProperty::WidgetX),
        1
    );
    assert_eq!(animator.stop_widget(panel), 1);
    assert!(!animator.is_animating_widget(panel));
}

#[test]
fn widget_animator_selection_bump_settle_moves_and_returns() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 80, 40));
    let panel = gui
        .add_panel(Rect::new(10, 20, 20, 10), Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<8, 8>::new();
    animator
        .preset_selection_bump_settle(panel, 20, 4, 60)
        .unwrap();
    animator.tick(20, &mut gui).unwrap();
    assert!(gui.absolute_rect(panel).unwrap().y < 20);
    animator.tick(60, &mut gui).unwrap();
    assert_eq!(gui.absolute_rect(panel).unwrap().y, 20);
}

#[test]
fn widget_animator_custom_curve_and_interpolator_helpers_work() {
    fn snap_interp(from: f32, to: f32, t: f32) -> f32 {
        if t < 0.5 { from } else { to }
    }
    fn hold_then_jump(t: f32) -> f32 {
        if t < 0.7 { 0.0 } else { 1.0 }
    }

    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 80, 40));
    let panel = gui
        .add_panel(Rect::new(10, 10, 20, 10), Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<8, 8>::new();
    animator
        .animate_widget_x_with_custom_interpolator(
            panel,
            10,
            30,
            40,
            Easing::Linear,
            snap_interp,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator
        .animate_widget_y_with_custom_curve(
            panel,
            10,
            30,
            40,
            Easing::Linear,
            hold_then_jump,
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    animator.tick(20, &mut gui).unwrap();
    let rect_mid = gui.absolute_rect(panel).unwrap();
    assert_eq!(rect_mid.x, 30);
    assert_eq!(rect_mid.y, 10);
    animator.tick(40, &mut gui).unwrap();
    let rect_end = gui.absolute_rect(panel).unwrap();
    assert_eq!(rect_end.y, 30);
}

#[test]
fn widget_animator_multi_property_keyframes_apply_in_sequence() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 100, 60));
    let panel = gui
        .add_panel(Rect::new(10, 10, 20, 10), Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<16, 16>::new();
    let count = animator
        .animate_widget_keyframes(
            panel,
            WidgetKeyframeState {
                x: 10,
                y: 10,
                opacity: 255,
            },
            &[
                WidgetPropertyKeyframe {
                    x: Some(20),
                    y: Some(12),
                    opacity: Some(220),
                    duration_ms: 20,
                    easing: Easing::Linear,
                },
                WidgetPropertyKeyframe {
                    x: Some(30),
                    y: Some(16),
                    opacity: Some(128),
                    duration_ms: 20,
                    easing: Easing::Linear,
                },
            ],
            AnimationConflictPolicy::Replace,
        )
        .unwrap();
    assert_eq!(count, 6);
    animator.tick(20, &mut gui).unwrap();
    let mid = gui.absolute_rect(panel).unwrap();
    assert_eq!(mid.x, 20);
    assert_eq!(mid.y, 12);
    animator.tick(20, &mut gui).unwrap();
    let end = gui.absolute_rect(panel).unwrap();
    assert_eq!(end.x, 30);
    assert_eq!(end.y, 16);
}

#[test]
fn sequence_player_can_seek_to_label() {
    let mut seq = AnimationSequence::<6>::new();
    seq.push_label(1).unwrap();
    seq.push_animation(Animation::new(0.0, 0.5, 40, Easing::Linear))
        .unwrap();
    seq.push_label(2).unwrap();
    seq.push_animation(Animation::new(0.5, 1.0, 40, Easing::Linear))
        .unwrap();

    let mut player = SequencePlayer::<2, 6>::new(seq);
    player.seek_to_label(2).unwrap();
    player.tick(20).unwrap();
    let status = player.status();
    assert!(status.active || status.done);
    assert!(player.active_value().is_some());
}

#[test]
fn widget_animator_can_snapshot_active_bindings() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let panel = gui
        .add_panel(Rect::new(1, 1, 20, 10), Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<8, 8>::new();
    animator
        .animate_widget_x(panel, 1, 8, 100, Easing::Linear)
        .unwrap();
    animator
        .animate_opacity(panel, 255, 64, 100, Easing::Linear)
        .unwrap();

    let mut snapshots = heapless::Vec::<BindingSnapshot, 8>::new();
    let count = animator.active_bindings(&mut snapshots);
    assert_eq!(count, 2);
    assert!(
        snapshots
            .iter()
            .any(|s| s.property == AnimatedProperty::WidgetX)
    );
    assert!(
        snapshots
            .iter()
            .any(|s| s.property == AnimatedProperty::Opacity)
    );
}

#[test]
fn sequence_player_loop_mode_restarts() {
    let mut seq = AnimationSequence::<2>::new();
    seq.push_animation(Animation::new(0.0, 1.0, 10, Easing::Linear))
        .unwrap();
    let mut player = SequencePlayer::<2, 2>::new(seq);
    player.set_repeat_mode(SequenceRepeatMode::Loop);
    player.tick(10).unwrap();
    let first = player.status().step_idx;
    player.tick(10).unwrap();
    assert_eq!(first, 0);
    assert_eq!(player.status().step_idx, 0);
}

#[test]
fn composed_animation_player_sequence_controls_repeat_and_reverse() {
    let mut composition = ComposedAnimation::<4>::new(CompositionMode::Sequence);
    composition
        .push(Animation::new(0.0, 1.0, 20, Easing::Linear))
        .unwrap();
    composition
        .push(Animation::new(1.0, 2.0, 20, Easing::Linear))
        .unwrap();
    composition = composition.with_controls(CompositionControls {
        start_delay_ms: 10,
        repeat_count: Some(2),
        reverse: true,
    });

    let mut player = ComposedAnimationPlayer::<8, 4>::new(composition);
    player.tick(10).unwrap();
    let status0 = player.status();
    assert!(status0.active || !status0.done);
    player.tick(60).unwrap();
    let status1 = player.status();
    assert!(!status1.done);
    for _ in 0..20 {
        player.tick(20).unwrap();
        if player.status().done {
            break;
        }
    }
    assert!(player.status().done);
}

#[test]
fn composed_animation_player_callbacks_pause_and_seek_active_work() {
    static STARTS: AtomicUsize = AtomicUsize::new(0);
    static COMPLETES: AtomicUsize = AtomicUsize::new(0);
    static DONE: AtomicUsize = AtomicUsize::new(0);
    fn on_start(_: u16) {
        STARTS.fetch_add(1, Ordering::Relaxed);
    }
    fn on_complete(_: u16) {
        COMPLETES.fetch_add(1, Ordering::Relaxed);
    }
    fn on_done() {
        DONE.fetch_add(1, Ordering::Relaxed);
    }

    STARTS.store(0, Ordering::Relaxed);
    COMPLETES.store(0, Ordering::Relaxed);
    DONE.store(0, Ordering::Relaxed);
    let mut composition = ComposedAnimation::<3>::new(CompositionMode::Spawn);
    composition
        .push(Animation::new(0.0, 1.0, 40, Easing::Linear))
        .unwrap();
    composition = composition.with_controls(CompositionControls {
        start_delay_ms: 0,
        repeat_count: Some(1),
        reverse: false,
    });
    let mut player = ComposedAnimationPlayer::<8, 3>::new(composition);
    player.set_callbacks(ComposedAnimationCallbacks {
        on_cycle_start: Some(on_start),
        on_cycle_complete: Some(on_complete),
        on_done: Some(on_done),
    });
    player.tick(1).unwrap();
    player.set_paused(true);
    player.tick(100).unwrap();
    player.set_paused(false);
    assert!(player.seek_active_stepped(20, 5));
    for _ in 0..8 {
        player.tick(10).unwrap();
        if player.status().done {
            break;
        }
    }
    assert!(player.status().done);
    assert_eq!(STARTS.load(Ordering::Relaxed), 1);
    assert!(COMPLETES.load(Ordering::Relaxed) >= 1);
    assert_eq!(DONE.load(Ordering::Relaxed), 1);
}

#[test]
fn menu_focus_choreography_preset_applies_motion_bundle() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 120, 60));
    let focused = gui
        .add_panel(Rect::new(20, 20, 40, 12), Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<16, 16>::new();
    presets::menu_focus_choreography(&mut animator, focused, 20, 20).unwrap();
    animator.tick(120, &mut gui).unwrap();
    let rect = gui.absolute_rect(focused).unwrap();
    assert!(rect.x >= 20);
    assert_eq!(rect.y, 20);
}

#[test]
fn dialog_and_neighbor_focus_choreography_presets_work() {
    let mut gui = GuiContext::<16, 16, 16>::new(Rect::new(0, 0, 140, 80));
    let dialog = gui
        .add_panel(Rect::new(40, 30, 40, 18), Style::panel())
        .unwrap();
    let focused = gui
        .add_panel(Rect::new(20, 12, 50, 10), Style::panel())
        .unwrap();
    let n1 = gui
        .add_panel(Rect::new(20, 24, 50, 10), Style::panel())
        .unwrap();
    let n2 = gui
        .add_panel(Rect::new(20, 36, 50, 10), Style::panel())
        .unwrap();
    let mut animator = WidgetAnimator::<24, 24>::new();
    presets::dialog_pop_choreography(&mut animator, dialog, 30).unwrap();
    presets::list_focus_with_neighbors(&mut animator, focused, &[n1, n2], 20, 12).unwrap();
    animator.tick(180, &mut gui).unwrap();
    assert_eq!(gui.absolute_rect(dialog).unwrap().y, 30);
    assert!(gui.absolute_rect(focused).unwrap().x >= 20);
    assert!(gui.absolute_rect(n1).unwrap().x <= 20);
    assert!(gui.absolute_rect(n2).unwrap().x <= 20);
}

#[test]
fn arc_gauge_widget_renders_and_updates() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let gauge = gui
        .add_arc_gauge(
            Rect::new(4, 4, 24, 24),
            0.2,
            0.0,
            1.0,
            135,
            405,
            2,
            true,
            Style::progress(),
        )
        .unwrap();
    gui.set_gauge_value(gauge, 0.8).unwrap();
    let mut target = TestBuffer::new(64, 32);
    gui.render(&mut target).unwrap();
    assert!(target.digest() != 0);
}

#[test]
fn stroke_style_and_text_overflow_render_paths_work() {
    let mut target = TestBuffer::new(32, 16);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 32, 16));
    ctx.set_backend_caps(RenderBackendCaps {
        color_format: ColorFormat::Rgb565,
        supports_layers: true,
        supports_subpixel: true,
    });
    assert!(ctx.backend_caps().supports_subpixel);
    ctx.draw_line_styled(
        0,
        0,
        12,
        0,
        StrokeStyle::new(Rgb565::WHITE)
            .with_width(3)
            .with_antialias_mode(AntiAliasMode::Subpixel),
    )
    .unwrap();
    ctx.stroke_arc_styled(
        16,
        8,
        6,
        0,
        180,
        StrokeStyle::new(Rgb565::GREEN)
            .with_width(2)
            .with_antialias(true),
    )
    .unwrap();
    ctx.draw_text_in(
        Rect::new(0, 0, 16, 6),
        "LONG TEXT HERE",
        TextStyle::new(Rgb565::YELLOW)
            .with_wrap(TextWrap::Word)
            .with_overflow(TextOverflow::Ellipsis)
            .with_max_lines(Some(1))
            .with_ellipsis_mode(EllipsisMode::SingleGlyph)
            .with_kerning(true),
    )
    .unwrap();
    assert!(target.digest() != 0);
}

#[test]
fn text_overflow_policy_wrap_then_ellipsis_is_supported() {
    let mut target = TestBuffer::new(24, 12);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 24, 12));
    ctx.draw_text_in(
        Rect::new(0, 0, 24, 12),
        "ONE TWO THREE FOUR FIVE",
        TextStyle::new(Rgb565::WHITE)
            .with_wrap(TextWrap::Word)
            .with_overflow_policy(TextOverflowPolicy::WrapThenEllipsis { max_lines: 2 })
            .with_ellipsis_mode(EllipsisMode::ThreeDots),
    )
    .unwrap();
    assert!(target.digest() != 0);
}

#[test]
fn render_ctx_transform_and_layer_blend_entrypoints_work() {
    let mut target = TestBuffer::new(24, 24);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 24, 24));
    ctx.push_layer(LayerState {
        opacity: 200,
        blend: BlendMode::Add,
        backdrop: Rgb565::new(4, 8, 4),
    });
    ctx.translate(4.0, 2.0);
    ctx.rotate(0.0);
    ctx.skew(0.0, 0.0);
    ctx.draw_line_styled(0, 0, 6, 0, StrokeStyle::new(Rgb565::RED))
        .unwrap();
    ctx.pop_transform();
    ctx.pop_layer();

    assert!(target.digest() != 0);
}

#[test]
fn physics_and_path_animators_advance() {
    let mut spring = SpringAnimator::new(0.0, 1.0);
    let v0 = spring.value;
    let v1 = spring.tick(16);
    assert!(v1 > v0);

    let mut inertia = InertiaAnimator::new(0.0, 10.0);
    let i1 = inertia.tick(100);
    let i2 = inertia.tick(100);
    assert!(i2 > i1);

    let mut path = PathAnimator::<4>::new(100, Easing::Linear);
    path.push_point(PathPoint::new(0.0, 0.0)).unwrap();
    path.push_point(PathPoint::new(10.0, 0.0)).unwrap();
    path.push_point(PathPoint::new(10.0, 10.0)).unwrap();
    path.tick(50);
    let p = path.value().unwrap();
    assert!(p.x >= 5.0);
}

#[test]
fn style_transition_interpolates_between_states() {
    let styles = WidgetStyle::new(Style::button());
    let mut transition = StyleTransition::new(
        VisualState::Normal,
        VisualState::Focused,
        100,
        Easing::Linear,
    );
    transition.tick(50);
    let blended = transition.style(styles);
    assert!(blended.opacity > 0);

    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 20, 20));
    let panel = gui.add_panel(Rect::new(0, 0, 10, 10), styles).unwrap();
    gui.apply_widget_style_transition(panel, VisualState::Normal, VisualState::Focused, 0.5)
        .unwrap();
}

#[test]
fn new_widgets_chart_spinner_dropdown_render_and_update() {
    static SERIES: [f32; 5] = [0.0, 0.5, 0.2, 0.8, 0.6];
    static ITEMS: [&str; 3] = ["ONE", "TWO", "THREE"];
    let mut gui = GuiContext::<16, 32, 16>::new(Rect::new(0, 0, 96, 48));
    let chart = gui
        .add_chart(Rect::new(0, 0, 32, 16), &SERIES, 0.0, 1.0, Style::panel())
        .unwrap();
    gui.set_chart_style(chart, 2, true, true).unwrap();
    gui.set_chart_decoration(chart, ChartMode::Bars, true, true, true)
        .unwrap();
    let spinner = gui
        .add_spinner(Rect::new(34, 0, 16, 16), 0.0, Style::progress())
        .unwrap();
    let dropdown = gui
        .add_dropdown(Rect::new(52, 0, 40, 16), &ITEMS, 0, Style::button())
        .unwrap();
    gui.tick_spinner(spinner, 16, 1.0).unwrap();
    gui.set_dropdown_selected(dropdown, 2).unwrap();
    assert_eq!(gui.dropdown_selected(dropdown), Some(2));
    gui.set_focus(Some(dropdown)).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.dropdown_open(dropdown), Some(true));
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.dropdown_selected(dropdown), Some(0));
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.dropdown_open(dropdown), Some(false));
    let roller = gui
        .add_roller(Rect::new(0, 18, 30, 20), &ITEMS, 0, Style::button())
        .unwrap();
    gui.set_focus(Some(roller)).unwrap();
    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.roller_selected(roller), Some(1));
    static TABLE_ROWS: [&[&str]; 2] = [&["A", "B"], &["C", "D"]];
    let table = gui
        .add_table(Rect::new(32, 18, 40, 20), &TABLE_ROWS, Style::panel())
        .unwrap();
    gui.set_table_style(table, true, 2, TextAlign::Center)
        .unwrap();
    let gauge = gui
        .add_gauge(Rect::new(74, 18, 22, 22), 0.4, 0.0, 1.0, Style::progress())
        .unwrap();
    gui.set_gauge_ticks(gauge, 5, 2, true).unwrap();

    let mut target = TestBuffer::new(96, 48);
    gui.render(&mut target).unwrap();
    assert!(target.digest() != 0);
}

#[test]
fn clipping_respects_clip_children_flag_and_scroll_marks_subtree_dirty() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 64, 32));
    let parent = gui
        .add_panel(Rect::new(4, 4, 16, 12), Style::panel())
        .unwrap();
    let child = gui
        .add_label(Rect::new(14, 0, 24, 8), "OVERFLOW", Style::label())
        .unwrap();
    gui.add_child(parent, child).unwrap();
    gui.clear_dirty();

    let mut clipped = MockTarget::new(64, 32);
    gui.render(&mut clipped).unwrap();
    let clipped_outside = clipped
        .pixels
        .iter()
        .any(|&(x, y, _)| x > 20 && y >= 4 && y <= 16);

    gui.remove_flag(parent, WidgetFlags::CLIP_CHILDREN).unwrap();
    let mut unclipped = MockTarget::new(64, 32);
    gui.render(&mut unclipped).unwrap();
    let unclipped_outside = unclipped
        .pixels
        .iter()
        .any(|&(x, y, _)| x > 20 && y >= 4 && y <= 16);

    assert!(!clipped_outside);
    assert!(unclipped_outside);

    let scroll = gui
        .add_scroll_view(Rect::new(0, 20, 24, 10), 0, 30, Style::panel())
        .unwrap();
    let nested = gui
        .add_label(Rect::new(1, 1, 16, 6), "SCROLL", Style::label())
        .unwrap();
    gui.add_child(scroll, nested).unwrap();
    gui.clear_dirty();
    gui.set_scroll_offset(scroll, 8).unwrap();
    assert!(!gui.dirty_regions().is_empty());
}

#[test]
fn image_transform_and_mask_paths_render() {
    let mut target = TestBuffer::new(24, 24);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 24, 24));
    let image = ImageRef::new(2, 2, &[0xFFFF, 0xF800, 0x07E0, 0x001F]);
    ctx.draw_image_transformed(Rect::new(4, 4, 8, 8), image, 1.2, 20.0)
        .unwrap();
    fn checker(x: i32, y: i32) -> bool {
        ((x + y) & 1) == 0
    }
    ctx.fill_rect_masked(Rect::new(0, 0, 8, 8), Rgb565::BLUE, checker)
        .unwrap();
    assert!(target.digest() != 0);
}

#[cfg(all(feature = "std", feature = "image-decode"))]
#[test]
fn ppm_decoder_and_sprite_atlas_helpers_work() {
    let mut pixels = heapless::Vec::<u16, 16>::new();
    let decoder = BasicImageDecoder;
    let (w, h) = decode_image_with(
        &decoder,
        EncodedImageFormat::PpmAscii,
        "P3 2 1 255 255 0 0 0 255 0",
        &mut pixels,
    )
    .unwrap();
    assert_eq!((w, h), (2, 1));
    let auto = decode_image_auto(
        "P3 1 1 255 255 255 255",
        &mut heapless::Vec::<u16, 4>::new(),
    )
    .unwrap();
    assert_eq!(auto, (1, 1));
    let image = ImageRef::new(w, h, pixels.as_slice());
    let sheet = SpriteSheet::new(image, 1, 1);
    assert_eq!(sheet.sprite_rect(1), Rect::new(1, 0, 1, 1));
    let entries = [ImageAtlasEntry {
        id: 7,
        rect: Rect::new(0, 0, 1, 1),
    }];
    let atlas = ImageAtlas::new(image, &entries);
    assert_eq!(atlas.rect_for(7), Some(Rect::new(0, 0, 1, 1)));
}

#[cfg(feature = "std")]
#[test]
fn layer_canvas_can_composite_into_target() {
    let mut base = TestBuffer::new(16, 16);
    let mut layer = LayerCanvas::new(16, 16);
    {
        let mut ctx = RenderCtx::new(layer.target_mut(), Rect::new(0, 0, 16, 16));
        ctx.fill_rect(Rect::new(2, 2, 8, 8), Rgb565::GREEN).unwrap();
    }
    layer.composite_into(&mut base, BlendMode::Normal, 220);
    assert!(base.digest() != 0);
}

#[test]
fn subpixel_antialias_falls_back_without_backend_support() {
    let mut target = TestBuffer::new(16, 8);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 16, 8));
    ctx.set_quality(RenderQuality::Medium);
    ctx.set_backend_caps(RenderBackendCaps {
        color_format: ColorFormat::Rgb565,
        supports_layers: true,
        supports_subpixel: false,
    });
    ctx.draw_line_styled(
        0,
        1,
        12,
        1,
        StrokeStyle::new(Rgb565::WHITE)
            .with_width(2)
            .with_antialias_mode(AntiAliasMode::Subpixel),
    )
    .unwrap();
    assert!(target.digest() != 0);
}

#[test]
fn textarea_keyboard_and_text_shaper_hooks_work() {
    static KEYS: [char; 6] = ['a', 'b', 'c', 'd', 'e', 'f'];
    static ALT_KEYS: [char; 6] = ['1', '2', '3', '4', '5', '6'];
    let mut gui = GuiContext::<16, 16, 16>::new(Rect::new(0, 0, 96, 48));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 50, 14), "HELLO", "TYPE", Style::panel())
        .unwrap();
    let keyboard = gui
        .add_keyboard_with_alt(
            Rect::new(0, 16, 50, 20),
            &KEYS,
            Some(&ALT_KEYS),
            3,
            Some(textarea),
            Style::button(),
        )
        .unwrap();
    gui.set_focus(Some(textarea)).unwrap();
    gui.move_textarea_cursor(textarea, -1).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(4));
    gui.set_textarea_text(textarea, "HI").unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("HI"));
    gui.set_focus(Some(keyboard)).unwrap();
    gui.set_keyboard_layout(keyboard, KeyboardLayout::Shift)
        .unwrap();
    assert_eq!(gui.keyboard_layout(keyboard), Some(KeyboardLayout::Shift));
    assert_eq!(gui.keyboard_selected_key(keyboard), Some('A'));
    gui.set_keyboard_layout(keyboard, KeyboardLayout::Symbols)
        .unwrap();
    assert_eq!(gui.keyboard_selected_key(keyboard), Some('1'));
    gui.handle_input(InputEvent::Down).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.keyboard_selected_key(keyboard), Some('2'));
    let mut saw_text_input = false;
    let mut saw_target_value_change = false;
    while let Some(event) = gui.pop_event() {
        if matches!(event, UiEvent::TextInput { id, ch } if id == textarea && ch == '2') {
            saw_text_input = true;
        }
        if event == UiEvent::ValueChanged(textarea) {
            saw_target_value_change = true;
        }
    }
    assert!(saw_text_input);
    assert!(saw_target_value_change);

    let shaper = BasicTextShaper;
    let mut shaped = heapless::Vec::<ShapedGlyph, 8>::new();
    shaper.shape("AB", ShapingConfig::default(), &mut shaped);
    assert_eq!(shaped.len(), 2);

    let mut target = TestBuffer::new(24, 8);
    let mut ctx = RenderCtx::new(&mut target, Rect::new(0, 0, 24, 8));
    ctx.draw_text_shaped_in::<_, 8>(
        Rect::new(0, 0, 24, 8),
        "ab",
        TextStyle::new(Rgb565::WHITE),
        &shaper,
        ShapingConfig::default(),
    )
    .unwrap();
    assert!(target.digest() != 0);
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

    assert_eq!(digest, 15_293_939_628_664_047_529);
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
fn double_select_emits_double_clicked_within_window() {
    let mut gui = GuiContext::<4, 24, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_double_select_window_ms(40);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Select).unwrap();
    while let Some(event) = gui.pop_event() {
        if event == UiEvent::DoubleClicked(button) {
            panic!("unexpected double click on first select");
        }
    }

    gui.tick_input(20).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();
    let mut saw_double = false;
    while let Some(event) = gui.pop_event() {
        if event == UiEvent::DoubleClicked(button) {
            saw_double = true;
        }
    }
    assert!(saw_double);
}

#[test]
fn double_select_timeout_prevents_double_clicked_event() {
    let mut gui = GuiContext::<4, 24, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_double_select_window_ms(20);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Select).unwrap();
    while gui.pop_event().is_some() {}
    gui.tick_input(25).unwrap();
    gui.handle_input(InputEvent::Select).unwrap();

    while let Some(event) = gui.pop_event() {
        assert_ne!(event, UiEvent::DoubleClicked(button));
    }
}

#[test]
fn raw_select_policy_uses_press_release_then_activation() {
    let mut gui = GuiContext::<4, 24, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_widget_key_input_policy(
        button,
        WidgetKeyInputPolicy {
            raw_select: true,
            raw_back: false,
        },
    )
    .unwrap();
    gui.set_focus(Some(button)).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::SelectPressed).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::Pressed(button)));

    gui.handle_input(InputEvent::SelectReleased).unwrap();
    let mut saw_released = false;
    let mut saw_pressed = false;
    let mut saw_clicked = false;
    let mut saw_activate = false;
    while let Some(event) = gui.pop_event() {
        if event == UiEvent::Released(button) {
            saw_released = true;
        } else if event == UiEvent::Pressed(button) {
            saw_pressed = true;
        } else if event == UiEvent::Clicked(button) {
            saw_clicked = true;
        } else if event == UiEvent::Activate(button) {
            saw_activate = true;
        }
    }
    assert!(saw_released && saw_pressed && saw_clicked && saw_activate);
}

#[test]
fn raw_back_policy_emits_press_release_and_runs_back_action() {
    static ITEMS: [&str; 3] = ["ONE", "TWO", "THREE"];
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 64, 32));
    let dropdown = gui
        .add_dropdown(Rect::new(0, 0, 40, 12), &ITEMS, 0, Style::button())
        .unwrap();
    gui.set_widget_key_input_policy(
        dropdown,
        WidgetKeyInputPolicy {
            raw_select: false,
            raw_back: true,
        },
    )
    .unwrap();
    gui.set_focus(Some(dropdown)).unwrap();
    gui.set_dropdown_open(dropdown, true).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::BackPressed).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::Pressed(dropdown)));

    gui.handle_input(InputEvent::BackReleased).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::Released(dropdown)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Closed(dropdown)));
    assert_eq!(gui.dropdown_open(dropdown), Some(false));
}

#[test]
fn widget_key_binding_can_ignore_select_activation() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_widget_key_bindings(
        button,
        WidgetKeyBindings {
            select: KeyBindingAction::Ignore,
            back: KeyBindingAction::Default,
        },
    )
    .unwrap();
    gui.set_focus(Some(button)).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Select).unwrap();
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn widget_key_binding_can_remap_back_to_activate() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_widget_key_bindings(
        button,
        WidgetKeyBindings {
            select: KeyBindingAction::Default,
            back: KeyBindingAction::Activate,
        },
    )
    .unwrap();
    gui.set_focus(Some(button)).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Back).unwrap();
    let mut saw_activate = false;
    while let Some(event) = gui.pop_event() {
        if event == UiEvent::Activate(button) {
            saw_activate = true;
        }
    }
    assert!(saw_activate);
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
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
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
fn pointer_double_click_emits_double_clicked_within_window() {
    let mut gui = GuiContext::<4, 24, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_double_pointer_window_ms(40);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();
    while let Some(event) = gui.pop_event() {
        assert_ne!(event, UiEvent::DoubleClicked(button));
    }

    gui.tick_input(20).unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();
    let mut saw_double = false;
    while let Some(event) = gui.pop_event() {
        if event == UiEvent::DoubleClicked(button) {
            saw_double = true;
        }
    }
    assert!(saw_double);
}

#[test]
fn pointer_double_click_timeout_prevents_double_clicked_event() {
    let mut gui = GuiContext::<4, 24, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_double_pointer_window_ms(20);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}
    gui.tick_input(25).unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();

    while let Some(event) = gui.pop_event() {
        assert_ne!(event, UiEvent::DoubleClicked(button));
    }
}

#[test]
fn pointer_release_reports_pressed_widget_even_when_pointer_leaves_hit_rect() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 20, 10), "ONE", Style::button())
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 40,
        y: 20,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();

    assert_eq!(gui.pop_event(), Some(UiEvent::Released(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::PointerReleased(button)));
}

#[test]
fn pointer_long_press_emits_once_after_threshold() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_long_press_threshold_ms(20);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}

    gui.tick_input(19).unwrap();
    assert_eq!(gui.pop_event(), None);

    gui.tick_input(1).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::LongPressed(button)));
    assert_eq!(gui.pop_event(), None);

    gui.tick_input(50).unwrap();
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn pointer_release_cancels_pending_long_press() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let _button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_long_press_threshold_ms(30);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}

    gui.tick_input(10).unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}

    gui.tick_input(40).unwrap();
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn dropdown_emits_open_close_events() {
    static ITEMS: [&str; 3] = ["ONE", "TWO", "THREE"];
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 64, 32));
    let dropdown = gui
        .add_dropdown(Rect::new(0, 0, 40, 12), &ITEMS, 0, Style::button())
        .unwrap();
    gui.set_focus(Some(dropdown)).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Select).unwrap();
    let mut saw_opened = false;
    while let Some(event) = gui.pop_event() {
        if event == UiEvent::Opened(dropdown) {
            saw_opened = true;
            break;
        }
    }
    assert!(saw_opened);
    assert_eq!(gui.dropdown_open(dropdown), Some(true));

    gui.handle_input(InputEvent::Select).unwrap();
    let mut saw_closed = false;
    while let Some(event) = gui.pop_event() {
        if event == UiEvent::Closed(dropdown) {
            saw_closed = true;
            break;
        }
    }
    assert!(saw_closed);
    assert_eq!(gui.dropdown_open(dropdown), Some(false));
}

#[test]
fn back_input_closes_focused_open_dropdown_before_global_back() {
    static ITEMS: [&str; 3] = ["ONE", "TWO", "THREE"];
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 64, 32));
    let dropdown = gui
        .add_dropdown(Rect::new(0, 0, 40, 12), &ITEMS, 0, Style::button())
        .unwrap();
    gui.set_focus(Some(dropdown)).unwrap();
    gui.set_dropdown_open(dropdown, true).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Back).unwrap();

    assert_eq!(gui.dropdown_open(dropdown), Some(false));
    assert_eq!(gui.pop_event(), Some(UiEvent::Closed(dropdown)));
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn dropdown_navigation_wraps_with_up_down_when_open() {
    static ITEMS: [&str; 3] = ["ONE", "TWO", "THREE"];
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 64, 32));
    let dropdown = gui
        .add_dropdown(Rect::new(0, 0, 40, 12), &ITEMS, 0, Style::button())
        .unwrap();
    gui.set_focus(Some(dropdown)).unwrap();
    gui.set_dropdown_open(dropdown, true).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Up).unwrap();
    assert_eq!(gui.dropdown_selected(dropdown), Some(2));
    assert_eq!(gui.pop_event(), Some(UiEvent::ValueChanged(dropdown)));

    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.dropdown_selected(dropdown), Some(0));
    assert_eq!(gui.pop_event(), Some(UiEvent::ValueChanged(dropdown)));
}

#[test]
fn list_navigation_updates_selection_and_offset() {
    static ITEMS: [&str; 5] = ["A", "B", "C", "D", "E"];
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 80, 40));
    let list = gui
        .add_list(Rect::new(0, 0, 40, 20), &ITEMS, 0, 2, Style::panel())
        .unwrap();
    gui.set_focus(Some(list)).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Down).unwrap();
    gui.handle_input(InputEvent::Down).unwrap();
    gui.handle_input(InputEvent::Down).unwrap();

    assert_eq!(gui.list_selected(list), Some(3));
    let node = gui.widgets().iter().find(|w| w.id == list).unwrap();
    match node.kind {
        WidgetKind::List { offset, .. } => assert_eq!(offset, 2),
        _ => panic!("expected list widget"),
    }
}

#[test]
fn tabs_left_right_and_encoder_wrap_selection() {
    static TABS: [&str; 3] = ["A", "B", "C"];
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 80, 32));
    let tabs = gui
        .add_tabs(Rect::new(0, 0, 60, 12), &TABS, 0, Style::button())
        .unwrap();
    gui.set_focus(Some(tabs)).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Left).unwrap();
    assert_eq!(gui.tab_selected(tabs), Some(2));
    assert_eq!(gui.pop_event(), Some(UiEvent::ValueChanged(tabs)));

    gui.handle_input(InputEvent::Right).unwrap();
    assert_eq!(gui.tab_selected(tabs), Some(0));
    assert_eq!(gui.pop_event(), Some(UiEvent::ValueChanged(tabs)));
}

#[test]
fn roller_navigation_wraps_selection() {
    static ITEMS: [&str; 3] = ["ONE", "TWO", "THREE"];
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 80, 32));
    let roller = gui
        .add_roller(Rect::new(0, 0, 40, 18), &ITEMS, 0, Style::button())
        .unwrap();
    gui.set_focus(Some(roller)).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Up).unwrap();
    assert_eq!(gui.roller_selected(roller), Some(2));
    assert_eq!(gui.pop_event(), Some(UiEvent::ValueChanged(roller)));

    gui.handle_input(InputEvent::Down).unwrap();
    assert_eq!(gui.roller_selected(roller), Some(0));
    assert_eq!(gui.pop_event(), Some(UiEvent::ValueChanged(roller)));
}

#[test]
fn textarea_edit_hooks_emit_text_input_events() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 64, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 50, 14), "HELLO", "TYPE", Style::panel())
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.textarea_insert_char(textarea, 'x').unwrap();
    gui.textarea_backspace(textarea).unwrap();
    gui.textarea_delete_forward(textarea).unwrap();

    assert_eq!(
        gui.pop_event(),
        Some(UiEvent::TextInput {
            id: textarea,
            ch: 'x'
        })
    );
    assert_eq!(gui.pop_event(), Some(UiEvent::ValueChanged(textarea)));
    assert_eq!(
        gui.pop_event(),
        Some(UiEvent::TextInput {
            id: textarea,
            ch: '\u{8}'
        })
    );
    assert_eq!(gui.pop_event(), Some(UiEvent::ValueChanged(textarea)));
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn textarea_edit_operations_mutate_internal_text() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 64, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 50, 14), "HELLO", "TYPE", Style::panel())
        .unwrap();

    gui.set_textarea_cursor(textarea, 5).unwrap();
    gui.textarea_insert_char(textarea, '!').unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("HELLO!"));

    gui.textarea_backspace(textarea).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("HELLO"));

    gui.set_textarea_cursor(textarea, 0).unwrap();
    gui.textarea_delete_forward(textarea).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ELLO"));
}

#[test]
fn textarea_cursor_word_navigation_helpers_work() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(
            Rect::new(0, 0, 80, 14),
            "ONE  TWO THREE",
            "TYPE",
            Style::panel(),
        )
        .unwrap();

    gui.set_textarea_cursor_home(textarea).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(0));

    gui.move_textarea_cursor_word(textarea, 1).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(5));

    gui.move_textarea_cursor_word(textarea, 1).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(9));

    gui.move_textarea_cursor_word(textarea, -1).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(5));

    gui.set_textarea_cursor_end(textarea).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(14));
}

#[test]
fn textarea_selection_and_cursor_blink_state_work() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(
            Rect::new(0, 0, 80, 14),
            "HELLO WORLD",
            "TYPE",
            Style::panel(),
        )
        .unwrap();
    gui.set_focus(Some(textarea)).unwrap();

    gui.set_textarea_selection(textarea, 8, 2).unwrap();
    assert_eq!(gui.textarea_selection(textarea), Some((2, 8)));
    gui.clear_textarea_selection(textarea).unwrap();
    assert_eq!(gui.textarea_selection(textarea), None);

    gui.set_textarea_cursor_blink_timing(10);
    assert_eq!(gui.textarea_cursor_visible(textarea), Some(true));
    gui.tick_input(10).unwrap();
    assert_eq!(gui.textarea_cursor_visible(textarea), Some(false));
    gui.tick_input(10).unwrap();
    assert_eq!(gui.textarea_cursor_visible(textarea), Some(true));
}

#[test]
fn textarea_insert_replaces_selected_range() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(
            Rect::new(0, 0, 80, 14),
            "HELLO WORLD",
            "TYPE",
            Style::panel(),
        )
        .unwrap();

    gui.set_textarea_selection(textarea, 6, 11).unwrap();
    gui.set_textarea_cursor(textarea, 11).unwrap();
    gui.textarea_insert_char(textarea, '!').unwrap();

    assert_eq!(gui.textarea_text(textarea), Some("HELLO !"));
    assert_eq!(gui.textarea_selection(textarea), None);
}

#[test]
fn textarea_line_home_end_follow_wrapped_rows() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 18, 14), "ABCDEFGH", "TYPE", Style::panel())
        .unwrap();

    gui.set_textarea_cursor(textarea, 6).unwrap();
    gui.set_textarea_cursor_line_home(textarea).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(4));

    gui.set_textarea_cursor(textarea, 5).unwrap();
    gui.set_textarea_cursor_line_end(textarea).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(8));
}

#[test]
fn textarea_navigation_input_events_drive_editor_cursor() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(
            Rect::new(0, 0, 18, 14),
            "ONE TWO THREE",
            "TYPE",
            Style::panel(),
        )
        .unwrap();
    gui.set_focus(Some(textarea)).unwrap();
    gui.set_textarea_cursor(textarea, 11).unwrap();

    gui.handle_input(InputEvent::WordLeft).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(8));

    gui.handle_input(InputEvent::WordRight).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(13));

    gui.set_textarea_cursor(textarea, 6).unwrap();
    gui.handle_input(InputEvent::Home).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(4));

    gui.handle_input(InputEvent::End).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(8));
}

#[test]
fn textarea_selection_navigation_events_expand_and_clear_selection() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(
            Rect::new(0, 0, 24, 14),
            "ONE TWO THREE",
            "TYPE",
            Style::panel(),
        )
        .unwrap();
    gui.set_focus(Some(textarea)).unwrap();
    gui.set_textarea_cursor(textarea, 4).unwrap();

    gui.handle_input(InputEvent::SelectWordRight).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(8));
    assert_eq!(gui.textarea_selection(textarea), Some((4, 8)));

    gui.handle_input(InputEvent::SelectEnd).unwrap();
    assert_eq!(gui.textarea_selection(textarea), Some((4, 10)));

    gui.handle_input(InputEvent::WordLeft).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(8));
    assert_eq!(gui.textarea_selection(textarea), None);
}

#[test]
fn textarea_undo_redo_restores_mutation_history() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 80, 14), "AB", "TYPE", Style::panel())
        .unwrap();
    gui.set_focus(Some(textarea)).unwrap();
    gui.set_textarea_cursor_end(textarea).unwrap();

    gui.textarea_insert_char(textarea, 'C').unwrap();
    gui.textarea_insert_char(textarea, 'D').unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ABCD"));

    gui.handle_input(InputEvent::Undo).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ABC"));
    gui.handle_input(InputEvent::Undo).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("AB"));

    gui.handle_input(InputEvent::Redo).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ABC"));
    gui.handle_input(InputEvent::Redo).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ABCD"));
}

#[test]
fn textarea_undo_redo_tracks_selection_replace_boundaries() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(
            Rect::new(0, 0, 80, 14),
            "HELLO WORLD",
            "TYPE",
            Style::panel(),
        )
        .unwrap();
    gui.set_focus(Some(textarea)).unwrap();
    gui.set_textarea_selection(textarea, 6, 11).unwrap();
    gui.set_textarea_cursor(textarea, 11).unwrap();

    gui.textarea_insert_char(textarea, '!').unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("HELLO !"));

    gui.handle_input(InputEvent::Undo).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("HELLO WORLD"));

    gui.handle_input(InputEvent::Redo).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("HELLO !"));
}

#[test]
fn textarea_wrapped_selection_replace_can_be_undone() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 18, 14), "ABCDEFGH", "TYPE", Style::panel())
        .unwrap();
    gui.set_focus(Some(textarea)).unwrap();
    gui.set_textarea_cursor(textarea, 6).unwrap();
    gui.handle_input(InputEvent::SelectHome).unwrap();
    assert_eq!(gui.textarea_selection(textarea), Some((4, 6)));

    gui.textarea_insert_char(textarea, 'Z').unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ABCDZGH"));

    gui.handle_input(InputEvent::Undo).unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ABCDEFGH"));
}

#[test]
fn textarea_read_only_blocks_mutation_ops() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 80, 14), "LOCKED", "TYPE", Style::panel())
        .unwrap();
    gui.set_textarea_capabilities(textarea, true, false, true)
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.textarea_insert_char(textarea, '!').unwrap();
    gui.textarea_backspace(textarea).unwrap();
    gui.textarea_delete_forward(textarea).unwrap();

    assert_eq!(gui.textarea_text(textarea), Some("LOCKED"));
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn textarea_single_line_rejects_newline_insert() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 80, 14), "ONE", "TYPE", Style::panel())
        .unwrap();
    gui.set_textarea_capabilities(textarea, false, true, true)
        .unwrap();
    gui.set_textarea_cursor_end(textarea).unwrap();
    while gui.pop_event().is_some() {}

    gui.textarea_insert_char(textarea, '\n').unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ONE"));
    assert_eq!(gui.pop_event(), None);

    gui.textarea_insert_char(textarea, 'X').unwrap();
    assert_eq!(gui.textarea_text(textarea), Some("ONEX"));
}

#[test]
fn textarea_noop_backspace_does_not_emit_events() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 80, 14), "A", "TYPE", Style::panel())
        .unwrap();
    gui.set_textarea_cursor(textarea, 0).unwrap();
    while gui.pop_event().is_some() {}

    gui.textarea_backspace(textarea).unwrap();

    assert_eq!(gui.textarea_text(textarea), Some("A"));
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn textarea_noop_delete_forward_does_not_emit_events() {
    let mut gui = GuiContext::<8, 32, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 80, 14), "A", "TYPE", Style::panel())
        .unwrap();
    gui.set_textarea_cursor(textarea, 1).unwrap();
    while gui.pop_event().is_some() {}

    gui.textarea_delete_forward(textarea).unwrap();

    assert_eq!(gui.textarea_text(textarea), Some("A"));
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn textarea_select_home_and_end_follow_wrapped_line_boundaries() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 96, 32));
    let textarea = gui
        .add_textarea(Rect::new(0, 0, 18, 14), "ABCDEFGH", "TYPE", Style::panel())
        .unwrap();
    gui.set_focus(Some(textarea)).unwrap();
    gui.set_textarea_cursor(textarea, 6).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::SelectHome).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(4));
    assert_eq!(gui.textarea_selection(textarea), Some((4, 6)));

    gui.handle_input(InputEvent::SelectEnd).unwrap();
    assert_eq!(gui.textarea_cursor(textarea), Some(8));
    assert_eq!(gui.textarea_selection(textarea), Some((6, 8)));
}

#[test]
fn text_alignment_accounts_for_kerning_width() {
    let mut without_kerning = TestBuffer::new(20, 8);
    let mut with_kerning = TestBuffer::new(20, 8);
    let rect = Rect::new(0, 0, 20, 8);

    {
        let mut ctx = RenderCtx::new(&mut without_kerning, rect);
        ctx.draw_text_in(
            rect,
            "AV",
            TextStyle::new(Rgb565::WHITE)
                .with_font(FontId::Tiny3x5)
                .with_align(TextAlign::Right),
        )
        .unwrap();
    }

    {
        let mut ctx = RenderCtx::new(&mut with_kerning, rect);
        ctx.draw_text_in(
            rect,
            "AV",
            TextStyle::new(Rgb565::WHITE)
                .with_font(FontId::Tiny3x5)
                .with_align(TextAlign::Right)
                .with_kerning(true),
        )
        .unwrap();
    }

    let left_no_kerning = (0..20)
        .find(|&x| (0..8).any(|y| without_kerning.pixel_at(x, y) == Some(Rgb565::WHITE)))
        .unwrap_or(20);
    let left_with_kerning = (0..20)
        .find(|&x| (0..8).any(|y| with_kerning.pixel_at(x, y) == Some(Rgb565::WHITE)))
        .unwrap_or(20);

    assert!(left_with_kerning > left_no_kerning);
}

#[test]
fn pointer_move_emits_gesture_once_after_threshold() {
    let mut gui = GuiContext::<4, 16, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 4,
        y: 4,
        state: PointerState::Moved,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert_eq!(gui.pop_event(), None);

    gui.handle_input(InputEvent::Pointer {
        x: 10,
        y: 4,
        state: PointerState::Moved,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::Gesture(button)));
    assert_eq!(gui.pop_event(), None);

    gui.handle_input(InputEvent::Pointer {
        x: 14,
        y: 4,
        state: PointerState::Moved,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert_eq!(gui.pop_event(), None);
}

#[test]
fn scrollable_drag_emits_scroll_event() {
    let mut gui = GuiContext::<8, 24, 8>::new(Rect::new(0, 0, 64, 32));
    let scroll = gui
        .add_scroll_view(Rect::new(0, 0, 30, 20), 0, 80, Style::panel())
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 8,
        state: PointerState::Moved,
        button: PointerButton::Primary,
    })
    .unwrap();

    let mut saw_scroll = false;
    while let Some(event) = gui.pop_event() {
        if matches!(event, UiEvent::Scroll { id, delta } if id == scroll && delta != 0) {
            saw_scroll = true;
            break;
        }
    }
    assert!(saw_scroll);
}

#[test]
fn long_press_repeat_emits_repeated_activate_for_button() {
    let mut gui = GuiContext::<4, 32, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_long_press_threshold_ms(10);
    gui.set_press_repeat_timing(20, 10);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}

    gui.tick_input(10).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::LongPressed(button)));
    assert_eq!(gui.pop_event(), None);

    gui.tick_input(20).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::Clicked(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Activate(button)));
}

#[test]
fn widget_press_timing_override_controls_long_press_and_repeat() {
    let mut gui = GuiContext::<4, 32, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_long_press_threshold_ms(1000);
    gui.set_press_repeat_timing(1000, 1000);
    gui.set_widget_press_timing(button, PressTiming::new(10, 20, 10))
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}

    gui.tick_input(10).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::LongPressed(button)));
    assert_eq!(gui.pop_event(), None);

    gui.tick_input(20).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::Clicked(button)));
    assert_eq!(gui.pop_event(), Some(UiEvent::Activate(button)));
}

#[test]
fn clearing_widget_press_timing_reverts_to_global_behavior() {
    let mut gui = GuiContext::<4, 32, 4>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", Style::button())
        .unwrap();
    gui.set_long_press_threshold_ms(40);
    gui.set_widget_press_timing(button, PressTiming::new(10, 20, 10))
        .unwrap();
    gui.clear_widget_press_timing(button).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    while gui.pop_event().is_some() {}
    gui.tick_input(10).unwrap();
    assert_eq!(gui.pop_event(), None);
    gui.tick_input(30).unwrap();
    assert_eq!(gui.pop_event(), Some(UiEvent::LongPressed(button)));
}

#[test]
fn drag_release_continues_scroll_with_inertia() {
    let mut gui = GuiContext::<8, 48, 8>::new(Rect::new(0, 0, 64, 32));
    let scroll = gui
        .add_scroll_view(Rect::new(0, 0, 30, 20), 0, 120, Style::panel())
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 14,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Moved,
        button: PointerButton::Primary,
    })
    .unwrap();
    let before_release = gui.scroll_offset(scroll).unwrap_or(0);
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.tick_input(16).unwrap();
    let after_tick = gui.scroll_offset(scroll).unwrap_or(0);
    assert_ne!(after_tick, before_release);
}

#[test]
fn scroll_physics_threshold_controls_inertia_start() {
    let mut gui = GuiContext::<8, 48, 8>::new(Rect::new(0, 0, 64, 32));
    let scroll = gui
        .add_scroll_view(Rect::new(0, 0, 30, 20), 0, 120, Style::panel())
        .unwrap();
    gui.set_scroll_physics(100.0, 0.86, 0.4);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 14,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Moved,
        button: PointerButton::Primary,
    })
    .unwrap();
    let before_release = gui.scroll_offset(scroll).unwrap_or(0);
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();
    gui.tick_input(16).unwrap();
    let after_tick = gui.scroll_offset(scroll).unwrap_or(0);
    assert_eq!(after_tick, before_release);
}

#[test]
fn scroll_physics_decay_controls_inertia_persistence() {
    let mut gui_fast_decay = GuiContext::<8, 48, 8>::new(Rect::new(0, 0, 64, 32));
    let mut gui_slow_decay = GuiContext::<8, 48, 8>::new(Rect::new(0, 0, 64, 32));
    let fast = gui_fast_decay
        .add_scroll_view(Rect::new(0, 0, 30, 20), 0, 120, Style::panel())
        .unwrap();
    let slow = gui_slow_decay
        .add_scroll_view(Rect::new(0, 0, 30, 20), 0, 120, Style::panel())
        .unwrap();
    gui_fast_decay.set_scroll_physics(0.01, 0.15, 0.5);
    gui_slow_decay.set_scroll_physics(0.01, 0.98, 0.5);

    for (gui, id) in [(&mut gui_fast_decay, fast), (&mut gui_slow_decay, slow)] {
        gui.handle_input(InputEvent::Pointer {
            x: 2,
            y: 14,
            state: PointerState::Pressed,
            button: PointerButton::Primary,
        })
        .unwrap();
        gui.handle_input(InputEvent::Pointer {
            x: 2,
            y: 2,
            state: PointerState::Moved,
            button: PointerButton::Primary,
        })
        .unwrap();
        gui.handle_input(InputEvent::Pointer {
            x: 2,
            y: 2,
            state: PointerState::Released,
            button: PointerButton::Primary,
        })
        .unwrap();
        let _ = id;
    }

    for _ in 0..5 {
        gui_fast_decay.tick_input(16).unwrap();
        gui_slow_decay.tick_input(16).unwrap();
    }
    let fast_offset = gui_fast_decay.scroll_offset(fast).unwrap_or(0);
    let slow_offset = gui_slow_decay.scroll_offset(slow).unwrap_or(0);
    assert!(slow_offset.abs() > fast_offset.abs());
}

#[test]
fn focus_state_transitions_progress_and_complete() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let first = gui
        .add_button(Rect::new(0, 0, 20, 10), "A", Style::button())
        .unwrap();
    let second = gui
        .add_button(Rect::new(0, 12, 20, 10), "B", Style::button())
        .unwrap();
    gui.set_state_transition_duration_ms(30);
    gui.set_focus(Some(first)).unwrap();
    while gui.pop_event().is_some() {}

    gui.set_focus(Some(second)).unwrap();
    assert!(gui.active_state_transitions() >= 1);
    gui.tick_input(10).unwrap();
    assert!(gui.active_state_transitions() >= 1);
    gui.tick_input(40).unwrap();
    assert_eq!(gui.active_state_transitions(), 0);
}

#[test]
fn pressed_state_uses_pressed_style_while_pointer_is_held() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let pressed_bg = Rgb565::new(31, 0, 0);
    let normal_bg = Rgb565::new(0, 0, 4);
    let normal = Style {
        background: Some(normal_bg),
        gradient: None,
        font: FontId::Tiny3x5,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::CYAN,
        opacity: 255,
        corner_radius: 0,
        shadow: None,
        border: Border::none(),
        padding: EdgeInsets::all(0),
    };
    let styles = WidgetStyle::new(normal).with_pressed(Style {
        background: Some(pressed_bg),
        ..normal
    });
    let _button = gui
        .add_button(Rect::new(0, 0, 30, 10), "ONE", styles)
        .unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();

    let mut target = MockTarget::new(64, 32);
    gui.render(&mut target).unwrap();
    assert!(target.pixels.iter().any(|&(_, _, c)| c == pressed_bg));
}

#[test]
fn pressed_state_transitions_progress_and_complete_on_release() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let _button = gui
        .add_button(Rect::new(0, 0, 30, 10), "A", Style::button())
        .unwrap();
    gui.set_state_transition_duration_ms(30);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert!(gui.active_state_transitions() >= 1);

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert!(gui.active_state_transitions() >= 1);

    gui.tick_input(10).unwrap();
    assert!(gui.active_state_transitions() >= 1);
    gui.tick_input(40).unwrap();
    assert_eq!(gui.active_state_transitions(), 0);
}

#[test]
fn select_activation_runs_pressed_feedback_transition_cycle() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 30, 10), "A", Style::button())
        .unwrap();
    gui.set_state_transition_duration_ms(20);
    gui.set_focus(Some(button)).unwrap();
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Select).unwrap();
    assert!(gui.active_state_transitions() >= 1);

    gui.tick_input(20).unwrap();
    assert!(gui.active_state_transitions() >= 1);

    gui.tick_input(25).unwrap();
    assert_eq!(gui.active_state_transitions(), 0);
}

#[test]
fn zero_duration_disables_state_transition_queue() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let first = gui
        .add_button(Rect::new(0, 0, 20, 10), "A", Style::button())
        .unwrap();
    let second = gui
        .add_button(Rect::new(0, 12, 20, 10), "B", Style::button())
        .unwrap();
    gui.set_state_transition_duration_ms(0);
    gui.set_focus(Some(first)).unwrap();
    gui.set_focus(Some(second)).unwrap();
    assert_eq!(gui.active_state_transitions(), 0);
}

#[test]
fn disabling_widget_starts_transition_toward_disabled_state() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let first = gui
        .add_button(Rect::new(0, 0, 20, 10), "A", Style::button())
        .unwrap();
    let _second = gui
        .add_button(Rect::new(0, 12, 20, 10), "B", Style::button())
        .unwrap();
    gui.set_state_transition_duration_ms(20);
    gui.set_focus(Some(first)).unwrap();
    while gui.pop_event().is_some() {}

    gui.set_disabled(first, true).unwrap();
    assert!(gui.active_state_transitions() >= 1);

    gui.tick_input(25).unwrap();
    assert_eq!(gui.active_state_transitions(), 0);
}

#[test]
fn enabling_widget_starts_transition_back_to_resting_state() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 20, 10), "A", Style::button())
        .unwrap();
    gui.set_state_transition_duration_ms(20);
    gui.set_disabled(button, true).unwrap();
    gui.tick_input(25).unwrap();
    while gui.pop_event().is_some() {}

    gui.set_disabled(button, false).unwrap();
    assert!(gui.active_state_transitions() >= 1);

    gui.tick_input(25).unwrap();
    assert_eq!(gui.active_state_transitions(), 0);
}

#[test]
fn disabling_pressed_widget_clears_pressed_state_immediately() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 24, 10), "A", Style::button())
        .unwrap();
    gui.set_state_transition_duration_ms(20);
    while gui.pop_event().is_some() {}

    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Pressed,
        button: PointerButton::Primary,
    })
    .unwrap();
    assert!(gui.active_state_transitions() >= 1);
    while gui.pop_event().is_some() {}

    gui.set_disabled(button, true).unwrap();
    gui.handle_input(InputEvent::Pointer {
        x: 2,
        y: 2,
        state: PointerState::Released,
        button: PointerButton::Primary,
    })
    .unwrap();

    assert_eq!(gui.pop_event(), None);
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
fn class_state_overrides_apply_to_transition_endpoints() {
    let mut gui = GuiContext::<8, 8, 8>::new(Rect::new(0, 0, 40, 20));
    let base = Style {
        background: Some(Rgb565::BLUE),
        gradient: None,
        font: FontId::Tiny3x5,
        foreground: Rgb565::WHITE,
        text: Rgb565::WHITE,
        accent: Rgb565::WHITE,
        opacity: 255,
        corner_radius: 0,
        shadow: None,
        border: Border::none(),
        padding: EdgeInsets::all(0),
    };
    let button = gui
        .add_button(Rect::new(2, 2, 24, 10), "", WidgetStyle::new(base))
        .unwrap();
    let class = StyleClassId::new(7);
    gui.set_widget_style_class(button, Some(class)).unwrap();
    gui.set_style_class_state(
        class,
        VisualState::Normal,
        Style {
            background: Some(Rgb565::BLACK),
            ..base
        },
    )
    .unwrap();
    gui.set_style_class_state(
        class,
        VisualState::Focused,
        Style {
            background: Some(Rgb565::RED),
            ..base
        },
    )
    .unwrap();
    gui.set_state_transition_duration_ms(100);
    gui.set_focus(Some(button)).unwrap();
    while gui.pop_event().is_some() {}
    gui.set_focus(None).unwrap();
    gui.tick_input(50).unwrap();

    let mut target = TestBuffer::new(40, 20);
    gui.render(&mut target).unwrap();
    let mid = target.pixel_at(8, 6).unwrap_or(Rgb565::BLACK);
    assert!(mid.r() > 0);
}

#[test]
fn state_transition_for_widget_is_replaced_by_new_state_change() {
    let mut gui = GuiContext::<8, 16, 8>::new(Rect::new(0, 0, 64, 32));
    let button = gui
        .add_button(Rect::new(0, 0, 20, 10), "A", Style::button())
        .unwrap();
    gui.set_state_transition_duration_ms(80);
    gui.set_focus(Some(button)).unwrap();
    while gui.pop_event().is_some() {}

    gui.set_focus(None).unwrap();
    assert_eq!(gui.active_state_transitions(), 1);
    gui.set_disabled(button, true).unwrap();
    assert_eq!(gui.active_state_transitions(), 1);
    gui.tick_input(100).unwrap();
    assert_eq!(gui.active_state_transitions(), 0);
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
