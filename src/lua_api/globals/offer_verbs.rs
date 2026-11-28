//! Ready-check / resurrect / duel verbs that consume pending-offer
//! slots on SimState and dispatch the matching WoW events.
//!
//! Migrates 6 entries off `GLOBAL_NIL_STUBS`:
//!
//! - `AcceptDuel()`           — consume `pending_duel`, fire `DUEL_FINISHED`.
//! - `DeclineDuel()`          — same as AcceptDuel for sim purposes.
//! - `AcceptResurrect()`      — consume `pending_resurrect`, revive the
//!                                player (clear `dead_since`), fire
//!                                `PLAYER_ALIVE`.
//! - `DeclineResurrect()`     — clear `pending_resurrect` silently.
//! - `RetrieveCorpse()`       — when `corpse_available`, clear the flag,
//!                                revive the player, fire `PLAYER_ALIVE`
//!                                + `CORPSE_IN_RANGE` (false-edge).
//! - `ResurrectGetOfferer()`  — return `pending_resurrect` or nil.
//!
//! Admin tests seed the pending slots directly on SimState and push the
//! matching *_REQUESTED event; these verbs handle the outbound side.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::event::Event;
use crate::lua_api::methods::{borrow_state_mut, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn push_event(state: &mut LuaState, name: &str) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: name.to_string(),
        args: Vec::new(),
    });
    Ok(())
}

/// A ghost returning to life raises PLAYER_UNGHOST; a corpse revived before
/// release raises PLAYER_ALIVE.
fn push_revival_event(state: &mut LuaState, was_ghost: bool) -> LuaResult<()> {
    push_event(
        state,
        if was_ghost {
            "PLAYER_UNGHOST"
        } else {
            "PLAYER_ALIVE"
        },
    )
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn left_ghost_form(st: &mut crate::lua_api::SimState) -> bool {
    crate::lua_api::globals::real::death_release::leave_ghost_form(st)
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn left_ghost_form(_st: &mut crate::lua_api::SimState) -> bool {
    false
}

/// `AcceptDuel()` — clear the pending offer, fire `DUEL_FINISHED`.
/// Silent no-op when no offer pending.
fn accept_duel(state: &mut LuaState) -> LuaResult<u32> {
    let had_offer = borrow_state_mut(state)?.pending_duel.take().is_some();
    if had_offer {
        push_event(state, "DUEL_FINISHED")?;
    }
    Ok(0)
}

/// `DeclineDuel()` — same shape as `AcceptDuel` for the sim.
fn decline_duel(state: &mut LuaState) -> LuaResult<u32> {
    let had_offer = borrow_state_mut(state)?.pending_duel.take().is_some();
    if had_offer {
        push_event(state, "DUEL_FINISHED")?;
    }
    Ok(0)
}

/// `AcceptResurrect()` — consume the offer, revive the player, fire
/// `PLAYER_ALIVE`.
fn accept_resurrect(state: &mut LuaState) -> LuaResult<u32> {
    let accepted = {
        let mut st = borrow_state_mut(state)?;
        if st.pending_resurrect.take().is_some() {
            st.player.health = st.player.health_max;
            Some(left_ghost_form(&mut st))
        } else {
            None
        }
    };
    if let Some(was_ghost) = accepted {
        push_revival_event(state, was_ghost)?;
    }
    Ok(0)
}

/// `DeclineResurrect()` — clear the offer silently.
fn decline_resurrect(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.pending_resurrect = None;
    Ok(0)
}

/// `RetrieveCorpse()` — when a corpse is available, clear the flag,
/// revive the player, fire `PLAYER_ALIVE` + `CORPSE_IN_RANGE` (false).
fn retrieve_corpse(state: &mut LuaState) -> LuaResult<u32> {
    let retrieved = {
        let mut st = borrow_state_mut(state)?;
        if st.corpse_available {
            st.corpse_available = false;
            st.player.health = st.player.health_max;
            Some(left_ghost_form(&mut st))
        } else {
            None
        }
    };
    if let Some(was_ghost) = retrieved {
        push_revival_event(state, was_ghost)?;
        push_event(state, "CORPSE_IN_RANGE")?;
    }
    Ok(0)
}

/// `ResurrectGetOfferer()` — return the pending offerer name or nil.
fn resurrect_get_offerer(state: &mut LuaState) -> LuaResult<u32> {
    let name = borrow_state_mut(state)?.pending_resurrect.clone();
    match name {
        Some(n) => {
            let val = create_string(state, &n);
            state.push(val);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "AcceptDuel", accept_duel)?;
    LuaApiMut::register_function(lua, "DeclineDuel", decline_duel)?;
    LuaApiMut::register_function(lua, "AcceptResurrect", accept_resurrect)?;
    LuaApiMut::register_function(lua, "DeclineResurrect", decline_resurrect)?;
    LuaApiMut::register_function(lua, "RetrieveCorpse", retrieve_corpse)?;
    LuaApiMut::register_function(lua, "ResurrectGetOfferer", resurrect_get_offerer)?;
    Ok(())
}
