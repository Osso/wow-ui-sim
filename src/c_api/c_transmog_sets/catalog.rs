//! Explicit set catalog and slot-source membership used by outfit imports.

use crate::c_api::crafting_input::read_array;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
    table_set_num,
};
use crate::lua_bridge::{TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct TransmogSets {
    pub entries: Vec<TransmogSetInfo>,
    pub slot_sources: BTreeMap<(i32, i32), Vec<i64>>,
}

#[derive(Debug, Clone, Default)]
pub struct TransmogSetPrimaryAppearanceInfo {
    pub appearance_id: i32,
    pub collected: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TransmogSetItemInfo {
    pub item_id: i32,
    pub item_modified_appearance_id: i32,
    pub inventory_slot: i32,
    pub inventory_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct TransmogSetInfo {
    pub set_id: i32,
    pub name: String,
    pub base_set_id: Option<i32>,
    pub description: Option<String>,
    pub label: Option<String>,
    pub expansion_id: i32,
    pub patch_id: i32,
    pub ui_order: i32,
    pub class_mask: i32,
    pub hidden_until_collected: bool,
    pub required_faction: Option<String>,
    pub collected: bool,
    pub favorite: bool,
    pub limited_time_set: bool,
    pub valid_for_character: bool,
    pub grant_as_preceding_variant: bool,
    /// Host filter classification, not a returned TransmogSetInfo field.
    pub is_pvp: bool,
    pub primary_appearances: Vec<TransmogSetPrimaryAppearanceInfo>,
    pub appearances: Vec<TransmogSetItemInfo>,
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetTransmogSetCatalog", set_catalog)
}

fn set_catalog(state: &mut LuaState) -> LuaResult<u32> {
    let rows = read_array(state, stack_val(state, 1))?;
    let mut entries = Vec::with_capacity(rows.len());
    let mut slot_sources = BTreeMap::new();
    for row in rows {
        let (entry, sources) = read_set(state, row)?;
        entries.push(entry);
        slot_sources.extend(sources);
    }
    let mut sim = borrow_state_mut(state)?;
    sim.transmog_sets.entries = entries;
    sim.transmog_sets.slot_sources = slot_sources;
    Ok(0)
}

fn read_set(
    state: &mut LuaState,
    row: Val,
) -> LuaResult<(TransmogSetInfo, BTreeMap<(i32, i32), Vec<i64>>)> {
    let set_id = required_integer(state, row, "setID")?;
    let entry = TransmogSetInfo {
        set_id,
        name: required_text(state, row, "name")?,
        base_set_id: optional_integer(state, row, "baseSetID")?,
        collected: required_bool(state, row, "collected")?,
        valid_for_character: required_bool(state, row, "validForCharacter")?,
        primary_appearances: read_primary_appearances(state, row)?,
        appearances: read_appearances(state, row)?,
        ..Default::default()
    };
    Ok((entry, read_slot_sources(state, row, set_id)?))
}

fn read_primary_appearances(
    state: &mut LuaState,
    row: Val,
) -> LuaResult<Vec<TransmogSetPrimaryAppearanceInfo>> {
    optional_array(state, row, "primaryAppearances")?
        .into_iter()
        .map(|appearance| {
            Ok(TransmogSetPrimaryAppearanceInfo {
                appearance_id: required_integer(state, appearance, "appearanceID")?,
                collected: required_bool(state, appearance, "collected")?,
            })
        })
        .collect()
}

fn read_appearances(state: &mut LuaState, row: Val) -> LuaResult<Vec<TransmogSetItemInfo>> {
    optional_array(state, row, "appearances")?
        .into_iter()
        .map(|item| {
            Ok(TransmogSetItemInfo {
                item_id: required_integer(state, item, "itemID")?,
                item_modified_appearance_id: required_integer(
                    state,
                    item,
                    "itemModifiedAppearanceID",
                )?,
                inventory_slot: required_integer(state, item, "invSlot")?,
                inventory_type: required_text(state, item, "invType")?,
            })
        })
        .collect()
}

fn read_slot_sources(
    state: &mut LuaState,
    row: Val,
    set_id: i32,
) -> LuaResult<BTreeMap<(i32, i32), Vec<i64>>> {
    optional_array(state, row, "slotSources")?
        .into_iter()
        .map(|source_row| {
            let slot = required_integer(state, source_row, "slot")?;
            let ids_value = table_get(state, source_row, "sourceIDs");
            let ids = read_array(state, ids_value)?
                .into_iter()
                .map(|value| integer_i64(value, "sourceID"))
                .collect::<LuaResult<Vec<_>>>()?;
            Ok(((set_id, slot), ids))
        })
        .collect()
}

fn optional_array(state: &mut LuaState, row: Val, key: &str) -> LuaResult<Vec<Val>> {
    match table_get(state, row, key) {
        Val::Nil => Ok(Vec::new()),
        values => read_array(state, values),
    }
}

fn required_integer(state: &mut LuaState, row: Val, key: &str) -> LuaResult<i32> {
    match table_get(state, row, key) {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value) =>
        {
            Ok(value as i32)
        }
        _ => Err(rilua::runtime_error(format!(
            "transmog set {key} must be an i32"
        ))),
    }
}

fn integer_i64(value: Val, key: &str) -> LuaResult<i64> {
    match value {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && value >= i64::MIN as f64
                && value < -(i64::MIN as f64) =>
        {
            Ok(value as i64)
        }
        _ => Err(rilua::runtime_error(format!(
            "transmog set {key} must be an i64"
        ))),
    }
}

fn optional_integer(state: &mut LuaState, row: Val, key: &str) -> LuaResult<Option<i32>> {
    match table_get(state, row, key) {
        Val::Nil => Ok(None),
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value) =>
        {
            Ok(Some(value as i32))
        }
        _ => Err(rilua::runtime_error(format!(
            "transmog set {key} must be an integer"
        ))),
    }
}

fn required_bool(state: &mut LuaState, row: Val, key: &str) -> LuaResult<bool> {
    match table_get(state, row, key) {
        Val::Bool(value) => Ok(value),
        _ => Err(rilua::runtime_error(format!(
            "transmog set {key} must be a boolean"
        ))),
    }
}

fn required_text(state: &mut LuaState, row: Val, key: &str) -> LuaResult<String> {
    let value = table_get(state, row, key);
    crate::lua_api::methods::val_to_string(state, value)
        .ok_or_else(|| rilua::runtime_error(format!("transmog set {key} must be a string")))
}

fn set_array(state: &mut LuaState, entries: &[TransmogSetInfo]) -> Val {
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, entry) in entries.iter().enumerate() {
        let row = build_set_info(state, entry);
        table_set_num(state, table, (index + 1) as f64, row);
    }
    Val::Table(table)
}

