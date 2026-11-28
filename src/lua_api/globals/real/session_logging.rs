//! `LoggingChat` / `LoggingCombat` — the client-side chat and combat log file
//! toggles driven by the original `/chatlog` and `/combatlog` slash commands
//! (`SlashCommands.lua`). Both start off; the simulator writes no log file.
//! INFERRED (R-ag10-17): `Logging*(newState)` sets the toggle and returns the
//! resulting state; a call without an argument only reads it. No event fires.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::SimState;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

/// Chat and combat log toggles held in `SimState.session_logging`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SessionLogging {
    pub chat: bool,
    pub combat: bool,
}

fn toggle(state: &mut LuaState, field: fn(&mut SimState) -> &mut bool) -> LuaResult<u32> {
    let requested = match stack_val(state, 1) {
        Val::Nil => None,
        Val::Bool(flag) => Some(flag),
        _ => Some(true),
    };
    let logging = {
        let mut sim = borrow_state_mut(state)?;
        let slot = field(&mut sim);
        if let Some(flag) = requested {
            *slot = flag;
        }
        *slot
    };
    state.push(Val::Bool(logging));
    Ok(1)
}

fn logging_chat(state: &mut LuaState) -> LuaResult<u32> {
    toggle(state, |sim| &mut sim.session_logging.chat)
}

fn logging_combat(state: &mut LuaState) -> LuaResult<u32> {
    toggle(state, |sim| &mut sim.session_logging.combat)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "LoggingChat", logging_chat)?;
    LuaApiMut::register_function(lua, "LoggingCombat", logging_combat)?;
    Ok(())
}
