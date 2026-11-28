//! Trade UI verbs that drive `SimState.active_trade` and dispatch
//! `TRADE_*` events.
//!
//! Migrates 5 entries off `GLOBAL_NIL_STUBS`:
//!
//! - `InitiateTrade(unit)`       — open a trade with `unit`; fires
//!                                    `TRADE_SHOW`. No-op when already
//!                                    trading.
//! - `AcceptTrade()`             — flag player-accepted. When both sides
//!                                    accept, finalize: clear `active_trade`
//!                                    and fire `TRADE_CLOSED`.
//!                                    Fires `TRADE_ACCEPT_UPDATE` on every
//!                                    accept click.
//! - `CancelTrade()`             — close the trade; fire `TRADE_CLOSED`.
//! - `SetTradeCurrency(copper)`  — set player_money.
//! - `SetCursorItemSlot(slot)`   — if cursor carries an Item, move it to
//!                                    player_slots[slot-1]; otherwise
//!                                    silent no-op.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::c_api::ensure_namespace;
use crate::event::Event;
use crate::items;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::state::TradeState;
use crate::lua_api::state_types::CursorInfo;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const TRADE_SLOTS: usize = 7;

fn push_event(state: &mut LuaState, name: &str) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: name.to_string(),
        args: Vec::new(),
    });
    Ok(())
}

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_u64(state: &mut LuaState, index: i32) -> Option<u64> {
    match stack_val(state, index) {
        Val::Num(n) if n >= 0.0 => Some(n as u64),
        _ => None,
    }
}

fn required_string(state: &mut LuaState, index: i32) -> Option<String> {
    Option::<String>::from_stack(state, index)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
}

/// `InitiateTrade(unit)` — open a trade. Silent no-op when already trading.
fn initiate_trade(state: &mut LuaState) -> LuaResult<u32> {
    let Some(target) = required_string(state, 1) else {
        return Ok(0);
    };
    let opened = {
        let mut st = borrow_state_mut(state)?;
        if st.active_trade.is_some() {
            false
        } else {
            st.active_trade = Some(TradeState {
                target,
                ..TradeState::default()
            });
            true
        }
    };
    if opened {
        push_event(state, "TRADE_SHOW")?;
    }
    Ok(0)
}

/// `AcceptTrade()` — flag player-accepted. Fires `TRADE_ACCEPT_UPDATE`;
/// finalizes when both sides accept, firing `TRADE_CLOSED`.
fn accept_trade(state: &mut LuaState) -> LuaResult<u32> {
    let finalize = {
        let mut st = borrow_state_mut(state)?;
        let Some(trade) = st.active_trade.as_mut() else {
            return Ok(0);
        };
        trade.player_accepted = true;
        let both = trade.player_accepted && trade.target_accepted;
        if both {
            st.active_trade = None;
        }
        both
    };
    push_event(state, "TRADE_ACCEPT_UPDATE")?;
    if finalize {
        push_event(state, "TRADE_CLOSED")?;
    }
    Ok(0)
}

/// `CancelTrade()` — close the trade window. Fires `TRADE_CLOSED`.
fn cancel_trade(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "player-requests")]
    {
        // Declining the TRADE popup cancels the pending request too.
        borrow_state_mut(state)?.player_requests.trade_request = None;
    }
    let had_trade = borrow_state_mut(state)?.active_trade.take().is_some();
    if had_trade {
        push_event(state, "TRADE_CLOSED")?;
    }
    Ok(0)
}

fn close_trade(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.active_trade = None;
    Ok(0)
}

/// `SetTradeCurrency(copper)` — set player money. Silent no-op without
/// a trade open.
fn set_trade_currency(state: &mut LuaState) -> LuaResult<u32> {
    let Some(money) = stack_u64(state, 1) else {
        return Ok(0);
    };
    let mut st = borrow_state_mut(state)?;
    if let Some(trade) = st.active_trade.as_mut() {
        trade.player_money = money;
    }
    Ok(0)
}

fn c_trade_info_add_trade_money(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = borrow_state_mut(state)?;
    let Some(CursorInfo::Money { copper }) = st.cursor_item.clone() else {
        return Ok(0);
    };
    if let Some(trade) = st.active_trade.as_mut() {
        trade.player_money += copper;
    }
    st.cursor_item = None;
    Ok(0)
}

/// `SetCursorItemSlot(slot)` — move the cursor-carried item into the
/// 1-based player trade slot. Silent no-op without a trade, without an
/// item cursor, or for out-of-range slots.
fn set_cursor_item_slot(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_i32(state, 1) else {
        return Ok(0);
    };
    move_cursor_item_to_trade_slot(state, slot)
}

fn get_player_trade_money(state: &mut LuaState) -> LuaResult<u32> {
    get_trade_money(state, TradeMoneyOwner::Player)
}

fn get_target_trade_money(state: &mut LuaState) -> LuaResult<u32> {
    get_trade_money(state, TradeMoneyOwner::Target)
}

enum TradeMoneyOwner {
    Player,
    Target,
}

fn get_trade_money(state: &mut LuaState, owner: TradeMoneyOwner) -> LuaResult<u32> {
    let money = borrow_state(state)?
        .active_trade
        .as_ref()
        .map(|trade| match owner {
            TradeMoneyOwner::Player => trade.player_money,
            TradeMoneyOwner::Target => trade.target_money,
        })
        .unwrap_or(0);
    state.push(Val::Num(money as f64));
    Ok(1)
}

enum TradeItemOwner {
    Player,
    Target,
}

fn get_trade_player_item_info(state: &mut LuaState) -> LuaResult<u32> {
    get_trade_item_info(state, TradeItemOwner::Player)
}

