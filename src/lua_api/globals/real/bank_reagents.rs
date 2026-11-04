//! PTR bank-reagent decision inputs (WM92, PTR 70668, not pinned 70493).

/// UNKNOWN universal character default. EMULATED game scenario: a present
/// shared object with its restrictive-rule flag clear. Fixtures supply this
/// explicitly; no bank-open, location, ownership or server inference is made.
pub(crate) fn unknown_character_default() -> Option<bool> {
    Some(false)
}

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    if ACTIVE == ClientProfile::Ptr {
        // Pinned PTR PlayerScriptDocumentation.lua:1611–1618.
        lua.state_mut()
            .register_function("ReagentsFromBankAllowed", query)?;
    }
    Ok(())
}

fn query(state: &mut LuaState) -> LuaResult<u32> {
    let allowed = {
        let sim = borrow_state(state)?;
        sim.bank_reagent_object.is_some_and(|flag| {
            !sim.game_rules.rules.contains_key("CraftFromBankRestricted") || flag
        })
    };
    publish_unknown_native_policy(state, allowed)
}

/// UNKNOWN native secret/publication policy. EMULATED ordinary Boolean using
/// current execution provenance; no forced secret marker or taint reset.
fn publish_unknown_native_policy(state: &mut LuaState, allowed: bool) -> LuaResult<u32> {
    state.push(Val::Bool(allowed));
    Ok(1)
}
