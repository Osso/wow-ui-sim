//! INFERRED world presence from the simulator startup/lifecycle model.
//!
//! Original SpellBookFrameMixin:OnLoad defers in-world subscriptions until
//! PLAYER_ENTERING_WORLD. Native flag mapping remains UNKNOWN (R88l.3).
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn is_player_in_world(state: &mut LuaState) -> LuaResult<u32> {
    let in_world = borrow_state(state)?.in_world;
    state.push(Val::Bool(in_world));
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "IsPlayerInWorld", is_player_in_world)
}
