//! Host-supplied group eligibility observations, independent of catalog membership.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Missing keys have no supplied observation; `false` is an explicit observation.
pub type GroupEligibilityObservations = std::collections::HashMap<i32, bool>;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    // Each pinned StorePublicUIDocumentation.lua declares this query:
    // Retail/PTR/Forever:11-24; Mists/Era/Anniversary:11-23. Wrath has no evidence.
    if !matches!(
        ACTIVE,
        ClientProfile::Retail
            | ClientProfile::Ptr
            | ClientProfile::WowForever
            | ClientProfile::Mists
            | ClientProfile::Era
            | ClientProfile::Anniversary
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_StorePublic")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "DoesGroupHavePurchaseableProducts",
        query_group_eligibility,
    )?;
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(state, namespace, "IsEnabled", is_store_enabled)?;
    }
    Ok(())
}

fn is_store_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?
        .cvars
        .get_server_mirrored_bool("bpayStoreEnable")
        .ok_or_else(|| {
            rilua::runtime_error(
                "missing-input: C_StorePublic.IsEnabled has no supplied bpayStoreEnable value",
            )
        })?;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn query_group_eligibility(state: &mut LuaState) -> LuaResult<u32> {
    let group_id = i32::from_stack(state, 1)?;
    let observation = borrow_state(state)?
        .store_public_eligibility
        .get(&group_id)
        .copied();
    let eligible = observation.ok_or_else(|| {
        rilua::runtime_error(format!(
            "missing-input: C_StorePublic.DoesGroupHavePurchaseableProducts groupID={group_id} has no supplied eligibility observation"
        ))
    })?;
    state.push(Val::Bool(eligible));
    Ok(1)
}
