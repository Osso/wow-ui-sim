//! Shared ItemLocation parsing and host evidence for R88s inventory bounds.

use std::collections::HashSet;

use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_api::state::{CursorItemOrigin, SimState};
use crate::lua_bridge::{TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const LAST_NATIVE_CONTAINER_INDEX: i32 = 15;
const REINDEXED_CONTAINER_INDEX_START: i32 = 5;
const REINDEXED_CONTAINER_INDEX_END: i32 = 10;
const SPECIAL_PUBLIC_SLOT_COUNT: i32 = 5;
const FIRST_SPECIAL_MAPPED_SLOT: i32 = 101;
const SPECIAL_EQUIPMENT_SLOT_RANGES: [(i32, i32); 2] = [(64, 69), (101, 105)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ItemSlot {
    Bag(i32, i32),
    Equipment(i32),
}

impl ItemSlot {
    pub(crate) fn item_id(self, sim: &SimState) -> Option<u32> {
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

    pub(crate) fn cursor_origin(self) -> CursorItemOrigin {
        match self {
            Self::Bag(bag, slot) => CursorItemOrigin::Bag { bag, slot },
            Self::Equipment(slot) => CursorItemOrigin::Equipped { slot },
        }
    }
}

/// Distinguish malformed shapes from valid locations that cannot be resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemLocationReadError {
    Invalid,
    Unavailable,
}

/// Explicit inputs; no simulator producer supplies native inventory state.
#[derive(Debug, Clone, Default)]
pub struct R88sInventoryInputs {
    /// Host-supplied native player-inventory entry count. `None` is UNKNOWN.
    pub inventory_count: Option<i32>,
    /// Host-supplied native inventory-mode state. `None` is UNKNOWN.
    pub inventory_mode_active: Option<bool>,
    /// Host observations for special equipment slots when mode is active.
    /// Missing slots are UNKNOWN and reject.
    pub available_special_equipment_slots: HashSet<i32>,
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return Ok(builder);
    }
    builder.set_function("SetItemLocationInputs", set_item_location_inputs)
}

fn set_item_location_inputs(state: &mut LuaState) -> LuaResult<u32> {
    let inventory_count = read_optional_i32(stack_val(state, 1), "inventoryCount")?;
    let inventory_mode_active = read_optional_bool(stack_val(state, 2), "inventoryModeActive")?;
    let slots = read_special_slots(state)?;
    borrow_state_mut(state)?.r88s_inventory_inputs = R88sInventoryInputs {
        inventory_count,
        inventory_mode_active,
        available_special_equipment_slots: slots,
    };
    Ok(0)
}

fn read_special_slots(state: &LuaState) -> LuaResult<HashSet<i32>> {
    let Val::Table(reference) = stack_val(state, 3) else {
        return Err(runtime_error(
            "A_Admin.SetItemLocationInputs: availableEquipmentSlots must be a table",
        ));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("availableEquipmentSlots table is unavailable"))?;
    table
        .array_slice()
        .iter()
        .map(|value| read_required_i32(*value, "availableEquipmentSlots"))
        .collect()
}

fn read_optional_i32(value: Val, field: &str) -> LuaResult<Option<i32>> {
    if value.is_nil() {
        Ok(None)
    } else {
        read_required_i32(value, field).map(Some)
    }
}

fn read_required_i32(value: Val, field: &str) -> LuaResult<i32> {
    match value {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value) =>
        {
            Ok(value as i32)
        }
        _ => Err(runtime_error(format!(
            "A_Admin.SetItemLocationInputs: {field} must be a signed integer"
        ))),
    }
}

fn read_optional_bool(value: Val, field: &str) -> LuaResult<Option<bool>> {
    match value {
        Val::Nil => Ok(None),
        Val::Bool(value) => Ok(Some(value)),
        _ => Err(runtime_error(format!(
            "A_Admin.SetItemLocationInputs: {field} must be a boolean or nil"
        ))),
    }
}

/// Read the Retail R88s.s1 ItemLocation forms with their inventory bounds.
/// The shared use by item-upgrade eligibility is a simulator policy.
pub(crate) fn read_r88s_item_slot(
    state: &mut LuaState,
    location: Val,
) -> Result<ItemSlot, ItemLocationReadError> {
    if !matches!(location, Val::Table(_)) {
        return Err(ItemLocationReadError::Invalid);
    }

    let equipment = table_get(state, location, "equipmentSlotIndex");
    if let Some(number) = location_number(state, equipment) {
        let equipment_slot = location_integer(number).ok_or(ItemLocationReadError::Invalid)?;
        return read_equipment_slot(state, equipment_slot)
            .ok_or(ItemLocationReadError::Unavailable);
    }

    let bag_value = table_get(state, location, "bagID");
    let bag_id = location_integer_field(state, bag_value)?;
    let slot_value = table_get(state, location, "slotIndex");
    let public_slot = location_integer_field(state, slot_value)?;
    read_bag_slot(state, bag_id, public_slot).ok_or(ItemLocationReadError::Unavailable)
}

