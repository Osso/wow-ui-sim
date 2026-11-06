//! `C_WarbandScene` collection surface backed by `WorldState.warband_scenes`.

use super::ensure_namespace;
use crate::event::Event;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
    table_set_num,
};
use crate::lua_api::state_types::WarbandSceneData;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const RANDOM_WARBAND_SCENE_ID: u32 = 0;

pub(super) fn register_warband_scene_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_WarbandScene")?;
    table_set_rust_fn_static(state, table_ref, "GetRandomEntryID", get_random_entry_id)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetWarbandSceneEntry",
        get_warband_scene_entry,
    )?;
    table_set_rust_fn_static(state, table_ref, "HasWarbandScene", has_warband_scene)?;
    table_set_rust_fn_static(state, table_ref, "IsFavorite", is_favorite)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SearchWarbandSceneEntries",
        search_warband_scene_entries,
    )?;
    table_set_rust_fn_static(state, table_ref, "SetFavorite", set_favorite)?;
    crate::c_api::collections_availability::register_warband_scenes_available(state, table_ref)
}

fn get_random_entry_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(RANDOM_WARBAND_SCENE_ID as f64));
    Ok(1)
}

fn get_warband_scene_entry(state: &mut LuaState) -> LuaResult<u32> {
    let warband_scene_id = i32::from_stack(state, 1)? as u32;
    let entry = if warband_scene_id == RANDOM_WARBAND_SCENE_ID {
        Some(random_entry())
    } else {
        let sim = borrow_state(state)?;
        sim.world
            .warband_scenes
            .iter()
            .find(|entry| entry.warband_scene_id == warband_scene_id)
            .cloned()
    };

    let Some(entry) = entry else {
        return Ok(0);
    };
    let table = entry_to_table(state, &entry);
    state.push(table);
    Ok(1)
}

fn has_warband_scene(state: &mut LuaState) -> LuaResult<u32> {
    let warband_scene_id = i32::from_stack(state, 1)? as u32;
    let has = if warband_scene_id == RANDOM_WARBAND_SCENE_ID {
        true
    } else {
        let sim = borrow_state(state)?;
        sim.world
            .warband_scenes
            .iter()
            .find(|entry| entry.warband_scene_id == warband_scene_id)
            .map(|entry| entry.is_collected)
            .unwrap_or(false)
    };
    state.push(Val::Bool(has));
    Ok(1)
}

fn is_favorite(state: &mut LuaState) -> LuaResult<u32> {
    let warband_scene_id = i32::from_stack(state, 1)? as u32;
    let favorite = if warband_scene_id == RANDOM_WARBAND_SCENE_ID {
        false
    } else {
        let sim = borrow_state(state)?;
        sim.world
            .warband_scenes
            .iter()
            .find(|entry| entry.warband_scene_id == warband_scene_id)
            .map(|entry| entry.is_favorite)
            .unwrap_or(false)
    };
    state.push(Val::Bool(favorite));
    Ok(1)
}

fn search_warband_scene_entries(state: &mut LuaState) -> LuaResult<u32> {
    let (owned_only, favorites_only) = parse_search_filters(state, stack_val(state, 1));
    let entry_ids = {
        let sim = borrow_state(state)?;
        sim.world
            .warband_scenes
            .iter()
            .filter(|entry| !owned_only || entry.is_collected)
            .filter(|entry| !favorites_only || entry.is_favorite)
            .map(|entry| entry.warband_scene_id as f64)
            .collect::<Vec<_>>()
    };

    let array = create_table(state);
    let Val::Table(array_ref) = array else {
        state.push(Val::Nil);
        return Ok(1);
    };
    for (index, warband_scene_id) in entry_ids.iter().enumerate() {
        table_set_num(
            state,
            array_ref,
            (index + 1) as f64,
            Val::Num(*warband_scene_id),
        );
    }
    state.push(array);
    Ok(1)
}

fn set_favorite(state: &mut LuaState) -> LuaResult<u32> {
    let warband_scene_id = i32::from_stack(state, 1)? as u32;
    let favorite = bool::from_stack(state, 2)?;
    if warband_scene_id == RANDOM_WARBAND_SCENE_ID {
        return Ok(0);
    }

    let mut changed = false;
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(entry) = sim
            .world
            .warband_scenes
            .iter_mut()
            .find(|entry| entry.warband_scene_id == warband_scene_id)
        {
            let next_favorite = favorite && entry.is_collected;
            if entry.is_favorite != next_favorite {
                entry.is_favorite = next_favorite;
                changed = true;
            }
        }

        if changed {
            sim.events.push(Event {
                name: "WARBAND_SCENE_FAVORITES_UPDATED".to_string(),
                args: vec![],
            });
        }
    }

    Ok(0)
}

fn parse_search_filters(state: &mut LuaState, search_params: Val) -> (bool, bool) {
    let owned_only = matches!(
        table_get(state, search_params.clone(), "ownedOnly"),
        Val::Bool(true)
    );
    let favorites_only = matches!(
        table_get(state, search_params, "favoritesOnly"),
        Val::Bool(true)
    );
    (owned_only, favorites_only)
}

fn entry_to_table(state: &mut LuaState, entry: &WarbandSceneData) -> Val {
    let table = create_table(state);
    let name = create_string(state, &entry.name);
    let description = create_string(state, &entry.description);
    let source = create_string(state, &entry.source);
    let texture_kit = create_string(state, &entry.texture_kit);
    table_set(
        state,
        table,
        "warbandSceneID",
        Val::Num(entry.warband_scene_id as f64),
    );
    table_set(state, table, "name", name);
    table_set(state, table, "description", description);
    table_set(state, table, "source", source);
    table_set(state, table, "quality", Val::Num(entry.quality as f64));
    table_set(state, table, "textureKit", texture_kit);
    table_set(state, table, "isFavorite", Val::Bool(entry.is_favorite));
    table_set(state, table, "hasFanfare", Val::Bool(entry.has_fanfare));
    table_set(
        state,
        table,
        "sourceType",
        Val::Num(entry.source_type as f64),
    );
    table
}

fn random_entry() -> WarbandSceneData {
    WarbandSceneData {
        warband_scene_id: RANDOM_WARBAND_SCENE_ID,
        name: "Random Campsite".to_string(),
        description: "Use a random owned campsite.".to_string(),
        source: String::new(),
        quality: 1,
        texture_kit: "campcollection-bg-random".to_string(),
        is_collected: true,
        is_favorite: false,
        has_fanfare: false,
        source_type: 0,
    }
}
