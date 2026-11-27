//! Empty-default catalog; no outfit creation, lifecycle, or persistence policy.

use crate::c_api::outfit_entry::{build_entry_array, build_entry_table};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

#[derive(Debug, Default)]
pub struct OutfitCatalog {
    pub entries: Vec<OutfitEntry>,
}

pub use crate::c_api::outfit_entry::OutfitEntry;

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
