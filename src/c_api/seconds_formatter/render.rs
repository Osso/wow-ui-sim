//! Private bootstrap callback: no public C_* renderer is installed.
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

use crate::c_api::intl_native::{DurationPart, format_duration_units};
use crate::lua_api::methods::call_function_state;
use crate::lua_bridge::stack_val;

pub(super) fn callback(state: &mut LuaState) -> Val {
    let closure = Closure::Rust(RustClosure::new(render, "SecondsFormatter unit renderer"));
    Val::Function(state.gc.alloc_closure(closure))
}

fn number(state: &mut LuaState, table: Val, index: i32) -> LuaResult<f64> {
    match state.gettable(table, Val::Num(f64::from(index)))? {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(runtime_error(
            "duration renderer expects finite numeric parts",
        )),
    }
}

fn integer(value: f64, minimum: i32, maximum: i32) -> LuaResult<i32> {
    let integral = value.fract() == 0.0;
    let in_range = value >= f64::from(minimum) && value <= f64::from(maximum);
    if !integral || !in_range {
        return Err(runtime_error(
            "duration renderer has an invalid interval or precision",
        ));
    }
    Ok(value as i32)
}

fn read_parts(state: &mut LuaState) -> LuaResult<Vec<DurationPart>> {
    let table @ Val::Table(reference) = stack_val(state, 1) else {
        return Err(runtime_error("duration renderer expects a parts table"));
    };
    let count = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("duration parts have been collected"))?
        .len(&state.gc.string_arena);
    if !(1..=4).contains(&count) {
        return Err(runtime_error(
            "duration renderer requires one to four parts",
        ));
    }
    (1..=count)
        .map(|index| {
            let part = state.gettable(table, Val::Num(index as f64))?;
            let value = number(state, part, 1)?;
            let unit = integer(number(state, part, 2)?, 0, 3)?;
            let fraction_digits = integer(number(state, part, 3)?, 0, 3)?;
            Ok(DurationPart {
                value,
                unit,
                fraction_digits,
            })
        })
        .collect()
}

fn current_locale(state: &mut LuaState) -> LuaResult<String> {
    let getter = crate::c_api::global_val(state, "GetLocale");
    let Val::Str(reference) = call_function_state(state, getter, &[])? else {
        return Err(runtime_error(
            "duration formatting requires a string GetLocale result",
        ));
    };
    let bytes = state
        .gc
        .string_arena
        .get(reference)
        .ok_or_else(|| runtime_error("duration locale has been collected"))?
        .data();
    crate::c_api::c_intl::parse_locale(bytes, "duration formatting")
        .map(|locale| locale.to_string())
}

fn render(state: &mut LuaState) -> LuaResult<u32> {
    let parts = read_parts(state)?;
    let Val::Num(width) = stack_val(state, 2) else {
        return Err(runtime_error("duration renderer requires a numeric width"));
    };
    let width = integer(width, 0, 2)?;
    let locale = current_locale(state)?;
    let text = format_duration_units(&locale, &parts, width)
        .map_err(|error| runtime_error(error.to_string()))?;
    let result = state.gc.intern_string(text.as_bytes());
    state.push(Val::Str(result));
    Ok(1)
}
