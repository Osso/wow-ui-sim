//! Player-initiated requests from `PlayerScriptDocumentation.lua` and
//! `UnitRoleDocumentation.lua` (byte-identical Retail/Forever declarations).
//!
//! - `InitiateRolePoll()`: the raid manager role-poll button. A group leader or
//!   assistant starts the poll: `ROLE_POLL_BEGIN(playerName)` opens
//!   `RolePollPopup` and the call returns true; otherwise it returns false.
//! - `BeginTrade()`: the `TRADE` popup (`TRADE_REQUEST`). Accepting the pending
//!   request opens the trade with the requester and fires `TRADE_SHOW`.
//!
//! INFERRED (R-ag10-11): a request without authority changes nothing.

use super::interaction_dialogs::{Arg, fire};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::{SimState, TradeState};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

/// Requests the player has open with the server.
#[derive(Clone, Debug, Default)]
pub struct PlayerRequests {
    pub trade_request: Option<String>,
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
    LuaApiMut::register_function(lua, "InitiateRolePoll", initiate_role_poll)?;
    LuaApiMut::register_function(lua, "BeginTrade", begin_trade)
}

pub(crate) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("RequestTrade", request_trade)
}
