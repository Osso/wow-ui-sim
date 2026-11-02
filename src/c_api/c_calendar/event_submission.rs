//! Create and update submissions for the editable Calendar event (Retail 70688 WM41-B).
//!
//! The protected-action gate is not modeled; the original consumer is untainted.
use super::{
    CalendarEditableAccess,
    club::parse_record_club_id,
    create_event::find_guild_club_id,
    editable_access::can_edit,
    submission_policy::{
        DateTimeParts, Outcome, finish, has_passed, is_throttled, refresh_throttle,
        within_creation_window,
    },
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "AddEvent", add_event)?;
    table_set_rust_fn_static(state, namespace, "UpdateEvent", update_event)
}

/// The record's date and time as civil parts; unset components stay -1. Stored months
/// and days are zero-based and years are relative to 2000.
fn event_parts(event: &CalendarEditableAccess) -> DateTimeParts {
    let (month, day, year) = event.date.unwrap_or((-1, -1, -1));
    let (hour, minute) = event
        .time
        .map_or((-1, -1), |(hour, minute)| (hour as i32, minute as i32));
    let civil = |value: i32, offset: i32| if value < 0 { -1 } else { value + offset };
    [
        civil(year, 2000),
        civil(month, 1),
        civil(day, 1),
        hour,
        minute,
    ]
}

fn add_event(state: &mut LuaState) -> LuaResult<u32> {
    let outcome = {
        let host = borrow_state(state)?;
        add_outcome(&host)
    };
    if matches!(outcome, Outcome::Request) {
        let mut host = borrow_state_mut(state)?;
        refresh_throttle(&mut host);
    }
    finish(state, outcome)
}

/// WM41-B order: pending, missing record, throttle, the silent creation window (before
/// the title), the new-record requirement, then the first validation failure.
fn add_outcome(host: &SimState) -> Outcome {
    let policy = &host.calendar_context_policy;
    if policy.action_pending {
        return Outcome::Ignore;
    }
    let Some(event) = policy.editable.as_ref() else {
        return Outcome::Error("CALENDAR_ERROR_INTERNAL");
    };
    if is_throttled(host) {
        return Outcome::Error("CALENDAR_ERROR_EVENT_THROTTLED");
    }
    let parts = event_parts(event);
    if !within_creation_window(host, [parts[0], parts[1], parts[2]]) || !event.is_new {
        return Outcome::Ignore;
    }
    first_add_failure(host, event, parts).map_or(Outcome::Request, Outcome::Error)
}

fn first_add_failure(
    host: &SimState,
    event: &CalendarEditableAccess,
    parts: DateTimeParts,
) -> Option<&'static str> {
    if event.title.is_empty() {
        Some("CALENDAR_ERROR_NEEDS_TITLE")
    } else if let Some(key) = missing_club(host, event) {
        Some(key)
    } else if parts[3] == -1 || parts[4] == -1 {
        Some("CALENDAR_ERROR_INVALID_TIME")
    } else if parts[..3].contains(&-1) {
        Some("CALENDAR_ERROR_INVALID_DATE")
    } else if has_passed(host, parts) {
        Some("CALENDAR_ERROR_EVENT_TIME_PASSED")
    } else {
        None
    }
}

/// INFERRED: guild (0x800) and announcement (0x40) records need the guild club; other
/// signup (0x400) records need a supplied club record matching their club ID. The
/// maximum-size check for other records is not modeled; creation always sets 100.
fn missing_club(host: &SimState, event: &CalendarEditableAccess) -> Option<&'static str> {
    if event.category_flags & 0x840 != 0 {
        find_guild_club_id(host)
            .is_none()
            .then_some("ERR_GUILD_PLAYER_NOT_IN_GUILD")
    } else if event.category_flags & 0x400 != 0 {
        let present = event.club_id.is_some_and(|id| {
            host.clubs
                .clubs
                .values()
                .any(|club| parse_record_club_id(&club.id) == Some(id))
        });
        (!present).then_some("ERR_COMMUNITY_PLAYER_NOT_IN_COMMUNITY")
    } else {
        None
    }
}

fn update_event(state: &mut LuaState) -> LuaResult<u32> {
    let outcome = {
        let mut host = borrow_state_mut(state)?;
        update_outcome(&mut host)
    };
    finish(state, outcome)
}

/// WM41-B: dirty clears before the cutoff comparison. Update repeats neither the title,
/// size, throttle nor creation-window checks.
fn update_outcome(host: &mut SimState) -> Outcome {
    let policy = &host.calendar_context_policy;
    let Some(event) = policy.editable.as_ref() else {
        return Outcome::Ignore;
    };
    if policy.action_pending || event.is_new {
        return Outcome::Ignore;
    }
    if !can_edit(host) {
        return Outcome::Error("CALENDAR_ERROR_PERMISSIONS");
    }
    if !event.dirty {
        return Outcome::Ignore;
    }
    let passed = has_passed(host, event_parts(event));
    host.calendar_context_policy
        .editable
        .as_mut()
        .expect("editable event checked above")
        .dirty = false;
    if passed {
        Outcome::Error("CALENDAR_ERROR_EVENT_PASSED")
    } else {
        Outcome::Request
    }
}
