//! Spatial grid for fast hit testing.
//!
//! Divides screen space into cells and indexes hittable frames by which cells
//! they overlap. Point queries only scan frames in the relevant cell instead
//! of the full list.

use iced::{Point, Rectangle};
use std::collections::HashMap;

use super::frame_collect::HitOrderKey;

/// Cell size in screen pixels. Each cell is CELL_SIZE × CELL_SIZE.
/// 64px gives ~192 cells at 1024×768 and ~510 at 1920×1080.
const CELL_SIZE: f32 = 64.0;

/// Spatial grid for O(1) cell lookup + O(k) scan within the cell.
pub struct HitGrid {
    /// Flat array of cells, indexed by `row * cols + col`.
    /// Each cell holds frame IDs that overlap it, in render-order (low→high).
    cells: Vec<Vec<u64>>,
    /// Rectangle for each hittable frame, keyed by frame ID.
    rects: HashMap<u64, Rectangle>,
    /// Current render-bucket ranks, including frames not currently hittable.
    keys: HashMap<u64, HitOrderKey>,
    cols: usize,
    rows: usize,
}

impl HitGrid {
    /// Build a grid from the sorted hittable list.
    ///
    /// `hittable` must be sorted lowest-render-rank-first (same order as
    /// `build_hittable_rects` produces), so reverse iteration yields the
    /// topmost frame.
    pub fn new(hittable: Vec<(u64, Rectangle, HitOrderKey)>, screen_w: f32, screen_h: f32) -> Self {
        let cols = (screen_w / CELL_SIZE).ceil() as usize;
        let rows = (screen_h / CELL_SIZE).ceil() as usize;
        let cell_count = cols * rows;
        let mut cells: Vec<Vec<u64>> = vec![Vec::new(); cell_count];
        let mut rects = HashMap::with_capacity(hittable.len());
        let mut keys = HashMap::with_capacity(hittable.len());

        for &(id, rect, key) in &hittable {
            rects.insert(id, rect);
            keys.insert(id, key);
            let (c0, r0, c1, r1) = cell_range(rect, cols, rows);
            for row in r0..=r1 {
                for col in c0..=c1 {
                    cells[row * cols + col].push(id);
                }
            }
        }

        let mut grid = Self {
            cells,
            rects,
            keys,
            cols,
            rows,
        };
        grid.sort_cells();
        grid
    }

    fn sort_cells(&mut self) {
        let keys = &self.keys;
        for cell in &mut self.cells {
            cell.sort_by_key(|id| keys.get(id).copied());
        }
    }

    /// Refresh ranks once per update batch without rebuilding spatial cells.
    /// Frames no longer present in the render order are removed from hit testing.
    pub fn update_render_order(&mut self, strata_buckets: &[Vec<u64>]) {
        let next_keys: HashMap<_, _> = strata_buckets
            .iter()
            .flatten()
            .enumerate()
            .map(|(rank, &id)| (id, rank))
            .collect();
        let order_changed = self
            .rects
            .keys()
            .any(|id| self.keys.get(id) != next_keys.get(id));
        let removed: Vec<_> = self
            .rects
            .keys()
            .filter(|id| !next_keys.contains_key(*id))
            .copied()
            .collect();
        for id in removed {
            self.remove(id);
        }
        self.keys = next_keys;
        if order_changed {
            self.sort_cells();
        }
    }

    pub fn render_order_key(&self, id: u64) -> Option<HitOrderKey> {
        self.keys.get(&id).copied()
    }

    /// Find the topmost frame containing `pos` that also matches `predicate`.
    pub fn topmost_matching_at<F>(&self, pos: Point, mut predicate: F) -> Option<u64>
    where
        F: FnMut(u64) -> bool,
    {
        let col = ((pos.x / CELL_SIZE) as usize).min(self.cols.saturating_sub(1));
        let row = ((pos.y / CELL_SIZE) as usize).min(self.rows.saturating_sub(1));
        let cell = &self.cells[row * self.cols + col];
        cell.iter()
            .rev()
            .find(|&&id| self.rects.get(&id).is_some_and(|r| r.contains(pos)) && predicate(id))
            .copied()
    }

    /// Check if a frame is in the hittable set and contains `pos` (Phase 2).
    #[cfg(test)]
    pub fn contains(&self, id: u64, pos: Point) -> bool {
        self.rects.get(&id).is_some_and(|r| r.contains(pos))
    }

    /// Remove a frame from the grid.
    ///
    /// Uses the stored rect to find which cells contained the frame.
    pub fn remove(&mut self, id: u64) {
        self.keys.remove(&id);
        let Some(rect) = self.rects.remove(&id) else {
            return;
        };
        let (c0, r0, c1, r1) = cell_range(rect, self.cols, self.rows);
        for row in r0..=r1 {
            for col in c0..=c1 {
                let cell = &mut self.cells[row * self.cols + col];
                cell.retain(|&fid| fid != id);
            }
        }
    }

