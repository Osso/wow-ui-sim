//! Group/party verbs that mutate `SimState.party_members` /
//! `party_group_active` and dispatch `GROUP_ROSTER_UPDATE` /
//! `READY_CHECK` events.
//!
//! Migrates 7 entries off `GLOBAL_NIL_STUBS` and adds one new global
//! (`InviteToGroup` was unregistered):
//!
//! - `InviteToGroup(name)`     — appends a synthesized member, sets
//!                                `party_group_active`, fires ROSTER.
//! - `AcceptGroup()`           — sets `party_group_active`, fires ROSTER.
//! - `DeclineGroup()`          — silent no-op (dialog UI decision only).
//! - `LeaveParty()`            — clears members + `party_group_active`,
//!                                fires ROSTER.
//! - `RemoveFromParty(name)`   — remove by name, fires ROSTER.
//! - `UninviteUnit(unit)`      — remove by unit token (partyN or name),
//!                                fires ROSTER.
//! - `KickUnit(unit)`          — alias of `UninviteUnit`.
//! - `ReadyCheck()`            — fires `READY_CHECK`.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::event::Event;
use crate::lua_api::game_data::PartyMember;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::globals::unit_api::parse_party_index;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

const DEFAULT_INVITE_HEALTH: i32 = 100_000;
const DEFAULT_INVITE_POWER: i32 = 10_000;

pub(crate) fn push_event(state: &mut LuaState, name: &str) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: name.to_string(),
        args: Vec::new(),
    });
    Ok(())
}

fn required_string(state: &mut LuaState, index: i32) -> Option<String> {
    Option::<String>::from_stack(state, index)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
}

fn synthesize_party_member(name: String) -> PartyMember {
    PartyMember {
        map_position: None,
        name,
        name_cached: true,
        connected: true,
        class_index: 1,
        level: 80,
        health: DEFAULT_INVITE_HEALTH,
        health_max: DEFAULT_INVITE_HEALTH,
        power: DEFAULT_INVITE_POWER,
        power_max: DEFAULT_INVITE_POWER,
        power_type: 0,
        power_type_name: "MANA".to_string(),
        is_leader: false,
        dead_since: None,
        buffs: vec![],
        debuffs: vec![],
    }
}

/// `InviteToGroup(name)` — append a synthesized party member, flag the
/// group active. Silent no-op on empty / non-string input.
fn invite_to_group(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    {
        let mut st = borrow_state_mut(state)?;
        st.party_members.push(synthesize_party_member(name));
        st.party_group_active = true;
    }
    push_event(state, "GROUP_ROSTER_UPDATE")?;
    Ok(0)
}

/// `AcceptGroup()` — flag the group active. Keeps existing roster (if any).
fn accept_group(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.party_group_active = true;
    push_event(state, "GROUP_ROSTER_UPDATE")?;
    Ok(0)
}

