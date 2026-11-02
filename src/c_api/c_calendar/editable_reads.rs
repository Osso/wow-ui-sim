//! Reads from the supplied being-edited event and its invite list.
use super::{CalendarInviteRecord, civil};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "EventHaveSettingsChanged",
        settings_changed,
    )?;
    table_set_rust_fn_static(state, namespace, "EventGetSelectedInvite", selected_invite)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "EventGetInviteSortCriterion",
        invite_sort_criterion,
    )?;
    table_set_rust_fn_static(state, namespace, "EventGetInvite", invite)?;
    table_set_rust_fn_static(state, namespace, "GetNumInvites", num_invites)
}

fn settings_changed(state: &mut LuaState) -> LuaResult<u32> {
    let dirty = borrow_state(state)?
        .calendar_context_policy
        .editable
        .as_ref()
        .is_some_and(|event| event.dirty);
    state.push(Val::Bool(dirty));
    Ok(1)
}

fn selected_invite(state: &mut LuaState) -> LuaResult<u32> {
    let selected = {
        let host = borrow_state(state)?;
        host.calendar_context_policy.editable.as_ref().map(|event| {
            event
                .selected_invite_id
                .as_ref()
                .and_then(|id| {
                    event
                        .invites
                        .iter()
                        .position(|invite| &invite.invite_id == id)
                })
                .map_or(0, |index| index + 1)
        })
    };
    state.push(selected.map_or(Val::Nil, |index| Val::Num(index as f64)));
    Ok(1)
}

fn invite_sort_criterion(state: &mut LuaState) -> LuaResult<u32> {
    let (criterion, reverse) = {
        let host = borrow_state(state)?;
        host.calendar_context_policy
            .editable
            .as_ref()
            .map(|event| (event.sort_criterion.clone(), event.sort_reverse))
            .unwrap_or_default()
    };
    let text = create_string(state, &criterion);
    state.push(text);
    state.push(Val::Bool(reverse));
    Ok(2)
}

fn num_invites(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .calendar_context_policy
        .editable
        .as_ref()
        .map_or(0, |event| event.invites.len());
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn invite(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: local info = C_Calendar.EventGetInvite(eventIndex)";
    let index = civil::numeric_argument(state, 1)?
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= f64::from(u32::MAX))
        .ok_or_else(|| runtime_error(USAGE))?;
    let index = (index - 1.0) as i64;
    let record = {
        let host = borrow_state(state)?;
        usize::try_from(index)
            .ok()
            .and_then(|index| {
                host.calendar_context_policy
                    .editable
                    .as_ref()
                    .and_then(|event| event.invites.get(index))
            })
            .cloned()
    };
    let Some(record) = record else {
        return Ok(0);
    };
    push_invite(state, &record);
    Ok(1)
}

fn set_text(state: &mut LuaState, table: Val, key: &'static str, text: &str) {
    let value = create_string(state, text);
    table_set_static(state, table, key, value);
}

fn push_invite(state: &mut LuaState, record: &CalendarInviteRecord) {
    let table = create_table(state);
    state.push(table);
    for (key, value) in [
        ("name", record.name.as_deref()),
        ("className", record.class_name.as_deref()),
        ("classFilename", record.class_filename.as_deref()),
        ("modStatus", record.mod_status.as_deref()),
    ] {
        if let Some(value) = value {
            set_text(state, table, key, value);
        }
    }
    set_text(state, table, "notes", &record.notes);
    set_text(state, table, "guid", &record.guid);
    table_set_static(state, table, "level", Val::Num(record.level));
    table_set_static(
        state,
        table,
        "inviteIsMine",
        Val::Bool(record.invite_is_mine),
    );
    table_set_static(
        state,
        table,
        "type",
        Val::Num(f64::from(record.invite_type)),
    );
    if let Some(status) = record.invite_status {
        table_set_static(state, table, "inviteStatus", Val::Num(f64::from(status)));
    }
    if let Some(class_id) = record.class_id {
        table_set_static(state, table, "classID", Val::Num(f64::from(class_id)));
    }
}
