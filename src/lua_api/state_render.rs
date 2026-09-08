//! Strata rendering, layout, and visibility methods for SimState.

use std::collections::{HashMap, HashSet};

use super::state::SimState;
#[path = "state_render_buckets.rs"]
mod state_render_buckets;
#[path = "state_render_groups.rs"]
mod state_render_groups;
#[path = "state_render_repairs.rs"]
mod state_render_repairs;
use state_render_buckets::{
    dfs_emit, is_region, is_strata_root_boundary, should_trace_strata_invalidations,
    uses_parent_alpha_fallback,
};
use state_render_repairs::{build_strata_bucket_repair_plan, splice_strata_bucket_repair};

struct RaisedToplevelSegment {
    owner_id: u64,
    show_order: u64,
    ids: Vec<u64>,
}

impl SimState {
    /// Initialize derived render state that must be propagated once after startup.
    pub fn initialize_render_state(&mut self) {
        self.widgets.propagate_all_effective_alpha();
        self.widgets.propagate_all_effective_scale();
    }

    /// Return the per-strata buckets, building lazily if needed.
    pub fn get_strata_buckets(&mut self) -> Option<&Vec<Vec<u64>>> {
        if self.strata_buckets.is_none() {
            self.strata_buckets = Some(self.build_strata_buckets());
        }
        self.strata_buckets.as_ref()
    }

    /// Build per-strata ID buckets for shown layout frames, sorted by render order.
    ///
    /// Alpha is filtered later while emitting render lists. Keeping transparent
    /// but shown frames in the order cache avoids rebuilding all strata buckets
    /// when fade animations cross alpha zero.
    fn build_strata_buckets(&mut self) -> Vec<Vec<u64>> {
        // Step 1: Collect frame IDs per strata (unordered).
        let mut visible: HashSet<u64> = HashSet::new();
        let mut strata_map: Vec<Vec<u64>> = vec![Vec::new(); crate::widget::FrameStrata::COUNT];
        for id in self.widgets.iter_ids() {
            let Some(f) = self.widgets.get(id) else {
                continue;
            };
            if !self.frame_belongs_in_strata_bucket(id, f) {
                continue;
            }
            visible.insert(id);
            let strata = self.frame_bucket_strata(f);
            strata_map[strata.as_index()].push(id);
        }

        // Step 2: For each strata, identify roots and DFS-emit in grouped order.
        let mut buckets = vec![Vec::new(); crate::widget::FrameStrata::COUNT];
        for (si, ids) in strata_map.iter().enumerate() {
            let mut region_roots = self.find_strata_region_roots(ids, si, &visible);
            self.sort_root_regions(&mut region_roots);
            let mut roots = self.find_strata_roots(ids, si, &visible);
            self.sort_by_frame_level(&mut roots);
            let bucket = &mut buckets[si];
            bucket.extend(region_roots);
            for root_id in roots {
                dfs_emit(root_id, si, &self.widgets, &visible, bucket);
            }
            self.move_raised_toplevel_segments_after_regular(bucket);
        }
        self.regroup_toplevel_subtrees(buckets)
    }

    fn move_raised_toplevel_segments_after_regular(&self, bucket: &mut Vec<u64>) {
        let (regular_ids, mut segments) = self.partition_raised_toplevel_segments(bucket);
        segments.sort_by_key(|segment| (segment.show_order, segment.owner_id));

        bucket.extend(regular_ids);
        for mut segment in segments {
            anchor_toplevel_owner(&mut segment.ids, segment.owner_id);
            bucket.extend(segment.ids);
        }
    }

