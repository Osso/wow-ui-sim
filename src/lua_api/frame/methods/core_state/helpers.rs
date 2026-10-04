//! Private helpers shared across core_state submodules.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn frame_id(state: &LuaState, index: i32) -> LuaResult<u64> {
    crate::lua_api::methods::frame_id_from_stack(state, index)
}

pub(super) fn arg_bool(state: &LuaState, index: i32) -> bool {
    use crate::lua_bridge::FromStack;
    bool::from_stack(state, index).unwrap_or(false)
}

pub(super) fn opt_f32(state: &LuaState, index: i32) -> f32 {
    match stack_val(state, index) {
        Val::Num(n) => n as f32,
        _ => 0.0,
    }
}

pub(super) use crate::widget::has_queryable_rect;

fn raw_frame_size(state: &crate::lua_api::state::SimState, id: u64) -> (f32, f32) {
    state
        .widgets
        .get(id)
        .map(|frame| (frame.width, frame.height))
        .unwrap_or((0.0, 0.0))
}

pub(super) fn frame_size(state: &mut LuaState, id: u64, raw: bool) -> LuaResult<(f32, f32)> {
    let mut sim = borrow_state_mut(state)?;
    if !raw
        && sim
            .widgets
            .get(id)
            .is_some_and(|frame| has_queryable_rect(frame, id))
    {
        sim.resolve_rect_if_dirty(id);
    }
    let size = if raw {
        raw_frame_size(&sim, id)
    } else {
        sim.widgets.resolved_size(id)
    };
    Ok(size)
}

pub(super) struct ExplicitSizeState {
    pub width: f32,
    pub height: f32,
    pub width_is_text_auto: bool,
    pub height_is_text_auto: bool,
}

pub(super) fn current_explicit_size_state(
    state: &crate::lua_api::state::SimState,
    id: u64,
) -> Option<ExplicitSizeState> {
    state.widgets.get(id).map(|frame| ExplicitSizeState {
        width: frame.width,
        height: frame.height,
        width_is_text_auto: frame.width_is_text_auto,
        height_is_text_auto: frame.height_is_text_auto,
    })
}

pub(super) fn clear_auto_width_flag(state: &mut crate::lua_api::state::SimState, id: u64) {
    if let Some(frame) = state.widgets.get_mut(id) {
        frame.width_is_text_auto = false;
    }
}

pub(super) fn clear_auto_height_flag(state: &mut crate::lua_api::state::SimState, id: u64) {
    if let Some(frame) = state.widgets.get_mut(id) {
        frame.height_is_text_auto = false;
    }
}

pub(super) fn apply_explicit_size(
    state: &mut crate::lua_api::state::SimState,
    id: u64,
    width: f32,
    height: f32,
) {
    if let Some(frame) = state.widgets.get_mut_visual(id) {
        frame.set_size(width, height);
        frame.width_is_text_auto = false;
        frame.height_is_text_auto = false;
    }
    state.widgets.mark_rect_dirty(id);
}

pub(super) fn apply_explicit_width(
    state: &mut crate::lua_api::state::SimState,
    id: u64,
    width: f32,
) {
    if let Some(frame) = state.widgets.get_mut_visual(id) {
        frame.width = width;
        frame.width_is_text_auto = false;
    }
    state.widgets.mark_rect_dirty(id);
}

pub(super) fn apply_explicit_height(
    state: &mut crate::lua_api::state::SimState,
    id: u64,
    height: f32,
) {
    if let Some(frame) = state.widgets.get_mut_visual(id) {
        frame.height = height;
        frame.height_is_text_auto = false;
    }
    state.widgets.mark_rect_dirty(id);
}
