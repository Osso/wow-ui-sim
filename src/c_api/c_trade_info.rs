//! Retail trade state supplied by the host; the PTR path is inferred from Retail.

use super::helpers::{ensure_namespace, global_val};
use crate::lua_api::globals::real::cursor_model;
use crate::lua_api::methods::{borrow_state_mut, create_table, table_get, table_set};
use crate::lua_api::state_types::CursorInfo;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const MAX_MONEY_AMOUNT: f64 = 9_007_199_254_740_991.0;
const CURSOR_MONEY_LIMIT: u64 = 100_000_000_000;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
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
    table_set_rust_fn_static(state, trade_info, "PickupTradeMoney", pickup_trade_money)?;

    let secure_transfer = ensure_namespace(state, "C_SecureTransfer")?;
    table_set_rust_fn_static(
        state,
        secure_transfer,
        "ShouldShowTradeOfferWarning",
        should_show_trade_offer_warning,
    )
}

/// VERIFIED — both Retail getters read the same client-local warning value.
/// INFERRED — this simulator stores the host input under `C_TradeInfo._state`.
/// Retail `Blizzard_EnvironmentCleanup/EnvironmentCleanup.lua:6` clears
/// `C_SecureTransfer` after login, so the input uses the persistent
/// `C_TradeInfo` namespace. An absent input keeps the existing simulator false
/// result; this does not claim a native initial value.
fn should_show_trade_offer_warning(state: &mut LuaState) -> LuaResult<u32> {
    let namespace = global_val(state, "C_TradeInfo");
    let state_table = table_get(state, namespace, "_state");
    let value = table_get(state, state_table, "shouldShowTradeOfferWarning");
    // INFERRED — publish the declared boolean result from the host input.
    state.push(Val::Bool(value == Val::Bool(true)));
    Ok(1)
}

/// INFERRED — the host Boolean represents the combined native restriction
/// predicate. Its metadata and cursor-category inputs are not modeled.
fn pickup_restricted(state: &mut LuaState) -> bool {
    let namespace = global_val(state, "C_TradeInfo");
    let host_state = table_get(state, namespace, "_state");
    table_get(state, host_state, "pickupTradeMoneyRestricted") == Val::Bool(true)
}

fn pickup_trade_money(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED — check the supplied restriction result before reading arguments.
    if pickup_restricted(state) {
        // UNKNOWN — the native restriction-report side effect is not modeled.
        return Ok(0);
    }

    let amount = read_money_amount(state)?;
    {
        let mut sim = borrow_state_mut(state)?;
        let Some(trade) = sim.active_trade.as_mut() else {
            // INFERRED — return no values without a host-supplied trade;
            // native no-trade behavior is unknown.
            return Ok(0);
        };
        if amount > trade.player_money {
            // VERIFIED — an amount above the offer skips removal and cursor handling.
            // INFERRED — this simulator returns no values; native response is unknown.
            return Ok(0);
        }
        if amount > 0 {
            trade.player_money -= amount;
            // UNKNOWN — packet submission, trade flag, optional hook and
            // change counter are not modeled.
        }
    }

    // VERIFIED — update the offer before attempting the cursor setter.
    set_trade_money_cursor(state, amount)?;
    // INFERRED — the simulator returns no values on success; native return count is unknown.
    Ok(0)
}

fn read_money_amount(state: &LuaState) -> LuaResult<u64> {
    let Some(amount) = Option::<f64>::from_stack(state, 1)? else {
        // VERIFIED — the native parser has a required-argument error path;
        // exact error details are unknown.
        return Err(runtime_error("PickupTradeMoney requires a money amount"));
    };
    if !amount.is_finite() || !(0.0..=MAX_MONEY_AMOUNT).contains(&amount) {
        // VERIFIED — the converter rejects this numeric range.
        // INFERRED — this simulator surfaces conversion failure as a Lua error.
        return Err(runtime_error(
            "PickupTradeMoney amount is outside the supported money range",
        ));
    }
    // VERIFIED — check the range first, then truncate toward zero.
    Ok(amount.trunc() as u64)
}

/// INFERRED — use the simulator's generic money cursor path below the verified
/// exclusive bound. Other native amount, mode and security checks are unknown.
fn set_trade_money_cursor(state: &mut LuaState, amount: u64) -> LuaResult<()> {
    if amount == 0 || amount >= CURSOR_MONEY_LIMIT {
        return Ok(());
    }
    cursor_model::place(state, CursorInfo::Money { copper: amount })
}
