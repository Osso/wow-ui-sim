//! Legacy global profiling helpers.

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn debug_profile_start(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn debug_profile_stop(state: &mut LuaState) -> LuaResult<u32> {
    let elapsed_ms = borrow_state(state)?.start_time.elapsed().as_secs_f64() * 1000.0;
    state.push(Val::Num(elapsed_ms));
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "debugprofilestart", debug_profile_start)?;
    LuaApiMut::register_function(lua, "debugprofilestop", debug_profile_stop)?;
    #[cfg(not(feature = "retail-12-0-0"))]
    crate::c_api::c_debug::register(lua.state_mut())?;
    Ok(())
}
