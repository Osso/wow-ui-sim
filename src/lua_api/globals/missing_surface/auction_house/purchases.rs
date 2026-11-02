use super::{
    SECONDS_48_HOURS, TIME_LEFT_BAND_VERY_LONG, deduct_player_money, read_optional_money_arg,
};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::state::{
    AuctionRowInfo, BidAuction, CommodityPurchaseQuote, CommoditySearchResultInfo,
};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const BID_ADDED: &str = "BID_ADDED";
const BIDS_UPDATED: &str = "BIDS_UPDATED";
const OWNED_AUCTION_BIDDER_INFO_RECEIVED: &str = "OWNED_AUCTION_BIDDER_INFO_RECEIVED";
const COMMODITY_PRICE_UPDATED: &str = "COMMODITY_PRICE_UPDATED";
const COMMODITY_QUOTE_UPDATED: &str = "COMMODITY_QUOTE_UPDATED";
const COMMODITY_PURCHASE_SUCCEEDED: &str = "COMMODITY_PURCHASE_SUCCEEDED";
const COMMODITY_PURCHASED: &str = "COMMODITY_PURCHASED";

/// Fresh quote-window seeded by `StartCommoditiesPurchase`. The retail
/// server holds a 15s lock; the sim mirrors that even though tests
/// usually confirm immediately.
const COMMODITY_QUOTE_WINDOW_SECONDS: i32 = 15;

/// Bag slot used when adding a purchased commodity to the player's
/// inventory. The sim has no real free-slot picker; Auctions UI tests
/// only care that *some* slot got the item, so the deterministic
/// (4, 0) slot keeps assertions simple.
const PURCHASE_DEPOSIT_BAG: i32 = 4;
const PURCHASE_DEPOSIT_SLOT: i32 = 0;

fn dispatch_bid_added(state: &mut LuaState, auction_id: i64) -> LuaResult<()> {
    dispatch_event_now(state, BID_ADDED, &[Val::Num(auction_id as f64)])
}

fn dispatch_bids_updated(state: &mut LuaState) -> LuaResult<()> {
    dispatch_event_now(state, BIDS_UPDATED, &[])
}

fn dispatch_owned_auction_bidder_info_received(
    state: &mut LuaState,
    auction_id: i32,
) -> LuaResult<()> {
    dispatch_event_now(
        state,
        OWNED_AUCTION_BIDDER_INFO_RECEIVED,
        &[Val::Num(auction_id as f64)],
    )
}

fn dispatch_commodity_price_updated(state: &mut LuaState, item_id: i32) -> LuaResult<()> {
    dispatch_event_now(state, COMMODITY_PRICE_UPDATED, &[Val::Num(item_id as f64)])
}

fn dispatch_commodity_quote_updated(state: &mut LuaState, item_id: i32) -> LuaResult<()> {
    dispatch_event_now(state, COMMODITY_QUOTE_UPDATED, &[Val::Num(item_id as f64)])
}

fn dispatch_commodity_purchase_succeeded(state: &mut LuaState, item_id: i32) -> LuaResult<()> {
    dispatch_event_now(
        state,
        COMMODITY_PURCHASE_SUCCEEDED,
        &[Val::Num(item_id as f64)],
    )
}

fn dispatch_commodity_purchased(state: &mut LuaState, item_id: i32) -> LuaResult<()> {
    dispatch_event_now(state, COMMODITY_PURCHASED, &[Val::Num(item_id as f64)])
}

/// Cheapest-first walk of the listings stack: returns the per-row
/// `(unit_price, drained)` pairs the buyer pays for, and the leftover
/// requested quantity if the stack runs dry. Pure function — does not
/// mutate listings, so callers can use the same plan for both the
/// `Start` (quote) and `Confirm` (drain) paths.
fn plan_commodity_purchase(
    listings: &[CommoditySearchResultInfo],
    requested: i32,
) -> (Vec<(i64, i32)>, i32) {
    let mut plan: Vec<(i64, i32)> = Vec::new();
    let mut remaining = requested.max(0);
    let mut sorted_listings: Vec<&CommoditySearchResultInfo> = listings.iter().collect();
    sorted_listings.sort_by_key(|row| row.unit_price);
    for row in sorted_listings {
        if remaining == 0 {
            break;
        }
        let take = row.quantity.min(remaining);
        if take > 0 {
            plan.push((row.unit_price, take));
            remaining -= take;
        }
    }
    (plan, remaining)
}

fn total_price_for_plan(plan: &[(i64, i32)]) -> i64 {
    plan.iter().map(|(price, qty)| price * (*qty as i64)).sum()
}

/// Drain the planned quantity from the cheapest-first stack. Rows that
/// are fully consumed are removed; partially consumed rows shrink in
/// place. Mirrors the retail server's commodity book draining the
/// lowest-priced listing first.
fn drain_commodity_listings(listings: &mut Vec<CommoditySearchResultInfo>, requested: i32) {
    let mut remaining = requested.max(0);
    listings.sort_by_key(|row| row.unit_price);
    listings.retain_mut(|row| {
        if remaining == 0 {
            return true;
        }
        let take = row.quantity.min(remaining);
        row.quantity -= take;
        remaining -= take;
        row.quantity > 0
    });
}

