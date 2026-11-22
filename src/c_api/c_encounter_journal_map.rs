//! INFERRED R88dj (native evidence pending): return host-supplied encounter rows for one UI map.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table_with_capacity, table_get, table_set_num,
    table_set_static,
};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

const INPUT_ERROR: &str =
    "A_Admin.SetEncounterJournalMapEncounters: expected an array of encounter tables";

#[derive(Clone, Copy, Debug)]
pub(crate) struct EncounterJournalMapEncounter {
    pub(crate) encounter_id: u32,
    pub(crate) map_x: f64,
    pub(crate) map_y: f64,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_EncounterJournal")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetEncountersOnMap",
        get_encounters_on_map,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetEncounterJournalMapEncounters", set_encounters)
}

fn get_encounters_on_map(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = u32::from_stack(state, 1)?;
    let rows = borrow_state(state)?
        .encounter_journal
        .map_encounters
        .get(&map_id)
        .cloned()
        .unwrap_or_default();
    let result = create_table_with_capacity(state, rows.len());
    for (index, row) in rows.iter().enumerate() {
        let info = encounter_map_info(state, row);
        table_set_num(state, result, (index + 1) as f64, info);
    }
    state.push(result);
    Ok(1)
}

fn encounter_map_info(state: &mut LuaState, row: &EncounterJournalMapEncounter) -> Val {
    let info = create_table_with_capacity(state, 3);
    table_set_static(
        state,
        info,
        "encounterID",
        Val::Num(f64::from(row.encounter_id)),
    );
    table_set_static(state, info, "mapX", Val::Num(row.map_x));
    table_set_static(state, info, "mapY", Val::Num(row.map_y));
    info
}

fn set_encounters(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = u32::from_stack(state, 1)?;
    let rows = read_encounters(state, stack_val(state, 2))?;
    borrow_state_mut(state)?
        .encounter_journal
        .map_encounters
        .insert(map_id, rows);
    Ok(0)
}

fn read_encounters(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<Vec<EncounterJournalMapEncounter>> {
    let Val::Table(array) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| runtime_error(INPUT_ERROR))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let row = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            read_encounter(state, row)
        })
        .collect()
}

fn read_encounter(state: &mut LuaState, value: Val) -> LuaResult<EncounterJournalMapEncounter> {
    let Val::Table(table) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    Ok(EncounterJournalMapEncounter {
        encounter_id: read_encounter_id(state, table)?,
        map_x: read_coordinate(state, table, "mapX")?,
        map_y: read_coordinate(state, table, "mapY")?,
    })
}

fn read_encounter_id(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<u32> {
    match field(state, table, "encounterID") {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=u32::MAX as f64).contains(&value) =>
        {
            Ok(value as u32)
        }
        _ => Err(invalid_field("encounterID")),
    }
}

fn read_coordinate(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<f64> {
    match field(state, table, key) {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(invalid_field(key)),
    }
}

fn field(state: &mut LuaState, table: GcRef<Table>, key: &str) -> Val {
    table_get(state, Val::Table(table), key)
}

fn invalid_field(key: &str) -> rilua::LuaError {
    runtime_error(format!(
        "A_Admin.SetEncounterJournalMapEncounters: invalid {key}"
    ))
}