    fn partition_raised_toplevel_segments(
        &self,
        bucket: &mut Vec<u64>,
    ) -> (Vec<u64>, Vec<RaisedToplevelSegment>) {
        let mut regular_ids = Vec::with_capacity(bucket.len());
        let mut segments = Vec::<RaisedToplevelSegment>::new();
        let mut segment_indices = HashMap::<u64, usize>::new();
        let mut owner_cache = HashMap::<u64, Option<(u64, u64)>>::new();

        for id in bucket.drain(..) {
            match self.nearest_toplevel_owner(id, &mut owner_cache, true) {
                Some((owner_id, show_order)) => append_raised_toplevel_id(
                    &mut segments,
                    &mut segment_indices,
                    owner_id,
                    show_order,
                    id,
                ),
                None => regular_ids.push(id),
            }
        }
        (regular_ids, segments)
    }

    fn nearest_toplevel_owner(
        &self,
        id: u64,
        cache: &mut HashMap<u64, Option<(u64, u64)>>,
        raised_only: bool,
    ) -> Option<(u64, u64)> {
        if let Some(owner) = cache.get(&id) {
            return *owner;
        }

        let mut path = Vec::new();
        let mut current_id = Some(id);
        let owner = loop {
            let Some(frame_id) = current_id else {
                break None;
            };
            if let Some(owner) = cache.get(&frame_id) {
                break *owner;
            }
            let Some(frame) = self.widgets.get(frame_id) else {
                break None;
            };
            path.push(frame_id);
            if is_strata_root_boundary(frame) {
                break None;
            }
            let order = self
                .active_toplevel_show_orders
                .get(&frame_id)
                .copied()
                .unwrap_or(0);
            if frame.toplevel && (!raised_only || order > 0) {
                break Some((frame_id, order));
            }
            current_id = frame.parent_id;
        };

        for frame_id in path {
            cache.insert(frame_id, owner);
        }
        owner
    }

    fn frame_belongs_in_strata_bucket(&self, id: u64, frame: &crate::widget::Frame) -> bool {
        self.widgets.is_ancestor_visible(id) || self.hidden_button_texture_has_visible_parent(frame)
    }

    fn hidden_button_texture_has_visible_parent(&self, frame: &crate::widget::Frame) -> bool {
        frame.alpha > 0.0
            && uses_parent_alpha_fallback(frame)
            && frame
                .parent_id
                .is_some_and(|parent_id| self.widgets.is_ancestor_visible(parent_id))
    }

    /// Find root frames in a strata: no parent, parent in different strata, or parent not visible.
    fn find_strata_roots(
        &self,
        ids: &[u64],
        strata_idx: usize,
        visible: &HashSet<u64>,
    ) -> Vec<u64> {
        ids.iter()
            .copied()
            .filter(|&id| {
                let Some(f) = self.widgets.get(id) else {
                    return false;
                };
                if is_region(f.widget_type) || is_strata_root_boundary(f) {
                    return false;
                }
                match f.parent_id {
                    None => true,
                    Some(pid) => {
                        let Some(parent) = self.widgets.get(pid) else {
                            return true;
                        };
                        let same_strata = self.frame_bucket_strata(parent).as_index() == strata_idx;
                        !same_strata || !visible.contains(&pid) || is_strata_root_boundary(parent)
                    }
                }
            })
            .collect()
    }

    fn find_strata_region_roots(
        &self,
        ids: &[u64],
        strata_idx: usize,
        visible: &HashSet<u64>,
    ) -> Vec<u64> {
        ids.iter()
            .copied()
            .filter(|&id| {
                let Some(frame) = self.widgets.get(id) else {
                    return false;
                };
                if !is_region(frame.widget_type) {
                    return false;
                }
                match frame.parent_id {
                    None => true,
                    Some(parent_id) => {
                        let Some(parent) = self.widgets.get(parent_id) else {
                            return true;
                        };
                        let same_strata = self.frame_bucket_strata(parent).as_index() == strata_idx;
                        !same_strata
                            || !visible.contains(&parent_id)
                            || is_strata_root_boundary(parent)
                    }
                }
            })
            .collect()
    }

