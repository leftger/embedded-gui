//! Coverage tests for scroll/list/tabs/slider state, signals, callbacks,
//! state machines, and slice models.

use embedded_gui::prelude::*;
use embedded_gui::state::{
    CallbackSlot, FeedTimelineState, ListState, ModelChange, ScrollState, Signal, SliceModel,
    SliderState, StateTransition, TabsState, WidgetStateMachine,
};
use embedded_gui::widget::WidgetId;

static CALLED: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
fn handler(_: u32) {
    CALLED.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
}

#[test]
fn list_tabs_scroll_slider_and_feed_state() {
    let mut list = ListState::new(0, 0, 3);
    assert!(list.next(5));
    assert!(list.previous(5));
    assert!(list.bump(5, 1));
    assert!(!list.bump(0, 1));
    list.set_selected(10, 5);
    list.keep_selected_visible();
    assert_eq!(list.selected, 4);

    let mut tabs = TabsState::new(0);
    assert!(tabs.next(3));
    assert!(tabs.previous(3));
    assert!(tabs.bump(3, 1));
    assert!(!tabs.bump(0, 1));
    tabs.set_selected(5, 3);
    assert_eq!(tabs.selected, 2);

    let mut scroll = ScrollState::new(0, 100);
    assert!(scroll.set_offset(50));
    assert!(scroll.scroll_by(10));
    assert_eq!(scroll.offset_y, 60);

    let mut slider = SliderState::new(0.0, 0.0, 1.0);
    assert!(slider.set_value(0.5));
    assert!(slider.step_by(1.0));
    assert!(slider.value > 0.5);

    let mut feed = FeedTimelineState::new(0, 0, 3, false);
    assert!(feed.set_selected(2, 5));
    assert!(feed.bump(5, 1));
    assert!(feed.set_expanded(true));
    assert!(!feed.bump(0, 1));
}

#[test]
fn signal_callback_and_state_machine() {
    let mut signal = Signal::<u32, 4>::new(0);
    assert_eq!(signal.get(), 0);
    let id = WidgetId::new(1);
    assert!(signal.subscribe(id));
    assert!(signal.subscribe(id));
    assert!(signal.set(7));
    assert!(!signal.set(7));
    assert!(signal.is_dirty());
    assert_eq!(signal.version(), 1);
    assert_eq!(signal.subscribers().len(), 1);
    signal.unsubscribe(id);
    assert_eq!(signal.subscribers().len(), 0);
    signal.mark_dirty();
    signal.clear_dirty();
    assert!(!signal.is_dirty());

    let mut cb: CallbackSlot<u32> = CallbackSlot::empty();
    assert!(!cb.is_bound());
    cb.set(handler);
    assert!(cb.is_bound());
    cb.emit(1);
    assert_eq!(CALLED.load(core::sync::atomic::Ordering::Relaxed), 1);
    cb.clear();
    assert!(!cb.is_bound());
    let cb2 = CallbackSlot::<u32>::new(handler);
    assert!(cb2.is_bound());

    let mut machine = WidgetStateMachine::new(VisualState::Normal);
    assert!(!machine.is_animating());
    assert!(machine.transition_to(VisualState::Focused, 10));
    assert!(machine.is_animating());
    assert!(machine.tick(5));
    assert!(machine.tick(5));
    assert!(!machine.is_animating());
    assert_eq!(machine.current(), VisualState::Focused);
    assert!(machine.transition_to(VisualState::Pressed, 0));
    assert!(!machine.is_animating());
    assert!(!machine.transition_to(VisualState::Pressed, 0));
    let _ = machine.lerp_scalar(0.0, 1.0);
    let _ = StateTransition::new(Some(VisualState::Normal), VisualState::Pressed, 5);
    let _ = machine.progress();
    let _ = machine.target();
}

#[test]
fn slice_model_counts_and_rows() {
    let data = [10, 20, 30];
    let model = SliceModel::new(&data);
    assert_eq!(model.row_count(), 3);
    assert_eq!(model.row_data(1), Some(20));
    assert_eq!(model.row_data(99), None);
    let _ = ModelChange::Reset;
}
