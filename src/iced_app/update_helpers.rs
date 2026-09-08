//! Helper functions extracted from update.rs for hit-grid, checkbutton, and dirty-ID merging.

use rustc_hash::FxHashSet;

/// Merge optional dirty-ID sets into one.
///
/// If any input is `None`, the result must stay `None` because a full rebuild
/// is required and the exact frame set is incomplete.
pub(super) fn merge_dirty_ids<I>(ids: I) -> Option<FxHashSet<u64>>
where
    I: IntoIterator<Item = Option<FxHashSet<u64>>>,
{
    let mut merged = FxHashSet::default();
    for dirty_ids in ids {
        let ids = dirty_ids?;
        merged.extend(ids);
    }
    Some(merged)
}

/// Apply one batch using the same order consumed by the renderer.
pub(super) fn apply_hit_grid_batch(
    grid: &mut super::hit_grid::HitGrid,
    registry: &crate::widget::WidgetRegistry,
    strata_buckets: &[Vec<u64>],
    geometry_roots: impl IntoIterator<Item = u64>,
    visibility_changes: &[(u64, bool)],
) {
    grid.update_render_order(strata_buckets);
    for root in geometry_roots {
        apply_subtree_hit_grid_change(grid, registry, root, true);
    }
    for &(root, visible) in visibility_changes {
        apply_subtree_hit_grid_change(grid, registry, root, visible);
    }
}

/// Walk a subtree using ranks refreshed by the batch owner.
fn apply_subtree_hit_grid_change(
    grid: &mut super::hit_grid::HitGrid,
    registry: &crate::widget::WidgetRegistry,
    root_id: u64,
    became_visible: bool,
) {
    let mut stack = vec![root_id];
    while let Some(id) = stack.pop() {
        let Some(f) = registry.get(id) else { continue };
        let hit = became_visible
            .then(|| {
                grid.render_order_key(id)
                    .zip(hittable_rect(registry, id, f))
            })
            .flatten();
        if let Some((key, rect)) = hit {
            grid.insert(id, rect, key);
        } else {
            grid.remove(id);
        }
        stack.extend_from_slice(&f.children);
    }
}

/// Compute the hit-testable rectangle for a frame, if eligible.
fn hittable_rect(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    f: &crate::widget::Frame,
) -> Option<iced::Rectangle> {
    if !crate::layout::frame_has_render_layout(registry, id) {
        return None;
    }
    let mouse_enabled =
        f.mouse_enabled || matches!(f.widget_type, crate::widget::WidgetType::EditBox);
    if !registry.is_ancestor_visible(id) || !mouse_enabled {
        return None;
    }
    if f.name
        .as_deref()
        .is_some_and(|n| super::frame_collect::HIT_TEST_EXCLUDED.contains(&n))
    {
        return None;
    }
    let rect = f.layout_rect?;
    let (il, ir, it, ib) = super::frame_collect::scaled_hit_rect_insets(f);
    Some(iced::Rectangle::new(
        iced::Point::new(
            (rect.x + il) * crate::render::texture::UI_SCALE,
            (rect.y + it) * crate::render::texture::UI_SCALE,
        ),
        iced::Size::new(
            (rect.width - il - ir).max(0.0) * crate::render::texture::UI_SCALE,
            (rect.height - it - ib).max(0.0) * crate::render::texture::UI_SCALE,
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::apply_hit_grid_batch;
    use crate::iced_app::frame_collect::collect_hittable_frames;
    use crate::iced_app::hit_grid::HitGrid;
    use crate::iced_app::strata_emit::build_hittable_rects;
    use crate::lua_api::WowLuaEnv;
    use iced::Point;

    #[test]
    fn coalesced_hide_show_preserves_final_hit_order_without_rebuilding_grid() {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_size(800.0, 600.0);
        env.exec(
            r#"
            BatchBottom = CreateFrame("Button", "BatchBottom", UIParent)
            BatchBottom:SetSize(100, 100)
            BatchBottom:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            BatchBottom:EnableMouse(true)
            BatchTop = CreateFrame("Button", "BatchTop", UIParent)
            BatchTop:SetAllPoints(BatchBottom)
            BatchTop:EnableMouse(true)
            "#,
        )
        .unwrap();
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let bottom = state.widgets.get_id_by_name("BatchBottom").unwrap();
        let top = state.widgets.get_id_by_name("BatchTop").unwrap();
        let buckets = vec![vec![bottom, top]];
        let collected = collect_hittable_frames(&state.widgets, &buckets);
        let rectangles = build_hittable_rects(&collected, &state.widgets);
        let mut incremental = HitGrid::new(rectangles.clone(), 800.0, 600.0);
        let rebuilt = HitGrid::new(rectangles, 800.0, 600.0);
        let point = Point::new(100.0, 100.0);
        assert_eq!(incremental.topmost_matching_at(point, |_| true), Some(top));

        apply_hit_grid_batch(
            &mut incremental,
            &state.widgets,
            &buckets,
            [top],
            &[(top, false), (top, true)],
        );
        assert_eq!(
            incremental.topmost_matching_at(point, |_| true),
            rebuilt.topmost_matching_at(point, |_| true),
            "coalesced visibility notifications must retain final visible membership",
        );
        assert_eq!(incremental.topmost_matching_at(point, |_| true), Some(top));
    }
}

/// Check if a frame is a CheckButton that should be auto-toggled (not an action bar button).
pub(super) fn is_toggleable_checkbutton(state: &crate::lua_api::SimState, frame_id: u64) -> bool {
    let is_checkbutton = state
        .widgets
        .get(frame_id)
        .map(|f| f.widget_type == crate::widget::WidgetType::CheckButton)
        .unwrap_or(false);
    if !is_checkbutton {
        return false;
    }
    !state
        .action_ui_buttons
        .iter()
        .any(|(id, _)| *id == frame_id)
}

/// Read the `__checked` attribute from a frame (defaults to false).
pub(super) fn get_checked_attribute(state: &crate::lua_api::SimState, frame_id: u64) -> bool {
    state
        .widgets
        .get(frame_id)
        .and_then(|f| f.attributes.get("__checked"))
        .and_then(|v| {
            if let crate::widget::AttributeValue::Boolean(b) = v {
                Some(*b)
            } else {
                None
            }
        })
        .unwrap_or(false)
}
