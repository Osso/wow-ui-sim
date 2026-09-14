//! Explicit set-filter state; native defaults and filtering effects are unmodeled.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_TransmogSets")?;
    table_set_rust_fn_static(state, namespace, "GetSetsFilter", get_sets_filter)?;
    table_set_rust_fn_static(state, namespace, "SetSetsFilter", set_sets_filter)
}

fn filter_index(state: &LuaState) -> LuaResult<i32> {
    let Val::Num(index) = stack_val(state, 1) else {
        return Err(rilua::runtime_error("SetsFilter requires a numeric index"));
    };
    Ok(index as i32)
}

fn get_sets_filter(state: &mut LuaState) -> LuaResult<u32> {
    let index = filter_index(state)?;
    let value = borrow_state(state)?
        .transmog_set_filters
        .get(&index)
        .copied();
    let Some(value) = value else {
        // Simulator unset policy; native defaults remain unverified.
        return Ok(0);
    };
    state.push(Val::Bool(value));
    Ok(1)
}

fn set_sets_filter(state: &mut LuaState) -> LuaResult<u32> {
    let index = filter_index(state)?;
    let Val::Bool(value) = stack_val(state, 2) else {
        return Err(rilua::runtime_error("SetSetsFilter requires a boolean"));
    };
    borrow_state_mut(state)?
        .transmog_set_filters
        .insert(index, value);
    Ok(0)
}
