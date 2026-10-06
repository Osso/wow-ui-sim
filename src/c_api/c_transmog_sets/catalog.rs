//! Explicit set catalog and slot-source membership used by outfit imports.

use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct TransmogSets {
    pub entries: Vec<TransmogSetInfo>,
    pub slot_sources: BTreeMap<(i32, i32), Vec<i64>>,
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
