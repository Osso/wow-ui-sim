//! Combat / lockdown probe globals backed by `SimState`.
//!
//! Migrates 6 entries off `GLOBAL_FALSE_STUBS` onto real Rust impls:
//!
//! - `InCombatLockdown()`    — `player.in_combat`
//! - `IsEncounterInProgress()` — `world.encounter_in_progress`
//! - `IsResting()`           — `player.is_resting`
//! - `IsFlyableArea()`       — `world.flyable_area`
//! - `IsInInstance()`        — world instance flag plus active Delve on 12.0.5;
//!                              returns the live host instance type.
//! - `IsBattlefieldArena()`  — `world.battlefield_arena`

use crate::c_api::c_instance_encounter::is_encounter_in_progress;
use crate::lua_api::methods::{borrow_state_mut, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn in_combat_lockdown(state: &mut LuaState) -> LuaResult<u32> {
    let in_combat = borrow_state_mut(state)?.player.in_combat;
    state.push(Val::Bool(in_combat));
    Ok(1)
}

fn is_resting(state: &mut LuaState) -> LuaResult<u32> {
    let resting = borrow_state_mut(state)?.player.is_resting;
    state.push(Val::Bool(resting));
    Ok(1)
}

fn is_flyable_area(state: &mut LuaState) -> LuaResult<u32> {
    let flyable = borrow_state_mut(state)?.world.flyable_area;
    state.push(Val::Bool(flyable));
    Ok(1)
}

/// `IsInInstance()` — returns `(isInInstance, instanceType)` from host state.
/// 12.0.5 active Delves are instances; the host supplies the type separately.
fn is_in_instance(state: &mut LuaState) -> LuaResult<u32> {
    let (in_instance, kind) = {
        let st = borrow_state_mut(state)?;
        let active_delve = cfg!(feature = "retail-12-0-5") && st.has_active_delve;
        // INFERRED composition: Delve exit does not clear an independent instance.
        (
            st.world.in_instance || active_delve,
            st.world.instance_type.clone(),
        )
    };
    state.push(Val::Bool(in_instance));
    let kind_val = create_string(state, &kind);
    state.push(kind_val);
    Ok(2)
}

fn is_battlefield_arena(state: &mut LuaState) -> LuaResult<u32> {
    let arena = borrow_state_mut(state)?.world.battlefield_arena;
    state.push(Val::Bool(arena));
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "InCombatLockdown", in_combat_lockdown)?;
    LuaApiMut::register_function(lua, "IsEncounterInProgress", is_encounter_in_progress)?;
    LuaApiMut::register_function(lua, "IsResting", is_resting)?;
    LuaApiMut::register_function(lua, "IsFlyableArea", is_flyable_area)?;
    LuaApiMut::register_function(lua, "IsInInstance", is_in_instance)?;
    LuaApiMut::register_function(lua, "IsBattlefieldArena", is_battlefield_arena)?;
    Ok(())
}
