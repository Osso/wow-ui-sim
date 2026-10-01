//! Documented 12.0.5 duration Format* consumers and trusted NumericFormatter dispatch.
use super::{install_method, require_duration};
use crate::c_api::{abbreviated_number_formatter, seconds_formatter};
use crate::lua_api::methods::{
    call_function_state, create_string, registry_get, table_get, val_to_string,
};
use crate::lua_bridge::stack_val;
use rilua::table_security::{
    is_secret_value, unwrap_secret, wrap_host_secret_number, wrap_host_secret_string,
};
use rilua::vm::{closure::Closure, state::LuaState};
use rilua::{LuaResult, Val, runtime_error};

pub(super) const METHOD_NAMES: &[&str] = &[
    "FormatElapsedDuration",
    "FormatRemainingDuration",
    "FormatTotalDuration",
];

#[derive(Clone, Copy)]
pub(crate) enum FormatterKind {
    Abbreviated,
    #[cfg(feature = "numeric-rule-formatters")]
    NumericRule,
    Seconds,
}

pub(crate) fn identify_formatter(state: &mut LuaState, object: Val) -> LuaResult<FormatterKind> {
    if abbreviated_number_formatter::is_formatter(state, object) {
        return Ok(FormatterKind::Abbreviated);
    }
    #[cfg(feature = "numeric-rule-formatters")]
    if crate::c_api::numeric_rule_formatter::is_formatter(state, object) {
        return Ok(FormatterKind::NumericRule);
    }
    if seconds_formatter::is_formatter(state, object)? {
        return Ok(FormatterKind::Seconds);
    }
    Err(runtime_error("expected a known NumericFormatter object"))
}

fn query_duration(
    state: &mut LuaState,
    object: Val,
    modifier: Val,
    getter: &str,
) -> LuaResult<f64> {
    // Never pass a decoded modifier to a Lua override on the duration table.
    let methods = registry_get(state, super::METHODS_KEY);
    let method = table_get(state, methods, getter);
    let Val::Function(reference) = method else {
        return Err(runtime_error(
            "duration formatting requires a native core getter",
        ));
    };
    let native = matches!(state.gc.closures.get(reference), Some(Closure::Rust(_)));
    if !native {
        return Err(runtime_error(
            "duration formatting requires a native core getter",
        ));
    }
    match call_function_state(state, method, &[object, modifier])? {
        Val::Num(number) => Ok(number),
        _ => Err(runtime_error("duration query must return a number")),
    }
}

fn native_text(state: &mut LuaState, text: &str, secret: bool) -> Val {
    if secret {
        wrap_host_secret_string(state, text)
    } else {
        create_string(state, text)
    }
}

pub(crate) fn format_number(
    state: &mut LuaState,
    kind: FormatterKind,
    object: Val,
    input: Val,
) -> LuaResult<Val> {
    match kind {
        FormatterKind::Abbreviated => {
            let (number, secret) = read_number(state, input)?;
            let (text, configuration_secret) =
                abbreviated_number_formatter::format_value(state, object, number)?;
            Ok(native_text(state, &text, secret || configuration_secret))
        }
        #[cfg(feature = "numeric-rule-formatters")]
        FormatterKind::NumericRule => {
            let (number, secret) = read_number(state, input)?;
            let result = crate::c_api::numeric_rule_formatter::format_value(state, object, number)?;
            let text = val_to_string(state, result)
                .ok_or_else(|| runtime_error("NumericFormatter must return a string"))?;
            Ok(native_text(state, &text, secret))
        }
        // This private Lua implementation receives the opaque input unchanged.
        FormatterKind::Seconds => seconds_formatter::call_format_number(state, object, input),
    }
}

fn read_number(state: &LuaState, input: Val) -> LuaResult<(f64, bool)> {
    match unwrap_secret(state, input)? {
        Val::Num(number) => Ok((number, is_secret_value(state, input))),
        _ => Err(runtime_error("NumericFormatter requires a number")),
    }
}

fn format_duration(state: &mut LuaState, getter: &str) -> LuaResult<u32> {
    let object = require_duration(state, 1)?;
    let formatter_input = stack_val(state, 2);
    let modifier_input = stack_val(state, 3);
    let secret = super::duration_has_secret_values(state, object)
        || is_secret_value(state, formatter_input)
        || is_secret_value(state, modifier_input);
    let formatter = unwrap_secret(state, formatter_input)?;
    let kind = identify_formatter(state, formatter)?;
    let modifier = unwrap_secret(state, modifier_input)?;
    let number = query_duration(state, object, modifier, getter)?;
    let input = if secret {
        wrap_host_secret_number(state, number)
    } else {
        Val::Num(number)
    };
    state.push(input);
    let output = format_number(state, kind, formatter, input)?;
    state.pop();
    state.push(output);
    Ok(1)
}

pub(super) fn register(state: &mut LuaState, methods: Val) {
    for (name, getter) in [
        ("FormatElapsedDuration", "GetElapsedDuration"),
        ("FormatRemainingDuration", "GetRemainingDuration"),
        ("FormatTotalDuration", "GetTotalDuration"),
    ] {
        let method: rilua::RustFn = match getter {
            "GetElapsedDuration" => |s| format_duration(s, "GetElapsedDuration"),
            "GetRemainingDuration" => |s| format_duration(s, "GetRemainingDuration"),
            _ => |s| format_duration(s, "GetTotalDuration"),
        };
        install_method(state, methods, name, name, method);
    }
}
