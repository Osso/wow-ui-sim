//! Explicit simulator inputs for the currently active timed cast.
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::spellcast_events::fire_player_cast_event;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn delay_casting(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(seconds) = stack_val(state, 1) else {
        return Err(runtime_error(
            "DelayCasting requires nonnegative finite seconds",
        ));
    };
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(runtime_error(
            "DelayCasting requires nonnegative finite seconds",
        ));
    }
    let identity = extend_cast(state, seconds)?;
    if let Some((cast_id, spell_id)) = identity {
        fire_player_cast_event(state, "UNIT_SPELLCAST_DELAYED", cast_id, spell_id);
    }
    state.push(Val::Bool(identity.is_some()));
    Ok(1)
}

fn extend_cast(state: &mut LuaState, seconds: f64) -> LuaResult<Option<(u32, u32)>> {
    let mut sim = borrow_state_mut(state)?;
    let Some(cast) = sim.casting.as_mut() else {
        return Ok(None);
    };
    let end = cast.end_time + seconds;
    let delay = cast.delay_time + seconds;
    let finite_timing = (end * 1000.0).is_finite() && (delay * 1000.0).is_finite();
    if !finite_timing {
        return Err(runtime_error("DelayCasting would overflow cast timing"));
    }
    cast.end_time = end;
    cast.delay_time = delay;
    Ok(Some((cast.cast_id, cast.spell_id)))
}

fn take_failed_cast(
    state: &mut LuaState,
) -> LuaResult<Option<crate::lua_api::game_data::CastingState>> {
    let mut sim = borrow_state_mut(state)?;
    let cast = sim.casting.take();
    if cast
        .as_ref()
        .is_some_and(|cast| cast.spell_id == crate::c_api::c_spec::SPEC_ACTIVATION_SPELL_ID)
    {
        sim.player.pending_spec_change = None;
    }
    Ok(cast)
}

pub(super) fn fail_casting(state: &mut LuaState) -> LuaResult<u32> {
    let quiet = match stack_val(state, 1) {
        Val::Nil => false,
        Val::Bool(value) => value,
        _ => return Err(runtime_error("FailCasting quiet must be a boolean")),
    };
    let cast = take_failed_cast(state)?;
    if let Some(cast) = &cast {
        let event = if quiet {
            "UNIT_SPELLCAST_FAILED_QUIET"
        } else {
            "UNIT_SPELLCAST_FAILED"
        };
        fire_player_cast_event(state, event, cast.cast_id, cast.spell_id);
        fire_player_cast_event(state, "UNIT_SPELLCAST_STOP", cast.cast_id, cast.spell_id);
    }
    state.push(Val::Bool(cast.is_some()));
    Ok(1)
}
