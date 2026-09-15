//! Two-tree occlusion-aware damage tracking.
//!
//! Ports the scheme described in
//! <https://bensimms.moe/reverse-engineering-scooter/> (footnote 7): instead
//! of one dirty-region set, keep two [`Quadtree`]s — `dirty` and
//! `overdrawn` — so that redraw work can be pruned using knowledge of what's
//! *about* to be freshly, opaquely repainted this frame, not just what
//! changed:
//!
//! - When a component changes, its prior bounding box is added to the
//!   **dirty** tree (something needs to be repainted there).
//! - When a component repaints and fully, opaquely covers its own bounding
//!   box, that box is added to the **overdrawn** tree, and any dirty
//!   entries fully contained within it are pruned — that damage has
//!   already been subsumed by a guaranteed-fresh repaint.
//! - A node's subtree is only worth visiting if its bounds overlap *either*
//!   tree (a cheap short-circuit before descending into children).
//! - After visiting children, a node still needs to repaint its own content
//!   if its bounds still overlap the **dirty** tree — i.e. its children
//!   didn't fully cover the damage underneath it (e.g. a container's own
//!   background peeking out around non-opaque or partially-sized children).
//!
//! This is a data/algorithm module only — see
//! [`crate::context::GuiContext::render_dirty_occlusion`] for how it's wired
//! into the actual widget tree and render walk.

use crate::geometry::Rect;
use crate::quadtree::{Quadtree, QuadtreeError};

/// Two quadtrees (dirty + overdrawn) implementing the damage-tracking rules
/// above. `NODES`/`PER_NODE` size both trees identically; see
/// [`crate::quadtree::node_count_for_depth`] to compute `NODES` from a
/// chosen split `depth`.
pub struct DamageTree<const NODES: usize, const PER_NODE: usize> {
    dirty: Quadtree<NODES, PER_NODE>,
    overdrawn: Quadtree<NODES, PER_NODE>,
}

impl<const NODES: usize, const PER_NODE: usize> DamageTree<NODES, PER_NODE> {
    pub fn new(bounds: Rect, depth: u8) -> Self {
        Self {
            dirty: Quadtree::new(bounds, depth),
            overdrawn: Quadtree::new(bounds, depth),
        }
    }

    /// A component changed: its prior (or unchanged) bounding box needs a
    /// repaint somewhere in its vicinity.
    ///
    /// Failure here means the region could not be recorded (quadtree
    /// capacity exhausted) — callers must treat that as "damage may be
    /// silently lost" and fall back to a safe, always-correct renderer
    /// rather than ignore the error, since under-painting (unlike
    /// over-painting) is a real visual bug.
    pub fn mark_dirty(&mut self, prior_bounds: Rect) -> Result<(), QuadtreeError> {
        self.dirty.insert(prior_bounds)
    }

    /// A component repainted and fully, opaquely covers `new_bounds`: record
    /// it as overdrawn, and prune any dirty entries it fully subsumes.
    ///
    /// Unlike [`Self::mark_dirty`], failure here is always safe to ignore —
    /// worst case is a redundant repaint later, never a missed one.
    pub fn mark_overdrawn(&mut self, new_bounds: Rect) -> Result<(), QuadtreeError> {
        self.overdrawn.insert(new_bounds)?;
        self.dirty.remove_contained_within(new_bounds);
        Ok(())
    }

    /// Whether `bounds` is worth descending into at all: true if it overlaps
    /// either tree. Used as the pre-recursion short-circuit.
    pub fn overlaps_before(&self, bounds: Rect) -> bool {
        self.dirty.intersects(bounds) || self.overdrawn.intersects(bounds)
    }

    /// Whether `bounds` still needs its own repaint after children have had
    /// a chance to mark themselves overdrawn (and thereby prune the dirty
    /// tree). Used as the post-recursion repaint decision.
    pub fn overlaps_after(&self, bounds: Rect) -> bool {
        self.dirty.intersects(bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quadtree::node_count_for_depth;

    const DEPTH: u8 = 3;
    const NODES: usize = node_count_for_depth(DEPTH);
    type TestDamage = DamageTree<NODES, 4>;

    #[test]
    fn unrelated_bounds_are_left_alone() {
        let mut damage = TestDamage::new(Rect::new(0, 0, 320, 240), DEPTH);
        damage.mark_dirty(Rect::new(10, 10, 20, 20)).unwrap();
        // Somewhere else entirely: no reason to visit it.
        assert!(!damage.overlaps_before(Rect::new(200, 200, 20, 20)));
    }

    #[test]
    fn dirty_region_triggers_before_and_after() {
        let mut damage = TestDamage::new(Rect::new(0, 0, 320, 240), DEPTH);
        damage.mark_dirty(Rect::new(10, 10, 20, 20)).unwrap();
        let bounds = Rect::new(0, 0, 50, 50);
        assert!(damage.overlaps_before(bounds));
        assert!(damage.overlaps_after(bounds));
    }

    #[test]
    fn opaque_child_repaint_prunes_ancestor_dirty_overlap() {
        let mut damage = TestDamage::new(Rect::new(0, 0, 320, 240), DEPTH);
        let child_bounds = Rect::new(10, 10, 20, 20);
        let ancestor_bounds = Rect::new(0, 0, 320, 240);

        damage.mark_dirty(child_bounds).unwrap();
        assert!(damage.overlaps_before(ancestor_bounds));
        assert!(damage.overlaps_after(ancestor_bounds));

        // The child repaints and is a full opaque cover of its own bounds.
        damage.mark_overdrawn(child_bounds).unwrap();

        // The ancestor's own content no longer needs to repaint on account
        // of this specific region: the child already handled it.
        assert!(!damage.overlaps_after(ancestor_bounds));
    }

    #[test]
    fn non_opaque_child_leaves_ancestor_dirty() {
        let mut damage = TestDamage::new(Rect::new(0, 0, 320, 240), DEPTH);
        let child_bounds = Rect::new(10, 10, 20, 20);
        let ancestor_bounds = Rect::new(0, 0, 320, 240);

        damage.mark_dirty(child_bounds).unwrap();
        // Child repaints but is *not* a full opaque cover (e.g. transparent
        // background) -> caller must not call mark_overdrawn for it.
        assert!(damage.overlaps_after(ancestor_bounds));
    }

    #[test]
    fn overdrawn_region_alone_still_gates_the_before_check() {
        // Nothing dirty here, but something else already guaranteed a fresh
        // repaint over this exact area (e.g. a sibling scheduled first) —
        // still worth visiting once to account for the coverage.
        let mut damage = TestDamage::new(Rect::new(0, 0, 320, 240), DEPTH);
        let bounds = Rect::new(5, 5, 10, 10);
        damage.mark_overdrawn(bounds).unwrap();
        assert!(damage.overlaps_before(bounds));
        assert!(!damage.overlaps_after(bounds));
    }

    #[test]
    fn partial_dirty_gap_under_container_still_repaints() {
        // A container's dirty region is only partially covered by a child's
        // overdrawn mark: the remaining gap must still cause a repaint.
        let mut damage = TestDamage::new(Rect::new(0, 0, 320, 240), DEPTH);
        let container_bounds = Rect::new(0, 0, 100, 100);
        damage.mark_dirty(container_bounds).unwrap();

        let child_bounds = Rect::new(0, 0, 40, 40);
        damage.mark_overdrawn(child_bounds).unwrap();

        // Only the child's corner was subsumed; the rest of the container's
        // dirty region is untouched.
        assert!(damage.overlaps_after(container_bounds));
    }
}
