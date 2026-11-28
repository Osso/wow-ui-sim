//! Merchant extended costs, purchases and buybacks (Retail publication rows).
//!
//! Consumer: `Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua`
//! (`MerchantFrame_UpdateAltCurrency` reads `GetMerchantItemCostInfo` /
//! `GetMerchantItemCostItem`, the split-stack path reads
//! `GetMerchantItemMaxStack`, item buttons call `BuyMerchantItem` and
//! `BuybackItem`). Offers are the visible `merchant_items` rows; their price and
//! extended cost come from `A_Admin.SetMerchantOfferCost`. Cost ordinals follow
//! `C_TooltipInfo.GetMerchantCostItem`: item costs, then currency costs.
//! INFERRED (R-ag10-13): an empty cost ordinal returns `nil, 0` (the confirm
//! path multiplies every ordinal's count); an unaffordable purchase or buyback, or one without
//! free backpack space, changes nothing; a purchase fires `PLAYER_MONEY`,
//! `BAG_UPDATE(bag)` and `MERCHANT_UPDATE`.

use super::interaction_dialogs::{Arg, fire, push_optional_string};
use super::merchant_filter::visible_merchant_item;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get_num};
use crate::lua_api::state::{BagItem, MerchantCostRecord, SimState};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const COST_USAGE: &str =
    "Usage: A_Admin.SetMerchantOfferCost(index, price, {{itemID, count}}, {{currencyID, amount}})";

/// One selectable extended cost: an item stack or a currency amount.
#[derive(Clone, Copy)]
enum Cost {
    Item(u32, u32),
    Currency(u32, u32),
}

fn costs(record: &MerchantCostRecord) -> Vec<Cost> {
    let items = record.item_costs.iter().zip(record.item_cost_counts);
    let currencies = record
        .currency_costs
        .iter()
        .zip(record.currency_cost_amounts);
    let items = items
        .filter(|(id, _)| **id != 0)
        .map(|(id, n)| Cost::Item(*id, n));
    let currencies = currencies.filter(|(id, _)| **id != 0);
    items
        .chain(currencies.map(|(id, n)| Cost::Currency(*id, n)))
        .collect()
}

/// The merchant row's offer slot, item and cost record.
fn offer(sim: &SimState, index: i64) -> Option<(u32, u32, MerchantCostRecord)> {
    let row = usize::try_from(index.checked_sub(1)?).ok()?;
    let (slot, item_id) = visible_merchant_item(sim, row)?;
    let slot = u32::try_from(slot).ok()?;
    let record = sim
        .merchant_offer_costs
        .get(&slot)
        .copied()
        .unwrap_or_default();
    Some((slot, item_id, record))
}

fn get_merchant_item_cost_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i64::from_stack(state, 1)?;
    let count =
        offer(&*borrow_state(state)?, index).map_or(0, |(_, _, record)| costs(&record).len());
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn push_cost(state: &mut LuaState, cost: Cost) -> LuaResult<u32> {
    let (icon, value, link, currency_name) = match cost {
        Cost::Item(item_id, count) => {
            let icon = crate::items::get_item(item_id).map_or(0, |item| item.icon_file_data_id);
            let link = crate::c_api::item_spell::item_link_for_id(item_id);
            (icon, count, link, None)
        }
        Cost::Currency(currency_id, amount) => {
            let record = borrow_state(state)?
                .currency_info
                .get(&(currency_id as i32))
                .cloned();
            let link = crate::c_api::item_spell::build_currency_link(
                state,
                currency_id as i32,
                amount as i32,
            )?;
            let name = record.as_ref().map(|record| record.name.clone());
            (
                record.map_or(0, |record| record.icon_file_id),
                amount,
                Some(link),
                name,
            )
        }
    };
    state.push(Val::Num(f64::from(icon)));
    state.push(Val::Num(f64::from(value)));
    push_optional_string(state, link.as_deref());
    push_optional_string(state, currency_name.as_deref());
    Ok(4)
}

fn get_merchant_item_cost_item(state: &mut LuaState) -> LuaResult<u32> {
    let index = i64::from_stack(state, 1)?;
    let ordinal = usize::try_from(i64::from_stack(state, 2)? - 1).ok();
    let cost = offer(&*borrow_state(state)?, index).and_then(|(_, _, record)| {
        ordinal.and_then(|ordinal| costs(&record).get(ordinal).copied())
    });
    let Some(cost) = cost else {
        // The original consumer multiplies the count of every ordinal up to
        // MAX_ITEM_COST, so an empty ordinal reads as no texture and zero count.
        state.push(Val::Nil);
        state.push(Val::Num(0.0));
        return Ok(2);
    };
    push_cost(state, cost)
}

fn get_merchant_item_max_stack(state: &mut LuaState) -> LuaResult<u32> {
    let index = i64::from_stack(state, 1)?;
    let item_id = offer(&*borrow_state(state)?, index).map(|(_, item_id, _)| item_id);
    let stack = item_id
        .and_then(crate::items::get_item)
        .map_or(0, |item| item.stackable.max(1));
    state.push(Val::Num(f64::from(stack)));
    Ok(1)
}

