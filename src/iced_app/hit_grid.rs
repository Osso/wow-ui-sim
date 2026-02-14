//! Spatial grid for fast hit testing.
//!
//! Divides screen space into cells and indexes hittable frames by which cells
//! they overlap. Point queries only scan frames in the relevant cell instead
//! of the full list.

use std::collections::HashMap;
use iced::{Point, Rectangle};

/// Cell size in screen pixels. Each cell is CELL_SIZE × CELL_SIZE.
/// 64px gives ~192 cells at 1024×768 and ~510 at 1920×1080.
const CELL_SIZE: f32 = 64.0;

/// Spatial grid for O(1) cell lookup + O(k) scan within the cell.
pub struct HitGrid {
    /// Flat array of cells, indexed by `row * cols + col`.
    /// Each cell holds frame IDs that overlap it, in strata/level order (low→high).
    cells: Vec<Vec<u64>>,
    /// Rectangle for each hittable frame, keyed by frame ID.
    rects: HashMap<u64, Rectangle>,
    cols: usize,
    rows: usize,
}

impl HitGrid {
    /// Build a grid from the sorted hittable list.
    ///
    /// `hittable` must be sorted lowest-strata-first (same order as
    /// `build_hittable_rects` produces), so reverse iteration yields the
    /// topmost frame.
    pub fn new(hittable: Vec<(u64, Rectangle)>, screen_w: f32, screen_h: f32) -> Self {
        let cols = (screen_w / CELL_SIZE).ceil() as usize;
        let rows = (screen_h / CELL_SIZE).ceil() as usize;
        let cell_count = cols * rows;
        let mut cells: Vec<Vec<u64>> = vec![Vec::new(); cell_count];
        let mut rects = HashMap::with_capacity(hittable.len());

        for &(id, rect) in &hittable {
            rects.insert(id, rect);
            let (c0, r0, c1, r1) = cell_range(rect, cols, rows);
            for row in r0..=r1 {
                for col in c0..=c1 {
                    cells[row * cols + col].push(id);
                }
            }
        }

        Self { cells, rects, cols, rows }
    }

    /// Find the topmost frame containing `pos` (Phase 1).
    ///
    /// Returns the frame with the highest strata/level whose rect contains
    /// the point, or `None`.
    pub fn topmost_at(&self, pos: Point) -> Option<u64> {
        let col = ((pos.x / CELL_SIZE) as usize).min(self.cols.saturating_sub(1));
        let row = ((pos.y / CELL_SIZE) as usize).min(self.rows.saturating_sub(1));
        let cell = &self.cells[row * self.cols + col];
        // Reverse: highest strata/level is last in the sorted order.
        cell.iter().rev().find(|&&id| {
            self.rects.get(&id).is_some_and(|r| r.contains(pos))
        }).copied()
    }

    /// Check if a frame is in the hittable set and contains `pos` (Phase 2).
    pub fn contains(&self, id: u64, pos: Point) -> bool {
        self.rects.get(&id).is_some_and(|r| r.contains(pos))
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
    (c0.min(max_col), r0.min(max_row), c1.min(max_col), r1.min(max_row))
}

/// Brute-force linear scan (equivalent to old hit_test Phase 1).
/// Used only in tests to verify grid results.
#[cfg(test)]
fn linear_topmost(hittable: &[(u64, Rectangle)], pos: Point) -> Option<u64> {
    hittable.iter().rev().find_map(|(id, rect)| {
        if rect.contains(pos) { Some(*id) } else { None }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Size;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rectangle {
        Rectangle::new(Point::new(x, y), Size::new(w, h))
    }

    #[test]
    fn topmost_matches_linear_scan() {
        // 3 overlapping frames at different strata (sorted low→high).
        let hittable = vec![
            (1, rect(0.0, 0.0, 200.0, 200.0)),   // low strata, big
            (2, rect(50.0, 50.0, 100.0, 100.0)),  // mid strata, overlaps
            (3, rect(80.0, 80.0, 40.0, 40.0)),    // high strata, small
        ];
        let grid = HitGrid::new(hittable.clone(), 256.0, 256.0);

        // Points that should hit different frames.
        let cases = [
            (Point::new(10.0, 10.0), Some(1)),    // only frame 1
            (Point::new(60.0, 60.0), Some(2)),    // frames 1+2, topmost=2
            (Point::new(90.0, 90.0), Some(3)),    // all three, topmost=3
            (Point::new(130.0, 130.0), Some(2)),   // frames 1+2 (3 ends at 120)
            (Point::new(180.0, 180.0), Some(1)),   // only frame 1
            (Point::new(250.0, 250.0), None),      // outside all
        ];
        for (pos, expected) in cases {
            let grid_result = grid.topmost_at(pos);
            let linear_result = linear_topmost(&hittable, pos);
            assert_eq!(grid_result, expected, "grid mismatch at {pos:?}");
            assert_eq!(grid_result, linear_result, "grid != linear at {pos:?}");
        }
    }

    #[test]
    fn frame_spanning_multiple_cells() {
        // One frame spanning several cells.
        let hittable = vec![
            (1, rect(10.0, 10.0, 200.0, 200.0)),
        ];
        let grid = HitGrid::new(hittable, 256.0, 256.0);

        // Test points in different cells within the frame.
        assert_eq!(grid.topmost_at(Point::new(20.0, 20.0)), Some(1));   // cell (0,0)
        assert_eq!(grid.topmost_at(Point::new(100.0, 100.0)), Some(1)); // cell (1,1)
        assert_eq!(grid.topmost_at(Point::new(200.0, 200.0)), Some(1)); // cell (3,3)
        // Just outside.
        assert_eq!(grid.topmost_at(Point::new(5.0, 5.0)), None);
    }

    #[test]
    fn contains_checks_rect() {
        let hittable = vec![
            (1, rect(100.0, 100.0, 50.0, 50.0)),
        ];
        let grid = HitGrid::new(hittable, 256.0, 256.0);

        assert!(grid.contains(1, Point::new(120.0, 120.0)));
        assert!(!grid.contains(1, Point::new(90.0, 90.0)));
        assert!(!grid.contains(999, Point::new(120.0, 120.0))); // unknown id
    }

    #[test]
    fn cell_boundary_frame() {
        // Frame exactly on cell boundary (64px).
        let hittable = vec![
            (1, rect(60.0, 60.0, 10.0, 10.0)),  // spans cells (0,0) and (1,1)
        ];
        let grid = HitGrid::new(hittable, 128.0, 128.0);

        assert_eq!(grid.topmost_at(Point::new(63.0, 63.0)), Some(1));
        assert_eq!(grid.topmost_at(Point::new(65.0, 65.0)), Some(1));
        assert_eq!(grid.topmost_at(Point::new(59.0, 59.0)), None);
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
        let grid = HitGrid::new(hittable.clone(), 1000.0, 400.0);

        // Check every frame is hittable at its center.
        for &(id, r) in &hittable {
            let center = Point::new(r.x + 5.0, r.y + 5.0);
            assert_eq!(grid.topmost_at(center), Some(id), "missed frame {id}");
        }

        // Check gaps between frames return None.
        assert_eq!(grid.topmost_at(Point::new(15.0, 5.0)), None);
    }
}