    /// Sort root frame IDs so explicit Raise() wins within the same raw level.
    fn sort_by_frame_level(&self, ids: &mut [u64]) {
        ids.sort_by(|&a, &b| {
            let fa = self.widgets.get(a);
            let fb = self.widgets.get(b);
            match (fa, fb) {
                (Some(fa), Some(fb)) => {
                    (fa.frame_level, fa.raise_order, a).cmp(&(fb.frame_level, fb.raise_order, b))
                }
                _ => a.cmp(&b),
            }
        });
    }

    fn sort_root_regions(&self, ids: &mut [u64]) {
        ids.sort_by(|&a, &b| {
            let (frame_a, frame_b) = match (self.widgets.get(a), self.widgets.get(b)) {
                (Some(frame_a), Some(frame_b)) => (frame_a, frame_b),
                _ => return a.cmp(&b),
            };
            let type_flag = |frame: &crate::widget::Frame| -> u8 {
                u8::from(matches!(
                    frame.widget_type,
                    crate::widget::WidgetType::FontString | crate::widget::WidgetType::SimpleHTML
                ))
            };
            (
                frame_a.draw_layer as i32,
                frame_a.draw_sub_layer,
                type_flag(frame_a),
                a,
            )
                .cmp(&(
                    frame_b.draw_layer as i32,
                    frame_b.draw_sub_layer,
                    type_flag(frame_b),
                    b,
                ))
        });
    }

    /// Eagerly recompute layout rect for a frame and all its descendants.
    /// Called when layout-affecting properties change (anchors, size, scale, parent).
    /// Stores the computed rect on each Frame so the renderer can use it directly.
    pub fn invalidate_layout(&mut self, id: u64) {
        let sw = self.screen_width;
        let sh = self.screen_height;
        let mut cache = crate::layout::LayoutCache::default();
        Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
        // Frame positions may have changed — schedule hit grid re-insertion
        // so apply_hit_grid_changes updates stale rectangles.
        self.pending_hit_grid_changes.push((id, true));
    }

    /// Like `invalidate_layout` but also recomputes sibling frames anchored to
    /// `id`. Uses the reverse anchor index for O(k) lookup where k = number of
    /// dependents. Called by SetWidth/SetHeight/SetSize/SetScale/SetAtlas so
    /// that cross-frame-anchored siblings (e.g. three-slice Center) update.
    pub fn invalidate_layout_with_dependents(&mut self, id: u64) {
        let sw = self.screen_width;
        let sh = self.screen_height;
        let mut cache = crate::layout::LayoutCache::default();
        Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
        Self::recompute_anchor_dependents(&mut self.widgets, id, sw, sh, &mut cache, 0);
        self.queue_hit_grid_layout_changes(id);
    }

    pub(crate) fn recompute_layout_subtree(
        widgets: &mut crate::widget::WidgetRegistry,
        id: u64,
        screen_width: f32,
        screen_height: f32,
        cache: &mut crate::layout::LayoutCache,
    ) {
        // Remove stale entry so compute_frame_rect_cached recomputes.
        cache.remove(&id);
        let rect = crate::layout::compute_frame_rect_cached(
            widgets,
            id,
            screen_width,
            screen_height,
            cache,
        )
        .rect;
        let children: Vec<u64> = widgets
            .get(id)
            .map(|f| f.children.clone())
            .unwrap_or_default();
        if let Some(f) = widgets.get_mut(id) {
            f.layout_rect = Some(rect);
        }
        widgets.mark_layout_resolved(id);
        for child_id in children {
            Self::recompute_layout_subtree(widgets, child_id, screen_width, screen_height, cache);
        }
    }

