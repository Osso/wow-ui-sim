//! Local request lifecycle only; cancellation and security policies are inferred.
//! See docs/specs/party-countdown.md. Visual expiry belongs to the GetTime consumer.

use crate::c_api::c_chat_info::reject_chat_messaging_lockdown;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::{borrow_state_mut, create_string};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::is_secret_value;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Debug)]
pub(crate) struct CountdownRequest {
    seconds: f64,
    initiated_by: String,
    initiated_by_name: String,
}

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "DoCountdown", do_countdown)
}

fn do_countdown(state: &mut LuaState) -> LuaResult<u32> {
    reject_chat_messaging_lockdown(state, "DoCountdown")?;
    let seconds = read_seconds(state)?;
    if let Some(request) = commit_request(state, seconds)? {
        publish_request(state, &request)?;
    }
    state.push(Val::Bool(true));
    Ok(1)
}

fn read_seconds(state: &LuaState) -> LuaResult<f64> {
    let value = stack_val(state, 1);
    // Conservative rejection does not unwrap secrets or clear caller taint.
    if is_secret_value(state, value) {
        return Err(runtime_error(
            "DoCountdown: secret seconds access is not modeled",
        ));
    }
    match value {
        Val::Num(seconds) if seconds.is_finite() && (0.0..=3600.0).contains(&seconds) => {
            Ok(seconds)
        }
        _ => Err(runtime_error(
            "DoCountdown: seconds must be a finite public number from 0 to 3600",
        )),
    }
}

fn snapshot_request(sim: &SimState, seconds: f64) -> LuaResult<CountdownRequest> {
    let initiated_by = existing_guid_for_unit(sim, "player")
        .ok_or_else(|| runtime_error("DoCountdown: modeled player GUID is unavailable"))?;
    Ok(CountdownRequest {
        seconds,
        initiated_by,
        initiated_by_name: sim.player.name.clone(),
    })
}

fn commit_request(state: &mut LuaState, seconds: f64) -> LuaResult<Option<CountdownRequest>> {
    let mut sim = borrow_state_mut(state)?;
    if seconds == 0.0 && sim.party_countdown_request.is_none() {
        return Ok(None);
    }
    let request = snapshot_request(&sim, seconds)?;
    // Commit before callbacks: nested requests observe the latest state.
    sim.party_countdown_request = if seconds > 0.0 {
        Some(request.clone())
    } else {
        None
    };
    Ok(Some(request))
}

fn publish_request(state: &mut LuaState, request: &CountdownRequest) -> LuaResult<()> {
    let saved_top = state.top;
    let guid = create_string(state, &request.initiated_by);
    state.push(guid);
    let name = create_string(state, &request.initiated_by_name);
    state.push(name);
    // Local-only policy: never fabricate a group-chat notification.
    let result = if request.seconds > 0.0 {
        dispatch_event_now(
            state,
            "START_PLAYER_COUNTDOWN",
            &[
                guid,
                Val::Num(request.seconds),
                Val::Num(request.seconds),
                Val::Bool(false),
                name,
            ],
        )
    } else {
        dispatch_event_now(
            state,
            "CANCEL_PLAYER_COUNTDOWN",
            &[guid, Val::Bool(false), name],
        )
    };
    state.top = saved_top;
    result
}
