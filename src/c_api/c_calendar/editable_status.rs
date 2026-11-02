//! Status options for an invite in the editable Calendar event.
use super::{editable_access::has_local_authority, editable_reads::read_invite_index};
use crate::c_api::helpers::set_table_array;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_static,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const OPTIONS: [(i32, &str); 6] = [
    (1, "CALENDAR_STATUS_ACCEPTED"),
    (2, "CALENDAR_STATUS_DECLINED"),
    (3, "CALENDAR_STATUS_CONFIRMED"),
    (4, "CALENDAR_STATUS_OUT"),
    (5, "CALENDAR_STATUS_STANDBY"),
    (8, "CALENDAR_STATUS_TENTATIVE"),
];

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "EventGetStatusOptions", status_options)
}

enum Selection {
    Empty,
    Denied,
    Invite { signup: bool, status: Option<i32> },
}

fn status_options(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_invite_index(
        state,
        "Usage: local options = C_Calendar.EventGetStatusOptions(eventIndex)",
    )?;
    let selection = {
        let host = borrow_state(state)?;
        match host.calendar_context_policy.editable.as_ref() {
            None => Selection::Empty,
            Some(_) if !has_local_authority(&host) => Selection::Denied,
            Some(event) => match index.and_then(|index| event.invites.get(index)) {
                None => Selection::Empty,
                Some(invite) => Selection::Invite {
                    signup: event.category_flags & 0x400 != 0,
                    status: invite.invite_status,
                },
            },
        }
    };
    if matches!(selection, Selection::Denied) {
        let message = create_string(state, "CALENDAR_ERROR_PERMISSIONS");
        dispatch_event_now(state, "CALENDAR_UPDATE_ERROR", &[message])?;
        borrow_state_mut(state)?
            .calendar_context_policy
            .action_pending = false;
        dispatch_event_now(state, "CALENDAR_ACTION_PENDING", &[Val::Bool(false)])?;
    }
    let array = create_table(state);
    state.push(array);
    if let Selection::Invite { signup, status } = selection {
        let mut index = 1;
        for (value, name) in OPTIONS {
            if (signup && value <= 2) || status == Some(value) {
                continue;
            }
            let row = create_table(state);
            state.push(row);
            table_set_static(state, row, "status", Val::Num(f64::from(value)));
            let name = create_string(state, name);
            table_set_static(state, row, "statusString", name);
            set_table_array(state, array, index, row);
            state.pop();
            index += 1;
        }
    }
    Ok(1)
}
