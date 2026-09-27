//! Pure scroll-child helpers shared by loader and template code.

use crate::lua_api::frame::methods::methods_hierarchy::reparent_widget;
use crate::widget::AnchorPoint;

pub(crate) fn assign_scroll_child(
    state: &mut crate::lua_api::SimState,
    parent_id: u64,
    child_id: u64,
    should_reparent: bool,
) {
    let old_child = state.widgets.get(parent_id).and_then(|f| f.scroll_child_id);
    if old_child != Some(child_id) {
        clear_scroll_child(state, parent_id);
    }
    invalidate_scroll_presentation(state, parent_id);
    if let Some(frame) = state.widgets.get_mut_visual(parent_id) {
        frame.scroll_child_id = Some(child_id);
        frame.scroll_child_rect_size = None;
    }
    if should_reparent {
        reparent_widget(&mut state.widgets, child_id, Some(parent_id));
    }
    anchor_scroll_child_to_parent_if_needed(state, parent_id, child_id);
    state.visible_on_update_cache = None;
    state.invalidate_layout(child_id);
    invalidate_scroll_presentation(state, parent_id);
}

pub(crate) fn clear_scroll_child(state: &mut crate::lua_api::SimState, parent_id: u64) {
    invalidate_scroll_presentation(state, parent_id);
    let old_child = state.widgets.get(parent_id).and_then(|f| f.scroll_child_id);
    if let Some(frame) = state.widgets.get_mut_visual(parent_id) {
        frame.scroll_child_id = None;
        frame.scroll_child_rect_size = None;
    }
    if let Some(child_id) = old_child {
        reparent_widget(&mut state.widgets, child_id, None);
        state.invalidate_layout(child_id);
        state.visible_on_update_cache = None;
    }
}

/// Scroll changes presentation only; dirty every affected strata and hit entry.
pub(crate) fn invalidate_scroll_presentation(
    state: &mut crate::lua_api::SimState,
    scroll_frame_id: u64,
) {
    let Some(child_id) = state
        .widgets
        .get(scroll_frame_id)
        .and_then(|f| f.scroll_child_id)
    else {
        return;
    };
    let mut pending = vec![child_id];
    while let Some(id) = pending.pop() {
        state.widgets.mark_visual_dirty(id);
        if let Some(frame) = state.widgets.get(id) {
            pending.extend_from_slice(&frame.children);
        }
    }
    state.pending_hit_grid_changes.push((child_id, true));
}

fn anchor_scroll_child_to_parent_if_needed(
    state: &mut crate::lua_api::SimState,
    parent_id: u64,
    child_id: u64,
) {
    let needs_anchor = state
        .widgets
        .get(child_id)
        .is_some_and(|child| child.anchors.is_empty());
    if !needs_anchor {
        return;
    }

    if let Some(child) = state.widgets.get_mut_visual(child_id) {
        child.set_point(
            AnchorPoint::TopLeft,
            Some(parent_id as usize),
            AnchorPoint::TopLeft,
            0.0,
            0.0,
        );
    }
    state.widgets.add_anchor_dependent(parent_id, child_id);
}