    /// Recompute frames anchored to `target_id` using the reverse index.
    /// Recurses into dependents-of-dependents so that transitive anchor chains
    /// (e.g. TitleCanvasSpacerFrame → ScrollContainer → overlay buttons) all
    /// get updated in a single pass.
    pub(crate) fn recompute_anchor_dependents(
        widgets: &mut crate::widget::WidgetRegistry,
        target_id: u64,
        sw: f32,
        sh: f32,
        cache: &mut crate::layout::LayoutCache,
        depth: u32,
    ) {
        if depth > 16 {
            return; // guard against cycles
        }
        for dep_id in anchor_dependent_ids(widgets, target_id, depth) {
            Self::recompute_layout_subtree(widgets, dep_id, sw, sh, cache);
            Self::recompute_anchor_dependents(widgets, dep_id, sw, sh, cache, depth + 1);
        }
    }

    /// Ensure every frame has a layout_rect and resolve dirty roots.
    /// Called before quad rebuilds (acts as the "next frame" layout resolution).
    pub fn ensure_layout_rects(&mut self) {
        // Phase 1: frames that never had layout computed
        let pending = self.widgets.drain_pending_layout();
        if !pending.is_empty() {
            let sw = self.screen_width;
            let sh = self.screen_height;
            let mut cache = crate::layout::LayoutCache::default();
            let pending_root_ids: Vec<u64> = pending
                .iter()
                .copied()
                .filter(|id| {
                    self.widgets
                        .get(*id)
                        .and_then(|f| f.parent_id)
                        .is_none_or(|parent_id| !pending.contains(&parent_id))
                })
                .collect();
            for id in pending_root_ids {
                if self
                    .widgets
                    .get(id)
                    .is_some_and(|f| f.layout_rect.is_none())
                {
                    Self::recompute_layout_subtree(&mut self.widgets, id, sw, sh, &mut cache);
                    self.widgets.clear_rect_dirty_subtree(id);
                }
            }
        }
        // Phase 2: dirty roots — recompute subtree + anchor dependents
        let dirty = self.widgets.drain_rect_dirty();
        if !dirty.is_empty() {
            let sw = self.screen_width;
            let sh = self.screen_height;
            let mut cache = crate::layout::LayoutCache::default();
            for id in &dirty {
                Self::recompute_layout_subtree(&mut self.widgets, *id, sw, sh, &mut cache);
                Self::recompute_anchor_dependents(&mut self.widgets, *id, sw, sh, &mut cache, 0);
            }
        }
    }

    /// Force layout resolution for a single frame, clearing its rect_dirty flag.
    /// Called by GetSize/GetWidth/GetHeight, rect query methods, and IsRectValid
    /// to match WoW behavior where layout resolves immediately.
    ///
    /// Fast path: when `id` is directly in `rect_dirty_ids` (common during
    /// loading — SetPoint marks dirty, then GetRect resolves), skips the
    /// ancestor walk but still resolves anchor dependents so immediate Lua
    /// geometry queries observe sibling frames that track `id`.
    /// Slow path: when `id` inherits dirtiness from an ancestor, resolves
    /// dirty ancestor subtrees first.
    pub fn resolve_rect_if_dirty(&mut self, id: u64) {
        if !self.widgets.is_rect_dirty(id) {
            return;
        }
        let dirty_roots = self.widgets.collect_dirty_ancestor_roots(id);
        // Fast path: only `id` itself is dirty, so no ancestor layout can be stale.
        if dirty_roots.len() == 1 && dirty_roots[0] == id {
            self.invalidate_layout_with_dependents(id);
            self.widgets.clear_rect_dirty(id);
            return;
        }
        // Slow path: dirty ancestor(s) need subtree recomputation first.
        self.resolve_dirty_roots(dirty_roots);
        self.invalidate_layout(id);
        self.widgets.clear_rect_dirty(id);
    }

