//! Local title edits on the supplied editable Calendar event.
use super::editable_access::can_edit;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "EventSetTitle", set_title)
}

fn set_title(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Str(value) = value else {
        return Err(runtime_error("Usage: C_Calendar.EventSetTitle(title)"));
    };
    let input = state
        .gc
        .string_arena
        .get(value)
        .expect("valid Lua string")
        .data();
    let input = input
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default()
        .to_vec();
    let allowed = {
        let host = borrow_state(state)?;
        can_edit(&host)
    };
    if allowed {
        let mut host = borrow_state_mut(state)?;
        let event = host
            .calendar_context_policy
            .editable
            .as_mut()
            .expect("editable event checked above");
        if event.title != input {
            event.title = filter_title(&input);
            event.dirty = true;
        }
    }
    Ok(0)
}

fn filter_title(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut count = 0;
    for (index, byte) in input.iter().copied().enumerate() {
        if byte == b'\r' || byte == b'\n' || (byte == b'\\' && input.get(index + 1) == Some(&b'n'))
        {
            break;
        }
        if byte == b'|' {
            continue;
        }
        if result.len() == 128 {
            break;
        }
        if byte & 0xc0 != 0x80 {
            if count == 32 {
                break;
            }
            count += 1;
        }
        result.push(byte);
    }
    result
}
