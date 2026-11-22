//! INFERRED (R88em (native evidence pending)): supplied Black Market listings.

use std::collections::HashSet;

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlackMarketItem {
    #[serde(rename = "marketID")]
    market_id: u32,
    #[serde(rename = "itemID")]
    _item_id: u32,
    name: String,
    texture: String,
    quantity: u32,
    item_type: String,
    usable: bool,
    level: u32,
    level_type: String,
    seller_name: String,
    min_bid: u64,
    min_increment: u64,
    curr_bid: u64,
    you_have_high_bid: bool,
    num_bids: u32,
    time_left: u32,
    link: String,
    quality: u32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlackMarketState {
    items: Vec<BlackMarketItem>,
    hot_item: Option<BlackMarketItem>,
    view_only: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_BlackMarket")?;
    table_set_rust_fn_static(state, namespace, "Close", close)?;
    table_set_rust_fn_static(state, namespace, "RequestItems", request_items)?;
    table_set_rust_fn_static(state, namespace, "ItemPlaceBid", item_place_bid)?;
    table_set_rust_fn_static(state, namespace, "IsViewOnly", is_view_only)?;
    table_set_rust_fn_static(state, namespace, "GetNumItems", get_num_items)?;
    table_set_rust_fn_static(state, namespace, "GetHotItem", get_hot_item)?;
    table_set_rust_fn_static(state, namespace, "GetItemInfoByID", get_item_info_by_id)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetItemInfoByIndex",
        get_item_info_by_index,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetBlackMarketState", set_black_market_state)
}

fn set_black_market_state(state: &mut LuaState) -> LuaResult<u32> {
    let json = String::from_stack(state, 1)?;
    let supplied: BlackMarketState = serde_json::from_str(&json).map_err(|_| {
        runtime_error("A_Admin.SetBlackMarketState requires a Black Market JSON object")
    })?;
    validate_state(&supplied)?;
    borrow_state_mut(state)?.black_market = supplied;
    dispatch_event_now(state, "BLACK_MARKET_ITEM_UPDATE", &[])?;
    Ok(0)
}

fn validate_state(supplied: &BlackMarketState) -> LuaResult<()> {
    let mut market_ids = HashSet::new();
    for item in &supplied.items {
        if !market_ids.insert(item.market_id) {
            return Err(runtime_error(
                "A_Admin.SetBlackMarketState requires unique listing marketIDs",
            ));
        }
    }
    if supplied
        .hot_item
        .as_ref()
        .is_some_and(|item| market_ids.contains(&item.market_id))
    {
        return Err(runtime_error(
            "A_Admin.SetBlackMarketState requires a distinct hot-item marketID",
        ));
    }
    Ok(())
}

fn close(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "BLACK_MARKET_CLOSE", &[])?;
    Ok(0)
}

fn request_items(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "BLACK_MARKET_ITEM_UPDATE", &[])?;
    Ok(0)
}

fn item_place_bid(state: &mut LuaState) -> LuaResult<u32> {
    let market_id = u32::from_stack(state, 1)?;
    let bid = u64::try_from(i64::from_stack(state, 2)?)
        .map_err(|_| runtime_error("C_BlackMarket.ItemPlaceBid requires a nonnegative bid"))?;
    let mut sim = borrow_state_mut(state)?;
    let item = sim
        .black_market
        .items
        .iter_mut()
        .chain(sim.black_market.hot_item.iter_mut())
        .find(|item| item.market_id == market_id);
    if let Some(item) = item {
        item.curr_bid = bid;
        item.you_have_high_bid = true;
        item.num_bids += 1;
    }
    Ok(0)
}

fn is_view_only(state: &mut LuaState) -> LuaResult<u32> {
    let view_only = borrow_state(state)?.black_market.view_only;
    state.push(Val::Bool(view_only));
    Ok(1)
}

fn get_num_items(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.black_market.items.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_hot_item(state: &mut LuaState) -> LuaResult<u32> {
    let item = borrow_state(state)?.black_market.hot_item.clone();
    let Some(item) = item else { return Ok(0) };
    push_item_info(state, &item);
    Ok(17)
}

fn get_item_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let market_id = u32::from_stack(state, 1)?;
    let item = {
        let sim = borrow_state(state)?;
        sim.black_market
            .items
            .iter()
            .chain(sim.black_market.hot_item.iter())
            .find(|item| item.market_id == market_id)
            .cloned()
    };
    push_optional_item_info(state, item)
}

fn get_item_info_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = u32::from_stack(state, 1)? as usize;
    let item = borrow_state(state)?
        .black_market
        .items
        .get(index.saturating_sub(1))
        .cloned();
    push_optional_item_info(state, item)
}

fn push_optional_item_info(state: &mut LuaState, item: Option<BlackMarketItem>) -> LuaResult<u32> {
    let Some(item) = item else { return Ok(0) };
    push_item_info(state, &item);
    Ok(17)
}

fn push_item_info(state: &mut LuaState, item: &BlackMarketItem) {
    push_string(state, &item.name);
    push_string(state, &item.texture);
    state.push(Val::Num(item.quantity as f64));
    push_string(state, &item.item_type);
    state.push(Val::Bool(item.usable));
    state.push(Val::Num(item.level as f64));
    push_string(state, &item.level_type);
    push_string(state, &item.seller_name);
    state.push(Val::Num(item.min_bid as f64));
    state.push(Val::Num(item.min_increment as f64));
    state.push(Val::Num(item.curr_bid as f64));
    state.push(Val::Bool(item.you_have_high_bid));
    state.push(Val::Num(item.num_bids as f64));
    state.push(Val::Num(item.time_left as f64));
    push_string(state, &item.link);
    state.push(Val::Num(item.market_id as f64));
    state.push(Val::Num(item.quality as f64));
}

fn push_string(state: &mut LuaState, text: &str) {
    let value = create_string(state, text);
    state.push(value);
}
