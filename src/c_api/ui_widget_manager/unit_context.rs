//! User-approved simulator default; native registration behavior is not established.
use std::collections::{HashMap, HashSet};

use super::{WidgetId, WidgetVisualization};

use crate::lua_api::methods::{borrow_state_mut, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProcessingUnit {
    Token(String),
    Guid(String),
}

#[derive(Debug, Default)]
pub struct WidgetUnitContext {
    pub registrations: HashSet<(String, bool)>,
    pub processing_unit: Option<ProcessingUnit>,
    pub unit_visualizations: HashMap<(ProcessingUnit, WidgetId), WidgetVisualization>,
    pub unit_widget_ids: HashSet<WidgetId>,
}

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "RegisterUnitForWidgetUpdates",
        register_unit,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "UnregisterUnitForWidgetUpdates",
        unregister_unit,
    )?;
    table_set_rust_fn_static(state, namespace, "SetProcessingUnit", set_processing_unit)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetProcessingUnitGuid",
        set_processing_guid,
    )
}

fn read_unit(state: &LuaState, nilable: bool) -> LuaResult<Option<String>> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Nil if nilable => Ok(None),
        Val::Str(_) => val_to_string(state, value)
            .map(Some)
            .ok_or_else(|| runtime_error("unit must be UTF-8")),
        _ => Err(runtime_error("unit must be a string")),
    }
}

fn read_registration(state: &LuaState) -> LuaResult<(String, bool)> {
    let token = read_unit(state, false)?.ok_or_else(|| runtime_error("unit must be a string"))?;
    let guid = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let is_guid = match guid {
        Val::Nil => false,
        Val::Bool(value) => value,
        _ => return Err(runtime_error("isGuid must be a boolean")),
    };
    Ok((token, is_guid))
}

fn register_unit(state: &mut LuaState) -> LuaResult<u32> {
    let key = read_registration(state)?;
    borrow_state_mut(state)?
        .ui_widget_inputs
        .unit_context
        .registrations
        .insert(key);
    Ok(0)
}

fn unregister_unit(state: &mut LuaState) -> LuaResult<u32> {
    let key = read_registration(state)?;
    borrow_state_mut(state)?
        .ui_widget_inputs
        .unit_context
        .registrations
        .remove(&key);
    Ok(0)
}

fn set_processing_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_unit(state, true)?.map(ProcessingUnit::Token);
    borrow_state_mut(state)?
        .ui_widget_inputs
        .unit_context
        .processing_unit = unit;
    Ok(0)
}

fn set_processing_guid(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_unit(state, true)?.map(ProcessingUnit::Guid);
    borrow_state_mut(state)?
        .ui_widget_inputs
        .unit_context
        .processing_unit = unit;
    Ok(0)
}
