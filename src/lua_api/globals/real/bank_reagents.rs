//! PTR and Forever bank-reagent permission query.

/// UNKNOWN universal character default. EMULATED game scenario: a present
/// player object with its permission bit clear and no active restriction rule.
/// Fixtures supply this state; no bank-open, location, ownership or server
/// inference is made.
pub(crate) fn unknown_character_default() -> Option<bool> {
    Some(false)
}

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::borrow_state;
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        lua.state_mut()
            .register_function("ReagentsFromBankAllowed", query)?;
    }
    Ok(())
}

/// Apply Forever's Game installer surface to the selected screen.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) {
    if ACTIVE != ClientProfile::WowForever {
        return;
    }

    let mut lua = env.lua.borrow_mut();
    let registration = if is_glue {
        LuaApiMut::set_global_val(&mut *lua, "ReagentsFromBankAllowed", Val::Nil)
    } else {
        LuaApiMut::register_function(&mut *lua, "ReagentsFromBankAllowed", query)
    };
    registration.expect("register ReagentsFromBankAllowed for the selected screen");
}

fn query(state: &mut LuaState) -> LuaResult<u32> {
    let allowed = {
        let sim = borrow_state(state)?;
        sim.bank_reagent_object.is_some_and(|permission| {
            !sim.game_rules.rules.contains_key("CraftFromBankRestricted") || permission
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
