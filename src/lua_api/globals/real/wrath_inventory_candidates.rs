//! Wrath inventory candidates for the original paper-doll flyout.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::items;
use crate::lua_api::methods::{borrow_state, table_set_num};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

// Wrath FrameXML/Constants.lua defines the location flags and bag shift.
const PLAYER_LOCATION: i32 = 0x0010_0000;
const BAG_LOCATION: i32 = 0x0020_0000;
const BAG_SHIFT: u32 = 8;
const FINGER_INVENTORY_TYPE: u8 = 11;

fn is_ring_candidate(sim: &SimState, item_id: u32) -> bool {
    sim.equippable_items.contains(&item_id)
        && items::get_item(item_id).is_some_and(|item| item.inventory_type == FINGER_INVENTORY_TYPE)
}

fn ring_candidates(sim: &SimState) -> Vec<(i32, u32)> {
    let equipped = sim
        .player
        .equipped_items
        .iter()
        .filter_map(|(&slot, item)| {
            (matches!(slot, 11 | 12) && is_ring_candidate(sim, item.item_id))
                .then_some((PLAYER_LOCATION + slot, item.item_id))
        });
    let carried = sim.bag_items.iter().filter_map(|(&(bag, slot), item)| {
        ((0..=4).contains(&bag) && is_ring_candidate(sim, item.item_id)).then_some((
            PLAYER_LOCATION + BAG_LOCATION + (bag << BAG_SHIFT) + slot,
            item.item_id,
        ))
    });
    equipped.chain(carried).collect()
}

fn get_inventory_items_for_slot(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(slot) = stack_val(state, 1) else {
        return Err(runtime_error(
            "GetInventoryItemsForSlot requires a numeric slot",
        ));
    };
    if !(1.0..=19.0).contains(&slot) {
        return Ok(0);
    }
    let Val::Table(table) = stack_val(state, 2) else {
        return Err(runtime_error(
            "GetInventoryItemsForSlot requires an item table",
        ));
    };
    let candidates = if slot == 11.0 || slot == 12.0 {
        let sim = borrow_state(state)?;
        ring_candidates(&sim)
    } else {
        Vec::new()
    };
    for (location, item_id) in candidates {
        table_set_num(
            state,
            table,
            f64::from(location),
            Val::Num(f64::from(item_id)),
        );
    }
    state.push(Val::Table(table));
    Ok(1)
}

fn get_inventory_item_gems(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let gems = borrow_state(state)?
        .player
        .equipped_items
        .get(&slot)
        .map(|item| item.gem_ids)
        .unwrap_or_default();
    Ok(push_gems(state, gems))
}

fn push_gems(state: &mut LuaState, gems: [u32; 3]) -> u32 {
    let count = gems.iter().rposition(|&id| id != 0).map_or(0, |i| i + 1);
    for &id in &gems[..count] {
        state.push(if id == 0 {
            Val::Nil
        } else {
            Val::Num(f64::from(id))
        });
    }
    count as u32
}

fn bag_location(state: &mut LuaState) -> LuaResult<(i32, i32)> {
    Ok((i32::from_stack(state, 1)?, i32::from_stack(state, 2)?))
}

fn parse_item_link_gems(link: &str) -> [u32; 3] {
    let mut gems = [0; 3];
    if let Some((_, item)) = link.split_once("item:") {
        for (gem, field) in gems.iter_mut().zip(item.split(':').skip(2)) {
            *gem = field.parse().unwrap_or(0);
        }
    }
    gems
}

fn get_container_item_gems(state: &mut LuaState) -> LuaResult<u32> {
    let location = bag_location(state)?;
    let gems = borrow_state(state)?
        .bag_items
        .get(&location)
        .and_then(|item| item.hyperlink.as_deref())
        .map(parse_item_link_gems)
        .unwrap_or_default();
    Ok(push_gems(state, gems))
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Wrath {
        LuaApiMut::register_function(
            lua,
            "GetInventoryItemsForSlot",
            get_inventory_items_for_slot,
        )?;
        LuaApiMut::register_function(lua, "GetInventoryItemGems", get_inventory_item_gems)?;
        LuaApiMut::register_function(
            lua,
            "GetContainerItemCooldown",
            crate::c_api::item_spell::c_container_get_container_item_cooldown,
        )?;
        LuaApiMut::register_function(
            lua,
            "GetContainerItemDurability",
            crate::c_api::item_spell::c_container_get_item_durability,
        )?;
        LuaApiMut::register_function(lua, "GetContainerItemGems", get_container_item_gems)?;
    }
    Ok(())
}