fn bag_item_count(sim: &SimState, item_id: u32) -> u64 {
    let stacks = sim
        .bag_items
        .values()
        .filter(|item| item.item_id == item_id);
    stacks.map(|item| item.stack_count.max(0) as u64).sum()
}

fn currency_amount(sim: &SimState, currency_id: u32) -> u64 {
    let record = sim.currency_info.get(&(currency_id as i32));
    record.map_or(0, |record| record.quantity.max(0) as u64)
}

fn affordable(sim: &SimState, price: u64, costs: &[Cost], quantity: u64) -> bool {
    let money = u64::try_from(sim.player.money).unwrap_or(0) >= price * quantity;
    money
        && costs.iter().all(|cost| match *cost {
            Cost::Item(id, n) => bag_item_count(sim, id) >= u64::from(n) * quantity,
            Cost::Currency(id, n) => currency_amount(sim, id) >= u64::from(n) * quantity,
        })
}

fn free_backpack_slot(sim: &SimState) -> Option<(i32, i32)> {
    (0..=4).find_map(|bag| {
        (1..=sim.bag_num_slots(bag))
            .map(|slot| (bag, slot))
            .find(|location| !sim.bag_items.contains_key(location))
    })
}

fn take_bag_items(sim: &mut SimState, item_id: u32, mut needed: i32) {
    let mut locations: Vec<_> = sim
        .bag_items
        .iter()
        .filter(|(_, item)| item.item_id == item_id)
        .map(|(location, _)| *location)
        .collect();
    locations.sort();
    for location in locations {
        let Some(item) = sim.bag_items.get_mut(&location) else {
            continue;
        };
        let taken = needed.min(item.stack_count);
        item.stack_count -= taken;
        needed -= taken;
        if item.stack_count <= 0 {
            sim.bag_items.remove(&location);
        }
    }
}

fn pay_costs(sim: &mut SimState, costs: &[Cost], quantity: u32) {
    for cost in costs {
        match *cost {
            Cost::Item(id, n) => take_bag_items(sim, id, (n * quantity) as i32),
            Cost::Currency(id, n) => {
                if let Some(record) = sim.currency_info.get_mut(&(id as i32)) {
                    record.quantity -= (n * quantity) as i32;
                }
            }
        }
    }
}

fn place_item(sim: &mut SimState, location: (i32, i32), item_id: u32, count: i32) {
    sim.bag_items.insert(
        location,
        BagItem {
            item_id,
            stack_count: count,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
            owned_identity: None,
        },
    );
}

fn fire_purchase(state: &mut LuaState, bag: i32, currency: bool) -> LuaResult<()> {
    fire(state, "PLAYER_MONEY", Vec::new())?;
    if currency {
        fire(state, "CURRENCY_DISPLAY_UPDATE", Vec::new())?;
    }
    fire(state, "BAG_UPDATE", vec![Arg::Num(f64::from(bag))])?;
    fire(state, "MERCHANT_UPDATE", Vec::new())
}

/// Pays for and stores `quantity` of the offer; returns the bag and whether a
/// currency was spent.
fn purchase(sim: &mut SimState, index: i64, quantity: u32) -> Option<(i32, bool)> {
    let (_, item_id, record) = offer(sim, index)?;
    let costs = costs(&record);
    if quantity == 0 || !affordable(sim, record.price, &costs, u64::from(quantity)) {
        return None;
    }
    let location = free_backpack_slot(sim)?;
    sim.player.money -= (record.price * u64::from(quantity)) as i64;
    pay_costs(sim, &costs, quantity);
    place_item(sim, location, item_id, quantity as i32);
    let currency = costs.iter().any(|cost| matches!(cost, Cost::Currency(..)));
    Some((location.0, currency))
}

fn buy_merchant_item(state: &mut LuaState) -> LuaResult<u32> {
    let index = i64::from_stack(state, 1)?;
    let quantity = Option::<u32>::from_stack(state, 2)?.unwrap_or(1);
    let bought = purchase(&mut *borrow_state_mut(state)?, index, quantity);
    if let Some((bag, currency)) = bought {
        fire_purchase(state, bag, currency)?;
    }
    Ok(0)
}

fn buy_back(sim: &mut SimState, index: i64) -> Option<i32> {
    let slot = usize::try_from(index.checked_sub(1)?).ok()?;
    let entry = sim.merchant_buyback_items.get(slot)?.clone();
    let money = u64::try_from(sim.player.money).unwrap_or(0);
    let location = free_backpack_slot(sim).filter(|_| money >= entry.price)?;
    sim.player.money -= entry.price as i64;
    sim.merchant_buyback_items.remove(slot);
    place_item(sim, location, entry.item_id, entry.quantity as i32);
    Some(location.0)
}

fn buyback_item(state: &mut LuaState) -> LuaResult<u32> {
    let index = i64::from_stack(state, 1)?;
    let bag = buy_back(&mut *borrow_state_mut(state)?, index);
    if let Some(bag) = bag {
        fire_purchase(state, bag, false)?;
    }
    Ok(0)
}

