//! Player-initiated requests from `PlayerScriptDocumentation.lua` and
//! `UnitRoleDocumentation.lua` (byte-identical Retail/Forever declarations).
//!
//! - `FollowUnit(name, exactMatch)`: `/follow` (`SlashCommands.lua`). Resolves a
//!   unit token or a target/party member name, stores the follow target and
//!   fires `AUTOFOLLOW_BEGIN(name)` for `AutoFollowStatus` (`ZoneText.lua`).
//! - `Dismount()`: `/dismount`. Clears `player.movement.mounted` and fires
//!   `PLAYER_MOUNT_DISPLAY_CHANGED`.
//! - `InitiateRolePoll()`: the raid manager role-poll button. A group leader or
//!   assistant starts the poll: `ROLE_POLL_BEGIN(playerName)` opens
//!   `RolePollPopup` and the call returns true; otherwise it returns false.
//! - `BeginTrade()`: the `TRADE` popup (`TRADE_REQUEST`). Accepting the pending
//!   request opens the trade with the requester and fires `TRADE_SHOW`.
//!
//! INFERRED (R-ag10-11): an unresolved follow name and a request without
//! authority change nothing and fire nothing; `AUTOFOLLOW_END` is the host input
//! `A_Admin.StopAutoFollow` because the client stops following on movement.

use super::interaction_dialogs::{Arg, fire};
use crate::lua_api::globals::unit_api::active_group_member_index;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::{SimState, TradeState};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

/// Requests the player has open with the server.
#[derive(Clone, Debug, Default)]
pub struct PlayerRequests {
    pub follow_target: Option<String>,
    pub trade_request: Option<String>,
}

fn resolve_follow_name(sim: &SimState, name: &str) -> Option<String> {
    let token = if name.is_empty() || name == "0" {
        "target"
    } else {
        name
    };
    let unit_name = match token {
        "target" => sim
            .current_target
            .as_ref()
            .map(|target| target.name.clone()),
        "focus" => sim.current_focus.as_ref().map(|focus| focus.name.clone()),
        unit => active_group_member_index(sim, unit).map(|i| sim.party_members[i].name.clone()),
    };
    unit_name.or_else(|| named_unit(sim, token))
}

fn named_unit(sim: &SimState, name: &str) -> Option<String> {
    let target = sim.current_target.iter().map(|target| &target.name);
    let party = sim.party_group_active.then_some(&sim.party_members);
    let members = party.into_iter().flatten().map(|member| &member.name);
    target
        .chain(members)
        .find(|candidate| candidate.eq_ignore_ascii_case(name))
        .cloned()
}

fn follow_unit(state: &mut LuaState) -> LuaResult<u32> {
    let name = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let resolved = resolve_follow_name(&*borrow_state(state)?, &name);
    let Some(resolved) = resolved else {
        return Ok(0);
    };
    borrow_state_mut(state)?.player_requests.follow_target = Some(resolved.clone());
    fire(state, "AUTOFOLLOW_BEGIN", vec![Arg::Str(resolved)])?;
    Ok(0)
}

fn stop_auto_follow(state: &mut LuaState) -> LuaResult<u32> {
    let was_following = borrow_state_mut(state)?
        .player_requests
        .follow_target
        .take();
    if was_following.is_some() {
        fire(state, "AUTOFOLLOW_END", Vec::new())?;
    }
    Ok(0)
}

fn dismount(state: &mut LuaState) -> LuaResult<u32> {
    let was_mounted = {
        let mut sim = borrow_state_mut(state)?;
        std::mem::replace(&mut sim.player.movement.mounted, false)
    };
    if was_mounted {
        fire(state, "PLAYER_MOUNT_DISPLAY_CHANGED", Vec::new())?;
    }
    Ok(0)
}

fn may_poll_roles(sim: &SimState) -> bool {
    #[cfg(feature = "retail-12-0-7")]
    let assistant = crate::c_api::c_party_info::roles_1207::is_assistant(sim, "player");
    #[cfg(not(feature = "retail-12-0-7"))]
    let assistant = sim.everyone_assistant;
    let grouped = sim.party_group_active && !sim.party_members.is_empty();
    grouped && (sim.party_leader_index.is_none() || assistant)
}

fn initiate_role_poll(state: &mut LuaState) -> LuaResult<u32> {
    let poller = {
        let sim = borrow_state(state)?;
        may_poll_roles(&sim).then(|| sim.player.name.clone())
    };
    let started = poller.is_some();
    if let Some(poller) = poller {
        fire(state, "ROLE_POLL_BEGIN", vec![Arg::Str(poller)])?;
    }
    state.push(Val::Bool(started));
    Ok(1)
}

fn request_trade(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    if name.is_empty() {
        return Err(runtime_error("Usage: A_Admin.RequestTrade(requesterName)"));
    }
    borrow_state_mut(state)?.player_requests.trade_request = Some(name.clone());
    fire(state, "TRADE_REQUEST", vec![Arg::Str(name)])?;
    Ok(0)
}

fn begin_trade(state: &mut LuaState) -> LuaResult<u32> {
    let opened = {
        let mut sim = borrow_state_mut(state)?;
        let request = sim.player_requests.trade_request.take();
        let target = request.filter(|_| sim.active_trade.is_none());
        target.map(|target| {
            sim.active_trade = Some(TradeState {
                target,
                ..TradeState::default()
            })
        })
    };
    if opened.is_some() {
        fire(state, "TRADE_SHOW", Vec::new())?;
    }
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "FollowUnit", follow_unit)?;
    LuaApiMut::register_function(lua, "Dismount", dismount)?;
    LuaApiMut::register_function(lua, "InitiateRolePoll", initiate_role_poll)?;
    LuaApiMut::register_function(lua, "BeginTrade", begin_trade)
}

pub(crate) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("StopAutoFollow", stop_auto_follow)?
        .set_function("RequestTrade", request_trade)
}
