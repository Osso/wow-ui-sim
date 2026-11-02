//! Shared bag/cursor/equipment transfers for C_Container and legacy globals.
//!
//! Hyperlinks, enchants, and gems do not survive cursor transfer in this model.

use crate::c_api::container_actions::ContainerUseRequest;
use crate::c_api::item_upgrade_eligibility::{self, ItemSlot};
use crate::lua_api::globals::real::cursor_model;
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
    )?;
    table_set_rust_fn_static(state, container, "SplitContainerItem", split_container_item)?;
    table_set_rust_fn_static(state, container, "UseContainerItem", use_container_item)
}

// Simulator-authored validation: finite integer arguments only; no coercion.
fn read_integer_argument(state: &LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(value) => {
            let in_range = value >= i32::MIN as f64 && value <= i32::MAX as f64;
            (in_range && value.fract() == 0.0).then_some(value as i32)
        }
        _ => None,
    }
}

pub(crate) fn split_container_item(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(bag), Some(slot), Some(amount)) = (
        read_integer_argument(state, 1),
        read_integer_argument(state, 2),
        read_integer_argument(state, 3),
    ) else {
        return Ok(0);
    };
    let cursor = {
        let mut sim = borrow_state_mut(state)?;
        if sim.cursor_item.is_some() {
            return Ok(0);
        }
        let Some(item) = sim.bag_items.get_mut(&(bag, slot)) else {
            return Ok(0);
        };
        if amount <= 0 || amount >= item.stack_count {
            return Ok(0);
        }
        item.stack_count -= amount;
        let cursor = CursorInfo::Item {
            owned_identity: None,
            item_id: item.item_id,
            stack_count: amount,
            origin: CursorItemOrigin::Bag { bag, slot },
            applied_transmog: item.applied_transmog,
        };
        item_upgrade_eligibility::discard_cursor(&mut sim);
        cursor
    };
    cursor_model::place(state, cursor)?;
    Ok(0)
}

// Accepted request is the only modeled use effect. No native eligibility,
// consumption, cast, cooldown, bank transfer, or event lifecycle is inferred.
pub(crate) fn use_container_item(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(bag), Some(slot)) = (
        read_integer_argument(state, 1),
        read_integer_argument(state, 2),
    ) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    let Some((item_id, count)) = sim.get_bag_item(bag, slot) else {
        return Ok(0);
    };
    if count <= 0 || !sim.container_actions.usable_items.contains(&item_id) {
        return Ok(0);
    }
    sim.container_actions
        .use_requests
        .push(ContainerUseRequest { bag, slot, item_id });
    Ok(0)
}

fn read_slot_argument(state: &LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(value) => Some(value as i32),
        _ => None,
    }
}

pub(crate) fn pickup_container_item(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(bag), Some(slot)) = (read_slot_argument(state, 1), read_slot_argument(state, 2))
    else {
        return Ok(0);
    };
    let (dropping, displaced) = {
        let mut sim = borrow_state_mut(state)?;
        let held = read_cursor_bag_item(&sim);
        let dropping = held.is_some();
        item_upgrade_eligibility::swap_with_cursor(&mut sim, ItemSlot::Bag(bag, slot));
        let displaced = match held {
            Some(item) => sim.bag_items.insert((bag, slot), item),
            None => sim.bag_items.remove(&(bag, slot)),
        };
        (dropping, displaced)
    };
    if dropping || displaced.is_some() {
        let next = displaced.map(|item| CursorInfo::Item {
            owned_identity: item.owned_identity,
            item_id: item.item_id,
            stack_count: item.stack_count,
            origin: CursorItemOrigin::Bag { bag, slot },
            applied_transmog: item.applied_transmog,
        });
        apply_cursor(state, next)?;
    }
    Ok(0)
}

/// Applies a transfer's resulting cursor through the shared clear/place model.
fn apply_cursor(state: &mut LuaState, next: Option<CursorInfo>) -> LuaResult<()> {
    match next {
        Some(cursor) => cursor_model::place(state, cursor),
        None => cursor_model::clear(state),
    }
}

fn read_cursor_bag_item(sim: &SimState) -> Option<BagItem> {
    match &sim.cursor_item {
        Some(CursorInfo::Item {
            item_id,
            stack_count,
            applied_transmog,
            owned_identity,
            ..
        }) => Some(BagItem {
            item_id: *item_id,
            stack_count: *stack_count,
            hyperlink: None,
            durability: None,
            applied_transmog: *applied_transmog,
            owned_identity: *owned_identity,
        }),
        _ => None,
    }
}

pub(crate) fn pickup_inventory_item(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = read_slot_argument(state, 1) else {
        return Ok(0);
    };
    let next = {
        let mut sim = borrow_state_mut(state)?;
        if matches!(sim.cursor_item, Some(CursorInfo::Item { .. })) {
            equip_held_item(&mut sim, slot)
        } else if sim.player.equipped_items.contains_key(&slot) {
            item_upgrade_eligibility::swap_with_cursor(&mut sim, ItemSlot::Equipment(slot));
            let item = sim
                .player
                .equipped_items
                .remove(&slot)
                .expect("checked equipment");
            Some(Some(create_equipped_cursor(item, slot)))
        } else {
            None
        }
    };
    if let Some(next) = next {
        apply_cursor(state, next)?;
    }
    Ok(0)
}

pub(crate) fn equip_cursor_item(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = read_slot_argument(state, 1) else {
        return Ok(0);
    };
    let next = equip_held_item(&mut *borrow_state_mut(state)?, slot);
    if let Some(next) = next {
        apply_cursor(state, next)?;
    }
    Ok(0)
}

// Existing EquipCursorItem swap operation, also used by PickupInventoryItem
// when a cursor item is held (Camelot PaperDollFrame's auto-equip path).
// Returns the resulting cursor, or None when no item is held.
fn equip_held_item(sim: &mut SimState, slot: i32) -> Option<Option<CursorInfo>> {
    let Some(CursorInfo::Item {
        item_id,
        applied_transmog,
        owned_identity,
        ..
    }) = &sim.cursor_item
    else {
        return None;
    };
    let incoming = EquippedItem {
        item_id: *item_id,
        enchant_id: 0,
        gem_ids: [0; 3],
        applied_transmog: *applied_transmog,
        owned_identity: *owned_identity,
    };
    item_upgrade_eligibility::swap_with_cursor(sim, ItemSlot::Equipment(slot));
    let displaced = sim.player.equipped_items.insert(slot, incoming);
    Some(displaced.map(|item| create_equipped_cursor(item, slot)))
}

fn create_equipped_cursor(item: EquippedItem, slot: i32) -> CursorInfo {
    CursorInfo::Item {
        owned_identity: item.owned_identity,
        item_id: item.item_id,
        stack_count: 1,
        origin: CursorItemOrigin::Equipped { slot },
        applied_transmog: item.applied_transmog,
    }
}
