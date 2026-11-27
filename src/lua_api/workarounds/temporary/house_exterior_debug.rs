//! Temporary house-exterior debug query payloads.
//!
//! VERIFIED: Retail's native publication table lists both methods in
//! `src/c_api/secure_publication_data.rs`. INFERRED R88g9: without a fixture
//! debug model, unavailable diagnostics return one nil value. Forever has no
//! matching publication evidence, so keep these methods absent there.

use crate::c_api::ensure_namespace;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::{LuaApiMut, LuaResult, Val, vm::state::LuaState};

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if cfg!(feature = "retail-12-0-0")
        && crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail
    {
        let state = lua.state_mut();
        let namespace = ensure_namespace(state, "C_HouseExterior")?;
        for name in ["GetFixtureDebugInfoForGUID", "GetSelectedFixtureDebugInfo"] {
            table_set_rust_fn_static(state, namespace, name, unavailable_debug_info)?;
        }
    }
    Ok(())
}

fn unavailable_debug_info(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}
