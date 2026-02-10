//! Frame collection and sorting helpers for rendering.

use crate::widget::{FrameStrata, WidgetType};
use super::layout::{LayoutCache, compute_frame_rect_cached};

/// Frame names excluded from hit testing (full-screen or non-interactive overlays).
const HIT_TEST_EXCLUDED: &[&str] = &[
    "UIParent", "Minimap", "WorldFrame",
    "DEFAULT_CHAT_FRAME", "ChatFrame1",
    "EventToastManagerFrame", "EditModeManagerFrame",
];

/// Result of collecting frames for rendering and hit testing.
pub struct CollectedFrames<'a> {
    /// Frames sorted by strata/level/draw-layer for rendering.
    pub render: Vec<(u64, &'a crate::widget::Frame, crate::LayoutRect, f32)>,
    /// Frames eligible for hit testing, sorted by strata/level/id (low to high).
    /// Rects are in unscaled WoW coordinates (caller applies UI_SCALE).
    pub hittable: Vec<(u64, crate::LayoutRect)>,
}

/// Collect all frame IDs in the subtree rooted at the named frame.
pub fn collect_subtree_ids(
    registry: &crate::widget::WidgetRegistry,
    root_name: &str,
) -> std::collections::HashSet<u64> {
    let mut ids = std::collections::HashSet::new();
    let root_id = registry.iter_ids().find(|&id| {
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
        .iter_ids()
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
        // Don't recurse into GameTooltip children — build_tooltip_quads handles
        // the complete tooltip rendering (background, border, text lines).
        // The NineSlice child + its corner/edge/center textures would otherwise
        // render on top, producing a grey overlay.
        if f.widget_type == WidgetType::GameTooltip {
            continue;
        }
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

/// Sort key within a strata bucket: level, draw-layer type, draw-layer, sub-layer, id.
fn intra_strata_cmp(
    a: &(u64, &crate::widget::Frame, crate::LayoutRect, f32),
    b: &(u64, &crate::widget::Frame, crate::LayoutRect, f32),
) -> std::cmp::Ordering {
    a.1.frame_level.cmp(&b.1.frame_level)
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
}

/// Collect frames with computed rects, sorted by strata/level/draw-layer.
///
/// When `strata_buckets` is provided (pre-built per-strata ID lists), iterates
/// buckets in strata order and sorts only within each bucket. Otherwise falls
/// back to scanning the full `ancestor_visible` map and doing a global sort.
///
/// Also builds a hit-test list as a side output: visible, mouse-enabled
/// frames sorted by strata/level/id, excluding non-interactive overlays.
pub fn collect_sorted_frames<'a>(
    registry: &'a crate::widget::WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
    ancestor_visible: &std::collections::HashMap<u64, f32>,
    strata_buckets: Option<&Vec<Vec<u64>>>,
    cache: &mut LayoutCache,
) -> CollectedFrames<'a> {
    let mut frames: Vec<(u64, &crate::widget::Frame, crate::LayoutRect, f32)> = Vec::new();
    let mut hittable: Vec<(u64, FrameStrata, i32, crate::LayoutRect)> = Vec::new();

    if let Some(buckets) = strata_buckets {
        // Iterate buckets in strata order — output is strata-sorted by construction.
        for bucket in buckets {
            let start = frames.len();
            for &id in bucket {
                let Some(&eff_alpha) = ancestor_visible.get(&id) else { continue };
                let Some(f) = registry.get(id) else { continue };
                let rect = compute_frame_rect_cached(registry, id, screen_width, screen_height, cache).rect;
                frames.push((id, f, rect, eff_alpha));
                if f.visible && f.mouse_enabled
                    && !f.name.as_deref().is_some_and(|n| HIT_TEST_EXCLUDED.contains(&n))
                {
                    hittable.push((id, f.frame_strata, f.frame_level, rect));
                }
            }
            // Sort only within this strata bucket (level/draw_layer/id).
            frames[start..].sort_by(intra_strata_cmp);
        }
    } else {
        // Fallback: scan all ancestor_visible and do global sort.
        for (&id, &eff_alpha) in ancestor_visible {
            let Some(f) = registry.get(id) else { continue };
            let rect = compute_frame_rect_cached(registry, id, screen_width, screen_height, cache).rect;
            frames.push((id, f, rect, eff_alpha));
            if f.visible && f.mouse_enabled
                && !f.name.as_deref().is_some_and(|n| HIT_TEST_EXCLUDED.contains(&n))
            {
                hittable.push((id, f.frame_strata, f.frame_level, rect));
            }
        }
        frames.sort_by(|a, b| {
            a.1.frame_strata.cmp(&b.1.frame_strata).then_with(|| intra_strata_cmp(a, b))
        });
    }

    hittable.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.0.cmp(&b.0))
    });

    CollectedFrames {
        render: frames,
        hittable: hittable.into_iter().map(|(id, _, _, r)| (id, r)).collect(),
    }
}
