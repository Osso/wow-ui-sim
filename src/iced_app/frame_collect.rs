//! Frame collection and sorting helpers for rendering.

use std::sync::LazyLock;

use rustc_hash::FxHashSet;

use crate::widget::{FrameStrata, WidgetType};

/// Frame names excluded from hit testing (full-screen or non-interactive overlays).
pub const HIT_TEST_EXCLUDED: &[&str] = &[
    "UIParent",
    "Minimap",
    "WorldFrame",
    "DEFAULT_CHAT_FRAME",
    "ChatFrame1",
    "EventToastManagerFrame",
    "EditModeManagerFrame",
];

static HIT_TEST_EXCLUDED_NAMES: LazyLock<FxHashSet<&'static str>> =
    LazyLock::new(|| HIT_TEST_EXCLUDED.iter().copied().collect());

/// Render-order key shared by headless frame collection and the GUI hit grid.
/// Kept here so the `--no-default-features` Docker image does not compile a
/// headless module through a GUI-only dependency.
pub type HitOrderKey = (FrameStrata, i32, i32, u64);

/// Result of collecting frames for hit testing.
///
/// Rects are in unscaled WoW coordinates (caller applies UI_SCALE).
pub struct CollectedFrames {
    /// Frames eligible for hit testing with their render-order key, sorted
    /// by strata/level/raise-order/id (low to high).
    pub hittable: Vec<(u64, HitOrderKey, crate::LayoutRect)>,
}

/// Collect all frame IDs in the subtree rooted at the named frame.
pub fn collect_subtree_ids(
    registry: &crate::widget::WidgetRegistry,
    root_name: &str,
) -> FxHashSet<u64> {
    let mut ids = FxHashSet::default();
    let root_id = find_best_named_root(registry, root_name);
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

fn find_best_named_root(registry: &crate::widget::WidgetRegistry, root_name: &str) -> Option<u64> {
    registry
        .iter_ids()
        .filter(|&id| {
            registry
                .get(id)
                .is_some_and(|frame| frame.name.as_deref() == Some(root_name))
        })
        .max_by_key(|&id| named_root_score(registry, id))
}

fn named_root_score(registry: &crate::widget::WidgetRegistry, id: u64) -> (bool, u64, usize, u64) {
    let Some(frame) = registry.get(id) else {
        return (false, 0, 0, id);
    };
    let area = frame
        .layout_rect
        .map(|rect| rect.width * rect.height)
        .unwrap_or_else(|| frame.width * frame.height)
        .max(0.0) as u64;

    (frame.visible, area, count_subtree_frames(registry, id), id)
}

fn count_subtree_frames(registry: &crate::widget::WidgetRegistry, root_id: u64) -> usize {
    let mut count = 0;
    let mut queue = vec![root_id];
    while let Some(id) = queue.pop() {
        count += 1;
        if let Some(frame) = registry.get(id) {
            queue.extend(frame.children.iter().copied());
        }
    }
    count
}

/// Sort key type for frame rendering order within a strata bucket.
pub type IntraStrataKey = (i32, i32, u64, u8, i32, i32, u8, u64);

/// Intra-strata sort key for rendering order within the same frame strata.
///
/// In WoW, regions (Texture/FontString) render as part of their parent frame,
/// not independently. Regions use their parent's frame_level and group with
/// their parent via `parent_id`, ensuring all regions of a frame render
/// immediately after that frame (before any higher-level content).
///
/// Non-regions sort by raw `frame_level`, then `raise_order` as a same-level
/// tie-breaker. Retail does not expose or apply Raise()/Lower() as a way to
/// cross raw frame levels in the simple sibling probes.
/// Regions follow the same parent ordering within the same parent draw
/// layer so later-created overlays do not get buried under earlier background
/// textures. FontStrings (type_flag=1) render above Textures (type_flag=0) in
/// the same draw layer per WoW rules.
pub fn intra_strata_sort_key(
    f: &crate::widget::Frame,
    id: u64,
    registry: &crate::widget::WidgetRegistry,
) -> IntraStrataKey {
    if matches!(
        f.widget_type,
        WidgetType::Texture | WidgetType::FontString | WidgetType::Line
    ) {
        let (parent_frame_level, parent_raise_order, parent_id) = f
            .parent_id
            .and_then(|pid| {
                registry
                    .get(pid)
                    .map(|p| (p.frame_level, p.raise_order, pid))
            })
            .unwrap_or((f.frame_level, f.raise_order, id));
        let type_flag = if f.widget_type == WidgetType::FontString {
            1u8
        } else {
            0u8
        };
        (
            parent_frame_level,
            parent_raise_order,
            parent_id,
            1,
            f.draw_layer as i32,
            f.draw_sub_layer,
            type_flag,
            id,
        )
    } else {
        (f.frame_level, f.raise_order, id, 0, 0, 0, 0, 0)
    }
}

/// Build a hit-test list from the widget registry.
///
/// Returns visible, mouse-enabled frames sorted by strata/level/id,
/// excluding non-interactive overlays.
/// Alpha is intentionally ignored: in WoW, transparent mouse-enabled
/// frames still receive mouse events.
pub fn collect_hittable_frames(
    registry: &crate::widget::WidgetRegistry,
    _strata_buckets: &[Vec<u64>],
) -> CollectedFrames {
    let mut hittable: Vec<HittableFrameEntry> = registry
        .iter_ids()
        .filter_map(|id| hittable_frame_entry(registry, id))
        .collect();

    hittable.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
            .then_with(|| a.0.cmp(&b.0))
    });

    CollectedFrames {
        hittable: hittable
            .into_iter()
            .map(|(id, strata, level, raise_order, r)| (id, (strata, level, raise_order, id), r))
            .collect(),
    }
}

