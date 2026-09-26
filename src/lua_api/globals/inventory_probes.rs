//! Inventory / interaction probe globals.
//!
//! Migrates 6 entries off `GLOBAL_FALSE_STUBS` onto real Rust impls:
//!
//! - `IsInventoryItemLocked(slot)` — cursor is carrying that equipped slot.
//! - `IsEquippableItem(itemId)`    — `equippable_items.contains(id)`.
//! - `IsConsumableItem(itemId)`    — `consumable_items.contains(id)`.
//! - `CanLootUnit(unit)`           — `unit is dead AND is_enemy`.
//! - `CanMerchant()`               — `SimState.merchant_frame_open`.
//! - `CanInspect(unit, showError?)` — unit resolves to a player-like entity.
//! - `NotifyInspect(unit)`         — queue `INSPECT_READY` for inspectable
//!                                   units.
//! - `GetInventoryItemID/Link/Texture/Quality/Count` — player equipment
//!   queries plus bag-slot texture fallbacks.

use super::missing_surface::item_link_for_id;
use crate::c_api::item_spell::{c_item_is_consumable_item, c_item_is_equippable_item};
use crate::event::{Event, EventArg};
use crate::items;
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state_types::CursorInfo;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const FIRST_EQUIPMENT_SLOT_ID: i32 = 1;
const LAST_PROFESSION_SLOT_ID: i32 = 28;

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

/// `IsInventoryItemLocked(slot)` — true when the cursor is carrying the
/// item from that equipment slot (i.e. the player is dragging it).
fn is_inventory_item_locked(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_i32(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let locked = {
        let st = borrow_state(state)?;
        matches!(
            &st.cursor_item,
            Some(CursorInfo::Item {
                origin: crate::lua_api::state_types::CursorItemOrigin::Equipped { slot: s },
                ..
            }) if *s == slot,
        )
    };
    state.push(Val::Bool(locked));
    Ok(1)
}

/// `CanLootUnit(unit)` — true when the unit is dead and is an enemy.
/// Sim only models combat-relevant loot windows on current_target.
fn can_loot_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let can = {
        let st = borrow_state(state)?;
        match unit.as_str() {
            "target" => st
                .current_target
                .as_ref()
                .is_some_and(|t| t.health <= 0 && t.is_enemy),
            "focus" => st
                .current_focus
                .as_ref()
                .is_some_and(|t| t.health <= 0 && t.is_enemy),
            _ => false,
        }
    };
    state.push(Val::Bool(can));
    Ok(1)
}

/// `CanMerchant()` — true when a merchant frame is open.
fn can_merchant(state: &mut LuaState) -> LuaResult<u32> {
    let b = borrow_state(state)?.merchant_frame_open;
    state.push(Val::Bool(b));
    Ok(1)
}

/// `CanInspect(unit, showError?)` — true for player-like units (player,
/// party members, friendly targets that are players).
fn can_inspect(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let can = {
        let st = borrow_state(state)?;
        can_inspect_unit(&st, &unit)
    };
    state.push(Val::Bool(can));
    Ok(1)
}

fn notify_inspect(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let guid = {
        let st = borrow_state(state)?;
        if !can_inspect_unit(&st, &unit) {
            return Ok(0);
        }
        inspect_guid_for_unit(&st, &unit)
    };
    borrow_state_mut(state)?.events.push(Event {
        name: "INSPECT_READY".to_string(),
        args: vec![EventArg::String(guid)],
    });
    Ok(0)
}

fn can_inspect_unit(st: &SimState, unit: &str) -> bool {
    match unit {
        "player" | "pet" | "vehicle" => true,
        "target" => st
            .current_target
            .as_ref()
            .is_some_and(|target| target.is_player),
        "focus" => st
            .current_focus
            .as_ref()
            .is_some_and(|target| target.is_player),
        other => crate::lua_api::globals::unit_api::parse_party_index(other)
            .is_some_and(|idx| st.party_group_active && idx < st.party_members.len()),
    }
}

fn inspect_guid_for_unit(st: &SimState, unit: &str) -> String {
    match unit {
        "player" | "pet" | "vehicle" => "Player-0000-00000001".to_string(),
        "target" => st
            .current_target
            .as_ref()
            .map(|target| target.guid.clone())
            .unwrap_or_else(|| "Creature-0000-00000000".to_string()),
        "focus" => st
            .current_focus
            .as_ref()
            .map(|focus| focus.guid.clone())
            .unwrap_or_else(|| "Creature-0000-00000000".to_string()),
        other => crate::lua_api::globals::unit_api::parse_party_index(other)
            .map(|idx| format!("Player-0000-000000{:02}", idx + 2))
            .unwrap_or_else(|| "Creature-0000-00000000".to_string()),
    }
}

fn player_equipped_item_id(state: &mut LuaState, unit: &str, slot: i32) -> Option<u32> {
    let is_equipment_slot = (FIRST_EQUIPMENT_SLOT_ID..=LAST_PROFESSION_SLOT_ID).contains(&slot);
    if unit != "player" || !is_equipment_slot {
        return None;
    }
    borrow_state(state)
        .ok()?
        .player
        .equipped_items
        .get(&slot)
        .map(|item| item.item_id)
}

