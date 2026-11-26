//! Frame-level helper globals backed by the simulator frame model.
//!
//! Blizzard's UIParent panel manager defines these as tiny wrappers around
//! `frame:SetFrameLevel(frame:GetFrameLevel() +/- 1)`. Registering them here
//! keeps that common compatibility surface available without loading the full
//! panel-manager addon into profiles that do not otherwise use it.

use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::frame::propagate_strata_level_pub;
use crate::lua_api::methods::{borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub fn lower_frame_level(state: &mut LuaState) -> LuaResult<u32> {
    adjust_frame_level(state, -1, "LowerFrameLevel")
}

pub fn raise_frame_level(state: &mut LuaState) -> LuaResult<u32> {
    adjust_frame_level(state, 1, "RaiseFrameLevel")
}

fn adjust_frame_level(state: &mut LuaState, delta: i32, action: &str) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, action);
        return Ok(0);
    }

    let mut sim = borrow_state_mut(state)?;
    let Some(current_level) = sim.widgets.get(id).map(|frame| frame.frame_level) else {
        return Ok(0);
    };
    let next_level = current_level.saturating_add(delta);
    if next_level == current_level {
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.frame_level = next_level;
    }
    propagate_strata_level_pub(&mut sim.widgets, id, next_level - current_level);
    sim.invalidate_strata_buckets();
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    let g = state.global;
    table_set_rust_fn_static(state, g, "LowerFrameLevel", lower_frame_level)?;
    table_set_rust_fn_static(state, g, "RaiseFrameLevel", raise_frame_level)?;
    Ok(())
}
