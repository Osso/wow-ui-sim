//! Legacy merchant affordability query backed by host-supplied offer and inventory state.

use crate::c_api::helpers::parse_numeric_string;
use crate::items;
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::{MerchantCostRecord, SimState};
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const USAGE: &str = "Usage: CanAffordMerchantItem(index)";

fn can_afford_merchant_item(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_merchant_index(state)?;
    let Some(affordable) = merchant_offer_affordability(state, index)? else {
        return Ok(0);
    };
    state.push(Val::Bool(affordable));
    Ok(1)
}

fn read_merchant_index(state: &LuaState) -> LuaResult<i32> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| parse_numeric_string(string.data())),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error(USAGE))?;

    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number) {
        return Err(rilua::runtime_error(format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    Ok(number.trunc() as i32)
}

fn merchant_offer_affordability(state: &LuaState, index: i32) -> LuaResult<Option<bool>> {
    let Some(slot) = usize::try_from(index.wrapping_sub(1)).ok() else {
        return Ok(None);
    };
    let sim = borrow_state(state)?;
    let Some(&item_id) = sim.merchant_items.get(slot) else {
        return Ok(None);
    };
    if item_id == 0 || !sim.player.present || items::get_item(item_id).is_none() {
        return Ok(None);
    }
    let Some(slot_key) = u32::try_from(slot).ok() else {
        return Ok(None);
    };
    let Some(costs) = sim.merchant_offer_costs.get(&slot_key) else {
        return Ok(None);
    };
    Ok(Some(is_offer_affordable(&sim, costs)))
}

fn is_offer_affordable(sim: &SimState, costs: &MerchantCostRecord) -> bool {
    let has_money = u64::try_from(sim.player.money).is_ok_and(|money| money >= costs.price);
    let has_required_items = costs
        .item_costs
        .iter()
        .zip(costs.item_cost_counts)
        .all(|(&item_id, required)| has_required_item_count(sim, item_id, required));
    let has_required_currencies = costs
        .currency_costs
        .iter()
        .zip(costs.currency_cost_amounts)
        .all(|(&currency_id, required)| has_required_currency_amount(sim, currency_id, required));
    // INFERRED: no host input describes the additional native record requirement.
    let other_record_requirements = true;

    has_money && has_required_items && has_required_currencies && other_record_requirements
}

fn has_required_item_count(sim: &SimState, item_id: u32, required: u32) -> bool {
    if item_id == 0 || items::get_item(item_id).is_none() {
        return true;
    }
    let available = sim
        .bag_items
        .values()
        .filter(|item| item.item_id == item_id)
        .map(|item| u64::try_from(item.stack_count.max(0)).unwrap_or_default())
        .sum::<u64>();
    available >= u64::from(required)
}

fn has_required_currency_amount(sim: &SimState, currency_id: u32, required: u32) -> bool {
    if currency_id == 0 {
        return true;
    }
    let available = i32::try_from(currency_id)
        .ok()
        .and_then(|currency_id| sim.currency_info.get(&currency_id))
        .map_or(0, |currency| currency.quantity.max(0) as u32);
    available >= required
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "CanAffordMerchantItem", can_afford_merchant_item)
}
