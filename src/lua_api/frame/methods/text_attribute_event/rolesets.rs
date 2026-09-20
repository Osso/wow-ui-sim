//! Roleset membership shared by retail 12.1 and Forever.
use crate::lua_api::methods::{
    create_table, frame_id_from_stack, get_or_create_frame_fields, table_get, table_set,
    val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "AddRoleset", add_roleset)?;
    table_set_rust_fn_static(state, table, "GetRolesetNames", get_roleset_names)?;
    table_set_rust_fn_static(state, table, "RemoveRoleset", remove_roleset)?;
    table_set_rust_fn_static(state, table, "SetRolesets", set_rolesets)
}

fn frame_fields(state: &mut LuaState) -> LuaResult<Val> {
    let id = frame_id_from_stack(state, 1)?;
    Ok(get_or_create_frame_fields(state, id))
}

fn roleset_table(state: &mut LuaState, fields: Val) -> Val {
    let rolesets = table_get(state, fields, "__rolesets");
    if matches!(rolesets, Val::Table(_)) {
        return rolesets;
    }
    let rolesets = create_table(state);
    table_set(state, fields, "__rolesets", rolesets);
    rolesets
}

fn add_roleset(state: &mut LuaState) -> LuaResult<u32> {
    let fields = frame_fields(state)?;
    let rolesets = roleset_table(state, fields);
    if let Some(name) = val_to_string(state, stack_val(state, 2)) {
        table_set(state, rolesets, &name, Val::Bool(true));
    }
    Ok(0)
}

fn set_rolesets(state: &mut LuaState) -> LuaResult<u32> {
    let fields = frame_fields(state)?;
    let rolesets = create_table(state);
    let count = state.top.saturating_sub(state.base);
    for index in 2..=count {
        if let Some(name) = val_to_string(state, stack_val(state, index as i32)) {
            table_set(state, rolesets, &name, Val::Bool(true));
        }
    }
    table_set(state, fields, "__rolesets", rolesets);
    Ok(0)
}

fn remove_roleset(state: &mut LuaState) -> LuaResult<u32> {
    let fields = frame_fields(state)?;
    let rolesets = roleset_table(state, fields);
    if let Some(name) = val_to_string(state, stack_val(state, 2)) {
        table_set(state, rolesets, &name, Val::Nil);
    }
    Ok(0)
}

fn get_roleset_names(state: &mut LuaState) -> LuaResult<u32> {
    let fields = frame_fields(state)?;
    let rolesets = roleset_table(state, fields);
    state.push(rolesets);
    Ok(1)
}