fn deposit_purchased_into_bag(state: &mut LuaState, item_id: i32, quantity: i32) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let slot = sim
        .bag_items
        .entry((PURCHASE_DEPOSIT_BAG, PURCHASE_DEPOSIT_SLOT))
        .or_insert_with(|| crate::lua_api::state::BagItem {
            item_id: item_id as u32,
            stack_count: 0,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
            owned_identity: None,
        });
    if slot.item_id != item_id as u32 {
        slot.item_id = item_id as u32;
        slot.stack_count = 0;
    }
    slot.stack_count += quantity;
    Ok(())
}

fn read_bid_target_buyout(state: &mut LuaState, auction_id: i64) -> LuaResult<Option<i64>> {
    let sim = borrow_state(state)?;
    if let Some(info) = sim.auction_index.get(&auction_id) {
        return Ok(Some(info.buyout_amount));
    }
    let from_owned = sim
        .auction_owned
        .iter()
        .find(|row| row.auction_id as i64 == auction_id)
        .map(|row| row.buyout_amount);
    if let Some(buyout) = from_owned {
        return Ok(Some(buyout));
    }
    let from_searches = sim.auction_item_searches.values().find_map(|bucket| {
        bucket
            .entries
            .iter()
            .find(|entry| entry.auction_id == auction_id)
            .map(|entry| entry.buyout_amount)
    });
    Ok(from_searches)
}

fn append_bid_record(
    state: &mut LuaState,
    auction_id: i64,
    bid_amount: i64,
    buyout_amount: i64,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let bidder_name = sim.player.name.clone();
    sim.auction_bids.push(BidAuction {
        auction_id: auction_id as i32,
        item_id: 0,
        item_level: 0,
        quantity: 1,
        bid_amount,
        buyout_amount,
        time_left: TIME_LEFT_BAND_VERY_LONG,
        time_left_seconds: SECONDS_48_HOURS,
        bidder: Some(bidder_name),
    });
    Ok(())
}

pub(super) fn c_auction_house_place_bid(state: &mut LuaState) -> LuaResult<u32> {
    let auction_id = i64::from_stack(state, 1)?;
    let bid_amount = read_optional_money_arg(state, 2)?;
    let Some(buyout_amount) = read_bid_target_buyout(state, auction_id)? else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    append_bid_record(state, auction_id, bid_amount, buyout_amount)?;
    deduct_player_money(state, bid_amount)?;
    dispatch_bid_added(state, auction_id)?;
    dispatch_bids_updated(state)?;
    state.push(Val::Bool(true));
    Ok(1)
}

pub(super) fn c_auction_house_get_auction_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let auction_id = i64::from_stack(state, 1)?;
    let info = borrow_state(state)?.auction_index.get(&auction_id).cloned();
    let Some(info) = info else { return Ok(0) };
    push_auction_row_info(state, &info);
    Ok(5)
}

fn push_auction_row_info(state: &mut LuaState, info: &AuctionRowInfo) {
    let owner = create_string(state, &info.owner);
    state.push(owner);
    state.push(Val::Num(info.bid_amount as f64));
    state.push(Val::Num(info.buyout_amount as f64));
    state.push(Val::Num(info.deposit as f64));
    state.push(Val::Num(info.consortium_cut as f64));
}

pub(super) fn c_auction_house_request_owned_auction_bidder_info(
    state: &mut LuaState,
) -> LuaResult<u32> {
    let auction_id = i32::from_stack(state, 1)?;
    dispatch_owned_auction_bidder_info_received(state, auction_id)?;
    state.push(Val::Bool(true));
    Ok(1)
}

pub(super) fn c_auction_house_start_commodities_purchase(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    let quantity = i32::from_stack(state, 2)?;
    let listings = borrow_state(state)?
        .auction_commodity_searches
        .get(&item_id)
        .map(|bucket| bucket.entries.clone())
        .unwrap_or_default();
    let (plan, _) = plan_commodity_purchase(&listings, quantity);
    let total_price = total_price_for_plan(&plan);
    borrow_state_mut(state)?.commodity_purchase_quote = Some(CommodityPurchaseQuote {
        item_id,
        quantity,
        total_price,
        quote_duration_remaining: COMMODITY_QUOTE_WINDOW_SECONDS,
    });
    dispatch_commodity_price_updated(state, item_id)?;
    dispatch_commodity_quote_updated(state, item_id)?;
    Ok(0)
}

pub(super) fn c_auction_house_confirm_commodities_purchase(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    let quantity = i32::from_stack(state, 2)?;
    let total_price = drain_and_total_commodity_purchase(state, item_id, quantity)?;
    deduct_player_money(state, total_price)?;
    deposit_purchased_into_bag(state, item_id, quantity)?;
    borrow_state_mut(state)?.commodity_purchase_quote = None;
    dispatch_commodity_purchase_succeeded(state, item_id)?;
    dispatch_commodity_purchased(state, item_id)?;
    Ok(0)
}

/// Build the per-row spend plan, drain the listings stack, and return
/// the total copper the buyer owes. Wraps the borrow scopes so the
/// caller's deduct/event sequence stays linear.
fn drain_and_total_commodity_purchase(
    state: &mut LuaState,
    item_id: i32,
    quantity: i32,
) -> LuaResult<i64> {
    let mut sim = borrow_state_mut(state)?;
    let bucket = sim.auction_commodity_searches.entry(item_id).or_default();
    let (plan, _) = plan_commodity_purchase(&bucket.entries, quantity);
    drain_commodity_listings(&mut bucket.entries, quantity);
    Ok(total_price_for_plan(&plan))
}

pub(super) fn c_auction_house_cancel_commodities_purchase(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.commodity_purchase_quote = None;
    Ok(0)
}
