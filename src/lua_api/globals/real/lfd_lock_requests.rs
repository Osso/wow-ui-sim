//! R88i.6 (Retail 12.1.5.70688): asynchronous LFD lock-info requests.
//!
//! Two host-controlled throttle gates are a neutral seam: native windows and
//! their reset policy are UNKNOWN. Both default to admitting, not a guessed
//! duration. Admission updates host diagnostic state without inventing lock
//! data or dispatching LFG_LOCK_INFO_RECEIVED/LFG_UPDATE_RANDOM_INFO.
use crate::lua_api::methods::borrow_state_mut;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

/// Host transport inputs and an observable request outbox, not a server reply.
#[derive(Debug, Default)]
pub struct LfdLockRequestState {
    pub first_throttle_blocked: bool,
    pub second_throttle_blocked: bool,
    pub connection_present: bool,
    /// Host bookkeeping for admitted calls; not a native counter/window claim.
    pub admitted_calls: u64,
    /// Same request kind, with VERIFIED player flag true (1) or false (0).
    pub submitted_player_flags: Vec<bool>,
}

fn request(state: &mut LuaState, player: bool) -> LuaResult<u32> {
    let admitted = {
        let mut sim = borrow_state_mut(state)?;
        let request = &mut sim.lfd_lock_requests;
        let admitted = !request.first_throttle_blocked && !request.second_throttle_blocked;
        if admitted {
            request.admitted_calls = request.admitted_calls.saturating_add(1);
            if request.connection_present {
                request.submitted_player_flags.push(player);
            }
        }
        admitted
    };
    // Supplied arguments are intentionally not parsed (VERIFIED).
    state.push(Val::Bool(admitted));
    // Player arity is VERIFIED. Party full arity is UNKNOWN; one boolean is
    // the explicitly authorized provisional convention, not native evidence.
    Ok(1)
}

fn player(state: &mut LuaState) -> LuaResult<u32> {
    request(state, true)
}

fn party(state: &mut LuaState) -> LuaResult<u32> {
    request(state, false)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "RequestLFDPlayerLockInfo", player)?;
    LuaApiMut::register_function(lua, "RequestLFDPartyLockInfo", party)?;
    Ok(())
}
