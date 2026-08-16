use embedded_gui::prelude::*;

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
