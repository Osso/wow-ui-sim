//! Supplied item observations, not an upgrade-policy or currency model.

use std::collections::HashMap;

pub(crate) use crate::c_api::item_location::ItemSlot;
use crate::c_api::item_location::{ItemLocationReadError, read_r88s_item_slot};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::{CursorInfo, CursorItemOrigin, SimState};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Clone, Copy)]
struct Observation {
    item_id: u32,
    eligible: bool,
}

#[derive(Debug, Default)]
pub struct ItemUpgradeEligibility {
    locations: HashMap<ItemSlot, Observation>,
    cursor: Option<(CursorItemOrigin, Observation)>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Wrath {
        return Ok(());
    }
    // ItemUpgradeDocumentation.lua: R/P/F11–24; M/E/A11–23.
    let namespace = super::ensure_namespace(state, "C_ItemUpgrade")?;
    table_set_rust_fn_static(state, namespace, "CanUpgradeItem", can_upgrade_item)
}

fn can_upgrade_item(state: &mut LuaState) -> LuaResult<u32> {
    let location = stack_val(state, 1);
    let slot = match read_r88s_item_slot(state, location) {
        Ok(slot) => slot,
        Err(ItemLocationReadError::Invalid) => {
            return Err(rilua::runtime_error(
                "C_ItemUpgrade.CanUpgradeItem: itemLocation must be a bag or equipment ItemLocation",
            ));
        }
        Err(ItemLocationReadError::Unavailable) => {
            state.push(Val::Bool(false));
            return Ok(1);
        }
    };
    // Best guess, native UNKNOWN: without a supplied observation the item is
    // not upgradable, as for most items and for empty locations.
    let sim = borrow_state(state)?;
    let item_id = slot.item_id(&sim);
    let eligible = sim
        .item_upgrade_eligibility
        .locations
        .get(&slot)
        .filter(|observation| Some(observation.item_id) == item_id)
        .is_some_and(|observation| observation.eligible);
    state.push(Val::Bool(eligible));
    Ok(1)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if ACTIVE == ClientProfile::Wrath {
        return Ok(builder);
    }
    builder
        .set_function("SetItemUpgradeEligibility", supply_eligibility)?
        .set_function("ClearItemUpgradeEligibility", clear_eligibility)
}

fn supply_eligibility(state: &mut LuaState) -> LuaResult<u32> {
    let location = stack_val(state, 1);
    let slot = read_r88s_item_slot(state, location).map_err(|_| {
        rilua::runtime_error("SetItemUpgradeEligibility requires a resolvable item location")
    })?;
    let eligible = bool::from_stack(state, 2)?;
    let item_id = {
        let sim = borrow_state(state)?;
        slot.item_id(&sim)
    }
    .ok_or_else(|| rilua::runtime_error("SetItemUpgradeEligibility requires an existing item"))?;
    borrow_state_mut(state)?
        .item_upgrade_eligibility
        .locations
        .insert(slot, Observation { item_id, eligible });
    Ok(0)
}

fn clear_eligibility(state: &mut LuaState) -> LuaResult<u32> {
    let location = stack_val(state, 1);
    if location == Val::Nil {
        borrow_state_mut(state)?.item_upgrade_eligibility = ItemUpgradeEligibility::default();
        return Ok(0);
    }
    let slot = read_r88s_item_slot(state, location).map_err(|_| {
        rilua::runtime_error("ClearItemUpgradeEligibility requires an item location or nil")
    })?;
    let mut sim = borrow_state_mut(state)?;
    discard_location(&mut sim, slot);
    Ok(0)
}

fn take_location_observation(sim: &mut SimState, slot: ItemSlot) -> Option<Observation> {
    let item_id = slot.item_id(sim);
    sim.item_upgrade_eligibility
        .locations
        .remove(&slot)
        .filter(|observation| Some(observation.item_id) == item_id)
}

fn take_cursor_observation(sim: &mut SimState) -> Option<Observation> {
    let (origin, observation) = sim.item_upgrade_eligibility.cursor.take()?;
    match &sim.cursor_item {
        Some(CursorInfo::Item {
            item_id,
            origin: current_origin,
            ..
        }) if *item_id == observation.item_id && *current_origin == origin => Some(observation),
        _ => None,
    }
}

/// Call immediately before the existing inventory swap, while both items resolve.
pub(crate) fn swap_with_cursor(sim: &mut SimState, slot: ItemSlot) {
    let incoming = take_cursor_observation(sim);
    let displaced = take_location_observation(sim, slot);
    if let Some(observation) = incoming {
        sim.item_upgrade_eligibility
            .locations
            .insert(slot, observation);
    }
    sim.item_upgrade_eligibility.cursor =
        displaced.map(|observation| (slot.cursor_origin(), observation));
}

pub(crate) fn discard_location(sim: &mut SimState, slot: ItemSlot) {
    sim.item_upgrade_eligibility.locations.remove(&slot);
}

pub(crate) fn discard_bags(sim: &mut SimState) {
    sim.item_upgrade_eligibility
        .locations
        .retain(|slot, _| matches!(slot, ItemSlot::Equipment(_)));
}

pub(crate) fn discard_cursor(sim: &mut SimState) {
    sim.item_upgrade_eligibility.cursor = None;
}
