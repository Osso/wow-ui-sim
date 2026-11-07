//! `C_TooltipInfo.GetMerchantCostItem(slot, costIndex)`.
//!
//! Evidence: decomp Round 6 §2, Retail 12.1.5 (70688). The callback reads two
//! `luaIndex` arguments and selects one entry of the merchant offer's
//! extended-cost record. The selectable sequence is the record's surviving item
//! costs first, then its surviving currency costs, so `costIndex` is a
//! zero-based ordinal over that sequence and not a record position.
//!
//! The selection block is decoded for Retail 12.1.5 only; Forever's block is
//! undecoded, so this getter is registered for Retail and PTR.

use crate::c_api::tooltip_getter_args;
use crate::lua_api::globals::missing_surface::tooltip_info::{
    tooltip_for_cost_currency, tooltip_for_item_context,
};
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::MerchantCostRecord;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetMerchantCostItem",
        get_merchant_cost_item,
    )
}

/// Item class 10 is the money cost. The native selection block skips such an
/// item row entirely, which is why a purely monetary vendor cost has no ordinal.
const MONEY_ITEM_CLASS: u8 = 10;

enum MerchantCost {
    Item(u32, &'static crate::items::ItemInfo),
    Currency(u32),
}

fn get_merchant_cost_item(state: &mut LuaState) -> LuaResult<u32> {
    let slot = tooltip_getter_args::lua_index(state, 1)?;
    let cost_index = tooltip_getter_args::lua_index(state, 2)?;
    let selected = match (slot, cost_index) {
        (Some(slot), Some(cost_index)) => select_cost(state, slot, cost_index),
        _ => None,
    };
    match selected {
        Some(MerchantCost::Item(item_id, item)) => {
            let tooltip = tooltip_for_item_context(state, item_id, Some(item));
            state.push(tooltip);
            Ok(1)
        }
        Some(MerchantCost::Currency(currency_id)) => {
            let tooltip = tooltip_for_cost_currency(state, currency_id);
            state.push(tooltip);
            Ok(1)
        }
        None => emulated_missing_tooltip(),
    }
}

/// A slot that does not resolve to a merchant offer, or an offer without a
/// supplied extended-cost record, selects nothing.
fn select_cost(state: &LuaState, slot: usize, cost_index: usize) -> Option<MerchantCost> {
    let record = {
        let sim = borrow_state(state).ok()?;
        let slot_key = u32::try_from(slot).ok()?;
        if !sim
            .merchant_items
            .get(slot)
            .is_some_and(|item_id| *item_id != 0)
        {
            return None;
        }
        sim.merchant_offer_costs.get(&slot_key).copied()?
    };
    selectable_costs(&record).nth(cost_index)
}

/// The record's item costs in record order, then its currency costs in record
/// order (decomp Round 6 §2c).
fn selectable_costs(record: &MerchantCostRecord) -> impl Iterator<Item = MerchantCost> + '_ {
    let item_costs = record
        .item_costs
        .iter()
        .copied()
        .filter_map(selectable_item_cost);
    let currency_costs = record
        .currency_costs
        .iter()
        .copied()
        .filter(|currency_id| *currency_id != 0)
        .map(MerchantCost::Currency);
    item_costs.chain(currency_costs)
}

/// Native item-cost skips: a zero identity, a failed sparse-item lookup, a
/// missing item row, and the money class leave no ordinal for the entry.
fn selectable_item_cost(item_id: u32) -> Option<MerchantCost> {
    if item_id == 0 {
        return None;
    }
    let item = crate::items::get_item(item_id)?;
    let class = crate::c_api::item_spell::item_spec_derive::item_class(item_id)?;
    (class != MONEY_ITEM_CLASS).then_some(MerchantCost::Item(item_id, item))
}

/// EMULATED: the native failure exit returns no values, or raises when a pending
/// error message exists (decomp Round 6 §2b). The simulator models no pending
/// merchant-cost error text, so every miss takes the no-values arm.
fn emulated_missing_tooltip() -> LuaResult<u32> {
    Ok(0)
}
