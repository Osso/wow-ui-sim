//! `C_AzeriteItem` Heart-of-Azeroth surface consumed by
//! `Blizzard_ActionBar/Mainline/AzeriteBar.lua`.
//!
//! State source: `state.azerite_item: Option<AzeriteItemState>`. `None` keeps
//! `FindActiveAzeriteItem` returning nil so `AzeriteBarMixin:Update`
//! short-circuits without touching the bar.
//!
//! `FindActiveAzeriteItem` returns a plain Lua table populated with
//! `bagID`/`slotIndex`/`equipmentSlotIndex` matching the shape produced by
//! Blizzard's `ItemLocationMixin`. The simulator only models one Heart of
//! Azeroth, so the location parameter handed back to the other
//! `C_AzeriteItem.*` getters is treated as opaque — the getters just read
//! `state.azerite_item` directly.
//!
//! `IsUnlimitedLevelingUnlocked` is registered without a location parameter to
//! match the actual `AzeriteBar.lua:20` call site (which passes no arguments
//! despite the signature in the documentation).

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_table_with_fields, table_get};
use crate::lua_api::state::AzeriteItemState;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_azerite_item_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_AzeriteItem")?;
    table_set_rust_fn_static(state, ns, "FindActiveAzeriteItem", find_active_azerite_item)?;
    table_set_rust_fn_static(state, ns, "GetAzeriteItemXPInfo", get_azerite_item_xp_info)?;
    table_set_rust_fn_static(state, ns, "GetPowerLevel", get_power_level)?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetUnlimitedPowerLevel",
        get_unlimited_power_level,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsUnlimitedLevelingUnlocked",
        is_unlimited_leveling_unlocked,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsAzeriteItemAtMaxLevel",
        is_azerite_item_at_max_level,
    )?;
    table_set_rust_fn_static(state, ns, "IsAzeriteItemEnabled", is_azerite_item_enabled)?;
    if matches!(ACTIVE, ClientProfile::Retail) {
        table_set_rust_fn_static(state, ns, "IsAzeriteItem", is_azerite_item)?;
    }
    Ok(())
}

fn is_azerite_item(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(value, Val::Table(_)) {
        return Err(rilua::runtime_error(
            "C_AzeriteItem.IsAzeriteItem: itemLocation must be a table",
        ));
    }
    let bag = location_field(state, value, "bagID")?;
    let slot = location_field(state, value, "slotIndex")?;
    let equipment = location_field(state, value, "equipmentSlotIndex")?;
    let matches = borrow_state(state)?
        .azerite_item
        .as_ref()
        .is_some_and(|item| {
            let location = &item.item_location;
            (equipment.is_some() || bag.is_some() && slot.is_some())
                && bag == location.bag_id
                && slot == location.slot_index
                && equipment == location.equipment_slot_index
        });
    state.push(Val::Bool(matches));
    Ok(1)
}

fn location_field(state: &mut LuaState, value: Val, key: &str) -> LuaResult<Option<i32>> {
    let field = table_get(state, value, key);
    let field = rilua::table_security::unwrap_secret(state, field)?;
    Ok(match field {
        Val::Num(n) => Some(n as i32),
        _ => None,
    })
}

fn find_active_azerite_item(state: &mut LuaState) -> LuaResult<u32> {
    let Some(item) = borrow_state(state)?.azerite_item.clone() else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let bag_id = optional_num_field(item.item_location.bag_id);
    let slot_index = optional_num_field(item.item_location.slot_index);
    let equipment_slot_index = optional_num_field(item.item_location.equipment_slot_index);
    let location = create_table_with_fields(
        state,
        &[
            ("bagID", bag_id),
            ("slotIndex", slot_index),
            ("equipmentSlotIndex", equipment_slot_index),
        ],
    );
    state.push(location);
    Ok(1)
}

fn get_azerite_item_xp_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(item) = current_item(state)? else {
        return Ok(0);
    };
    state.push(Val::Num(item.current_xp as f64));
    state.push(Val::Num(item.max_xp as f64));
    Ok(2)
}

fn get_power_level(state: &mut LuaState) -> LuaResult<u32> {
    push_power_level_field(state, |item| item.power_level)
}

fn get_unlimited_power_level(state: &mut LuaState) -> LuaResult<u32> {
    push_power_level_field(state, |item| item.unlimited_power_level)
}

fn is_unlimited_leveling_unlocked(state: &mut LuaState) -> LuaResult<u32> {
    push_bool_field(state, |item| item.unlimited_unlocked)
}

fn is_azerite_item_at_max_level(state: &mut LuaState) -> LuaResult<u32> {
    push_bool_field(state, |item| item.at_max_level)
}

fn is_azerite_item_enabled(state: &mut LuaState) -> LuaResult<u32> {
    push_bool_field(state, |item| item.enabled)
}

fn current_item(state: &mut LuaState) -> LuaResult<Option<AzeriteItemState>> {
    Ok(borrow_state(state)?.azerite_item.clone())
}

fn push_power_level_field(
    state: &mut LuaState,
    read: impl FnOnce(&AzeriteItemState) -> i32,
) -> LuaResult<u32> {
    let level = current_item(state)?.as_ref().map_or(0, read);
    state.push(Val::Num(level as f64));
    Ok(1)
}

fn push_bool_field(
    state: &mut LuaState,
    read: impl FnOnce(&AzeriteItemState) -> bool,
) -> LuaResult<u32> {
    let value = current_item(state)?.as_ref().is_some_and(read);
    state.push(Val::Bool(value));
    Ok(1)
}

fn optional_num_field(value: Option<i32>) -> Val {
    match value {
        Some(n) => Val::Num(n as f64),
        None => Val::Nil,
    }
}
