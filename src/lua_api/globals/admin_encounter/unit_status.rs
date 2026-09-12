//! Explicit encounter-status snapshots supplied by simulator callers.

use crate::lua_api::methods::{
    create_table, create_table_with_fields, table_get, table_get_num, table_set_num,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn copy_status_input(state: &mut LuaState) -> LuaResult<Val> {
    let rows = read_status_rows(state, stack_val(state, 5))?;
    let output = create_table(state);
    let Val::Table(output_ref) = output else {
        unreachable!("create_table returns a table")
    };
    for (index, [id, name, health]) in rows.into_iter().enumerate() {
        let row = create_table_with_fields(
            state,
            &[
                ("creatureID", id),
                ("creatureName", name),
                ("remainingHealthPercent", health),
            ],
        );
        table_set_num(state, output_ref, (index + 1) as f64, row);
    }
    Ok(output)
}

fn read_status_rows(state: &mut LuaState, input: Val) -> LuaResult<Vec<[Val; 3]>> {
    if input == Val::Nil {
        return Ok(Vec::new());
    }
    let Val::Table(reference) = input else {
        return Err(runtime_error(
            "SimulateBossKill: status must be a dense array",
        ));
    };
    let table = state.gc.tables.get(reference).expect("live input table");
    let count = table
        .array_slice()
        .iter()
        .filter(|v| **v != Val::Nil)
        .count()
        + table
            .hash_entries()
            .into_iter()
            .filter(|(_, v)| *v != Val::Nil)
            .count();
    (1..=count)
        .map(|index| {
            let row = table_get_num(state, reference, index as f64);
            read_status_row(state, row, index)
        })
        .collect()
}

fn read_status_row(state: &mut LuaState, row: Val, index: usize) -> LuaResult<[Val; 3]> {
    if !matches!(row, Val::Table(_)) {
        return Err(runtime_error(format!(
            "SimulateBossKill: status[{index}] must be a table in a dense array"
        )));
    }
    let id = table_get(state, row, "creatureID");
    let name = table_get(state, row, "creatureName");
    let health = table_get(state, row, "remainingHealthPercent");
    let valid_id = matches!(id, Val::Num(n) if n.is_finite() && n > 0.0 && n.fract() == 0.0);
    if !valid_id {
        return Err(invalid_field(index, "creatureID", "positive integer"));
    }
    if !matches!(name, Val::Str(_)) {
        return Err(invalid_field(index, "creatureName", "string"));
    }
    let valid_health = matches!(health, Val::Num(n) if n.is_finite() && (0.0..=100.0).contains(&n));
    if !valid_health {
        return Err(invalid_field(
            index,
            "remainingHealthPercent",
            "number from 0 to 100",
        ));
    }
    Ok([id, name, health])
}

fn invalid_field(index: usize, field: &str, expected: &str) -> rilua::LuaError {
    runtime_error(format!(
        "SimulateBossKill: status[{index}].{field} must be a {expected}"
    ))
}
