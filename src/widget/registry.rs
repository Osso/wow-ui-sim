//! Global widget registry for tracking all widgets.

use super::Frame;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

/// Registry of all widgets in the UI.
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    /// Widgets by ID.
    widgets: HashMap<u64, Frame>,
    /// Widget IDs by name.
    names: HashMap<String, u64>,
    /// Frame IDs whose visual properties changed since last render.
    /// Checked and drained by the render loop.
    render_dirty_ids: RefCell<HashSet<u64>>,
    /// Reverse index: target_id → set of frame IDs anchored to it.
    anchor_dependents: HashMap<u64, HashSet<u64>>,
    /// Frames with `rect_dirty = true`, for fast lookup in `ensure_layout_rects`.
    rect_dirty_ids: HashSet<u64>,
    /// Frames with `layout_rect = None` that need layout computation.
    pending_layout_ids: HashSet<u64>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new widget.
    pub fn register(&mut self, widget: Frame) -> u64 {
        let id = widget.id;
        // Debug: check for re-registration that would lose children
        if let Some(existing) = self.widgets.get(&id)
            && !existing.children.is_empty() {
                eprintln!("[WARN] Re-registering widget id={} name={:?} which has {} children!",
                    id, existing.name, existing.children.len());
            }
        if let Some(ref name) = widget.name {
            self.names.insert(name.clone(), id);
        }
        if widget.layout_rect.is_none() {
            self.pending_layout_ids.insert(id);
        }
        self.widgets.insert(id, widget);
        id
    }

    /// Get a widget by ID.
    pub fn get(&self, id: u64) -> Option<&Frame> {
        self.widgets.get(&id)
    }

    /// Get a mutable widget by ID. Does not mark dirty.
    ///
    /// Use for non-visual mutations (event registration, attributes, input
    /// config, animation offsets, layout cache, parent-child bookkeeping).
    /// For visual mutations, use `get_mut_visual()` instead.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Frame> {
        self.widgets.get_mut(&id)
    }

    /// Get a mutable widget by ID and mark it visually dirty.
    ///
    /// Use when changing visual properties: texture, text, alpha, color,
    /// visibility, size, anchors, draw_layer, frame_strata, backdrop, etc.
    pub fn get_mut_visual(&mut self, id: u64) -> Option<&mut Frame> {
        self.render_dirty_ids.borrow_mut().insert(id);
        self.widgets.get_mut(&id)
    }

    /// Mark a frame as visually dirty (needs re-render).
    ///
    /// Call after changing visual properties: texture, text, alpha, color,
    /// visibility, size, anchors, tex_coords, atlas, blend_mode, vertex_color,
    /// nine_slice, backdrop, rotation, desaturated.
    pub fn mark_visual_dirty(&self, id: u64) {
        self.render_dirty_ids.borrow_mut().insert(id);
    }

    /// Mark all frames as visually dirty (e.g. after screen resize).
    pub fn mark_all_visual_dirty(&self) {
        // Insert a sentinel value that consumers check via has_dirty_frames().
        // Avoids iterating all 50K frames just to insert their IDs.
        self.render_dirty_ids.borrow_mut().insert(u64::MAX);
    }

    /// Get a widget by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Frame> {
        self.names.get(name).and_then(|id| self.widgets.get(id))
    }

    /// Get a widget ID by name.
    pub fn get_id_by_name(&self, name: &str) -> Option<u64> {
        self.names.get(name).copied()
    }

    /// Get all widgets registered for a specific event.
    pub fn get_event_listeners(&self, event: &str) -> Vec<u64> {
        self.widgets
            .values()
            .filter(|w| w.is_registered_for_event(event))
            .map(|w| w.id)
            .collect()
    }

    /// Add a child to a parent widget.
    pub fn add_child(&mut self, parent_id: u64, child_id: u64) {
        if let Some(parent) = self.widgets.get_mut(&parent_id) {
            parent.children.push(child_id);
        }
    }

    /// Iterate over all widget IDs.
    pub fn iter_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.widgets.keys().copied()
    }

    /// Clear all cached layout rects (e.g. after screen resize).
    pub fn clear_all_layout_rects(&mut self) {
        for (&id, frame) in self.widgets.iter_mut() {
            frame.layout_rect = None;
            self.pending_layout_ids.insert(id);
        }
        self.mark_all_visual_dirty();
    }

    /// Check whether any frames have been visually dirtied since last drain.
    pub fn has_dirty_frames(&self) -> bool {
        !self.render_dirty_ids.borrow().is_empty()
    }

    /// Drain and return the set of visually dirty frame IDs. Clears the set.
    pub fn take_render_dirty(&self) -> bool {
        let mut ids = self.render_dirty_ids.borrow_mut();
        let had_any = !ids.is_empty();
        ids.clear();
        had_any
    }

    /// Set a widget's visibility flag and mark it visually dirty.
    ///
    /// Prefer `SimState::set_frame_visible` which also updates the OnUpdate cache.
    pub fn set_visible(&mut self, id: u64, visible: bool) {
        if let Some(f) = self.widgets.get_mut(&id) {
            if f.visible != visible {
                f.visible = visible;
                self.mark_visual_dirty(id);
            }
        }
    }

    /// Check if a frame and all its ancestors are visible.
    ///
    /// Matches WoW's `IsVisible()` semantics: uses eagerly-propagated
    /// `effective_alpha` — a frame is visible when effective_alpha > 0 and
    /// its own `visible` flag is true.
    pub fn is_ancestor_visible(&self, id: u64) -> bool {
        self.widgets.get(&id)
            .is_some_and(|f| f.visible && f.effective_alpha > 0.0)
    }

    /// Recompute `effective_alpha` for a frame and propagate to all descendants.
    ///
    /// effective_alpha = parent_effective_alpha × own_alpha when visible,
    /// 0.0 when the frame itself is hidden.
    pub fn propagate_effective_alpha(&mut self, id: u64, parent_effective_alpha: f32) {
        let Some(f) = self.widgets.get_mut(&id) else { return };
        let eff = if f.visible { parent_effective_alpha * f.alpha } else { 0.0 };
        f.effective_alpha = eff;
        let children: Vec<u64> = f.children.clone();
        for child_id in children {
            self.propagate_effective_alpha(child_id, eff);
        }
    }

    /// Propagate effective_alpha for ALL frames from root. Called once at startup
    /// to initialize effective_alpha after all frames are created and parented.
    pub fn propagate_all_effective_alpha(&mut self) {
        let root_ids: Vec<u64> = self.widgets.keys().copied()
            .filter(|&id| self.widgets.get(&id).is_some_and(|f| f.parent_id.is_none()))
            .collect();
        for id in root_ids {
            self.propagate_effective_alpha(id, 1.0);
        }
    }

    /// Propagate effective_scale for ALL frames from root. Called once at startup.
    pub fn propagate_all_effective_scale(&mut self) {
        let root_ids: Vec<u64> = self.widgets.keys().copied()
            .filter(|&id| self.widgets.get(&id).is_some_and(|f| f.parent_id.is_none()))
            .collect();
        for id in root_ids {
            self.propagate_effective_scale(id, 1.0);
        }
    }

    /// Recompute `effective_scale` for a frame and propagate to all descendants.
    ///
    /// effective_scale = parent_effective_scale × own_scale.
    pub fn propagate_effective_scale(&mut self, id: u64, parent_effective_scale: f32) {
        let Some(f) = self.widgets.get_mut(&id) else { return };
        let eff = parent_effective_scale * f.scale;
        f.effective_scale = eff;
        let children: Vec<u64> = f.children.clone();
        for child_id in children {
            self.propagate_effective_scale(child_id, eff);
        }
    }

    /// Mark a frame and all its descendants as rect-dirty (for `IsRectValid()`).
    pub fn mark_rect_dirty_subtree(&mut self, id: u64) {
        if let Some(f) = self.widgets.get_mut(&id) {
            f.rect_dirty = true;
            self.rect_dirty_ids.insert(id);
            let children = f.children.clone();
            for child_id in children {
                self.mark_rect_dirty_subtree(child_id);
            }
        }
    }

    /// Clear rect-dirty on a single frame (after layout resolution).
    pub fn clear_rect_dirty(&mut self, id: u64) {
        if let Some(f) = self.widgets.get_mut(&id) {
            f.rect_dirty = false;
        }
        self.rect_dirty_ids.remove(&id);
    }

    /// Drain rect_dirty_ids, clearing dirty flags. Returns the set for callers that need it.
    pub fn drain_rect_dirty(&mut self) -> HashSet<u64> {
        let ids = std::mem::take(&mut self.rect_dirty_ids);
        for &id in &ids {
            if let Some(f) = self.widgets.get_mut(&id) {
                f.rect_dirty = false;
            }
        }
        ids
    }

    /// Drain pending_layout_ids (frames missing layout_rect).
    pub fn drain_pending_layout(&mut self) -> HashSet<u64> {
        std::mem::take(&mut self.pending_layout_ids)
    }

    /// Mark a frame's layout_rect as resolved (remove from pending set).
    pub fn mark_layout_resolved(&mut self, id: u64) {
        self.pending_layout_ids.remove(&id);
    }

    /// Check if setting a point from `frame_id` to `relative_to_id` would create a cycle.
    /// A cycle exists if relative_to (or any of its anchor dependencies) already
    /// depends on frame_id.
    pub fn would_create_anchor_cycle(&self, frame_id: u64, relative_to_id: u64) -> bool {
        // Can't anchor to yourself
        if frame_id == relative_to_id {
            return true;
        }

        // BFS from relative_to, checking if any dependency points back to frame_id
        let mut queue = VecDeque::new();
        let mut seen = HashSet::new();

        queue.push_back(relative_to_id);
        seen.insert(relative_to_id);

        while let Some(check_id) = queue.pop_front() {
            if let Some(frame) = self.widgets.get(&check_id) {
                for anchor in &frame.anchors {
                    if let Some(anchor_target) = anchor.relative_to_id {
                        let target_id = anchor_target as u64;
                        if target_id == frame_id {
                            return true;
                        }
                        if seen.insert(target_id) {
                            queue.push_back(target_id);
                        }
                    }
                }
            }
        }

        false
    }

    /// Record that `frame_id` is anchored to `target_id`.
    pub fn add_anchor_dependent(&mut self, target_id: u64, frame_id: u64) {
        self.anchor_dependents.entry(target_id).or_default().insert(frame_id);
    }

    /// Remove `frame_id` from `target_id`'s dependents.
    pub fn remove_anchor_dependent(&mut self, target_id: u64, frame_id: u64) {
        if let Some(set) = self.anchor_dependents.get_mut(&target_id) {
            set.remove(&frame_id);
            if set.is_empty() {
                self.anchor_dependents.remove(&target_id);
            }
        }
    }

    /// Remove `frame_id` from all reverse-index entries by reading its current
    /// anchors to find the targets.
    pub fn remove_all_anchor_dependents_for(&mut self, frame_id: u64) {
        let targets: Vec<u64> = self.widgets.get(&frame_id)
            .map(|f| f.anchors.iter()
                .filter_map(|a| a.relative_to_id.map(|t| t as u64))
                .collect())
            .unwrap_or_default();
        for target in targets {
            self.remove_anchor_dependent(target, frame_id);
        }
    }

    /// Get frame IDs anchored to `target_id`.
    pub fn get_anchor_dependents(&self, target_id: u64) -> Option<&HashSet<u64>> {
        self.anchor_dependents.get(&target_id)
    }

    /// Rebuild the reverse anchor index from all existing anchors.
    /// Call once after initial load to capture anchors set during XML parsing
    /// and frame creation.
    pub fn rebuild_anchor_index(&mut self) {
        self.anchor_dependents.clear();
        let entries: Vec<(u64, u64)> = self.widgets.values()
            .flat_map(|f| {
                f.anchors.iter().filter_map(move |a| {
                    a.relative_to_id.map(|target| (target as u64, f.id))
                })
            })
            .collect();
        for (target, frame_id) in entries {
            self.anchor_dependents.entry(target).or_default().insert(frame_id);
        }
    }
}
