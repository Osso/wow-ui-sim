//! Anchor dependency tracking: reverse index and cycle detection.

use std::collections::{HashMap, VecDeque};

use rustc_hash::FxHashSet;

use super::WidgetRegistry;
use crate::widget::clamp_geometry::{
    COORDINATE_ROOT, COORDINATE_ROOT_PROPAGATION, uses_retail_clamp_producers,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorCyclePath {
    pub relative_to_id: u64,
    pub dependent_id: u64,
    pub dependent_ancestors: Vec<u64>,
}

impl WidgetRegistry {
    fn is_parent_token(segment: &str) -> bool {
        matches!(segment, "$parent" | "$Parent" | "$parentKey")
    }

    fn resolve_anchor_relative_expr(&self, frame_id: u64, expr: &str) -> Option<u64> {
        let mut segments = expr.split('.');
        let first = segments.next()?;
        if Self::is_parent_token(first) {
            let mut current_id = self.widgets.get(&frame_id)?.parent_id?;
            for segment in segments {
                if Self::is_parent_token(segment) {
                    current_id = self.widgets.get(&current_id)?.parent_id?;
                } else {
                    let frame = self.widgets.get(&current_id)?;
                    current_id = *frame.children_keys.get(segment)?;
                }
            }
            return Some(current_id);
        }
        if let Some(suffix) = first
            .strip_prefix("$parent")
            .or_else(|| first.strip_prefix("$Parent"))
            && !suffix.is_empty()
            && segments.clone().next().is_none()
        {
            let parent_id = self.widgets.get(&frame_id)?.parent_id?;
            let parent_name = self.widgets.get(&parent_id)?.name.as_deref()?;
            let resolved = format!("{parent_name}{suffix}");
            return self.names.get(resolved.as_str()).copied();
        }
        self.names.get(expr).copied()
    }

    fn collect_resolved_named_targets(&self, frame_id: u64) -> Vec<(usize, u64)> {
        self.widgets
            .get(&frame_id)
            .map(|frame| {
                frame
                    .anchors
                    .iter()
                    .enumerate()
                    .filter_map(|(index, anchor)| {
                        if anchor.relative_to_id.is_some() {
                            return None;
                        }
                        let expr = anchor.relative_to.as_deref()?;
                        self.resolve_anchor_relative_expr(frame_id, expr)
                            .map(|target| (index, target))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn apply_resolved_named_targets(&mut self, frame_id: u64, resolved_targets: &[(usize, u64)]) {
        if resolved_targets.is_empty() {
            return;
        }
        if let Some(frame) = self.widgets.get_mut(&frame_id) {
            for (index, target_id) in resolved_targets.iter().copied() {
                if let Some(anchor) = frame.anchors.get_mut(index) {
                    anchor.relative_to_id = Some(target_id as usize);
                }
            }
        }
    }

    fn collect_direct_anchor_targets(&self, frame_id: u64) -> Vec<u64> {
        self.widgets
            .get(&frame_id)
            .map(|frame| {
                frame
                    .anchors
                    .iter()
                    .filter_map(|anchor| anchor.relative_to_id.map(|target| target as u64))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn reindex_anchor_dependents_for_frame(&mut self, frame_id: u64) {
        let direct_targets = self.collect_direct_anchor_targets(frame_id);
        if uses_retail_clamp_producers() {
            let mut removed = self
                .frame_anchor_targets
                .get(&frame_id)
                .cloned()
                .unwrap_or_default();
            removed.retain(|target| !direct_targets.contains(target));
            for target in removed {
                self.remove_anchor_dependent(target, frame_id);
            }
            for target in direct_targets {
                self.add_anchor_dependent(target, frame_id);
            }
            return;
        }
        for target in direct_targets {
            self.anchor_dependents
                .entry(target)
                .or_default()
                .insert(frame_id);
        }
    }

    /// Resolve any named/relative-key anchors on a frame into concrete target IDs.
    /// Safe to call repeatedly as children/layer regions become available.
    pub fn resolve_named_anchor_targets_for_frame(&mut self, frame_id: u64) {
        if !uses_retail_clamp_producers() {
            self.remove_all_anchor_dependents_for(frame_id);
        }
        let resolved_targets = self.collect_resolved_named_targets(frame_id);
        self.apply_resolved_named_targets(frame_id, &resolved_targets);
        self.reindex_anchor_dependents_for_frame(frame_id);
    }

    /// Check if setting a point from `frame_id` to `relative_to_id` would create a cycle.
    /// A cycle exists if relative_to (or any of its anchor dependencies) already
    /// depends on frame_id.
    pub fn would_create_anchor_cycle(&self, frame_id: u64, relative_to_id: u64) -> bool {
        frame_id == relative_to_id || self.anchor_dependency_reaches_frame(relative_to_id, frame_id)
    }

    pub fn describe_anchor_cycle(
        &self,
        frame_id: u64,
        relative_to_id: u64,
    ) -> Option<AnchorCyclePath> {
        if frame_id == relative_to_id {
            return Some(AnchorCyclePath {
                relative_to_id,
                dependent_id: relative_to_id,
                dependent_ancestors: Vec::new(),
            });
        }

        let path = self.anchor_dependency_path(relative_to_id, frame_id)?;
        let dependent_id = path.iter().rev().nth(1).copied().unwrap_or(relative_to_id);
        let dependent_ancestors = path[..path.len().saturating_sub(2)]
            .iter()
            .rev()
            .copied()
            .collect();

        Some(AnchorCyclePath {
            relative_to_id,
            dependent_id,
            dependent_ancestors,
        })
    }

    fn anchor_dependency_reaches_frame(&self, start_id: u64, target_id: u64) -> bool {
        let mut queue = VecDeque::from([start_id]);
        let mut seen = FxHashSet::from_iter([start_id]);

        while let Some(check_id) = queue.pop_front() {
            if self.enqueue_anchor_dependencies(check_id, target_id, &mut queue, &mut seen) {
                return true;
            }
        }

        false
    }

    fn anchor_dependency_path(&self, start_id: u64, target_id: u64) -> Option<Vec<u64>> {
        let mut queue = VecDeque::from([start_id]);
        let mut seen = FxHashSet::from_iter([start_id]);
        let mut parents = HashMap::from([(start_id, start_id)]);

        while let Some(check_id) = queue.pop_front() {
            for anchor_target_id in self.anchor_target_ids(check_id) {
                if seen.insert(anchor_target_id) {
                    parents.insert(anchor_target_id, check_id);
                    if anchor_target_id == target_id {
                        return Some(rebuild_anchor_cycle_path(&parents, start_id, target_id));
                    }
                    queue.push_back(anchor_target_id);
                }
            }
        }

        None
    }

    fn enqueue_anchor_dependencies(
        &self,
        frame_id: u64,
        target_id: u64,
        queue: &mut VecDeque<u64>,
        seen: &mut FxHashSet<u64>,
    ) -> bool {
        for anchor_target_id in self.anchor_target_ids(frame_id) {
            if anchor_target_id == target_id {
                return true;
            }
            if seen.insert(anchor_target_id) {
                queue.push_back(anchor_target_id);
            }
        }

        false
    }

    fn anchor_target_ids(&self, frame_id: u64) -> impl Iterator<Item = u64> + '_ {
        self.widgets
            .get(&frame_id)
            .into_iter()
            .flat_map(|frame| frame.anchors.iter())
            .filter_map(|anchor| anchor.relative_to_id.map(|target_id| target_id as u64))
    }

    /// Inspect active anchors only, not historical reverse dirty dependencies.
    pub(crate) fn anchor_chain_reaches_region(&self, start_id: u64) -> bool {
        let mut pending = vec![start_id];
        let mut seen = FxHashSet::default();

        while let Some(id) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(frame) = self.widgets.get(&id) else {
                continue;
            };
            if matches!(
                frame.widget_type,
                crate::widget::WidgetType::Texture | crate::widget::WidgetType::FontString
            ) {
                return true;
            }
            pending.extend(self.anchor_target_ids(id));
        }

        false
    }

    /// Record that `frame_id` is anchored to `target_id`.
    pub fn add_anchor_dependent(&mut self, target_id: u64, frame_id: u64) {
        let inserted = self
            .anchor_dependents
            .entry(target_id)
            .or_default()
            .insert(frame_id);
        self.frame_anchor_targets
            .entry(frame_id)
            .or_default()
            .insert(target_id);
        if uses_retail_clamp_producers() && inserted {
            self.ordered_anchor_dependents
                .entry(target_id)
                .or_default()
                .push(frame_id);
            self.inherit_anchor_propagation(target_id, frame_id);
        }
    }

    fn inherit_anchor_propagation(&mut self, target_id: u64, frame_id: u64) {
        let requested = if target_id == COORDINATE_ROOT as u64 {
            Some(COORDINATE_ROOT_PROPAGATION)
        } else {
            self.widgets
                .get(&target_id)
                .map(|frame| frame.anchor_propagation)
        };
        if let Some(requested) = requested {
            self.request_anchor_propagation(frame_id, requested);
        }
    }

    /// Remove `frame_id` from `target_id`'s dependents.
    pub fn remove_anchor_dependent(&mut self, target_id: u64, frame_id: u64) {
        if let Some(set) = self.frame_anchor_targets.get_mut(&frame_id) {
            set.remove(&target_id);
            if set.is_empty() {
                self.frame_anchor_targets.remove(&frame_id);
            }
        }
        self.remove_anchor_dependent_from_target(target_id, frame_id);
    }

    fn remove_anchor_dependent_from_target(&mut self, target_id: u64, frame_id: u64) {
        if let Some(set) = self.anchor_dependents.get_mut(&target_id) {
            set.remove(&frame_id);
            if set.is_empty() {
                self.anchor_dependents.remove(&target_id);
            }
        }
        if uses_retail_clamp_producers()
            && let Some(ids) = self.ordered_anchor_dependents.get_mut(&target_id)
        {
            ids.retain(|id| *id != frame_id);
            if ids.is_empty() {
                self.ordered_anchor_dependents.remove(&target_id);
            }
        }
    }

    /// Remove `frame_id` from every reverse-index entry it is registered in.
    pub fn remove_all_anchor_dependents_for(&mut self, frame_id: u64) {
        if let Some(targets) = self.frame_anchor_targets.remove(&frame_id) {
            for target in targets {
                self.remove_anchor_dependent_from_target(target, frame_id);
            }
        }
    }

    /// Get frame IDs anchored to `target_id`.
    pub fn get_anchor_dependents(&self, target_id: u64) -> Option<&FxHashSet<u64>> {
        self.anchor_dependents.get(&target_id)
    }

    /// Rebuild the reverse anchor index from all existing anchors.
    /// Call once after initial load to capture anchors set during XML parsing
    /// and frame creation.
    pub fn rebuild_anchor_index(&mut self) {
        let frame_ids = if uses_retail_clamp_producers() {
            // Keep existing dependency order; resolve newly available targets
            // in simulator creation order.
            self.ordered_ids.clone()
        } else {
            self.anchor_dependents.clear();
            self.frame_anchor_targets.clear();
            self.widgets.keys().copied().collect()
        };
        for frame_id in frame_ids {
            self.resolve_named_anchor_targets_for_frame(frame_id);
        }
    }
}

fn rebuild_anchor_cycle_path(
    parents: &HashMap<u64, u64>,
    start_id: u64,
    target_id: u64,
) -> Vec<u64> {
    let mut path = vec![target_id];
    let mut current = target_id;

    while current != start_id {
        current = parents.get(&current).copied().unwrap_or(start_id);
        path.push(current);
    }

    path.reverse();
    path
}