    /// Resolve dirty ancestor roots that cause `id` to appear dirty via the
    /// `is_rect_dirty` ancestor walk. Computes their layout rects and clears
    /// their dirty flags so descendants become clean.
    fn resolve_dirty_roots(&mut self, dirty_roots: Vec<u64>) {
        if dirty_roots.is_empty() {
            return;
        }
        let sw = self.screen_width;
        let sh = self.screen_height;
        let mut cache = crate::layout::LayoutCache::default();
        // Process topmost first (reverse of bottom-up collection order).
        // Recompute the full subtree so siblings of `id` also get updated
        // layout_rects before we clear the dirty flag.
        for &root_id in dirty_roots.iter().rev() {
            Self::recompute_layout_subtree(&mut self.widgets, root_id, sw, sh, &mut cache);
            Self::recompute_anchor_dependents(&mut self.widgets, root_id, sw, sh, &mut cache, 0);
            self.queue_hit_grid_layout_changes(root_id);
            self.widgets.clear_rect_dirty(root_id);
        }
    }

    fn queue_hit_grid_layout_changes(&mut self, root_id: u64) {
        let mut ids = vec![root_id];
        collect_transitive_anchor_dependent_ids(&self.widgets, root_id, &mut ids, 0);
        for id in ids {
            self.pending_hit_grid_changes.push((id, true));
        }
    }

    /// Set whether a shown frame participates in top-level show ordering.
    pub fn set_frame_toplevel(&mut self, id: u64, toplevel: bool) {
        let Some(frame) = self.widgets.get_mut(id) else {
            return;
        };
        let was_toplevel = frame.toplevel;
        frame.toplevel = toplevel;

        // Enabling the flag does not raise an already-shown frame. Native
        // created controls retain zero until Hide/Show or explicit Raise.
        let order_changed = !toplevel && self.active_toplevel_show_orders.remove(&id).is_some();
        if was_toplevel != toplevel || order_changed {
            self.pending_hit_grid_changes.push((id, true));
            self.invalidate_strata_buckets();
        }
    }

    /// Set a frame's visibility and eagerly propagate effective_alpha.
    /// Surgically updates strata_buckets: inserts on show, removes on hide.
    pub fn set_frame_visible(&mut self, id: u64, visible: bool) {
        let was_visible = self.widgets.get(id).map(|f| f.visible).unwrap_or(false);
        self.widgets.set_visible(id, visible);
        if was_visible == visible {
            return;
        }
        let toplevel_order_changed = self.update_toplevel_show_order(id, visible);
        self.update_on_update_cache(id, visible);
        // Propagate effective_alpha: look up parent's effective_alpha.
        let parent_eff = self
            .widgets
            .get(id)
            .and_then(|f| f.parent_id)
            .and_then(|pid| self.widgets.get(pid))
            .map(|p| p.effective_alpha)
            .unwrap_or(1.0);
        if !visible {
            crate::lua_api::frame::methods::button_anchor_hierarchy::stop_animation_groups_for_hidden_subtree(
                self, id,
            );
            // Hide: remove subtree from buckets BEFORE propagating alpha to 0.
            self.remove_subtree_from_buckets(id);
        }
        self.widgets.propagate_effective_alpha(id, parent_eff);
        if visible {
            // A grouped subtree spans local strata. A same-strata splice
            // cannot rebuild its complete owner-anchored segment.
            if toplevel_order_changed
                || self.toplevel_ancestors(id).next().is_some()
                || (!self.try_repair_strata_buckets_after_show(id)
                    && !self.try_append_tooltip_root_after_show(id))
            {
                self.invalidate_strata_buckets();
            }
        }
        // Record for incremental HitGrid update (applied by App after Lua runs).
        self.pending_hit_grid_changes.push((id, visible));
    }

    fn update_toplevel_show_order(&mut self, id: u64, visible: bool) -> bool {
        let is_toplevel = self.widgets.get(id).is_some_and(|frame| frame.toplevel);
        if !is_toplevel {
            return false;
        }
        if visible {
            self.assign_next_toplevel_show_order(id);
            return true;
        }
        self.active_toplevel_show_orders.remove(&id).is_some()
    }

    fn assign_next_toplevel_show_order(&mut self, id: u64) {
        self.next_toplevel_show_order = self
            .next_toplevel_show_order
            .checked_add(1)
            .expect("top-level show order overflow");
        self.active_toplevel_show_orders
            .insert(id, self.next_toplevel_show_order);
    }

