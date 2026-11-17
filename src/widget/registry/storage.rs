use super::{RenderDirtySource, WidgetRegistry};
use crate::widget::Frame;
use rustc_hash::{FxHashMap, FxHashSet};

pub(super) fn registry_storage_estimate_bytes(registry: &WidgetRegistry) -> usize {
    std::mem::size_of::<WidgetRegistry>()
        + widgets_bytes(registry)
        + names_bytes(registry)
        + registry.ordered_ids.capacity() * std::mem::size_of::<u64>()
        + hash_set_u64_bytes(&registry.render_dirty_ids.borrow())
        + render_dirty_sources_bytes(&registry.render_dirty_sources.borrow())
        + dirty_source_bytes(&registry.current_dirty_source.borrow())
        + hash_map_u64_hash_set_u64_bytes(&registry.anchor_dependents)
        + registry.ordered_anchor_dependents.capacity() * std::mem::size_of::<(u64, Vec<u64>)>()
        + registry
            .ordered_anchor_dependents
            .values()
            .map(|ids| ids.capacity() * std::mem::size_of::<u64>())
            .sum::<usize>()
        + hash_map_u64_hash_set_u64_bytes(&registry.frame_anchor_targets)
        + hash_set_u64_bytes(&registry.rect_dirty_ids)
        + hash_set_u64_bytes(&registry.hit_grid_dirty_ids)
        + hash_set_u64_bytes(&registry.pending_layout_ids)
        + hash_set_u64_bytes(&registry.editbox_ids)
}

fn widgets_bytes(registry: &WidgetRegistry) -> usize {
    registry.widgets.capacity() * std::mem::size_of::<(u64, Frame)>()
        + registry
            .widgets
            .values()
            .map(Frame::storage_estimate_bytes)
            .sum::<usize>()
}

fn names_bytes(registry: &WidgetRegistry) -> usize {
    registry.names.capacity() * std::mem::size_of::<(String, u64)>()
        + registry.names.keys().map(String::capacity).sum::<usize>()
}

fn hash_set_u64_bytes(values: &FxHashSet<u64>) -> usize {
    values.capacity() * std::mem::size_of::<u64>()
}

fn hash_map_u64_hash_set_u64_bytes(values: &FxHashMap<u64, FxHashSet<u64>>) -> usize {
    values.capacity() * std::mem::size_of::<(u64, FxHashSet<u64>)>()
        + values
            .values()
            .map(|s| s.capacity() * std::mem::size_of::<u64>())
            .sum::<usize>()
}

fn dirty_source_bytes(value: &Option<RenderDirtySource>) -> usize {
    usize::from(value.is_some()) * std::mem::size_of::<RenderDirtySource>()
}

fn render_dirty_sources_bytes(values: &FxHashMap<u64, FxHashSet<RenderDirtySource>>) -> usize {
    values.capacity() * std::mem::size_of::<(u64, FxHashSet<RenderDirtySource>)>()
        + values
            .values()
            .map(render_dirty_source_set_bytes)
            .sum::<usize>()
}

fn render_dirty_source_set_bytes(values: &FxHashSet<RenderDirtySource>) -> usize {
    values.capacity() * std::mem::size_of::<RenderDirtySource>()
}
