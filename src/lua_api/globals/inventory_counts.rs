//! Inventory / merchant / loot / auction count globals.
//!
//! Migrates 4 entries off `GLOBAL_ZERO_STUBS`:
//!
//! - `GetContainerNumFreeSlots(bagID)` → `(numFree, bagType)` from
//!   `SimState.bag_info` and `SimState.bag_items`, matching C_Container.
//! - `GetNumLootItems()`            → `SimState.loot_slots.len()`
//! - `GetMerchantNumItems()`        → `SimState.merchant_items.len()`
//! - `GetNumAuctionItems(listType)` → `(numItems, totalItems)` from
//!   `SimState.auction_browse_items` for `"list"`; `"owner"` / `"bidder"`
//!   always report 0 in the sim.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

/// Retail `GetContainerNumFreeSlots(bagID)` returns `(numFreeSlots, bagType)`.
fn get_container_num_free_slots(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let free = sim.bag_free_slots(bag);
    let family = sim.bag_info.get(&bag).map_or(0, |info| info.family);
    drop(sim);
    state.push(Val::Num(free as f64));
    state.push(Val::Num(family as f64));
    Ok(2)
}

fn get_num_loot_items(state: &mut LuaState) -> LuaResult<u32> {
    let n = borrow_state(state)?.loot_slots.len() as f64;
    state.push(Val::Num(n));
    Ok(1)
}

fn get_merchant_num_items(state: &mut LuaState) -> LuaResult<u32> {
    let n = borrow_state(state)?.merchant_items.len() as f64;
    state.push(Val::Num(n));
    Ok(1)
}

/// Retail `GetNumAuctionItems(listType)` returns `(numItems, totalItems)`.
/// We only model the `"list"` bucket (browse results). `"owner"` (the
/// player's active auctions) and `"bidder"` always report 0 because
/// neither queue is simulated.
fn get_num_auction_items(state: &mut LuaState) -> LuaResult<u32> {
    let list_type = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let n = if list_type == "list" {
        borrow_state(state)?.auction_browse_items.len() as f64
    } else {
        0.0
    };
    state.push(Val::Num(n));
    state.push(Val::Num(n));
    Ok(2)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(
        lua,
        "GetContainerNumFreeSlots",
        get_container_num_free_slots,
    )?;
    LuaApiMut::register_function(lua, "GetNumLootItems", get_num_loot_items)?;
    LuaApiMut::register_function(lua, "GetMerchantNumItems", get_merchant_num_items)?;
    LuaApiMut::register_function(lua, "GetNumAuctionItems", get_num_auction_items)?;
    Ok(())
}
