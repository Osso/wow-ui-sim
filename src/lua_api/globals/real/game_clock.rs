//! Globals backed by the supplied game clock.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

fn get_game_time(state: &mut LuaState) -> LuaResult<u32> {
    let time = borrow_state(state)?.game_clock.time;
    let (hour, minute) = time
        .map(|(h, m)| (f64::from(h), f64::from(m)))
        .unwrap_or((-1.0, -1.0));
    state.push(Val::Num(hour));
    state.push(Val::Num(minute));
    Ok(2)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "GetGameTime", get_game_time)?;
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(state, state.global, "GetLocalGameTime", get_game_time)?;
    }
    Ok(())
}
