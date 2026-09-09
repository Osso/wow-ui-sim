//! PTR options adapter. Lifecycle and validation assumptions: docs/specs/create-frame-with-options.md.
use super::helpers::table_get_str;
use super::{
    CreateFrameArgs, InitialFrameFlags, create_frame_from_args, resolve_runtime_widget_type,
};
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::collections::BTreeMap;

pub(super) fn create_frame_with_options(state: &mut LuaState) -> LuaResult<u32> {
    let options = stack_val(state, 1);
    if !matches!(options, Val::Table(_)) {
        return Err(runtime_error(
            "CreateFrameWithOptions: options must be a table",
        ));
    }
    let args = decode_options(state, options)?;
    create_frame_from_args(state, args)
}

fn decode_options(state: &mut LuaState, options: Val) -> LuaResult<CreateFrameArgs> {
    let frame_type = read_string(state, options, "frameType")?
        .ok_or_else(|| invalid_field("frameType", "required string"))?;
    let widget_type = resolve_runtime_widget_type(&frame_type)?;
    let name = read_string(state, options, "name")?;
    let parent_val = table_get_str(state, options, "parent");
    let inherits = read_inherits(state, options)?;
    let id = read_id(table_get_str(state, options, "id"))?;
    let initial_flags = InitialFrameFlags {
        hidden: read_bool(table_get_str(state, options, "hidden"), "hidden")?,
        forbidden: read_bool(table_get_str(state, options, "forbidden"), "forbidden")?,
    };
    Ok(CreateFrameArgs {
        frame_type,
        widget_type,
        name,
        parent_val,
        default_parent_allowed: false,
        inherits,
        id,
        template_initializer: Val::Nil,
        initial_flags: Some(initial_flags),
    })
}

fn invalid_field(field: &str, expected: &str) -> rilua::LuaError {
    runtime_error(format!(
        "CreateFrameWithOptions: {field} must be {expected}"
    ))
}

fn read_string(state: &mut LuaState, options: Val, key: &str) -> LuaResult<Option<String>> {
    let value = table_get_str(state, options, key);
    match value {
        Val::Nil => Ok(None),
        Val::Str(_) => Ok(val_to_string(state, value)),
        _ => Err(invalid_field(key, "a string or nil")),
    }
}

fn read_bool(value: Val, key: &str) -> LuaResult<bool> {
    match value {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(invalid_field(key, "a boolean or nil")),
    }
}

fn read_id(value: Val) -> LuaResult<Option<i32>> {
    match value {
        Val::Nil => Ok(None),
        Val::Num(id)
            if id.is_finite()
                && id.fract() == 0.0
                && id >= i32::MIN as f64
                && id <= i32::MAX as f64 =>
        {
            Ok(Some(id as i32))
        }
        _ => Err(invalid_field("id", "a finite 32-bit integer or nil")),
    }
}

fn read_inherits(state: &mut LuaState, options: Val) -> LuaResult<Option<String>> {
    let value = table_get_str(state, options, "inherits");
    if value == Val::Nil {
        return Ok(None);
    }
    let Val::Table(reference) = value else {
        return Err(invalid_field("inherits", "an ordered string array or nil"));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| invalid_field("inherits", "a live table"))?;
    let mut entries: Vec<_> = table
        .array_slice()
        .iter()
        .enumerate()
        .filter(|(_, value)| **value != Val::Nil)
        .map(|(index, value)| (Val::Num((index + 1) as f64), *value))
        .collect();
    entries.extend(table.hash_entries());
    let ordered = decode_template_entries(state, entries)?;
    Ok(Some(ordered.into_values().collect::<Vec<_>>().join(",")))
}

fn decode_template_entries(
    state: &LuaState,
    entries: Vec<(Val, Val)>,
) -> LuaResult<BTreeMap<usize, String>> {
    let mut ordered = BTreeMap::new();
    for (key, value) in entries {
        if value == Val::Nil {
            continue;
        }
        let index = template_index(key)?;
        let Val::Str(_) = value else {
            return Err(invalid_field("inherits", "an ordered string array"));
        };
        let name =
            val_to_string(state, value).ok_or_else(|| invalid_field("inherits", "live strings"))?;
        if name.trim().is_empty() || name.contains(',') {
            return Err(invalid_field(
                "inherits",
                "individual nonempty template names",
            ));
        }
        ordered.insert(index, name);
    }
    if ordered.keys().copied().ne(1..=ordered.len()) {
        return Err(invalid_field("inherits", "a dense one-based array"));
    }
    Ok(ordered)
}

fn template_index(key: Val) -> LuaResult<usize> {
    match key {
        Val::Num(index)
            if index.is_finite()
                && index.fract() == 0.0
                && index >= 1.0
                && index <= i32::MAX as f64 =>
        {
            Ok(index as usize)
        }
        _ => Err(invalid_field("inherits", "a dense one-based array")),
    }
}
