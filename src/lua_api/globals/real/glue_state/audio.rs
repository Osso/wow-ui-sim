//! R88ad.10 Glue music and ambience requests (Retail 12.1.5.70688).
//!
//! Requests are recorded in `GlueAudioState::requests`; the simulator has no
//! Glue audio backend, so handle activity stays a host input.

use super::{GlueAudioHandle, GlueAudioRequest, GlueAudioState};
use crate::c_api::{
    bad_argument, parse_numeric_string, read_untainted_signed32_argument, read_untainted_value,
};
use crate::lua_api::methods::borrow_state_mut;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, RustFn, Val};

const MUSIC_USAGE: &str = "Usage: PlayGlueMusic(soundKitID)";
const AMBIENCE: &str = "PlayGlueAmbience";
const AMBIENCE_USAGE: &str = "Usage: PlayGlueAmbience(soundKitID [, fadeInTime])";
const FLOAT_RANGE: &str = "outside of expected range -3.402823e+38 to 3.402823e+38";

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let globals: [(&str, RustFn); 4] = [
        ("PlayGlueMusic", play_glue_music),
        ("StopGlueMusic", stop_glue_music),
        (AMBIENCE, play_glue_ambience),
        ("StopGlueAmbience", stop_glue_ambience),
    ];
    for (name, function) in globals {
        LuaApiMut::register_function(state, name, function)?;
    }
    Ok(())
}

fn play_glue_music(state: &mut LuaState) -> LuaResult<u32> {
    let kit = read_untainted_signed32_argument(state, 1, "PlayGlueMusic", MUSIC_USAGE)?;
    play_music(&mut borrow_state_mut(state)?.glue_state.audio, kit);
    Ok(0)
}

fn stop_glue_music(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let audio = &mut sim.glue_state.audio;
    stop_music_slot(audio, 0);
    stop_music_slot(audio, 1);
    Ok(0)
}

fn play_glue_ambience(state: &mut LuaState) -> LuaResult<u32> {
    let kit = read_untainted_signed32_argument(state, 1, AMBIENCE, AMBIENCE_USAGE)?;
    let fade_ms = read_fade(state)?.map_or(-1, fade_to_milliseconds);
    play_ambience(&mut borrow_state_mut(state)?.glue_state.audio, kit, fade_ms);
    Ok(0)
}

fn stop_glue_ambience(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let audio = &mut sim.glue_state.audio;
    audio.ambience_stop_marker = true;
    let handle = audio.ambience.id;
    audio
        .requests
        .push(GlueAudioRequest::StopAmbience { handle });
    Ok(0)
}

fn play_music(audio: &mut GlueAudioState, kit: i32) {
    if audio.music_suppressed == [true; 2] || (audio.music.active && audio.music.kit == kit) {
        return;
    }
    if audio.music.active {
        stop_music_slot(audio, 0);
    }
    stop_music_slot(audio, 1);
    if kit <= 0 {
        return;
    }
    audio.requested_music_kit = kit;
    audio.music = next_handle(audio, kit);
    let handle = audio.music.id;
    audio
        .requests
        .push(GlueAudioRequest::StartMusic { kit, handle });
    audio.music_stop_markers[0] = false;
}

/// Slot 0 is the current music handle; slot 1 is the other stored handle.
fn stop_music_slot(audio: &mut GlueAudioState, slot: usize) {
    audio.music_stop_markers[slot] = true;
    let handle = [audio.music, audio.other_music][slot].id;
    audio.requests.push(GlueAudioRequest::StopMusic { handle });
}

fn play_ambience(audio: &mut GlueAudioState, kit: i32, fade_ms: i32) {
    if audio.ambience.active && audio.ambience.kit == kit {
        return;
    }
    if audio.ambience.active {
        audio.ambience_stop_marker = true;
        let handle = audio.ambience.id;
        audio
            .requests
            .push(GlueAudioRequest::StopAmbience { handle });
    }
    if kit <= 0 {
        return;
    }
    audio.ambience_stop_marker = false;
    audio.requested_ambience_kit = kit;
    audio.ambience = next_handle(audio, kit);
    let handle = audio.ambience.id;
    let request = GlueAudioRequest::StartAmbience {
        kit,
        fade_ms,
        handle,
    };
    audio.requests.push(request);
}

fn next_handle(audio: &mut GlueAudioState, kit: i32) -> GlueAudioHandle {
    audio.next_handle += 1;
    GlueAudioHandle {
        id: audio.next_handle,
        kit,
        active: false,
    }
}

/// Optional finite single-precision fade; missing and nil select the default.
fn read_fade(state: &LuaState) -> LuaResult<Option<f32>> {
    if crate::lua_bridge::stack_val(state, 2).is_nil() {
        return Ok(None);
    }
    let (value, marked) = read_untainted_value(state, 2, AMBIENCE, AMBIENCE_USAGE)?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(text) if !marked => state
            .gc
            .string_arena
            .get(text)
            .and_then(|text| parse_numeric_string(text.data())),
        _ => None,
    };
    let reason = match number {
        Some(number) if !number.is_finite() => "must be a finite number",
        Some(number) if number.abs() <= f64::from(f32::MAX) => return Ok(Some(number as f32)),
        _ => FLOAT_RANGE,
    };
    Err(bad_argument(
        2,
        AMBIENCE,
        &format!("{reason} - {AMBIENCE_USAGE}"),
    ))
}

/// Single-precision seconds times 1000, truncated; overflow yields `i32::MIN`.
fn fade_to_milliseconds(seconds: f32) -> i32 {
    let milliseconds = seconds * 1000.0_f32;
    if (-2_147_483_648.0_f32..2_147_483_648.0_f32).contains(&milliseconds) {
        milliseconds as i32
    } else {
        i32::MIN
    }
}
