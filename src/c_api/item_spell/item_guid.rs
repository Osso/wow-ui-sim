//! `C_Item` GUID queries and script item locks over the bag-item model.
//!
//! The simulator issues bag item GUIDs as `Item-<bag>-<slot>-<itemID>`
//! (`C_Item.GetItemGUID`). A GUID resolves only while that bag slot still
//! holds the same item. INFERRED: R88g9 (native evidence pending): equipped
//! and bank-tab GUID resolution is not modeled; such GUIDs resolve to nothing.

use super::item_equip::{dispatch_lock_transition, ensure_identity, set_lock_reason};
use crate::c_api::item_location::{ItemSlot, read_numeric_item_slot};
use crate::c_api::item_spell::{item_link_for_id, parse_item_guid};
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::state::CursorItemOrigin;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

/// Simulator-internal lock reason bit for `C_Item.LockItem`; not a native value.
const SCRIPT_LOCK_REASON: u32 = 4;

/// ItemDocumentation.lua:544/:749/:764/:1498 (Retail), :566/:771/:786/:1535 (Forever).
pub(super) fn register_guid_queries(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "GetItemIDByGUID", get_item_id_by_guid)?;
    table_set_rust_fn_static(state, table, "GetItemLinkByGUID", get_item_link_by_guid)?;
    table_set_rust_fn_static(state, table, "GetItemLocation", get_item_location)?;
    table_set_rust_fn_static(state, table, "IsItemGUIDInInventory", is_guid_in_inventory)
}

/// ItemDocumentation.lua:1621/:1631/:1683/:1693 (Retail). Only Retail models
/// item locks (`cursor_model::modeled`), so only Retail publishes these.
pub(super) fn register_lock_requests(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "LockItem", lock_item)?;
    table_set_rust_fn_static(state, table, "UnlockItem", unlock_item)?;
    table_set_rust_fn_static(state, table, "LockItemByGUID", lock_item_by_guid)?;
    table_set_rust_fn_static(state, table, "UnlockItemByGUID", unlock_item_by_guid)
}

/// The bag slot that currently holds the item a GUID names.
fn resolve_guid(sim: &SimState, guid: &str) -> Option<(i32, i32, u32)> {
    let (bag, slot, item_id) = parse_item_guid(guid)?;
    let held = sim.get_bag_item(bag, slot).map(|(held, _)| held);
    (held == Some(item_id)).then_some((bag, slot, item_id))
}

fn read_guid(state: &LuaState) -> LuaResult<String> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(value, Val::Str(_)) {
        return Err(runtime_error("Usage: expected itemGUID string"));
    }
    String::from_stack(state, 1)
}

fn resolved_guid(state: &LuaState) -> LuaResult<Option<(i32, i32, u32)>> {
    let guid = read_guid(state)?;
    Ok(resolve_guid(&*borrow_state(state)?, &guid))
}

fn get_item_id_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    let item = resolved_guid(state)?;
    state.push(item.map_or(Val::Nil, |(_, _, id)| Val::Num(f64::from(id))));
    Ok(1)
}

fn get_item_link_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    let link = resolved_guid(state)?.and_then(|(bag, slot, item_id)| {
        let sim = borrow_state(state).ok()?;
        let supplied = sim.bag_items.get(&(bag, slot))?.hyperlink.clone();
        supplied.or_else(|| item_link_for_id(item_id))
    });
    let value = link.map_or(Val::Nil, |link| create_string(state, &link));
    state.push(value);
    Ok(1)
}

fn is_guid_in_inventory(state: &mut LuaState) -> LuaResult<u32> {
    let found = resolved_guid(state)?.is_some();
    state.push(Val::Bool(found));
    Ok(1)
}

fn get_item_location(state: &mut LuaState) -> LuaResult<u32> {
    let Some((bag, slot, _)) = resolved_guid(state)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let origin = CursorItemOrigin::Bag { bag, slot };
    let location = crate::c_api::c_cursor::item_location_val(state, origin);
    state.push(location);
    Ok(1)
}

fn set_script_lock(state: &mut LuaState, slot: ItemSlot, locked: bool) -> LuaResult<u32> {
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        if slot.item_id(&sim).is_none() {
            return Ok(0);
        }
        let Some(identity) = ensure_identity(&mut sim, slot) else {
            return Ok(0);
        };
        set_lock_reason(&mut sim, slot, identity, SCRIPT_LOCK_REASON, locked)
    };
    if changed {
        dispatch_lock_transition(state, slot, locked)?;
    }
    Ok(0)
}

fn location_slot(state: &mut LuaState, api: &str) -> LuaResult<ItemSlot> {
    let location = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    read_numeric_item_slot(state, location)
        .ok_or_else(|| runtime_error(format!("C_Item.{api}: expected item location")))
}

fn guid_slot(state: &mut LuaState) -> LuaResult<Option<ItemSlot>> {
    Ok(resolved_guid(state)?.map(|(bag, slot, _)| ItemSlot::Bag(bag, slot)))
}

fn lock_item(state: &mut LuaState) -> LuaResult<u32> {
    let slot = location_slot(state, "LockItem")?;
    set_script_lock(state, slot, true)
}

fn unlock_item(state: &mut LuaState) -> LuaResult<u32> {
    let slot = location_slot(state, "UnlockItem")?;
    set_script_lock(state, slot, false)
}

fn lock_item_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    match guid_slot(state)? {
        Some(slot) => set_script_lock(state, slot, true),
        None => Ok(0),
    }
}

fn unlock_item_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    match guid_slot(state)? {
        Some(slot) => set_script_lock(state, slot, false),
        None => Ok(0),
    }
}
