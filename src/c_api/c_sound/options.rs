//! `C_Sound.PlaySoundWithOptions`: the parsed `PlaySoundParams` request is
//! recorded on simulator state and forwarded to the audio sink when one exists.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state_mut, table_get, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// One `PlaySoundParams` request as passed by Lua (SoundDocumentation.lua).
#[derive(Clone, Debug, PartialEq)]
pub struct PlaySoundRequest {
    pub sound_kit_id: u32,
    /// `UISoundSubType` channel name; `None` selects the documented default.
    pub ui_sound_sub_type: Option<String>,
    pub force_no_duplicates: bool,
    pub run_finish_callback: bool,
    pub override_priority: Option<f64>,
    /// Linear 0..1 volume scale applied to this one sound.
    pub volume_override: Option<f64>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Sound")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "PlaySoundWithOptions",
        play_sound_with_options,
    )
}

fn field(state: &mut LuaState, params: Val, name: &str) -> Val {
    table_get(state, params, name)
}

fn optional_number(state: &mut LuaState, params: Val, name: &str) -> LuaResult<Option<f64>> {
    match field(state, params, name) {
        Val::Nil => Ok(None),
        Val::Num(value) if value.is_finite() => Ok(Some(value)),
        _ => Err(runtime_error(format!(
            "PlaySoundParams.{name} must be a number"
        ))),
    }
}

fn defaulted_bool(state: &mut LuaState, params: Val, name: &str) -> LuaResult<bool> {
    match field(state, params, name) {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "PlaySoundParams.{name} must be a boolean"
        ))),
    }
}

fn read_request(state: &mut LuaState, params: Val) -> LuaResult<PlaySoundRequest> {
    if !matches!(params, Val::Table(_)) {
        return Err(runtime_error(
            "PlaySoundWithOptions requires a PlaySoundParams table",
        ));
    }
    let Some(sound_kit_id) = optional_number(state, params, "soundKitID")? else {
        return Err(runtime_error("PlaySoundParams.soundKitID is required"));
    };
    let sub_type = field(state, params, "uiSoundSubType");
    let ui_sound_sub_type = match sub_type {
        Val::Nil => None,
        _ => val_to_string(state, sub_type),
    };
    Ok(PlaySoundRequest {
        sound_kit_id: sound_kit_id as u32,
        ui_sound_sub_type,
        force_no_duplicates: defaulted_bool(state, params, "forceNoDuplicates")?,
        run_finish_callback: defaulted_bool(state, params, "runFinishCallback")?,
        override_priority: optional_number(state, params, "overridePriority")?,
        volume_override: optional_number(state, params, "volumeOverride")?,
    })
}

fn play_sound_with_options(state: &mut LuaState) -> LuaResult<u32> {
    let params = stack_val(state, 1);
    let request = read_request(state, params)?;
    let mut sim = borrow_state_mut(state)?;
    sim.last_sound_kit_requested = Some(request.sound_kit_id);
    let handle = sim.sound_manager.as_mut().and_then(|manager| {
        let handle = manager.play_sound(request.sound_kit_id)?;
        if let Some(volume) = request.volume_override {
            manager.set_volume(handle, volume as f32);
        }
        Some(handle)
    });
    sim.last_sound_request = Some(request);
    drop(sim);
    // MayReturnNothing: like the PlaySound global, nothing is returned when
    // no audio sink produced a handle (INFERRED for the headless simulator).
    let Some(handle) = handle else {
        return Ok(0);
    };
    state.push(Val::Bool(true));
    state.push(Val::Num(f64::from(handle)));
    Ok(2)
}