    fn try_repair_strata_buckets_after_show(&mut self, shown_id: u64) -> bool {
        if self.strata_buckets.is_none() {
            return false;
        }
        let Some(repair_root) = self.visible_same_strata_ancestor(shown_id) else {
            return false;
        };
        let Some(repair_plan) = build_strata_bucket_repair_plan(self, repair_root) else {
            return false;
        };
        let Some(buckets) = self.strata_buckets.as_mut() else {
            return false;
        };
        splice_strata_bucket_repair(&mut buckets[repair_plan.strata_idx], repair_plan)
    }

    fn try_append_tooltip_root_after_show(&mut self, shown_id: u64) -> bool {
        let Some(frame) = self.widgets.get(shown_id) else {
            return false;
        };
        if self.frame_bucket_strata(frame) != crate::widget::FrameStrata::Tooltip {
            return false;
        }
        let Some(repair_plan) = build_strata_bucket_repair_plan(self, shown_id) else {
            return false;
        };
        let Some(bucket) = self
            .strata_buckets
            .as_mut()
            .and_then(|buckets| buckets.get_mut(repair_plan.strata_idx))
        else {
            return false;
        };
        if bucket.iter().any(|&id| id == shown_id) {
            return true;
        }
        bucket.extend(repair_plan.replacement_segment);
        true
    }

    fn visible_same_strata_ancestor(&self, id: u64) -> Option<u64> {
        let frame = self.widgets.get(id)?;
        let target_strata = self.frame_bucket_strata(frame).as_index();
        let mut current_id = frame.parent_id;
        while let Some(parent_id) = current_id {
            let parent = self.widgets.get(parent_id)?;
            if is_strata_root_boundary(parent) {
                return None;
            }
            if self.frame_bucket_strata(parent).as_index() == target_strata
                && self.frame_belongs_in_strata_bucket(parent_id, parent)
            {
                return Some(parent_id);
            }
            current_id = parent.parent_id;
        }
        None
    }

    /// Remove a frame and all its descendants from strata_buckets.
    fn remove_subtree_from_buckets(&mut self, root_id: u64) {
        let Some(buckets) = self.strata_buckets.as_mut() else {
            return;
        };
        // Collect all IDs in the subtree.
        let mut subtree = HashSet::new();
        let mut queue = vec![root_id];
        while let Some(fid) = queue.pop() {
            subtree.insert(fid);
            if let Some(f) = self.widgets.get(fid) {
                queue.extend(f.children.iter().copied());
            }
        }
        for bucket in buckets.iter_mut() {
            bucket.retain(|id| !subtree.contains(id));
        }
    }

    /// Invalidate strata buckets so they rebuild on next access.
    /// Used after show/reparent operations that change DFS traversal order.
    #[track_caller]
    pub(crate) fn invalidate_strata_buckets(&mut self) {
        if should_trace_strata_invalidations(&self.start_time) {
            let caller = std::panic::Location::caller();
            eprintln!(
                "{} [strata-invalid] {}:{}",
                crate::logging::global_elapsed_prefix(),
                caller.file(),
                caller.line()
            );
        }
        self.strata_buckets = None;
    }

    pub(crate) fn frame_bucket_strata(
        &self,
        frame: &crate::widget::Frame,
    ) -> crate::widget::FrameStrata {
        use crate::widget::WidgetType;

        if matches!(
            frame.widget_type,
            WidgetType::Texture | WidgetType::FontString | WidgetType::Line
        ) {
            return frame
                .parent_id
                .and_then(|parent_id| self.widgets.get(parent_id))
                .map(|parent| parent.frame_strata)
                .unwrap_or(frame.frame_strata);
        }
        frame.frame_strata
    }

