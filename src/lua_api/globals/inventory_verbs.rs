//! Inventory pickup verbs that move items through `SimState.cursor_item`
//! and the existing `bag_items` / `equipped_items` maps.
//!
//! Migrates 7 entries off `GLOBAL_NIL_STUBS`:
//!
//! - `PickupContainerItem(bag, slot)` — bag → cursor
//! - `PickupInventoryItem(slot)`      — equip slot → cursor
//! - `PickupBagFromSlot(slot)`        — bag container slot → cursor
//!   (bag container slots are the same equipment slot range covered by
//!    `PickupInventoryItem`; this is a named alias for the bag slots 20-23)
//! - `PickupMerchantItem(index)`      — merchant row → cursor (synthesized)
//! - `PutItemInBackpack()`            — cursor item → backpack slot 1-16
//! - `PutItemInBag(slot)`             — cursor item → equipped bag slot
//! - `EquipCursorItem(slot)`          — cursor → equip slot
//! - `DeleteCursorItem()`             — clears cursor
//! - `PlaceAction(slot)`              — cursor spell/action → action bar slot
//!
//! Registered from `register_tail_globals` after `missing_surface` so the
//! Rust impls supersede any `stub_nil` entries that slipped through.

use crate::c_api::container_inventory::{
    equip_cursor_item, pickup_container_item, pickup_inventory_item,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, frame_ref, table_get,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state_types::{CursorInfo, CursorItemOrigin};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_u32(state: &mut LuaState, index: i32) -> Option<u32> {
    stack_i32(state, index).and_then(|n| u32::try_from(n).ok())
}

fn fire_actionbar_slot_changed(state: &mut LuaState) {
    fire_named_event_state(state, "ACTIONBAR_SLOT_CHANGED", &[Val::Num(0.0)]);
}

fn refresh_action_ui_buttons(state: &mut LuaState, slot: u32) {
    let button_ids = {
        let Ok(sim) = borrow_state(state) else {
            return;
        };
        sim.action_ui_buttons
            .iter()
            .filter_map(|(button_id, action)| (*action == slot).then_some(*button_id))
            .collect::<Vec<_>>()
    };

    for button_id in button_ids {
        let Ok(button) = frame_ref(state, button_id) else {
            continue;
        };
        let update_action = table_get(state, button, "UpdateAction");
        if matches!(update_action, Val::Function(_)) {
            let _ = call_function_state(state, update_action, &[button, Val::Bool(true)]);
        }
    }
}

fn action_spell_id(state: &mut LuaState, slot: u32) -> Option<u32> {
    borrow_state(state).ok()?.action_bars.get(&slot).copied()
}

fn action_outfit_id(state: &mut LuaState, slot: u32) -> Option<i64> {
    borrow_state(state).ok()?.action_outfits.get(&slot).copied()
}

fn place_cursor_item_in_backpack(state: &mut LuaState) -> LuaResult<u32> {
    let Some((item_id, stack_count)) = take_cursor_item(state) else {
        return Ok(0);
    };
    let Some(slot) = find_first_free_backpack_slot(state) else {
        return Ok(0);
    };

    store_cursor_item_in_backpack(state, slot, item_id, stack_count);
    fire_named_event_state(state, "CURSOR_CHANGED", &[]);
    state.push(Val::Bool(true));
    Ok(1)
}

fn take_cursor_item(state: &mut LuaState) -> Option<(u32, i32)> {
    let Ok(st) = borrow_state(state) else {
        return None;
    };
    let cursor = st.cursor_item.clone()?;
    let CursorInfo::Item {
        item_id,
        stack_count,
        ..
    } = cursor
    else {
        return None;
    };
    Some((item_id, stack_count))
}

fn find_first_free_backpack_slot(state: &mut LuaState) -> Option<i32> {
    let Ok(st) = borrow_state(state) else {
        return None;
    };
    (1..=16).find(|slot| !st.bag_items.contains_key(&(0, *slot)))
}

fn store_cursor_item_in_backpack(state: &mut LuaState, slot: i32, item_id: u32, stack_count: i32) {
    if let Ok(mut st) = borrow_state_mut(state) {
        st.bag_items.insert(
            (0, slot),
            crate::lua_api::state::BagItem {
                item_id,
                stack_count,
                hyperlink: None,
            },
        );
        st.cursor_item = None;
    }
}

/// `PickupBagFromSlot(slot)` — alias for `PickupInventoryItem` targeted at
/// the bag container slot range. Shares the same implementation.
fn pickup_bag_from_slot(state: &mut LuaState) -> LuaResult<u32> {
    pickup_inventory_item(state)
}

/// `PickupAction(slot [, ignoreRemoval])` — place the spell/action from the
/// action bar on the cursor. When `ignoreRemoval` is truthy, the source slot
/// stays populated.
fn pickup_action(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_u32(state, 1) else {
        return Ok(0);
    };
    let ignore_removal = matches!(stack_val(state, 2), Val::Bool(true));
    let Ok(mut st) = borrow_state_mut(state) else {
        return Ok(0);
    };
    let cursor = if let Some(&macro_index) = st.action_macros.get(&slot) {
        CursorInfo::Macro { macro_index }
    } else if let Some(&spell_id) = st.action_bars.get(&slot) {
        CursorInfo::Action { slot, spell_id }
    } else {
        return Ok(0);
    };
    if !ignore_removal {
        crate::c_api::action_macros::clear_slot(&mut st, slot);
    }
    st.cursor_item = Some(cursor);
    drop(st);
    if !ignore_removal {
        fire_actionbar_slot_changed(state);
        refresh_action_ui_buttons(state, slot);
    }
    fire_named_event_state(state, "CURSOR_CHANGED", &[]);
    Ok(0)
}

fn has_action(state: &mut LuaState) -> LuaResult<u32> {
    let has = stack_u32(state, 1).is_some_and(|slot| {
        action_spell_id(state, slot).is_some()
            || action_outfit_id(state, slot).is_some()
            || borrow_state(state).is_ok_and(|sim| sim.action_macros.contains_key(&slot))
    });
    state.push(Val::Bool(has));
    Ok(1)
}

fn get_action_texture(state: &mut LuaState) -> LuaResult<u32> {
    let texture = stack_u32(state, 1)
        .and_then(|slot| action_spell_id(state, slot))
        .and_then(|spell_id| {
            crate::spells::get_spell(spell_id).and_then(|spell| {
                crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id)
            })
        });
    match texture {
        Some(path) => {
            let path_val = create_string(state, path);
            state.push(path_val);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_action_info(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_u32(state, 1);
    if let Some(outfit_id) = slot.and_then(|slot| action_outfit_id(state, slot)) {
        let kind = create_string(state, "outfit");
        state.push(kind);
        state.push(Val::Num(outfit_id as f64));
        state.push(Val::Nil);
        return Ok(3);
    }
    let macro_id =
        slot.and_then(|slot| borrow_state(state).ok()?.action_macros.get(&slot).copied());
    if let Some(id) = macro_id {
        let kind = create_string(state, "macro");
        state.push(kind);
        state.push(Val::Num(id as f64));
        state.push(Val::Nil);
        return Ok(3);
    }
    if let Some(spell_id) = slot.and_then(|slot| action_spell_id(state, slot)) {
        let kind = create_string(state, "spell");
        state.push(kind);
        state.push(Val::Num(spell_id as f64));
        state.push(Val::Nil);
        return Ok(3);
    }
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(3)
}

/// `PickupMerchantItem(index)` — synthesize a merchant item on the cursor
/// with `item_id = 100_000 + index`. Silent no-op without an index.
fn pickup_merchant_item(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_u32(state, 1) else {
        return Ok(0);
    };
    let Ok(mut st) = borrow_state_mut(state) else {
        return Ok(0);
    };
    st.cursor_item = Some(CursorInfo::Item {
        item_id: 100_000 + index,
        stack_count: 1,
        origin: CursorItemOrigin::Merchant { index },
    });
    Ok(0)
}

/// `PutItemInBackpack()` — move a cursor item into the first free backpack
/// slot. Silent no-op for non-item cursors or when the backpack is full.
fn put_item_in_backpack(state: &mut LuaState) -> LuaResult<u32> {
    place_cursor_item_in_backpack(state)
}

/// `PutItemInBag(slot)` — move a cursor item into an equipped bag slot.
/// The sim only models the backpack, so non-backpack bags are a no-op.
fn put_item_in_bag(state: &mut LuaState) -> LuaResult<u32> {
    let Some(bag_slot) = stack_i32(state, 1) else {
        return Ok(0);
    };
    if bag_slot != 0 {
        return Ok(0);
    }
    place_cursor_item_in_backpack(state)
}

/// `DeleteCursorItem()` — clear the cursor. If the cursor was carrying a
/// bag item, it is NOT returned to the bag (deletion semantics).
fn delete_cursor_item(state: &mut LuaState) -> LuaResult<u32> {
    clear_cursor_payload(state)
}

/// `ClearCursor()` — clear any cursor payload and fire `CURSOR_CHANGED`.
fn clear_cursor(state: &mut LuaState) -> LuaResult<u32> {
    clear_cursor_payload(state)
}

fn clear_cursor_payload(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(mut st) = borrow_state_mut(state) else {
        return Ok(0);
    };
    st.cursor_item = None;
    drop(st);
    fire_named_event_state(state, "CURSOR_CHANGED", &[]);
    Ok(0)
}

/// `CursorHasItem()` — true when the cursor carries an item stack.
fn cursor_has_item(state: &mut LuaState) -> LuaResult<u32> {
    let has_item = borrow_state(state)?
        .cursor_item
        .as_ref()
        .is_some_and(|cursor| matches!(cursor, CursorInfo::Item { .. }));
    state.push(Val::Bool(has_item));
    Ok(1)
}

/// `GetCursorInfo()` — expose the cursor payload in WoW's coarse-grained
/// `(kind, id, ...)` shape. Only the spell/item cases used by the simulator
/// test surface are modeled.
fn get_cursor_info(state: &mut LuaState) -> LuaResult<u32> {
    let cursor = borrow_state(state)?.cursor_item.clone();
    let Some(cursor) = cursor else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_cursor_info(state, cursor)
}

fn push_cursor_info(state: &mut LuaState, cursor: CursorInfo) -> LuaResult<u32> {
    match cursor {
        CursorInfo::Action { spell_id, .. }
        | CursorInfo::Spell { spell_id }
        | CursorInfo::PetAction { spell_id, .. } => push_cursor_kind_id(state, "spell", spell_id),
        CursorInfo::Talent { talent_id, .. } => push_cursor_kind_id(state, "talent", talent_id),
        CursorInfo::Macro { macro_index } => push_cursor_kind_id(state, "macro", macro_index),
        CursorInfo::Item { item_id, .. } => push_cursor_kind_id(state, "item", item_id),
        CursorInfo::Money { copper } => push_cursor_kind_number(state, "money", copper as f64),
    }
}

fn push_cursor_kind_id(state: &mut LuaState, kind: &str, id: u32) -> LuaResult<u32> {
    push_cursor_kind_number(state, kind, id as f64)
}

fn push_cursor_kind_number(state: &mut LuaState, kind: &str, number: f64) -> LuaResult<u32> {
    let kind = create_string(state, kind);
    state.push(kind);
    state.push(Val::Num(number));
    Ok(2)
}

/// `PlaceAction(slot)` — if the cursor is carrying a spell/action, write
/// the spell id into `action_bars[slot]` and clear the cursor. Items are
/// not placeable on action bars; silent no-op in that case.
fn place_action(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_u32(state, 1) else {
        return Ok(0);
    };
    let Ok(mut st) = borrow_state_mut(state) else {
        return Ok(0);
    };
    let Some(cursor) = st.cursor_item.clone() else {
        return Ok(0);
    };
    match cursor {
        CursorInfo::Macro { macro_index } => {
            crate::c_api::action_macros::assign_macro(&mut st, slot, macro_index)?;
        }
        CursorInfo::Action { spell_id, .. }
        | CursorInfo::Spell { spell_id }
        | CursorInfo::PetAction { spell_id, .. } => {
            crate::c_api::action_macros::clear_slot(&mut st, slot);
            st.action_bars.insert(slot, spell_id);
        }
        CursorInfo::Talent { talent_id, .. } => {
            crate::c_api::action_macros::clear_slot(&mut st, slot);
            st.action_bars.insert(slot, talent_id);
        }
        CursorInfo::Item { .. } | CursorInfo::Money { .. } => return Ok(0),
    }
    st.cursor_item = None;
    drop(st);
    fire_actionbar_slot_changed(state);
    refresh_action_ui_buttons(state, slot);
    fire_named_event_state(state, "CURSOR_CHANGED", &[]);
    Ok(0)
}

/// `PickupPlayerMoney(amount)` — move `amount` copper from `player.money`
/// onto the cursor as a `Money` payload, replacing whatever was there. If
/// the player doesn't have enough money the call is a silent no-op (matches
/// the live client, which clamps below the available balance).
fn pickup_player_money(state: &mut LuaState) -> LuaResult<u32> {
    let Some(amount) = stack_i32(state, 1).and_then(|n| u64::try_from(n).ok()) else {
        return Ok(0);
    };
    if amount == 0 {
        return Ok(0);
    }
    let mut st = match borrow_state_mut(state) {
        Ok(s) => s,
        Err(_) => return Ok(0),
    };
    let available = u64::try_from(st.player.money.max(0)).unwrap_or(0);
    if amount > available {
        return Ok(0);
    }
    st.player.money -= amount as i64;
    st.cursor_item = Some(CursorInfo::Money { copper: amount });
    drop(st);
    fire_named_event_state(state, "CURSOR_CHANGED", &[]);
    fire_named_event_state(state, "PLAYER_MONEY", &[]);
    Ok(0)
}

/// `DropCursorMoney()` — return any money currently on the cursor back to
/// the player and clear the cursor. Other cursor payloads are left intact
/// (matches the live client, which only acts on money).
fn drop_cursor_money(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = match borrow_state_mut(state) {
        Ok(s) => s,
        Err(_) => return Ok(0),
    };
    let Some(CursorInfo::Money { copper }) = st.cursor_item.clone() else {
        return Ok(0);
    };
    st.player.money += copper as i64;
    st.cursor_item = None;
    drop(st);
    fire_named_event_state(state, "CURSOR_CHANGED", &[]);
    fire_named_event_state(state, "PLAYER_MONEY", &[]);
    Ok(0)
}

/// `GetCursorMoney()` — copper currently held on the cursor (0 when the
/// cursor is empty or carrying a non-money payload).
fn get_cursor_money(state: &mut LuaState) -> LuaResult<u32> {
    let copper = match borrow_state(state)?.cursor_item {
        Some(CursorInfo::Money { copper }) => copper,
        _ => 0,
    };
    state.push(Val::Num(copper as f64));
    Ok(1)
}

/// Install in the global table. Exposed for tests that want to bypass the
/// full `register_globals` chain.
pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "PickupContainerItem", pickup_container_item)?;
    LuaApiMut::register_function(lua, "PickupInventoryItem", pickup_inventory_item)?;
    LuaApiMut::register_function(lua, "PickupBagFromSlot", pickup_bag_from_slot)?;
    LuaApiMut::register_function(lua, "PickupAction", pickup_action)?;
    LuaApiMut::register_function(lua, "HasAction", has_action)?;
    LuaApiMut::register_function(lua, "GetActionTexture", get_action_texture)?;
    LuaApiMut::register_function(lua, "GetActionInfo", get_action_info)?;
    LuaApiMut::register_function(lua, "PickupMerchantItem", pickup_merchant_item)?;
    LuaApiMut::register_function(lua, "PutItemInBackpack", put_item_in_backpack)?;
    LuaApiMut::register_function(lua, "PutItemInBag", put_item_in_bag)?;
    LuaApiMut::register_function(lua, "EquipCursorItem", equip_cursor_item)?;
    LuaApiMut::register_function(lua, "DeleteCursorItem", delete_cursor_item)?;
    LuaApiMut::register_function(lua, "ClearCursor", clear_cursor)?;
    LuaApiMut::register_function(lua, "GetCursorInfo", get_cursor_info)?;
    LuaApiMut::register_function(lua, "CursorHasItem", cursor_has_item)?;
    LuaApiMut::register_function(lua, "PlaceAction", place_action)?;
    LuaApiMut::register_function(lua, "PickupPlayerMoney", pickup_player_money)?;
    LuaApiMut::register_function(lua, "DropCursorMoney", drop_cursor_money)?;
    LuaApiMut::register_function(lua, "GetCursorMoney", get_cursor_money)?;
    Ok(())
}
