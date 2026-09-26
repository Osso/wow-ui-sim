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
    let visibility_roots = visibility_changes.iter().map(|&(root, _)| root);
    for id in collect_touched_subtrees(registry, geometry_roots.into_iter().chain(visibility_roots))
    {
        sync_hit_grid_frame(grid, registry, id);
    }
}

/// Union of the subtrees under `roots`, each frame once. Overlapping roots
/// (nested layout roots, repeated visibility notifications) would otherwise
/// re-walk the same frames.
fn collect_touched_subtrees(
    registry: &crate::widget::WidgetRegistry,
    roots: impl IntoIterator<Item = u64>,
) -> Vec<u64> {
    let mut visited = FxHashSet::default();
    let mut touched = Vec::new();
    let mut stack: Vec<u64> = roots.into_iter().collect();
    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        let Some(frame) = registry.get(id) else {
            continue;
        };
        touched.push(id);
        stack.extend_from_slice(&frame.children);
    }
    touched
}

/// Set a frame's hit-grid entry from current registry state. Visibility is
/// read from the registry, so the result does not depend on the order of
/// coalesced Show/Hide notifications in the batch.
fn sync_hit_grid_frame(
    grid: &mut super::hit_grid::HitGrid,
    registry: &crate::widget::WidgetRegistry,
    id: u64,
) {
    let hit = registry.get(id).and_then(|frame| {
        grid.render_order_key(id)
            .zip(hittable_rect(registry, id, frame))
    });
    match hit {
        Some((key, rect)) => grid.insert(id, rect, key),
        None => grid.remove(id),
    }
}

/// Compute the hit-testable rectangle for a frame, if eligible.
fn hittable_rect(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    f: &crate::widget::Frame,
) -> Option<iced::Rectangle> {
    let mouse_enabled =
        f.mouse_enabled || matches!(f.widget_type, crate::widget::WidgetType::EditBox);
    if !mouse_enabled || !registry.is_ancestor_visible(id) {
        return None;
    }
    if !crate::layout::frame_has_render_layout(registry, id) {
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

    fn rebuilt_grid(state: &crate::lua_api::SimState, buckets: &[Vec<u64>]) -> HitGrid {
        let collected = collect_hittable_frames(&state.widgets, buckets);
        HitGrid::new(
            build_hittable_rects(&collected, &state.widgets),
            800.0,
            600.0,
        )
    }

    #[test]
    fn overlapping_roots_and_visibility_changes_match_rebuilt_grid() {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_size(800.0, 600.0);
        env.exec(
            r#"
            OverlapParent = CreateFrame("Button", "OverlapParent", UIParent)
            OverlapParent:SetSize(200, 200)
            OverlapParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            OverlapParent:EnableMouse(true)
            OverlapChild = CreateFrame("Button", "OverlapChild", OverlapParent)
            OverlapChild:SetSize(50, 50)
            OverlapChild:SetPoint("TOPLEFT", OverlapParent, "TOPLEFT", 10, -10)
            OverlapChild:EnableMouse(true)
            "#,
        )
        .unwrap();
        let ids = |state: &crate::lua_api::SimState| {
            (
                state.widgets.get_id_by_name("OverlapParent").unwrap(),
                state.widgets.get_id_by_name("OverlapChild").unwrap(),
            )
        };
        let (parent, child) = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            ids(&state)
        };
        let buckets = vec![vec![parent, child]];
        let mut grid = rebuilt_grid(&env.state().borrow(), &buckets);
        let points = [
            Point::new(120.0, 120.0),
            Point::new(250.0, 250.0),
            Point::new(170.0, 170.0),
            Point::new(20.0, 20.0),
        ];
        let assert_matches_rebuilt = |grid: &HitGrid, label: &str| {
            let state = env.state().borrow();
            let rebuilt = rebuilt_grid(&state, &buckets);
            for point in points {
                assert_eq!(
                    grid.topmost_matching_at(point, |_| true),
                    rebuilt.topmost_matching_at(point, |_| true),
                    "{label}: hit at {point:?} differs from rebuilt grid",
                );
            }
        };

        env.exec("OverlapChild:SetPoint('TOPLEFT', OverlapParent, 'TOPLEFT', 60, -60)")
            .unwrap();
        env.exec("OverlapParent:Hide()").unwrap();
        env.state().borrow_mut().ensure_layout_rects();
        {
            let state = env.state().borrow();
            apply_hit_grid_batch(
                &mut grid,
                &state.widgets,
                &buckets,
                [parent, child],
                &[(parent, false)],
            );
        }
        assert_matches_rebuilt(&grid, "after hide + move");
        assert_eq!(
            grid.topmost_matching_at(Point::new(120.0, 120.0), |_| true),
            None
        );

        env.exec("OverlapParent:Show()").unwrap();
        env.state().borrow_mut().ensure_layout_rects();
        {
            let state = env.state().borrow();
            apply_hit_grid_batch(
                &mut grid,
                &state.widgets,
                &buckets,
                [child, parent],
                &[(child, false), (parent, true)],
            );
        }
        assert_matches_rebuilt(&grid, "after show");
        assert_eq!(
            grid.topmost_matching_at(Point::new(170.0, 170.0), |_| true),
            Some(child)
        );
    }

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