    /// Insert a frame into the grid at its render-order position.
    ///
    /// A plain append would make the newest insert the topmost hit in its
    /// cells regardless of strata/level — every incremental update would
    /// shadow overlapping frames (the staleness bug that previously forced
    /// full rebuilds). Reinserting the same frame ID replaces the old cells so
    /// callers do not need to coordinate a separate remove/insert sequence.
    pub fn insert(&mut self, id: u64, rect: Rectangle, key: HitOrderKey) {
        if let Some(old_rect) = self.rects.get(&id).copied() {
            self.remove_from_cells(id, old_rect);
        }
        self.rects.insert(id, rect);
        self.keys.insert(id, key);
        let keys = &self.keys;
        let (c0, r0, c1, r1) = cell_range(rect, self.cols, self.rows);
        for row in r0..=r1 {
            for col in c0..=c1 {
                let cell = &mut self.cells[row * self.cols + col];
                let pos = cell.partition_point(|other| {
                    keys.get(other)
                        .copied()
                        .is_some_and(|other_key| other_key <= key)
                });
                cell.insert(pos, id);
            }
        }
    }

    fn remove_from_cells(&mut self, id: u64, rect: Rectangle) {
        let (c0, r0, c1, r1) = cell_range(rect, self.cols, self.rows);
        for row in r0..=r1 {
            for col in c0..=c1 {
                let cell = &mut self.cells[row * self.cols + col];
                cell.retain(|&fid| fid != id);
            }
        }
    }
}

/// Compute the inclusive cell range `(col_start, row_start, col_end, row_end)`
/// for a rectangle.
fn cell_range(rect: Rectangle, cols: usize, rows: usize) -> (usize, usize, usize, usize) {
    let max_col = cols.saturating_sub(1);
    let max_row = rows.saturating_sub(1);
    let c0 = (rect.x / CELL_SIZE) as usize;
    let r0 = (rect.y / CELL_SIZE) as usize;
    let c1 = ((rect.x + rect.width) / CELL_SIZE) as usize;
    let r1 = ((rect.y + rect.height) / CELL_SIZE) as usize;
    (
        c0.min(max_col),
        r0.min(max_row),
        c1.min(max_col),
        r1.min(max_row),
    )
}

