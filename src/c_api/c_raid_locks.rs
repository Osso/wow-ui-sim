//! INFERRED R88fb (native evidence pending): supplied saved-instance lock state.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

const INPUT_ERROR: &str =
    "A_Admin.SetCompletedEncounters: expected an array of completed encounter tables";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CompletedEncounter {
    pub(crate) map_id: u32,
    pub(crate) encounter_id: u32,
    pub(crate) difficulty_id: u32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_RaidLocks")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsEncounterComplete",
        is_encounter_complete,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetCompletedEncounters", set_completed_encounters)
}

fn is_encounter_complete(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = u32::from_stack(state, 1)?;
    let encounter_id = u32::from_stack(state, 2)?;
    let difficulty_id = match stack_val(state, 3) {
        Val::Nil => None,
        _ => Some(u32::from_stack(state, 3)?),
    };
    let complete = borrow_state(state)?
        .completed_encounters
        .iter()
        .any(|lock| {
            lock.map_id == map_id
                && lock.encounter_id == encounter_id
                && difficulty_id.is_none_or(|difficulty| lock.difficulty_id == difficulty)
        });
    state.push(Val::Bool(complete));
    Ok(1)
}

fn set_completed_encounters(state: &mut LuaState) -> LuaResult<u32> {
    let encounters = read_completed_encounters(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.completed_encounters = encounters;
    Ok(0)
}

fn read_completed_encounters(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<Vec<CompletedEncounter>> {
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
            read_completed_encounter(state, row)
        })
        .collect()
}

fn read_completed_encounter(state: &mut LuaState, value: Val) -> LuaResult<CompletedEncounter> {
    let Val::Table(table) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    Ok(CompletedEncounter {
        map_id: read_id(state, table, "mapID")?,
        encounter_id: read_id(state, table, "encounterID")?,
        difficulty_id: read_id(state, table, "difficultyID")?,
    })
}

fn read_id(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<u32> {
    match table_get(state, Val::Table(table), key) {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=u32::MAX as f64).contains(&value) =>
        {
            Ok(value as u32)
        }
        _ => Err(runtime_error(format!(
            "A_Admin.SetCompletedEncounters: invalid {key}"
        ))),
    }
}
