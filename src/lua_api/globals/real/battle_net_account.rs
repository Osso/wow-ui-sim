//! INFERRED local Battle.net account identity from supplied simulator state.

use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "BNGetInfo", get_info)
}

fn get_info(state: &mut LuaState) -> LuaResult<u32> {
    let (info, eligible) = {
        let sim = borrow_state(state)?;
        (
            sim.player_bnet_info.clone(),
            sim.bnet_connection.features_enabled && sim.player_bnet_account_present,
        )
    };
    if !eligible {
        return Ok(0);
    }
    let battle_tag = info
        .battle_tag
        .as_deref()
        .map_or(Val::Nil, |value| create_string(state, value));
    let broadcast_text = create_string(state, &info.broadcast_text);
    state.push(Val::Nil);
    state.push(battle_tag);
    state.push(Val::Num(info.account_id as f64));
    state.push(broadcast_text);
    state.push(Val::Bool(info.is_afk));
    state.push(Val::Bool(info.is_dnd));
    state.push(Val::Bool(info.is_real_id_enabled));
    state.push(Val::Bool(info.is_appear_offline));
    Ok(8)
}
