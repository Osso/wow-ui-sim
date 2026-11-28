//! Incoming duel requests declined with `CancelDuel`.
//!
//! Declaration: `DuelInfoDocumentation.lua` `DUEL_REQUESTED(playerName)`.
//! Consumer: `Blizzard_Game/Shared/EventImplementation.lua` shows the
//! `DUEL_REQUESTED` popup, whose Decline button calls `CancelDuel`
//! (`Blizzard_StaticPopup_Game/GameDialogDefs.lua`); `DUEL_FINISHED` hides it.
//! The request shares `SimState::pending_duel` with the existing `AcceptDuel`.
//! INFERRED (R-ag10-4): declining ends the request with one `DUEL_FINISHED`.

use super::{Arg, fire};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, runtime_error};

fn request_duel(state: &mut LuaState) -> LuaResult<u32> {
    let challenger = String::from_stack(state, 1)?;
    if challenger.is_empty() {
        return Err(runtime_error("Usage: A_Admin.RequestDuel(challengerName)"));
    }
    borrow_state_mut(state)?.pending_duel = Some(challenger.clone());
    fire(state, "DUEL_REQUESTED", vec![Arg::Str(challenger)])?;
    Ok(0)
}

fn cancel_duel(state: &mut LuaState) -> LuaResult<u32> {
    let declined = borrow_state_mut(state)?.pending_duel.take().is_some();
    if declined {
        fire(state, "DUEL_FINISHED", Vec::new())?;
    }
    Ok(0)
}

pub(super) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "CancelDuel", cancel_duel)
}

pub(super) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("RequestDuel", request_duel)
}
