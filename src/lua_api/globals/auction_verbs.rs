//! Legacy Auction House globals used by the Mists `Blizzard_AuctionUI`.
//!
//! The simulator's richer auction model lives behind `C_AuctionHouse`.
//! Mists can still load the classic auction panel, which reads the older
//! global API. These functions expose that same backing state in the legacy
//! row/verb shapes instead of creating a parallel stub-only panel. Wrath shares
//! the row/verb adapters, but its subclass catalog uses the authenticated DBC
//! producer in `real::wrath_auction_catalog`, not this Mists numeric adapter.

use crate::items;
use crate::lua_api::globals::missing_surface::item_link_for_id;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::state::{
    AuctionBrowseResult, AuctionRowInfo, BagItem, BidAuction, OwnedAuction,
};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const AUCTION_ITEM_LIST_UPDATE: &str = "AUCTION_ITEM_LIST_UPDATE";
const AUCTION_BIDDER_LIST_UPDATE: &str = "AUCTION_BIDDER_LIST_UPDATE";
const AUCTION_OWNED_LIST_UPDATE: &str = "AUCTION_OWNED_LIST_UPDATE";
const NEW_AUCTION_UPDATE: &str = "NEW_AUCTION_UPDATE";
const BID_ADDED: &str = "BID_ADDED";

const AUCTION_STATUS_ACTIVE: i32 = 0;
const AUCTION_STATUS_SOLD: i32 = 1;
const TIME_LEFT_BAND_VERY_LONG: i32 = 4;
const SECONDS_48_HOURS: i64 = 48 * 60 * 60;

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_i64(state: &mut LuaState, index: i32) -> Option<i64> {
    stack_i32(state, index).map(i64::from)
}