/// `DeclineGroup()` — silent no-op (retail just dismisses the popup).
fn decline_group(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

/// Clear roster and group metadata when the player leaves the group.
pub(crate) fn clear_party_roster(state: &mut LuaState) -> LuaResult<()> {
    let mut st = borrow_state_mut(state)?;
    st.party_members.clear();
    st.party_group_active = false;
    st.party_leader_index = None;
    st.is_party_lfg = false;
    st.everyone_assistant = false;
    #[cfg(feature = "retail-12-0-7")]
    {
        st.party_assistants.clear();
        st.party_assistant_exclusions.clear();
    }
    Ok(())
}

/// `LeaveParty()` — clear members + active flag.
fn leave_party(state: &mut LuaState) -> LuaResult<u32> {
    clear_party_roster(state)?;
    push_event(state, "GROUP_ROSTER_UPDATE")?;
    Ok(0)
}

/// `RemoveFromParty(name)` — drop the first member whose name matches.
/// Silent no-op when missing or not present.
fn remove_from_party(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    let removed = {
        let mut st = borrow_state_mut(state)?;
        let before = st.party_members.len();
        st.party_members.retain(|m| m.name != name);
        before != st.party_members.len()
    };
    if removed {
        push_event(state, "GROUP_ROSTER_UPDATE")?;
    }
    Ok(0)
}

/// `UninviteUnit(unit)` — drop by unit token (`party1`..`party4`) or by
/// player name. Silent no-op when input is missing or unknown.
fn uninvite_unit(state: &mut LuaState) -> LuaResult<u32> {
    let Some(token) = required_string(state, 1) else {
        return Ok(0);
    };
    let removed = {
        let mut st = borrow_state_mut(state)?;
        if let Some(index) = parse_party_index(&token)
            && index < st.party_members.len()
        {
            st.party_members.remove(index);
            true
        } else {
            let before = st.party_members.len();
            st.party_members.retain(|m| m.name != token);
            before != st.party_members.len()
        }
    };
    if removed {
        push_event(state, "GROUP_ROSTER_UPDATE")?;
    }
    Ok(0)
}

/// `KickUnit(unit)` — alias for `UninviteUnit`.
fn kick_unit(state: &mut LuaState) -> LuaResult<u32> {
    uninvite_unit(state)
}

pub(crate) fn start_ready_check(state: &mut LuaState) -> LuaResult<()> {
    #[cfg(feature = "retail-12-0-5")]
    crate::c_api::c_chat_info::reject_chat_messaging_lockdown(state, "Start ready check")?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.ready_check.active = true;
        sim.ready_check.response = None;
    }
    dispatch_event_now(state, "READY_CHECK", &[])
}

pub(crate) fn confirm_ready_check(state: &mut LuaState, is_ready: bool) -> LuaResult<()> {
    #[cfg(feature = "retail-12-0-5")]
    crate::c_api::c_chat_info::reject_chat_messaging_lockdown(state, "Confirm ready check")?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.ready_check.active = false;
        sim.ready_check.response = Some(is_ready);
    }
    let player = create_string(state, "player");
    dispatch_event_now(
        state,
        "READY_CHECK_CONFIRM",
        &[player, rilua::Val::Bool(is_ready)],
    )?;
    dispatch_event_now(state, "READY_CHECK_FINISHED", &[])
}

fn get_ready_check_status(state: &mut LuaState) -> LuaResult<u32> {
    let (active, response) = {
        let sim = borrow_state(state)?;
        (sim.ready_check.active, sim.ready_check.response)
    };
    let status = match response {
        Some(true) => Some("ready"),
        Some(false) => Some("notready"),
        None if active => Some("waiting"),
        None => None,
    };
    match status {
        Some(value) => {
            let value = create_string(state, value);
            state.push(value);
        }
        None => state.push(rilua::Val::Nil),
    }
    Ok(1)
}

fn get_ready_check_time_left(state: &mut LuaState) -> LuaResult<u32> {
    let time_left = if borrow_state(state)?.ready_check.active {
        30.0
    } else {
        0.0
    };
    state.push(rilua::Val::Num(time_left));
    Ok(1)
}

/// `ReadyCheck()` — start a ready check and fire `READY_CHECK`.
fn ready_check(state: &mut LuaState) -> LuaResult<u32> {
    start_ready_check(state)?;
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "InviteToGroup", invite_to_group)?;
    LuaApiMut::register_function(lua, "AcceptGroup", accept_group)?;
    LuaApiMut::register_function(lua, "DeclineGroup", decline_group)?;
    LuaApiMut::register_function(lua, "LeaveParty", leave_party)?;
    LuaApiMut::register_function(lua, "RemoveFromParty", remove_from_party)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    LuaApiMut::register_function(lua, "UninviteUnit", uninvite_unit)?;
    LuaApiMut::register_function(lua, "KickUnit", kick_unit)?;
    LuaApiMut::register_function(lua, "ReadyCheck", ready_check)?;
    LuaApiMut::register_function(lua, "GetReadyCheckStatus", get_ready_check_status)?;
    LuaApiMut::register_function(lua, "GetReadyCheckTimeLeft", get_ready_check_time_left)?;
    Ok(())
}
