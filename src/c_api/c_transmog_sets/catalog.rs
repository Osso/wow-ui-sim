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
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetTransmogSetCatalog", set_catalog)
}

fn set_catalog(state: &mut LuaState) -> LuaResult<u32> {
    let rows = read_array(state, stack_val(state, 1))?;
    let entries = rows
        .into_iter()
        .map(|row| read_set(state, row))
        .collect::<LuaResult<Vec<_>>>()?;
    borrow_state_mut(state)?.transmog_sets.entries = entries;
    Ok(0)
}

fn read_set(state: &mut LuaState, row: Val) -> LuaResult<TransmogSetInfo> {
    Ok(TransmogSetInfo {
        set_id: required_integer(state, row, "setID")?,
        name: required_text(state, row, "name")?,
        base_set_id: optional_integer(state, row, "baseSetID")?,
        collected: required_bool(state, row, "collected")?,
        valid_for_character: required_bool(state, row, "validForCharacter")?,
        primary_appearances: read_primary_appearances(state, row)?,
        ..Default::default()
    })
}

fn read_primary_appearances(
    state: &mut LuaState,
    row: Val,
) -> LuaResult<Vec<TransmogSetPrimaryAppearanceInfo>> {
    read_array(state, table_get(state, row, "primaryAppearances"))?
        .into_iter()
        .map(|appearance| {
            Ok(TransmogSetPrimaryAppearanceInfo {
                appearance_id: required_integer(state, appearance, "appearanceID")?,
                collected: required_bool(state, appearance, "collected")?,
            })
        })
        .collect()
}

fn required_integer(state: &LuaState, row: Val, key: &str) -> LuaResult<i32> {
    match table_get(state, row, key) {
        Val::Num(value) if value.is_finite() && value.fract() == 0.0 => Ok(value as i32),
        _ => Err(rilua::runtime_error(format!(
            "transmog set {key} must be an integer"
        ))),
    }
}

fn optional_integer(state: &LuaState, row: Val, key: &str) -> LuaResult<Option<i32>> {
    match table_get(state, row, key) {
        Val::Nil => Ok(None),
        Val::Num(value) if value.is_finite() && value.fract() == 0.0 => Ok(Some(value as i32)),
        _ => Err(rilua::runtime_error(format!(
            "transmog set {key} must be an integer"
        ))),
    }
}

fn required_bool(state: &LuaState, row: Val, key: &str) -> LuaResult<bool> {
    match table_get(state, row, key) {
        Val::Bool(value) => Ok(value),
        _ => Err(rilua::runtime_error(format!(
            "transmog set {key} must be a boolean"
        ))),
    }
}

fn required_text(state: &LuaState, row: Val, key: &str) -> LuaResult<String> {
    crate::lua_api::methods::val_to_string(state, table_get(state, row, key))
        .ok_or_else(|| rilua::runtime_error(format!("transmog set {key} must be a string")))
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
