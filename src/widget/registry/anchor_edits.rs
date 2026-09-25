//! Deferred render invalidation for anchor edits.
//!
//! Blizzard layout code often re-anchors frames to where they already are
//! (`ClearAllPoints()` then the same `SetPoint()`), e.g. every OnUpdate while
//! a layout frame stays dirty. Anchor setters therefore record the frame's
//! anchors and rect before its first edit, and the frame is only marked
//! visually dirty when the dirty set is read and the edit changed something.

use super::WidgetRegistry;
use crate::LayoutRect;
use crate::widget::{Anchor, AnchorPoint, Frame};

#[derive(Debug, Clone)]
pub(super) struct AnchorEditBaseline {
    anchors: Vec<Anchor>,
    secret_anchor_points: Vec<AnchorPoint>,
    layout_rect: Option<LayoutRect>,
}

impl AnchorEditBaseline {
    fn capture(frame: &Frame) -> Self {
        Self {
            anchors: frame.anchors.clone(),
            secret_anchor_points: frame.secret_anchor_points.clone(),
            layout_rect: frame.layout_rect,
        }
    }
}

impl WidgetRegistry {
    /// Canvas size used to resolve rects when settling anchor edits.
    pub(crate) fn set_layout_canvas_size(&mut self, width: f32, height: f32) {
        self.layout_canvas_size = Some((width, height));
    }

    /// Mutable access for an anchor edit. Records the pre-edit anchors and rect
    /// instead of marking the frame visually dirty; see [`Self::mark_anchor_rect_dirty`].
    pub fn get_mut_for_anchor_edit(&mut self, id: u64) -> Option<&mut Frame> {
        let frame = self.widgets.get(&id)?;
        self.anchor_edit_baselines
            .get_mut()
            .entry(id)
            .or_insert_with(|| AnchorEditBaseline::capture(frame));
        self.widgets.get_mut(&id)
    }

    /// Queue layout for an anchor edit without marking it visually dirty.
    pub fn mark_anchor_rect_dirty(&mut self, id: u64) {
        if self.widgets.contains_key(&id) {
            self.rect_dirty_ids.insert(id);
            self.hit_grid_dirty_ids.insert(id);
        }
    }

    /// Mark anchor-edited frames visually dirty when their anchors or resolved
    /// rect differ from before the first edit.
    pub(super) fn settle_anchor_edits(&self) {
        let baselines = std::mem::take(&mut *self.anchor_edit_baselines.borrow_mut());
        for (id, baseline) in baselines {
            if self.anchor_edit_changed_frame(id, &baseline) {
                self.record_visual_dirty(id);
            }
        }
    }

    fn anchor_edit_changed_frame(&self, id: u64, baseline: &AnchorEditBaseline) -> bool {
        let Some(frame) = self.widgets.get(&id) else {
            return false;
        };
        let anchors_changed = frame.anchors != baseline.anchors
            || frame.secret_anchor_points != baseline.secret_anchor_points;
        anchors_changed || self.resolved_rect_changed(id, baseline.layout_rect)
    }

    /// An unchanged anchor list can still move the frame when an anchor
    /// target or ancestor moved, so compare a freshly resolved rect.
    fn resolved_rect_changed(&self, id: u64, baseline_rect: Option<LayoutRect>) -> bool {
        let (Some(baseline_rect), Some((width, height))) = (baseline_rect, self.layout_canvas_size)
        else {
            return true;
        };
        crate::layout::compute_frame_rect(self, id, width, height) != baseline_rect
    }
}
