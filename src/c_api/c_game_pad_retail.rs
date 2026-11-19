//! Retail gamepad queries backed by empty device input and the existing CVar.

use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::helpers::ensure_namespace(state, "C_GamePad")?;
    table_set_rust_fn_static(state, namespace, "GetAllDeviceIDs", get_all_device_ids)?;
    table_set_rust_fn_static(state, namespace, "GetDeviceRawState", get_device_raw_state)?;
    table_set_rust_fn_static(state, namespace, "IsEnabled", is_enabled)
}

fn get_all_device_ids(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (simulator mapping): no physical-device input means an empty list.
    let device_ids = create_table(state);
    state.push(device_ids);
    Ok(1)
}

fn get_device_raw_state(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (simulator mapping): no supplied device means no raw state.
    state.push(Val::Nil);
    Ok(1)
}

fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (simulator mapping): project the existing CVar, not hardware state.
    let enabled = borrow_state(state)?.cvars.get_bool("GamePadEnable");
    state.push(Val::Bool(enabled));
    Ok(1)
}
