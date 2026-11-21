//! Retail trade state supplied by the host; the PTR path is inferred from Retail.

use super::helpers::ensure_namespace;
use crate::lua_api::globals::real::cursor_model;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state_types::CursorInfo;
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
#[cfg(feature = "client-ptr")]
use rilua::Val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

const MAX_MONEY_AMOUNT: f64 = 9_007_199_254_740_991.0;
const CURSOR_MONEY_LIMIT: u64 = 100_000_000_000;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }

    let trade_info = ensure_namespace(state, "C_TradeInfo")?;
    table_set_rust_fn_static(state, trade_info, "PickupTradeMoney", pickup_trade_money)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    {
        // VERIFIED — both Retail getters read the same client-local warning value.
        // INFERRED (R88co) — Forever's selected TradeFrame uses this shared warning source.
        table_set_rust_fn_static(
            state,
            trade_info,
            "ShouldShowTradeOfferWarning",
            super::secure_transfer::should_show_trade_offer_warning,
        )?;
    }
    #[cfg(feature = "client-ptr")]
    {
        table_set_rust_fn_static(
            state,
            trade_info,
            "ShouldShowTradeOfferWarning",
            should_show_ptr_trade_offer_warning,
        )?;
        let secure_transfer = ensure_namespace(state, "C_SecureTransfer")?;
        table_set_rust_fn_static(
            state,
            secure_transfer,
            "ShouldShowTradeOfferWarning",
            should_show_ptr_trade_offer_warning,
        )?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    let builder = builder
        .set_function(
            "SetTradeMoneyPickupRestricted",
            set_pickup_trade_money_restricted,
        )?
        .set_function("SetTradeInfoCursorMoney", set_trade_info_cursor_money)?;
    #[cfg(feature = "client-ptr")]
    let builder = builder.set_function(
        "SetSecureTransferTradeOfferWarning",
        set_ptr_trade_offer_warning,
    )?;
    Ok(builder)
}

#[cfg(feature = "client-ptr")]
/// INFERRED — the PTR host input is stored in Rust; its native initial value is unknown.
fn should_show_ptr_trade_offer_warning(state: &mut LuaState) -> LuaResult<u32> {
    let show_warning = borrow_state(state)?.trade_offer_warning;
    state.push(Val::Bool(show_warning));
    Ok(1)
}

#[cfg(feature = "client-ptr")]
fn set_ptr_trade_offer_warning(state: &mut LuaState) -> LuaResult<u32> {
    let show_warning = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.trade_offer_warning = show_warning;
    Ok(0)
}

fn pickup_restricted(state: &LuaState) -> LuaResult<bool> {
    Ok(borrow_state(state)?.pickup_trade_money_restricted)
}

fn set_pickup_trade_money_restricted(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.pickup_trade_money_restricted = restricted;
    Ok(0)
}

fn set_trade_info_cursor_money(state: &mut LuaState) -> LuaResult<u32> {
    let amount = read_money_amount(state, "A_Admin.SetTradeInfoCursorMoney")?;
    cursor_model::place(state, CursorInfo::Money { copper: amount })?;
    Ok(0)
}

fn pickup_trade_money(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED — check the supplied restriction result before reading arguments.
    if pickup_restricted(state)? {
        // UNKNOWN — the native restriction-report side effect is not modeled.
        return Ok(0);
    }

    let amount = read_money_amount(state, "PickupTradeMoney")?;
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

fn read_money_amount(state: &LuaState, api: &str) -> LuaResult<u64> {
    let Some(amount) = Option::<f64>::from_stack(state, 1)? else {
        // VERIFIED — the native parser has a required-argument error path;
        // exact error details are unknown.
        return Err(runtime_error(format!("{api} requires a money amount")));
    };
    if !amount.is_finite() || !(0.0..=MAX_MONEY_AMOUNT).contains(&amount) {
        // VERIFIED — the converter rejects this numeric range.
        // INFERRED — this simulator surfaces conversion failure as a Lua error.
        return Err(runtime_error(format!(
            "{api} amount is outside the supported money range"
        )));
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
