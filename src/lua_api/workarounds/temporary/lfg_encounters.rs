//! Retire when GetSearchResultEncounterInfo has a documented native contract.
//! Pinned generated API documentation has no declaration; nil for no encounters is a best guess.
use crate::lua_api::globals::lfg_list::premade_listing;
use crate::lua_api::methods::{create_string, create_table};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn get_search_result_encounter_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let Some(listing) = premade_listing(state, id) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    if listing.metadata.defeated_encounters.is_empty() {
        state.push(Val::Nil);
        return Ok(1);
    }
    let result = create_table(state);
    for (index, name) in listing.metadata.defeated_encounters.iter().enumerate() {
        let value = create_string(state, name);
        if let Val::Table(table_ref) = result {
            if let Some(table) = state.gc.tables.get_mut(table_ref) {
                let _ = table.raw_set(Val::Num(index as f64 + 1.0), value, &state.gc.string_arena);
            }
            state.gc.barrier_back(table_ref);
        }
    }
    state.push(result);
    Ok(1)
}