fn stack_string(state: &mut LuaState, index: i32) -> String {
    Option::<String>::from_stack(state, index)
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn is_using_legacy_auction_client(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

/// The modeled fixture has no active auctioneer; reconstructed 12340 close is inert.
/// Active-session cleanup and reentrant event ordering remain unmodeled.
fn close_inactive_auction_house(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn can_send_auction_query(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

#[cfg(feature = "client-mists")]
fn get_auction_item_sub_classes(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = stack_i32(state, 1).unwrap_or(-1);
    let subclass_count = standard_subclass_count(class_id);
    for sub_class_id in 0..subclass_count {
        state.push(Val::Num(sub_class_id as f64));
    }
    Ok(subclass_count as u32)
}

#[cfg(feature = "client-mists")]
fn standard_subclass_count(class_id: i32) -> i32 {
    match class_id {
        0 => 12,
        1 => 8,
        2 => 21,
        3 => 11,
        4 => 12,
        5 => 5,
        6 => 6,
        7 => 21,
        8 => 4,
        9 => 13,
        11 => 6,
        12 => 1,
        15 => 6,
        16 => 12,
        17 => 3,
        19 => 1,
        _ => 0,
    }
}

fn sort_auction_clear_sort(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn sort_auction_set_sort(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_auction_sort(state: &mut LuaState) -> LuaResult<u32> {
    let sort_table = stack_string(state, 1);
    let column = match sort_table.as_str() {
        "owner" => "status",
        "bidder" => "quality",
        _ => "name",
    };
    let column = create_string(state, column);
    state.push(column);
    state.push(Val::Bool(false));
    Ok(2)
}

fn set_auctions_tab_showing(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn query_auction_items(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, AUCTION_ITEM_LIST_UPDATE, &[])?;
    Ok(0)
}

fn get_bidder_auction_items(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, AUCTION_BIDDER_LIST_UPDATE, &[])?;
    Ok(0)
}

fn get_owner_auction_items(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, AUCTION_OWNED_LIST_UPDATE, &[])?;
    Ok(0)
}

fn get_num_auction_items(state: &mut LuaState) -> LuaResult<u32> {
    let list_type = stack_string(state, 1);
    let count = match list_type.as_str() {
        "list" => {
            let sim = borrow_state(state)?;
            if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath
                || !sim.auction_browse_results.is_empty()
            {
                sim.auction_browse_results.len()
            } else {
                sim.auction_browse_items.len()
            }
        }
        "owner" => borrow_state(state)?.auction_owned.len(),
        "bidder" => borrow_state(state)?.auction_bids.len(),
        _ => 0,
    } as f64;
    state.push(Val::Num(count));
    state.push(Val::Num(count));
    Ok(2)
}

fn get_auction_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let list_type = stack_string(state, 1);
    let Some(index) = stack_i32(state, 2) else {
        return Ok(0);
    };
    let Some(row) = legacy_auction_row(state, &list_type, index)? else {
        return Ok(0);
    };
    push_legacy_auction_row(state, &row)
}

fn legacy_auction_row(
    state: &mut LuaState,
    list_type: &str,
    index: i32,
) -> LuaResult<Option<LegacyAuctionRow>> {
    let Some(row_index) = auction_row_index(index) else {
        return Ok(None);
    };
    let sim = borrow_state(state)?;
    let row = match list_type {
        "list" => sim
            .auction_browse_results
            .get(row_index)
            .map(|entry| browse_auction_row(entry, index)),
        "owner" => sim.auction_owned.get(row_index).map(owned_auction_row),
        "bidder" => sim.auction_bids.get(row_index).map(bid_auction_row),
        _ => None,
    };
    Ok(row)
}

fn auction_row_index(index: i32) -> Option<usize> {
    (index >= 1).then_some((index - 1) as usize)
}

struct LegacyAuctionRow {
    auction_id: i64,
    item_id: i32,
    item_level: i32,
    quantity: i32,
    min_bid: i64,
    min_increment: i64,
    buyout_amount: i64,
    bid_amount: i64,
    high_bidder: bool,
    owner: Option<String>,
    sale_status: i32,
    time_left: i32,
}

fn browse_auction_row(entry: &AuctionBrowseResult, fallback_index: i32) -> LegacyAuctionRow {
    LegacyAuctionRow {
        auction_id: entry.auction_id.unwrap_or(fallback_index as i64),
        item_id: entry.item_id,
        item_level: entry.item_level,
        quantity: entry.total_quantity,
        min_bid: entry.min_price,
        min_increment: 0,
        buyout_amount: entry.min_price,
        bid_amount: 0,
        high_bidder: false,
        owner: Some("Browse Seller".to_string()),
        sale_status: AUCTION_STATUS_ACTIVE,
        time_left: TIME_LEFT_BAND_VERY_LONG,
    }
}

fn owned_auction_row(entry: &OwnedAuction) -> LegacyAuctionRow {
    LegacyAuctionRow {
        auction_id: entry.auction_id as i64,
        item_id: entry.item_id,
        item_level: entry.item_level,
        quantity: entry.quantity,
        min_bid: entry.bid_amount,
        min_increment: 0,
        buyout_amount: entry.buyout_amount,
        bid_amount: entry.bid_amount,
        high_bidder: false,
        owner: None,
        sale_status: entry.status,
        time_left: entry.time_left,
    }
}

fn bid_auction_row(entry: &BidAuction) -> LegacyAuctionRow {
    LegacyAuctionRow {
        auction_id: entry.auction_id as i64,
        item_id: entry.item_id,
        item_level: entry.item_level,
        quantity: entry.quantity,
        min_bid: entry.bid_amount,
        min_increment: 0,
        buyout_amount: entry.buyout_amount,
        bid_amount: entry.bid_amount,
        high_bidder: entry.bidder.is_some(),
        owner: Some("Browse Seller".to_string()),
        sale_status: AUCTION_STATUS_ACTIVE,
        time_left: entry.time_left,
    }
}

fn push_legacy_auction_row(state: &mut LuaState, row: &LegacyAuctionRow) -> LuaResult<u32> {
    push_auction_item_fields(state, row);
    let owner = row.owner.as_deref().unwrap_or("Player");
    let owner = create_string(state, owner);
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        push_wrath_auction_prices(state, row, owner)
    } else {
        push_mists_auction_prices(state, row, owner)
    }
}

fn push_auction_item_fields(state: &mut LuaState, row: &LegacyAuctionRow) {
    let item = items::get_item(row.item_id as u32);
    let name = item
        .map(|item| item.name.to_string())
        .unwrap_or_else(|| format!("Item {}", row.item_id));
    let icon = item.map(item_icon_file_id).unwrap_or(134400.0);
    let quality = item.map(|item| item.quality as f64).unwrap_or(0.0);
    let name = create_string(state, &name);
    state.push(name);
    state.push(Val::Num(icon));
    state.push(Val::Num(row.quantity as f64));
    state.push(Val::Num(quality));
    let usable = if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        Val::Num(1.0)
    } else {
        Val::Bool(true)
    };
    state.push(usable);
    state.push(Val::Num(row.item_level as f64));
}

fn push_mists_auction_prices(
    state: &mut LuaState,
    row: &LegacyAuctionRow,
    owner: Val,
) -> LuaResult<u32> {
    let level_header = create_string(state, "Item Level");
    state.push(level_header);
    state.push(Val::Num(row.min_bid as f64));
    state.push(Val::Num(row.min_increment as f64));
    state.push(Val::Num(row.buyout_amount as f64));
    state.push(Val::Num(row.bid_amount as f64));
    state.push(Val::Bool(row.high_bidder));
    state.push(Val::Nil);
    state.push(owner);
    state.push(owner);
    state.push(Val::Num(row.sale_status as f64));
    state.push(Val::Num(row.item_id as f64));
    state.push(Val::Bool(true));
    Ok(18)
}

/// Reconstructed 12340 rows omit Mists' level header and full-name columns.
fn push_wrath_auction_prices(
    state: &mut LuaState,
    row: &LegacyAuctionRow,
    owner: Val,
) -> LuaResult<u32> {
    state.push(Val::Num(row.min_bid as f64));
    state.push(Val::Num(row.min_increment as f64));
    state.push(Val::Num(row.buyout_amount as f64));
    state.push(Val::Num(row.bid_amount as f64));
    let high_bidder = if row.high_bidder {
        Val::Num(1.0)
    } else {
        Val::Nil
    };
    state.push(high_bidder);
    state.push(owner);
    state.push(Val::Num(row.sale_status as f64));
    Ok(13)
}

fn item_icon_file_id(item: &items::ItemInfo) -> f64 {
    match item.icon_file_data_id {
        0 => 134400.0,
        icon => icon as f64,
    }
}

fn get_auction_item_time_left(state: &mut LuaState) -> LuaResult<u32> {
    let list_type = stack_string(state, 1);
    let index = stack_i32(state, 2).unwrap_or(0);
    let time_left = legacy_auction_row(state, &list_type, index)?
        .map(|row| row.time_left)
        .unwrap_or(0);
    state.push(Val::Num(time_left as f64));
    Ok(1)
}

fn get_auction_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let list_type = stack_string(state, 1);
    let index = stack_i32(state, 2).unwrap_or(0);
    let link = legacy_auction_row(state, &list_type, index)?
        .and_then(|row| item_link_for_id(row.item_id as u32));
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_auction_item_battle_pet_info(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn place_auction_bid(state: &mut LuaState) -> LuaResult<u32> {
    let list_type = stack_string(state, 1);
    let Some(index) = stack_i32(state, 2) else {
        return Ok(0);
    };
    let Some(bid_amount) = stack_i64(state, 3) else {
        return Ok(0);
    };
    let Some(row) = legacy_auction_row(state, &list_type, index)? else {
        return Ok(0);
    };
    let bidder_name = borrow_state(state)?.player.name.clone();
    {
        let mut sim = borrow_state_mut(state)?;
        sim.player.money -= bid_amount;
        sim.auction_bids.push(BidAuction {
            auction_id: row.auction_id as i32,
            item_id: row.item_id,
            item_level: row.item_level,
            quantity: row.quantity,
            bid_amount,
            buyout_amount: row.buyout_amount,
            time_left: row.time_left,
            time_left_seconds: SECONDS_48_HOURS,
            bidder: Some(bidder_name),
        });
    }
    dispatch_event_now(state, BID_ADDED, &[Val::Num(row.auction_id as f64)])?;
    dispatch_event_now(state, AUCTION_BIDDER_LIST_UPDATE, &[])?;
    Ok(0)
}

fn click_auction_sell_item_button(state: &mut LuaState) -> LuaResult<u32> {
    let cursor_item = {
        let sim = borrow_state(state)?;
        match sim.cursor_item.clone() {
            Some(crate::lua_api::state::CursorInfo::Item {
                item_id,
                stack_count,
                ..
            }) => Some(BagItem {
                item_id,
                stack_count,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
            }),
            _ => None,
        }
    };
    let mut sim = borrow_state_mut(state)?;
    sim.auction_sell_item = cursor_item;
    sim.cursor_item = None;
    drop(sim);
    dispatch_event_now(state, NEW_AUCTION_UPDATE, &[])?;
    Ok(0)
}

fn get_auction_sell_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let sell_item = borrow_state(state)?.auction_sell_item.clone();
    let Some(sell_item) = sell_item else {
        return Ok(0);
    };
    let item = items::get_item(sell_item.item_id);
    let name = item
        .map(|item| item.name.to_string())
        .unwrap_or_else(|| format!("Item {}", sell_item.item_id));
    let icon = item.map(item_icon_file_id).unwrap_or(134400.0);
    let quality = item.map(|item| item.quality as f64).unwrap_or(0.0);
    let sell_price = item.map(|item| item.sell_price as f64).unwrap_or(0.0);

    let name = create_string(state, &name);
    state.push(name);
    state.push(Val::Num(icon));
    state.push(Val::Num(sell_item.stack_count as f64));
    state.push(Val::Num(quality));
    state.push(Val::Bool(true));
    state.push(Val::Num(sell_price));
    state.push(Val::Num(sell_price));
    state.push(Val::Num(sell_item.stack_count as f64));
    state.push(Val::Num(sell_item.stack_count as f64));
    state.push(Val::Num(sell_item.item_id as f64));
    Ok(10)
}

fn start_auction(state: &mut LuaState) -> LuaResult<u32> {
    let request = StartAuctionRequest::from_stack(state);
    let Some(sell_item) = borrow_state(state)?.auction_sell_item.clone() else {
        return Ok(0);
    };

    let auction_id = next_owned_auction_id(state)?;
    let row = build_started_auction_row(auction_id, &sell_item, &request);
    append_started_auction(state, row, &request)?;
    dispatch_event_now(state, AUCTION_OWNED_LIST_UPDATE, &[])?;
    Ok(0)
}

struct StartAuctionRequest {
    bid_amount: i64,
    buyout_amount: i64,
    duration: i32,
    stack_size: i32,
    num_stacks: i32,
}

impl StartAuctionRequest {
    fn from_stack(state: &mut LuaState) -> Self {
        Self {
            bid_amount: stack_i64(state, 1).unwrap_or(0),
            buyout_amount: stack_i64(state, 2).unwrap_or(0),
            duration: stack_i32(state, 3).unwrap_or(3),
            stack_size: stack_i32(state, 4).unwrap_or(1).max(1),
            num_stacks: stack_i32(state, 5).unwrap_or(1).max(1),
        }
    }
}

fn build_started_auction_row(
    auction_id: i32,
    sell_item: &BagItem,
    request: &StartAuctionRequest,
) -> OwnedAuction {
    let quantity = request.stack_size.min(sell_item.stack_count).max(1);
    OwnedAuction {
        auction_id,
        item_id: sell_item.item_id as i32,
        item_level: item_level(sell_item.item_id),
        quantity,
        bid_amount: request.bid_amount,
        buyout_amount: request.buyout_amount,
        status: AUCTION_STATUS_ACTIVE,
        time_left: TIME_LEFT_BAND_VERY_LONG,
        time_left_seconds: time_left_seconds_for_duration(request.duration),
    }
}

fn append_started_auction(
    state: &mut LuaState,
    row: OwnedAuction,
    request: &StartAuctionRequest,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let owner = sim.player.name.clone();
    let auction_id = row.auction_id;
    sim.auction_owned.push(row);
    sim.auction_index.insert(
        i64::from(auction_id),
        AuctionRowInfo {
            owner,
            bid_amount: request.bid_amount,
            buyout_amount: request.buyout_amount,
            deposit: 0,
            consortium_cut: 0,
        },
    );
    if request.num_stacks <= 1 {
        sim.auction_sell_item = None;
    }
    Ok(())
}

fn item_level(item_id: u32) -> i32 {
    items::get_item(item_id)
        .map(|item| item.item_level as i32)
        .unwrap_or(0)
}

fn next_owned_auction_id(state: &mut LuaState) -> LuaResult<i32> {
    let id = borrow_state(state)?
        .auction_owned
        .iter()
        .map(|row| row.auction_id)
        .max()
        .unwrap_or(0)
        + 1;
    Ok(id)
}

fn time_left_seconds_for_duration(duration: i32) -> i64 {
    match duration {
        1 => 12 * 60 * 60,
        2 => 24 * 60 * 60,
        _ => SECONDS_48_HOURS,
    }
}

fn can_cancel_auction(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    let can_cancel = owned_auction_by_index(state, index)?
        .map(|row| row.status == AUCTION_STATUS_ACTIVE)
        .unwrap_or(false);
    state.push(Val::Bool(can_cancel));
    Ok(1)
}

fn cancel_auction(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    if index < 1 {
        return Ok(0);
    }
    let canceled = {
        let mut sim = borrow_state_mut(state)?;
        if let Some(row) = sim.auction_owned.get_mut((index - 1) as usize) {
            row.status = AUCTION_STATUS_SOLD;
            true
        } else {
            false
        }
    };
    if canceled {
        dispatch_event_now(state, AUCTION_OWNED_LIST_UPDATE, &[])?;
    }
    Ok(0)
}

fn owned_auction_by_index(state: &mut LuaState, index: i32) -> LuaResult<Option<OwnedAuction>> {
    if index < 1 {
        return Ok(None);
    }
    Ok(borrow_state(state)?
        .auction_owned
        .get((index - 1) as usize)
        .cloned())
}

fn get_auction_deposit(state: &mut LuaState) -> LuaResult<u32> {
    let duration = stack_i64(state, 1).unwrap_or(1).clamp(1, 3);
    let start_price = stack_i64(state, 2).unwrap_or(0).max(0);
    let buyout_price = stack_i64(state, 3).unwrap_or(0).max(0);
    let stack_size = stack_i64(state, 4).unwrap_or(1).max(1);
    let num_stacks = stack_i64(state, 5).unwrap_or(1).max(1);
    let price_basis = buyout_price.max(start_price);
    let deposit = (price_basis * stack_size * num_stacks * duration * 15) / 1000;
    state.push(Val::Num(deposit as f64));
    Ok(1)
}

fn calculate_auction_deposit(state: &mut LuaState) -> LuaResult<u32> {
    get_auction_deposit(state)
}

fn get_selected_auction_item(state: &mut LuaState) -> LuaResult<u32> {
    let list_type = stack_string(state, 1);
    let selected = {
        let sim = borrow_state(state)?;
        match list_type.as_str() {
            "list" => sim.selected_auction_list,
            "owner" => sim.selected_auction_owner,
            "bidder" => sim.selected_auction_bidder,
            _ => 0,
        }
    };
    state.push(Val::Num(selected as f64));
    Ok(1)
}

fn set_selected_auction_item(state: &mut LuaState) -> LuaResult<u32> {
    let list_type = stack_string(state, 1);
    let selected = stack_i32(state, 2).unwrap_or(0);
    let mut sim = borrow_state_mut(state)?;
    match list_type.as_str() {
        "list" => sim.selected_auction_list = selected,
        "owner" => sim.selected_auction_owner = selected,
        "bidder" => sim.selected_auction_bidder = selected,
        _ => {}
    }
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        LuaApiMut::register_function(lua, "CloseAuctionHouse", close_inactive_auction_house)?;
    }
    register_auction_query_globals(lua)?;
    register_auction_item_globals(lua)?;
    register_auction_sell_globals(lua)?;
    register_auction_selection_globals(lua)
}

