//! `C_SocialContractGlue`: per-account social-contract acceptance.
//!
//! Best guess, native UNKNOWN: the query answers through
//! `SOCIAL_CONTRACT_STATUS_UPDATE(shouldShow)`, delivered synchronously here.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Wrath {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_SocialContractGlue")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetShouldShowSocialContract",
        get_should_show_social_contract,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "TryUpdateSocialContract",
        try_update_social_contract,
    )
}

fn get_should_show_social_contract(state: &mut LuaState) -> LuaResult<u32> {
    let should_show = !borrow_state(state)?.social_contract_accepted;
    fire_named_event_state(
        state,
        "SOCIAL_CONTRACT_STATUS_UPDATE",
        &[Val::Bool(should_show)],
    );
    Ok(0)
}

fn try_update_social_contract(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.social_contract_accepted = true;
    Ok(0)
}
