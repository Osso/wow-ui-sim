//! Retail visual-table and item-eligibility successors backed by outfit/equipment state.

use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, create_table, table_get, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let transmog = super::super::ensure_namespace(state, "C_Transmog")?;
    table_set_rust_fn_static(state, transmog, "GetSlotVisualInfo", get_slot_visual_info)?;
    let item = super::super::ensure_namespace(state, "C_Item")?;
    table_set_rust_fn_static(
        state,
        item,
        "CanItemTransmogAppearance",
        can_item_transmog_appearance,
    )
}

fn can_item_transmog_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let location = stack_val(state, 1);
    if !matches!(location, Val::Table(_)) {
        return Err(rilua::runtime_error(
            "itemLoc requires an ItemLocation table",
        ));
    }
    let item = super::super::item_spell::location_item(state, location)
        .and_then(|(id, _)| crate::items::get_item(id));
    // INFERRED eligibility subset: item existence, transmoggable inventory type
    // and uncommon-or-better quality. Soulbinding/class/race/form restrictions
    // need inputs the item database does not expose; no native parity is claimed.
    let error = match item {
        None => 1, // NoItem
        Some(item)
            if super::slots::outfit_slot_for_inventory_type(i32::from(item.inventory_type))
                .is_none() =>
        {
            4
        }
        Some(item) if item.quality < 2 => 9, // InvalidSourceQuality
        Some(_) => 0,
    };
    state.push(Val::Bool(error == 0));
    state.push(Val::Num(error as f64));
    Ok(2)
}

fn read_location_number(state: &mut LuaState, location: Val, key: &str) -> LuaResult<i32> {
    let Val::Num(number) = table_get(state, location, key) else {
        return Err(rilua::runtime_error(format!(
            "transmogLocation.{key} requires a number"
        )));
    };
    Ok(number as i32)
}

fn read_location(state: &mut LuaState) -> LuaResult<Option<(i32, i32, i32)>> {
    let location = stack_val(state, 1);
    if !matches!(location, Val::Table(_)) {
        return Err(rilua::runtime_error("transmogLocation requires a table"));
    }
    // TransmogLocationMixin:GetData() is an inventory-indexed API payload,
    // not the outfit-indexed locationData used to construct that mixin.
    let inventory_slot = read_location_number(state, location, "slotID")?;
    let kind = read_location_number(state, location, "type")?;
    let modification = read_location_number(state, location, "modification")?;
    let Some(primary) = super::slots::outfit_slot_from_inventory(inventory_slot - 1) else {
        return Ok(None);
    };
    let slot = if primary == 1 && modification == 1 {
        2
    } else {
        primary
    };
    Ok(Some((slot, inventory_slot, kind)))
}

fn get_slot_visual_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some((slot, inventory_slot, kind)) = read_location(state)? else {
        return Ok(0);
    };
    if !(0..=14).contains(&slot) || !(0..=1).contains(&kind) {
        return Ok(0);
    }
    let values = {
        let sim = borrow_state(state)?;
        build_visual_values(&sim, slot, inventory_slot, kind)
    };
    let table = create_table(state);
    for (key, value) in [
        "baseSourceID",
        "baseVisualID",
        "appliedSourceID",
        "appliedVisualID",
        "pendingSourceID",
        "pendingVisualID",
        "itemSubclass",
    ]
    .iter()
    .zip(values.numbers)
    {
        table_set(state, table, key, Val::Num(value as f64));
    }
    table_set(state, table, "hasUndo", Val::Bool(false));
    table_set(state, table, "isHideVisual", Val::Bool(values.hidden));
    state.push(table);
    Ok(1)
}

struct VisualValues {
    numbers: [i64; 7],
    hidden: bool,
}

fn build_visual_values(sim: &SimState, slot: i32, inventory_slot: i32, kind: i32) -> VisualValues {
    let item = sim.player.equipped_items.get(&inventory_slot);
    let base = item
        .and_then(|item| {
            sim.world
                .transmog_appearances
                .iter()
                .find(|source| source.item_id == item.item_id as i32)
        })
        .map(|source| i64::from(source.source_id))
        .unwrap_or(0);
    let option = sim
        .transmog_outfits
        .viewed_weapon_options
        .get(&slot)
        .copied()
        .unwrap_or(0);
    let key = (slot, kind, option);
    let active = sim
        .active_transmog_outfit_id
        .and_then(|id| sim.transmog_outfits.saved.get(&id));
    let applied = active.and_then(|outfit| outfit.slots.get(&key));
    let pending = sim.transmog_outfits.pending_slots.get(&key);
    let applied_id = applied.map(|row| row.transmog_id).unwrap_or(0);
    let pending_id = pending.map(|row| row.transmog_id).unwrap_or(0);
    // INFERRED: no new outfit undo operation; empty/unmapped identities are zero.
    let subclass = sim
        .transmog_appearance_sources
        .get(&base)
        .map(|row| row.item_subclass)
        .unwrap_or(0);
    VisualValues {
        numbers: [
            base,
            visual_id(sim, base),
            applied_id,
            visual_id(sim, applied_id),
            pending_id,
            visual_id(sim, pending_id),
            i64::from(subclass),
        ],
        hidden: pending.or(applied).is_some_and(|row| row.display_type == 3),
    }
}

fn visual_id(sim: &SimState, source: i64) -> i64 {
    if let Some(row) = sim.transmog_appearance_sources.get(&source) {
        return row.item_appearance_id;
    }
    sim.world
        .transmog_appearances
        .iter()
        .find(|row| i64::from(row.source_id) == source)
        .map(|row| i64::from(row.visual_id))
        .unwrap_or(0)
}
