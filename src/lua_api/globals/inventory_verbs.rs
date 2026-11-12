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
    equip_cursor_item, pickup_container_item, pickup_inventory_item, split_container_item,
    use_container_item,
};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::real::cursor_model;
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
    let Some(item) = take_cursor_item(state) else {
        return Ok(0);
    };
    let Some(slot) = find_first_free_backpack_slot(state) else {
        return Ok(0);
    };

    store_cursor_item_in_backpack(state, slot, item);
    cursor_model::clear(state)?;
    cursor_model::legacy_changed(state);
    state.push(Val::Bool(true));
    Ok(1)
}

fn take_cursor_item(state: &mut LuaState) -> Option<crate::lua_api::state::BagItem> {
    let Ok(st) = borrow_state(state) else {
        return None;
    };
    let cursor = st.cursor_item.clone()?;
    let CursorInfo::Item {
        item_id,
        stack_count,
        applied_transmog,
        owned_identity,
        ..
    } = cursor
    else {
        return None;
    };
    Some(crate::lua_api::state::BagItem {
        item_id,
        stack_count,
        hyperlink: None,
        durability: None,
        applied_transmog,
        owned_identity,
    })
}

fn find_first_free_backpack_slot(state: &mut LuaState) -> Option<i32> {
    let Ok(st) = borrow_state(state) else {
        return None;
    };
    (1..=st.bag_num_slots(0)).find(|slot| !st.bag_items.contains_key(&(0, *slot)))
}

