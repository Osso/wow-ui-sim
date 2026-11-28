//! Arena crowd-control state from GUID-keyed host inputs: the removal spell
//! (`GetArenaCrowdControlInfo`, `RequestCrowdControlSpell`) and its cooldown
//! window (`GetArenaCrowdControlDuration`). Retail 12.0.5+ and Forever 1.60.1
//! PvpInfoDocumentation.lua declare the same members.
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Clone, Copy)]
pub struct CrowdControlWindow {
    pub start_time: f64,
    pub duration: f64,
}

#[derive(Debug, Clone)]
pub struct ArenaOpponent {
    pub guid: String,
    pub class_index: i32,
    pub spec_id: u32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_PvP")?;
    table_set_rust_fn_static(state, ns, "GetArenaCrowdControlDuration", query_duration)?;
    table_set_rust_fn_static(state, ns, "GetArenaCrowdControlInfo", query_info)?;
    table_set_rust_fn_static(state, ns, "RequestCrowdControlSpell", request_spell)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetArenaOpponent", set_arena_opponent)?
        .set_function("SetArenaCrowdControl", set_arena_crowd_control)
}

fn valid_arena_unit(unit: &str) -> bool {
    let Some(index) = unit.strip_prefix("arena") else {
        return false;
    };
    index
        .parse::<u8>()
        .is_ok_and(|value| (1..=5).contains(&value) && index == value.to_string())
}

fn set_arena_opponent(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let exists = bool::from_stack(state, 2)?;
    let guid = Option::<String>::from_stack(state, 3)?;
    let class_index = Option::<i32>::from_stack(state, 4)?;
    let spec_id = Option::<u32>::from_stack(state, 5)?;
    if !valid_arena_unit(&unit) {
        return Err(runtime_error(
            "A_Admin.SetArenaOpponent: unit must be arena1 through arena5",
        ));
    }
    update_arena_opponent(state, &unit, exists, guid, class_index, spec_id)?;
    dispatch_arena_opponent_update(state, &unit)?;
    Ok(0)
}

fn update_arena_opponent(
    state: &mut LuaState,
    unit: &str,
    exists: bool,
    guid: Option<String>,
    class_index: Option<i32>,
    spec_id: Option<u32>,
) -> LuaResult<()> {
    if exists {
        let guid = required_opponent_guid(guid)?;
        let class_index = required_opponent_class(class_index)?;
        let spec_id =
            spec_id.ok_or_else(|| runtime_error("A_Admin.SetArenaOpponent: specID is required"))?;
        replace_arena_opponent(
            state,
            unit,
            ArenaOpponent {
                guid,
                class_index,
                spec_id,
            },
        )?;
    } else {
        validate_absent_opponent_details(guid, class_index, spec_id)?;
        remove_arena_opponent(state, unit)?;
    }
    Ok(())
}

fn validate_absent_opponent_details(
    guid: Option<String>,
    class_index: Option<i32>,
    spec_id: Option<u32>,
) -> LuaResult<()> {
    if guid.is_some() || class_index.is_some() || spec_id.is_some() {
        return Err(runtime_error(
            "A_Admin.SetArenaOpponent: absent opponents require nil details",
        ));
    }
    Ok(())
}

fn replace_arena_opponent(
    state: &mut LuaState,
    unit: &str,
    opponent: ArenaOpponent,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    if let Some(previous) = sim.arena_opponents.insert(unit.to_string(), opponent) {
        sim.arena_crowd_control.remove(&previous.guid);
        sim.arena_crowd_control_spells.remove(&previous.guid);
    }
    Ok(())
}

fn remove_arena_opponent(state: &mut LuaState, unit: &str) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    if let Some(previous) = sim.arena_opponents.remove(unit) {
        sim.arena_crowd_control.remove(&previous.guid);
        sim.arena_crowd_control_spells.remove(&previous.guid);
    }
    Ok(())
}

fn required_opponent_guid(guid: Option<String>) -> LuaResult<String> {
    guid.filter(|guid| !guid.is_empty())
        .ok_or_else(|| runtime_error("A_Admin.SetArenaOpponent: guid must be a nonempty string"))
}

fn required_opponent_class(class_index: Option<i32>) -> LuaResult<i32> {
    let index = class_index
        .ok_or_else(|| runtime_error("A_Admin.SetArenaOpponent: classIndex is required"))?;
    let (name, _, _) = crate::lua_api::game_data::class_info_by_index(index);
    if name == "Unknown" {
        return Err(runtime_error(
            "A_Admin.SetArenaOpponent: classIndex is out of range",
        ));
    }
    Ok(index)
}

/// INFERRED R88g9 (native evidence pending): supplied host roster changes signal these events.
fn dispatch_arena_opponent_update(state: &mut LuaState, unit: &str) -> LuaResult<()> {
    let value = Val::Str(state.gc.intern_string(unit.as_bytes()));
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "ARENA_PREP_OPPONENT_SPECIALIZATIONS",
        &[],
    )?;
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "ARENA_OPPONENT_UPDATE",
        &[value],
    )
}

