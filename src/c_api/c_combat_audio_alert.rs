//! Ordinary combat-audio setting state; playback and CVar coupling are unmodeled.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// INFERRED: unconfigured category values are zero; categories are independent,
/// shared across specs, and not coupled to playback or CVars.
#[cfg(feature = "retail-12-0-5")]
#[derive(Debug, Default)]
pub struct CategorySettings {
    pub voices: std::collections::HashMap<i32, f64>,
    pub volumes: std::collections::HashMap<i32, f64>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CombatAudioAlert")?;
    #[cfg(feature = "retail-12-0-5")]
    register_category_settings(state, namespace)?;
    table_set_rust_fn_static(state, namespace, "GetSpeakerSpeed", get_speaker_speed)?;
    table_set_rust_fn_static(state, namespace, "SetSpeakerSpeed", set_speaker_speed)?;
    #[cfg(not(feature = "retail-12-0-5"))]
    {
        table_set_rust_fn_static(state, namespace, "GetSpeakerVolume", get_speaker_volume)?;
        table_set_rust_fn_static(state, namespace, "SetSpeakerVolume", set_speaker_volume)?;
    }
    table_set_rust_fn_static(state, namespace, "GetFormatSetting", get_format_setting)?;
    table_set_rust_fn_static(state, namespace, "SetFormatSetting", set_format_setting)?;
    #[cfg(feature = "retail-12-0-0")]
    {
        table_set_rust_fn_static(state, namespace, "GetSpecSetting", get_spec_setting)?;
        table_set_rust_fn_static(state, namespace, "SetSpecSetting", set_spec_setting)?;
        table_set_rust_fn_static(state, namespace, "GetThrottle", get_throttle)?;
        table_set_rust_fn_static(state, namespace, "SetThrottle", set_throttle)?;
    }
    Ok(())
}

#[cfg(feature = "retail-12-0-5")]
fn register_category_settings(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetCategoryVoice", |s| {
        get_category_setting(s, true)
    })?;
    table_set_rust_fn_static(state, namespace, "GetCategoryVolume", |s| {
        get_category_setting(s, false)
    })?;
    table_set_rust_fn_static(state, namespace, "SetCategoryVoice", |s| {
        set_category_setting(s, true)
    })?;
    table_set_rust_fn_static(state, namespace, "SetCategoryVolume", |s| {
        set_category_setting(s, false)
    })
}

