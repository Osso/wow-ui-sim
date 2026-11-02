//! Context-menu copy, paste and removal requests (Retail 70688 WM41-A).
//!
//! Server responses are UNKNOWN: an accepted request only marks the action pending.
use super::{
    civil::{days_in_month, numeric_argument},
    context_policy::can_modify,
    editable_access::local_player_guid,
    editable_time::report_invalid,
    selection::context_row,
    submission_policy::{
        DateParts, clamp_month_index, is_throttled, refresh_throttle, within_creation_window,
    },
};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "ContextMenuEventCopy", copy)?;
    table_set_rust_fn_static(state, namespace, "ContextMenuEventPaste", paste)?;
    table_set_rust_fn_static(state, namespace, "ContextMenuEventRemove", remove_context)?;
    table_set_rust_fn_static(state, namespace, "RemoveEvent", remove_own_invite)
}

enum Outcome {
    Ignore,
    Error(&'static str),
    Request,
}

fn finish(state: &mut LuaState, outcome: Outcome) -> LuaResult<u32> {
    match outcome {
        Outcome::Ignore => Ok(0),
        Outcome::Error(key) => report_invalid(state, key),
        Outcome::Request => {
            borrow_state_mut(state)?
                .calendar_context_policy
                .action_pending = true;
            dispatch_event_now(state, "CALENDAR_ACTION_PENDING", &[Val::Bool(true)])?;
            Ok(0)
        }
    }
}

/// WM41-A evaluates edit permission twice; a second denial would clear the clipboard.
/// Both evaluations read the same supplied inputs here, so only the first can deny,
/// and that denial keeps the clipboard.
fn copy(state: &mut LuaState) -> LuaResult<u32> {
    let copied = {
        let host = borrow_state(state)?;
        context_row(&host).map(|row| can_modify(&host, row, false).then(|| row.clone()))
    };
    match copied {
        None => Ok(0),
        Some(None) => report_invalid(state, "CALENDAR_ERROR_PERMISSIONS"),
        Some(Some(row)) => {
            borrow_state_mut(state)?.calendar_clipboard = Some(row);
            Ok(0)
        }
    }
}

fn paste(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: C_Calendar.ContextMenuEventPaste(offsetMonths, monthDay)";
    let offset = numeric_argument(state, 1)?
        .filter(|n| n.is_finite() && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(n))
        .ok_or_else(|| runtime_error(USAGE))? as i32;
    let day = numeric_argument(state, 2)?
        .filter(|n| n.is_finite() && (0.0..=f64::from(u32::MAX)).contains(n))
        .ok_or_else(|| runtime_error(USAGE))? as u32;
    let outcome = {
        let host = borrow_state(state)?;
        paste_outcome(&host, offset, day)
    };
    if matches!(outcome, Outcome::Request) {
        refresh_throttle(&mut borrow_state_mut(state)?);
    }
    finish(state, outcome)
}

/// Busy state, an empty clipboard, an invalid day and out-of-window dates stop silently.
/// The throttle's position relative to target construction is UNKNOWN; it precedes the
/// date policy as in AddEvent.
fn paste_outcome(host: &SimState, offset: i32, day: u32) -> Outcome {
    if host.calendar_context_policy.action_pending || host.calendar_clipboard.is_none() {
        return Outcome::Ignore;
    }
    let Some(date) = paste_target(host, offset, day) else {
        return Outcome::Ignore;
    };
    if is_throttled(host) {
        Outcome::Error("CALENDAR_ERROR_EVENT_THROTTLED")
    } else if within_creation_window(host, date) {
        Outcome::Request
    } else {
        Outcome::Ignore
    }
}

/// The displayed month plus the offset; the simulator's displayed month is always
/// initialized, so the current-time fallback does not arise.
fn paste_target(host: &SimState, offset: i32, day: u32) -> Option<DateParts> {
    let shown = host.calendar_displayed_month;
    let index = clamp_month_index(
        i64::from(shown.year) * 12 + i64::from(shown.month) - 1 + i64::from(offset),
    );
    let year = index.div_euclid(12) as i32;
    let month = (index.rem_euclid(12) + 1) as u8;
    (1..=u32::from(days_in_month(year, month)))
        .contains(&day)
        .then_some([year, i32::from(month), day as i32])
}

/// Permission precedes the pending check; removal leaves local rows and selections intact.
fn remove_context(state: &mut LuaState) -> LuaResult<u32> {
    let outcome = {
        let host = borrow_state(state)?;
        match context_row(&host) {
            None => Outcome::Ignore,
            Some(row) if !can_modify(&host, row, true) => {
                Outcome::Error("CALENDAR_ERROR_PERMISSIONS")
            }
            Some(_) if host.calendar_context_policy.action_pending => Outcome::Ignore,
            Some(_) => Outcome::Request,
        }
    };
    finish(state, outcome)
}

/// Removes the local player's own invitation, not the event; the invite list is unchanged.
fn remove_own_invite(state: &mut LuaState) -> LuaResult<u32> {
    let outcome = {
        let host = borrow_state(state)?;
        own_invite_outcome(&host)
    };
    finish(state, outcome)
}

fn own_invite_outcome(host: &SimState) -> Outcome {
    let policy = &host.calendar_context_policy;
    let Some(event) = policy.editable.as_ref() else {
        return Outcome::Ignore;
    };
    if event.is_new || policy.action_pending {
        return Outcome::Ignore;
    }
    let player = local_player_guid(host);
    let own = event
        .invites
        .iter()
        .find(|invite| player.as_deref() == Some(invite.guid.as_str()));
    match own {
        None => Outcome::Ignore,
        Some(invite) if invite.mod_status.as_deref() == Some("CREATOR") => {
            Outcome::Error("CALENDAR_ERROR_DELETE_CREATOR_FAILED")
        }
        Some(_) => Outcome::Request,
    }
}
