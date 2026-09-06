//! Global widget registry for tracking all widgets.

mod anchor;
mod pixel_scale;
mod storage;

use super::Frame;
pub use anchor::AnchorCyclePath;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderDirtySource {
    pub frame_id: u64,
    pub method: &'static str,
}

#[derive(Debug, Default)]
pub struct RenderDirtyBatch {
    pub strata_mask: u16,
    pub frame_ids: Option<FxHashSet<u64>>,
    pub sources: FxHashMap<u64, FxHashSet<RenderDirtySource>>,
}

/// Registry of all widgets in the UI.
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    layout_pixel_scale: pixel_scale::LayoutPixelScale,
    /// Widgets by ID.
    pub(super) widgets: FxHashMap<u64, Frame>,
    /// Widget IDs by name.
    names: FxHashMap<String, u64>,
    /// Widget IDs in creation order (monotonically increasing, always sorted).
    ordered_ids: Vec<u64>,
    /// Existing parent-child edges for O(1) duplicate checks.
    child_links: FxHashSet<(u64, u64)>,
    /// Reverse event index for frames registered for a specific event.
    event_listeners: FxHashMap<String, FxHashSet<u64>>,
    /// Reverse event index for frames registered for every event.
    all_event_listeners: FxHashSet<u64>,
    /// Frame IDs whose visual properties changed since last render.
    render_dirty_ids: RefCell<FxHashSet<u64>>,
    /// Dirty provenance captured while a specific script/method is running.
    render_dirty_sources: RefCell<FxHashMap<u64, FxHashSet<RenderDirtySource>>>,
    /// Current source context for dirty attribution.
    current_dirty_source: RefCell<Option<RenderDirtySource>>,
    /// Reverse index: target_id -> set of frame IDs anchored to it.
    pub(super) anchor_dependents: FxHashMap<u64, FxHashSet<u64>>,
    /// Forward index: frame_id -> set of target IDs it is anchored to.
    pub(super) frame_anchor_targets: FxHashMap<u64, FxHashSet<u64>>,
    /// Frames with `rect_dirty = true`, for fast lookup in `ensure_layout_rects`.
    rect_dirty_ids: FxHashSet<u64>,
    /// Layout roots whose hit-grid entries must be repaired even after layout resolves.
    hit_grid_dirty_ids: FxHashSet<u64>,
    /// Frames with `layout_rect = None` that need layout computation.
    pending_layout_ids: FxHashSet<u64>,
}

impl WidgetRegistry {
    /// Pre-allocated capacity for typical Blizzard UI load (~45k frames measured
    /// 2026-04 via `dump-tree` with --no-addons --no-saved-vars). Sized to avoid
    /// the 3k→6k→12k→24k→48k rehash cascade that dominated startup page faults.
    const INITIAL_CAPACITY: usize = 65536;

    pub fn new() -> Self {
        Self {
            layout_pixel_scale: pixel_scale::LayoutPixelScale::default(),
            widgets: FxHashMap::with_capacity_and_hasher(
                Self::INITIAL_CAPACITY,
                Default::default(),
            ),
            names: FxHashMap::with_capacity_and_hasher(Self::INITIAL_CAPACITY, Default::default()),
            ordered_ids: Vec::with_capacity(Self::INITIAL_CAPACITY),
            child_links: Self::initial_id_pair_set(),
            event_listeners: FxHashMap::with_capacity_and_hasher(512, Default::default()),
            all_event_listeners: FxHashSet::with_capacity_and_hasher(64, Default::default()),
            render_dirty_ids: RefCell::new(FxHashSet::with_capacity_and_hasher(
                256,
                Default::default(),
            )),
            render_dirty_sources: RefCell::new(FxHashMap::with_capacity_and_hasher(
                64,
                Default::default(),
            )),
            current_dirty_source: RefCell::new(None),
            anchor_dependents: FxHashMap::with_capacity_and_hasher(
                Self::INITIAL_CAPACITY,
                Default::default(),
            ),
            frame_anchor_targets: FxHashMap::with_capacity_and_hasher(
                Self::INITIAL_CAPACITY,
                Default::default(),
            ),
            rect_dirty_ids: Self::initial_id_set(),
            hit_grid_dirty_ids: Self::initial_id_set(),
            pending_layout_ids: Self::initial_id_set(),
        }
    }

