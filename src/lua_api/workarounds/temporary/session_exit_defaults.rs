//! Existing CancelLogout placeholder, separated from modeled session actions.
//! Retire when a pending logout countdown/cancellation state machine is modeled.

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::{LuaApiMut, LuaResult};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    // Retail/Forever now have the R54 accepted-path state clear in real/session_exit.
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let globals = lua.state_mut().global;
    table_set_rust_fn_static(lua.state_mut(), globals, "CancelLogout", |_| Ok(0))
}
