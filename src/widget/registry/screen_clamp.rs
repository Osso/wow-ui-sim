//! R88n.5r geometry-local clamp state and ordered anchor propagation.
use super::WidgetRegistry;
use rustc_hash::FxHashSet;

impl WidgetRegistry {
    /// Adopt a requested value only for a changed, unguarded region. Traverse
    /// dependents depth-first in stored insertion order, then write suppression.
    pub(crate) fn request_anchor_propagation(&mut self, id: u64, requested: bool) {
        use crate::widget::clamp_geometry::uses_retail_clamp_producers;
        if !uses_retail_clamp_producers() || !self.adopt_anchor_propagation(id, requested) {
            return;
        }

        let mut pending = Vec::new();
        self.push_anchor_propagation_children(id, &mut pending);
        while let Some((id, suppress)) = pending.pop() {
            if suppress {
                self.set_screen_clamp_suppressed(id, requested);
            } else if self.adopt_anchor_propagation(id, requested) {
                self.push_anchor_propagation_children(id, &mut pending);
            }
        }
    }

    fn adopt_anchor_propagation(&mut self, id: u64, requested: bool) -> bool {
        let Some(frame) = self.widgets.get_mut(&id) else {
            return false;
        };
        if frame.anchor_propagation_guard || frame.anchor_propagation == requested {
            return false;
        }
        frame.anchor_propagation = requested;
        true
    }

    fn push_anchor_propagation_children(&self, id: u64, pending: &mut Vec<(u64, bool)>) {
        pending.push((id, true));
        if let Some(dependents) = self.ordered_anchor_dependents.get(&id) {
            pending.extend(dependents.iter().rev().map(|id| (*id, false)));
        }
    }

    /// Verified Boolean suppression writer. A dirty root is already queued in
    /// the registry's set-backed layout queue; do not enqueue it twice.
    pub fn set_screen_clamp_suppressed(&mut self, id: u64, suppressed: bool) {
        let Some(frame) = self.widgets.get_mut(&id) else {
            return;
        };
        frame.screen_clamp_suppressed = suppressed;
        self.queue_clamp_geometry(id);
    }

    /// Pre-store notification; never touches the layout or rectangle queue.
    pub(crate) fn notify_clamp_inset_secret_value(&mut self, id: u64) {
        let Some(frame) = self.widgets.get_mut(&id) else {
            return;
        };
        if frame.clamp_secret_notified {
            return;
        }
        frame.clamp_secret_notified = true;

        // The pinned native answer establishes geometry propagation for the
        // GameTooltip callback, not the callbacks of every other Frame type.
        if frame.widget_type != crate::widget::WidgetType::GameTooltip {
            return;
        }
        let mut pending = vec![id];
        while let Some(region_id) = pending.pop() {
            if self.mark_clamp_notification_geometry(region_id)
                && let Some(dependents) = self.ordered_anchor_dependents.get(&region_id)
            {
                pending.extend(dependents.iter().rev().copied());
            }
        }
    }

    fn mark_clamp_notification_geometry(&mut self, id: u64) -> bool {
        let Some(frame) = self.widgets.get_mut(&id) else {
            return false;
        };
        if frame.clamp_geometry_notified {
            return false;
        }
        frame.clamp_geometry_notified = true;
        if frame.widget_type == crate::widget::WidgetType::GameTooltip
            && frame.clamp_notification_ancestor_eligible
            && frame.clamp_notification_ancestor_gate
        {
            let ancestor = frame.parent_id;
            self.mark_clamp_boundary_parent(ancestor);
        }
        true
    }

    fn mark_clamp_boundary_parent(&mut self, mut ancestor: Option<u64>) {
        while let Some(region) = ancestor.and_then(|id| self.widgets.get(&id)) {
            ancestor = region.parent_id;
            if region.clamp_notification_boundary {
                if let Some(parent) = ancestor.and_then(|id| self.widgets.get_mut(&id)) {
                    parent.clamp_ancestor_notified = true;
                }
                return;
            }
        }
    }

    /// Accepted enable change compares against effective, not stored, state.
    pub fn set_screen_clamp_enabled(&mut self, id: u64, enabled: bool) {
        let Some(frame) = self.widgets.get_mut(&id) else {
            return;
        };
        if frame.effective_screen_clamp() == enabled {
            return;
        }
        frame.clamped_to_screen = enabled;
        self.queue_clamp_geometry(id);
    }

    fn queue_clamp_geometry(&mut self, id: u64) {
        let mut pending = vec![id];
        let mut seen = FxHashSet::default();
        while let Some(current) = pending.pop() {
            if !seen.insert(current) {
                continue;
            }
            // A dirty root from another operation need not have invalidated
            // anchor dependents yet. Deduplicate each node, not the traversal.
            if !self.rect_dirty_ids.contains(&current) {
                self.mark_rect_dirty(current);
            }
            if let Some(dependents) = self.anchor_dependents.get(&current) {
                pending.extend(dependents.iter().copied());
            }
        }
    }
}
