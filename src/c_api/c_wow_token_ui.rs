//! PTR-only auction-house visibility query for the WoW Token category.

use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_wow_token_ui(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Ptr {
        return Ok(());
    }

    let namespace = ensure_namespace(state, "C_WowTokenUI")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "ShouldDisplayInAuctionHouse",
        should_display_in_auction_house,
    )
}

fn should_display_in_auction_house(state: &mut LuaState) -> LuaResult<u32> {
    // R22's read-only PTR override is modeled as true before and after server status updates.
    state.push(Val::Bool(true));
    Ok(1)
}
