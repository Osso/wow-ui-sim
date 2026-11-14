//! R88n.5r geometry-local clamp state. Unknown anchor propagation predicates
//! remain host-supplied; movement flags are not aliases for suppression.
use super::WidgetRegistry;
use rustc_hash::FxHashSet;

impl WidgetRegistry {
    /// Verified Boolean suppression writer. A dirty root is already queued in
    /// the registry's set-backed layout queue; do not enqueue it twice.
    pub fn set_screen_clamp_suppressed(&mut self, id: u64, suppressed: bool) {
        let Some(frame) = self.widgets.get_mut(&id) else {
            return;
        };
        frame.screen_clamp_suppressed = suppressed;
        self.queue_clamp_geometry(id);
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
