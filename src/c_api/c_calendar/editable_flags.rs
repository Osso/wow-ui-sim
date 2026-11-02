//! Local flag changes on the editable Calendar event.
use super::editable_access::can_edit;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "EventSetAutoApprove", set_auto_approve)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "EventClearAutoApprove",
        clear_auto_approve,
    )?;
    table_set_rust_fn_static(state, namespace, "EventSetLocked", set_locked)?;
    table_set_rust_fn_static(state, namespace, "EventClearLocked", clear_locked)
}

enum Flag {
    AutoApprove,
    Locked,
}

fn set_auto_approve(state: &mut LuaState) -> LuaResult<u32> {
    change_flag(state, Flag::AutoApprove, true)
}
fn clear_auto_approve(state: &mut LuaState) -> LuaResult<u32> {
    change_flag(state, Flag::AutoApprove, false)
}
fn set_locked(state: &mut LuaState) -> LuaResult<u32> {
    change_flag(state, Flag::Locked, true)
}
fn clear_locked(state: &mut LuaState) -> LuaResult<u32> {
    change_flag(state, Flag::Locked, false)
}

fn change_flag(state: &mut LuaState, flag: Flag, value: bool) -> LuaResult<u32> {
    let mut host = borrow_state_mut(state)?;
    if !can_edit(&host) {
        return Ok(0);
    }
    let event = host
        .calendar_context_policy
        .editable
        .as_mut()
        .expect("editable event checked above");
    let field = match flag {
        Flag::Locked => &mut event.locked,
        Flag::AutoApprove => &mut event.auto_approve,
    };
    if *field != value {
        *field = value;
        event.dirty = true;
    }
    Ok(0)
}
