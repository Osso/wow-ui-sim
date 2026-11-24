//! INFERRED local Battle.net account identity from supplied simulator state.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "BNGetInfo", get_info)
}

fn get_info(state: &mut LuaState) -> LuaResult<u32> {
    let info = borrow_state(state)?.player_bnet_info.clone();
    state.push(Val::Nil);
    state.push(Val::Str(info.battle_tag.into()));
    state.push(Val::Num(info.account_id as f64));
    state.push(Val::Str(info.broadcast_text.into()));
    state.push(Val::Bool(info.is_afk));
    state.push(Val::Bool(info.is_dnd));
    state.push(Val::Bool(info.is_real_id_enabled));
    state.push(Val::Bool(info.is_appear_offline));
    Ok(8)
}
