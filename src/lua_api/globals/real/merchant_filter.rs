//! INFERRED (R88dx; native evidence pending): merchant offer filtering.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::SimState;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const FILTER_CLASS: i32 = 2;
const FILTER_SPEC1: i32 = 3;
const FILTER_SPEC4: i32 = 6;
const FILTER_BOE: i32 = 7;

pub(crate) fn filtered_merchant_items(sim: &SimState) -> Vec<(usize, u32)> {
    sim.merchant_items
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, item_id)| merchant_item_matches_filter(sim, *item_id))
        .collect()
}

fn merchant_item_matches_filter(sim: &SimState, item_id: u32) -> bool {
    match sim.merchant_filter {
        FILTER_CLASS => item_matches_class(sim, item_id),
        FILTER_SPEC1..=FILTER_SPEC4 => {
            item_matches_spec(sim, item_id, (sim.merchant_filter - FILTER_SPEC1) as usize)
        }
        FILTER_BOE => crate::items::get_item(item_id).is_none_or(|item| item.bonding == 2),
        _ => true,
    }
}

fn item_matches_class(sim: &SimState, item_id: u32) -> bool {
    let Some(record) =
        crate::c_api::item_spell::item_spec_derive::derive_record(item_id, sim.player.level)
    else {
        return true;
    };
    record.no_traits_and_specializations
        || record.no_stat_traits
        || record.specialization_ids.iter().any(|spec_id| {
            crate::specializations::spec_by_id(*spec_id)
                .is_some_and(|spec| spec.class_id as i32 == sim.player.class_index)
        })
        || !record.reject_unmatched
}

fn item_matches_spec(sim: &SimState, item_id: u32, spec_index: usize) -> bool {
    let class_id = sim.player.class_index.max(0) as u32;
    let Some(spec) = crate::specializations::specs_for_class(class_id).nth(spec_index) else {
        return false;
    };
    let Some(record) =
        crate::c_api::item_spell::item_spec_derive::derive_record(item_id, sim.player.level)
    else {
        return true;
    };
    record.no_traits_and_specializations
        || record.no_stat_traits
        || record.specialization_ids.contains(&spec.id)
        || !record.reject_unmatched
}

pub(crate) fn visible_merchant_item(sim: &SimState, index: usize) -> Option<(usize, u32)> {
    filtered_merchant_items(sim).get(index).copied()
}

fn get_merchant_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = borrow_state(state)?.merchant_filter;
    state.push(Val::Num(f64::from(filter)));
    Ok(1)
}

fn set_merchant_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = Option::<i32>::from_stack(state, 1)?.unwrap_or(0);
    borrow_state_mut(state)?.merchant_filter = filter;
    Ok(0)
}

fn reset_set_merchant_filter(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.merchant_filter = FILTER_CLASS;
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    if cfg!(any(
        feature = "client-retail",
        feature = "client-wowforever"
    )) {
        LuaApiMut::register_function(lua, "GetMerchantFilter", get_merchant_filter)?;
        LuaApiMut::register_function(lua, "SetMerchantFilter", set_merchant_filter)?;
        LuaApiMut::register_function(lua, "ResetSetMerchantFilter", reset_set_merchant_filter)?;
    }
    Ok(())
}
