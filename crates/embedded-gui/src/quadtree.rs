//! Fixed-capacity, `no_std`, zero-allocation region quadtree.
//!
//! Spatially buckets [`Rect`] regions into a complete 4-ary tree over a fixed
//! `bounds`/`depth`, so that unrelated regions (e.g. two widgets in opposite
//! screen corners) never compete for the same small entry list the way a
//! single flat capped `Vec<Rect, N>` does. Each node still merges
//! intersecting entries within its own bucket (same trick as
//! [`crate::geometry::DirtyTracker`]), so the structure stays compact even
//! under heavy churn.
//!
//! Node `i` (0-indexed, root at `0`) has children `4*i + 1 ..= 4*i + 4`,
//! covering the four quadrants of `i`'s own bounds. This is the data
//! structure behind [`crate::damage::DamageTree`]'s dirty/overdrawn region
//! tracking.

use crate::geometry::Rect;

/// Per-node entry capacity is small and fixed; this is the error returned
/// when a node's bucket (and, transitively, insertion couldn't descend
/// further) is already full.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuadtreeError {
    Full,
}

/// Total node count for a complete 4-ary tree of `depth` splits (`depth = 0`
/// is a single root node with no children). Use this to size the `NODES`
/// const generic of [`Quadtree`]: `Quadtree::<{ node_count_for_depth(3) }, 4>`.
pub const fn node_count_for_depth(depth: u8) -> usize {
    let mut total = 0usize;
    let mut count = 1usize;
    let mut level = 0u8;
    while level <= depth {
        total += count;
        count *= 4;
        level += 1;
    }
    total
}

fn quadrant_bounds(bounds: Rect, quadrant: u8) -> Rect {
    let half_w = bounds.w / 2;
    let half_h = bounds.h / 2;
    match quadrant {
        0 => Rect::new(bounds.x, bounds.y, half_w, half_h),
        1 => Rect::new(
            bounds.x + half_w as i32,
            bounds.y,
            bounds.w - half_w,
            half_h,
        ),
        2 => Rect::new(
            bounds.x,
            bounds.y + half_h as i32,
            half_w,
            bounds.h - half_h,
        ),
        _ => Rect::new(
            bounds.x + half_w as i32,
            bounds.y + half_h as i32,
            bounds.w - half_w,
            bounds.h - half_h,
        ),
    }
}

fn push_merge<const CAP: usize>(
    bucket: &mut heapless::Vec<Rect, CAP>,
    rect: Rect,
) -> Result<(), QuadtreeError> {
    if rect.is_empty() {
        return Ok(());
    }
    if bucket.iter().any(|r| r.intersects(rect)) {
        let mut merged = rect;
        let mut i = 0;
        while i < bucket.len() {
            if bucket[i].intersects(merged) {
                merged = merged.union(bucket.swap_remove(i));
            } else {
                i += 1;
            }
        }
        return bucket.push(merged).map_err(|_| QuadtreeError::Full);
    }
    bucket.push(rect).map_err(|_| QuadtreeError::Full)
}

/// A fixed-capacity region quadtree over `bounds`, split up to `depth` times.
///
/// `NODES` must be at least [`node_count_for_depth(depth)`](node_count_for_depth);
/// this is checked with a `debug_assert` in [`Quadtree::new`]. `PER_NODE` is
/// the entry capacity of each individual node's bucket.
#[derive(Clone, Debug)]
pub struct Quadtree<const NODES: usize, const PER_NODE: usize> {
    bounds: Rect,
    depth: u8,
    nodes: [heapless::Vec<Rect, PER_NODE>; NODES],
}

impl<const NODES: usize, const PER_NODE: usize> Quadtree<NODES, PER_NODE> {
    pub fn new(bounds: Rect, depth: u8) -> Self {
        debug_assert!(
            NODES >= node_count_for_depth(depth),
            "Quadtree NODES too small for requested depth"
        );
        Self {
            bounds,
            depth,
            nodes: core::array::from_fn(|_| heapless::Vec::new()),
        }
    }

