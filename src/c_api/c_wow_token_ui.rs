//! Auction House visibility for the WoW Token category.

use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !supports_token_ui() {
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

/// Apply a token-status update supplied by the simulated server.
#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
pub fn set_feature_status(env: &WowLuaEnv, enabled: bool) -> crate::Result<()> {
    env.state()
        .borrow_mut()
        .wow_token
        .auction_house_feature_enabled = enabled;
    env.fire_event("TOKEN_STATUS_CHANGED")
}

fn should_display_in_auction_house(state: &mut LuaState) -> LuaResult<u32> {
    // R22 verifies that PTR always displays the token entry, independent of updates.
    let should_display = if ACTIVE == ClientProfile::Ptr {
        true
    } else {
        borrow_state(state)?.wow_token.auction_house_feature_enabled
    };
    state.push(Val::Bool(should_display));
    Ok(1)
}

fn supports_token_ui() -> bool {
    matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever)
}
