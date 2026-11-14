//! Shared publication for the existing combat-audio manager/CVar query.

use super::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let publishes_enabled = match ACTIVE {
        ClientProfile::Retail | ClientProfile::Ptr => cfg!(feature = "retail-12-0-0"),
        ClientProfile::WowForever => true,
        _ => false,
    };
    if !publishes_enabled {
        return Ok(());
    }

    // PTR 70493 and Forever 70707 CombatAudioAlertDocumentation.lua:112
    // declare this query. Sharing the existing provider is INFERRED.
    let namespace = ensure_namespace(state, "C_CombatAudioAlert")?;
    table_set_rust_fn_static(state, namespace, "IsEnabled", is_enabled)
}

/// VERIFIED (R88f.L.3, Retail 12.1.5.70688): an unavailable manager gives false;
/// otherwise `IsEnabled` reads `CAAEnabled` and returns false if that CVar is absent.
fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = {
        let sim = borrow_state(state)?;
        sim.sound_manager.is_some()
            && sim
                .cvars
                .cached_integer_nonzero("CAAEnabled")
                .unwrap_or(false)
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}
