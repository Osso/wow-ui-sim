//! Simulator channel lifecycle; no spell damage or native interrupt attribution.
mod durations;
mod inputs;
mod queries;

pub(crate) use inputs::{
    start_channel, start_empower, stop_channel, update_channel, update_empower,
};
pub(crate) fn register_queries(lua: &mut rilua::Lua) -> rilua::LuaResult<()> {
    queries::register_queries(lua)?;
    durations::register(lua)
}

use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use super::game_data::CastingState;
use super::methods::{borrow_state, borrow_state_mut, create_string};
use super::script_helpers::fire_named_event_state;
use super::spellcast_events::{fire_player_cast_interrupted, player_cast_args};

fn actor_guid(state: &LuaState) -> LuaResult<String> {
    let sim = borrow_state(state)?;
    Ok(super::globals::unit_misc::guid_for_unit(&sim, "player"))
}

fn emit_stop(state: &mut LuaState, cast: &CastingState, complete: bool) -> LuaResult<()> {
    let actor = if complete {
        None
    } else {
        Some(actor_guid(state)?)
    };
    let [unit, guid, spell, bar] =
        player_cast_args(cast.cast_id, cast.spell_id, |s| create_string(state, s));
    let interrupted_by = actor
        .as_deref()
        .map_or(Val::Nil, |s| create_string(state, s));
    if cast.empower.is_some() {
        fire_named_event_state(
            state,
            "UNIT_SPELLCAST_EMPOWER_STOP",
            &[unit, guid, spell, Val::Bool(complete), interrupted_by, bar],
        );
    } else {
        fire_named_event_state(
            state,
            "UNIT_SPELLCAST_CHANNEL_STOP",
            &[unit, guid, spell, interrupted_by, bar],
        );
    }
    Ok(())
}

pub(crate) fn stop(state: &mut LuaState, complete: bool) -> LuaResult<bool> {
    let old = borrow_state_mut(state)?.channeling.take();
    if let Some(cast) = &old {
        emit_stop(state, cast, complete)?;
    }
    Ok(old.is_some())
}

/// Called at the existing OnUpdate boundary, using the same clock as GetTime.
pub(crate) fn tick(state: &mut LuaState) -> LuaResult<()> {
    let old = {
        let mut sim = borrow_state_mut(state)?;
        let now = sim.start_time.elapsed().as_secs_f64();
        let due = sim
            .channeling
            .as_ref()
            .is_some_and(|cast| now >= cast.completion_time());
        if due { sim.channeling.take() } else { None }
    };
    if let Some(cast) = old {
        emit_stop(state, &cast, true)?;
    }
    Ok(())
}

/// A newly installed ordinary cast replaces a channel before publishing START.
pub(crate) fn cancel_for_cast(state: &mut LuaState) -> LuaResult<()> {
    stop(state, false)?;
    Ok(())
}

fn is_current(state: &LuaState, id: u32) -> LuaResult<bool> {
    Ok(borrow_state(state)?
        .channeling
        .as_ref()
        .is_some_and(|cast| cast.cast_id == id))
}

fn install_incoming(
    state: &mut LuaState,
    mut incoming: CastingState,
) -> LuaResult<(Option<CastingState>, Option<CastingState>, u32)> {
    let mut sim = borrow_state_mut(state)?;
    incoming.cast_id = sim.next_cast_id;
    sim.next_cast_id = sim.next_cast_id.wrapping_add(1);
    let id = incoming.cast_id;
    super::spellcast_events::clear_replaced_specialization(&mut sim);
    let old_cast = sim.casting.take();
    let old_channel = sim.channeling.replace(incoming);
    Ok((old_cast, old_channel, id))
}

fn replace(state: &mut LuaState, incoming: CastingState) -> LuaResult<()> {
    let spell = incoming.spell_id;
    let empowered = incoming.empower.is_some();
    let (old_cast, old_channel, id) = install_incoming(state, incoming)?;
    if let Some(cast) = old_cast {
        let actor = actor_guid(state)?;
        fire_player_cast_interrupted(state, cast.cast_id, cast.spell_id, &actor);
    }
    if let Some(cast) = old_channel {
        emit_stop(state, &cast, false)?;
    }
    if is_current(state, id)? {
        let event = if empowered {
            "UNIT_SPELLCAST_EMPOWER_START"
        } else {
            "UNIT_SPELLCAST_CHANNEL_START"
        };
        super::spellcast_events::fire_player_cast_event(state, event, id, spell);
    }
    Ok(())
}