fn store_cursor_item_in_backpack(
    state: &mut LuaState,
    slot: i32,
    item: crate::lua_api::state::BagItem,
) {
    if let Ok(mut st) = borrow_state_mut(state) {
        crate::c_api::item_upgrade_eligibility::swap_with_cursor(
            &mut st,
            crate::c_api::item_upgrade_eligibility::ItemSlot::Bag(0, slot),
        );
        st.bag_items.insert((0, slot), item);
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
    } else if let Some(&outfit_id) = st.action_outfits.get(&slot) {
        CursorInfo::TransmogOutfit { outfit_id }
    } else if let Some(&spell_id) = st.action_bars.get(&slot) {
        CursorInfo::Action { slot, spell_id }
    } else {
        return Ok(0);
    };
    if !ignore_removal {
        crate::c_api::action_macros::clear_slot(&mut st, slot);
    }
    drop(st);
    cursor_model::place(state, cursor)?;
    if !ignore_removal {
        fire_actionbar_slot_changed(state);
        refresh_action_ui_buttons(state, slot);
    }
    cursor_model::legacy_changed(state);
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
    let slot = stack_u32(state, 1);
    let texture = {
        let sim = borrow_state(state)?;
        slot.and_then(|slot| crate::c_api::c_action_bar::action_texture_path(&sim, slot))
    };
    match texture {
        Some(path) => {
            let path_val = create_string(state, &path);
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
    let cursor = CursorInfo::Item {
        owned_identity: None,
        item_id: 100_000 + index,
        stack_count: 1,
        origin: CursorItemOrigin::Merchant { index },
        applied_transmog: None,
    };
    cursor_model::place(state, cursor)?;
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
    crate::c_api::item_upgrade_eligibility::discard_cursor(&mut *borrow_state_mut(state)?);
    cursor_model::clear(state)?;
    cursor_model::legacy_changed(state);
    Ok(0)
}

/// `CursorHasItem()` — Retail tests native mode 1 only (WM46); other
/// profiles accept any item stack, merchant rows included.
fn cursor_has_item(state: &mut LuaState) -> LuaResult<u32> {
    let has_item = if cursor_model::modeled() {
        cursor_model::current_mode(state)? == cursor_model::MODE_ITEM
    } else {
        borrow_state(state)?
            .cursor_item
            .as_ref()
            .is_some_and(|cursor| matches!(cursor, CursorInfo::Item { .. }))
    };
    state.push(Val::Bool(has_item));
    Ok(1)
}

fn push_mode_test(state: &mut LuaState, mode: u8) -> LuaResult<u32> {
    let matches = cursor_model::current_mode(state)? == mode;
    state.push(Val::Bool(matches));
    Ok(1)
}

/// Retail `CursorHasMoney()`: mode 2 only; guild-bank money fails (WM46).
fn cursor_has_money(state: &mut LuaState) -> LuaResult<u32> {
    push_mode_test(state, cursor_model::MODE_MONEY)
}

/// Retail `CursorHasSpell()`: mode 3 only (WM46).
fn cursor_has_spell(state: &mut LuaState) -> LuaResult<u32> {
    push_mode_test(state, cursor_model::MODE_SPELL)
}

/// Retail `CursorHasMacro()`: mode 7 only (WM46).
fn cursor_has_macro(state: &mut LuaState) -> LuaResult<u32> {
    push_mode_test(state, cursor_model::MODE_MACRO)
}

/// `GetCursorInfo()` — expose the cursor payload in WoW's coarse-grained
/// `(kind, id, ...)` shape. Only the spell/item cases used by the simulator
/// test surface are modeled.
fn get_cursor_info(state: &mut LuaState) -> LuaResult<u32> {
    let cursor = borrow_state(state)?.cursor_item.clone();
    // WM44-A: the empty cursor (mode 0) projects zero values.
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    push_cursor_info(state, cursor)
}

fn push_cursor_info(state: &mut LuaState, cursor: CursorInfo) -> LuaResult<u32> {
    if ACTIVE == ClientProfile::Ptr
        && let CursorInfo::Action { spell_id, .. } | CursorInfo::Spell { spell_id } = &cursor
    {
        return push_spell_cursor(state, *spell_id);
    }
    if cursor_model::modeled()
        && let Some(count) = push_native_projection(state, &cursor)?
    {
        return Ok(count);
    }
    match cursor {
        CursorInfo::Action { spell_id, .. }
        | CursorInfo::Spell { spell_id }
        | CursorInfo::PetAction { spell_id, .. } => push_cursor_kind_id(state, "spell", spell_id),
        CursorInfo::Talent { talent_id, .. } => push_cursor_kind_id(state, "talent", talent_id),
        CursorInfo::Macro { macro_index } => push_cursor_kind_id(state, "macro", macro_index),
        CursorInfo::Item {
            item_id,
            stack_count,
            ..
        } => {
            push_cursor_kind_id(state, "item", item_id)?;
            // Simulator-authored third result: supplied stack count.
            state.push(Val::Num(stack_count as f64));
            Ok(3)
        }
        CursorInfo::ActionBar { item_id } => push_item_id_cursor(state, item_id),
        CursorInfo::Money { copper } => push_cursor_kind_number(state, "money", copper as f64),
        // INFERRED cursor kind; native documentation does not specify outfit payloads.
        CursorInfo::TransmogOutfit { outfit_id } => {
            push_cursor_kind_number(state, "transmogoutfit", outfit_id as f64)
        }
        // Retail-only producers, projected above.
        CursorInfo::EquipmentSet { .. } | CursorInfo::Currency { .. } => Ok(0),
    }
}

/// Retail WM46 projections that differ from the simulator's historical tuples;
/// None leaves the payload to the shared match.
fn push_native_projection(state: &mut LuaState, cursor: &CursorInfo) -> LuaResult<Option<u32>> {
    let count = match *cursor {
        // Mode 5: the stored zero-based row plus one, the Lua index.
        CursorInfo::Item {
            origin: CursorItemOrigin::Merchant { index },
            ..
        } => push_cursor_kind_id(state, "merchant", index)?,
        // Mode 1: ("item", owned item ID, generated link or nil). The cursor
        // carries no owned modifiers, so the link is built from the item ID.
        CursorInfo::Item { item_id, .. } => {
            let link = crate::c_api::item_spell::item_link_for_id(item_id);
            push_cursor_kind_id(state, "item", item_id)?;
            let link = link.map_or(Val::Nil, |link| create_string(state, &link));
            state.push(link);
            3
        }
        // Mode 3, spell content of any producer.
        CursorInfo::Spell { spell_id } | CursorInfo::Action { spell_id, .. } => {
            push_spell_cursor(state, spell_id)?
        }
        CursorInfo::PetAction { packed_action, .. } => {
            push_pet_action_cursor(state, packed_action)?
        }
        // Mode 12: the record's text, INFERRED to be the set name.
        CursorInfo::EquipmentSet { set_id } => push_equipment_set_cursor(state, set_id)?,
        // Mode 13: the metadata integer, INFERRED to be the icon; zero is nil.
        CursorInfo::Currency { currency_id } => push_currency_cursor(state, currency_id)?,
        // Mode 21.
        CursorInfo::TransmogOutfit { outfit_id } => {
            push_cursor_kind_number(state, "outfit", outfit_id as f64)?
        }
        _ => return Ok(None),
    };
    Ok(Some(count))
}

/// Mode 3: ("spell", book slot, book type, spell ID, override or nil) (WM46,
/// WM56; PTR R72 for ordinary Spell and spell-valued Action cursors). The main
/// book is searched first; a miss is slot 0 with "spell".
/// Gaps: only a pet-book hit selects "pet", but the simulator's pet book has no
/// slot order, so its search is a miss; overrides are identity-only, so the
/// override is nil; companion classification is not modelled.
fn push_spell_cursor(state: &mut LuaState, spell_id: u32) -> LuaResult<u32> {
    let slot = crate::lua_api::globals::spellbook_data::find_spell_slot(spell_id)
        .map_or(0, |(slot, _)| slot);
    push_cursor_kind_number(state, "spell", f64::from(slot))?;
    let book = create_string(state, "spell");
    state.push(book);
    state.push(Val::Num(f64::from(spell_id)));
    state.push(Val::Nil);
    Ok(5)
}

/// Mode 4: ("petaction", low 24 bits, pet-book slot or nil, override or nil)
/// (WM46, WM56). Only category `(word >> 24) & 63 == 1` enables the pet-book
/// and override lookups. The pet-book search is a miss (0) and the override
/// nil, for the same gaps as spells.
fn push_pet_action_cursor(state: &mut LuaState, packed_action: u32) -> LuaResult<u32> {
    const CATEGORY_SPELL: u32 = 1;
    push_cursor_kind_number(state, "petaction", f64::from(packed_action & 0x00FF_FFFF))?;
    let book = if (packed_action >> 24) & 63 == CATEGORY_SPELL {
        Val::Num(0.0)
    } else {
        Val::Nil
    };
    state.push(book);
    state.push(Val::Nil);
    Ok(4)
}

/// Zero values when the set record is missing (WM46).
fn push_equipment_set_cursor(state: &mut LuaState, set_id: i32) -> LuaResult<u32> {
    let name = borrow_state(state)?
        .equipment_manager
        .sets
        .iter()
        .find(|set| set.id == set_id)
        .map(|set| set.name.clone());
    let Some(name) = name else {
        return Ok(0);
    };
    let kind = create_string(state, "equipmentset");
    let name = create_string(state, &name);
    state.push(kind);
    state.push(name);
    Ok(2)
}

/// Zero values when the currency record is missing (WM46).
fn push_currency_cursor(state: &mut LuaState, currency_id: i32) -> LuaResult<u32> {
    let icon = borrow_state(state)?
        .currency_info
        .get(&currency_id)
        .map(|info| info.icon_file_id);
    let Some(icon) = icon else {
        return Ok(0);
    };
    push_cursor_kind_number(state, "currency", f64::from(currency_id))?;
    state.push(match icon {
        0 => Val::Nil,
        icon => Val::Num(f64::from(icon)),
    });
    Ok(3)
}

/// WM44-A mode 6 (ActionBar): `("item", itemID, link)` rebuilt from the ID when item
/// metadata is present, otherwise zero values.
fn push_item_id_cursor(state: &mut LuaState, item_id: u32) -> LuaResult<u32> {
    let Some(link) = crate::c_api::item_spell::item_link_for_id(item_id) else {
        return Ok(0);
    };
    push_cursor_kind_id(state, "item", item_id)?;
    let link = create_string(state, &link);
    state.push(link);
    Ok(3)
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
        CursorInfo::TransmogOutfit { outfit_id } => {
            crate::c_api::action_macros::clear_slot(&mut st, slot);
            st.action_outfits.insert(slot, outfit_id);
            if outfit_id == 0 {
                st.equipped_gear_outfit_action_slots.insert(slot);
            }
        }
        // Equipment-set and currency action slots are not modelled.
        CursorInfo::Item { .. }
        | CursorInfo::ActionBar { .. }
        | CursorInfo::Money { .. }
        | CursorInfo::EquipmentSet { .. }
        | CursorInfo::Currency { .. } => {
            return Ok(0);
        }
    }
    drop(st);
    cursor_model::clear(state)?;
    fire_actionbar_slot_changed(state);
    refresh_action_ui_buttons(state, slot);
    cursor_model::legacy_changed(state);
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
    drop(st);
    cursor_model::place(state, CursorInfo::Money { copper: amount })?;
    cursor_model::legacy_changed(state);
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
    drop(st);
    cursor_model::clear(state)?;
    cursor_model::legacy_changed(state);
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
    if crate::client_profile::ACTIVE.has_legacy_item_globals() {
        LuaApiMut::register_function(lua, "SplitContainerItem", split_container_item)?;
    }
    LuaApiMut::register_function(lua, "UseContainerItem", use_container_item)?;
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
    if cursor_model::modeled() {
        LuaApiMut::register_function(lua, "CursorHasMoney", cursor_has_money)?;
        LuaApiMut::register_function(lua, "CursorHasSpell", cursor_has_spell)?;
        LuaApiMut::register_function(lua, "CursorHasMacro", cursor_has_macro)?;
    }
    LuaApiMut::register_function(lua, "PlaceAction", place_action)?;
    LuaApiMut::register_function(lua, "PickupPlayerMoney", pickup_player_money)?;
    LuaApiMut::register_function(lua, "DropCursorMoney", drop_cursor_money)?;
    LuaApiMut::register_function(lua, "GetCursorMoney", get_cursor_money)?;
    Ok(())
}