/// Keep the integral-number parser for APIs outside the R88s shared path.
pub(crate) fn read_numeric_item_slot(state: &mut LuaState, location: Val) -> Option<ItemSlot> {
    if !matches!(location, Val::Table(_)) {
        return None;
    }

    let equipment = table_get(state, location, "equipmentSlotIndex");
    let bag = table_get(state, location, "bagID");
    let slot = table_get(state, location, "slotIndex");
    match (equipment, bag, slot) {
        (Val::Num(equipment), Val::Nil, Val::Nil) => {
            read_integral(equipment).map(ItemSlot::Equipment)
        }
        (Val::Nil, Val::Num(bag), Val::Num(slot)) => {
            Some(ItemSlot::Bag(read_integral(bag)?, read_integral(slot)?))
        }
        _ => None,
    }
}

fn location_integer_field(state: &LuaState, value: Val) -> Result<i32, ItemLocationReadError> {
    location_number(state, value)
        .and_then(location_integer)
        .ok_or(ItemLocationReadError::Invalid)
}

fn location_number(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| crate::c_api::helpers::parse_numeric_string(string.data())),
        _ => None,
    }
}

fn location_integer(number: f64) -> Option<i32> {
    let in_signed_32_range = number >= f64::from(i32::MIN) && number <= f64::from(i32::MAX);
    if !number.is_finite() || !in_signed_32_range {
        return None;
    }
    Some(number.trunc() as i32)
}

fn read_bag_slot(state: &LuaState, bag_id: i32, public_slot: i32) -> Option<ItemSlot> {
    if public_slot < 1 {
        return None;
    }

    let bag_index = bag_id.checked_sub(1)?;
    match bag_index {
        -1 => read_backpack_slot(state, public_slot),
        0..=LAST_NATIVE_CONTAINER_INDEX => read_container_slot(state, bag_index, public_slot),
        _ => read_player_inventory_slot(state, bag_index, public_slot),
    }
}

fn read_backpack_slot(state: &LuaState, public_slot: i32) -> Option<ItemSlot> {
    let sim = borrow_state(state).ok()?;
    let inventory_count = sim.r88s_inventory_inputs.inventory_count?;
    let backpack_count = sim.bag_num_slots(0);
    (public_slot <= inventory_count && public_slot <= backpack_count)
        .then_some(ItemSlot::Bag(0, public_slot))
}

fn read_container_slot(state: &LuaState, bag_index: i32, public_slot: i32) -> Option<ItemSlot> {
    let bag = bag_index + 1;
    let native_slot = public_slot.checked_sub(1)?;
    if native_slot >= borrow_state(state).ok()?.bag_num_slots(bag) {
        return None;
    }

    if (REINDEXED_CONTAINER_INDEX_START..=REINDEXED_CONTAINER_INDEX_END).contains(&bag_index) {
        if native_slot >= SPECIAL_PUBLIC_SLOT_COUNT {
            return None;
        }
        return Some(ItemSlot::Bag(bag, FIRST_SPECIAL_MAPPED_SLOT + native_slot));
    }

    Some(ItemSlot::Bag(bag, native_slot + 1))
}

fn read_player_inventory_slot(
    state: &LuaState,
    bag_index: i32,
    public_slot: i32,
) -> Option<ItemSlot> {
    let equipment_slot = match bag_index {
        -3 | -4 => {
            let slot_index = public_slot.checked_sub(1)?;
            if slot_index >= SPECIAL_PUBLIC_SLOT_COUNT {
                return None;
            }
            FIRST_SPECIAL_MAPPED_SLOT + slot_index
        }
        index if index <= -5 => public_slot,
        _ => return None,
    };
    read_equipment_slot(state, equipment_slot)
}

fn read_equipment_slot(state: &LuaState, slot: i32) -> Option<ItemSlot> {
    if slot < 1 {
        return None;
    }

    let sim = borrow_state(state).ok()?;
    // INFERRED simulator policy: PlayerState.present proxies native player-inventory availability.
    if !sim.player.present {
        return None;
    }

    let inputs = &sim.r88s_inventory_inputs;
    if slot > inputs.inventory_count? {
        return None;
    }
    if requires_special_equipment_availability(slot) {
        match inputs.inventory_mode_active? {
            true if !inputs.available_special_equipment_slots.contains(&slot) => return None,
            true | false => {}
        }
    }

    Some(ItemSlot::Equipment(slot))
}

fn requires_special_equipment_availability(slot: i32) -> bool {
    SPECIAL_EQUIPMENT_SLOT_RANGES
        .iter()
        .any(|(first, last)| (*first..=*last).contains(&slot))
}

fn read_integral(number: f64) -> Option<i32> {
    let in_range = number >= f64::from(i32::MIN) && number <= f64::from(i32::MAX);
    (in_range && number.fract() == 0.0).then_some(number as i32)
}
