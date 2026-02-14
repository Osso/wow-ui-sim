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
    let c0 = (rect.x / CELL_SIZE) as usize;
    let r0 = (rect.y / CELL_SIZE) as usize;
    let c1 = ((rect.x + rect.width) / CELL_SIZE) as usize;
    let r1 = ((rect.y + rect.height) / CELL_SIZE) as usize;
    (
        c0.min(cols.saturating_sub(1)),
        r0.min(rows.saturating_sub(1)),
        c1.min(cols.saturating_sub(1)),
        r1.min(rows.saturating_sub(1)),
    )
}
