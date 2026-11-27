//! PTR string operations on raw Lua bytes, not Unicode text.

use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const DEFAULT_TRIM_BYTES: &[u8] = b" \r\n\t";

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "CalculateStringEditDistance", edit_distance)?;
    }
    let state = lua.state_mut();
    let key = state.gc.intern_string_static(b"string");
    let globals = state
        .gc
        .tables
        .get(state.global)
        .ok_or_else(|| runtime_error("global table has been collected"))?;
    let Val::Table(table) = globals.get_str(key, &state.gc.string_arena) else {
        return Err(runtime_error("string library is unavailable"));
    };
    table_set_rust_fn_static(state, table, "contains", contains)?;
    table_set_rust_fn_static(state, table, "startswith", starts_with)?;
    table_set_rust_fn_static(state, table, "endswith", ends_with)?;
    table_set_rust_fn_static(state, table, "ltrim", trim_left)?;
    table_set_rust_fn_static(state, table, "rtrim", trim_right)?;
    Ok(())
}

fn edit_distance(state: &mut LuaState) -> LuaResult<u32> {
    let first = optional_string_bytes(state, 1);
    let second = optional_string_bytes(state, 2);
    let distance = match (first, second) {
        (Some(first), Some(second)) => byte_edit_distance(first, second),
        _ => 0,
    };
    state.push(Val::Num(distance as f64));
    Ok(1)
}

fn optional_string_bytes(state: &LuaState, index: i32) -> Option<&[u8]> {
    let Val::Str(reference) = stack_val(state, index) else {
        return None;
    };
    state
        .gc
        .string_arena
        .get(reference)
        .map(|string| string.data())
}

fn byte_edit_distance(first: &[u8], second: &[u8]) -> usize {
    let mut previous = (0..=second.len()).collect::<Vec<_>>();
    for (row, first_byte) in first.iter().enumerate() {
        let mut current = vec![row + 1; second.len() + 1];
        for (column, second_byte) in second.iter().enumerate() {
            let cost = usize::from(first_byte != second_byte);
            current[column + 1] = (previous[column + 1] + 1)
                .min(current[column] + 1)
                .min(previous[column] + cost);
        }
        previous = current;
    }
    previous[second.len()]
}

fn argument_bytes(state: &LuaState, index: i32) -> LuaResult<&[u8]> {
    let value = stack_val(state, index);
    let Val::Str(reference) = value else {
        return Err(runtime_error(format!(
            "bad argument #{index} (string expected, got {})",
            value.type_name()
        )));
    };
    state
        .gc
        .string_arena
        .get(reference)
        .map(|string| string.data())
        .ok_or_else(|| runtime_error("string argument has been collected"))
}

fn contains(state: &mut LuaState) -> LuaResult<u32> {
    let found = contains_bytes(argument_bytes(state, 1)?, argument_bytes(state, 2)?);
    state.push(Val::Bool(found));
    Ok(1)
}

fn starts_with(state: &mut LuaState) -> LuaResult<u32> {
    let found = argument_bytes(state, 1)?.starts_with(argument_bytes(state, 2)?);
    state.push(Val::Bool(found));
    Ok(1)
}

fn ends_with(state: &mut LuaState) -> LuaResult<u32> {
    let found = argument_bytes(state, 1)?.ends_with(argument_bytes(state, 2)?);
    state.push(Val::Bool(found));
    Ok(1)
}

// KMP avoids quadratic rescanning on repeated prefixes, without decoding bytes.
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    let prefixes = matching_prefix_lengths(needle);
    let mut matched = 0;
    for &byte in haystack {
        while matched > 0 && needle[matched] != byte {
            matched = prefixes[matched - 1];
        }
        if needle[matched] == byte {
            matched += 1;
        }
        if matched == needle.len() {
            return true;
        }
    }
    false
}

fn matching_prefix_lengths(needle: &[u8]) -> Vec<usize> {
    let mut prefixes = vec![0; needle.len()];
    let mut matched = 0;
    for index in 1..needle.len() {
        while matched > 0 && needle[matched] != needle[index] {
            matched = prefixes[matched - 1];
        }
        if needle[matched] == needle[index] {
            matched += 1;
        }
        prefixes[index] = matched;
    }
    prefixes
}

fn trim_left(state: &mut LuaState) -> LuaResult<u32> {
    trim_bytes(state, true)
}

fn trim_right(state: &mut LuaState) -> LuaResult<u32> {
    trim_bytes(state, false)
}

fn trim_bytes(state: &mut LuaState, left: bool) -> LuaResult<u32> {
    let bytes = argument_bytes(state, 1)?;
    let characters = if stack_val(state, 2).is_nil() {
        DEFAULT_TRIM_BYTES
    } else {
        argument_bytes(state, 2)?
    };
    let mut members = [false; 256];
    for &byte in characters {
        members[usize::from(byte)] = true;
    }
    let retained = if left {
        let start = bytes
            .iter()
            .position(|byte| !members[usize::from(*byte)])
            .unwrap_or(bytes.len());
        &bytes[start..]
    } else {
        let end = bytes
            .iter()
            .rposition(|byte| !members[usize::from(*byte)])
            .map_or(0, |index| index + 1);
        &bytes[..end]
    }
    .to_vec();
    let result = state.gc.intern_string(&retained);
    state.push(Val::Str(result));
    Ok(1)
}
