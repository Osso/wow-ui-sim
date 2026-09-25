//! Player combo points: retail owns the pool, Forever assigns it to a target.

use crate::lua_api::globals::targeting_verbs::resolve_unit_snapshot;
use crate::lua_api::globals::unit_misc::guid_for_unit;
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::SimState;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const COMBO_POINTS_POWER_TYPE: i32 = 4;

/// Forever simulator policy: a power input assigns the existing pool to the selected target.
/// Selection changes alone leave UnitPower's snapshot and this assignment untouched.
#[cfg(feature = "client-wowforever")]
pub(crate) fn bind_player_combo_points(sim: &mut SimState) {
    let has_points = sim
        .player
        .secondary_powers
        .get(&COMBO_POINTS_POWER_TYPE)
        .is_some_and(|pool| pool.current != 0);
    sim.player.combo_points_target_guid = if has_points {
        sim.current_target
            .as_ref()
            .map(|target| target.guid.clone())
    } else {
        None
    };
}

fn read_player_combo_points(sim: &SimState, unit: &str, _target: &str) -> LuaResult<i32> {
    let Some(owner) = resolve_unit_snapshot(sim, unit) else {
        return Ok(0);
    };
    #[cfg(feature = "client-wowforever")]
    let Some(target) = resolve_unit_snapshot(sim, _target) else {
        return Ok(0);
    };
    if owner.guid != guid_for_unit(sim, "player") {
        return Err(runtime_error(
            "GetComboPoints currently supports the modeled player owner only",
        ));
    }
    #[cfg(feature = "client-wowforever")]
    if sim.player.combo_points_target_guid.as_deref() != Some(target.guid.as_str()) {
        return Ok(0);
    }
    Ok(sim
        .player
        .secondary_powers
        .get(&COMBO_POINTS_POWER_TYPE)
        .map_or(0, |pool| pool.current))
}

fn get_combo_points(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let target = String::from_stack(state, 2)?;
    let sim = borrow_state(state)?;
    let points = read_player_combo_points(&sim, &unit, &target)?;
    drop(sim);
    state.push(Val::Num(points as f64));
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetComboPoints", get_combo_points)
}