type HittableFrameEntry = (u64, FrameStrata, i32, i32, crate::LayoutRect);

fn hittable_frame_entry(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
) -> Option<HittableFrameEntry> {
    let frame = registry.get(id)?;
    if !crate::layout::frame_has_render_layout(registry, id) {
        return None;
    }
    let rect = frame.layout_rect?;

    is_frame_hittable(registry, id, frame).then(|| {
        (
            id,
            frame.frame_strata,
            frame.frame_level,
            frame.raise_order,
            rect,
        )
    })
}

fn is_frame_hittable(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    frame: &crate::widget::Frame,
) -> bool {
    registry.is_ancestor_visible(id)
        && (frame.mouse_enabled || matches!(frame.widget_type, WidgetType::EditBox))
        && !is_hit_test_excluded(frame)
}

fn is_hit_test_excluded(frame: &crate::widget::Frame) -> bool {
    frame
        .name
        .as_deref()
        .is_some_and(|name| HIT_TEST_EXCLUDED_NAMES.contains(name))
}

pub fn frame_accepts_mouse_button(frame: &crate::widget::Frame, button_name: &str) -> bool {
    let mouse_enabled = frame.mouse_enabled || matches!(frame.widget_type, WidgetType::EditBox);
    mouse_enabled
        && !frame
            .pass_through_buttons
            .contains(&button_name.to_ascii_lowercase())
        && frame_has_registered_mouse_button(frame, button_name)
}

pub fn scaled_hit_rect_insets(frame: &crate::widget::Frame) -> (f32, f32, f32, f32) {
    let scale = frame.effective_scale;
    let (left, right, top, bottom) = frame.hit_rect_insets;
    (left * scale, right * scale, top * scale, bottom * scale)
}

fn frame_has_registered_mouse_button(frame: &crate::widget::Frame, button_name: &str) -> bool {
    if frame.registered_mouse_buttons.is_empty() {
        return true;
    }

    frame_mouse_registration_matches(frame, button_name, true)
        || frame_mouse_registration_matches(frame, button_name, false)
}

pub fn frame_mouse_registration_matches(
    frame: &crate::widget::Frame,
    button_name: &str,
    down: bool,
) -> bool {
    if frame.registered_mouse_buttons.is_empty() {
        return true;
    }

    let edge = if down { "Down" } else { "Up" };
    registration_set_matches(
        &frame.registered_mouse_buttons,
        &format!("{button_name}{edge}"),
    ) || registration_set_matches(&frame.registered_mouse_buttons, &format!("Any{edge}"))
}

fn registration_set_matches(
    registrations: &std::collections::HashSet<String>,
    target: &str,
) -> bool {
    registrations
        .iter()
        .any(|registered| registered.eq_ignore_ascii_case(target))
}

#[cfg(test)]
mod tests {
    use super::{collect_subtree_ids, intra_strata_sort_key};
    use crate::widget::{AnchorPoint, Frame, WidgetRegistry, WidgetType};

    #[cfg(feature = "gui")]
    #[test]
    fn collect_subtree_ids_uses_best_matching_named_frame() {
        let mut registry = WidgetRegistry::new();
        let old_root = registry.register(Frame::new(
            WidgetType::Frame,
            Some("DuplicateRoot".to_string()),
            None,
        ));
        let old_child = registry.register(Frame::new(WidgetType::Texture, None, Some(old_root)));
        registry.add_child(old_root, old_child);

        let mut new_root_frame =
            Frame::new(WidgetType::Frame, Some("DuplicateRoot".to_string()), None);
        new_root_frame.set_size(200.0, 100.0);
        let new_root = registry.register(new_root_frame);
        let new_child = registry.register(Frame::new(WidgetType::Texture, None, Some(new_root)));
        registry.add_child(new_root, new_child);

        let ids = collect_subtree_ids(&registry, "DuplicateRoot");

        assert!(ids.contains(&new_root));
        assert!(ids.contains(&new_child));
        assert!(!ids.contains(&old_root));
        assert!(!ids.contains(&old_child));
    }

