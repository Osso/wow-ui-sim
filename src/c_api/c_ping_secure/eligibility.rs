//! Shared send eligibility and submission bookkeeping (WM47 Q1/Q2, WM51-F1/F2).

use super::burst::PingCooldownInterval;
use crate::lua_api::globals::group_queries::active_party_count;
use crate::lua_api::globals::group_queries::relationships::{is_group_assistant, is_group_leader};
use crate::lua_api::globals::real::unit_roles::read_role;
use crate::lua_api::methods::{
    borrow_state_mut, call_function_state, create_table, table_get, table_set_static,
};
use crate::lua_api::script_helpers::call_error_handler_state;
use crate::lua_api::state::SimState;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// Enum.PingResult (PingConstantsDocumentation.lua:18-35).
const FAILED_SPAMMING: u8 = 2;
const FAILED_DISABLED_BY_LEADER: u8 = 3;
const FAILED_DISABLED_BY_SETTINGS: u8 = 4;
const FAILED_SQUELCHED: u8 = 6;

/// WM51-F2: a disabled `enablePings` (4), then an active cooldown (2), then in a
/// group the squelch (6) and the leader restriction (3). The player's own
/// squelch flag and the secondary role-flag remap are gaps.
pub(super) fn check_eligibility(state: &mut LuaState) -> LuaResult<Option<u8>> {
    let in_group = active_party_count(state)? > 0;
    let role = read_role(state, "player")?.0;
    let mut sim = borrow_state_mut(state)?;
    if !pings_enabled(&sim) {
        return Ok(Some(FAILED_DISABLED_BY_SETTINGS));
    }
    let now = sim.cached_clock_ms();
    if !sim.ping_burst.admit(now) {
        return Ok(Some(FAILED_SPAMMING));
    }
    if !in_group {
        return Ok(None);
    }
    if sim.social_restrictions.is_squelched {
        return Ok(Some(FAILED_SQUELCHED));
    }
    Ok(restricted_by_leader(&sim, role).then_some(FAILED_DISABLED_BY_LEADER))
}

/// WM51-F2: the CVar's integer must be nonzero, and an absent CVar is enabled.
/// INFERRED: text that is not a number reads as 0.
fn pings_enabled(sim: &SimState) -> bool {
    sim.cvars.get("enablePings").is_none_or(|value| {
        value
            .trim()
            .parse::<f64>()
            .is_ok_and(|number| number as i64 != 0)
    })
}

/// WM51-F2: rank 2 passes. `Lead` (1) denies other ranks, `Assist` (2) denies
/// ranks other than 1, and `TankHealer` (3) denies role index 2. INFERRED: rank
/// 2 is the leader and rank 1 an assistant.
fn restricted_by_leader(sim: &SimState, role: &str) -> bool {
    let rank = if is_group_leader(sim, "player") {
        2
    } else if is_group_assistant(sim, "player") {
        1
    } else {
        0
    };
    if rank == 2 {
        return false;
    }
    match restriction_mode(sim) {
        1 => true,
        2 => rank != 1,
        3 => role_index(role) == 2,
        _ => false,
    }
}

/// WM51-F2: the first set role mask among 2, 4 and 8 gives index 0, 1 or 2,
/// otherwise 3. INFERRED: the masks are tank, healer and damage.
fn role_index(role: &str) -> u8 {
    match role {
        "TANK" => 0,
        "HEALER" => 1,
        "DAMAGER" => 2,
        _ => 3,
    }
}

#[cfg(feature = "retail-12-0-5")]
fn restriction_mode(sim: &SimState) -> u8 {
    sim.party_ping_restriction
}

/// Epochs before C_PartyInfo.SetRestrictPings have no restriction mode.
#[cfg(not(feature = "retail-12-0-5"))]
fn restriction_mode(_sim: &SimState) -> u8 {
    0
}

/// WM47 Q1/WM51-F1: rollover and increment; a limit crossing stores the pair
/// and calls the cooldown-started callback before the last submission time is
/// recorded. There is no network model.
pub(super) fn record_submission(state: &mut LuaState) -> LuaResult<()> {
    let (now, crossing) = {
        let mut sim = borrow_state_mut(state)?;
        let now = sim.cached_clock_ms();
        (now, sim.ping_burst.count_submission(now))
    };
    if let Some(interval) = crossing {
        notify_cooldown_started(state, interval)?;
    }
    borrow_state_mut(state)?.ping_burst.finish_submission(now);
    Ok(())
}

/// WM51-F1: one new {startTimeMs, endTimeMs} table, dispatched synchronously;
/// callback errors go to the error handler.
fn notify_cooldown_started(state: &mut LuaState, interval: PingCooldownInterval) -> LuaResult<()> {
    let callbacks = super::callback_table(state);
    let callback = table_get(state, callbacks, "pingCooldownStarted");
    if !matches!(callback, Val::Function(_)) {
        return Ok(());
    }
    let top = state.top;
    let info = create_table(state);
    // Root the table on the stack while its fields allocate.
    state.push(info);
    let start = Val::Num(f64::from(interval.start_time_ms));
    table_set_static(state, info, "startTimeMs", start);
    let end = Val::Num(f64::from(interval.end_time_ms));
    table_set_static(state, info, "endTimeMs", end);
    let result = call_function_state(state, callback, &[info]);
    state.top = top;
    if let Err(error) = result {
        call_error_handler_state(state, &error.to_string());
    }
    Ok(())
}
