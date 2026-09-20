//! Read-only interaction capabilities on existing resolved unit snapshots.
use crate::lua_api::game_data::UnitInteraction;
use crate::lua_api::globals::targeting_verbs::resolve_unit_snapshot;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::{LuaApiMut, LuaResult, Val, vm::state::LuaState};

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    LuaApiMut::register_function(state, "UnitIsGameObject", is_game_object)?;
    if cfg!(feature = "client-wowforever") {
        LuaApiMut::register_function(state, "UnitHasLootInteraction", has_loot)?;
        LuaApiMut::register_function(state, "UnitIsInInteractRange", in_range)?;
        LuaApiMut::register_function(state, "UnitIsInteractable", interactable)?;
    }
    Ok(())
}

fn query(state: &mut LuaState, predicate: fn(UnitInteraction) -> bool) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let result = resolve_unit_snapshot(&borrow_state(state)?, &unit)
        .is_some_and(|unit| predicate(unit.interaction));
    state.push(Val::Bool(result));
    Ok(1)
}

fn is_game_object(state: &mut LuaState) -> LuaResult<u32> {
    query(state, |value| value.is_game_object)
}
fn has_loot(state: &mut LuaState) -> LuaResult<u32> {
    query(state, |value| value.has_loot)
}
fn in_range(state: &mut LuaState) -> LuaResult<u32> {
    query(state, |value| value.in_range)
}
fn interactable(state: &mut LuaState) -> LuaResult<u32> {
    query(state, |value| value.interactable)
}
