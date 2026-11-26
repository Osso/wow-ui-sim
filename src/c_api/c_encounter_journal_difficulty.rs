//! Encounter Journal difficulty queries backed by selected-instance map data.

use std::sync::OnceLock;

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const MAP_DIFFICULTY_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/db2/retail-12.1.5.70961/MapDifficulty.csv"
));

#[derive(Clone, Copy)]
struct MapDifficulty {
    map_id: u32,
    difficulty_id: i16,
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function(
        "SetEncounterJournalDifficultyMask",
        set_instance_difficulty_mask,
    )
}

fn set_instance_difficulty_mask(state: &mut LuaState) -> LuaResult<u32> {
    let instance_id = u32::from_stack(state, 1)?;
    let mask = u64::try_from(i64::from_stack(state, 2)?)
        .map_err(|_| runtime_error("A_Admin.SetEncounterJournalDifficultyMask: invalid mask"))?;
    borrow_state_mut(state)?
        .encounter_journal
        .instance_difficulty_masks
        .insert(instance_id, mask);
    Ok(0)
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

pub(crate) fn get_base_difficulty_id(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (Retail 12.1.5.70688, R88gb): identity mapping outside 233 and 250.
    let difficulty_id = i32::from_stack(state, 1)? as i16;
    let base_difficulty_id = base_difficulty_id(i32::from(difficulty_id));
    state.push(Val::Num(f64::from(base_difficulty_id)));
    Ok(1)
}

fn instance_has_difficulty_id(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (Retail 12.1.5.70688 / Forever 1.60.1.70707, R88gb): no instance argument.
    let difficulty_id = i32::from_stack(state, 1)?;
    let has_difficulty_id = selected_instance_has_difficulty(state, difficulty_id)?;
    state.push(Val::Bool(has_difficulty_id));
    Ok(1)
}

pub(crate) fn ej_is_valid_instance_difficulty(state: &mut LuaState) -> LuaResult<u32> {
    // UNKNOWN (Retail 12.1.5.70688 / Forever 1.60.1.70707, R88gb): narrowing is not proved.
    // UNKNOWN (same builds, R88gb.1): no mask producer or named DB2 field is identified.
    let difficulty_id = i32::from_stack(state, 1).unwrap_or(0);
    let base_id = base_difficulty_id(difficulty_id);
    let mask = selected_instance_mask(state)?;
    let mask_matches = (1..=63).contains(&base_id) && mask & (1_u64 << (base_id - 1)) != 0;
    let is_valid = mask_matches && selected_instance_has_difficulty(state, difficulty_id)?;
    state.push(Val::Bool(is_valid));
    Ok(1)
}

fn selected_instance_has_difficulty(state: &LuaState, difficulty_id: i32) -> LuaResult<bool> {
    let instance_id = borrow_state(state)?.encounter_journal.current_instance;
    let Some(instance) = crate::encounter_journal_data::instance_by_id(instance_id) else {
        return Ok(false);
    };
    Ok(map_difficulties()
        .iter()
        .any(|row| row.map_id == instance.map_id && i32::from(row.difficulty_id) == difficulty_id))
}

fn selected_instance_mask(state: &LuaState) -> LuaResult<u64> {
    let journal = &borrow_state(state)?.encounter_journal;
    Ok(journal
        .instance_difficulty_masks
        .get(&journal.current_instance)
        .copied()
        .unwrap_or(0))
}

fn base_difficulty_id(difficulty_id: i32) -> i32 {
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
                    difficulty_id: parse_field::<i16>(&fields, 2),
                    map_id: parse_field::<u32>(&fields, 11),
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