#[cfg(feature = "retail-12-0-5")]
fn get_category_setting(state: &mut LuaState, voice: bool) -> LuaResult<u32> {
    let category = numeric_arg(state, 1, "category")? as i32;
    let value = {
        let sim = borrow_state(state)?;
        let values = if voice {
            &sim.combat_audio_categories.voices
        } else {
            &sim.combat_audio_categories.volumes
        };
        values.get(&category).copied().unwrap_or(0.0)
    };
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn set_category_setting(state: &mut LuaState, voice: bool) -> LuaResult<u32> {
    let category = numeric_arg(state, 1, "category")? as i32;
    let value = numeric_arg(state, 2, "newVal")?;
    let mut sim = borrow_state_mut(state)?;
    let values = if voice {
        &mut sim.combat_audio_categories.voices
    } else {
        &mut sim.combat_audio_categories.volumes
    };
    values.insert(category, value);
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}

fn numeric_arg(state: &LuaState, index: i32, what: &str) -> LuaResult<f64> {
    match stack_val(state, index) {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(rilua::runtime_error(&format!(
            "{what} requires a finite number"
        ))),
    }
}

/// Spec settings belong to the active specialization; switching spec exposes that
/// spec's own (initially zero) values. INFERRED: per-spec scope, unset reads zero.
#[cfg(feature = "retail-12-0-0")]
fn spec_setting_key(state: &LuaState) -> LuaResult<(i32, i32)> {
    let setting = numeric_arg(state, 1, "SpecSetting setting")? as i32;
    let spec = borrow_state(state)?.player.active_spec_index;
    Ok((spec, setting))
}

#[cfg(feature = "retail-12-0-0")]
fn get_spec_setting(state: &mut LuaState) -> LuaResult<u32> {
    let key = spec_setting_key(state)?;
    let value = borrow_state(state)?
        .combat_audio_spec_settings
        .get(&key)
        .copied()
        .unwrap_or(0.0);
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn set_spec_setting(state: &mut LuaState) -> LuaResult<u32> {
    let key = spec_setting_key(state)?;
    let value = numeric_arg(state, 2, "SetSpecSetting newVal")?;
    borrow_state_mut(state)?
        .combat_audio_spec_settings
        .insert(key, value);
    state.push(Val::Bool(true));
    Ok(1)
}

/// Throttles are shared across specs. INFERRED: unset reads zero.
#[cfg(feature = "retail-12-0-0")]
fn get_throttle(state: &mut LuaState) -> LuaResult<u32> {
    let throttle_type = numeric_arg(state, 1, "Throttle throttleType")? as i32;
    let value = borrow_state(state)?
        .combat_audio_throttles
        .get(&throttle_type)
        .copied()
        .unwrap_or(0.0);
    state.push(Val::Num(value));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn set_throttle(state: &mut LuaState) -> LuaResult<u32> {
    let throttle_type = numeric_arg(state, 1, "SetThrottle throttleType")? as i32;
    let value = numeric_arg(state, 2, "SetThrottle newVal")?;
    borrow_state_mut(state)?
        .combat_audio_throttles
        .insert(throttle_type, value);
    state.push(Val::Bool(true));
    Ok(1)
}

fn format_setting_key(state: &LuaState) -> LuaResult<(i32, i32)> {
    let (Val::Num(unit), Val::Num(alert_type)) = (stack_val(state, 1), stack_val(state, 2)) else {
        return Err(rilua::runtime_error(
            "FormatSetting requires numeric unit and alertType",
        ));
    };
    Ok((unit as i32, alert_type as i32))
}

fn get_format_setting(state: &mut LuaState) -> LuaResult<u32> {
    let key = format_setting_key(state)?;
    let value = borrow_state(state)?
        .combat_audio_format_settings
        .get(&key)
        .copied()
        .unwrap_or(0.0);
    state.push(Val::Num(value));
    Ok(1)
}

fn set_format_setting(state: &mut LuaState) -> LuaResult<u32> {
    let key = format_setting_key(state)?;
    let Val::Num(value) = stack_val(state, 3) else {
        return Err(rilua::runtime_error(
            "SetFormatSetting requires numeric newVal",
        ));
    };
    borrow_state_mut(state)?
        .combat_audio_format_settings
        .insert(key, value);
    // Accepted-write policy; native bounds and success semantics are unverified.
    state.push(Val::Bool(true));
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
fn get_speaker_volume(state: &mut LuaState) -> LuaResult<u32> {
    let volume = borrow_state(state)?.combat_audio_speaker_volume;
    state.push(Val::Num(volume));
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
fn set_speaker_volume(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(volume) = stack_val(state, 1) else {
        return Err(rilua::runtime_error("SetSpeakerVolume requires a number"));
    };
    borrow_state_mut(state)?.combat_audio_speaker_volume = volume;
    // Accepted-write policy; native bounds and success semantics are unverified.
    state.push(Val::Bool(true));
    Ok(1)
}

fn get_speaker_speed(state: &mut LuaState) -> LuaResult<u32> {
    let speed = borrow_state(state)?.combat_audio_speaker_speed;
    state.push(Val::Num(speed));
    Ok(1)
}

fn set_speaker_speed(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(speed) = stack_val(state, 1) else {
        return Err(rilua::runtime_error("SetSpeakerSpeed requires a number"));
    };
    borrow_state_mut(state)?.combat_audio_speaker_speed = speed;
    // Accepted-write policy; native bounds and success semantics are unverified.
    state.push(Val::Bool(true));
    Ok(1)
}
