//! Bulk reads of the existing auction result cache.
use super::{ensure_namespace, helpers::set_table_array};
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, create_table_with_fields,
};
use crate::lua_api::state::{BidAuction, OwnedAuction};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_AuctionHouse")?;
    table_set_rust_fn_static(state, ns, "GetNumReplicateItems", replicate_count)?;
    table_set_rust_fn_static(state, ns, "GetOwnedAuctions", owned_auctions)?;
    table_set_rust_fn_static(state, ns, "GetBids", bids)
}

fn replicate_count(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.auction_replicate_items.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn item_key(state: &mut LuaState, item: i32, level: i32) -> Val {
    // Same item-key defaults as the existing indexed auction reads.
    create_table_with_fields(
        state,
        &[
            ("itemID", Val::Num(f64::from(item))),
            ("itemLevel", Val::Num(f64::from(level))),
            ("itemSuffix", Val::Num(0.0)),
            ("battlePetSpeciesID", Val::Num(0.0)),
        ],
    )
}

fn owned_row(state: &mut LuaState, row: &OwnedAuction) -> Val {
    let key = item_key(state, row.item_id, row.item_level);
    create_table_with_fields(
        state,
        &[
            ("itemKey", key),
            ("auctionID", Val::Num(f64::from(row.auction_id))),
            ("status", Val::Num(f64::from(row.status))),
            ("quantity", Val::Num(f64::from(row.quantity))),
            ("timeLeft", Val::Num(f64::from(row.time_left))),
            ("timeLeftSeconds", Val::Num(row.time_left_seconds as f64)),
            ("bidAmount", Val::Num(row.bid_amount as f64)),
            ("buyoutAmount", Val::Num(row.buyout_amount as f64)),
        ],
    )
}

fn bid_row(state: &mut LuaState, row: &BidAuction) -> Val {
    let key = item_key(state, row.item_id, row.item_level);
    let bidder = row
        .bidder
        .as_deref()
        .map_or(Val::Nil, |name| create_string(state, name));
    create_table_with_fields(
        state,
        &[
            ("itemKey", key),
            ("auctionID", Val::Num(f64::from(row.auction_id))),
            ("timeLeft", Val::Num(f64::from(row.time_left))),
            ("bidAmount", Val::Num(row.bid_amount as f64)),
            ("buyoutAmount", Val::Num(row.buyout_amount as f64)),
            ("bidder", bidder),
        ],
    )
}

fn owned_auctions(state: &mut LuaState) -> LuaResult<u32> {
    let rows = borrow_state(state)?.auction_owned.clone();
    push_results(state, &rows, owned_row)
}

fn bids(state: &mut LuaState) -> LuaResult<u32> {
    let rows = borrow_state(state)?.auction_bids.clone();
    push_results(state, &rows, bid_row)
}

fn push_results<T>(
    state: &mut LuaState,
    rows: &[T],
    project: fn(&mut LuaState, &T) -> Val,
) -> LuaResult<u32> {
    let result = create_table(state);
    for (index, row) in rows.iter().enumerate() {
        let projected = project(state, row);
        set_table_array(state, result, index as i64 + 1, projected);
    }
    state.push(result);
    Ok(1)
}

#[cfg(test)]
#[test]
fn auction_result_arrays_track_supplied_rows() {
    crate::lua_api::WowLuaEnv::new().unwrap().exec(r#"
        A_Admin.ClearAuctionReplicateItems()
        A_Admin.ClearOwnedAuctions()
        A_Admin.ClearAuctionBids()
        assert(C_AuctionHouse.GetNumReplicateItems() == 0)
        assert(#C_AuctionHouse.GetOwnedAuctions() == 0 and #C_AuctionHouse.GetBids() == 0)
        A_Admin.AddAuctionReplicateItem("Temporal Alloy",134400,17,3,true,80,"Item Level")
        A_Admin.AddAuctionReplicateItem("Second Alloy",134401,6,2,true,70,"Item Level")
        A_Admin.AddOwnedAuction(101,501,70,3,90,150,0,2,200)
        A_Admin.AddOwnedAuction(102,502,80,4,80,180,0,3,300)
        A_Admin.AddAuctionBid(201,601,60,1,123,456,1,100,"Player-1-12345678")
        assert(C_AuctionHouse.GetNumReplicateItems() == 2)
        local owned = C_AuctionHouse.GetOwnedAuctions()
        assert(#owned == 2 and owned[1].auctionID == 101 and owned[2].itemKey.itemID == 502)
        assert(owned[2].quantity == 4 and owned[2].buyoutAmount == 180 and owned[2].timeLeftSeconds == 300)
        local bids = C_AuctionHouse.GetBids()
        assert(#bids == 1 and bids[1].auctionID == 201 and bids[1].itemKey.itemLevel == 60)
        assert(bids[1].bidder == "Player-1-12345678" and bids[1].bidAmount == 123)
        owned[2].quantity = 99; bids[1].bidAmount = 0
        assert(C_AuctionHouse.GetOwnedAuctions()[2].quantity == 4 and C_AuctionHouse.GetBids()[1].bidAmount == 123)
        A_Admin.ClearAuctionBids(); A_Admin.ClearOwnedAuctions(); A_Admin.ClearAuctionReplicateItems()
        assert(C_AuctionHouse.GetNumReplicateItems() == 0 and #C_AuctionHouse.GetBids() == 0 and #C_AuctionHouse.GetOwnedAuctions() == 0)
    "#).unwrap();
}
