//! Retail trade APIs backed by the simulator's supplied trade and cursor state.

use super::helpers::{ensure_namespace, global_val};
use crate::lua_api::methods::table_get;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return Ok(());
    }

    let trade_info = ensure_namespace(state, "C_TradeInfo")?;
    table_set_rust_fn_static(
        state,
        trade_info,
        "ShouldShowTradeOfferWarning",
        should_show_trade_offer_warning,
    )?;

    let secure_transfer = ensure_namespace(state, "C_SecureTransfer")?;
    table_set_rust_fn_static(
        state,
        secure_transfer,
        "ShouldShowTradeOfferWarning",
        should_show_trade_offer_warning,
    )
}

/// VERIFIED — both Retail getters read the same client-local warning slot.
/// The temporary SecureTransfer bootstrap supplies the simulator's existing
/// false value when no host value is set; that is not a native-default claim.
fn should_show_trade_offer_warning(state: &mut LuaState) -> LuaResult<u32> {
    let namespace = global_val(state, "C_SecureTransfer");
    let state_table = table_get(state, namespace, "_state");
    let value = table_get(state, state_table, "shouldShowTradeOfferWarning");
    state.push(Val::Bool(value == Val::Bool(true)));
    Ok(1)
}