    pub const fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn clear(&mut self) {
        for node in &mut self.nodes {
            node.clear();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.iter().all(|node| node.is_empty())
    }

    /// Inserts `rect`, descending into the smallest single quadrant that
    /// fully contains it (or storing it at the current node if it straddles
    /// a split, or the configured `depth` is exhausted).
    pub fn insert(&mut self, rect: Rect) -> Result<(), QuadtreeError> {
        if rect.is_empty() {
            return Ok(());
        }
        self.insert_at(0, self.bounds, self.depth, rect)
    }

    fn insert_at(
        &mut self,
        index: usize,
        bounds: Rect,
        remaining_depth: u8,
        rect: Rect,
    ) -> Result<(), QuadtreeError> {
        if remaining_depth > 0 {
            for quadrant in 0..4u8 {
                let child_bounds = quadrant_bounds(bounds, quadrant);
                if child_bounds.contains_rect(rect) {
                    let child_index = 4 * index + 1 + quadrant as usize;
                    return self.insert_at(child_index, child_bounds, remaining_depth - 1, rect);
                }
            }
        }
        push_merge(&mut self.nodes[index], rect)
    }

    /// Whether any stored region intersects `query`.
    pub fn intersects(&self, query: Rect) -> bool {
        if query.is_empty() {
            return false;
        }
        self.intersects_at(0, self.bounds, self.depth, query)
    }

    fn intersects_at(&self, index: usize, bounds: Rect, remaining_depth: u8, query: Rect) -> bool {
        if !bounds.intersects(query) {
            return false;
        }
        if self.nodes[index].iter().any(|r| r.intersects(query)) {
            return true;
        }
        if remaining_depth == 0 {
            return false;
        }
        for quadrant in 0..4u8 {
            let child_bounds = quadrant_bounds(bounds, quadrant);
            if child_bounds.intersects(query) {
                let child_index = 4 * index + 1 + quadrant as usize;
                if self.intersects_at(child_index, child_bounds, remaining_depth - 1, query) {
                    return true;
                }
            }
        }
        false
    }

    /// Removes every stored region fully contained within `region` (used to
    /// prune dirty regions that are subsumed once something guarantees a
    /// fresh, opaque repaint over them — see [`crate::damage::DamageTree`]).
    pub fn remove_contained_within(&mut self, region: Rect) {
        if region.is_empty() {
            return;
        }
        self.remove_contained_at(0, self.bounds, self.depth, region);
    }

    fn remove_contained_at(
        &mut self,
        index: usize,
        bounds: Rect,
        remaining_depth: u8,
        region: Rect,
    ) {
        if !bounds.intersects(region) {
            return;
        }
        self.nodes[index].retain(|r| !region.contains_rect(*r));
        if remaining_depth == 0 {
            return;
        }
        for quadrant in 0..4u8 {
            let child_bounds = quadrant_bounds(bounds, quadrant);
            if child_bounds.intersects(region) {
                let child_index = 4 * index + 1 + quadrant as usize;
                self.remove_contained_at(child_index, child_bounds, remaining_depth - 1, region);
            }
        }
    }

    /// Copies every stored region into `out`, in no particular order and
    /// stopping (without error) once `out` is full.
    pub fn collect_into<const M: usize>(&self, out: &mut heapless::Vec<Rect, M>) {
        for node in &self.nodes {
            for &rect in node.iter() {
                if out.push(rect).is_err() {
                    return;
                }
            }
        }
    }

    /// Total number of stored regions across every node (not merged/deduped
    /// across nodes — regions straddling a split are only ever stored once,
    /// at the ancestor node where they were inserted).
    pub fn len(&self) -> usize {
        self.nodes.iter().map(|node| node.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEPTH: u8 = 3;
    const NODES: usize = node_count_for_depth(DEPTH);
    type TestTree = Quadtree<NODES, 4>;

    #[test]
    fn node_count_matches_geometric_series() {
        assert_eq!(node_count_for_depth(0), 1);
        assert_eq!(node_count_for_depth(1), 5);
        assert_eq!(node_count_for_depth(2), 21);
        assert_eq!(node_count_for_depth(3), 85);
    }

    #[test]
    fn opposite_corners_stay_in_separate_buckets() {
        // The whole point of spatial bucketing: two far-apart rects must not
        // compete for the same node's small PER_NODE capacity.
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        let top_left = Rect::new(0, 0, 10, 10);
        let bottom_right = Rect::new(300, 220, 10, 10);
        tree.insert(top_left).unwrap();
        tree.insert(bottom_right).unwrap();

        assert!(tree.intersects(Rect::new(0, 0, 5, 5)));
        assert!(tree.intersects(Rect::new(305, 225, 2, 2)));
        assert!(!tree.intersects(Rect::new(150, 100, 5, 5)));
    }

    #[test]
    fn insert_query_and_miss() {
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        tree.insert(Rect::new(20, 20, 40, 40)).unwrap();
        assert!(tree.intersects(Rect::new(30, 30, 5, 5)));
        assert!(!tree.intersects(Rect::new(200, 200, 5, 5)));
        assert!(!tree.is_empty());
    }

    #[test]
    fn straddling_rect_is_stored_at_ancestor() {
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        // Spans the vertical split down the middle: doesn't fit fully in
        // any single quadrant at any depth, so it must land at the root.
        tree.insert(Rect::new(150, 0, 20, 240)).unwrap();
        assert!(!tree.intersects(Rect::new(0, 0, 1, 1)));
        assert!(tree.intersects(Rect::new(155, 5, 1, 1)));
    }

    #[test]
    fn remove_contained_within_prunes_subsumed_entries() {
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        tree.insert(Rect::new(20, 20, 10, 10)).unwrap();
        tree.insert(Rect::new(200, 150, 10, 10)).unwrap();
        assert_eq!(tree.len(), 2);

        // Fully covers the first rect but not the second.
        tree.remove_contained_within(Rect::new(0, 0, 40, 40));
        assert_eq!(tree.len(), 1);
        assert!(!tree.intersects(Rect::new(25, 25, 1, 1)));
        assert!(tree.intersects(Rect::new(205, 155, 1, 1)));
    }

    #[test]
    fn remove_contained_within_does_not_remove_partial_overlap() {
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        tree.insert(Rect::new(20, 20, 40, 40)).unwrap();
        // Overlaps but doesn't fully contain the stored rect.
        tree.remove_contained_within(Rect::new(30, 30, 10, 10));
        assert!(tree.intersects(Rect::new(21, 21, 1, 1)));
    }

    #[test]
    fn collect_into_returns_all_regions() {
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        tree.insert(Rect::new(0, 0, 10, 10)).unwrap();
        tree.insert(Rect::new(300, 230, 10, 10)).unwrap();
        tree.insert(Rect::new(150, 0, 20, 240)).unwrap(); // straddling -> root

        let mut out: heapless::Vec<Rect, 8> = heapless::Vec::new();
        tree.collect_into(&mut out);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn merges_intersecting_inserts_within_the_same_bucket() {
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        tree.insert(Rect::new(10, 10, 10, 10)).unwrap();
        tree.insert(Rect::new(15, 15, 10, 10)).unwrap();
        // Both land in the same small quadrant and overlap, so they merge
        // into a single bucket entry instead of consuming two slots.
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn insert_ignores_empty_rect() {
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        tree.insert(Rect::empty()).unwrap();
        assert!(tree.is_empty());
    }

    #[test]
    fn clear_empties_every_node() {
        let mut tree = TestTree::new(Rect::new(0, 0, 320, 240), DEPTH);
        tree.insert(Rect::new(0, 0, 10, 10)).unwrap();
        tree.insert(Rect::new(300, 230, 10, 10)).unwrap();
        tree.clear();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn depth_zero_tree_behaves_like_a_flat_merge_list() {
        let mut tree: Quadtree<1, 4> = Quadtree::new(Rect::new(0, 0, 320, 240), 0);
        tree.insert(Rect::new(0, 0, 10, 10)).unwrap();
        tree.insert(Rect::new(300, 230, 10, 10)).unwrap();
        assert_eq!(tree.len(), 2);
        assert!(tree.intersects(Rect::new(5, 5, 1, 1)));
    }

    #[test]
    fn full_bucket_reports_error_instead_of_silently_dropping() {
        let mut tree: Quadtree<1, 2> = Quadtree::new(Rect::new(0, 0, 8, 8), 0);
        tree.insert(Rect::new(0, 0, 1, 1)).unwrap();
        tree.insert(Rect::new(2, 0, 1, 1)).unwrap();
        assert_eq!(tree.insert(Rect::new(4, 0, 1, 1)), Err(QuadtreeError::Full));
    }
}
