//! Pure allocation planning and inventory preflight, shared by crafting and orders.
use super::crafting_reagents::*;
use crate::lua_api::state::SimState;
use rilua::{LuaResult, runtime_error};
use std::collections::BTreeMap;

pub(crate) struct ReagentDeltas {
    pub items: Vec<(u32, i32)>,
    pub currencies: Vec<(u32, i32)>,
    pub returns: Vec<RegularReagentInfo>,
    pub required: Vec<RegularReagentInfo>,
}
pub(crate) fn default_allocations(slots: &[ReagentSlotSchematic]) -> Vec<CraftingReagentInfo> {
    slots
        .iter()
        .filter(|slot| slot.required)
        .filter_map(|slot| {
            slot.reagents.first().map(|reagent| CraftingReagentInfo {
                reagent: *reagent,
                quantity: slot.quantity_for(*reagent),
                data_slot_index: slot.data_slot_index,
            })
        })
        .collect()
}
pub(crate) fn plan_deltas(
    inputs: &CraftingInputs,
    recipe_id: i32,
    allocations: &[CraftingReagentInfo],
    count: i32,
) -> LuaResult<ReagentDeltas> {
    if count <= 0 {
        return Err(runtime_error("craft count must be positive"));
    }
    let slots = inputs
        .recipe_slots
        .get(&recipe_id)
        .ok_or_else(|| runtime_error("recipe reagent slots unavailable"))?;
    validate_allocations(slots, allocations).map_err(runtime_error)?;
    let mut totals = sum_allocations(allocations, count)?;
    let required = totals
        .iter()
        .map(|(reagent, quantity)| RegularReagentInfo {
            reagent: *reagent,
            quantity: *quantity,
        })
        .collect();
    let returns = subtract_resource_returns(inputs, recipe_id, count, &mut totals)?;
    Ok(build_deltas(&totals, returns, required))
}
fn build_deltas(
    totals: &BTreeMap<CraftingReagent, i32>,
    returns: Vec<RegularReagentInfo>,
    required: Vec<RegularReagentInfo>,
) -> ReagentDeltas {
    let items = totals
        .iter()
        .filter_map(|(key, value)| match key {
            CraftingReagent::Item(id) => Some((*id, *value)),
            _ => None,
        })
        .collect();
    let currencies = totals
        .iter()
        .filter_map(|(key, value)| match key {
            CraftingReagent::Currency(id) => Some((*id, *value)),
            _ => None,
        })
        .collect();
    ReagentDeltas {
        items,
        currencies,
        returns,
        required,
    }
}
fn sum_allocations(
    allocations: &[CraftingReagentInfo],
    count: i32,
) -> LuaResult<BTreeMap<CraftingReagent, i32>> {
    let mut totals = BTreeMap::<CraftingReagent, i32>::new();
    for allocation in allocations {
        let quantity = allocation
            .quantity
            .checked_mul(count)
            .ok_or_else(|| runtime_error("crafting quantity overflow"))?;
        let total = totals.entry(allocation.reagent).or_default();
        *total = total
            .checked_add(quantity)
            .ok_or_else(|| runtime_error("crafting quantity overflow"))?;
    }
    Ok(totals)
}
fn subtract_resource_returns(
    inputs: &CraftingInputs,
    recipe_id: i32,
    count: i32,
    totals: &mut BTreeMap<CraftingReagent, i32>,
) -> LuaResult<Vec<RegularReagentInfo>> {
    let mut returns = inputs
        .resource_returns
        .get(&recipe_id)
        .cloned()
        .unwrap_or_default();
    for entry in &mut returns {
        entry.quantity = entry
            .quantity
            .checked_mul(count)
            .ok_or_else(|| runtime_error("resource return overflow"))?;
        let total = totals
            .get_mut(&entry.reagent)
            .ok_or_else(|| runtime_error("returned resource not allocated"))?;
        if entry.quantity <= 0 || entry.quantity > *total {
            return Err(runtime_error("invalid resource return quantity"));
        }
        *total -= entry.quantity;
    }
    Ok(returns)
}
pub(crate) fn has_resources(sim: &SimState, deltas: &ReagentDeltas) -> bool {
    deltas.required.iter().all(|entry| {
        let available = match entry.reagent {
            CraftingReagent::Item(id) => sim
                .bag_items
                .values()
                .filter(|slot| slot.item_id == id)
                .map(|slot| i64::from(slot.stack_count))
                .sum(),
            CraftingReagent::Currency(id) => sim
                .currency_info
                .get(&(id as i32))
                .map_or(0, |info| i64::from(info.quantity)),
        };
        available >= i64::from(entry.quantity)
    })
}
