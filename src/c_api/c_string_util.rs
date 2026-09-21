//! C_StringUtil: string escaping helpers used by Blizzard diagnostics.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_string, create_string_bytes, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
#[cfg(feature = "retail-12-0-0")]
use rilua::runtime_error;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use super::helpers::ensure_namespace;

#[cfg(feature = "retail-12-0-0")]
mod hyperlinks;

pub fn register_c_string_util(state: &mut LuaState) -> LuaResult<()> {
    let c_string_util_ref = ensure_namespace(state, "C_StringUtil")?;
    table_set_rust_fn_static(
        state,
        c_string_util_ref,
        "EscapeQuotedCodes",
        c_string_util_escape_quoted_codes,
    )?;
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        table_set_rust_fn_static(
            state,
            c_string_util_ref,
            "EscapeLuaFormatString",
            c_string_util_escape_lua_format_string,
        )?;
        table_set_rust_fn_static(
            state,
            c_string_util_ref,
            "EscapeLuaPatterns",
            c_string_util_escape_lua_patterns,
        )?;
        table_set_rust_fn_static(
            state,
            c_string_util_ref,
            "WrapString",
            c_string_util_wrap_string,
        )?;
    }
    #[cfg(feature = "retail-12-0-0")]
    table_set_rust_fn_static(
        state,
        c_string_util_ref,
        "RemoveContiguousSpaces",
        c_string_util_remove_contiguous_spaces,
    )?;
    #[cfg(feature = "retail-12-0-0")]
    table_set_rust_fn_static(
        state,
        c_string_util_ref,
        "TruncateWhenZero",
        c_string_util_truncate_when_zero,
    )?;
    #[cfg(feature = "retail-12-0-0")]
    table_set_rust_fn_static(
        state,
        c_string_util_ref,
        "StripHyperlinks",
        c_string_util_strip_hyperlinks,
    )?;
    #[cfg(feature = "numeric-rule-formatters")]
    super::numeric_rule_formatter::register(state, c_string_util_ref)?;
    Ok(())
}

pub fn c_string_util_escape_quoted_codes(state: &mut LuaState) -> LuaResult<u32> {
    let Some(input) = val_to_string(state, stack_val(state, 1)) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let escaped = input.replace('|', "||");
    let escaped_value = create_string(state, &escaped);
    state.push(escaped_value);
    Ok(1)
}

pub fn c_string_util_escape_lua_format_string(state: &mut LuaState) -> LuaResult<u32> {
    transform_string_bytes(state, escape_lua_format_string)
}

pub fn c_string_util_escape_lua_patterns(state: &mut LuaState) -> LuaResult<u32> {
    transform_string_bytes(state, escape_lua_patterns)
}

pub fn c_string_util_wrap_string(state: &mut LuaState) -> LuaResult<u32> {
    let Some(infix) = string_bytes(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    if infix.is_empty() {
        let empty_value = create_string_bytes(state, b"");
        state.push(empty_value);
        return Ok(1);
    }

    let prefix = string_bytes(state, 2).unwrap_or_default();
    let suffix = string_bytes(state, 3).unwrap_or_default();
    let mut wrapped = Vec::with_capacity(prefix.len() + infix.len() + suffix.len());
    wrapped.extend(prefix);
    wrapped.extend(infix);
    wrapped.extend(suffix);
    let wrapped_value = create_string_bytes(state, &wrapped);
    state.push(wrapped_value);
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn c_string_util_remove_contiguous_spaces(state: &mut LuaState) -> LuaResult<u32> {
    let input = string_bytes(state, 1)
        .ok_or_else(|| runtime_error("RemoveContiguousSpaces expects string text"))?;
    let Val::Num(limit) = stack_val(state, 2) else {
        return Err(runtime_error(
            "RemoveContiguousSpaces expects numeric maxAllowedSpaces",
        ));
    };
    // Simulator policy; native invalid-limit validation remains unverified.
    let valid_limit = limit.is_finite() && limit >= 0.0 && limit.fract() == 0.0;
    if !valid_limit {
        return Err(runtime_error(
            "maxAllowedSpaces must be a finite nonnegative integer",
        ));
    }
    let max_spaces = limit.min(input.len() as f64) as usize;
    let trimmed = truncate_ascii_space_runs(&input, max_spaces);
    let result = create_string_bytes(state, &trimmed);
    state.push(result);
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn c_string_util_truncate_when_zero(state: &mut LuaState) -> LuaResult<u32> {
    // Strict numeric validation is simulator policy, not native conformance.
    let Val::Num(number) = stack_val(state, 1) else {
        return Err(runtime_error("TruncateWhenZero expects a number"));
    };
    if !number.is_finite() {
        return Err(runtime_error("TruncateWhenZero expects a finite number"));
    }
    let integer = number.floor();
    let text = if integer == 0.0 {
        String::new()
    } else {
        format!("{integer:.0}")
    };
    let result = create_string(state, &text);
    state.push(result);
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn c_string_util_strip_hyperlinks(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Str(_) = stack_val(state, 1) else {
        return Err(runtime_error("StripHyperlinks expects string text"));
    };
    let text = val_to_string(state, stack_val(state, 1))
        .ok_or_else(|| runtime_error("StripHyperlinks expects UTF-8 text"))?;
    // Ordinary Lua truthiness is simulator policy for optional flags.
    let enabled = |index| !matches!(stack_val(state, index), Val::Nil | Val::Bool(false));
    let options = hyperlinks::StripHyperlinksOptions {
        maintain_color: enabled(2),
        maintain_brackets: enabled(3),
        strip_newlines: enabled(4),
        maintain_atlases: enabled(5),
        maintain_textures: enabled(6),
    };
    let stripped = hyperlinks::strip_hyperlinks(&text, &options);
    let result = create_string(state, &stripped);
    state.push(result);
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn truncate_ascii_space_runs(input: &[u8], max_spaces: usize) -> Vec<u8> {
    let mut spaces = 0;
    input
        .iter()
        .copied()
        .filter(|&byte| {
            if byte != b' ' {
                spaces = 0;
                return true;
            }
            if spaces == max_spaces {
                return false;
            }
            spaces += 1;
            true
        })
        .collect()
}

fn transform_string_bytes(state: &mut LuaState, transform: fn(&[u8]) -> Vec<u8>) -> LuaResult<u32> {
    let Some(input) = string_bytes(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let transformed = transform(&input);
    let transformed_value = create_string_bytes(state, &transformed);
    state.push(transformed_value);
    Ok(1)
}

fn string_bytes(state: &LuaState, index: i32) -> Option<Vec<u8>> {
    let Val::Str(string_ref) = stack_val(state, index) else {
        return None;
    };
    state
        .gc
        .string_arena
        .get(string_ref)
        .map(|string| string.data().to_vec())
}

fn escape_lua_format_string(input: &[u8]) -> Vec<u8> {
    escape_ascii_characters(input, &[b'%'])
}

fn escape_lua_patterns(input: &[u8]) -> Vec<u8> {
    escape_ascii_characters(input, b"^$()%.[]*+-?")
}

fn escape_ascii_characters(input: &[u8], characters: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(input.len());
    for &byte in input {
        if characters.contains(&byte) {
            escaped.push(b'%');
        }
        escaped.push(byte);
    }
    escaped
}
