//! Row411's inferred localized decimal model; natural truncation is an explicit guess.
//! Native natural semantics, arg1 permissions, and result secrecy remain unknown.

use crate::c_api::intl_native::{NumberStyle, format_number};
use crate::lua_bridge::stack_val;
use rilua::table_security::is_secret_value;
use rilua::vm::state::{LuaState, MAXCALLS};
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

fn break_up_large_numbers(state: &mut LuaState) -> LuaResult<u32> {
    let number = stack_val(state, 1);
    let natural = stack_val(state, 2);
    // Authenticate both arguments before any payload/type/locale access. Arg1
    // rejection is conservative local policy; arg2 is the NeverSecret boundary.
    for (index, value) in [(1, number), (2, natural)] {
        if is_secret_value(state, value) {
            return Err(runtime_error(format!(
                "BreakUpLargeNumbers argument #{index} must not be secret"
            )));
        }
    }
    let Val::Num(number) = number else {
        return Err(runtime_error(
            "BreakUpLargeNumbers argument #1 must be a finite number",
        ));
    };
    if !number.is_finite() {
        return Err(runtime_error(
            "BreakUpLargeNumbers argument #1 must be a finite number",
        ));
    }
    let natural = match natural {
        Val::Nil => false,
        Val::Bool(value) => value,
        _ => {
            return Err(runtime_error(
                "BreakUpLargeNumbers argument #2 must be a boolean or nil",
            ));
        }
    };
    let locale = read_current_locale(state)?;
    // EXPLICIT GUESS, not native-verified: natural truncates toward zero.
    let number = if natural { number.trunc() } else { number };
    let result = format_number(&locale, number, NumberStyle::Decimal)
        .map_err(|error| runtime_error(format!("BreakUpLargeNumbers: {error}")))?;
    let result = state.gc.intern_string(result.as_bytes());
    state.push(Val::Str(result));
    Ok(1)
}

fn read_current_locale(state: &mut LuaState) -> LuaResult<String> {
    let getter = crate::c_api::global_val(state, "GetLocale");
    let top = state.top;
    let base = state.base;
    let ci = state.ci;
    state.push(getter);
    // The getter and its one result stay stack-rooted through the Lua call and
    // string copy; caller arguments remain rooted below this temporary slot.
    let result = state.call_function(top, 1).and_then(|()| {
        let value = state.stack_get(top);
        if is_secret_value(state, value) {
            return Err(runtime_error(
                "BreakUpLargeNumbers GetLocale must return a public string",
            ));
        }
        let Val::Str(reference) = value else {
            return Err(runtime_error(
                "BreakUpLargeNumbers GetLocale must return a public string",
            ));
        };
        let bytes = state
            .gc
            .string_arena
            .get(reference)
            .ok_or_else(|| runtime_error("BreakUpLargeNumbers locale is unavailable"))?
            .data();
        let locale = std::str::from_utf8(bytes)
            .map_err(|_| runtime_error("BreakUpLargeNumbers locale must be UTF-8"))?;
        Ok(normalize_wow_locale(locale))
    });
    state.top = top;
    state.base = base;
    state.ci = ci;
    if ci < MAXCALLS {
        state.ci_overflow = false;
    }
    result
}

fn normalize_wow_locale(locale: &str) -> String {
    // Generic compact language/region normalization, not a locale catalogue.
    if locale.len() == 4 && locale.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        format!("{}_{}", &locale[..2], &locale[2..])
    } else {
        locale.to_owned()
    }
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "BreakUpLargeNumbers", break_up_large_numbers)
}
