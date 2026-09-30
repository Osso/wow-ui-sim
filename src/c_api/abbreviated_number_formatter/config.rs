use super::model::{Breakpoint, sort_breakpoints, validate_number};
use crate::lua_api::methods::{
    create_string, create_table, table_get, table_set_num, table_set_static, val_to_string,
};
use rilua::table_security::{is_secret_value, unwrap_secret, wrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

fn read_value(state: &LuaState, value: Val, secret: &mut bool) -> LuaResult<Val> {
    *secret |= is_secret_value(state, value);
    unwrap_secret(state, value)
}

fn read_field(state: &mut LuaState, table: Val, key: &str, secret: &mut bool) -> LuaResult<Val> {
    let value = table_get(state, table, key);
    read_value(state, value, secret)
}

fn read_number(state: &mut LuaState, table: Val, key: &str, secret: &mut bool) -> LuaResult<f64> {
    let Val::Num(value) = read_field(state, table, key, secret)? else {
        return Err(runtime_error(format!(
            "abbreviation {key} must be a number"
        )));
    };
    validate_number(value)
}

pub(super) fn read_row(
    state: &mut LuaState,
    value: Val,
    inherited_secret: bool,
) -> LuaResult<Breakpoint> {
    let mut secret = inherited_secret;
    let table = read_value(state, value, &mut secret)?;
    let Val::Table(reference) = table else {
        return Err(runtime_error("abbreviation breakpoint must be a table"));
    };
    rilua::table_security::check_table_access(state, reference, None)?;
    let threshold = read_number(state, table, "breakpoint", &mut secret)?;
    let significand = read_number(state, table, "significandDivisor", &mut secret)?;
    let fraction = read_number(state, table, "fractionDivisor", &mut secret)?;
    if !(significand * fraction).is_finite() {
        return Err(runtime_error("abbreviation divisor product must be finite"));
    }
    let abbreviation = read_field(state, table, "abbreviation", &mut secret)?;
    if !matches!(abbreviation, Val::Str(_)) {
        return Err(runtime_error("abbreviation must be a string"));
    }
    let abbreviation = val_to_string(state, abbreviation)
        .ok_or_else(|| runtime_error("abbreviation must be UTF-8"))?;
    let global = match read_field(state, table, "abbreviationIsGlobal", &mut secret)? {
        Val::Nil | Val::Bool(true) => true,
        Val::Bool(false) => false,
        _ => return Err(runtime_error("abbreviationIsGlobal must be a boolean")),
    };
    Ok(Breakpoint {
        threshold,
        abbreviation,
        significand,
        fraction,
        global,
        secret,
    })
}

pub(super) fn read_rows(state: &mut LuaState, value: Val) -> LuaResult<Vec<Breakpoint>> {
    let mut secret = false;
    let value = read_value(state, value, &mut secret)?;
    let Val::Table(reference) = value else {
        return Err(runtime_error(
            "abbreviation breakpoints must be an array table",
        ));
    };
    rilua::table_security::check_table_access(state, reference, None)?;
    let values = read_array(state, reference)?;
    let rows = values
        .into_iter()
        .map(|value| read_row(state, value, secret))
        .collect::<LuaResult<_>>()?;
    sort_breakpoints(rows)
}

fn read_array(
    state: &LuaState,
    reference: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<Vec<Val>> {
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("abbreviation configuration is unavailable"))?;
    let mut entries = Vec::new();
    let mut key = Val::Nil;
    while let Some((next, value)) = table.next(key, &state.gc.string_arena)? {
        let Val::Num(index) = next else {
            return Err(runtime_error(
                "abbreviation configuration must be a dense array",
            ));
        };
        entries.push((index, value));
        key = next;
    }
    entries.sort_by(|left, right| left.0.total_cmp(&right.0));
    if entries
        .iter()
        .enumerate()
        .any(|(index, entry)| entry.0 != (index + 1) as f64)
    {
        return Err(runtime_error(
            "abbreviation configuration must be a dense array",
        ));
    }
    Ok(entries.into_iter().map(|(_, value)| value).collect())
}

pub(super) fn write_rows(state: &mut LuaState, rows: &[Breakpoint]) -> LuaResult<Val> {
    let result = create_table(state);
    state.push(result);
    for (index, row) in rows.iter().enumerate() {
        let output = create_table(state);
        state.push(output);
        write_row(state, output, row)?;
        table_set_num(state, result, (index + 1) as i64, output);
        state.pop();
    }
    state.pop();
    Ok(result)
}

fn write_row(state: &mut LuaState, output: Val, row: &Breakpoint) -> LuaResult<()> {
    let abbreviation = create_string(state, &row.abbreviation);
    state.push(abbreviation);
    for (key, value) in [
        ("breakpoint", Val::Num(row.threshold)),
        ("abbreviation", abbreviation),
        ("significandDivisor", Val::Num(row.significand)),
        ("fractionDivisor", Val::Num(row.fraction)),
        ("abbreviationIsGlobal", Val::Bool(row.global)),
    ] {
        let value = if row.secret {
            wrap_secret(state, value)?
        } else {
            value
        };
        table_set_static(state, output, key, value);
    }
    state.pop();
    Ok(())
}
