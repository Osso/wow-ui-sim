//! Local type edits on the supplied editable Calendar event.
use super::{civil::numeric_argument, editable_access::can_edit};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "EventSetType", set_type)
}

fn set_type(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: C_Calendar.EventSetType(typeIndex)";
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if matches!(value, Val::Nil) {
        return Err(runtime_error(USAGE));
    }
    let number = numeric_argument(state, 1)?.unwrap_or(0.0);
    if !number.is_finite() || !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number) {
        return Err(runtime_error(USAGE));
    }
    let kind = (number as i32) as u8;
    if kind > 5 {
        return Err(runtime_error(USAGE));
    }
    let mut host = borrow_state_mut(state)?;
    if can_edit(&host) {
        let event = host
            .calendar_context_policy
            .editable
            .as_mut()
            .expect("editable event checked above");
        let old_type = if event.event_type == 5 {
            1
        } else {
            event.event_type
        };
        if old_type != kind {
            event.event_type = kind;
            event.dirty = true;
        }
    }
    Ok(0)
}
