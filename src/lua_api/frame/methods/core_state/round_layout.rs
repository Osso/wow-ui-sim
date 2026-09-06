//! PTR native per-region layout rounding controls.

use super::helpers::{arg_bool, frame_id};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "GetRoundLayoutToNearestPixel", get_round_layout)?;
    table_set_rust_fn_static(state, mt, "SetRoundLayoutToNearestPixel", set_round_layout)
}

fn get_round_layout(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let enabled = borrow_state(state)?
        .widgets
        .get(id)
        .is_some_and(|frame| frame.round_layout_to_nearest_pixel);
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_round_layout(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let enabled = arg_bool(state, 2);
    let mut sim = borrow_state_mut(state)?;
    let Some(frame) = sim.widgets.get_mut_visual(id) else {
        return Ok(0);
    };
    frame.round_layout_to_nearest_pixel = enabled;
    sim.widgets.mark_rect_dirty(id);
    Ok(0)
}
