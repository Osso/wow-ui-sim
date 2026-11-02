//! Create/paste policy shared by Calendar submissions (Retail 70688 WM41).
use super::civil::days_in_month;
use crate::lua_api::state::SimState;
use std::cmp::Ordering;

/// WM41: a nonzero previous timestamp fewer than 5000 ticks old throttles submission.
const THROTTLE_TICKS: i32 = 5000;
/// INFERRED hard limits of the creation window.
const EARLIEST_DATE: DateParts = [2004, 11, 24];
const LATEST_DATE: DateParts = [2030, 12, 31];

/// Calendar year, one-based month and day; a negative component is unset.
pub(super) type DateParts = [i32; 3];

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
/// Weekday follows from coherent dates and clock times are equalized, so year, month
/// and day decide the window.
fn compare(left: DateParts, right: DateParts) -> Ordering {
    left.iter()
        .zip(right.iter())
        .filter(|(left, right)| **left >= 0 && **right >= 0)
        .map(|(left, right)| left.cmp(right))
        .find(|order| order.is_ne())
        .unwrap_or(Ordering::Equal)
}

/// INFERRED window: the supplied calendar date through the last day of its month next
/// year, intersected with the hard limits. Reject strictly before or strictly after.
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
    compare(date, lower) != Ordering::Less && compare(date, upper) != Ordering::Greater
}

/// WM41: target-month construction clamps to November 2004 through December 2030.
pub(super) fn clamp_month_index(month_index: i64) -> i64 {
    month_index.clamp(2004 * 12 + 10, 2030 * 12 + 11)
}
