//! C_WowTokenSecure — token buy/sell/redeem simulation with event firing.

use crate::lua_api::methods::{create_table, val_to_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

use crate::c_api::{ensure_global_table, global_val, set_global_val};

const WOWTOKEN_SECURE_FUNCTIONS: &[(&str, rilua::vm::closure::RustFn)] = &[
    ("CanRedeemForBalance", c_wowtoken_can_redeem_for_balance),
    ("CancelRedeem", c_wowtoken_cancel_redeem),
    ("ConfirmBuyToken", c_wowtoken_confirm_buy_token),
    ("ConfirmSellToken", c_wowtoken_confirm_sell_token),
    (
        "GetBalanceRedeemAmount",
        c_wowtoken_get_balance_redeem_amount,
    ),
    (
        "GetBalanceRedemptionInfo",
        c_wowtoken_get_balance_redemption_info,
    ),
    (
        "GetGameTimeRedemptionInfo",
        c_wowtoken_get_game_time_redemption_info,
    ),
    ("GetPriceLockDuration", c_wowtoken_get_price_lock_duration),
    ("GetRemainingGameTime", c_wowtoken_get_remaining_game_time),
    ("GetTokenCount", c_wowtoken_get_token_count),
    (
        "IsRedemptionStillValid",
        c_wowtoken_is_redemption_still_valid,
    ),
    ("RedeemToken", c_wowtoken_redeem_token),
    ("RedeemTokenConfirm", c_wowtoken_redeem_token_confirm),
    (
        "SetBalanceAmountString",
        c_wowtoken_set_balance_amount_string,
    ),
    ("WillKickFromWorld", c_wowtoken_will_kick_from_world),
];

pub fn register_c_wowtoken_secure(state: &mut LuaState) -> LuaResult<()> {
    wowtoken_state_table(state);
    let t = ensure_global_table(state, "C_WowTokenSecure");
    let Val::Table(t_ref) = t else {
        unreachable!("C_WowTokenSecure must be a table");
    };
    for (name, func) in WOWTOKEN_SECURE_FUNCTIONS {
        table_set_rust_fn_static(state, t_ref, name, *func)?;
    }
    Ok(())
}

fn wowtoken_state_table(state: &mut LuaState) -> Val {
    match global_val(state, "__wowtoken_state") {
        table @ Val::Table(_) => table,
        _ => init_wowtoken_state(state),
    }
}

fn init_wowtoken_state(state: &mut LuaState) -> Val {
    let table = create_table(state);
    let Val::Table(tbl_ref) = table else {
        unreachable!()
    };
    set_table_kv(state, tbl_ref, "tokenCount", Val::Num(2.0));
    set_table_kv(state, tbl_ref, "currentBalance", Val::Num(2500.0));
    set_table_kv(state, tbl_ref, "balanceRedeemAmount", Val::Num(1500.0));
    set_table_kv(state, tbl_ref, "cannotRedeemReason", Val::Num(0.0));
    set_table_kv(state, tbl_ref, "isSubscribed", Val::Bool(false));
    set_table_kv(state, tbl_ref, "remainingGameTime", Val::Num(1440.0));
    set_table_kv(state, tbl_ref, "pendingRedeemType", Val::Nil);
    set_table_kv(state, tbl_ref, "priceLockDuration", Val::Num(900.0));
    set_table_kv(state, tbl_ref, "willKickFromWorld", Val::Bool(false));
    set_global_val(state, "__wowtoken_state", table);
    table
}

fn set_table_kv(state: &mut LuaState, tbl_ref: GcRef<Table>, key: &str, value: Val) {
    let k = state.gc.intern_string(key.as_bytes());
    if let Some(t) = state.gc.tables.get_mut(tbl_ref) {
        let _ = t.raw_set(Val::Str(k), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(tbl_ref);
}

fn wowtoken_get(state: &mut LuaState, key: &str) -> Val {
    let token_state = wowtoken_state_table(state);
    let Val::Table(tbl_ref) = token_state else {
        return Val::Nil;
    };
    let k = state.gc.intern_string(key.as_bytes());
    state
        .gc
        .tables
        .get(tbl_ref)
        .map(|t| t.get_str(k, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

fn wowtoken_num(state: &mut LuaState, key: &str, default: f64) -> f64 {
    match wowtoken_get(state, key) {
        Val::Num(v) => v,
        _ => default,
    }
}

fn wowtoken_bool(state: &mut LuaState, key: &str, default: bool) -> bool {
    match wowtoken_get(state, key) {
        Val::Bool(v) => v,
        _ => default,
    }
}

fn wowtoken_pending_redeem_type(state: &mut LuaState) -> Option<i32> {
    match wowtoken_get(state, "pendingRedeemType") {
        Val::Num(v) => Some(v as i32),
        _ => None,
    }
}

fn wowtoken_set(state: &mut LuaState, key: &str, value: Val) {
    let token_state = wowtoken_state_table(state);
    let Val::Table(tbl_ref) = token_state else {
        return;
    };
    set_table_kv(state, tbl_ref, key, value);
}

fn wowtoken_set_num(state: &mut LuaState, key: &str, value: f64) {
    wowtoken_set(state, key, Val::Num(value));
}

fn wowtoken_set_bool(state: &mut LuaState, key: &str, value: bool) {
    wowtoken_set(state, key, Val::Bool(value));
}

fn wowtoken_set_pending_redeem_type(state: &mut LuaState, value: Option<i32>) {
    let v = value.map_or(Val::Nil, |n| Val::Num(n as f64));
    wowtoken_set(state, "pendingRedeemType", v);
}

fn first_bool_arg(state: &LuaState) -> bool {
    (1..=2)
        .find_map(|index| match stack_val(state, index) {
            Val::Bool(v) => Some(v),
            _ => None,
        })
        .unwrap_or(false)
}

fn first_num_arg(state: &LuaState) -> Option<i32> {
    (1..=2).find_map(|index| match stack_val(state, index) {
        Val::Num(v) => Some(v as i32),
        _ => None,
    })
}

fn first_string_arg(state: &LuaState) -> String {
    (1..=2)
        .find_map(|index| val_to_string(state, stack_val(state, index)))
        .unwrap_or_default()
}

fn parse_balance_amount(text: &str) -> Option<i64> {
    let digits_only: String = text.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits_only.len() >= 3 {
        return digits_only.parse().ok();
    }
    if digits_only.is_empty() {
        return None;
    }
    digits_only.parse::<i64>().ok().map(|dollars| dollars * 100)
}

fn c_wowtoken_can_redeem_for_balance(state: &mut LuaState) -> LuaResult<u32> {
    fire_named_event_state(state, "TOKEN_REDEEM_BALANCE_UPDATED", &[]);
    let result = if wowtoken_num(state, "tokenCount", 0.0) > 0.0 {
        0.0
    } else {
        1.0
    };
    state.push(Val::Num(result));
    Ok(1)
}

fn c_wowtoken_cancel_redeem(state: &mut LuaState) -> LuaResult<u32> {
    wowtoken_set_pending_redeem_type(state, None);
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_wowtoken_confirm_buy_token(state: &mut LuaState) -> LuaResult<u32> {
    let accepted = first_bool_arg(state);
    if !accepted {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    let token_count = wowtoken_num(state, "tokenCount", 0.0) + 1.0;
    wowtoken_set_num(state, "tokenCount", token_count);
    fire_named_event_state(state, "TOKEN_STATUS_CHANGED", &[]);
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_wowtoken_confirm_sell_token(state: &mut LuaState) -> LuaResult<u32> {
    let accepted = first_bool_arg(state);
    if !accepted {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    let token_count = wowtoken_num(state, "tokenCount", 0.0);
    if token_count > 0.0 {
        wowtoken_set_num(state, "tokenCount", token_count - 1.0);
    }
    fire_named_event_state(state, "TOKEN_STATUS_CHANGED", &[]);
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_wowtoken_get_balance_redeem_amount(state: &mut LuaState) -> LuaResult<u32> {
    let v = wowtoken_num(state, "balanceRedeemAmount", 1500.0);
    state.push(Val::Num(v));
    Ok(1)
}

fn c_wowtoken_get_balance_redemption_info(state: &mut LuaState) -> LuaResult<u32> {
    let current_balance = wowtoken_num(state, "currentBalance", 2500.0);
    let balance_redeem_amount = wowtoken_num(state, "balanceRedeemAmount", 1500.0);
    let token_count = wowtoken_num(state, "tokenCount", 0.0);
    let cannot_redeem_reason = wowtoken_num(state, "cannotRedeemReason", 0.0);
    state.push(Val::Num(current_balance));
    state.push(Val::Num(balance_redeem_amount));
    state.push(Val::Bool(token_count > 0.0));
    state.push(Val::Num(cannot_redeem_reason));
    Ok(4)
}

fn c_wowtoken_get_game_time_redemption_info(state: &mut LuaState) -> LuaResult<u32> {
    let is_subscribed = wowtoken_bool(state, "isSubscribed", false);
    let remaining_game_time = wowtoken_num(state, "remainingGameTime", 1440.0);
    state.push(Val::Bool(is_subscribed));
    state.push(Val::Num(remaining_game_time));
    Ok(2)
}

fn c_wowtoken_get_price_lock_duration(state: &mut LuaState) -> LuaResult<u32> {
    let v = wowtoken_num(state, "priceLockDuration", 900.0);
    state.push(Val::Num(v));
    Ok(1)
}

fn c_wowtoken_get_remaining_game_time(state: &mut LuaState) -> LuaResult<u32> {
    fire_named_event_state(state, "TOKEN_REDEEM_GAME_TIME_UPDATED", &[]);
    let v = wowtoken_num(state, "remainingGameTime", 1440.0);
    state.push(Val::Num(v));
    Ok(1)
}

fn c_wowtoken_get_token_count(state: &mut LuaState) -> LuaResult<u32> {
    let v = wowtoken_num(state, "tokenCount", 2.0);
    state.push(Val::Num(v));
    Ok(1)
}

fn c_wowtoken_is_redemption_still_valid(state: &mut LuaState) -> LuaResult<u32> {
    let pending_redeem_type = wowtoken_pending_redeem_type(state);
    let token_count = wowtoken_num(state, "tokenCount", 0.0);
    state.push(Val::Bool(
        pending_redeem_type.is_some() && token_count > 0.0,
    ));
    Ok(1)
}

fn c_wowtoken_redeem_token(state: &mut LuaState) -> LuaResult<u32> {
    let redeem_type = first_num_arg(state).unwrap_or(0);
    if wowtoken_num(state, "tokenCount", 0.0) <= 0.0 {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    wowtoken_set_pending_redeem_type(state, Some(redeem_type));
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_wowtoken_redeem_token_confirm(state: &mut LuaState) -> LuaResult<u32> {
    let redeem_type = first_num_arg(state).unwrap_or(0);
    if wowtoken_pending_redeem_type(state) != Some(redeem_type)
        || wowtoken_num(state, "tokenCount", 0.0) <= 0.0
    {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    wowtoken_set_pending_redeem_type(state, None);
    let token_count = wowtoken_num(state, "tokenCount", 0.0);
    wowtoken_set_num(state, "tokenCount", token_count - 1.0);
    match redeem_type {
        1 => confirm_game_time_redemption(state),
        2 => confirm_balance_redemption(state),
        _ => state.push(Val::Bool(false)),
    }
    Ok(1)
}

fn confirm_game_time_redemption(state: &mut LuaState) {
    wowtoken_set_bool(state, "isSubscribed", true);
    let remaining = wowtoken_num(state, "remainingGameTime", 1440.0);
    wowtoken_set_num(state, "remainingGameTime", remaining + 30.0 * 24.0 * 60.0);
    fire_named_event_state(state, "TOKEN_STATUS_CHANGED", &[]);
    fire_named_event_state(state, "TOKEN_REDEEM_GAME_TIME_UPDATED", &[]);
    state.push(Val::Bool(true));
}

fn confirm_balance_redemption(state: &mut LuaState) {
    let current_balance = wowtoken_num(state, "currentBalance", 2500.0);
    let balance_redeem_amount = wowtoken_num(state, "balanceRedeemAmount", 1500.0);
    wowtoken_set_num(
        state,
        "currentBalance",
        current_balance + balance_redeem_amount,
    );
    fire_named_event_state(state, "TOKEN_STATUS_CHANGED", &[]);
    fire_named_event_state(state, "TOKEN_REDEEM_BALANCE_UPDATED", &[]);
    state.push(Val::Bool(true));
}

fn c_wowtoken_set_balance_amount_string(state: &mut LuaState) -> LuaResult<u32> {
    let value = first_string_arg(state);
    if let Some(parsed_amount) = parse_balance_amount(&value) {
        wowtoken_set_num(state, "balanceRedeemAmount", parsed_amount as f64);
    }
    Ok(0)
}

fn c_wowtoken_will_kick_from_world(state: &mut LuaState) -> LuaResult<u32> {
    let v = wowtoken_bool(state, "willKickFromWorld", false);
    state.push(Val::Bool(v));
    Ok(1)
}
