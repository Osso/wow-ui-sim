//! VERIFIED (Forever 1.60.1.70707) / INFERRED (Retail 12.1.5.70688) R88fb: supplied raid-lock rows.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

const INPUT_ERROR: &str =
    "A_Admin.SetCompletedEncounters: expected an array of raid-lock row tables";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RaidLockRow {
    pub(crate) map_id: u32,
    pub(crate) difficulty_id: u32,
    pub(crate) completed_encounter_ids: Vec<u32>,
}

const REDIRECT_INPUT_ERROR: &str =
    "A_Admin.SetRaidDifficultyRedirects: expected an array of redirect row tables";

/// Supplied difficulty redirect: lockouts for `difficulty_id` on `map_id`
/// are shared with `redirected_difficulty_id`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DifficultyRedirect {
    pub(crate) map_id: u32,
    pub(crate) difficulty_id: u32,
    pub(crate) redirected_difficulty_id: u32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_RaidLocks")?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
        // Forever 70707 RaidLocksDocumentation.lua:11-24.
        table_set_rust_fn_static(
            state,
            namespace,
            "GetRedirectedDifficultyID",
            get_redirected_difficulty_id,
        )?;
    }
    table_set_rust_fn_static(
        state,
        namespace,
        "IsEncounterComplete",
        is_encounter_complete,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    let builder = builder.set_function("SetCompletedEncounters", set_completed_encounters)?;
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return Ok(builder);
    }
    builder.set_function("SetRaidDifficultyRedirects", set_difficulty_redirects)
}

/// INFERRED (R-request pending): a map/difficulty pair without a supplied
/// redirect keeps its own lockout and returns `difficultyID` unchanged.
fn get_redirected_difficulty_id(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = u32::from_stack(state, 1)?;
    let difficulty_id = u32::from_stack(state, 2)?;
    let redirected = borrow_state(state)?
        .raid_difficulty_redirects
        .iter()
        .find(|row| row.map_id == map_id && row.difficulty_id == difficulty_id)
        .map_or(difficulty_id, |row| row.redirected_difficulty_id);
    state.push(Val::Num(f64::from(redirected)));
    Ok(1)
}

fn set_difficulty_redirects(state: &mut LuaState) -> LuaResult<u32> {
    let rows = read_rows(state, stack_val(state, 1), REDIRECT_INPUT_ERROR)?
        .into_iter()
        .map(|row| read_redirect_row(state, row))
        .collect::<LuaResult<Vec<_>>>()?;
    borrow_state_mut(state)?.raid_difficulty_redirects = rows;
    Ok(0)
}

fn read_redirect_row(state: &mut LuaState, value: Val) -> LuaResult<DifficultyRedirect> {
    let Val::Table(table) = value else {
        return Err(runtime_error(REDIRECT_INPUT_ERROR));
    };
    let mut field = |key: &str| {
        read_numeric_id(table_get(state, Val::Table(table), key), key).map_err(|_| {
            runtime_error(format!("A_Admin.SetRaidDifficultyRedirects: invalid {key}"))
        })
    };
    Ok(DifficultyRedirect {
        map_id: field("mapID")?,
        difficulty_id: field("difficultyID")?,
        redirected_difficulty_id: field("redirectedDifficultyID")?,
    })
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
        .find(|row| {
            row.map_id == map_id
                && difficulty_id.is_none_or(|difficulty| row.difficulty_id == difficulty)
        })
        .is_some_and(|row| row.completed_encounter_ids.contains(&encounter_id));
    state.push(Val::Bool(complete));
    Ok(1)
}

fn set_completed_encounters(state: &mut LuaState) -> LuaResult<u32> {
    let rows = read_raid_lock_rows(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.completed_encounters = rows;
    Ok(0)
}

fn read_raid_lock_rows(state: &mut LuaState, value: Val) -> LuaResult<Vec<RaidLockRow>> {
    read_rows(state, value, INPUT_ERROR)?
        .into_iter()
        .map(|row| read_raid_lock_row(state, row))
        .collect()
}

fn read_rows(state: &LuaState, value: Val, error: &str) -> LuaResult<Vec<Val>> {
    let Val::Table(array) = value else {
        return Err(runtime_error(error));
    };
    let table = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| runtime_error(error))?;
    let length = table.len(&state.gc.string_arena);
    Ok((1..=length)
        .map(|index| table.get_int(index as i64))
        .collect())
}

fn read_raid_lock_row(state: &mut LuaState, value: Val) -> LuaResult<RaidLockRow> {
    let Val::Table(table) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    Ok(RaidLockRow {
        map_id: read_id(state, table, "mapID")?,
        difficulty_id: read_id(state, table, "difficultyID")?,
        completed_encounter_ids: read_ids(state, table, "completedEncounterIDs")?,
    })
}

fn read_ids(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<Vec<u32>> {
    let value = table_get(state, Val::Table(table), key);
    let Val::Table(array) = value else {
        return Err(runtime_error(format!(
            "A_Admin.SetCompletedEncounters: invalid {key}"
        )));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| runtime_error(format!("A_Admin.SetCompletedEncounters: invalid {key}")))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let value = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            read_numeric_id(value, key)
        })
        .collect()
}

fn read_id(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<u32> {
    read_numeric_id(table_get(state, Val::Table(table), key), key)
}

fn read_numeric_id(value: Val, key: &str) -> LuaResult<u32> {
    match value {
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
