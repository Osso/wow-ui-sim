//! `C_CharacterServices` probe surface backed by `SimState.character_services`.
//!
//! Active boost probes live here because they read modeled simulator state.
//! Static display/assignment defaults for unmodeled service flows stay in
//! `lua_api::workarounds::temporary::character_services_defaults`.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

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
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        // Normal initial registration precedes the existing protected wrappers.
        for (name, function) in [
            ("GetAutomaticBoost", get_automatic_boost as rilua::RustFn),
            (
                "GetAutomaticBoostCharacter",
                get_automatic_boost_character as rilua::RustFn,
            ),
            ("SetAutomaticBoost", set_automatic_boost as rilua::RustFn),
            (
                "SetAutomaticBoostCharacter",
                set_automatic_boost_character as rilua::RustFn,
            ),
        ] {
            table_set_rust_fn_static(state, table_ref, name, function)?;
        }
    }
    Ok(())
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

// R54: independent local session values, not a server purchase record.
fn get_automatic_boost(state: &mut LuaState) -> LuaResult<u32> {
    let boost = borrow_state(state)?.character_services.automatic_boost_type;
    if let Some(boost) = boost {
        state.push(Val::Num(f64::from(boost)));
        Ok(1)
    } else {
        // Absence returning no values is INFERRED; consumers use assignment nil.
        Ok(0)
    }
}

fn get_automatic_boost_character(state: &mut LuaState) -> LuaResult<u32> {
    let guid = borrow_state(state)?
        .character_services
        .automatic_boost_character_guid
        .clone();
    let result = guid
        .as_deref()
        .map(|guid| create_string(state, guid))
        .unwrap_or(Val::Nil);
    state.push(result);
    Ok(1)
}

fn set_automatic_boost(state: &mut LuaState) -> LuaResult<u32> {
    // Existing shared readers cover ordinary int32/nil. Broader native writer
    // conversions and errors are UNKNOWN and are not asserted by conformance tests.
    let boost = Option::<i32>::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .character_services
        .automatic_boost_type = boost;
    Ok(0)
}

fn set_automatic_boost_character(state: &mut LuaState) -> LuaResult<u32> {
    // Full native GUID grammar/format failure behavior is not modeled here.
    let guid = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .character_services
        .automatic_boost_character_guid = guid;
    Ok(0)
}
