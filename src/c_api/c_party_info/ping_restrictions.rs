//! Bounded explicit ping restriction state; no ping delivery or permission model.

use crate::c_api::c_chat_info::reject_chat_messaging_lockdown;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::is_secret_value;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetRestrictPings", get_restrict_pings)?;
    table_set_rust_fn_static(state, namespace, "SetRestrictPings", set_restrict_pings)
}

fn get_restrict_pings(state: &mut LuaState) -> LuaResult<u32> {
    let restriction = borrow_state(state)?.party_ping_restriction;
    state.push(Val::Num(f64::from(restriction)));
    Ok(1)
}

fn set_restrict_pings(state: &mut LuaState) -> LuaResult<u32> {
    reject_chat_messaging_lockdown(state, "SetRestrictPings")?;
    let restriction = read_restriction(state)?;
    borrow_state_mut(state)?.party_ping_restriction = restriction;
    Ok(0)
}

fn read_restriction(state: &LuaState) -> LuaResult<u8> {
    let value = stack_val(state, 1);
    // Inferred conservative policy, not native AllowedWhenUntainted parity.
    if is_secret_value(state, value) {
        return Err(runtime_error(
            "SetRestrictPings: secret enum access is not modeled",
        ));
    }
    // Inferred strict enum validation: no coercion or mutation on rejection.
    match value {
        Val::Num(number) if matches!(number, 0.0 | 1.0 | 2.0 | 3.0) => Ok(number as u8),
        _ => Err(runtime_error(
            "SetRestrictPings: restrictTo must be a public enum number from 0 to 3",
        )),
    }
}
