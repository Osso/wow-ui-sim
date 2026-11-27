//! Runtime debug queries backed by rilua call state.

use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetCallstackHeight", get_callstack_height)
}

fn get_callstack_height(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(state.ci as f64));
    Ok(1)
}
