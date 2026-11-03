//! Retail `C_PaperDollInfo.CanCursorCanGoInSlot` (WM65, WM67).
//!
//! A local compatibility decision, not equip eligibility: it reads no lock,
//! combat or input-permission state.

use std::ops::RangeInclusive;

use super::item_spec_derive::{ItemSpecInput, item_spec_input};
use crate::lua_api::globals::missing_surface::professions::primary_profession_skill_lines;
use crate::lua_api::globals::real::cursor_model::{MODE_ITEM, native_mode};
use crate::lua_api::state::{CursorInfo, CursorItemOrigin, SimState};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use sim_data::profession_rows::PROFESSION_ROWS;

/// Compatibility masks for InventoryType 0–30; bit N is native slot N (WM67).
const INVENTORY_TYPE_MASKS: [u64; 31] = [
    0,
    0x1,
    0x2,
    0x4,
    0x8,
    0x10,
    0x20,
    0x40,
    0x80,
    0x100,
    0x200,
    0xc00,
    0x3000,
    0x18000,
    0x10000,
    0x8000,
    0x4000,
    0x18000,
    0x7_c000_0000,
    0x40000,
    0x10,
    0x8000,
    0x10000,
    0x10000,
    0,
    0x8000,
    0x8000,
    0,
    0,
    0xa48_0000,
    0x35b0_0000,
];
const MAX_SLOT_ARGUMENT: f64 = 4_294_967_295.0;
/// Native slots from here on accept before inventory-type matching.
const FIRST_UNMASKED_SLOT: u32 = 35;
const OFFHAND_SLOT: u32 = 16;
const REAGENT_BAG_SLOT: u32 = 34;
const TWO_HAND_TYPE: u8 = 17;
const BAG_TYPE: u8 = 18;
const REAGENT_BAG_SUBCLASS: u8 = 11;
const WEAPON_CLASS: u8 = 2;
const PROFESSION_CLASS: u8 = 19;
/// Weapon subclasses (low five bits) a two-handed off-hand item may have.
const OFFHAND_TWO_HAND_SUBCLASSES: [u8; 5] = [1, 5, 6, 8, 10];
const SUBCLASS_LOW_BITS: u8 = 31;
/// Profession subclasses 0–13 select Profession rows 1–14; others use key 15.
const LAST_ROW_SUBCLASS: u8 = 13;
const UNLISTED_PROFESSION_KEY: u32 = 15;
/// Fixed keys without a player-field comparison; their names are UNKNOWN.
const FIXED_KEY_25_26: u32 = 5;
const FIXED_KEY_27_29: u32 = 10;

pub(crate) fn can_cursor_go_in_slot(state: &mut LuaState) -> LuaResult<u32> {
    let native_slot = read_native_slot(state)?;
    let primaries = primary_profession_skill_lines(state);
    let compatible = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        is_cursor_compatible(&sim, native_slot, primaries)
    };
    state.push(Val::Bool(compatible));
    Ok(1)
}

/// Range-check, then decrement before truncation; public 0 wraps to all-ones.
fn read_native_slot(state: &mut LuaState) -> LuaResult<u32> {
    let value = crate::lua_bridge::stack_val(state, 1);
    match rilua::table_security::unwrap_secret(state, value)? {
        Val::Num(slot) if (0.0..=MAX_SLOT_ARGUMENT).contains(&slot) => {
            let decremented = (slot - 1.0).trunc() as i64;
            Ok(decremented as u32)
        }
        _ => Err(rilua::runtime_error(
            "C_PaperDollInfo.CanCursorCanGoInSlot: expected slotIndex",
        )),
    }
}

fn is_cursor_compatible(sim: &SimState, native_slot: u32, primaries: [Option<i32>; 2]) -> bool {
    let Some((item_id, origin)) = cursor_owned_item(sim) else {
        return false;
    };
    let Some(item) = item_spec_input(item_id).filter(|_| sim.player.present) else {
        return false;
    };
    let marker = sim.item_equip.offhand_two_hand_marker;
    let slot_fits = !is_excluded_slot(native_slot)
        && !has_child_in_slot(sim, item_id, native_slot)
        && matches_inventory_type(&item, native_slot, marker);
    let contents_allowed =
        is_container_exempt(native_slot) || !holds_contents(sim, origin, item_id);
    let profession_allowed =
        item.class != PROFESSION_CLASS || is_profession_slot(&item, native_slot, primaries);
    slot_fits && contents_allowed && profession_allowed
}

