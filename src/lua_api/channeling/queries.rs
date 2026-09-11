//! Source-consumer contract: stage and hold queries return milliseconds.
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_queries(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetUnitEmpowerStageDuration", stage_duration)?;
    LuaApiMut::register_function(lua, "GetUnitEmpowerHoldAtMaxTime", hold_at_max)
}

fn player(state: &LuaState) -> bool {
    val_to_string(state, stack_val(state, 1)).as_deref() == Some("player")
}

fn stage_duration(state: &mut LuaState) -> LuaResult<u32> {
    if !player(state) {
        return Ok(0);
    }
    let Val::Num(index) = stack_val(state, 2) else {
        return Ok(0);
    };
    let is_finite = index.is_finite();
    let is_integral = index.fract() == 0.0;
    let is_in_range = index >= 0.0;
    let is_valid_index = is_finite && is_integral && is_in_range;
    if !is_valid_index {
        return Ok(0);
    }
    let value = borrow_state(state)?
        .channeling
        .as_ref()
        .and_then(|cast| cast.empower.as_ref())
        .and_then(|timing| timing.stage_durations.get(index as usize))
        .copied();
    if let Some(value) = value {
        state.push(Val::Num(value * 1000.0));
        return Ok(1);
    }
    Ok(0)
}

fn hold_at_max(state: &mut LuaState) -> LuaResult<u32> {
    if !player(state) {
        return Ok(0);
    }
    let value = borrow_state(state)?
        .channeling
        .as_ref()
        .and_then(|cast| cast.empower.as_ref())
        .map(|timing| timing.hold_at_max);
    if let Some(value) = value {
        state.push(Val::Num(value * 1000.0));
        return Ok(1);
    }
    Ok(0)
}
