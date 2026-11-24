//! INFERRED (R88fx; native evidence pending): `GetTaxiMapID` reads the supplied taxi-map ID.

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetTaxiMapID", get_taxi_map_id)
}

fn get_taxi_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = borrow_state(state)?.taxi_map.ui_map_id;
    if map_id == 0 {
        return Ok(0);
    }
    state.push(Val::Num(f64::from(map_id)));
    Ok(1)
}