    fn initial_id_set() -> FxHashSet<u64> {
        FxHashSet::with_capacity_and_hasher(Self::INITIAL_CAPACITY, Default::default())
    }

    fn initial_id_pair_set() -> FxHashSet<(u64, u64)> {
        FxHashSet::with_capacity_and_hasher(Self::INITIAL_CAPACITY, Default::default())
    }

    /// Register a new widget.
    pub fn register(&mut self, widget: Frame) -> u64 {
        let id = widget.id;
        let is_new = !self.widgets.contains_key(&id);
        // Debug: check for re-registration that would lose children
        if let Some(existing) = self.widgets.get(&id)
            && !existing.children.is_empty()
        {
            eprintln!(
                "[WARN] Re-registering widget id={} name={:?} which has {} children!",
                id,
                existing.name,
                existing.children.len()
            );
        }
        if let Some(ref name) = widget.name {
            self.names.insert(name.clone(), id);
        }
        if widget.layout_rect.is_none() {
            self.pending_layout_ids.insert(id);
        }
        if !is_new && let Some(existing) = self.widgets.get(&id) {
            for &child_id in &existing.children {
                self.child_links.remove(&(id, child_id));
            }
        }
        for &child_id in &widget.children {
            self.child_links.insert((id, child_id));
        }
        self.widgets.insert(id, widget);
        if is_new {
            self.ordered_ids.push(id);
        }
        id
    }

    pub fn register_preserving_existing_name(&mut self, widget: Frame) -> u64 {
        let existing_name_binding = widget
            .name
            .as_ref()
            .and_then(|name| self.names.get(name).map(|id| (name.clone(), *id)));
        let id = self.register(widget);
        if let Some((name, existing_id)) = existing_name_binding {
            self.names.insert(name, existing_id);
        }
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
        self.record_visual_dirty(id);
        self.widgets.get_mut(&id)
    }

    /// Mark a frame as visually dirty (needs re-render).
    ///
    /// Call after changing visual properties: texture, text, alpha, color,
    /// visibility, size, anchors, tex_coords, atlas, blend_mode, vertex_color,
    /// nine_slice, backdrop, rotation, desaturated.
    pub fn mark_visual_dirty(&self, id: u64) {
        self.record_visual_dirty(id);
    }

    /// Mark all frames as visually dirty (e.g. after screen resize).
    pub fn mark_all_visual_dirty(&self) {
        // Insert a sentinel value that consumers check via has_dirty_frames().
        // Avoids iterating all 50K frames just to insert their IDs.
        self.render_dirty_ids.borrow_mut().insert(u64::MAX);
    }

    pub fn set_render_dirty_source(&self, source: Option<RenderDirtySource>) {
        *self.current_dirty_source.borrow_mut() = source;
    }

    /// Set (or update) the name of a widget, updating the names index.
    pub fn set_name(&mut self, id: u64, name: String) {
        self.names.insert(name.clone(), id);
        if let Some(frame) = self.widgets.get_mut(&id) {
            frame.name = Some(name);
        }
    }

