//! Project frame references into the destination Lua environment's object table.

use super::methods::{
    call_function_state, native_frame_id_from_val, registry_get, registry_set, table_get,
};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};
use std::ops::Range;

const FORBIDDEN_TABLES: &str = "__wow_forbidden_object_tables";
const FORBIDDEN_CONSTRUCTOR: &str = "__wow_forbidden_object_constructor";

pub(crate) fn install(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    let constructor = table_get(state, Val::Table(state.global), "GetForbiddenObjectTable");
    if !matches!(constructor, Val::Function(_)) {
        return Err(runtime_error(
            "GetForbiddenObjectTable is missing during bootstrap",
        ));
    }
    registry_set(state, FORBIDDEN_CONSTRUCTOR, constructor);
    state.environment_transfer_hook = Some(transfer_script_objects);
    Ok(())
}

fn transfer_script_objects(
    state: &mut LuaState,
    source: GcRef<Table>,
    target: GcRef<Table>,
    values: Range<usize>,
) -> LuaResult<()> {
    let Val::Table(secure) = registry_get(state, "__secureenv") else {
        return Err(runtime_error(
            "secure object transfer has no secure environment",
        ));
    };
    if (source == secure) == (target == secure) {
        return Ok(());
    }
    for slot in values {
        let value = state.stack_get(slot);
        let converted = if target == secure {
            project_forbidden(state, value)?
        } else {
            project_public(state, value)
        };
        state.stack_set(slot, converted);
    }
    Ok(())
}

fn project_forbidden(state: &mut LuaState, value: Val) -> LuaResult<Val> {
    if native_frame_id_from_val(state, value).is_none() {
        return Ok(value);
    }
    let constructor = registry_get(state, FORBIDDEN_CONSTRUCTOR);
    call_function_state(state, constructor, &[value])
}

fn project_public(state: &mut LuaState, value: Val) -> Val {
    let Val::Table(table) = value else {
        return value;
    };
    let key = state.gc.intern_string_static(b"__wowPublicObject");
    let public = state
        .gc
        .tables
        .get(table)
        .map(|entry| entry.raw_get(Val::Str(key), &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if native_frame_id_from_val(state, public).is_none() {
        return value;
    }
    let Val::Table(tables) = registry_get(state, FORBIDDEN_TABLES) else {
        return value;
    };
    let registered = state
        .gc
        .tables
        .get(tables)
        .map(|entries| entries.raw_get(public, &state.gc.string_arena));
    if registered == Some(value) {
        public
    } else {
        value
    }
}
