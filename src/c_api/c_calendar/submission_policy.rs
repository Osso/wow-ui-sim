//! Request policy shared by Calendar submissions (Retail 70688 WM41).
//!
//! Server requests and responses are UNKNOWN: an accepted request only marks the action
//! pending and notifies `CALENDAR_ACTION_PENDING(true)`.
use super::civil::days_in_month;
use super::editable_time::report_invalid;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::SimState;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::cmp::Ordering;

/// WM41: a nonzero previous timestamp fewer than 5000 ticks old throttles submission.
const THROTTLE_TICKS: i32 = 5000;
/// INFERRED hard limits of the creation window.
const EARLIEST_DATE: DateParts = [2004, 11, 24];
const LATEST_DATE: DateParts = [2030, 12, 31];

/// Calendar year, one-based month and day; a negative component is unset.
pub(super) type DateParts = [i32; 3];
/// Date parts followed by hour and minute; a negative component is unset.
pub(super) type DateTimeParts = [i32; 5];

pub(super) enum Outcome {
    Ignore,
    Error(&'static str),
    Request,
}

pub(super) fn finish(state: &mut LuaState, outcome: Outcome) -> LuaResult<u32> {
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

/// INFERRED: the native cached tick counter is the simulator clock in milliseconds.
fn current_tick(host: &SimState) -> u32 {
    host.clock_elapsed().as_millis() as u32
}

/// WM41: signed, wrapping 32-bit elapsed ticks since a nonzero previous timestamp.
pub(super) fn is_throttled(host: &SimState) -> bool {
    let previous = host.calendar_context_policy.last_submit_tick;
    previous != 0 && (current_tick(host).wrapping_sub(previous) as i32) < THROTTLE_TICKS
}

pub(super) fn refresh_throttle(host: &mut SimState) {
    host.calendar_context_policy.last_submit_tick = current_tick(host);
}

/// WM41: components compare in order, skipping any component negative on either side.
fn compare(left: &[i32], right: &[i32]) -> Ordering {
    left.iter()
        .zip(right)
        .filter(|(left, right)| **left >= 0 && **right >= 0)
        .map(|(left, right)| left.cmp(right))
        .find(|order| order.is_ne())
        .unwrap_or(Ordering::Equal)
}

/// INFERRED window: the supplied calendar date through the last day of its month next
/// year, intersected with the hard limits. Reject strictly before or strictly after.
/// Weekday follows from coherent dates and clock times are equalized, so year, month
/// and day decide.
pub(super) fn within_creation_window(host: &SimState, date: DateParts) -> bool {
    let now = host.calendar_now;
    let today = [now.year, i32::from(now.month), i32::from(now.day)];
    let last_next_year = [
        now.year + 1,
        i32::from(now.month),
        i32::from(days_in_month(now.year + 1, now.month)),
    ];
    let lower = today.max(EARLIEST_DATE);
    let upper = last_next_year.min(LATEST_DATE);
    compare(&date, &lower) != Ordering::Less && compare(&date, &upper) != Ordering::Greater
}

/// WM41: target-month construction clamps to November 2004 through December 2030.
pub(super) fn clamp_month_index(month_index: i64) -> i64 {
    month_index.clamp(2004 * 12 + 10, 2030 * 12 + 11)
}

/// INFERRED cutoff: the supplied calendar date with the game clock's hour and minute
/// (`GetLocalGameTime`), minus one minute with day rollover. The local clock's day and
/// minute offset writer is not modeled, so the offset is zero and the local date is the
/// calendar date. An unknown game clock leaves hour and minute unset.
fn local_cutoff(host: &SimState) -> DateTimeParts {
    let now = host.calendar_now;
    let [year, month, day] = [now.year, i32::from(now.month), i32::from(now.day)];
    let Some((hour, minute)) = host.game_clock.time else {
        return [year, month, day, -1, -1];
    };
    let minutes = i32::from(hour) * 60 + i32::from(minute) - 1;
    if minutes >= 0 {
        return [year, month, day, minutes / 60, minutes % 60];
    }
    let [year, month, day] = previous_day([year, month, day]);
    [year, month, day, 23, 59]
}

fn previous_day([year, month, day]: DateParts) -> DateParts {
    if day > 1 {
        [year, month, day - 1]
    } else if month > 1 {
        [
            year,
            month - 1,
            i32::from(days_in_month(year, (month - 1) as u8)),
        ]
    } else {
        [year - 1, 12, 31]
    }
}

/// WM41-B: an event at or before the local cutoff has passed.
pub(super) fn has_passed(host: &SimState, event: DateTimeParts) -> bool {
    compare(&event, &local_cutoff(host)) != Ordering::Greater
}
