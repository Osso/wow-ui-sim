//! Supplied remote (Speak For Me) text-to-speech voice catalog.
//!
//! Retail `VoiceChatDocumentation.lua:381-388` and Forever `:390-397` declare
//! `C_VoiceChat.GetRemoteTtsVoices()` returning `VoiceTtsVoiceType` entries
//! (Retail `:1479-1486`, Forever `:1564-1571`: `voiceID`, `name`).
//! INFERRED R88g9 (native evidence pending): the simulator has no voice
//! service, so the catalog starts empty and changes only through
//! `A_Admin.SetRemoteTtsVoices`. Supplying it does not fire
//! `VOICE_CHAT_TTS_VOICES_UPDATE`.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
    val_to_string,
};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState, namespace: Val) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRemoteTtsVoices",
        get_remote_tts_voices,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetRemoteTtsVoices", set_remote_tts_voices)
}

fn get_remote_tts_voices(state: &mut LuaState) -> LuaResult<u32> {
    let voices = borrow_state(state)?.voice_chat.remote_tts_voices.clone();
    let list = create_table(state);
    for (index, (voice_id, name)) in voices.iter().enumerate() {
        let entry = create_table(state);
        table_set(state, entry, "voiceID", Val::Num(f64::from(*voice_id)));
        let name = create_string(state, name);
        table_set(state, entry, "name", name);
        set_array_entry(state, list, index + 1, entry);
    }
    state.push(list);
    Ok(1)
}

fn set_array_entry(state: &mut LuaState, list: Val, index: usize, value: Val) {
    let Val::Table(list_ref) = list else { return };
    if let Some(table) = state.gc.tables.get_mut(list_ref) {
        let _ = table.raw_set(Val::Num(index as f64), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(list_ref);
}

fn set_remote_tts_voices(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Table(list_ref) = stack_val(state, 1) else {
        return Err(runtime_error(
            "A_Admin.SetRemoteTtsVoices: expected a table",
        ));
    };
    let mut voices = Vec::new();
    for index in 1.. {
        let entry = state
            .gc
            .tables
            .get(list_ref)
            .map_or(Val::Nil, |table| table.get_int(index));
        if matches!(entry, Val::Nil) {
            break;
        }
        voices.push(read_voice(state, entry, index)?);
    }
    borrow_state_mut(state)?.voice_chat.remote_tts_voices = voices;
    Ok(0)
}

fn read_voice(state: &mut LuaState, entry: Val, index: i64) -> LuaResult<(i32, String)> {
    let voice_id = table_get(state, entry, "voiceID");
    let name = table_get(state, entry, "name");
    match (voice_id, val_to_string(state, name), name) {
        (Val::Num(id), Some(name), Val::Str(_)) if id.fract() == 0.0 => Ok((id as i32, name)),
        _ => Err(runtime_error(format!(
            "A_Admin.SetRemoteTtsVoices: entry {index} needs an integer voiceID and a string name"
        ))),
    }
}