fn register_auction_query_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(
        lua,
        "IsUsingLegacyAuctionClient",
        is_using_legacy_auction_client,
    )?;
    LuaApiMut::register_function(lua, "CanSendAuctionQuery", can_send_auction_query)?;
    #[cfg(feature = "client-mists")]
    LuaApiMut::register_function(
        lua,
        "GetAuctionItemSubClasses",
        get_auction_item_sub_classes,
    )?;
    LuaApiMut::register_function(lua, "SortAuctionClearSort", sort_auction_clear_sort)?;
    LuaApiMut::register_function(lua, "SortAuctionSetSort", sort_auction_set_sort)?;
    LuaApiMut::register_function(lua, "GetAuctionSort", get_auction_sort)?;
    LuaApiMut::register_function(lua, "SetAuctionsTabShowing", set_auctions_tab_showing)?;
    LuaApiMut::register_function(lua, "QueryAuctionItems", query_auction_items)?;
    LuaApiMut::register_function(lua, "GetBidderAuctionItems", get_bidder_auction_items)?;
    LuaApiMut::register_function(lua, "GetOwnerAuctionItems", get_owner_auction_items)?;
    LuaApiMut::register_function(lua, "GetNumAuctionItems", get_num_auction_items)?;
    Ok(())
}

fn register_auction_item_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetAuctionItemInfo", get_auction_item_info)?;
    LuaApiMut::register_function(lua, "GetAuctionItemTimeLeft", get_auction_item_time_left)?;
    LuaApiMut::register_function(lua, "GetAuctionItemLink", get_auction_item_link)?;
    LuaApiMut::register_function(
        lua,
        "GetAuctionItemBattlePetInfo",
        get_auction_item_battle_pet_info,
    )?;
    LuaApiMut::register_function(lua, "PlaceAuctionBid", place_auction_bid)?;
    Ok(())
}

fn register_auction_sell_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(
        lua,
        "ClickAuctionSellItemButton",
        click_auction_sell_item_button,
    )?;
    LuaApiMut::register_function(lua, "GetAuctionSellItemInfo", get_auction_sell_item_info)?;
    LuaApiMut::register_function(lua, "StartAuction", start_auction)?;
    LuaApiMut::register_function(lua, "CanCancelAuction", can_cancel_auction)?;
    LuaApiMut::register_function(lua, "CancelAuction", cancel_auction)?;
    LuaApiMut::register_function(lua, "GetAuctionDeposit", get_auction_deposit)?;
    LuaApiMut::register_function(lua, "CalculateAuctionDeposit", calculate_auction_deposit)?;
    Ok(())
}

fn register_auction_selection_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetSelectedAuctionItem", get_selected_auction_item)?;
    LuaApiMut::register_function(lua, "SetSelectedAuctionItem", set_selected_auction_item)?;
    Ok(())
}
