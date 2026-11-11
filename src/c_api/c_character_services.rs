//! `C_CharacterServices` probe surface backed by `SimState.character_services`.
//!
//! Active boost and queued-upgrade presence probes read modeled simulator state.
//! Static display/assignment defaults for unmodeled service flows stay in
//! `lua_api::workarounds::temporary::character_services_defaults`.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{borrow_state, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_c_character_services_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_CharacterServices")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetActiveCharacterUpgradeBoostType",
        get_active_character_upgrade_boost_type,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetActiveClassTrialBoostType",
        get_active_class_trial_boost_type,
    )?;
    Ok(())
}

/// Update the Glue-only queued-upgrade query for the selected screen.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return;
    }

    let mut lua = env.lua.borrow_mut();
    update_queued_upgrade_registration(lua.state_mut(), is_glue)
        .expect("register Glue-only C_CharacterServices.HasQueuedUpgrade for the selected screen");
}

fn update_queued_upgrade_registration(state: &mut LuaState, is_glue: bool) -> LuaResult<()> {
    let namespace = match LuaApiMut::get_global_val(state, "C_CharacterServices") {
        Val::Table(namespace) => namespace,
        _ if is_glue => ensure_namespace(state, "C_CharacterServices")?,
        _ => return Ok(()),
    };

    if is_glue {
        table_set_rust_fn_static(state, namespace, "HasQueuedUpgrade", has_queued_upgrade)
    } else {
        table_set_static(state, Val::Table(namespace), "HasQueuedUpgrade", Val::Nil);
        Ok(())
    }
}

fn has_queued_upgrade(state: &mut LuaState) -> LuaResult<u32> {
    let is_set = borrow_state(state)?
        .character_services
        .queued_upgrade_guid_is_set;
    state.push(Val::Bool(is_set));
    Ok(1)
}

fn get_active_character_upgrade_boost_type(state: &mut LuaState) -> LuaResult<u32> {
    let boost_type = borrow_state(state)?
        .character_services
        .active_upgrade_boost_type;
    match boost_type {
        Some(t) => state.push(Val::Num(t as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_active_class_trial_boost_type(state: &mut LuaState) -> LuaResult<u32> {
    let trial_type = borrow_state(state)?
        .character_services
        .active_class_trial_boost_type;
    match trial_type {
        Some(t) => state.push(Val::Num(t as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}