fn get_trade_target_item_info(state: &mut LuaState) -> LuaResult<u32> {
    get_trade_item_info(state, TradeItemOwner::Target)
}

fn get_trade_item_info(state: &mut LuaState, owner: TradeItemOwner) -> LuaResult<u32> {
    let Some(item_id) = trade_slot_item_id(state, &owner) else {
        push_empty_trade_item_info(state, owner);
        return Ok(6);
    };
    push_trade_item_info(state, item_id, owner);
    Ok(6)
}

fn trade_slot_item_id(state: &mut LuaState, owner: &TradeItemOwner) -> Option<u32> {
    let slot = stack_i32(state, 1)?;
    let slot_index = usize::try_from(slot.saturating_sub(1)).ok()?;
    if slot_index >= TRADE_SLOTS {
        return None;
    }
    let trade = borrow_state(state).ok()?.active_trade.clone()?;
    let item_id = match owner {
        TradeItemOwner::Player => trade.player_slots[slot_index],
        TradeItemOwner::Target => trade.target_slots[slot_index],
    };
    (item_id != 0).then_some(item_id)
}

fn push_empty_trade_item_info(state: &mut LuaState, owner: TradeItemOwner) {
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Num(0.0));
    state.push(Val::Num(1.0));
    match owner {
        TradeItemOwner::Player => {
            state.push(Val::Nil);
            state.push(Val::Bool(false));
        }
        TradeItemOwner::Target => {
            state.push(Val::Bool(true));
            state.push(Val::Nil);
        }
    }
}

fn push_trade_item_info(state: &mut LuaState, item_id: u32, owner: TradeItemOwner) {
    let item = items::get_item(item_id);
    let name = item.map(|item| item.name).unwrap_or("Unknown Item");
    let texture = item_icon_file_id(item);
    let quality = item.map(|item| item.quality).unwrap_or(1);

    let name = create_string(state, name);
    state.push(name);
    state.push(Val::Num(texture as f64));
    state.push(Val::Num(1.0));
    state.push(Val::Num(quality as f64));
    match owner {
        TradeItemOwner::Player => {
            state.push(Val::Nil);
            state.push(Val::Bool(false));
        }
        TradeItemOwner::Target => {
            state.push(Val::Bool(true));
            state.push(Val::Nil);
        }
    }
}

fn item_icon_file_id(item: Option<&items::ItemInfo>) -> u32 {
    item.map(|item| item.icon_file_data_id)
        .filter(|texture| *texture != 0)
        .unwrap_or(134400)
}

fn get_trade_player_item_link(state: &mut LuaState) -> LuaResult<u32> {
    get_trade_item_link(state, TradeItemOwner::Player)
}

fn get_trade_target_item_link(state: &mut LuaState) -> LuaResult<u32> {
    get_trade_item_link(state, TradeItemOwner::Target)
}

fn get_trade_item_link(state: &mut LuaState, owner: TradeItemOwner) -> LuaResult<u32> {
    let link =
        trade_slot_item_id(state, &owner).and_then(crate::c_api::item_spell::item_link_for_id);
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn click_trade_button(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_i32(state, 1) else {
        return Ok(0);
    };
    move_cursor_item_to_trade_slot(state, slot)
}

fn move_cursor_item_to_trade_slot(state: &mut LuaState, slot: i32) -> LuaResult<u32> {
    let Some(zero_based) = slot_index(slot) else {
        return Ok(0);
    };
    let mut st = borrow_state_mut(state)?;
    let Some(CursorInfo::Item { item_id, .. }) = st.cursor_item.clone() else {
        return Ok(0);
    };
    if let Some(trade) = st.active_trade.as_mut() {
        trade.player_slots[zero_based] = item_id;
    }
    st.cursor_item = None;
    Ok(0)
}

fn click_target_trade_button(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn slot_index(slot: i32) -> Option<usize> {
    let zero_based = usize::try_from(slot.saturating_sub(1)).ok()?;
    (zero_based < TRADE_SLOTS).then_some(zero_based)
}

fn register_c_trade_info(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_TradeInfo")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "AddTradeMoney",
        c_trade_info_add_trade_money,
    )?;
    table_set_rust_fn_static(state, table_ref, "SetTradeMoney", set_trade_currency)?;
    Ok(())
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "InitiateTrade", initiate_trade)?;
    LuaApiMut::register_function(lua, "AcceptTrade", accept_trade)?;
    LuaApiMut::register_function(lua, "CancelTrade", cancel_trade)?;
    LuaApiMut::register_function(lua, "CloseTrade", close_trade)?;
    LuaApiMut::register_function(lua, "SetTradeCurrency", set_trade_currency)?;
    LuaApiMut::register_function(lua, "SetCursorItemSlot", set_cursor_item_slot)?;
    LuaApiMut::register_function(lua, "GetPlayerTradeMoney", get_player_trade_money)?;
    LuaApiMut::register_function(lua, "GetTargetTradeMoney", get_target_trade_money)?;
    LuaApiMut::register_function(lua, "GetTradePlayerItemInfo", get_trade_player_item_info)?;
    LuaApiMut::register_function(lua, "GetTradeTargetItemInfo", get_trade_target_item_info)?;
    LuaApiMut::register_function(lua, "GetTradePlayerItemLink", get_trade_player_item_link)?;
    LuaApiMut::register_function(lua, "GetTradeTargetItemLink", get_trade_target_item_link)?;
    LuaApiMut::register_function(lua, "ClickTradeButton", click_trade_button)?;
    LuaApiMut::register_function(lua, "ClickTargetTradeButton", click_target_trade_button)?;
    register_c_trade_info(lua.state_mut())?;
    Ok(())
}
