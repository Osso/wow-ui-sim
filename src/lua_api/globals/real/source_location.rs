//! Retail source-location exclusions and capture for `CreateTexture`.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, registry_get};
use crate::lua_bridge::{stack_val, table_set_rust_fn_unpaired};
use rilua::table_security::{can_access_secrets, is_secret_value, unwrap_secret};
use rilua::vm::closure::Closure;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const ADD_SOURCE_LOCATION_EXCLUDE: &str = "AddSourceLocationExclude";
const ADD_SOURCE_LOCATION_EXCLUDE_FLAGS: u8 = 3;
const ADD_SOURCE_LOCATION_EXCLUDE_USAGE: &str = "Usage: AddSourceLocationExclude(fileName)";
const UNKNOWN_SOURCE_FILE: &str = "UnknownFile";
const SOURCE_LINE_MODULUS: u32 = 65_536;

/// Publish separate ordinary and secure function objects for Retail flag 3.
pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(());
    }

    let (publish_global, publish_secure) =
        crate::lua_api::globals::security::native_registration_destinations(
            ADD_SOURCE_LOCATION_EXCLUDE_FLAGS,
        );
    if publish_global {
        LuaApiMut::register_function(
            lua,
            ADD_SOURCE_LOCATION_EXCLUDE,
            add_source_location_exclude,
        )?;
    }
    if publish_secure {
        let state = lua.state_mut();
        if let Val::Table(secure_environment) = registry_get(state, "__secureenv") {
            table_set_rust_fn_unpaired(
                state,
                secure_environment,
                ADD_SOURCE_LOCATION_EXCLUDE,
                add_source_location_exclude,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn capture_texture_source_location(
    state: &LuaState,
    object_type_name: Option<&str>,
) -> LuaResult<Option<String>> {
    if object_type_name.is_some() || !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(None);
    }

    let sim = borrow_state(state)?;
    Ok(Some(select_texture_source_location(
        state,
        &sim.source_location_exclusions,
    )))
}

fn add_source_location_exclude(state: &mut LuaState) -> LuaResult<u32> {
    let exclusion = read_exclusion_argument(state)?;
    if !exclusion.is_empty() {
        borrow_state_mut(state)?
            .source_location_exclusions
            .push(exclusion);
    }
    Ok(0)
}

fn read_exclusion_argument(state: &LuaState) -> LuaResult<Vec<u8>> {
    let value = read_untainted_exclusion_argument(state)?;
    let bytes = convert_exclusion_value(state, value)?;
    let length = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    Ok(bytes[..length].to_vec())
}

fn read_untainted_exclusion_argument(state: &LuaState) -> LuaResult<Val> {
    let value = stack_val(state, 1);
    if is_secret_value(state, value) && !can_access_secrets(state) {
        return Err(exclusion_argument_error(
            "Secret values are only allowed during untainted execution for this argument.",
        ));
    }
    unwrap_secret(state, value).map_err(|_| {
        exclusion_argument_error(
            "Secret values are only allowed during untainted execution for this argument.",
        )
    })
}

fn convert_exclusion_value(state: &LuaState, value: Val) -> LuaResult<Vec<u8>> {
    match value {
        Val::Str(string) => state
            .gc
            .string_arena
            .get(string)
            .map(|string| string.data().to_vec())
            .ok_or_else(|| runtime_error(ADD_SOURCE_LOCATION_EXCLUDE_USAGE)),
        Val::Num(_) => Ok(format!("{value}").into_bytes()),
        _ => Err(exclusion_argument_error(ADD_SOURCE_LOCATION_EXCLUDE_USAGE)),
    }
}

fn exclusion_argument_error(reason: &str) -> rilua::LuaError {
    crate::c_api::helpers::bad_argument(1, ADD_SOURCE_LOCATION_EXCLUDE, reason)
}

/// Select the first non-excluded Lua file frame at a texture's creation site.
pub(crate) fn select_texture_source_location(state: &LuaState, exclusions: &[Vec<u8>]) -> String {
    let frame_count = state.call_stack.len().min(state.ci.saturating_add(1));
    for frame_index in (0..frame_count).rev() {
        let Some((source, line)) = file_source_for_frame(state, frame_index) else {
            continue;
        };
        if exclusions
            .iter()
            .any(|exclusion| source.as_bytes().ends_with(exclusion))
        {
            continue;
        }

        let Some(file) = source.strip_prefix('@') else {
            continue;
        };
        if file.is_empty() {
            continue;
        }
        let line = if line > 0 {
            line % SOURCE_LINE_MODULUS
        } else {
            0
        };
        return format!("{file}:{line}");
    }

    format!("{UNKNOWN_SOURCE_FILE}:0")
}

fn file_source_for_frame(state: &LuaState, frame_index: usize) -> Option<(String, u32)> {
    let frame = state.call_stack.get(frame_index)?;
    if !frame.is_lua {
        return None;
    }
    let Val::Function(closure_ref) = state.stack_get(frame.func) else {
        return None;
    };
    let Closure::Lua(closure) = state.gc.closures.get(closure_ref)? else {
        return None;
    };

    let source = closure.proto.source.split('\0').next().unwrap_or_default();
    if !source.starts_with('@') {
        return None;
    }
    let line_info = &closure.proto.line_info;
    let line = if frame.saved_pc > 0 && frame.saved_pc <= line_info.len() {
        line_info[frame.saved_pc - 1]
    } else {
        line_info.first().copied().unwrap_or(0)
    };
    Some((source.to_owned(), line))
}