    /// Get a widget by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Frame> {
        self.names.get(name).and_then(|id| self.widgets.get(id))
    }

    /// Get a widget ID by name.
    pub fn get_id_by_name(&self, name: &str) -> Option<u64> {
        self.names.get(name).copied()
    }

    /// Iterate all (id, name) pairs in the registry.
    pub fn named_frames(&self) -> impl Iterator<Item = (u64, &String)> {
        self.names.iter().map(|(name, &id)| (id, name))
    }

    /// Get all widgets registered for a specific event.
    /// Individual RegisterEvent listeners fire before RegisterAllEvents listeners.
    /// Within each group, frames fire in creation order (ascending ID).
    pub fn get_event_listeners(&self, event: &str) -> Vec<u64> {
        let mut individual = self
            .event_listeners
            .get(event)
            .map(|listeners| listeners.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut all_events = self
            .all_event_listeners
            .iter()
            .copied()
            .filter(|id| !individual.contains(id))
            .collect::<Vec<_>>();
        individual.sort_unstable();
        all_events.sort_unstable();
        individual.extend(all_events);
        individual
    }

    pub fn register_event_listener(&mut self, id: u64, event: &str) -> bool {
        let was_inserted = self
            .widgets
            .get_mut(&id)
            .map(|frame| frame.registered_events.insert(event.to_string()))
            .unwrap_or(false);
        if was_inserted {
            self.event_listeners
                .entry(event.to_string())
                .or_default()
                .insert(id);
        }
        was_inserted
    }

    pub fn register_unit_event_listener(&mut self, id: u64, event: &str, unit: &str) -> bool {
        let was_inserted = self
            .widgets
            .get_mut(&id)
            .map(|frame| {
                frame
                    .registered_unit_events
                    .insert(event.to_string(), unit.to_string());
                frame.registered_events.insert(event.to_string())
            })
            .unwrap_or(false);
        if was_inserted {
            self.event_listeners
                .entry(event.to_string())
                .or_default()
                .insert(id);
        }
        was_inserted
    }

    pub fn unregister_event_listener(&mut self, id: u64, event: &str) -> bool {
        let was_removed = self
            .widgets
            .get_mut(&id)
            .map(|frame| {
                frame.registered_unit_events.remove(event);
                frame.registered_events.remove(event)
            })
            .unwrap_or(false);
        if was_removed {
            self.remove_event_listener_index(id, event);
        }
        was_removed
    }

    pub fn unregister_all_event_listeners(&mut self, id: u64) {
        let registered_events = self
            .widgets
            .get_mut(&id)
            .map(|frame| {
                let registered_events = frame.registered_events.iter().cloned().collect::<Vec<_>>();
                frame.registered_events.clear();
                frame.registered_unit_events.clear();
                frame.register_all_events = false;
                registered_events
            })
            .unwrap_or_default();
        for event in registered_events {
            self.remove_event_listener_index(id, &event);
        }
        self.all_event_listeners.remove(&id);
    }

    pub fn register_all_event_listener(&mut self, id: u64) {
        if let Some(frame) = self.widgets.get_mut(&id) {
            frame.register_all_events = true;
            self.all_event_listeners.insert(id);
        }
    }

    fn remove_event_listener_index(&mut self, id: u64, event: &str) {
        if let Some(listeners) = self.event_listeners.get_mut(event) {
            listeners.remove(&id);
            if listeners.is_empty() {
                self.event_listeners.remove(event);
            }
        }
    }

    /// Add a child to a parent widget.
    pub fn add_child(&mut self, parent_id: u64, child_id: u64) {
        let (parent_eff_alpha, parent_eff_scale) = self
            .widgets
            .get(&parent_id)
            .map(|p| (p.effective_alpha, p.effective_scale))
            .unwrap_or((1.0, 1.0));
        let old_parent_id = self
            .widgets
            .get(&child_id)
            .and_then(|child| child.parent_id);
        if old_parent_id != Some(parent_id) {
            if let Some(old_parent_id) = old_parent_id {
                self.child_links.remove(&(old_parent_id, child_id));
                if let Some(old_parent) = self.widgets.get_mut(&old_parent_id) {
                    old_parent.children.retain(|&id| id != child_id);
                    old_parent
                        .children_keys
                        .retain(|_, mapped_id| *mapped_id != child_id);
                }
            }
            if let Some(child) = self.widgets.get_mut(&child_id) {
                child.parent_id = Some(parent_id);
            }
        }
        if let Some(parent) = self.widgets.get_mut(&parent_id) {
            if self.child_links.insert((parent_id, child_id)) {
                parent.children.push(child_id);
            }
        }
        self.propagate_effective_alpha(child_id, parent_eff_alpha);
        self.propagate_effective_scale(child_id, parent_eff_scale);
    }

    /// Iterate over all widget IDs.
    pub fn iter_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.widgets.keys().copied()
    }

    pub fn storage_estimate_bytes(&self) -> usize {
        storage::registry_storage_estimate_bytes(self)
    }

    /// Collect unique texture paths from all visible frames.
    pub fn visible_texture_paths(&self) -> Vec<String> {
        let mut paths = FxHashSet::default();
        for frame in self.widgets.values() {
            if !self.is_ancestor_visible(frame.id) {
                continue;
            }
            for path in [
                &frame.texture,
                &frame.normal_texture,
                &frame.pushed_texture,
                &frame.highlight_texture,
                &frame.disabled_texture,
            ]
            .into_iter()
            .flatten()
            {
                paths.insert(path.clone());
            }
        }
        paths.into_iter().collect()
    }

    pub fn next_enumerable_id_after(&self, after_id: u64) -> Option<u64> {
        let ids = &self.ordered_ids[self.ordered_ids.partition_point(|&id| id <= after_id)..];
        ids.iter()
            .copied()
            .find(|id| self.widgets.get(id).is_some_and(|frame| !frame.forbidden))
    }

    /// Clear all cached layout rects (e.g. after screen resize).
    pub fn clear_all_layout_rects(&mut self) {
        self.pending_layout_ids.clear();
        for (&id, frame) in self.widgets.iter_mut() {
            frame.layout_rect = None;
            if frame.parent_id.is_none() {
                self.pending_layout_ids.insert(id);
            }
        }
        self.mark_all_visual_dirty();
    }

    /// Check whether any frames have been visually dirtied since last drain.
    pub fn has_dirty_frames(&self) -> bool {
        !self.render_dirty_ids.borrow().is_empty()
    }

    /// Drain the set of visually dirty frame IDs and return a per-strata
    /// bitmask indicating which strata contain dirty frames.
    ///
    /// Bit `i` is set when at least one dirty frame lives in strata index `i`.
    /// The sentinel `u64::MAX` (from `mark_all_visual_dirty`) produces the
    /// all-strata mask `(1 << COUNT) - 1`.
    pub fn take_render_dirty(&self) -> u16 {
        self.take_render_dirty_batch().strata_mask
    }

    /// Drain the dirty set, returning both the strata bitmask and the set of
    /// dirty frame IDs. Returns `None` for the ID set when the sentinel
    /// (`u64::MAX`) was present, signalling that a full rebuild is needed.
    pub fn take_render_dirty_with_ids(&self) -> (u16, Option<FxHashSet<u64>>) {
        let batch = self.take_render_dirty_batch();
        (batch.strata_mask, batch.frame_ids)
    }

    pub fn take_render_dirty_batch(&self) -> RenderDirtyBatch {
        let mut ids = self.render_dirty_ids.borrow_mut();
        if ids.is_empty() {
            return self.empty_render_dirty_batch();
        }

        self.build_render_dirty_batch(&mut ids)
    }

    fn build_render_dirty_batch(&self, ids: &mut FxHashSet<u64>) -> RenderDirtyBatch {
        let (strata_mask, has_sentinel) = self.render_dirty_mask(ids);
        let frame_ids = Self::drain_render_dirty_ids(ids, has_sentinel);

        RenderDirtyBatch {
            strata_mask,
            frame_ids,
            sources: self.take_render_dirty_sources(),
        }
    }

    fn empty_render_dirty_batch(&self) -> RenderDirtyBatch {
        RenderDirtyBatch {
            strata_mask: 0,
            frame_ids: Some(FxHashSet::default()),
            sources: FxHashMap::default(),
        }
    }

    fn render_dirty_mask(&self, ids: &FxHashSet<u64>) -> (u16, bool) {
        let all_mask = (1u16 << super::FrameStrata::COUNT) - 1;
        let has_sentinel = ids.contains(&u64::MAX);
        if has_sentinel {
            return (all_mask, true);
        }

        let mut mask: u16 = 0;
        for &id in ids {
            mask |= self.strata_bit_for(id);
            if mask == all_mask {
                break;
            }
        }
        (mask, false)
    }

    fn drain_render_dirty_ids(
        ids: &mut FxHashSet<u64>,
        has_sentinel: bool,
    ) -> Option<FxHashSet<u64>> {
        if has_sentinel {
            ids.clear();
            None
        } else {
            Some(std::mem::take(ids))
        }
    }

    fn take_render_dirty_sources(&self) -> FxHashMap<u64, FxHashSet<RenderDirtySource>> {
        std::mem::take(&mut *self.render_dirty_sources.borrow_mut())
    }

    /// Return the strata bitmask for a single frame ID.
    ///
    /// Regions (Texture, FontString, Line) use their parent's strata.
    fn strata_bit_for(&self, id: u64) -> u16 {
        let Some(f) = self.widgets.get(&id) else {
            return 0;
        };
        let strata = match f.widget_type {
            super::WidgetType::Texture
            | super::WidgetType::FontString
            | super::WidgetType::Line => f
                .parent_id
                .and_then(|pid| self.widgets.get(&pid))
                .map(|p| p.frame_strata)
                .unwrap_or(f.frame_strata),
            _ => f.frame_strata,
        };
        1u16 << strata.as_index()
    }

    /// Set a widget's visibility flag and mark it visually dirty.
    ///
    /// Prefer `SimState::set_frame_visible` which also updates the OnUpdate cache.
    pub fn set_visible(&mut self, id: u64, visible: bool) {
        if let Some(f) = self.widgets.get_mut(&id)
            && f.visible != visible
        {
            f.visible = visible;
            self.mark_visual_dirty(id);
        }
    }

    /// Check if a frame and all its ancestors are visible (shown).
    ///
    /// Matches WoW's `IsVisible()` semantics: a frame is visible when its
    /// own `visible` flag is true AND all ancestors are visible. Alpha does
    /// NOT affect visibility — a frame with alpha=0 is still "visible" and
    /// receives OnUpdate, events, etc.
    pub fn is_ancestor_visible(&self, id: u64) -> bool {
        let mut current_id = id;
        loop {
            let Some(f) = self.widgets.get(&current_id) else {
                return false;
            };
            if !f.visible {
                return false;
            }
            match f.parent_id {
                Some(parent_id) => {
                    current_id = parent_id;
                }
                None => return true,
            }
        }
    }

    /// Recompute `effective_alpha` for a frame and propagate to all descendants.
    ///
    /// Visible frames inherit their parent's effective alpha unless they
    /// explicitly ignore it. Hidden frames always resolve to 0.0.
    /// Also marks frames as visually dirty when their effective_alpha changes,
    /// so cached quad snapshots with baked-in alpha are invalidated.
    pub fn propagate_effective_alpha(&mut self, id: u64, parent_effective_alpha: f32) {
        let Some(f) = self.widgets.get_mut(&id) else {
            return;
        };
        let eff = if f.visible {
            if f.ignore_parent_alpha {
                f.alpha
            } else {
                parent_effective_alpha * f.alpha
            }
        } else {
            0.0
        };
        let became_dirty = (eff - f.effective_alpha).abs() > f32::EPSILON;
        f.effective_alpha = eff;
        let children: Vec<u64> = f.children.clone();
        let _ = f;
        if became_dirty {
            self.record_visual_dirty(id);
        }
        for child_id in children {
            self.propagate_effective_alpha(child_id, eff);
        }
    }

    /// Propagate effective_alpha for ALL frames from root. Called once at startup
    /// to initialize effective_alpha after all frames are created and parented.
    pub fn propagate_all_effective_alpha(&mut self) {
        self.propagate_all_roots(|registry, id| registry.propagate_effective_alpha(id, 1.0));
    }

    /// Propagate effective_scale for ALL frames from root. Called once at startup.
    pub fn propagate_all_effective_scale(&mut self) {
        self.propagate_all_roots(|registry, id| registry.propagate_effective_scale(id, 1.0));
    }

    fn propagate_all_roots(&mut self, mut propagate: impl FnMut(&mut Self, u64)) {
        let root_ids: Vec<u64> = self
            .widgets
            .keys()
            .copied()
            .filter(|&id| self.widgets.get(&id).is_some_and(|f| f.parent_id.is_none()))
            .collect();
        for id in root_ids {
            propagate(self, id);
        }
    }

    /// Recompute `effective_scale` for a frame and propagate to all descendants.
    ///
    /// Frames that ignore parent scale use only their own scale. All other
    /// frames inherit the parent's effective scale.
    pub fn propagate_effective_scale(&mut self, id: u64, parent_effective_scale: f32) {
        let Some(f) = self.widgets.get_mut(&id) else {
            return;
        };
        let eff = if f.ignore_parent_scale {
            f.scale
        } else {
            parent_effective_scale * f.scale
        };
        let became_dirty = (eff - f.effective_scale).abs() > f32::EPSILON;
        f.effective_scale = eff;
        let children: Vec<u64> = f.children.clone();
        let _ = f;
        if became_dirty {
            self.record_visual_dirty(id);
        }
        for child_id in children {
            self.propagate_effective_scale(child_id, eff);
        }
    }

    /// Mark a frame as rect-dirty root. O(1), no subtree walk.
    /// Descendants discover dirtiness lazily via `is_rect_dirty` ancestor walk.
    pub fn mark_rect_dirty(&mut self, id: u64) {
        if self.widgets.contains_key(&id) {
            self.rect_dirty_ids.insert(id);
            self.hit_grid_dirty_ids.insert(id);
            self.record_visual_dirty(id);
        }
    }

    pub fn drain_hit_grid_dirty(&mut self) -> FxHashSet<u64> {
        std::mem::take(&mut self.hit_grid_dirty_ids)
    }

    /// Whether the next layout pass can change one or more cached frame rects.
    pub fn has_pending_layout_work(&self) -> bool {
        !self.pending_layout_ids.is_empty() || !self.rect_dirty_ids.is_empty()
    }

    /// Snapshot pending layout roots before a layout pass drains them.
    pub fn pending_layout_roots(&self) -> Vec<u64> {
        let mut roots: FxHashSet<u64> = self.pending_layout_ids.clone();
        roots.extend(self.rect_dirty_ids.iter().copied());
        self.prune_descendant_layout_roots(roots)
    }

    fn prune_descendant_layout_roots(&self, roots: FxHashSet<u64>) -> Vec<u64> {
        let mut pruned: Vec<u64> = roots
            .iter()
            .copied()
            .filter(|id| !self.has_layout_root_ancestor(*id, &roots))
            .collect();
        pruned.sort_unstable();
        pruned
    }

    fn has_layout_root_ancestor(&self, id: u64, roots: &FxHashSet<u64>) -> bool {
        let mut current = self.widgets.get(&id).and_then(|frame| frame.parent_id);
        while let Some(parent_id) = current {
            if roots.contains(&parent_id) {
                return true;
            }
            current = self
                .widgets
                .get(&parent_id)
                .and_then(|frame| frame.parent_id);
        }
        false
    }

    pub fn is_rect_dirty_self(&self, id: u64) -> bool {
        self.rect_dirty_ids.contains(&id)
    }

    /// Check if a frame or any ancestor is rect-dirty.
    pub fn is_rect_dirty(&self, id: u64) -> bool {
        let mut current = Some(id);
        while let Some(cid) = current {
            if self.rect_dirty_ids.contains(&cid) {
                return true;
            }
            current = self.widgets.get(&cid).and_then(|f| f.parent_id);
        }
        false
    }

    /// Walk up the parent chain and collect all frame IDs (including `id`)
    /// that are in `rect_dirty_ids`. Returns in bottom-up order.
    pub fn collect_dirty_ancestor_roots(&self, id: u64) -> Vec<u64> {
        let mut roots = Vec::new();
        let mut current = Some(id);
        while let Some(cid) = current {
            if self.is_rect_dirty_self(cid) {
                roots.push(cid);
            }
            current = self.widgets.get(&cid).and_then(|f| f.parent_id);
        }
        roots
    }

    pub fn clear_rect_dirty(&mut self, id: u64) {
        self.rect_dirty_ids.remove(&id);
    }

    pub fn clear_rect_dirty_subtree(&mut self, id: u64) {
        self.rect_dirty_ids.remove(&id);
        let children = self
            .widgets
            .get(&id)
            .map(|f| f.children.clone())
            .unwrap_or_default();
        for child_id in children {
            self.clear_rect_dirty_subtree(child_id);
        }
    }

    pub fn drain_rect_dirty(&mut self) -> FxHashSet<u64> {
        std::mem::take(&mut self.rect_dirty_ids)
    }

    pub fn drain_pending_layout(&mut self) -> FxHashSet<u64> {
        std::mem::take(&mut self.pending_layout_ids)
    }

    pub fn mark_layout_resolved(&mut self, id: u64) {
        self.pending_layout_ids.remove(&id);
    }

    fn record_visual_dirty(&self, id: u64) {
        self.render_dirty_ids.borrow_mut().insert(id);
        let Some(source) = self.current_dirty_source.borrow().clone() else {
            return;
        };
        self.render_dirty_sources
            .borrow_mut()
            .entry(id)
            .or_default()
            .insert(source);
    }
}

#[cfg(test)]
mod tests;