pub(super) fn get_all_sets(state: &mut LuaState) -> LuaResult<u32> {
    let entries = borrow_state(state)?.transmog_sets.entries.clone();
    let array = set_array(state, &entries);
    state.push(array);
    Ok(1)
}

// INFERRED (R88cs pending): a base entry has no parent or names itself.
pub(super) fn get_base_sets(state: &mut LuaState) -> LuaResult<u32> {
    let entries = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .filter(|entry| {
            entry
                .base_set_id
                .is_none_or(|base_id| base_id == entry.set_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    let array = set_array(state, &entries);
    state.push(array);
    Ok(1)
}

pub(super) fn get_usable_sets(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (R88cs pending): the shared UI defines usable sets as complete sets.
    let entries = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .filter(|entry| entry.valid_for_character && entry.collected)
        .cloned()
        .collect::<Vec<_>>();
    let array = set_array(state, &entries);
    state.push(array);
    Ok(1)
}

// INFERRED (R88cs pending): missing records retain the shim's zero result.
pub(super) fn get_base_set_id(state: &mut LuaState) -> LuaResult<u32> {
    let id: i32 = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let base_id = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .find(|entry| entry.set_id == id)
        .map(|entry| entry.base_set_id.unwrap_or(entry.set_id))
        .unwrap_or(0);
    state.push(Val::Num(base_id as f64));
    Ok(1)
}

pub(super) fn get_primary_appearances(state: &mut LuaState) -> LuaResult<u32> {
    let id: i32 = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let appearances = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .find(|entry| entry.set_id == id)
        .map(|entry| entry.primary_appearances.clone())
        .unwrap_or_default();
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, appearance) in appearances.iter().enumerate() {
        let row = create_table(state);
        table_set(
            state,
            row,
            "appearanceID",
            Val::Num(appearance.appearance_id as f64),
        );
        table_set(state, row, "collected", Val::Bool(appearance.collected));
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(Val::Table(table));
    Ok(1)
}

// INFERRED (R88cs pending): collection includes collected variants of a base.
pub(super) fn is_base_set_collected(state: &mut LuaState) -> LuaResult<u32> {
    let id: i32 = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let collected = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .any(|entry| (entry.set_id == id || entry.base_set_id == Some(id)) && entry.collected);
    state.push(Val::Bool(collected));
    Ok(1)
}

pub(super) fn get_sources_for_slot(state: &mut LuaState) -> LuaResult<u32> {
    let set_id: i32 = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let slot: i32 = crate::lua_bridge::FromStack::from_stack(state, 2)?;
    let ids = borrow_state(state)?
        .transmog_sets
        .slot_sources
        .get(&(set_id, slot))
        .cloned()
        .unwrap_or_default();
    let sources = {
        let sim = borrow_state(state)?;
        ids.iter()
            .filter_map(|id| crate::c_api::c_transmog_collection::read_source_info(&sim, *id))
            .collect::<Vec<_>>()
    };
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, source) in sources.iter().enumerate() {
        let row = crate::c_api::c_transmog_collection::appearance_sources::build_source_table(
            state, source,
        );
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(Val::Table(table));
    Ok(1)
}

pub(super) fn get_all_set_appearances(state: &mut LuaState) -> LuaResult<u32> {
    let id: i32 = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let items = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .find(|entry| entry.set_id == id)
        .map(|entry| entry.appearances.clone())
        .unwrap_or_default();
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, item) in items.iter().enumerate() {
        let row = create_table(state);
        table_set(state, row, "itemID", Val::Num(item.item_id as f64));
        table_set(
            state,
            row,
            "itemModifiedAppearanceID",
            Val::Num(item.item_modified_appearance_id as f64),
        );
        table_set(state, row, "invSlot", Val::Num(item.inventory_slot as f64));
        let inventory_type = create_string(state, &item.inventory_type);
        table_set(state, row, "invType", inventory_type);
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(Val::Table(table));
    Ok(1)
}

pub(super) fn get_available(state: &mut LuaState) -> LuaResult<u32> {
    let entries = {
        let sim = borrow_state(state)?;
        sim.transmog_sets
            .entries
            .iter()
            .filter(|entry| {
                let collection_filter = if entry.collected { 1 } else { 2 };
                let content_filter = if entry.is_pvp { 4 } else { 3 };
                let checked = |index| {
                    sim.transmog_set_filters
                        .get(&index)
                        .copied()
                        .unwrap_or(true)
                };
                is_available(entry) && checked(collection_filter) && checked(content_filter)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, entry) in entries.iter().enumerate() {
        let row = build_set_info(state, entry);
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(array);
    Ok(1)
}

fn is_available(entry: &TransmogSetInfo) -> bool {
    entry.valid_for_character && (!entry.hidden_until_collected || entry.collected)
}

pub(super) fn has_any_valid(state: &mut LuaState) -> LuaResult<u32> {
    let valid = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .any(|entry| entry.valid_for_character);
    state.push(Val::Bool(valid));
    Ok(1)
}

pub(super) fn has_available(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: availability ignores UI filters, so a zero-result filter does
    // not hide the controls needed to recover the player's available sets.
    let available = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .any(is_available);
    state.push(Val::Bool(available));
    Ok(1)
}

fn build_set_info(state: &mut LuaState, entry: &TransmogSetInfo) -> Val {
    let row = create_table(state);
    for (key, value) in [
        ("setID", entry.set_id),
        ("expansionID", entry.expansion_id),
        ("patchID", entry.patch_id),
        ("uiOrder", entry.ui_order),
        ("classMask", entry.class_mask),
    ] {
        table_set(state, row, key, Val::Num(value as f64));
    }
    for (key, value) in [
        ("hiddenUntilCollected", entry.hidden_until_collected),
        ("collected", entry.collected),
        ("favorite", entry.favorite),
        ("limitedTimeSet", entry.limited_time_set),
        ("validForCharacter", entry.valid_for_character),
        ("grantAsPrecedingVariant", entry.grant_as_preceding_variant),
    ] {
        table_set(state, row, key, Val::Bool(value));
    }
    let name = create_string(state, &entry.name);
    table_set(state, row, "name", name);
    if let Some(id) = entry.base_set_id {
        table_set(state, row, "baseSetID", Val::Num(id as f64));
    }
    for (key, value) in [
        ("description", &entry.description),
        ("label", &entry.label),
        ("requiredFaction", &entry.required_faction),
    ] {
        if let Some(value) = value {
            let value = create_string(state, value);
            table_set(state, row, key, value);
        }
    }
    row
}

pub(super) fn get_info(state: &mut LuaState) -> LuaResult<u32> {
    let id: i32 = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let entry = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .find(|e| e.set_id == id)
        .cloned();
    match entry {
        Some(entry) => {
            let row = build_set_info(state, &entry);
            state.push(row);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn get_variants(state: &mut LuaState) -> LuaResult<u32> {
    let id: i32 = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let entries: Vec<_> = borrow_state(state)?
        .transmog_sets
        .entries
        .iter()
        .filter(|e| e.base_set_id == Some(id) && e.set_id != id)
        .cloned()
        .collect();
    let array = create_table(state);
    state.push(array);
    for (index, entry) in entries.iter().enumerate() {
        let row = build_set_info(state, entry);
        crate::c_api::helpers::set_table_array(state, array, index as i64 + 1, row);
    }
    Ok(1)
}
