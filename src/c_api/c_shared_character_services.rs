//! Retail/PTR shared character-service promotional-upgrade inputs.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Host-supplied result of the native qualifying-promotion check.
#[derive(Debug, Default, Clone)]
pub struct SharedCharacterServicesState {
    pub has_free_promotional_upgrade: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(());
    }

    let namespace = ensure_namespace(state, "C_SharedCharacterServices")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "HasFreePromotionalUpgrade",
        has_free_promotional_upgrade,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "HasSeenFreePromotionalUpgradePopup",
        has_seen_free_promotional_upgrade_popup,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetPromotionalPopupSeen",
        set_promotional_popup_seen,
    )
}

fn has_free_promotional_upgrade(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?
        .shared_character_services
        .has_free_promotional_upgrade;
    state.push(Val::Bool(available));
    Ok(1)
}

fn has_seen_free_promotional_upgrade_popup(state: &mut LuaState) -> LuaResult<u32> {
    let seen = borrow_state(state)?
        .cvars
        .get_bool("seenAsiaCharacterUpgradePopup");
    state.push(Val::Bool(seen));
    Ok(1)
}

fn set_promotional_popup_seen(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(seen) = stack_val(state, 1) else {
        return Err(rilua::runtime_error("seen must be a boolean"));
    };
    borrow_state_mut(state)?.cvars.set(
        "seenAsiaCharacterUpgradePopup",
        if seen { "1" } else { "0" },
    );
    Ok(0)
}
