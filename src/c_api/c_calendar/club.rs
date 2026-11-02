//! Calendar's optional next community selection.
use super::editable_access::can_edit;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, call_function_state, create_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const MAX_EXACT: u64 = 9_007_199_254_740_991;
const USAGE: &str = "Usage: C_Calendar.SetNextClubId([clubId])";

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetNextClubId", get_next_club_id)?;
    table_set_rust_fn_static(state, namespace, "SetNextClubId", set_next_club_id)?;
    table_set_rust_fn_static(state, namespace, "EventSetClubId", set_event_club_id)
}

fn get_next_club_id(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.calendar_next_club_id;
    let result = club_id_value(state, value);
    state.push(result);
    Ok(1)
}

pub(super) fn club_id_value(state: &mut LuaState, value: Option<u64>) -> Val {
    match value {
        None => Val::Nil,
        Some(id) if id <= MAX_EXACT => Val::Num(id as f64),
        Some(id) => create_string(state, &format!("0x{id:016X}")),
    }
}

/// Club record keys hold decimal or `0x`-prefixed hexadecimal club IDs.
pub(super) fn parse_record_club_id(text: &str) -> Option<u64> {
    match text.get(..2) {
        Some(prefix) if prefix.eq_ignore_ascii_case("0x") => {
            u64::from_str_radix(&text[2..], 16).ok()
        }
        _ => text.parse().ok(),
    }
}

fn set_next_club_id(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let id = if matches!(value, Val::Nil) {
        None
    } else {
        Some(read_club_id(state, value, USAGE)?)
    };
    borrow_state_mut(state)?.calendar_next_club_id = id;
    Ok(0)
}

fn set_event_club_id(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let id = if matches!(value, Val::Nil) {
        0
    } else {
        read_club_id(state, value, "Usage: C_Calendar.EventSetClubId([clubId])")?
    };
    let allowed = {
        let host = borrow_state(state)?;
        can_edit(&host)
    };
    if !allowed {
        let message = create_string(state, "CALENDAR_ERROR_PERMISSIONS");
        dispatch_event_now(state, "CALENDAR_UPDATE_ERROR", &[message])?;
        borrow_state_mut(state)?
            .calendar_context_policy
            .action_pending = false;
        dispatch_event_now(state, "CALENDAR_ACTION_PENDING", &[Val::Bool(false)])?;
        return Ok(0);
    }
    let mut host = borrow_state_mut(state)?;
    let event = host
        .calendar_context_policy
        .editable
        .as_mut()
        .expect("editable event checked above");
    if event.club_id != id {
        event.club_id = id;
        event.dirty = true;
    }
    Ok(0)
}

fn read_club_id(state: &mut LuaState, value: Val, usage: &str) -> LuaResult<u64> {
    if let Val::Str(s) = value {
        let bytes = state
            .gc
            .string_arena
            .get(s)
            .expect("valid Lua string")
            .data();
        let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
        if bytes.len() == 18
            && bytes[..2].eq_ignore_ascii_case(b"0x")
            && bytes[2..].iter().all(u8::is_ascii_hexdigit)
        {
            let text = std::str::from_utf8(&bytes[2..]).expect("ASCII hex");
            let id = u64::from_str_radix(text, 16).expect("validated hex");
            return if id > MAX_EXACT {
                Ok(id)
            } else {
                Err(runtime_error(usage))
            };
        }
    }
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => {
            let tonumber = crate::c_api::global_val(state, "tonumber");
            match call_function_state(state, tonumber, &[value])? {
                Val::Num(number) => Some(number),
                _ => None,
            }
        }
        _ => None,
    };
    match number {
        Some(n) if n.is_finite() && (0.0..=MAX_EXACT as f64).contains(&n) => Ok(n as u64),
        Some(_) => Err(runtime_error(usage)),
        None => Ok(0),
    }
}
