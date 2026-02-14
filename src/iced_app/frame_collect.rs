//! Frame collection and sorting helpers for rendering.

use crate::widget::{FrameStrata, WidgetType};

/// Frame names excluded from hit testing (full-screen or non-interactive overlays).
pub const HIT_TEST_EXCLUDED: &[&str] = &[
    "UIParent", "Minimap", "WorldFrame",
    "DEFAULT_CHAT_FRAME", "ChatFrame1",
    "EventToastManagerFrame", "EditModeManagerFrame",
];

/// Result of collecting frames for hit testing.
///
/// Rects are in unscaled WoW coordinates (caller applies UI_SCALE).
pub struct CollectedFrames {
    /// Frames eligible for hit testing, sorted by strata/level/id (low to high).
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

/// Sort key type for frame rendering order within a strata bucket.
pub type IntraStrataKey = (i32, std::cmp::Reverse<u64>, u8, i32, i32, u8, std::cmp::Reverse<u64>);

/// Intra-strata sort key for rendering order within the same frame strata.
///
/// In WoW, regions (Texture/FontString) render as part of their parent frame,
/// not independently. Regions use their parent's frame_level and group with
/// their parent via `parent_id`, ensuring all regions of a frame render
/// immediately after that frame (before any higher-level content).
///
/// Non-regions sort by `(frame_level, Reverse(id))` — higher IDs (later-created
/// frames) render first (lower in the sort), so earlier-created frames render on
/// top. This matches WoW's stacking where action bar icon textures (created
/// early) must render above the bar background.
/// FontStrings (type_flag=1) render above Textures (type_flag=0) in the same
/// draw layer per WoW rules.
pub fn intra_strata_sort_key(
    f: &crate::widget::Frame,
    id: u64,
    registry: &crate::widget::WidgetRegistry,
) -> IntraStrataKey {
    if matches!(f.widget_type, WidgetType::Texture | WidgetType::FontString | WidgetType::Line) {
        let (parent_level, parent_id) = f.parent_id
            .and_then(|pid| registry.get(pid).map(|p| (p.frame_level, pid)))
            .unwrap_or((f.frame_level, id));
        let type_flag = if f.widget_type == WidgetType::FontString { 1u8 } else { 0u8 };
        (parent_level, std::cmp::Reverse(parent_id), 1, f.draw_layer as i32, f.draw_sub_layer, type_flag, std::cmp::Reverse(id))
    } else {
        (f.frame_level, std::cmp::Reverse(id), 0, 0, 0, 0, std::cmp::Reverse(0))
    }
}

/// Build a hit-test list from visible-only strata buckets.
///
/// Returns visible, mouse-enabled frames sorted by strata/level/id,
/// excluding non-interactive overlays.
pub fn collect_hittable_frames(
    registry: &crate::widget::WidgetRegistry,
    strata_buckets: &[Vec<u64>],
) -> CollectedFrames {
    let mut hittable: Vec<(u64, FrameStrata, i32, crate::LayoutRect)> = Vec::new();

    for bucket in strata_buckets {
        for &id in bucket {
            let Some(f) = registry.get(id) else { continue };
            let Some(rect) = f.layout_rect else { continue };
            if f.visible && f.effective_alpha > 0.0 && f.mouse_enabled
                && !f.name.as_deref().is_some_and(|n| HIT_TEST_EXCLUDED.contains(&n))
            {
                hittable.push((id, f.frame_strata, f.frame_level, rect));
            }
        }
    }

    hittable.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.0.cmp(&b.0))
    });

    CollectedFrames {
        hittable: hittable.into_iter().map(|(id, _, _, r)| (id, r)).collect(),
    }
}
