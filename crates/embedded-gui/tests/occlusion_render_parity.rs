//! Verifies `GuiContext::render_dirty_occlusion` (the two-tree, occlusion-aware
//! damage tracker in `crate::damage`/`crate::quadtree`) never diverges from
//! the trusted flat-dirty-rect renderer, `GuiContext::render_dirty`, and that
//! its safety fallback actually engages when the quadtree's fixed capacity
//! is exhausted.

use embedded_gui::framebuffer::Framebuffer;
use embedded_gui::prelude::*;

const W: u32 = 64;
const H: u32 = 64;

#[test]
fn occlusion_render_matches_flat_dirty_render_after_mutation() {
    let mut gui = GuiContext::<16, 24, 24>::new(Rect::new(0, 0, W, H));

    // An opaque, square-cornered panel confined to the top-left quadrant
    // (not the whole viewport), with a value label nested inside it, plus
    // an unrelated sibling widget elsewhere on screen.
    let opaque = Style::new()
        .with_background(Some(Rgb565::new(4, 8, 6)))
        .with_opacity(255)
        .with_corner_radius(0);

    let panel = gui.add_panel(Rect::new(0, 0, 40, 20), opaque).unwrap();
    let label = gui
        .add_value_label(Rect::new(4, 4, 20, 10), "V", 0, opaque)
        .unwrap();
    gui.add_child(panel, label).unwrap();

    let button = gui
        .add_button(Rect::new(0, 40, 40, 10), "Btn", Style::button())
        .unwrap();

    let mut fb = Framebuffer::<{ (W * H) as usize }>::new(W, H);
    gui.render(&mut fb).unwrap();
    gui.clear_dirty();

    // Mutate a nested widget and an unrelated sibling in the same frame.
    gui.set_value_label(label, 42).unwrap();
    gui.set_widget_opacity(button, 180).unwrap();

    let mut fb_dirty = fb.clone();
    let mut fb_occlusion = fb.clone();
    gui.render_dirty(&mut fb_dirty).unwrap();
    gui.render_dirty_occlusion(&mut fb_occlusion).unwrap();

    assert_eq!(fb_dirty.pixels(), fb_occlusion.pixels());
}

#[test]
fn six_touching_1px_rects_overflow_a_single_leaf_bucket() {
    // Confirms the exact geometry `occlusion_render_falls_back_safely_...`
    // relies on: `render_dirty_occlusion` hardcodes `depth = 3` over the
    // GuiContext viewport and 4 entries per node. For a 64x64 viewport that
    // makes each leaf quadrant 8x8 px. Six 1px-wide rects at x = 0..6 are
    // pairwise non-intersecting (touching, not overlapping, so they never
    // merge within a bucket) yet all land in the same (0,0,8,8) leaf,
    // exceeding its 4-entry capacity on the 5th insert.
    use embedded_gui::geometry::Rect as GRect;
    use embedded_gui::quadtree::{Quadtree, QuadtreeError, node_count_for_depth};

    const DEPTH: u8 = 3;
    let mut tree: Quadtree<{ node_count_for_depth(DEPTH) }, 4> =
        Quadtree::new(GRect::new(0, 0, W as i32 as u32, H), DEPTH);
    for x in 0..4 {
        tree.insert(GRect::new(x, 0, 1, 1)).unwrap();
    }
    assert_eq!(
        tree.insert(GRect::new(4, 0, 1, 1)),
        Err(QuadtreeError::Full)
    );
}

#[test]
fn occlusion_render_falls_back_safely_when_quadtree_capacity_is_exhausted() {
    // Six touching (non-merging) 1px-wide widgets packed into the same 8x8
    // leaf quadrant (see `six_touching_1px_rects_overflow_a_single_leaf_bucket`
    // for why), forcing `render_dirty_occlusion`'s documented fallback to
    // `render_dirty` — proving damage is never silently dropped rather than
    // just asserting "no panic".
    let mut gui = GuiContext::<16, 24, 32>::new(Rect::new(0, 0, W, H));

    let mut buttons = heapless::Vec::<WidgetId, 8>::new();
    for i in 0..6 {
        let id = gui
            .add_button(Rect::new(i, 0, 1, 1), "", Style::button())
            .unwrap();
        buttons.push(id).unwrap();
    }

    let mut fb = Framebuffer::<{ (W * H) as usize }>::new(W, H);
    gui.render(&mut fb).unwrap();
    gui.clear_dirty();

    for &id in &buttons {
        gui.set_widget_opacity(id, 200).unwrap();
    }
    assert!(
        gui.dirty_regions().len() >= 6,
        "test setup must produce at least 6 disjoint dirty regions"
    );

    let mut fb_dirty = fb.clone();
    let mut fb_occlusion = fb.clone();
    gui.render_dirty(&mut fb_dirty).unwrap();
    gui.render_dirty_occlusion(&mut fb_occlusion).unwrap();

    // The fallback must reproduce render_dirty's output exactly, not just
    // "render something".
    assert_eq!(fb_dirty.pixels(), fb_occlusion.pixels());
}
