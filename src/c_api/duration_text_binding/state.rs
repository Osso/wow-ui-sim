//! Scalar binding state and live duration sampling for the existing provider.
use crate::lua_api::methods::call_function_state;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::table_security::unwrap_secret;
use rilua::vm::{state::LuaState, value::Userdata};
use rilua::{LuaResult, Val, runtime_error};

struct BindingSettings {
    enabled: bool,
    interval: f64,
    modifier: f64,
}

pub(super) fn create(state: &mut LuaState) -> LuaResult<u32> {
    let metatable = rilua::stdlib::new_metatable(state, "DurationBindingSettings")?;
    // INFERRED: retain simulator scalar defaults, not historical client evidence.
    let settings = BindingSettings {
        enabled: true,
        interval: 1.0,
        modifier: 0.0,
    };
    let reference = state
        .gc
        .alloc_userdata(Userdata::with_metatable(Box::new(settings), metatable));
    state.push(Val::Userdata(reference));
    Ok(1)
}

fn read_settings(state: &LuaState, value: Val) -> LuaResult<&BindingSettings> {
    let Val::Userdata(reference) = value else {
        return Err(runtime_error("duration binding settings expected"));
    };
    state
        .gc
        .userdata
        .get(reference)
        .and_then(|object| object.downcast_ref::<BindingSettings>())
        .ok_or_else(|| runtime_error("duration binding settings expected"))
}

fn read_key(state: &LuaState, index: i32) -> LuaResult<String> {
    String::from_stack(state, index)
}

pub(super) fn read(state: &mut LuaState) -> LuaResult<u32> {
    let settings = read_settings(state, stack_val(state, 1))?;
    let value = match read_key(state, 2)?.as_str() {
        "enabled" => Val::Bool(settings.enabled),
        "updateInterval" => Val::Num(settings.interval),
        "timeModifier" => Val::Num(settings.modifier),
        _ => return Err(runtime_error("unknown duration binding setting")),
    };
    state.push(value);
    Ok(1)
}

fn validate_setting(key: &str, value: Val) -> LuaResult<()> {
    let valid = match (key, value) {
        ("enabled", Val::Bool(_)) => true,
        ("updateInterval", Val::Num(number)) => {
            number.is_finite() && (!cfg!(feature = "retail-12-0-7") || number >= 0.0)
        }
        ("timeModifier", Val::Num(number)) => {
            let accepts_legacy_modifier = !cfg!(feature = "retail-12-0-7");
            accepts_legacy_modifier || matches!(number, 0.0 | 1.0)
        }
        _ => false,
    };
    // INFERRED: finite nonnegative interval; enum values RealTime/BaseTime only.
    if valid {
        Ok(())
    } else {
        Err(runtime_error("invalid duration binding setting"))
    }
}

pub(super) fn write(state: &mut LuaState) -> LuaResult<u32> {
    let key = read_key(state, 2)?;
    let value = unwrap_secret(state, stack_val(state, 3))?;
    validate_setting(&key, value)?;
    let Val::Userdata(reference) = stack_val(state, 1) else {
        return Err(runtime_error("duration binding settings expected"));
    };
    let settings = state
        .gc
        .userdata
        .get_mut(reference)
        .and_then(|object| object.downcast_mut::<BindingSettings>())
        .ok_or_else(|| runtime_error("duration binding settings expected"))?;
    match (key.as_str(), value) {
        ("enabled", Val::Bool(enabled)) => settings.enabled = enabled,
        ("updateInterval", Val::Num(interval)) => settings.interval = interval,
        ("timeModifier", Val::Num(modifier)) => settings.modifier = modifier,
        _ => return Err(runtime_error("invalid duration binding setting")),
    }
    Ok(0)
}

/// Authentication precedes all receiver/type validation, including ignored extras.
pub(super) fn authenticate(state: &mut LuaState) -> LuaResult<u32> {
    crate::c_api::duration_clock::authenticate_arguments(state)?;
    let count = state.top - state.base;
    for index in 1..=count {
        let value = unwrap_secret(state, stack_val(state, index as i32))?;
        state.push(value);
    }
    Ok(count as u32)
}

pub(super) fn validate_duration(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::globals::lua_duration_object::require_duration(state, 1)?;
    Ok(0)
}

/// Read the same host scalar settings and existing live duration-clock producer.
pub(super) fn sample_remaining(state: &mut LuaState) -> LuaResult<u32> {
    let settings = read_settings(state, stack_val(state, 1))?;
    let modifier = Val::Num(settings.modifier);
    let duration = stack_val(state, 2);
    let key = state.gc.intern_string(b"GetRemainingDuration");
    let method = state.gettable(duration, Val::Str(key))?;
    let value = call_function_state(state, method, &[duration, modifier])?;
    state.push(value);
    Ok(1)
}