    fn toplevel_ancestors(&self, id: u64) -> impl Iterator<Item = &crate::widget::Frame> {
        std::iter::successors(self.widgets.get(id), |frame| {
            frame.parent_id.and_then(|parent| self.widgets.get(parent))
        })
        .take_while(|frame| !is_strata_root_boundary(frame))
        .filter(|frame| frame.toplevel)
    }

    /// Raised level belongs to the active top-level ancestor, not tooltip owner.
    pub(crate) fn raised_frame_level(&self, id: u64) -> u64 {
        self.toplevel_ancestors(id)
            .find_map(|frame| self.active_toplevel_show_orders.get(&frame.id).copied())
            .unwrap_or(0)
    }

    /// Raise within the existing raw level, or advance a shown top-level group.
    /// Neither operation changes the frame's raw strata or frame level.
    pub fn raise_frame(&mut self, id: u64) {
        let (parent_id, strata, level, raised_group) = match self.widgets.get(id) {
            Some(f) => (
                f.parent_id,
                f.frame_strata,
                f.frame_level,
                f.toplevel && f.visible,
            ),
            None => return,
        };
        if raised_group {
            self.assign_next_toplevel_show_order(id);
        }
        let sibling_max_order = self
            .sibling_raise_order_range(id, parent_id, strata, level)
            .1;
        if let Some(f) = self.widgets.get_mut_visual(id) {
            f.raise_order = sibling_max_order.saturating_add(1);
        }
        // raise_order is part of the hit-grid render-order key.
        self.pending_hit_grid_changes.push((id, true));
        // Re-sort the affected subtree in strata buckets.
        // Avoid setting strata_buckets = None: Show/Hide calls later in the
        // same handler chain rely on buckets being Some for surgical insert/remove.
        if self.strata_buckets.is_some() {
            self.remove_subtree_from_buckets(id);
            self.invalidate_strata_buckets();
        }
    }

    /// Lower a frame below same-level siblings in the same strata.
    ///
    /// Mirrors `Raise()` by adjusting only the same-level tie-breaker.
    pub fn lower_frame(&mut self, id: u64) {
        let (parent_id, strata, level, current_order) = match self.widgets.get(id) {
            Some(f) => (f.parent_id, f.frame_strata, f.frame_level, f.raise_order),
            None => return,
        };
        let min_order = self
            .sibling_raise_order_range(id, parent_id, strata, level)
            .0;
        if current_order < min_order {
            return; // Already at bottom
        }
        if let Some(f) = self.widgets.get_mut_visual(id) {
            f.raise_order = min_order.saturating_sub(1);
        }
        // raise_order is part of the hit-grid render-order key.
        self.pending_hit_grid_changes.push((id, true));
        if self.strata_buckets.is_some() {
            self.remove_subtree_from_buckets(id);
            self.invalidate_strata_buckets();
        }
    }

    /// Return (min, max) raise order among same-level siblings.
    fn sibling_raise_order_range(
        &self,
        id: u64,
        parent_id: Option<u64>,
        strata: crate::widget::FrameStrata,
        level: i32,
    ) -> (i32, i32) {
        let sibling_ids: Vec<u64> = if let Some(pid) = parent_id {
            self.widgets
                .get(pid)
                .map(|p| p.children.clone())
                .unwrap_or_default()
        } else {
            // Root frames: all frames with no parent
            self.widgets
                .iter_ids()
                .filter(|&fid| {
                    self.widgets
                        .get(fid)
                        .map(|f| f.parent_id.is_none())
                        .unwrap_or(false)
                })
                .collect()
        };
        let levels: Vec<i32> = sibling_ids
            .iter()
            .filter(|&&sid| sid != id)
            .filter_map(|&sid| self.widgets.get(sid))
            .filter(|f| f.frame_strata == strata)
            .filter(|f| f.frame_level == level)
            .map(|f| f.raise_order)
            .collect();
        let min = levels.iter().copied().min().unwrap_or(0);
        let max = levels.iter().copied().max().unwrap_or(0);
        (min, max)
    }

