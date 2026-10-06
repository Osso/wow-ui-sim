//! Roleset membership shared by retail 12.1 and Forever.
//!
//! Tags live on `Frame::rolesets`; every change reevaluates the frame against
//! the active `C_Roleset` filters.
use crate::c_api::c_roleset::{push_names, split_roleset_list, update_frame_rolesets};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::LuaResult;
#[cfg(feature = "retail-12-1-0")]
use rilua::Val;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

/// `GetRolesetNames` result for a frame without tags.
const ROLELESS: &str = "roleless";

pub(super) fn register(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table, "AddRoleset", add_roleset)?;
    table_set_rust_fn_static(state, table, "GetRolesetNames", get_roleset_names)?;
    table_set_rust_fn_static(state, table, "RemoveRoleset", remove_roleset)?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(state, table, "IsRolesetFiltered", is_roleset_filtered)?;
    table_set_rust_fn_static(state, table, "SetRolesets", set_rolesets)
}

#[cfg(feature = "retail-12-1-0")]
fn is_roleset_filtered(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let filtered = borrow_state(state)?
        .widgets
        .get(id)
        .is_some_and(|frame| frame.roleset_filtered);
    state.push(Val::Bool(filtered));
    Ok(1)
}

fn update_rolesets(
    state: &LuaState,
    id: u64,
    update: impl FnOnce(&mut Vec<String>),
) -> LuaResult<()> {
    update_frame_rolesets(&mut *borrow_state_mut(state)?, id, update);
    Ok(())
}

fn roleset_arg(state: &LuaState) -> Option<String> {
    val_to_string(state, stack_val(state, 2))
}

fn add_roleset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if let Some(name) = roleset_arg(state) {
        update_rolesets(state, id, |rolesets| {
            if !rolesets.contains(&name) {
                rolesets.push(name);
            }
        })?;
    }
    Ok(0)
}

/// `SetRolesets("a,b")` assigns comma-separated tags; nil clears them.
fn set_rolesets(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let names = roleset_arg(state).map(|list| split_roleset_list(&list));
    update_rolesets(state, id, |rolesets| {
        *rolesets = names.unwrap_or_default();
    })?;
    Ok(0)
}

fn remove_roleset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if let Some(name) = roleset_arg(state) {
        update_rolesets(state, id, |rolesets| {
            rolesets.retain(|existing| *existing != name);
        })?;
    }
    Ok(0)
}

fn get_roleset_names(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut names = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.rolesets.clone())
        .unwrap_or_default();
    if names.is_empty() {
        names.push(ROLELESS.to_string());
    }
    push_names(state, &names);
    Ok(1)
}
