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

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_PvP")?;
    table_set_rust_fn_static(state, ns, "GetArenaCrowdControlDuration", query_duration)?;
    table_set_rust_fn_static(state, ns, "GetArenaCrowdControlInfo", query_info)?;
    table_set_rust_fn_static(state, ns, "RequestCrowdControlSpell", request_spell)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetArenaCrowdControl", set_arena_crowd_control)
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
    let start_time = Option::<f64>::from_stack(state, 3)?.unwrap_or(0.0);
    let duration = Option::<f64>::from_stack(state, 4)?.unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    let guid = unit_guid(&sim, &unit)
        .ok_or_else(|| runtime_error("A_Admin.SetArenaCrowdControl: unknown unit"))?;
    if spell == 0 {
        sim.arena_crowd_control_spells.remove(&guid);
        sim.arena_crowd_control.remove(&guid);
        return Ok(0);
    }
    sim.arena_crowd_control_spells.insert(guid.clone(), spell);
    let window = CrowdControlWindow {
        start_time,
        duration,
    };
    sim.arena_crowd_control.insert(guid, window);
    Ok(0)
}
