//! Ordered, per-environment speech requests without audio playback.
//! Secret text remains unmodeled: rilua unwrap_secret denies tainted callers.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::is_secret_value;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const API_NAME: &str = "C_VoiceChat.SpeakText";

/// Accepted public request, recorded verbatim without synthesis or XML parsing.
#[derive(Clone, Debug, PartialEq)]
pub struct SpeakTextRequest {
    pub voice_id: f64,
    pub text: String,
    pub rate: f64,
    pub volume: f64,
    pub overlap: bool,
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[derive(Clone, Debug, PartialEq)]
pub struct CombatAudioAlertSpeakTextRequest {
    pub text: String,
    pub category: f64,
    pub allow_overlap: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_VoiceChat")?;
    table_set_rust_fn_static(state, namespace, "SpeakText", speak_text)?;
    table_set_rust_fn_static(state, namespace, "StopSpeakingText", stop_speaking_text)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_combat_audio_alert(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_CombatAudioAlert")?;
    table_set_rust_fn_static(state, namespace, "SpeakText", combat_audio_speak_text)
}

fn stop_speaking_text(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.voice_chat_speak_stop_count += 1;
    Ok(0)
}

fn speak_text(state: &mut LuaState) -> LuaResult<u32> {
    reject_secret_arguments(state)?;
    let request = read_public_request(state)?;
    borrow_state_mut(state)?
        .voice_chat_speak_requests
        .push(request);
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
// INFERRED (R88fc, native evidence pending): queue speech and finish on a later tick.
fn combat_audio_speak_text(state: &mut LuaState) -> LuaResult<u32> {
    let request = read_combat_audio_request(state)?;
    let utterance_id = {
        let mut sim = borrow_state_mut(state)?;
        let id = sim.next_combat_audio_utterance_id;
        sim.next_combat_audio_utterance_id = id
            .checked_add(1)
            .ok_or_else(|| runtime_error("C_CombatAudioAlert.SpeakText IDs exhausted"))?;
        sim.combat_audio_speak_requests.push(request);
        sim.pending_combat_audio_utterance_ids.push_back(id);
        id
    };
    state.push(Val::Num(f64::from(utterance_id)));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_combat_audio_request(state: &LuaState) -> LuaResult<CombatAudioAlertSpeakTextRequest> {
    Ok(CombatAudioAlertSpeakTextRequest {
        text: read_combat_audio_text(state)?,
        category: read_combat_audio_category(state)?,
        allow_overlap: read_combat_audio_overlap(state)?,
    })
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_combat_audio_text(state: &LuaState) -> LuaResult<String> {
    match stack_val(state, 1) {
        Val::Str(_) => val_to_string(state, stack_val(state, 1))
            .ok_or_else(|| runtime_error("C_CombatAudioAlert.SpeakText text must be a string")),
        _ => Err(runtime_error(
            "C_CombatAudioAlert.SpeakText text must be a string",
        )),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_combat_audio_category(state: &LuaState) -> LuaResult<f64> {
    match stack_val(state, 2) {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(runtime_error(
            "C_CombatAudioAlert.SpeakText category must be a number",
        )),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_combat_audio_overlap(state: &LuaState) -> LuaResult<bool> {
    match stack_val(state, 3) {
        Val::Nil => Ok(true),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(
            "C_CombatAudioAlert.SpeakText allowOverlap must be a boolean",
        )),
    }
}

fn reject_secret_arguments(state: &LuaState) -> LuaResult<()> {
    // NeverSecret applies to original wrappers, before any conversion or mutation.
    for (index, name) in [(1, "voiceID"), (3, "rate"), (4, "volume"), (5, "overlap")] {
        if is_secret_value(state, stack_val(state, index)) {
            return Err(runtime_error(format!(
                "{API_NAME}: argument #{index} ({name}) must not be secret"
            )));
        }
    }
    // AllowedWhenTainted cannot honestly use unwrap_secret's untainted-only guard.
    // Conservative unmodeled policy: reject secret text even for secure callers.
    // Never decode the payload or clear caller taint to populate public state.
    if is_secret_value(state, stack_val(state, 2)) {
        return Err(runtime_error(format!(
            "{API_NAME}: secret text access is not modeled"
        )));
    }
    Ok(())
}

fn read_public_request(state: &LuaState) -> LuaResult<SpeakTextRequest> {
    let voice_id = read_finite_number(state, 1, "voiceID")?;
    // INFERRED representation: UTF-8 strings only, without numeric coercion.
    let text = match stack_val(state, 2) {
        Val::Str(_) => val_to_string(state, stack_val(state, 2))
            .ok_or_else(|| runtime_error(format!("{API_NAME}: text must be a UTF-8 string")))?,
        _ => return Err(runtime_error(format!("{API_NAME}: text must be a string"))),
    };
    let rate = read_finite_number(state, 3, "rate")?;
    let volume = read_finite_number(state, 4, "volume")?;
    let overlap = match stack_val(state, 5) {
        Val::Bool(overlap) => overlap,
        // Declared default false; INFERRED: explicit nil also requests the default.
        Val::Nil => false,
        _ => {
            return Err(runtime_error(format!(
                "{API_NAME}: overlap must be a boolean"
            )));
        }
    };
    Ok(SpeakTextRequest {
        voice_id,
        text,
        rate,
        volume,
        overlap,
    })
}

fn read_finite_number(state: &LuaState, index: i32, name: &str) -> LuaResult<f64> {
    // INFERRED representation: finite Lua numbers only, no coercion or range policy.
    match stack_val(state, index) {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error(format!(
            "{API_NAME}: {name} must be a finite number"
        ))),
    }
}
