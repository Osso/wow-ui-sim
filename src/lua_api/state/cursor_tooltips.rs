use super::SimState;
use crate::lua_api::tooltip::build_cursor_anchor;
use crate::widget::Anchor;

pub(super) fn reanchor_visible_cursor_tooltips(state: &mut SimState, mx: f32, my: f32) {
    let cursor_tooltips = collect_cursor_tooltip_positions(state, mx, my);
    for (tooltip_id, anchor) in cursor_tooltips {
        reanchor_tooltip_to_cursor(state, tooltip_id, anchor);
    }
}

fn collect_cursor_tooltip_positions(state: &SimState, mx: f32, my: f32) -> Vec<(u64, Anchor)> {
    state
        .tooltips
        .iter()
        .filter_map(|(&tooltip_id, td)| {
            let frame = state.widgets.get(tooltip_id)?;
            (td.anchor_type.starts_with("ANCHOR_CURSOR") && frame.visible).then(|| {
                let anchor = build_cursor_anchor(
                    mx,
                    my,
                    td.anchor_x_offset,
                    td.anchor_y_offset,
                    &td.anchor_type,
                    state.screen_height,
                    frame.effective_scale,
                );
                (tooltip_id, anchor)
            })
        })
        .collect()
}

fn reanchor_tooltip_to_cursor(state: &mut SimState, tooltip_id: u64, anchor: Anchor) {
    if state
        .widgets
        .get(tooltip_id)
        .is_some_and(|frame| frame.anchors == [anchor.clone()])
    {
        return;
    }
    let Some(frame) = state.widgets.get_mut_visual(tooltip_id) else {
        return;
    };
    frame.anchors.clear();
    frame.anchors.push(anchor);
    let _ = frame;
    state.widgets.mark_rect_dirty(tooltip_id);
    state.widgets.mark_visual_dirty(tooltip_id);
}
