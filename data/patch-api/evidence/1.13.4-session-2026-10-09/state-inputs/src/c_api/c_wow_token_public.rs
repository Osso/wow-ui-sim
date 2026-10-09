//! `C_WowTokenPublic` — commerce + market-price probe surface backed by
//! `SimState.wow_token`. Replaces the runtime Lua stub that returned
//! hardcoded zeros.
//!
//! Methods:
//! - `GetCommerceSystemStatus()` → `(commerceEnabled, pollSeconds, balanceEnabled)`
//! - `UpdateTokenCount()` → no-op refresh hook
//! - `GetCurrentMarketPrice()` → `(price, price)` (retail returns the
//!   guaranteed price twice — once as "current", once as the
//!   internally-cached value)
//! - `GetGuaranteedPrice()` → `price`
//! - `BuyToken()` → fires `TOKEN_BUY_RESULT(Success)` and increments
//!   the player's owned-token count
//! - `UpdateListedAuctionableTokens()` → no-op (live client refreshes
//!   the listing list; sim returns whatever was seeded)
//! - `UpdateMarketPrice()` → fires `TOKEN_MARKET_PRICE_UPDATED(Success)`
//! - `IsAuctionableWowToken(itemID)` → `true` for the canonical token
//!   item id (122270), `false` otherwise

use crate::c_api::ensure_namespace;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Canonical WoW Token item id. `IsAuctionableWowToken` returns true
/// only for this id; everything else is treated as a non-token.
const WOW_TOKEN_ITEM_ID: i32 = 122270;

/// `Enum.LE_TOKEN_RESULT_TYPE.LE_TOKEN_RESULT_SUCCESS`. The token
/// panel only branches on success vs. non-zero, so we always emit
/// success in the sim.
const TOKEN_RESULT_SUCCESS: f64 = 0.0;

type WowTokenPublicMethod = fn(&mut LuaState) -> LuaResult<u32>;

const WOW_TOKEN_PUBLIC_METHODS: &[(&str, WowTokenPublicMethod)] = &[
    (
        "GetCommerceSystemStatus",
        c_wow_token_public_get_commerce_system_status,
    ),
    ("UpdateTokenCount", c_wow_token_public_update_token_count),
    (
        "GetCurrentMarketPrice",
        c_wow_token_public_get_current_market_price,
    ),
    (
        "GetGuaranteedPrice",
        c_wow_token_public_get_guaranteed_price,
    ),
    ("BuyToken", c_wow_token_public_buy_token),
    (
        "UpdateListedAuctionableTokens",
        c_wow_token_public_update_listed_auctionable_tokens,
    ),
    #[cfg(feature = "client-mists")]
    (
        "GetNumListedAuctionableTokens",
        c_wow_token_public_get_num_listed_auctionable_tokens,
    ),
    #[cfg(feature = "client-mists")]
    (
        "GetListedAuctionableTokenInfo",
        c_wow_token_public_get_listed_auctionable_token_info,
    ),
    ("UpdateMarketPrice", c_wow_token_public_update_market_price),
    (
        "IsAuctionableWowToken",
        c_wow_token_public_is_auctionable_wow_token,
    ),
];

pub fn register_c_wow_token_public(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_WowTokenPublic")?;
    for &(name, func) in WOW_TOKEN_PUBLIC_METHODS {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn c_wow_token_public_get_commerce_system_status(state: &mut LuaState) -> LuaResult<u32> {
    let token = borrow_state(state)?.wow_token.clone();
    state.push(Val::Bool(token.commerce_enabled));
    state.push(Val::Num(token.poll_seconds as f64));
    state.push(Val::Bool(token.balance_enabled));
    Ok(3)
}

fn c_wow_token_public_update_token_count(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn c_wow_token_public_get_current_market_price(state: &mut LuaState) -> LuaResult<u32> {
    let price = borrow_state(state)?.wow_token.current_market_price as f64;
    state.push(Val::Num(price));
    state.push(Val::Num(price));
    Ok(2)
}

fn c_wow_token_public_get_guaranteed_price(state: &mut LuaState) -> LuaResult<u32> {
    let price = borrow_state(state)?.wow_token.guaranteed_price as f64;
    state.push(Val::Num(price));
    Ok(1)
}

fn c_wow_token_public_buy_token(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.wow_token.owned_token_count += 1;
    }
    dispatch_event_now(state, "TOKEN_BUY_RESULT", &[Val::Num(TOKEN_RESULT_SUCCESS)])?;
    Ok(0)
}

fn c_wow_token_public_update_listed_auctionable_tokens(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

#[cfg(feature = "client-mists")]
fn c_wow_token_public_get_num_listed_auctionable_tokens(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.wow_token.listed_auctionable.len() as f64;
    state.push(Val::Num(count));
    Ok(1)
}

#[cfg(feature = "client-mists")]
fn c_wow_token_public_get_listed_auctionable_token_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    if index < 1 {
        return Ok(0);
    }
    let token = borrow_state(state)?
        .wow_token
        .listed_auctionable
        .get((index - 1) as usize)
        .cloned();
    let Some(token) = token else { return Ok(0) };
    state.push(Val::Num(token.auction_id as f64));
    state.push(Val::Num(token.price as f64));
    Ok(2)
}

fn c_wow_token_public_update_market_price(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(
        state,
        "TOKEN_MARKET_PRICE_UPDATED",
        &[Val::Num(TOKEN_RESULT_SUCCESS)],
    )?;
    Ok(0)
}

fn c_wow_token_public_is_auctionable_wow_token(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    state.push(Val::Bool(item_id == WOW_TOKEN_ITEM_ID));
    Ok(1)
}