fn read_cost_pairs(state: &mut LuaState, index: i32) -> LuaResult<Vec<(u32, u32)>> {
    let rows = match stack_val(state, index) {
        Val::Nil => return Ok(Vec::new()),
        Val::Table(table) => {
            let rows = state.gc.tables.get(table).map(|t| t.array_slice().to_vec());
            rows.unwrap_or_default()
        }
        _ => return Err(runtime_error(COST_USAGE)),
    };
    if rows.len() > 5 {
        return Err(runtime_error(COST_USAGE));
    }
    rows.into_iter()
        .map(|row| read_cost_pair(state, row))
        .collect()
}

fn read_cost_pair(state: &mut LuaState, row: Val) -> LuaResult<(u32, u32)> {
    let Val::Table(table) = row else {
        return Err(runtime_error(COST_USAGE));
    };
    match (
        table_get_num(state, table, 1.0),
        table_get_num(state, table, 2.0),
    ) {
        (Val::Num(id), Val::Num(n)) if id >= 1.0 && n >= 1.0 => Ok((id as u32, n as u32)),
        _ => Err(runtime_error(COST_USAGE)),
    }
}

fn set_merchant_offer_cost(state: &mut LuaState) -> LuaResult<u32> {
    let index = u32::from_stack(state, 1)?;
    let price = f64::from_stack(state, 2)?;
    if !price.is_finite() || price < 0.0 {
        return Err(runtime_error(COST_USAGE));
    }
    let price = price as u64;
    let items = read_cost_pairs(state, 3)?;
    let currencies = read_cost_pairs(state, 4)?;
    let slot = index
        .checked_sub(1)
        .ok_or_else(|| runtime_error(COST_USAGE))?;
    let mut record = MerchantCostRecord {
        price,
        ..MerchantCostRecord::default()
    };
    for (i, (id, n)) in items.into_iter().enumerate() {
        (record.item_costs[i], record.item_cost_counts[i]) = (id, n);
    }
    for (i, (id, n)) in currencies.into_iter().enumerate() {
        (record.currency_costs[i], record.currency_cost_amounts[i]) = (id, n);
    }
    borrow_state_mut(state)?
        .merchant_offer_costs
        .insert(slot, record);
    fire(state, "MERCHANT_UPDATE", Vec::new())?;
    Ok(0)
}

/// Appends a sold stack to the buyback tab: `itemID, quantity, price, usable, bound`.
/// INFERRED: a buyback slot offers its whole stack (`numAvailable = -1`).
fn add_buyback_item(state: &mut LuaState) -> LuaResult<u32> {
    let usage = "Usage: A_Admin.AddBuybackItem(itemID, quantity, price, isUsable, isBound)";
    let item_id = u32::from_stack(state, 1)?;
    let quantity = u32::from_stack(state, 2)?;
    let price = f64::from_stack(state, 3)?;
    if crate::items::get_item(item_id).is_none() || quantity == 0 || !(price >= 0.0) {
        return Err(runtime_error(usage));
    }
    let entry = super::merchant_buyback::BuybackItem {
        item_id,
        quantity,
        price: price as u64,
        num_available: -1,
        is_usable: bool::from_stack(state, 4)?,
        is_bound: bool::from_stack(state, 5)?,
    };
    borrow_state_mut(state)?.merchant_buyback_items.push(entry);
    fire(state, "MERCHANT_UPDATE", Vec::new())?;
    Ok(0)
}

/// Opens the merchant interaction the original `MerchantFrame` registers with
/// the player-interaction manager (`Enum.PlayerInteractionType.Merchant`).
/// `canRepair` makes it a repair merchant (`CanMerchantRepair`).
fn open_merchant(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::enum_data::PLAYER_INTERACTION_TYPE;
    let merchant = PLAYER_INTERACTION_TYPE
        .1
        .iter()
        .position(|entry| *entry == "Merchant")
        .ok_or_else(|| runtime_error("Enum.PlayerInteractionType.Merchant is not defined"))?
        as i32;
    let can_repair = Option::<bool>::from_stack(state, 1)?.unwrap_or(false);
    {
        let mut sim = borrow_state_mut(state)?;
        sim.merchant_frame_open = true;
        sim.merchant_repair_capable = can_repair;
        sim.active_player_interactions.insert(merchant);
    }
    let args = vec![Arg::Num(f64::from(merchant))];
    // The manager calls MerchantFrame_MerchantShow for this interaction.
    fire(state, "PLAYER_INTERACTION_MANAGER_FRAME_SHOW", args)?;
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetMerchantItemCostInfo", get_merchant_item_cost_info)?;
    LuaApiMut::register_function(lua, "GetMerchantItemCostItem", get_merchant_item_cost_item)?;
    LuaApiMut::register_function(lua, "GetMerchantItemMaxStack", get_merchant_item_max_stack)?;
    LuaApiMut::register_function(lua, "BuyMerchantItem", buy_merchant_item)?;
    LuaApiMut::register_function(lua, "BuybackItem", buyback_item)
}

pub(crate) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("SetMerchantOfferCost", set_merchant_offer_cost)?
        .set_function("OpenMerchant", open_merchant)?
        .set_function("AddBuybackItem", add_buyback_item)
}
