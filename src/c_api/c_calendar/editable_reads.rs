//! Reads from the supplied being-edited event and its invite list.
use super::{
    CalendarInviteRecord, CalendarTextureRecord, civil, club, day_event_projection::publish_time,
};
use crate::c_api::helpers::set_table_array;
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
    table_set_rust_fn_static(state, namespace, "GetNumInvites", num_invites)?;
    table_set_rust_fn_static(state, namespace, "EventGetCalendarType", calendar_type)?;
    table_set_rust_fn_static(state, namespace, "EventGetClubId", event_club_id)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "EventGetInviteResponseTime",
        invite_response_time,
    )?;
    table_set_rust_fn_static(state, namespace, "EventGetTextures", event_textures)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "EventHasPendingInvite",
        has_pending_invite,
    )
}

fn calendar_type(state: &mut LuaState) -> LuaResult<u32> {
    let flags = borrow_state(state)?
        .calendar_context_policy
        .editable
        .as_ref()
        .map(|event| event.category_flags);
    let value = match flags {
        None => Val::Nil,
        Some(flags) => {
            let name = if flags & 0x40 != 0 {
                "GUILD_ANNOUNCEMENT"
            } else if flags & 0x800 != 0 {
                "GUILD_EVENT"
            } else if flags & 0x400 != 0 {
                "COMMUNITY_EVENT"
            } else if flags & 0x4 != 0 {
                "SYSTEM"
            } else if flags & 0x8 != 0 {
                "HOLIDAY"
            } else if flags & 0x80 != 0 {
                "RAID_LOCKOUT"
            } else if flags & 0x200 != 0 {
                "RAID_RESET"
            } else if flags & 0x1 != 0 {
                // WM40: 0x1 identifies player events; its priority is UNKNOWN, so it is last.
                "PLAYER"
            } else {
                ""
            };
            create_string(state, name)
        }
    };
    state.push(value);
    Ok(1)
}

fn event_club_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?
        .calendar_context_policy
        .editable
        .as_ref()
        .and_then(|event| event.club_id);
    let value = club::club_id_value(state, id);
    state.push(value);
    Ok(1)
}

fn has_pending_invite(state: &mut LuaState) -> LuaResult<u32> {
    let pending = {
        let host = borrow_state(state)?;
        host.calendar_context_policy
            .editable
            .as_ref()
            .is_some_and(|event| {
                host.calendar_pending_invite_event_ids
                    .contains(&event.event_id)
            })
    };
    state.push(Val::Bool(pending));
    Ok(1)
}

fn invite_response_time(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: local time = C_Calendar.EventGetInviteResponseTime(eventIndex)";
    let index = read_invite_index(state, USAGE)?;
    let time = {
        let host = borrow_state(state)?;
        index
            .and_then(|index| {
                host.calendar_context_policy
                    .editable
                    .as_ref()
                    .and_then(|event| event.invites.get(index))
            })
            .and_then(|invite| invite.response_time.clone())
    };
    let Some(time) = time else {
        return Ok(0);
    };
    let outer = create_table(state);
    state.push(outer);
    publish_time(state, outer, "time", &time);
    let value = crate::lua_api::methods::table_get(state, outer, "time");
    state.pop();
    state.push(value);
    Ok(1)
}

fn event_textures(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: local textures = C_Calendar.EventGetTextures(eventType)";
    let kind = civil::numeric_argument(state, 1)?
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= 5.0)
        .ok_or_else(|| runtime_error(USAGE))? as u8;
    let rows = {
        let host = borrow_state(state)?;
        match kind {
            0 => host.calendar_texture_lists[0].clone(),
            1 | 5 => host.calendar_texture_lists[1].clone(),
            _ => Vec::new(),
        }
    };
    let array = create_table(state);
    state.push(array);
    for (index, row) in rows.iter().enumerate() {
        push_texture(state, array, index, row);
    }
    Ok(1)
}

fn push_texture(state: &mut LuaState, array: Val, index: usize, record: &CalendarTextureRecord) {
    let table = create_table(state);
    state.push(table);
    set_text(state, table, "title", &record.title);
    for (key, value) in [
        ("iconTexture", f64::from(record.icon_texture)),
        ("expansionLevel", f64::from(record.expansion_level)),
        ("difficultyId", f64::from(record.difficulty_id)),
        ("mapId", f64::from(record.map_id)),
    ] {
        table_set_static(state, table, key, Val::Num(value));
    }
    table_set_static(state, table, "isLfr", Val::Bool(record.is_lfr));
    set_table_array(state, array, index as i64 + 1, table);
    state.pop();
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
    let index = read_invite_index(state, USAGE)?;
    let record = {
        let host = borrow_state(state)?;
        index
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

pub(super) fn read_invite_index(state: &mut LuaState, usage: &str) -> LuaResult<Option<usize>> {
    let index = civil::numeric_argument(state, 1)?
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= f64::from(u32::MAX))
        .ok_or_else(|| runtime_error(usage.to_owned()))?;
    Ok(usize::try_from((index - 1.0) as i64).ok())
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