    fn update_on_update_cache(&mut self, id: u64, visible: bool) {
        let Some(mut cache) = self.visible_on_update_cache.take() else {
            return;
        };
        if visible {
            self.add_on_update_descendants(id, &mut cache);
        } else {
            self.remove_on_update_descendants(id, &mut cache);
        }
        self.visible_on_update_cache = Some(cache);
    }

    /// Add `id` and its descendants to cache if they have OnUpdate and are ancestor-visible.
    fn add_on_update_descendants(&self, id: u64, cache: &mut Vec<u64>) {
        let should_cache_id =
            self.on_update_frames.contains(&id) && self.widgets.is_ancestor_visible(id);
        let already_cached = cache.iter().any(|&cached_id| cached_id == id);
        cache.extend((should_cache_id && !already_cached).then_some(id));
        let children: Vec<u64> = self
            .widgets
            .get(id)
            .map(|f| f.children.clone())
            .unwrap_or_default();
        for child_id in children {
            if self.widgets.get(child_id).is_some_and(|f| f.visible) {
                self.add_on_update_descendants(child_id, cache);
            }
        }
    }
    /// Remove `id` and all its descendants from cache (hidden ancestor = all hidden).
    fn remove_on_update_descendants(&self, id: u64, cache: &mut Vec<u64>) {
        cache.retain(|&cached_id| cached_id != id);
        let children: Vec<u64> = self
            .widgets
            .get(id)
            .map(|f| f.children.clone())
            .unwrap_or_default();
        for child_id in children {
            self.remove_on_update_descendants(child_id, cache);
        }
    }

    /// Keep only OnUpdate handlers owned by the named addon. Invalidates cache.
    pub fn retain_on_update_for_addon(&mut self, addon_name: &str) {
        let idx = self.addons.iter().position(|a| a.folder_name == addon_name);
        let addon_idx = idx.map(|i| i as u16);
        let before = self.on_update_frames.len();
        self.on_update_frames
            .retain(|&id| self.widgets.get(id).and_then(|f| f.owner_addon) == addon_idx);
        self.visible_on_update_cache = None;
        let after = self.on_update_frames.len();
        eprintln!("[self-test] stripped OnUpdate: {before} → {after} (keeping {addon_name})");
    }
}

fn append_raised_toplevel_id(
    segments: &mut Vec<RaisedToplevelSegment>,
    segment_indices: &mut HashMap<u64, usize>,
    owner_id: u64,
    show_order: u64,
    id: u64,
) {
    let segment_index = *segment_indices.entry(owner_id).or_insert_with(|| {
        let index = segments.len();
        segments.push(RaisedToplevelSegment {
            owner_id,
            show_order,
            ids: Vec::new(),
        });
        index
    });
    segments[segment_index].ids.push(id);
}

fn anchor_toplevel_owner(ids: &mut Vec<u64>, owner_id: u64) {
    let Some(owner_position) = ids.iter().position(|&id| id == owner_id) else {
        return;
    };
    if owner_position > 0 {
        let owner = ids.remove(owner_position);
        ids.insert(0, owner);
    }
}

fn anchor_dependent_ids(
    widgets: &crate::widget::WidgetRegistry,
    target_id: u64,
    depth: u32,
) -> Vec<u64> {
    if depth > 16 {
        return Vec::new();
    }
    widgets
        .get_anchor_dependents(target_id)
        .map(|set| set.iter().copied().collect())
        .unwrap_or_default()
}

fn collect_transitive_anchor_dependent_ids(
    widgets: &crate::widget::WidgetRegistry,
    target_id: u64,
    ids: &mut Vec<u64>,
    depth: u32,
) {
    for dep_id in anchor_dependent_ids(widgets, target_id, depth) {
        ids.push(dep_id);
        collect_transitive_anchor_dependent_ids(widgets, dep_id, ids, depth + 1);
    }
}

#[cfg(test)]
mod tests {
    include!("state_render_tests.rs");
}
