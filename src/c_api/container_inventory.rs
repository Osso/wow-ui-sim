//! Shared bag/cursor/equipment transfers for C_Container and legacy globals.
//!
//! Retains the existing cursor representation: item ID, count, and origin only.
//! Hyperlinks, enchants, and gems do not survive cursor transfer in this model.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::{BagItem, CursorInfo, CursorItemOrigin, EquippedItem, SimState};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let container = super::ensure_namespace(state, "C_Container")?;
    table_set_rust_fn_static(
        state,
        container,
        "PickupContainerItem",
        pickup_container_item,
    )
}

fn slot_argument(state: &LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(value) => Some(value as i32),
        _ => None,
    }
}

pub(crate) fn pickup_container_item(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(bag), Some(slot)) = (slot_argument(state, 1), slot_argument(state, 2)) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    let held = bag_item_on_cursor(&sim);
    let dropping = held.is_some();
    let displaced = match held {
        Some(item) => sim.bag_items.insert((bag, slot), item),
        None => sim.bag_items.remove(&(bag, slot)),
    };
    if dropping || displaced.is_some() {
        sim.cursor_item = displaced.map(|item| CursorInfo::Item {
            item_id: item.item_id,
            stack_count: item.stack_count,
            origin: CursorItemOrigin::Bag { bag, slot },
        });
    }
    Ok(0)
}

fn bag_item_on_cursor(sim: &SimState) -> Option<BagItem> {
    match &sim.cursor_item {
        Some(CursorInfo::Item {
            item_id,
            stack_count,
            ..
        }) => Some(BagItem {
            item_id: *item_id,
            stack_count: *stack_count,
            hyperlink: None,
        }),
        _ => None,
    }
}

pub(crate) fn pickup_inventory_item(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = slot_argument(state, 1) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if matches!(sim.cursor_item, Some(CursorInfo::Item { .. })) {
        equip_held_item(&mut sim, slot);
    } else if let Some(item) = sim.player.equipped_items.remove(&slot) {
        sim.cursor_item = Some(equipped_cursor(item, slot));
    }
    Ok(0)
}

pub(crate) fn equip_cursor_item(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = slot_argument(state, 1) else {
        return Ok(0);
    };
    equip_held_item(&mut borrow_state_mut(state)?, slot);
    Ok(0)
}

// Existing EquipCursorItem swap operation, also used by PickupInventoryItem
// when a cursor item is held (Camelot PaperDollFrame's auto-equip path).
fn equip_held_item(sim: &mut SimState, slot: i32) {
    let Some(CursorInfo::Item { item_id, .. }) = &sim.cursor_item else {
        return;
    };
    let incoming = EquippedItem {
        item_id: *item_id,
        enchant_id: 0,
        gem_ids: [0; 3],
    };
    let displaced = sim.player.equipped_items.insert(slot, incoming);
    sim.cursor_item = displaced.map(|item| equipped_cursor(item, slot));
}

fn equipped_cursor(item: EquippedItem, slot: i32) -> CursorInfo {
    CursorInfo::Item {
        item_id: item.item_id,
        stack_count: 1,
        origin: CursorItemOrigin::Equipped { slot },
    }
}
