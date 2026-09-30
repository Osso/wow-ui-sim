//! Documented 12.0.5 duration Format* consumers, using typed host formatter dispatch.
use super::{install_method, require_duration};
use crate::lua_api::methods::{call_function_state, create_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::{is_secret_value, unwrap_secret, wrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) const METHOD_NAMES: &[&str] = &[
    "FormatElapsedDuration",
    "FormatRemainingDuration",
    "FormatTotalDuration",
];

fn format_duration(state: &mut LuaState, getter: &str) -> LuaResult<u32> {
    let object = require_duration(state, 1)?;
    let formatter_input = stack_val(state, 2);
    let modifier_input = stack_val(state, 3);
    let secret = super::duration_has_secret_values(state, object)
        || is_secret_value(state, formatter_input)
        || is_secret_value(state, modifier_input);
    let formatter = unwrap_secret(state, formatter_input)?;
    let modifier = unwrap_secret(state, modifier_input)?;
    let key = create_string(state, getter);
    let method = state.gettable(object, key)?;
    let value = call_function_state(state, method, &[object, modifier])?;
    let Val::Num(number) = value else {
        return Err(runtime_error("duration query must return a number"));
    };
    let (text, formatter_secret) =
        crate::c_api::abbreviated_number_formatter::format_value(state, formatter, number)?;
    let value = create_string(state, &text);
    state.push(value);
    let result = if secret || formatter_secret {
        wrap_secret(state, value)?
    } else {
        value
    };
    state.pop();
    state.push(result);
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
