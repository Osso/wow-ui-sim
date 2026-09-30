//! Empty-default catalog; no outfit creation, lifecycle, or persistence policy.

use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

#[derive(Debug, Default)]
pub struct OutfitCatalog {
    pub entries: Vec<OutfitEntry>,
}

#[derive(Debug, Clone)]
pub struct OutfitEntry {
    pub outfit_id: i64,
    pub name: String,
    pub situation_categories: Vec<String>,
    pub icon: i64,
    pub is_event_outfit: bool,
    pub is_disabled: bool,
    pub player_facing_outfit_index: i64,
}

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, handler) in [
        (
            "GetOutfitInfo",
            get_outfit_info as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("GetOutfitInfoByName", get_outfit_info_by_name),
        (
            "GetOutfitInfoByPlayerFacingIndex",
            get_outfit_info_by_player_facing_index,
        ),
        ("GetOutfitsInfo", get_outfits_info),
    ] {
        table_set_rust_fn_static(state, namespace, name, handler)?;
    }
    Ok(())
}

fn read_number(state: &LuaState) -> LuaResult<f64> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error("outfit lookup requires a number"));
    };
    Ok(number)
}

fn get_outfit_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_number(state)?;
    push_matching_entry(state, |entry| entry.outfit_id as f64 == id)
}

fn get_outfit_info_by_player_facing_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_number(state)?;
    push_matching_entry(state, |entry| {
        entry.player_facing_outfit_index as f64 == index
    })
}

fn get_outfit_info_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Str(name) = value else {
        return Err(rilua::runtime_error("outfit lookup requires a name string"));
    };
    let bytes = state
        .gc
        .string_arena
        .get(name)
        .ok_or_else(|| rilua::runtime_error("outfit name string has been collected"))?
        .data();
    let name = std::str::from_utf8(bytes)
        .map_err(|_| rilua::runtime_error("outfit name must be valid UTF-8"))?
        .to_lowercase();
    // Inferred Unicode policy; official documentation specifies case-insensitivity only.
    push_matching_entry(state, |entry| entry.name.to_lowercase() == name)
}

fn push_matching_entry(
    state: &mut LuaState,
    matches: impl Fn(&OutfitEntry) -> bool,
) -> LuaResult<u32> {
    let entry = borrow_state(state)?
        .transmog_outfit_catalog
        .entries
        .iter()
        .find(|entry| matches(entry))
        .cloned();
    let Some(entry) = entry else { return Ok(0) };
    let table = build_entry_table(state, &entry);
    state.push(table);
    Ok(1)
}

fn get_outfits_info(state: &mut LuaState) -> LuaResult<u32> {
    let entries = borrow_state(state)?.transmog_outfit_catalog.entries.clone();
    let array = build_entry_array(state, &entries);
    state.push(array);
    Ok(1)
}

fn build_entry_array(state: &mut LuaState, entries: &[OutfitEntry]) -> Val {
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, entry) in entries.iter().enumerate() {
        let row = build_entry_table(state, entry);
        table_set_num(state, table, (index + 1) as f64, row);
    }
    array
}

fn build_entry_table(state: &mut LuaState, entry: &OutfitEntry) -> Val {
    let table = create_table(state);
    let name = create_string(state, &entry.name);
    let categories = build_categories_table(state, &entry.situation_categories);
    table_set(state, table, "outfitID", Val::Num(entry.outfit_id as f64));
    table_set(state, table, "name", name);
    table_set(state, table, "situationCategories", categories);
    table_set(state, table, "icon", Val::Num(entry.icon as f64));
    table_set(
        state,
        table,
        "isEventOutfit",
        Val::Bool(entry.is_event_outfit),
    );
    table_set(state, table, "isDisabled", Val::Bool(entry.is_disabled));
    table_set(
        state,
        table,
        "playerFacingOutfitIndex",
        Val::Num(entry.player_facing_outfit_index as f64),
    );
    table
}

fn build_categories_table(state: &mut LuaState, categories: &[String]) -> Val {
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, category) in categories.iter().enumerate() {
        let value = create_string(state, category);
        table_set_num(state, table, (index + 1) as f64, value);
    }
    array
}
