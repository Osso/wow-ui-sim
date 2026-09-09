//! State-backed weather. Native intensity defaults/ranges and event timing are unverified.
use rilua::LuaResult;
use rilua::vm::state::LuaState;

#[cfg(feature = "retail-12-1-5")]
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table_with_fields};
#[cfg(feature = "retail-12-1-5")]
use crate::lua_bridge::{runtime_error, stack_val, table_set_rust_fn_static};
#[cfg(feature = "retail-12-1-5")]
use rilua::Val;

/// Clear/zero is the simulator's initial state, not an observed native default.
#[cfg(feature = "retail-12-1-5")]
#[derive(Clone, Copy, Default)]
pub(crate) struct WeatherState {
    weather_type: u8,
    intensity: f64,
}

#[cfg(feature = "retail-12-1-5")]
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Weather")?;
    table_set_rust_fn_static(state, namespace, "GetCurrentWeather", get_current_weather)
}

#[cfg(not(feature = "retail-12-1-5"))]
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let absent = super::ensure_namespace(state, "__wow_absent_namespaces")?;
    crate::lua_bridge::table_set_static(
        state,
        rilua::Val::Table(absent),
        "C_Weather",
        rilua::Val::Bool(true),
    );
    Ok(())
}

#[cfg(feature = "retail-12-1-5")]
fn get_current_weather(state: &mut LuaState) -> LuaResult<u32> {
    let weather = borrow_state(state)?.weather;
    let info = create_table_with_fields(
        state,
        &[
            ("type", Val::Num(f64::from(weather.weather_type))),
            ("intensity", Val::Num(weather.intensity)),
        ],
    );
    state.push(info);
    Ok(1)
}

/// Admin validation is a simulator contract; mutation never schedules events.
#[cfg(feature = "retail-12-1-5")]
pub(crate) fn set_weather(state: &mut LuaState) -> LuaResult<u32> {
    let (Val::Num(weather_type), Val::Num(intensity)) = (stack_val(state, 1), stack_val(state, 2))
    else {
        return Err(runtime_error(
            "A_Admin.SetWeather requires numeric type and intensity",
        ));
    };
    if !(0.0..=4.0).contains(&weather_type) || weather_type.fract() != 0.0 {
        return Err(runtime_error(
            "A_Admin.SetWeather type must be a WeatherType value (0..4)",
        ));
    }
    if !intensity.is_finite() {
        return Err(runtime_error("A_Admin.SetWeather intensity must be finite"));
    }
    borrow_state_mut(state)?.weather = WeatherState {
        weather_type: weather_type as u8,
        intensity,
    };
    Ok(0)
}