fn fallback_icon_for_slot(slot: i32) -> f64 {
    match slot {
        2 => 133311.0,       // Necklace
        11 | 12 => 133358.0, // Ring
        13 | 14 => 338783.0, // Trinket
        _ => 134400.0,       // Question mark fallback
    }
}

fn inventory_item_texture_for_slot(slot: i32, item_id: u32) -> f64 {
    items::get_item(item_id)
        .map(|item| {
            if item.icon_file_data_id == 0 {
                fallback_icon_for_slot(slot)
            } else {
                item.icon_file_data_id as f64
            }
        })
        .unwrap_or_else(|| fallback_icon_for_slot(slot))
}

fn is_bag_inventory_slot(slot: i32) -> bool {
    matches!(slot, 20..=25)
}

fn get_inventory_item_id(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let Some(slot) = stack_i32(state, 2) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    match player_equipped_item_id(state, &unit, slot) {
        Some(item_id) => state.push(Val::Num(item_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_inventory_item_texture(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let Some(slot) = stack_i32(state, 2) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    if unit != "player" {
        state.push(Val::Nil);
        return Ok(1);
    }
    if let Some(item_id) = player_equipped_item_id(state, &unit, slot) {
        state.push(Val::Num(inventory_item_texture_for_slot(slot, item_id)));
        return Ok(1);
    }
    if is_bag_inventory_slot(slot) {
        let texture =
            crate::lua_api::methods::create_string(state, "Interface\\Icons\\INV_Misc_Bag_08");
        state.push(texture);
        return Ok(1);
    }
    state.push(Val::Nil);
    Ok(1)
}

fn get_inventory_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let Some(slot) = stack_i32(state, 2) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some(item_id) = player_equipped_item_id(state, &unit, slot) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let captured = borrow_state(state)?
        .bag_info
        .values()
        .find(|bag| bag.inventory_slot == Some(slot) && bag.item_id == Some(item_id))
        .and_then(|bag| bag.hyperlink.clone());
    match captured.or_else(|| item_link_for_id(item_id)) {
        Some(link) => {
            let link = crate::lua_api::methods::create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_inventory_item_broken(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = Option::<String>::from_stack(state, 1)?;
    let _slot = stack_i32(state, 2);
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_inventory_item_durability(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_i32(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let has_equipped_item = player_equipped_item_id(state, "player", slot).is_some();
    if !has_equipped_item {
        state.push(Val::Nil);
        return Ok(1);
    }

    state.push(Val::Num(100.0));
    state.push(Val::Num(100.0));
    Ok(2)
}

fn get_inventory_item_equipped_unusable(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = Option::<String>::from_stack(state, 1)?;
    let _slot = stack_i32(state, 2);
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_inventory_item_quality(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let Some(slot) = stack_i32(state, 2) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some(item_id) = player_equipped_item_id(state, &unit, slot) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let quality = items::get_item(item_id)
        .map(|item| item.quality as f64)
        .unwrap_or(0.0);
    state.push(Val::Num(quality));
    Ok(1)
}

fn get_inventory_item_count(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let Some(slot) = stack_i32(state, 2) else {
        state.push(Val::Num(0.0));
        return Ok(1);
    };
    let count = player_equipped_item_id(state, &unit, slot)
        .map(|_| 1.0)
        .unwrap_or(0.0);
    state.push(Val::Num(count));
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "IsInventoryItemLocked", is_inventory_item_locked)?;
    LuaApiMut::register_function(lua, "IsEquippableItem", c_item_is_equippable_item)?;
    LuaApiMut::register_function(lua, "IsConsumableItem", c_item_is_consumable_item)?;
    LuaApiMut::register_function(lua, "CanLootUnit", can_loot_unit)?;
    LuaApiMut::register_function(lua, "CanMerchant", can_merchant)?;
    LuaApiMut::register_function(lua, "CanInspect", can_inspect)?;
    LuaApiMut::register_function(lua, "NotifyInspect", notify_inspect)?;
    LuaApiMut::register_function(lua, "GetInventoryItemID", get_inventory_item_id)?;
    LuaApiMut::register_function(lua, "GetInventoryItemTexture", get_inventory_item_texture)?;
    LuaApiMut::register_function(lua, "GetInventoryItemLink", get_inventory_item_link)?;
    LuaApiMut::register_function(lua, "GetInventoryItemBroken", get_inventory_item_broken)?;
    LuaApiMut::register_function(
        lua,
        "GetInventoryItemDurability",
        get_inventory_item_durability,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetInventoryItemEquippedUnusable",
        get_inventory_item_equipped_unusable,
    )?;
    LuaApiMut::register_function(lua, "GetInventoryItemQuality", get_inventory_item_quality)?;
    LuaApiMut::register_function(lua, "GetInventoryItemCount", get_inventory_item_count)?;
    LuaApiMut::register_function(
        lua,
        "GetInventoryItemDurability",
        get_inventory_item_durability,
    )?;
    Ok(())
}
