//! Supplied item observations, not an upgrade-policy or currency model.

use std::collections::HashMap;

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_api::state::{CursorInfo, CursorItemOrigin, SimState};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ItemSlot {
    Bag(i32, i32),
    Equipment(i32),
}

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

impl ItemSlot {
    fn item_id(self, sim: &SimState) -> Option<u32> {
        match self {
            Self::Bag(bag, slot) => sim.bag_items.get(&(bag, slot)).map(|item| item.item_id),
            Self::Equipment(slot) => sim
                .player
                .equipped_items
                .get(&slot)
                .map(|item| item.item_id)
                .or_else(|| {
                    sim.bag_info
                        .values()
                        .find(|bag| bag.inventory_slot == Some(slot))
                        .and_then(|bag| bag.item_id)
                }),
        }
    }

    fn cursor_origin(self) -> CursorItemOrigin {
        match self {
            Self::Bag(bag, slot) => CursorItemOrigin::Bag { bag, slot },
            Self::Equipment(slot) => CursorItemOrigin::Equipped { slot },
        }
    }
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
    let slot = read_item_slot(state, location).ok_or_else(|| {
        rilua::runtime_error(
            "C_ItemUpgrade.CanUpgradeItem: itemLocation must be a bag or equipment ItemLocation",
        )
    })?;
    let item_id = super::item_spell::location_item(state, location).map(|(id, _)| id);
    // Best guess, native UNKNOWN: without a supplied observation the item is
    // not upgradable, as for most items and for empty locations.
    let eligible = borrow_state(state)?
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
    let slot = read_item_slot(state, location).ok_or_else(|| {
        rilua::runtime_error("SetItemUpgradeEligibility requires a resolvable item location")
    })?;
    let eligible = bool::from_stack(state, 2)?;
    let item_id = super::item_spell::location_item(state, location)
        .map(|(id, _)| id)
        .ok_or_else(|| {
            rilua::runtime_error("SetItemUpgradeEligibility requires an existing item")
        })?;
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
    let slot = read_item_slot(state, location).ok_or_else(|| {
        rilua::runtime_error("ClearItemUpgradeEligibility requires an item location or nil")
    })?;
    let mut sim = borrow_state_mut(state)?;
    discard_location(&mut sim, slot);
    Ok(0)
}

fn read_item_slot(state: &mut LuaState, location: Val) -> Option<ItemSlot> {
    if !matches!(location, Val::Table(_)) {
        return None;
    }
    let equipment = table_get(state, location, "equipmentSlotIndex");
    let bag = table_get(state, location, "bagID");
    let slot = table_get(state, location, "slotIndex");
    match (equipment, bag, slot) {
        (Val::Num(equipment), Val::Nil, Val::Nil) => {
            read_integer(equipment).map(ItemSlot::Equipment)
        }
        (Val::Nil, Val::Num(bag), Val::Num(slot)) => {
            Some(ItemSlot::Bag(read_integer(bag)?, read_integer(slot)?))
        }
        _ => None,
    }
}

fn read_integer(number: f64) -> Option<i32> {
    let in_range = number >= i32::MIN as f64 && number <= i32::MAX as f64;
    (in_range && number.fract() == 0.0).then_some(number as i32)
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
