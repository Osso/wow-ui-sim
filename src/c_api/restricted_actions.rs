//! State supplied by the simulator for addon restriction queries.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const INACTIVE: u8 = 0;
const ACTIVATING: u8 = 1;
const ACTIVE_STATE: u8 = 2;
const CHALLENGE_MODE: usize = 2;

pub(crate) fn supported() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !supported() {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_RestrictedActions")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAddOnRestrictionState",
        restriction_state,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsAddOnRestrictionActive",
        restriction_active,
    )
}

fn restriction_type(state: &mut LuaState) -> LuaResult<usize> {
    let value = f64::from_stack(state, 1)?;
    if value.fract() != 0.0 || !(0.0..=5.0).contains(&value) {
        return Err(rilua::runtime_error(
            "expected an addon restriction type from 0 to 5",
        ));
    }
    Ok(value as usize)
}

fn restriction_state(state: &mut LuaState) -> LuaResult<u32> {
    let kind = restriction_type(state)?;
    let value = borrow_state(state)?.restricted_actions[kind];
    state.push(Val::Num(f64::from(value)));
    Ok(1)
}

fn restriction_active(state: &mut LuaState) -> LuaResult<u32> {
    let kind = restriction_type(state)?;
    let active = borrow_state(state)?.restricted_actions[kind] == ACTIVE_STATE;
    state.push(Val::Bool(active));
    Ok(1)
}

pub(crate) fn set_supplied_restriction(state: &mut LuaState) -> LuaResult<u32> {
    let kind = restriction_type(state)?;
    if kind == 0 {
        return Err(rilua::runtime_error(
            "combat restriction follows player combat state",
        ));
    }
    let active = bool::from_stack(state, 2)?;
    set_restriction(state, kind, active)?;
    Ok(0)
}

pub(crate) fn apply_host_event(state: &mut LuaState, event: &str) -> LuaResult<()> {
    let transition = match event {
        "ENCOUNTER_START" => Some((1, true)),
        "ENCOUNTER_END" => Some((1, false)),
        "PVP_MATCH_ACTIVE" => Some((3, true)),
        "PVP_MATCH_COMPLETE" | "PVP_MATCH_INACTIVE" => Some((3, false)),
        _ => None,
    };
    if let Some((kind, active)) = transition {
        set_restriction(state, kind, active)?;
    }
    Ok(())
}

pub(crate) fn set_challenge_mode(state: &mut LuaState, active: bool) -> LuaResult<()> {
    set_restriction(state, CHALLENGE_MODE, active)
}

pub(crate) fn set_restriction(state: &mut LuaState, kind: usize, active: bool) -> LuaResult<()> {
    if !supported() {
        return Ok(());
    }
    let old = borrow_state(state)?.restricted_actions[kind];
    if (old != INACTIVE) == active {
        return Ok(());
    }
    let next = if active { ACTIVATING } else { INACTIVE };
    borrow_state_mut(state)?.restricted_actions[kind] = next;
    dispatch_event_now(
        state,
        "ADDON_RESTRICTION_STATE_CHANGED",
        &[Val::Num(kind as f64), Val::Num(f64::from(next))],
    )?;
    if active && borrow_state(state)?.restricted_actions[kind] == ACTIVATING {
        borrow_state_mut(state)?.restricted_actions[kind] = ACTIVE_STATE;
    }
    Ok(())
}