fn active_window(sim: &SimState, guid: &str) -> Option<CrowdControlWindow> {
    let now = sim.clock_elapsed().as_secs_f64();
    sim.arena_crowd_control
        .get(guid)
        .copied()
        .filter(|window| window.duration > 0.0 && now < window.start_time + window.duration)
}

fn unit_guid(sim: &SimState, unit: &str) -> Option<String> {
    crate::lua_api::globals::unit_misc::existing_guid_for_unit(sim, unit)
}

fn query_duration(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let window = {
        let sim = borrow_state(state)?;
        unit_guid(&sim, &unit).and_then(|guid| active_window(&sim, &guid))
    };
    // INFERRED: unknown or inactive units receive an empty duration object.
    // Native arena CC selection, production and secrecy remain unmodeled.
    let (start, duration) =
        window.map_or((0.0, 0.0), |window| (window.start_time, window.duration));
    crate::lua_api::globals::lua_duration_object::push_timed_duration_object(state, start, duration)
}

/// `spellID, startTime, duration` in milliseconds, as the original
/// `ArenaUnitFrameCcRemoverMixin:UpdateCooldown` divides them by 1000.
/// INFERRED R88g9 (native evidence pending): a unit without a supplied spell
/// returns nothing; a known spell without an active window reports `0, 0`.
fn query_info(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let info = {
        let sim = borrow_state(state)?;
        unit_guid(&sim, &unit).and_then(|guid| {
            let spell = *sim.arena_crowd_control_spells.get(&guid)?;
            let window = active_window(&sim, &guid);
            Some((
                spell,
                window.map_or((0.0, 0.0), |w| (w.start_time, w.duration)),
            ))
        })
    };
    let Some((spell, (start, duration))) = info else {
        return Ok(0);
    };
    state.push(Val::Num(f64::from(spell)));
    state.push(Val::Num(start * 1000.0));
    state.push(Val::Num(duration * 1000.0));
    Ok(3)
}

/// Fires the synchronous `ARENA_CROWD_CONTROL_SPELL_UPDATE(unit, spellID)`
/// (UnitDocumentation.lua:3410-3419) when the unit's spell is known.
/// INFERRED R88g9: an unknown unit or spell fires nothing.
fn request_spell(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let spell = {
        let sim = borrow_state(state)?;
        unit_guid(&sim, &unit).and_then(|guid| sim.arena_crowd_control_spells.get(&guid).copied())
    };
    if let Some(spell) = spell {
        let unit = Val::Str(state.gc.intern_string(unit.as_bytes()));
        crate::lua_api::globals::state_backed_queries::dispatch_event_now(
            state,
            "ARENA_CROWD_CONTROL_SPELL_UPDATE",
            &[unit, Val::Num(f64::from(spell))],
        )?;
    }
    Ok(0)
}

/// `A_Admin.SetArenaCrowdControl(unit, spellID[, startTime, duration])`:
/// supplies the unit's removal spell and its cooldown window in `GetTime`
/// seconds. A spell ID of 0 clears the unit.
fn set_arena_crowd_control(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let spell = i32::from_stack(state, 2)?;
    let start = Option::<f64>::from_stack(state, 3)?.unwrap_or(0.0);
    let duration = Option::<f64>::from_stack(state, 4)?.unwrap_or(0.0);
    validate_crowd_control(spell, start, duration)?;
    update_crowd_control(state, &unit, spell, start, duration)?;
    if valid_arena_unit(&unit) {
        dispatch_crowd_control_update(state, &unit, spell)?;
    }
    Ok(0)
}

fn validate_crowd_control(spell: i32, start: f64, duration: f64) -> LuaResult<()> {
    if spell < 0 || !start.is_finite() || start < 0.0 || !duration.is_finite() || duration < 0.0 {
        return Err(runtime_error(
            "A_Admin.SetArenaCrowdControl: invalid spell or cooldown window",
        ));
    }
    Ok(())
}

fn update_crowd_control(
    state: &mut LuaState,
    unit: &str,
    spell: i32,
    start: f64,
    duration: f64,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let guid = unit_guid(&sim, unit)
        .ok_or_else(|| runtime_error("A_Admin.SetArenaCrowdControl: unknown unit"))?;
    if spell == 0 {
        sim.arena_crowd_control_spells.remove(&guid);
        sim.arena_crowd_control.remove(&guid);
    } else {
        sim.arena_crowd_control_spells.insert(guid.clone(), spell);
        sim.arena_crowd_control.insert(
            guid.clone(),
            CrowdControlWindow {
                start_time: start,
                duration,
            },
        );
    }
    Ok(())
}

/// INFERRED R88g9 (native evidence pending): a supplied spell change signals the unit event.
fn dispatch_crowd_control_update(state: &mut LuaState, unit: &str, spell: i32) -> LuaResult<()> {
    let token = Val::Str(state.gc.intern_string(unit.as_bytes()));
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "ARENA_CROWD_CONTROL_SPELL_UPDATE",
        &[token, Val::Num(f64::from(spell))],
    )
}
