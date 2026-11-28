//! Battleground area spirit healer queue.
//!
//! Declarations: Retail and Forever `PlayerScriptDocumentation.lua`
//! (`AcceptAreaSpiritHeal`, `CancelAreaSpiritHeal`, `GetAreaSpiritHealerTime ->
//! number`, `IsCemeterySelectionAvailable -> bool`) and `DeathInfoDocumentation.lua`
//! (`AREA_SPIRIT_HEALER_IN_RANGE` / `_OUT_OF_RANGE`).
//! Consumer: `Blizzard_Game/Shared/EventImplementation.lua` answers IN_RANGE
//! with `AcceptAreaSpiritHeal()` and the `AREA_SPIRIT_HEAL` popup
//! (`GameDialogDefs.lua`: `OnShow` reads `GetAreaSpiritHealerTime`, button1 is
//! shown by `IsCemeterySelectionAvailable`, Cancel calls `CancelAreaSpiritHeal`).
//! INFERRED (R-ag10-5): the time is the seconds left until the supplied next
//! resurrection wave while the player is queued, and 0 when not queued.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

/// An in-range spirit healer: next wave on the simulator clock, cemetery choice.
#[derive(Clone, Debug, PartialEq)]
pub struct SpiritHealer {
    pub wave_at: f64,
    pub cemetery_selection: bool,
    pub queued: bool,
}

fn now(state: &LuaState) -> LuaResult<f64> {
    Ok(borrow_state(state)?.clock_elapsed().as_secs_f64())
}

fn set_area_spirit_healer(state: &mut LuaState) -> LuaResult<u32> {
    let seconds = Option::<f64>::from_stack(state, 1)?;
    let cemetery_selection = Option::<bool>::from_stack(state, 2)?.unwrap_or(false);
    if seconds.is_some_and(|seconds| !seconds.is_finite() || seconds < 0.0) {
        return Err(runtime_error(
            "Usage: A_Admin.SetAreaSpiritHealer(secondsUntilWave|nil, cemeterySelection)",
        ));
    }
    let healer = match seconds {
        Some(seconds) => Some(SpiritHealer {
            wave_at: now(state)? + seconds,
            cemetery_selection,
            queued: false,
        }),
        None => None,
    };
    let in_range = healer.is_some();
    borrow_state_mut(state)?.interaction_dialogs.spirit_healer = healer;
    let event = if in_range {
        "AREA_SPIRIT_HEALER_IN_RANGE"
    } else {
        "AREA_SPIRIT_HEALER_OUT_OF_RANGE"
    };
    super::fire(state, event, Vec::new())?;
    Ok(0)
}

fn set_queued(state: &mut LuaState, queued: bool) -> LuaResult<u32> {
    if let Some(healer) = borrow_state_mut(state)?
        .interaction_dialogs
        .spirit_healer
        .as_mut()
    {
        healer.queued = queued;
    }
    Ok(0)
}

fn accept_area_spirit_heal(state: &mut LuaState) -> LuaResult<u32> {
    set_queued(state, true)
}

fn cancel_area_spirit_heal(state: &mut LuaState) -> LuaResult<u32> {
    set_queued(state, false)
}

fn get_area_spirit_healer_time(state: &mut LuaState) -> LuaResult<u32> {
    let now = now(state)?;
    let healer = borrow_state(state)?
        .interaction_dialogs
        .spirit_healer
        .clone();
    let remaining = healer
        .filter(|healer| healer.queued)
        .map_or(0.0, |healer| (healer.wave_at - now).max(0.0));
    state.push(Val::Num(remaining));
    Ok(1)
}

fn is_cemetery_selection_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?
        .interaction_dialogs
        .spirit_healer
        .as_ref()
        .is_some_and(|healer| healer.cemetery_selection);
    state.push(Val::Bool(available));
    Ok(1)
}

pub(super) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "AcceptAreaSpiritHeal", accept_area_spirit_heal)?;
    LuaApiMut::register_function(lua, "CancelAreaSpiritHeal", cancel_area_spirit_heal)?;
    LuaApiMut::register_function(lua, "GetAreaSpiritHealerTime", get_area_spirit_healer_time)?;
    LuaApiMut::register_function(
        lua,
        "IsCemeterySelectionAvailable",
        is_cemetery_selection_available,
    )
}

pub(super) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("SetAreaSpiritHealer", set_area_spirit_healer)
}
