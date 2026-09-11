//! Simulator input validation happens before any cast/channel mutation.
use crate::lua_api::game_data::{CastingState, EmpowerTiming};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

fn seconds(value: Val, positive: bool) -> LuaResult<f64> {
    let Val::Num(value) = value else {
        return Err(runtime_error("channel timing requires a number"));
    };
    let finite_ms = (value * 1000.0).is_finite();
    let valid_sign = if positive { value > 0.0 } else { value >= 0.0 };
    if !finite_ms || !valid_sign {
        return Err(runtime_error(
            "channel timing must be finite and nonnegative; stages and duration must be positive",
        ));
    }
    Ok(value)
}

fn empower_timing(state: &LuaState, index: i32) -> LuaResult<EmpowerTiming> {
    let Val::Table(reference) = stack_val(state, index) else {
        return Err(runtime_error(
            "empower stages must be a dense nonempty array",
        ));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("empower stages table unavailable"))?;
    let stage_durations = read_stages(table)?;
    let hold_at_max = seconds(stack_val(state, index + 1), false)?;
    Ok(EmpowerTiming {
        stage_durations,
        hold_at_max,
    })
}

fn read_stages(table: &rilua::vm::table::Table) -> LuaResult<Vec<f64>> {
    let count = table
        .array_slice()
        .iter()
        .filter(|v| **v != Val::Nil)
        .count()
        + table
            .hash_entries()
            .iter()
            .filter(|(_, v)| *v != Val::Nil)
            .count();
    if count == 0 {
        return Err(runtime_error("empower stages must not be empty"));
    }
    (1..=count)
        .map(|i| seconds(table.get_int(i as i64), true))
        .collect()
}

fn end_time(start: f64, duration: f64, hold: f64) -> LuaResult<f64> {
    let end = start + duration;
    let finite = (end * 1000.0).is_finite() && ((end + hold) * 1000.0).is_finite();
    if !finite {
        return Err(runtime_error("channel deadline would overflow"));
    }
    Ok(end)
}

fn input_cast(
    state: &LuaState,
    empower: Option<EmpowerTiming>,
    duration: f64,
) -> LuaResult<CastingState> {
    let Val::Num(spell) = stack_val(state, 1) else {
        return Err(runtime_error("channel spell ID must be a positive integer"));
    };
    let integer = spell.fract() == 0.0;
    let in_range = (1.0..=u32::MAX as f64).contains(&spell);
    if !integer || !in_range {
        return Err(runtime_error("channel spell ID must be a positive integer"));
    }
    let spell_name = String::from_stack(state, 2)?;
    let icon_path = String::from_stack(state, 3)?;
    let start_time = borrow_state(state)?.start_time.elapsed().as_secs_f64();
    let hold = empower.as_ref().map_or(0.0, |timing| timing.hold_at_max);
    Ok(CastingState {
        spell_id: spell as u32,
        spell_name,
        icon_path,
        start_time,
        end_time: end_time(start_time, duration, hold)?,
        cast_id: 0,
        empower,
        delay_time: 0.0,
    })
}

pub(crate) fn start_channel(state: &mut LuaState) -> LuaResult<u32> {
    let duration = seconds(stack_val(state, 4), true)?;
    let cast = input_cast(state, None, duration)?;
    super::replace(state, cast)?;
    Ok(0)
}

pub(crate) fn start_empower(state: &mut LuaState) -> LuaResult<u32> {
    let timing = empower_timing(state, 4)?;
    let duration = timing.stage_durations.iter().sum();
    let cast = input_cast(state, Some(timing), duration)?;
    super::replace(state, cast)?;
    Ok(0)
}

fn update_timing(
    state: &mut LuaState,
    duration: f64,
    timing: Option<EmpowerTiming>,
) -> LuaResult<Option<(u32, u32)>> {
    let mut sim = borrow_state_mut(state)?;
    let Some(cast) = sim
        .channeling
        .as_mut()
        .filter(|cast| cast.empower.is_some() == timing.is_some())
    else {
        return Ok(None);
    };
    let hold = timing.as_ref().map_or(0.0, |value| value.hold_at_max);
    let end = end_time(cast.start_time, duration, hold)?;
    cast.end_time = end;
    cast.empower = timing;
    Ok(Some((cast.cast_id, cast.spell_id)))
}

fn update(state: &mut LuaState, duration: f64, timing: Option<EmpowerTiming>) -> LuaResult<u32> {
    let empowered = timing.is_some();
    let identity = update_timing(state, duration, timing)?;
    if let Some((id, spell)) = identity {
        let event = if empowered {
            "UNIT_SPELLCAST_EMPOWER_UPDATE"
        } else {
            "UNIT_SPELLCAST_CHANNEL_UPDATE"
        };
        crate::lua_api::spellcast_events::fire_player_cast_event(state, event, id, spell);
    }
    state.push(Val::Bool(identity.is_some()));
    Ok(1)
}

pub(crate) fn update_channel(state: &mut LuaState) -> LuaResult<u32> {
    let duration = seconds(stack_val(state, 1), true)?;
    update(state, duration, None)
}

pub(crate) fn update_empower(state: &mut LuaState) -> LuaResult<u32> {
    let timing = empower_timing(state, 1)?;
    update(state, timing.stage_durations.iter().sum(), Some(timing))
}

pub(crate) fn stop_channel(state: &mut LuaState) -> LuaResult<u32> {
    let complete = match stack_val(state, 1) {
        Val::Nil => false,
        Val::Bool(value) => value,
        _ => return Err(runtime_error("StopChannel complete must be a boolean")),
    };
    let stopped = super::stop(state, complete)?;
    state.push(Val::Bool(stopped));
    Ok(1)
}
