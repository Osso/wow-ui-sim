use rustc_hash::FxHashSet;

use super::super::state::SimState;
use super::state_render_buckets::{
    collect_same_strata_subtree_ids, dfs_emit, same_strata_subtree_segment_end,
};

pub(super) struct StrataBucketRepairPlan {
    pub(super) strata_idx: usize,
    pub(super) repair_root: u64,
    pub(super) subtree_ids: FxHashSet<u64>,
    pub(super) replacement_segment: Vec<u64>,
}

pub(super) fn build_strata_bucket_repair_plan(
    state: &SimState,
    repair_root: u64,
) -> Option<StrataBucketRepairPlan> {
    let root_frame = state.widgets.get(repair_root)?;
    let strata_idx = state.frame_bucket_strata(root_frame).as_index();
    let subtree_ids = same_strata_subtree_ids(repair_root, strata_idx, &state.widgets);
    let visible_ids = visible_same_strata_subtree_ids(state, &subtree_ids);
    if !visible_ids.contains(&repair_root) {
        return None;
    }

    Some(StrataBucketRepairPlan {
        strata_idx,
        repair_root,
        replacement_segment: emit_visible_same_strata_segment(
            repair_root,
            strata_idx,
            &state.widgets,
            &visible_ids,
        ),
        subtree_ids,
    })
}

pub(super) fn splice_strata_bucket_repair(
    bucket: &mut Vec<u64>,
    repair_plan: StrataBucketRepairPlan,
) -> bool {
    let Some(start) = bucket.iter().position(|&id| id == repair_plan.repair_root) else {
        return false;
    };
    let end = same_strata_subtree_segment_end(bucket, start, &repair_plan.subtree_ids);
    bucket.splice(start..end, repair_plan.replacement_segment);
    true
}

fn same_strata_subtree_ids(
    repair_root: u64,
    strata_idx: usize,
    widgets: &crate::widget::WidgetRegistry,
) -> FxHashSet<u64> {
    let mut subtree_ids = FxHashSet::default();
    collect_same_strata_subtree_ids(repair_root, strata_idx, widgets, &mut subtree_ids);
    subtree_ids
}

fn visible_same_strata_subtree_ids(
    state: &SimState,
    subtree_ids: &FxHashSet<u64>,
) -> FxHashSet<u64> {
    subtree_ids
        .iter()
        .copied()
        .filter(|&id| {
            state
                .widgets
                .get(id)
                .is_some_and(|frame| state.frame_belongs_in_strata_bucket(id, frame))
        })
        .collect()
}

fn emit_visible_same_strata_segment(
    repair_root: u64,
    strata_idx: usize,
    widgets: &crate::widget::WidgetRegistry,
    visible_ids: &FxHashSet<u64>,
) -> Vec<u64> {
    let mut replacement_segment = Vec::new();
    dfs_emit(
        repair_root,
        strata_idx,
        widgets,
        visible_ids,
        &mut replacement_segment,
    );
    replacement_segment
}

#[cfg(test)]
mod tests {
    use super::build_strata_bucket_repair_plan;
    use crate::lua_api::state::SimState;
    use crate::widget::{Frame, FrameStrata, WidgetType};

    fn test_frame(
        id: u64,
        widget_type: WidgetType,
        parent_id: Option<u64>,
        visible: bool,
    ) -> Frame {
        let mut frame = Frame {
            id,
            widget_type,
            parent_id,
            visible,
            width: 10.0,
            height: 10.0,
            layout_rect: Some(crate::LayoutRect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            }),
            ..Default::default()
        };
        frame.effective_alpha = if visible { 1.0 } else { 0.0 };
        frame
    }

    fn register_child(
        state: &mut SimState,
        id: u64,
        widget_type: WidgetType,
        parent_id: u64,
        visible: bool,
    ) {
        state
            .widgets
            .register(test_frame(id, widget_type, Some(parent_id), visible));
        state.widgets.add_child(parent_id, id);
    }

    #[test]
    fn build_strata_bucket_repair_plan_collects_visible_replacement_segment() {
        let mut state = SimState::default();
        state
            .widgets
            .register(test_frame(30, WidgetType::Frame, None, true));
        register_child(&mut state, 31, WidgetType::Texture, 30, true);
        register_child(&mut state, 32, WidgetType::Texture, 30, false);
        register_child(&mut state, 33, WidgetType::Frame, 30, true);
        register_child(&mut state, 34, WidgetType::Texture, 33, true);
        register_child(&mut state, 35, WidgetType::FontString, 30, true);

        let repair_plan =
            build_strata_bucket_repair_plan(&state, 30).expect("visible root should repair");

        assert_eq!(repair_plan.strata_idx, FrameStrata::Medium.as_index());
        assert_eq!(repair_plan.repair_root, 30);
        assert_eq!(
            repair_plan.subtree_ids,
            rustc_hash::FxHashSet::from_iter([30, 31, 32, 33, 34, 35])
        );
        assert_eq!(repair_plan.replacement_segment, vec![30, 31, 33, 34, 35]);
    }
}
