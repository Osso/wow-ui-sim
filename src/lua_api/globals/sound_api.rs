//! Sound playback API: PlaySound, PlaySoundFile, StopSound, C_Sound namespace.

use super::super::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all sound-related globals and the C_Sound namespace.
pub fn register_sound_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let g = lua.globals();

    g.set("PlaySound", create_play_sound(lua, Rc::clone(&state))?)?;
    g.set("PlaySoundFile", create_play_sound_file(lua, Rc::clone(&state))?)?;
    g.set("StopSound", create_stop_sound(lua, Rc::clone(&state))?)?;

    register_c_sound(lua, state)?;
    register_game_message_info(lua)?;
    Ok(())
}

/// PlaySound(soundKitID, [channel]) -> willPlay, soundHandle
fn create_play_sound(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    lua.create_function(move |_, (id, _channel): (Value, Option<String>)| {
        let soundkit_id = match &id {
            Value::Integer(n) => *n as u32,
            Value::Number(n) => *n as u32,
            _ => return Ok((false, Value::Nil)),
        };
        let mut st = state.borrow_mut();
        let handle = st.sound_manager.as_mut().and_then(|mgr| mgr.play_sound(soundkit_id));
        match handle {
            Some(h) => Ok((true, Value::Integer(h as i64))),
            None => Ok((false, Value::Nil)),
        }
    })
}

/// PlaySoundFile(path_or_id, [channel]) -> willPlay, soundHandle
fn create_play_sound_file(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    lua.create_function(move |_, (path_or_id, _channel): (Value, Option<String>)| {
        match &path_or_id {
            Value::String(s) => {
                let path = s.to_string_lossy().to_string();
                let mut st = state.borrow_mut();
                let handle = st.sound_manager.as_mut().and_then(|mgr| mgr.play_sound_file(&path));
                match handle {
                    Some(h) => Ok((true, Value::Integer(h as i64))),
                    None => Ok((false, Value::Nil)),
                }
            }
            // Numeric FileDataID — not supported
            _ => Ok((false, Value::Nil)),
        }
    })
}

/// StopSound(soundHandle, [fadeTime])
fn create_stop_sound(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    lua.create_function(move |_, (handle, _fade): (Value, Option<f64>)| {
        let h = match &handle {
            Value::Integer(n) => *n as u32,
            Value::Number(n) => *n as u32,
            _ => return Ok(()),
        };
        let mut st = state.borrow_mut();
        if let Some(mgr) = st.sound_manager.as_mut() {
            mgr.stop_sound(h);
        }
        Ok(())
    })
}

/// Register C_Sound namespace with IsPlaying and stub methods.
/// Also includes PlaySound/PlaySoundFile since Sound.lua reassigns globals from C_Sound.
fn register_c_sound(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let snd = lua.create_table()?;

    // C_Sound.PlaySound — Sound.lua does `PlaySound = C_Sound.PlaySound`
    snd.set("PlaySound", create_play_sound(lua, Rc::clone(&state))?)?;
    snd.set("PlaySoundFile", create_play_sound_file(lua, Rc::clone(&state))?)?;

    let st = Rc::clone(&state);
    snd.set("IsPlaying", lua.create_function(move |_, handle: Value| {
        let h = match &handle {
            Value::Integer(n) => *n as u32,
            Value::Number(n) => *n as u32,
            _ => return Ok(false),
        };
        let st = st.borrow();
        Ok(st.sound_manager.as_ref().is_some_and(|mgr| mgr.is_playing(h)))
    })?)?;

    snd.set("GetSoundScaledVolume", lua.create_function(|_, _id: Value| Ok(1.0f64))?)?;
    snd.set("PlayItemSound", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    snd.set("PlayVocalErrorSound", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;

    lua.globals().set("C_Sound", snd)?;
    Ok(())
}

/// GetGameMessageInfo(gameErrorIndex) -> errorName, soundKitID, voiceID
///
/// Returns error string ID and optional sound info for a game error type.
/// UIErrorsFrame.lua calls this after AddMessage to play error sounds.
/// We return nil (MayReturnNothing per API docs) since we don't have the
/// GameErrors.db2 table loaded.
fn register_game_message_info(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "GetGameMessageInfo",
        lua.create_function(|_, _index: Value| Ok(mlua::MultiValue::new()))?,
    )?;
    Ok(())
}