/// Only cursor mode 1 supplies an owned item.
pub(super) fn cursor_owned_item(sim: &SimState) -> Option<(u32, CursorItemOrigin)> {
    let cursor = sim.cursor_item.as_ref()?;
    if native_mode(cursor) != Some(MODE_ITEM) {
        return None;
    }
    match cursor {
        CursorInfo::Item {
            item_id, origin, ..
        } => Some((*item_id, *origin)),
        _ => None,
    }
}

fn is_excluded_slot(native_slot: u32) -> bool {
    matches!(native_slot, 17 | 86..=87 | 90..=91 | 93..=95 | 97..=99)
}

fn is_container_exempt(native_slot: u32) -> bool {
    matches!(native_slot, 30..=34 | 63..=68 | 100..=104)
}

/// Any ItemChildEquipment row naming this native slot rejects it.
fn has_child_in_slot(sim: &SimState, item_id: u32, native_slot: u32) -> bool {
    let names_slot = |slot_index: i32| i64::from(slot_index) == i64::from(native_slot);
    sim.item_children
        .get(&item_id)
        .is_some_and(|rows| rows.iter().any(|row| names_slot(row.slot_index)))
}

fn matches_inventory_type(item: &ItemSpecInput, native_slot: u32, marker: bool) -> bool {
    if native_slot >= FIRST_UNMASKED_SLOT {
        return true;
    }
    let is_reagent_bag = item.inventory_type == BAG_TYPE && item.subclass == REAGENT_BAG_SUBCLASS;
    if native_slot == REAGENT_BAG_SLOT || is_reagent_bag {
        return native_slot == REAGENT_BAG_SLOT && is_reagent_bag;
    }
    let two_hand_offhand = item.inventory_type == TWO_HAND_TYPE && native_slot == OFFHAND_SLOT;
    if two_hand_offhand && !allows_two_hand_offhand(item, marker) {
        return false;
    }
    // Best guess: types beyond the recovered rows 0–30 fit no slot below 35 (UNKNOWN).
    let mask = INVENTORY_TYPE_MASKS
        .get(usize::from(item.inventory_type))
        .copied()
        .unwrap_or(0);
    mask & (1_u64 << native_slot) != 0
}

fn allows_two_hand_offhand(item: &ItemSpecInput, marker: bool) -> bool {
    let subclass = item.subclass & SUBCLASS_LOW_BITS;
    marker && item.class == WEAPON_CLASS && OFFHAND_TWO_HAND_SUBCLASSES.contains(&subclass)
}

/// A container's contents are modeled only for an equipped bag.
fn holds_contents(sim: &SimState, origin: CursorItemOrigin, item_id: u32) -> bool {
    let CursorItemOrigin::Equipped { slot } = origin else {
        return false;
    };
    let mut bags = sim
        .bag_info
        .iter()
        .filter(|(_, bag)| bag.inventory_slot == Some(slot) && bag.item_id == Some(item_id));
    bags.any(|(&bag_id, _)| sim.bag_items.keys().any(|&(bag, _)| bag == bag_id))
}

fn is_profession_slot(item: &ItemSpecInput, native_slot: u32, primaries: [Option<i32>; 2]) -> bool {
    profession_key(item.subclass)
        .and_then(|key| allowed_profession_slots(key, primaries))
        .is_some_and(|slots| slots.contains(&native_slot))
}

/// Profession field 2 (INFERRED `ProfessionEnumValue`) of the selected row.
fn profession_key(subclass: u8) -> Option<u32> {
    if subclass > LAST_ROW_SUBCLASS {
        return Some(UNLISTED_PROFESSION_KEY);
    }
    let row_id = u32::from(subclass) + 1;
    let row = PROFESSION_ROWS.iter().find(|row| row.0 == row_id)?;
    Some(row.2)
}

/// Player-specific native slot list; `None` is the empty list.
fn allowed_profession_slots(key: u32, primaries: [Option<i32>; 2]) -> Option<RangeInclusive<u32>> {
    match key {
        FIXED_KEY_25_26 => Some(25..=26),
        FIXED_KEY_27_29 => Some(27..=29),
        _ => {
            let row = PROFESSION_ROWS.iter().find(|row| row.2 == key)?;
            let skill_line = i32::try_from(row.1).ok().filter(|line| *line > 0)?;
            let [first, second] = primaries;
            if first == Some(skill_line) {
                Some(19..=21)
            } else if second == Some(skill_line) {
                Some(22..=24)
            } else {
                None
            }
        }
    }
}
