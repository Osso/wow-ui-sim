//! Character-creation naming policy shared with the local player name queries.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::WowForever {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_CharacterCreation")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "AreRegionalUniqueNamesEnabled",
        regional_unique_names_enabled,
    )
}

fn regional_unique_names_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.player.regional_unique_names_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}
