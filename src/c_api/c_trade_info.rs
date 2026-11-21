//! Retail and WoW Forever trade-info behavior backed by simulator trade state.

use super::helpers::ensure_namespace;
use crate::lua_api::globals::real::cursor_model;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state_types::CursorInfo;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

const MAX_MONEY_AMOUNT: f64 = 9_007_199_254_740_991.0;
const CURSOR_MONEY_LIMIT: u64 = 100_000_000_000;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }

    let trade_info = ensure_namespace(state, "C_TradeInfo")?;
    // INFERRED (R88co) — share the trade-warning input already exposed by C_SecureTransfer.
    table_set_rust_fn_static(
        state,
        trade_info,
        "ShouldShowTradeOfferWarning",
        super::secure_transfer::should_show_trade_offer_warning,
    )?;
    table_set_rust_fn_static(state, trade_info, "PickupTradeMoney", pickup_trade_money)
}

/// INFERRED (R88co) — move the requested amount from the active offer to the cursor.
fn pickup_trade_money(state: &mut LuaState) -> LuaResult<u32> {
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
