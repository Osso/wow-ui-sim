//! Typed 12.0.5 abbreviated formatters implementing the documented NumericFormatter interface.
mod config;
mod model;

use crate::lua_api::methods::{call_function_state, table_get, table_set_static, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use model::Breakpoint;
use rilua::table_security::{is_secret_value, unwrap_secret};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table, value::Userdata};
use rilua::{LuaResult, Val, runtime_error};

const METATABLE: &str = "AbbreviatedNumberFormatter";

#[derive(Clone, Debug)]
struct AbbreviatedNumberFormatter {
    rows: Vec<Breakpoint>,
}

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "CreateAbbreviatedNumberFormatter", create)
}

fn push_formatter(state: &mut LuaState, object: AbbreviatedNumberFormatter) -> LuaResult<u32> {
    let metatable = rilua::stdlib::new_metatable(state, METATABLE)?;
    if table_get(state, Val::Table(metatable), "__index") == Val::Nil {
        table_set_static(
            state,
            Val::Table(metatable),
            "__index",
            Val::Table(metatable),
        );
        for (name, method) in [
            ("AddBreakpoint", add_breakpoint as rilua::RustFn),
            ("FormatNumber", format_number),
            ("ClearBreakpoints", clear_breakpoints),
            ("Copy", copy),
            ("GetBreakpoints", get_breakpoints),
            ("ResetBreakpoints", reset_breakpoints),
            ("SetBreakpoints", set_breakpoints),
        ] {
            table_set_rust_fn_static(state, metatable, name, method)?;
        }
    }
    let reference = state
        .gc
        .alloc_userdata(Userdata::with_metatable(Box::new(object), metatable));
    state.push(Val::Userdata(reference));
    Ok(1)
}

fn default_rows(state: &mut LuaState) -> LuaResult<Vec<Breakpoint>> {
    let getter = super::global_val(state, "GetLocale");
    let locale = call_function_state(state, getter, &[])?;
    let locale = val_to_string(state, locale)
        .ok_or_else(|| runtime_error("GetLocale must return a string"))?;
    match locale.as_str() {
        "enUS" | "enGB" => Ok(model::english_defaults()),
        _ => Err(runtime_error(format!(
            "abbreviation defaults are not modeled for locale {locale}"
        ))),
    }
}

fn create(state: &mut LuaState) -> LuaResult<u32> {
    let rows = default_rows(state)?;
    push_formatter(state, AbbreviatedNumberFormatter { rows })
}

fn formatter(state: &LuaState, value: Val) -> LuaResult<&AbbreviatedNumberFormatter> {
    let Val::Userdata(reference) = value else {
        return Err(runtime_error("expected AbbreviatedNumberFormatter"));
    };
    state
        .gc
        .userdata
        .get(reference)
        .and_then(|object| object.downcast_ref())
        .ok_or_else(|| runtime_error("incompatible AbbreviatedNumberFormatter receiver"))
}

/// Test actual host-owned identity without consulting Lua methods or metatables.
pub(crate) fn is_formatter(state: &LuaState, object: Val) -> bool {
    formatter(state, object).is_ok()
}

fn formatter_mut(state: &mut LuaState) -> LuaResult<&mut AbbreviatedNumberFormatter> {
    let Val::Userdata(reference) = stack_val(state, 1) else {
        return Err(runtime_error("expected AbbreviatedNumberFormatter"));
    };
    state
        .gc
        .userdata
        .get_mut(reference)
        .and_then(|object| object.downcast_mut())
        .ok_or_else(|| runtime_error("incompatible AbbreviatedNumberFormatter receiver"))
}

fn set_breakpoints(state: &mut LuaState) -> LuaResult<u32> {
    formatter(state, stack_val(state, 1))?;
    let rows = config::read_rows(state, stack_val(state, 2))?;
    formatter_mut(state)?.rows = rows;
    Ok(0)
}

fn add_breakpoint(state: &mut LuaState) -> LuaResult<u32> {
    let mut rows = formatter(state, stack_val(state, 1))?.rows.clone();
    rows.push(config::read_row(state, stack_val(state, 2), false)?);
    let rows = model::sort_breakpoints(rows)?;
    formatter_mut(state)?.rows = rows;
    Ok(0)
}

fn clear_breakpoints(state: &mut LuaState) -> LuaResult<u32> {
    formatter_mut(state)?.rows.clear();
    Ok(0)
}

fn reset_breakpoints(state: &mut LuaState) -> LuaResult<u32> {
    formatter(state, stack_val(state, 1))?;
    let rows = default_rows(state)?;
    formatter_mut(state)?.rows = rows;
    Ok(0)
}

fn copy(state: &mut LuaState) -> LuaResult<u32> {
    let copied = formatter(state, stack_val(state, 1))?.clone();
    push_formatter(state, copied)
}

fn get_breakpoints(state: &mut LuaState) -> LuaResult<u32> {
    let rows = formatter(state, stack_val(state, 1))?.rows.clone();
    let result = config::write_rows(state, &rows)?;
    state.push(result);
    Ok(1)
}

fn format_number(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 2);
    let secret = is_secret_value(state, input);
    let Val::Num(number) = unwrap_secret(state, input)? else {
        return Err(runtime_error("abbreviated formatter requires a number"));
    };
    let (text, formatter_secret) = format_value(state, stack_val(state, 1), number)?;
    let value = crate::lua_api::methods::create_string(state, &text);
    state.push(value);
    let result = if secret || formatter_secret {
        rilua::table_security::wrap_secret(state, value)?
    } else {
        value
    };
    state.pop();
    state.push(result);
    Ok(1)
}

/// Host-only NumericFormatter dispatch. Neither receiver nor decoded inputs reach Lua callbacks.
pub(crate) fn format_value(
    state: &mut LuaState,
    object: Val,
    number: f64,
) -> LuaResult<(String, bool)> {
    let rows = &formatter(state, object)?.rows;
    let secret = rows.iter().any(|row| row.secret);
    if secret && !rilua::api::state_is_secure(state) {
        return Err(runtime_error(
            "secret formatter configuration requires an untainted caller",
        ));
    }
    let end = rows.partition_point(|row| row.threshold <= number.abs());
    let row = end
        .checked_sub(1)
        .and_then(|index| rows.get(index))
        .cloned();
    let text = model::render_number(number, row.as_ref())?;
    let Some(row) = row else {
        return Ok((text, secret));
    };
    if !row.global {
        return Ok((format!("{text}{}", row.abbreviation), secret));
    }
    let abbreviation = super::global_val(state, &row.abbreviation);
    let secret = secret || is_secret_value(state, abbreviation);
    let abbreviation = unwrap_secret(state, abbreviation)?;
    let Val::Str(_) = abbreviation else {
        return Err(runtime_error(format!(
            "abbreviation global {} must be a string",
            row.abbreviation
        )));
    };
    let suffix = val_to_string(state, abbreviation)
        .ok_or_else(|| runtime_error("abbreviation global must be UTF-8"))?;
    Ok((format!("{text}{suffix}"), secret))
}
