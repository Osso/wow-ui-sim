//! Retail trade warning state supplied by the simulator host.

use super::helpers::{ensure_namespace, global_val};
use crate::lua_api::methods::{create_table, table_get, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return Ok(());
    }

    let trade_info = ensure_namespace(state, "C_TradeInfo")?;
    let trade_info_value = Val::Table(trade_info);
    if !matches!(table_get(state, trade_info_value, "_state"), Val::Table(_)) {
        let input = create_table(state);
        table_set(state, trade_info_value, "_state", input);
    }
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

/// VERIFIED — both Retail getters read one host-supplied client-local value.
/// Retail `Blizzard_EnvironmentCleanup/EnvironmentCleanup.lua:6` clears
/// `C_SecureTransfer` after login, so the slot lives under the persistent
/// `C_TradeInfo._state` namespace. Its absence keeps the existing simulator
/// false result; this does not claim a native initial value.
fn should_show_trade_offer_warning(state: &mut LuaState) -> LuaResult<u32> {
    let namespace = global_val(state, "C_TradeInfo");
    let state_table = table_get(state, namespace, "_state");
    let value = table_get(state, state_table, "shouldShowTradeOfferWarning");
    state.push(Val::Bool(value == Val::Bool(true)));
    Ok(1)
}
