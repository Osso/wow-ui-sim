//! Encounter Journal difficulty queries backed by selected-instance map data.

use std::sync::OnceLock;

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// INFERRED R88gb (native evidence pending): Forever uses Retail rows when its map IDs match.
const MAP_DIFFICULTY_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/db2/retail-12.1.5.70961/MapDifficulty.csv"
));

#[derive(Clone, Copy)]
struct MapDifficulty {
    map_id: u32,
    difficulty_id: i16,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_EncounterJournal")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetBaseDifficultyID",
        get_base_difficulty_id,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "InstanceHasDifficultyID",
        instance_has_difficulty_id,
    )
}

fn get_base_difficulty_id(state: &mut LuaState) -> LuaResult<u32> {
    let difficulty_id = i32::from_stack(state, 1)? as i16;
    let base_difficulty_id = base_difficulty_id(difficulty_id);
    state.push(Val::Num(f64::from(base_difficulty_id)));
    Ok(1)
}

fn instance_has_difficulty_id(state: &mut LuaState) -> LuaResult<u32> {
    let difficulty_id = i32::from_stack(state, 1)? as i16;
    let has_difficulty_id = selected_instance_has_difficulty(state, difficulty_id)?;
    state.push(Val::Bool(has_difficulty_id));
    Ok(1)
}

pub(crate) fn ej_is_valid_instance_difficulty(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED R88gb.1 (native evidence pending): map rows supply the mask bits.
    // The extra native difficulty-8 predicate remains UNKNOWN and is not modeled.
    let difficulty_id = i32::from_stack(state, 1).unwrap_or(0) as i16;
    let base_id = base_difficulty_id(difficulty_id);
    let is_valid = (1..=63).contains(&base_id)
        && selected_instance_has_difficulty(state, difficulty_id)?
        && selected_instance_mask(state)? & (1_u64 << (base_id - 1)) != 0;
    state.push(Val::Bool(is_valid));
    Ok(1)
}

fn selected_instance_has_difficulty(state: &LuaState, difficulty_id: i16) -> LuaResult<bool> {
    let instance_id = borrow_state(state)?.encounter_journal.current_instance;
    let Some(instance) = crate::encounter_journal_data::instance_by_id(instance_id) else {
        return Ok(false);
    };
    Ok(map_difficulties()
        .iter()
        .any(|row| row.map_id == instance.map_id as u32 && row.difficulty_id == difficulty_id))
}

fn selected_instance_mask(state: &LuaState) -> LuaResult<u64> {
    let instance_id = borrow_state(state)?.encounter_journal.current_instance;
    let Some(instance) = crate::encounter_journal_data::instance_by_id(instance_id) else {
        return Ok(0);
    };
    Ok(map_difficulties()
        .iter()
        .filter(|row| row.map_id == instance.map_id as u32)
        .filter_map(|row| {
            let base_id = base_difficulty_id(row.difficulty_id);
            (1..=63)
                .contains(&base_id)
                .then_some(1_u64 << (base_id - 1))
        })
        .fold(0, |mask, bit| mask | bit))
}

fn base_difficulty_id(difficulty_id: i16) -> i16 {
    match difficulty_id {
        233 => 16,
        250 => 17,
        other => other,
    }
}

fn map_difficulties() -> &'static [MapDifficulty] {
    static ROWS: OnceLock<Vec<MapDifficulty>> = OnceLock::new();
    ROWS.get_or_init(|| {
        MAP_DIFFICULTY_CSV
            .lines()
            .skip(1)
            .filter(|line| !line.is_empty())
            .map(|line| {
                let fields = csv_fields(line);
                MapDifficulty {
                    difficulty_id: parse_field(&fields, 2),
                    map_id: parse_field(&fields, 11),
                }
            })
            .collect()
    })
}

fn parse_field<T>(fields: &[&str], index: usize) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    fields[index]
        .parse()
        .unwrap_or_else(|error| panic!("invalid Encounter Journal DB2 field {index}: {error:?}"))
}

fn csv_fields(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut fields = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' if quoted && bytes.get(index + 1) == Some(&b'"') => index += 1,
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                fields.push(&line[start..index]);
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    fields.push(&line[start..]);
    fields
}
