//! Shared combat-audio throttle settings, independent of playback or CVars.

use crate::lua_api::methods::borrow_state;
#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_CombatAudioAlert")?;
    table_set_rust_fn_static(state, namespace, "GetThrottle", get_throttle)?;
    #[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
    table_set_rust_fn_static(state, namespace, "SetThrottle", set_throttle)?;
    Ok(())
}

fn read_number(state: &LuaState, position: i32, name: &str) -> LuaResult<f64> {
    match stack_val(state, position) {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(rilua::runtime_error(format!(
            "{name} requires a finite number"
        ))),
    }
}

/// Preserve existing simulator setting policy: unset zero; shared across specs.
fn get_throttle(state: &mut LuaState) -> LuaResult<u32> {
    let throttle_type = read_number(state, 1, "Throttle throttleType")? as i32;
    let value = borrow_state(state)?
        .combat_audio_throttles
        .get(&throttle_type)
        .copied()
        .unwrap_or(0.0);
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
fn set_throttle(state: &mut LuaState) -> LuaResult<u32> {
    let throttle_type = read_number(state, 1, "SetThrottle throttleType")? as i32;
    let value = read_number(state, 2, "SetThrottle newVal")?;
    borrow_state_mut(state)?
        .combat_audio_throttles
        .insert(throttle_type, value);
    state.push(Val::Bool(true));
    Ok(1)
}
