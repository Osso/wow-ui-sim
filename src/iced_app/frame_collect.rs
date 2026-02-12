//! Frame collection and sorting helpers for rendering.

use crate::widget::{FrameStrata, WidgetType};
use super::layout::LayoutCache;

/// Frame names excluded from hit testing (full-screen or non-interactive overlays).
const HIT_TEST_EXCLUDED: &[&str] = &[
    "UIParent", "Minimap", "WorldFrame",
    "DEFAULT_CHAT_FRAME", "ChatFrame1",
    "EventToastManagerFrame", "EditModeManagerFrame",
];

/// Result of collecting frames for rendering and hit testing.
///
/// Render list stores `(id, rect, effective_alpha)` — frame data is looked up
/// from the registry during emit. This allows the render list to be cached
/// across rebuilds (no borrowed references).
pub struct CollectedFrames {
    /// Frames sorted by strata/level/draw-layer for rendering.
    pub render: Vec<(u64, crate::LayoutRect, f32)>,
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

/// Effective strata for rendering: regions use their parent's strata.
fn effective_strata(
    f: &crate::widget::Frame,
    registry: &crate::widget::WidgetRegistry,
) -> FrameStrata {
    if matches!(f.widget_type, WidgetType::Texture | WidgetType::FontString) {
        f.parent_id
            .and_then(|pid| registry.get(pid))
            .map(|p| p.frame_strata)
            .unwrap_or(f.frame_strata)
    } else {
        f.frame_strata
    }
}

/// Sort key type for frame rendering order within a strata bucket.
pub type IntraStrataKey = (i32, u64, u8, i32, i32, u8, u64);

/// Intra-strata sort key for rendering order within the same frame strata.
///
/// In WoW, regions (Texture/FontString) render as part of their parent frame,
/// not independently. Regions use their parent's frame_level and group with
/// their parent via `parent_id`, ensuring all regions of a frame render
/// immediately after that frame (before any higher-level content).
///
/// Non-regions sort by `(frame_level, id)` — higher IDs (later-created frames)
/// render on top at the same level, matching WoW's creation-order stacking.
/// FontStrings (type_flag=1) render above Textures (type_flag=0) in the same
/// draw layer per WoW rules.
pub fn intra_strata_sort_key(
    f: &crate::widget::Frame,
    id: u64,
    registry: &crate::widget::WidgetRegistry,
) -> IntraStrataKey {
    if matches!(f.widget_type, WidgetType::Texture | WidgetType::FontString) {
        let (parent_level, parent_id) = f.parent_id
            .and_then(|pid| registry.get(pid).map(|p| (p.frame_level, pid)))
            .unwrap_or((f.frame_level, id));
        let type_flag = if f.widget_type == WidgetType::FontString { 1u8 } else { 0u8 };
        (parent_level, parent_id, 1, f.draw_layer as i32, f.draw_sub_layer, type_flag, id)
    } else {
        (f.frame_level, id, 0, 0, 0, 0, 0)
    }
}

/// Collect frames with computed rects, sorted by strata/level/draw-layer.
///
/// When `strata_buckets` is provided (pre-built per-strata ID lists), iterates
/// buckets in strata order (already sorted). Otherwise falls back to scanning
/// the full `ancestor_visible` map and doing a global sort.
///
/// Also builds a hit-test list as a side output: visible, mouse-enabled
/// frames sorted by strata/level/id, excluding non-interactive overlays.
pub fn collect_sorted_frames(
    registry: &crate::widget::WidgetRegistry,
    _screen_width: f32,
    _screen_height: f32,
    ancestor_visible: &std::collections::HashMap<u64, f32>,
    strata_buckets: Option<&Vec<Vec<u64>>>,
    _cache: &mut LayoutCache,
) -> CollectedFrames {
    let mut frames: Vec<(u64, crate::LayoutRect, f32)> = Vec::new();
    let mut hittable: Vec<(u64, FrameStrata, i32, crate::LayoutRect)> = Vec::new();

    if let Some(buckets) = strata_buckets {
        // Buckets are maintained in sorted order — no sort needed.
        for bucket in buckets {
            for &id in bucket {
                let Some(&eff_alpha) = ancestor_visible.get(&id) else { continue };
                let Some(f) = registry.get(id) else { continue };
                let Some(rect) = f.layout_rect else { continue };
                frames.push((id, rect, eff_alpha));
                if f.visible && f.mouse_enabled
                    && !f.name.as_deref().is_some_and(|n| HIT_TEST_EXCLUDED.contains(&n))
                {
                    hittable.push((id, f.frame_strata, f.frame_level, rect));
                }
            }
        }
    } else {
        // Fallback: scan all ancestor_visible and do global sort.
        for (&id, &eff_alpha) in ancestor_visible {
            let Some(f) = registry.get(id) else { continue };
            let Some(rect) = f.layout_rect else { continue };
            frames.push((id, rect, eff_alpha));
            if f.visible && f.mouse_enabled
                && !f.name.as_deref().is_some_and(|n| HIT_TEST_EXCLUDED.contains(&n))
            {
                hittable.push((id, f.frame_strata, f.frame_level, rect));
            }
        }
        frames.sort_by(|&(id_a, _, _), &(id_b, _, _)| {
            match (registry.get(id_a), registry.get(id_b)) {
                (Some(fa), Some(fb)) => {
                    let strata_a = effective_strata(fa, registry);
                    let strata_b = effective_strata(fb, registry);
                    strata_a.cmp(&strata_b)
                        .then_with(|| intra_strata_sort_key(fa, id_a, registry).cmp(&intra_strata_sort_key(fb, id_b, registry)))
                }
                _ => id_a.cmp(&id_b),
            }
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
