//! `C_CreatureInfo` helpers backed by static game data.

use super::ensure_namespace;
use crate::lua_api::game_data::{RACE_DATA, class_info_by_index};
use crate::lua_api::methods::{
    create_string, create_table, create_table_with_capacity, table_set_static,
};
use crate::lua_bridge::FromStack;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CREATURE_CLASS_INFO_HASH_FIELDS: usize = 3;
const CREATURE_RACE_INFO_HASH_FIELDS: usize = 3;
const CREATURE_TYPE_INFO_HASH_FIELDS: usize = 2;
const CREATURE_FACTION_INFO_HASH_FIELDS: usize = 3;

pub(super) fn register_creature_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_CreatureInfo")?;
    #[cfg(feature = "retail-12-0-0")]
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetCreatureID",
        crate::c_api::c_creature_info::get_creature_id,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetClassInfo",
        c_creature_info_get_class_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetRaceInfo",
        c_creature_info_get_race_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetCreatureTypeIDs",
        c_creature_info_get_creature_type_ids,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetCreatureTypeInfo",
        c_creature_info_get_creature_type_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetCreatureFamilyIDs",
        c_creature_info_get_creature_family_ids,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetCreatureFamilyInfo",
        c_creature_info_get_creature_family_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFactionInfo",
        c_creature_info_get_faction_info,
    )?;
    Ok(())
}

fn c_creature_info_get_class_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1).unwrap_or(1);
    let (class_label, class_file, class_id) = class_info_by_index(index);
    let info = create_table_with_capacity(state, CREATURE_CLASS_INFO_HASH_FIELDS);
    let class_name = create_string(state, class_label);
    let class_file = create_string(state, class_file);
    table_set_static(state, info, "className", class_name);
    table_set_static(state, info, "classFile", class_file);
    table_set_static(state, info, "classID", Val::Num(class_id as f64));
    state.push(info);
    Ok(1)
}

fn c_creature_info_get_race_info(state: &mut LuaState) -> LuaResult<u32> {
    let race_id = i32::from_stack(state, 1).unwrap_or(1);
    let (race_name, client_file, race_id) = creature_race_info_by_id(race_id);
    let info = create_table_with_capacity(state, CREATURE_RACE_INFO_HASH_FIELDS);
    let race_name = create_string(state, race_name);
    let client_file = create_string(state, client_file);
    table_set_static(state, info, "raceName", race_name);
    table_set_static(state, info, "clientFileString", client_file);
    table_set_static(state, info, "raceID", Val::Num(race_id as f64));
    state.push(info);
    Ok(1)
}

fn c_creature_info_get_creature_type_ids(state: &mut LuaState) -> LuaResult<u32> {
    let ids = create_table(state);
    if let Val::Table(table_ref) = &ids {
        for (index, value) in (1_i32..=10).enumerate() {
            if let Some(table) = state.gc.tables.get_mut(*table_ref) {
                let _ = table.raw_set(
                    Val::Num((index + 1) as f64),
                    Val::Num(value as f64),
                    &state.gc.string_arena,
                );
            }
        }
    }
    state.push(ids);
    Ok(1)
}

fn c_creature_info_get_creature_type_info(state: &mut LuaState) -> LuaResult<u32> {
    let creature_type_id = i32::from_stack(state, 1).unwrap_or(0);
    let creature_type_name = match creature_type_id {
        1 => "Beast",
        2 => "Dragonkin",
        3 => "Demon",
        4 => "Elemental",
        5 => "Undead",
        6 => "Critter",
        7 => "Humanoid",
        8 => "Mechanical",
        9 => "Not specified",
        10 => "Other",
        _ => "Unknown",
    };
    let info = create_table_with_capacity(state, CREATURE_TYPE_INFO_HASH_FIELDS);
    let name = create_string(state, creature_type_name);
    table_set_static(state, info, "name", name);
    table_set_static(
        state,
        info,
        "creatureTypeID",
        Val::Num(creature_type_id as f64),
    );
    state.push(info);
    Ok(1)
}

fn c_creature_info_get_creature_family_ids(state: &mut LuaState) -> LuaResult<u32> {
    let ids = create_table(state);
    state.push(ids);
    Ok(1)
}

fn c_creature_info_get_creature_family_info(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn c_creature_info_get_faction_info(state: &mut LuaState) -> LuaResult<u32> {
    let faction_id = i32::from_stack(state, 1).unwrap_or(0);
    let Some((name, group_tag)) = creature_faction_info(faction_id) else {
        return Ok(0);
    };
    let info = create_table_with_capacity(state, CREATURE_FACTION_INFO_HASH_FIELDS);
    let name = create_string(state, name);
    let group_tag = create_string(state, group_tag);
    table_set_static(state, info, "name", name);
    table_set_static(state, info, "groupTag", group_tag);
    table_set_static(state, info, "factionID", Val::Num(faction_id as f64));
    state.push(info);
    Ok(1)
}

fn creature_race_info_by_id(race_id: i32) -> (&'static str, &'static str, i32) {
    match race_id {
        52 | 70 => ("Dracthyr", "Dracthyr", race_id),
        84 | 85 => ("Earthen", "Earthen", race_id),
        1..=15 => {
            let idx = (race_id - 1) as usize;
            let (race_name, client_file, _) = RACE_DATA[idx];
            (race_name, client_file, race_id)
        }
        _ => ("Unknown", "Unknown", race_id),
    }
}

fn creature_faction_info(faction_id: i32) -> Option<(&'static str, &'static str)> {
    match faction_id {
        1 => Some(("Alliance", "Alliance")),
        2 => Some(("Horde", "Horde")),
        24 => Some(("Neutral", "Neutral")),
        _ => None,
    }
}
