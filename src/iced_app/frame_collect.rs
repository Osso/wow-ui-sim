//! Frame collection and sorting helpers for rendering.

use crate::widget::WidgetType;
use super::layout::compute_frame_rect;

/// Collect all frame IDs in the subtree rooted at the named frame.
pub fn collect_subtree_ids(
    registry: &crate::widget::WidgetRegistry,
    root_name: &str,
) -> std::collections::HashSet<u64> {
    let mut ids = std::collections::HashSet::new();
    let root_id = registry.all_ids().into_iter().find(|&id| {
        registry
            .get(id)
            .map(|f| f.name.as_deref() == Some(root_name))
            .unwrap_or(false)
    });
    if let Some(root_id) = root_id {
        let mut queue = vec![root_id];
        while let Some(id) = queue.pop() {
            ids.insert(id);
            if let Some(f) = registry.get(id) {
                queue.extend(f.children.iter().copied());
            }
        }
    }
    ids
}

/// Collect IDs of frames whose ancestor chain is fully visible.
///
/// Walks the tree top-down from root frames, pruning entire subtrees when a
/// frame is hidden. Returns a map from frame ID to effective alpha (the product
/// of all ancestor alphas). This avoids computing layout for invisible subtrees
/// and provides correct alpha propagation for rendering.
pub fn collect_ancestor_visible_ids(
    registry: &crate::widget::WidgetRegistry,
) -> std::collections::HashMap<u64, f32> {
    let mut visible = std::collections::HashMap::new();
    // Queue entries: (frame_id, parent_effective_alpha)
    let mut queue: Vec<(u64, f32)> = registry
        .all_ids()
        .into_iter()
        .filter(|&id| {
            registry
                .get(id)
                .map(|f| f.parent_id.is_none())
                .unwrap_or(false)
        })
        .map(|id| (id, 1.0f32))
        .collect();

    while let Some((id, parent_alpha)) = queue.pop() {
        let Some(f) = registry.get(id) else { continue };
        if !f.visible {
            // Button state textures (HighlightTexture etc.) start with visible=false
            // but their visibility is resolved later by button_vis::should_skip_frame.
            if is_button_state_texture(f, id, registry) {
                visible.insert(id, parent_alpha * f.alpha);
            }
            continue;
        }
        let effective_alpha = parent_alpha * f.alpha;
        visible.insert(id, effective_alpha);
        for &child_id in &f.children {
            queue.push((child_id, effective_alpha));
        }
    }
    visible
}

/// Check if a frame is a button state texture child (NormalTexture, PushedTexture, etc.).
///
/// These textures have state-driven visibility that overrides `frame.visible`,
/// so they must not be pruned by ancestor-visibility checks.
fn is_button_state_texture(
    f: &crate::widget::Frame,
    id: u64,
    registry: &crate::widget::WidgetRegistry,
) -> bool {
    if !matches!(f.widget_type, WidgetType::Texture) {
        return false;
    }
    let Some(parent_id) = f.parent_id else { return false };
    let Some(parent) = registry.get(parent_id) else { return false };
    if !matches!(parent.widget_type, WidgetType::Button | WidgetType::CheckButton) {
        return false;
    }
    ["NormalTexture", "PushedTexture", "HighlightTexture", "DisabledTexture"]
        .iter()
        .any(|key| parent.children_keys.get(*key) == Some(&id))
}

/// Collect frames with computed rects, sorted by strata/level/draw-layer.
///
/// Only frames in `ancestor_visible` are considered, skipping layout
/// computation for frames hidden by an ancestor. Each frame carries its
/// effective alpha (product of all ancestor alphas).
pub fn collect_sorted_frames<'a>(
    registry: &'a crate::widget::WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
    ancestor_visible: &std::collections::HashMap<u64, f32>,
) -> Vec<(u64, &'a crate::widget::Frame, crate::LayoutRect, f32)> {
    let mut frames: Vec<_> = ancestor_visible
        .iter()
        .filter_map(|(&id, &eff_alpha)| {
            let f = registry.get(id)?;
            let rect = compute_frame_rect(registry, id, screen_width, screen_height);
            Some((id, f, rect, eff_alpha))
        })
        .collect();

    frames.sort_by(|a, b| {
        a.1.frame_strata
            .cmp(&b.1.frame_strata)
            .then_with(|| a.1.frame_level.cmp(&b.1.frame_level))
            .then_with(|| {
                let is_region = |t: &WidgetType| {
                    matches!(t, WidgetType::Texture | WidgetType::FontString)
                };
                match (is_region(&a.1.widget_type), is_region(&b.1.widget_type)) {
                    (true, true) => a.1.draw_layer.cmp(&b.1.draw_layer)
                        .then_with(|| a.1.draw_sub_layer.cmp(&b.1.draw_sub_layer)),
                    (false, true) => std::cmp::Ordering::Less,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, false) => std::cmp::Ordering::Equal,
                }
            })
            .then_with(|| a.0.cmp(&b.0))
    });

    frames
}
