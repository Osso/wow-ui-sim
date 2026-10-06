//! PvP / battlefield verbs that mutate `SimState.battlefield_queue` /
//! `battlefield_minimap_visible` and dispatch `UPDATE_BATTLEFIELD_STATUS`.
//!
//! Migrates 6 entries off `GLOBAL_NIL_STUBS`:
//!
//! - `JoinBattlefield(index, joinAs)`    — queue up, status => `Queued`.
//!                                           Fires `UPDATE_BATTLEFIELD_STATUS`.
//! - `AcceptBattlefieldPort(index, yes)` — consume the port dialog.
//!                                           `yes=truthy` → `Active`; else
//!                                           → `None`. Fires the event.
//! - `LeaveBattlefield()`                — clear the queue. Fires the event.
//! - `QueueForLFG(dungeon)`              — alternate queue source with
//!                                           `name = "LFG Dungeon <id>"`.
//!                                           Status => `Queued`. Fires event.
//! - `ToggleBattlefieldMinimap()`        — flip `battlefield_minimap_visible`.
//!                                           Silent (UI-only).
//! - `RequestBattlefieldPositions()`     — refresh signal. Fires
//!                                           `UPDATE_BATTLEFIELD_SCORE`.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::event::Event;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::BattlefieldStatus;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn push_event(state: &mut LuaState, name: &str) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: name.to_string(),
        args: Vec::new(),
    });
    Ok(())
}

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_truthy(state: &mut LuaState, index: i32) -> bool {
    match stack_val(state, index) {
        Val::Nil => false,
        Val::Bool(b) => b,
        Val::Num(n) => n != 0.0,
        _ => true,
    }
}

/// `JoinBattlefield(index, joinAs)` — enter the queue at `index`
/// (defaults to 1). `joinAs` is accepted but ignored (retail uses it to
/// distinguish party/group join flavour; sim has one queue slot).
pub(super) fn join_battlefield(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(1);
    queue_battlefield(state, index, format!("Battleground {index}"))?;
    Ok(0)
}

/// Places the player in the single modeled battlefield queue slot.
pub(crate) fn queue_battlefield(state: &mut LuaState, index: i32, name: String) -> LuaResult<()> {
    {
        let mut st = borrow_state_mut(state)?;
        st.battlefield_queue.status = BattlefieldStatus::Queued;
        st.battlefield_queue.index = index;
        st.battlefield_queue.name = name;
    }
    push_event(state, "UPDATE_BATTLEFIELD_STATUS")
}

/// `AcceptBattlefieldPort(index, accept)` — accept/decline the port
/// dialog. Truthy `accept` transitions to `Active`; falsy clears to `None`.
fn accept_battlefield_port(state: &mut LuaState) -> LuaResult<u32> {
    let _index = stack_i32(state, 1);
    let accept = stack_truthy(state, 2);
    {
        let mut st = borrow_state_mut(state)?;
        st.battlefield_queue.status = if accept {
            BattlefieldStatus::Active
        } else {
            BattlefieldStatus::None
        };
        if !accept {
            st.battlefield_queue.index = 0;
            st.battlefield_queue.name.clear();
        }
    }
    push_event(state, "UPDATE_BATTLEFIELD_STATUS")?;
    Ok(0)
}

/// `LeaveBattlefield()` — drop the queue slot. Fires
/// `UPDATE_BATTLEFIELD_STATUS` so addons observing the drained edge can
/// reset their UI.
fn leave_battlefield(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut st = borrow_state_mut(state)?;
        st.battlefield_queue.status = BattlefieldStatus::None;
        st.battlefield_queue.index = 0;
        st.battlefield_queue.name.clear();
    }
    push_event(state, "UPDATE_BATTLEFIELD_STATUS")?;
    Ok(0)
}

/// `QueueForLFG(dungeon)` — populate the queue with an LFG-sourced name.
fn queue_for_lfg(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon = stack_i32(state, 1).unwrap_or(0);
    {
        let mut st = borrow_state_mut(state)?;
        st.battlefield_queue.status = BattlefieldStatus::Queued;
        st.battlefield_queue.index = 1;
        st.battlefield_queue.name = format!("LFG Dungeon {dungeon}");
    }
    push_event(state, "UPDATE_BATTLEFIELD_STATUS")?;
    Ok(0)
}

/// `ToggleBattlefieldMinimap()` — flip `battlefield_minimap_visible`.
/// Silent (UI-only toggle; no event in retail either).
fn toggle_battlefield_minimap(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = borrow_state_mut(state)?;
    st.battlefield_minimap_visible = !st.battlefield_minimap_visible;
    Ok(0)
}

/// `RequestBattlefieldPositions()` — server refresh signal.
fn request_battlefield_positions(state: &mut LuaState) -> LuaResult<u32> {
    push_event(state, "UPDATE_BATTLEFIELD_SCORE")?;
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    // Removed in 11.2.7, before the earliest supported retail epoch.
    // C_PvP.JoinBattlefield retains this queue producer; classic keeps the global.
    #[cfg(not(feature = "retail-12-0-0"))]
    LuaApiMut::register_function(lua, "JoinBattlefield", join_battlefield)?;
    LuaApiMut::register_function(lua, "AcceptBattlefieldPort", accept_battlefield_port)?;
    LuaApiMut::register_function(lua, "LeaveBattlefield", leave_battlefield)?;
    LuaApiMut::register_function(lua, "QueueForLFG", queue_for_lfg)?;
    LuaApiMut::register_function(lua, "ToggleBattlefieldMinimap", toggle_battlefield_minimap)?;
    LuaApiMut::register_function(
        lua,
        "RequestBattlefieldPositions",
        request_battlefield_positions,
    )?;
    Ok(())
}
