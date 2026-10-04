//! Per-object conditional access-restriction masks.
//!
//! These mask APIs do not enforce access. Forever's context-access query reads
//! an explicit aura-secret context input; automatic activation remains unmodeled.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "AddAccessRestrictions",
        add_access_restrictions,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "GetAccessRestrictions",
        get_access_restrictions,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "HasAnyAccessRestrictions",
        has_any_access_restrictions,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "CanBeAccessedInContext",
        can_be_accessed_in_context,
    )
}

/// Secure execution may access anything; tainted execution is denied for
/// forbidden objects and objects carrying any access-restriction mask.
fn can_be_accessed_in_context(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let restricted = {
        let sim = borrow_state(state)?;
        let frame = sim
            .widgets
            .get(id)
            .ok_or_else(|| runtime_error("invalid frame"))?;
        frame.forbidden || frame.access_restrictions != 0
    };
    let can_access = rilua::api::state_is_secure(state) || !restricted;
    state.push(Val::Bool(can_access));
    Ok(1)
}

fn add_access_restrictions(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let restrictions = u32::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    let frame = sim
        .widgets
        .get_mut(id)
        .ok_or_else(|| runtime_error("invalid frame"))?;
    frame.access_restrictions |= restrictions;
    Ok(0)
}

fn stored_restrictions(state: &LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let frame = sim
        .widgets
        .get(id)
        .ok_or_else(|| runtime_error("invalid frame"))?;
    Ok(frame.access_restrictions)
}

fn get_access_restrictions(state: &mut LuaState) -> LuaResult<u32> {
    let restrictions = stored_restrictions(state)?;
    state.push(Val::Num(f64::from(restrictions)));
    Ok(1)
}

fn has_any_access_restrictions(state: &mut LuaState) -> LuaResult<u32> {
    let restrictions = stored_restrictions(state)?;
    let has_any = if matches!(stack_val(state, 2), Val::Nil) {
        restrictions != 0
    } else {
        let filter = u32::from_stack(state, 2)?;
        restrictions & filter != 0
    };
    state.push(Val::Bool(has_any));
    Ok(1)
}