    #[test]
    fn excluded_overlay_names_are_not_hittable() {
        let mut registry = WidgetRegistry::new();
        let excluded_id = register_hittable_frame(&mut registry, "UIParent", 10);
        let included_id = register_hittable_frame(&mut registry, "ClickableFrame", 20);
        registry.get_mut(included_id).unwrap().parent_id = Some(excluded_id);
        registry.get_mut(included_id).unwrap().set_point(
            AnchorPoint::TopLeft,
            Some(excluded_id as usize),
            AnchorPoint::TopLeft,
            20.0,
            0.0,
        );
        registry.add_child(excluded_id, included_id);

        let strata_buckets = vec![vec![excluded_id, included_id]];
        let collected = super::collect_hittable_frames(&registry, &strata_buckets);
        let collected_ids: Vec<u64> = collected
            .hittable
            .into_iter()
            .map(|(id, _key, _rect)| id)
            .collect();

        assert_eq!(collected_ids, vec![included_id]);
    }

    #[test]
    fn unanchored_frames_are_not_hittable_at_parent_origin() {
        let mut registry = WidgetRegistry::new();
        let mut parent = Frame::new(WidgetType::Frame, Some("UIParent".to_string()), None);
        parent.id = 1;
        parent.layout_rect = Some(crate::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 1600.0,
            height: 1200.0,
        });
        registry.register(parent);

        let child = register_hittable_frame(&mut registry, "UnanchoredPanel", 0);
        registry.get_mut(child).unwrap().parent_id = Some(1);
        registry.add_child(1, child);

        let strata_buckets = vec![vec![child]];
        let collected = super::collect_hittable_frames(&registry, &strata_buckets);

        assert!(
            collected.hittable.is_empty(),
            "visible unanchored frames should not be mouse targets at parent origin"
        );
    }

    #[test]
    fn frames_under_hidden_ancestors_are_not_hittable() {
        let mut registry = WidgetRegistry::new();
        let mut parent = Frame::new(WidgetType::Frame, Some("HiddenPanel".to_string()), None);
        parent.id = 1;
        parent.visible = false;
        parent.layout_rect = Some(crate::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 300.0,
        });
        registry.register(parent);

        let child = register_hittable_frame(&mut registry, "HiddenPanelChild", 20);
        let child_frame = registry.get_mut(child).unwrap();
        child_frame.parent_id = Some(1);
        child_frame.set_point(
            AnchorPoint::TopLeft,
            Some(1),
            AnchorPoint::TopLeft,
            20.0,
            20.0,
        );
        registry.add_child(1, child);

        let strata_buckets = vec![vec![child]];
        let collected = super::collect_hittable_frames(&registry, &strata_buckets);

        assert!(
            collected.hittable.is_empty(),
            "mouse-enabled children of hidden panels must not remain in the hit grid"
        );
    }

    #[test]
    fn later_created_regions_sort_after_earlier_regions_in_same_layer() {
        let mut registry = WidgetRegistry::new();

        let parent = Frame::new(WidgetType::Frame, Some("Parent".to_string()), None);
        let parent_id = parent.id;
        registry.register(parent);

        let first = Frame::new(
            WidgetType::Texture,
            Some("First".to_string()),
            Some(parent_id),
        );
        let first_id = first.id;
        registry.register(first);
        registry.add_child(parent_id, first_id);

        let second = Frame::new(
            WidgetType::Texture,
            Some("Second".to_string()),
            Some(parent_id),
        );
        let second_id = second.id;
        registry.register(second);
        registry.add_child(parent_id, second_id);

        let first_key = intra_strata_sort_key(registry.get(first_id).unwrap(), first_id, &registry);
        let second_key =
            intra_strata_sort_key(registry.get(second_id).unwrap(), second_id, &registry);

        assert!(
            first_key < second_key,
            "later-created texture should sort later/on top within the same parent layer"
        );
    }

    fn register_hittable_frame(registry: &mut WidgetRegistry, name: &str, x: i32) -> u64 {
        let mut frame = Frame::new(WidgetType::Frame, Some(name.to_string()), None);
        frame.mouse_enabled = true;
        frame.layout_rect = Some(crate::LayoutRect {
            x: x as f32,
            y: 0.0,
            width: 20.0,
            height: 20.0,
        });
        let id = frame.id;
        registry.register(frame);
        id
    }
}
