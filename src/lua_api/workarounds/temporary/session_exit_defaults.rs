//! Existing CancelLogout placeholder, separated from modeled session actions.
//! Retire when a pending logout countdown/cancellation state machine is modeled.

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::{LuaApiMut, LuaResult};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let globals = lua.state_mut().global;
    table_set_rust_fn_static(lua.state_mut(), globals, "CancelLogout", |_| Ok(0))
}
