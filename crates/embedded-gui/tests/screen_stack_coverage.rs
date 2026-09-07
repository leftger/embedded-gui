//! Coverage tests for `ScreenStack` commands and lifecycle events.

use embedded_gui::screen::{ScreenCommand, ScreenId, ScreenLifecycleEvent, ScreenStack};

#[test]
fn screen_stack_push_pop_replace_clear_lifecycle() {
    let mut stack = ScreenStack::<4>::with_root(ScreenId::new(1)).unwrap();
    let mut events = heapless::Vec::<ScreenLifecycleEvent, 16>::new();

    let transition = stack
        .apply_lifecycle(ScreenCommand::Push(ScreenId::new(2)), &mut events)
        .unwrap();
    assert_eq!(transition.to, Some(ScreenId::new(2)));
    assert!(events.contains(&ScreenLifecycleEvent::Pause(ScreenId::new(1))));
    assert!(events.contains(&ScreenLifecycleEvent::Mount(ScreenId::new(2))));

    events.clear();
    stack
        .apply_lifecycle(ScreenCommand::Replace(ScreenId::new(3)), &mut events)
        .unwrap();
    assert_eq!(stack.current(), Some(ScreenId::new(3)));

    events.clear();
    stack
        .apply_lifecycle(ScreenCommand::Pop, &mut events)
        .unwrap();
    assert_eq!(stack.current(), Some(ScreenId::new(1)));
    assert!(events.contains(&ScreenLifecycleEvent::Resume(ScreenId::new(1))));

    events.clear();
    stack
        .apply_lifecycle(ScreenCommand::ClearTo(ScreenId::new(9)), &mut events)
        .unwrap();
    assert_eq!(stack.current(), Some(ScreenId::new(9)));
    assert_eq!(stack.len(), 1);

    stack
        .apply_lifecycle(ScreenCommand::None, &mut events)
        .unwrap();
    assert_eq!(stack.as_slice().len(), 1);
}

#[test]
fn screen_stack_errors_and_empty_transitions() {
    let mut stack = ScreenStack::<2>::new();
    assert!(stack.is_empty());
    assert!(stack.pop().is_err());

    stack.push(ScreenId::new(1)).unwrap();
    stack.push(ScreenId::new(2)).unwrap();
    assert!(stack.push(ScreenId::new(3)).is_err());

    stack.pop().unwrap();
    stack.pop().unwrap();
    assert!(stack.replace(ScreenId::new(4)).is_err());

    let stack = ScreenStack::<2>::with_root_lifecycle(
        ScreenId::new(5),
        &mut heapless::Vec::<ScreenLifecycleEvent, 8>::new(),
    )
    .unwrap();
    assert_eq!(stack.current(), Some(ScreenId::new(5)));
}