/// Brute-force linear scan (equivalent to old hit_test Phase 1).
/// Used only in tests to verify grid results.
#[cfg(test)]
fn linear_topmost(hittable: &[(u64, Rectangle)], pos: Point) -> Option<u64> {
    hittable.iter().rev().find_map(
        |(id, rect)| {
            if rect.contains(pos) { Some(*id) } else { None }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Size;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rectangle {
        Rectangle::new(Point::new(x, y), Size::new(w, h))
    }

    /// Assign render-order keys following list order (low to high), matching
    /// the sorted input contract of `HitGrid::new`.
    fn keyed(hittable: &[(u64, Rectangle)]) -> Vec<(u64, Rectangle, HitOrderKey)> {
        hittable
            .iter()
            .enumerate()
            .map(|(rank, &(id, r))| (id, r, rank))
            .collect()
    }

    #[test]
    fn topmost_matches_linear_scan() {
        // 3 overlapping frames at different strata (sorted low→high).
        let hittable = vec![
            (1, rect(0.0, 0.0, 200.0, 200.0)),   // low strata, big
            (2, rect(50.0, 50.0, 100.0, 100.0)), // mid strata, overlaps
            (3, rect(80.0, 80.0, 40.0, 40.0)),   // high strata, small
        ];
        let grid = HitGrid::new(keyed(&hittable), 256.0, 256.0);

        // Points that should hit different frames.
        let cases = [
            (Point::new(10.0, 10.0), Some(1)),   // only frame 1
            (Point::new(60.0, 60.0), Some(2)),   // frames 1+2, topmost=2
            (Point::new(90.0, 90.0), Some(3)),   // all three, topmost=3
            (Point::new(130.0, 130.0), Some(2)), // frames 1+2 (3 ends at 120)
            (Point::new(180.0, 180.0), Some(1)), // only frame 1
            (Point::new(250.0, 250.0), None),    // outside all
        ];
        for (pos, expected) in cases {
            let grid_result = grid.topmost_matching_at(pos, |_| true);
            let linear_result = linear_topmost(&hittable, pos);
            assert_eq!(grid_result, expected, "grid mismatch at {pos:?}");
            assert_eq!(grid_result, linear_result, "grid != linear at {pos:?}");
        }
    }

    #[test]
    fn frame_spanning_multiple_cells() {
        // One frame spanning several cells.
        let hittable = vec![(1, rect(10.0, 10.0, 200.0, 200.0))];
        let grid = HitGrid::new(keyed(&hittable), 256.0, 256.0);

        // Test points in different cells within the frame.
        assert_eq!(
            grid.topmost_matching_at(Point::new(20.0, 20.0), |_| true),
            Some(1)
        ); // cell (0,0)
        assert_eq!(
            grid.topmost_matching_at(Point::new(100.0, 100.0), |_| true),
            Some(1)
        ); // cell (1,1)
        assert_eq!(
            grid.topmost_matching_at(Point::new(200.0, 200.0), |_| true),
            Some(1)
        ); // cell (3,3)
        // Just outside.
        assert_eq!(
            grid.topmost_matching_at(Point::new(5.0, 5.0), |_| true),
            None
        );
    }

    #[test]
    fn contains_checks_rect() {
        let hittable = vec![(1, rect(100.0, 100.0, 50.0, 50.0))];
        let grid = HitGrid::new(keyed(&hittable), 256.0, 256.0);

        assert!(grid.contains(1, Point::new(120.0, 120.0)));
        assert!(!grid.contains(1, Point::new(90.0, 90.0)));
        assert!(!grid.contains(999, Point::new(120.0, 120.0))); // unknown id
    }

    #[test]
    fn cell_boundary_frame() {
        // Frame exactly on cell boundary (64px).
        let hittable = vec![
            (1, rect(60.0, 60.0, 10.0, 10.0)), // spans cells (0,0) and (1,1)
        ];
        let grid = HitGrid::new(keyed(&hittable), 128.0, 128.0);

        assert_eq!(
            grid.topmost_matching_at(Point::new(63.0, 63.0), |_| true),
            Some(1)
        );
        assert_eq!(
            grid.topmost_matching_at(Point::new(65.0, 65.0), |_| true),
            Some(1)
        );
        assert_eq!(
            grid.topmost_matching_at(Point::new(59.0, 59.0), |_| true),
            None
        );
    }

    #[test]
    fn incremental_insert_respects_render_order() {
        // A lower frame re-inserted incrementally (e.g. after moving) must
        // NOT shadow a higher overlapping frame. The old append-based insert
        // made the newest insert win every overlapping cell, which is the
        // staleness bug that previously forced full grid rebuilds.
        let hittable = vec![
            (1, rect(0.0, 0.0, 200.0, 200.0)),
            (2, rect(50.0, 50.0, 100.0, 100.0)),
        ];
        let mut grid = HitGrid::new(keyed(&hittable), 256.0, 256.0);

        // Re-insert the bottom frame with its original (lowest) key.
        grid.remove(1);
        grid.insert(1, rect(0.0, 0.0, 200.0, 200.0), 0);

        assert_eq!(
            grid.topmost_matching_at(Point::new(60.0, 60.0), |_| true),
            Some(2),
            "re-inserted bottom frame must stay below the overlapping top frame"
        );

        // A frame inserted with a HIGHER key still wins.
        grid.remove(2);
        grid.insert(2, rect(50.0, 50.0, 100.0, 100.0), 1);
        assert_eq!(
            grid.topmost_matching_at(Point::new(60.0, 60.0), |_| true),
            Some(2)
        );
    }

    #[test]
    fn reinserting_same_id_replaces_stale_cells() {
        let mut grid = HitGrid::new(vec![], 128.0, 128.0);
        let key = 0;

        grid.insert(1, rect(10.0, 10.0, 20.0, 20.0), key);
        grid.insert(1, rect(90.0, 90.0, 20.0, 20.0), key);

        assert_eq!(
            grid.topmost_matching_at(Point::new(20.0, 20.0), |_| true),
            None
        );
        assert_eq!(
            grid.topmost_matching_at(Point::new(100.0, 100.0), |_| true),
            Some(1)
        );
    }

    #[test]
    fn many_frames_stress_test() {
        // 1000 non-overlapping 10x10 frames in a grid pattern.
        let mut hittable = Vec::new();
        for i in 0..1000u64 {
            let x = (i % 50) as f32 * 20.0;
            let y = (i / 50) as f32 * 20.0;
            hittable.push((i, rect(x, y, 10.0, 10.0)));
        }
        let grid = HitGrid::new(keyed(&hittable), 1000.0, 400.0);

        // Check every frame is hittable at its center.
        for &(id, r) in &hittable {
            let center = Point::new(r.x + 5.0, r.y + 5.0);
            assert_eq!(
                grid.topmost_matching_at(center, |_| true),
                Some(id),
                "missed frame {id}"
            );
        }

        // Check gaps between frames return None.
        assert_eq!(
            grid.topmost_matching_at(Point::new(15.0, 5.0), |_| true),
            None
        );
    }
}
