//! Edit Mode frame helper methods used by Blizzard's managed-frame code.

use crate::lua_api::methods::table_get_static;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "IsEditModeDragging", is_edit_mode_dragging)?;
    table_set_rust_fn_static(state, mt, "IsInitialized", is_initialized)?;
    table_set_rust_fn_static(state, mt, "IsInDefaultPosition", is_in_default_position)?;
    table_set_rust_fn_static(
        state,
        mt,
        "IsSystemSettingDefault",
        is_system_setting_default,
    )?;
    Ok(())
}

fn is_edit_mode_dragging(state: &mut LuaState) -> LuaResult<u32> {
    let frame = Val::from_stack(state, 1)?;
    let dragging = truthy_field(state, frame, "isEditModeDragging");
    state.push(Val::Bool(dragging));
    Ok(1)
}

fn is_initialized(state: &mut LuaState) -> LuaResult<u32> {
    let frame = Val::from_stack(state, 1)?;
    let has_layout_info = !matches!(table_get_static(state, frame, "layoutInfo"), Val::Nil);
    let has_system_info = !matches!(table_get_static(state, frame, "systemInfo"), Val::Nil);
    state.push(Val::Bool(has_layout_info || has_system_info));
    Ok(1)
}

/// A frame without edit-mode `systemInfo` counts as in its default position:
/// the client has this method only on edit-mode systems, and its callers
/// outside the mixin (AlertFrames.lua:416, EditModeUtil.lua:22) treat a
/// missing method that way. Every simulator frame carries the method, and
/// answering false for plain frames kept `AlertContainerMixin:UpdateAnchors`
/// from advancing past the text-to-speech button, so it and the Quick Join
/// toast overlapped.
fn is_in_default_position(state: &mut LuaState) -> LuaResult<u32> {
    let frame = Val::from_stack(state, 1)?;
    let system_info = table_get_static(state, frame, "systemInfo");
    let is_default = match system_info {
        Val::Table(_) => matches!(
            table_get_static(state, system_info, "isInDefaultPosition"),
            Val::Bool(true)
        ),
        _ => true,
    };
    state.push(Val::Bool(is_default));
    Ok(1)
}

fn is_system_setting_default(state: &mut LuaState) -> LuaResult<u32> {
    let frame = Val::from_stack(state, 1)?;
    let setting = Val::from_stack(state, 2)?;
    let system_info = table_get_static(state, frame, "systemInfo");
    let settings = table_get_static(state, system_info, "settings");
    let is_default = setting_value(state, settings, setting).is_some_and(default_setting_value);
    state.push(Val::Bool(is_default));
    Ok(1)
}

fn setting_value(state: &mut LuaState, settings: Val, requested_setting: Val) -> Option<Val> {
    let Val::Table(settings_ref) = settings else {
        return None;
    };
    let entries = state
        .gc
        .tables
        .get(settings_ref)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();

    entries.into_iter().find_map(|entry| {
        setting_entry_matches(state, entry, requested_setting)
            .then(|| table_get_static(state, entry, "value"))
    })
}

fn setting_entry_matches(state: &mut LuaState, entry: Val, requested_setting: Val) -> bool {
    table_get_static(state, entry, "setting") == requested_setting
}

fn default_setting_value(value: Val) -> bool {
    match value {
        Val::Nil => true,
        Val::Bool(value) => value,
        Val::Num(value) => value == 0.0,
        _ => false,
    }
}

fn truthy_field(state: &mut LuaState, table: Val, key: &'static str) -> bool {
    !matches!(
        table_get_static(state, table, key),
        Val::Nil | Val::Bool(false)
    )
}
