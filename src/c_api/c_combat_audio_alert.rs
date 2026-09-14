//! Ordinary combat-audio setting state; playback and CVar coupling are unmodeled.

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CombatAudioAlert")?;
    table_set_rust_fn_static(state, namespace, "GetSpeakerSpeed", get_speaker_speed)?;
    table_set_rust_fn_static(state, namespace, "SetSpeakerSpeed", set_speaker_speed)?;
    table_set_rust_fn_static(state, namespace, "GetSpeakerVolume", get_speaker_volume)?;
    table_set_rust_fn_static(state, namespace, "SetSpeakerVolume", set_speaker_volume)?;
    table_set_rust_fn_static(state, namespace, "GetFormatSetting", get_format_setting)?;
    table_set_rust_fn_static(state, namespace, "SetFormatSetting", set_format_setting)
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

fn get_speaker_volume(state: &mut LuaState) -> LuaResult<u32> {
    let volume = borrow_state(state)?.combat_audio_speaker_volume;
    state.push(Val::Num(volume));
    Ok(1)
}

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
