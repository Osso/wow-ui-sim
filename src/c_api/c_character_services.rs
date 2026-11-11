//! `C_CharacterServices` probe surface backed by `SimState.character_services`.
//!
//! Active boost probes live here because they read modeled simulator state.
//! Static display/assignment defaults for unmodeled service flows stay in
//! `lua_api::workarounds::temporary::character_services_defaults`.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Host-supplied state queried by `C_CharacterServices`.
#[derive(Debug, Default, Clone)]
pub struct CharacterServicesState {
    /// Active character-upgrade boost type id, or `None` when no boost
    /// purchase is pending. Retail values include 5 for the level-60 boost.
    pub active_upgrade_boost_type: Option<i32>,
    /// Active class-trial boost type id, or `None` when no trial is running.
    pub active_class_trial_boost_type: Option<i32>,
    /// True when the queued-upgrade GUID's type bits are nonzero.
    pub queued_upgrade_guid_type_nonzero: bool,
}

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
    let screen_kind = borrow_state(state)?.screen_kind;
    update_queued_upgrade_registration(state, screen_kind)
}

pub(crate) fn update_queued_upgrade_registration(
    state: &mut LuaState,
    screen_kind: crate::screen::ScreenKind,
) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CharacterServices")?;
    if screen_kind.is_glue()
        && matches!(
            ACTIVE,
            ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
        )
    {
        table_set_rust_fn_static(state, namespace, "HasQueuedUpgrade", has_queued_upgrade)
    } else {
        table_set(state, Val::Table(namespace), "HasQueuedUpgrade", Val::Nil);
        Ok(())
    }
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

fn has_queued_upgrade(state: &mut LuaState) -> LuaResult<u32> {
    let queued = borrow_state(state)?
        .character_services
        .queued_upgrade_guid_type_nonzero;
    state.push(Val::Bool(queued));
    Ok(1)
}
