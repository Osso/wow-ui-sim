//! System utility functions.
//!
//! This module contains WoW's core system functions including:
//! - `type()` - Type introspection with Frame userdata support
//! - `rawget()` - Raw table access with userdata compatibility
//! - `xpcall()` - Protected call with error handler and varargs (Lua 5.2+ feature)
//! - `SlashCmdList` - Slash command registry table
//! - `FireEvent()` - Simulator utility to fire events for testing
//! - `ReloadUI()` - Reload the interface (fires startup events again)
//! - `GetTime()` - Returns seconds since UI load
//! - Build type checks: `IsPublicTestClient()`, `IsBetaBuild()`, `IsPublicBuild()`
//! - Battle.net stubs: `BNFeaturesEnabled()`, `BNConnected()`, etc.
//! - Streaming stubs: `GetFileStreamingStatus()`, `GetBackgroundLoadingStatus()`

use crate::lua_api::frame::FrameHandle;
use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

/// Register system utility functions in the Lua global namespace.
pub fn register_system_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_type_overrides(lua)?;
    register_xpcall(lua)?;
    register_slash_cmd_list(lua)?;
    register_fire_event(lua, Rc::clone(&state))?;
    register_reload_ui(lua, Rc::clone(&state))?;
    register_build_type_checks(lua)?;
    register_battlenet_stubs(lua)?;
    register_secure_stubs(lua)?;
    register_time_functions(lua)?;
    register_streaming_stubs(lua)?;
    register_error_callstack_stubs(lua)?;
    Ok(())
}

/// Override `type()` and `rawget()` to handle FrameHandle userdata as "table".
fn register_type_overrides(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // Override type() to recognize FrameHandle as "table"
    // Blizzard's Dump.lua does `type(v) == "table"` checks and we want FrameHandle to pass
    let type_fn = lua.create_function(|_lua, value: Value| {
        let type_str = match &value {
            Value::Nil => "nil",
            Value::Boolean(_) => "boolean",
            Value::Integer(_) | Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Function(_) => "function",
            Value::Thread(_) => "thread",
            Value::UserData(ud) => {
                if ud.is::<FrameHandle>() {
                    return Ok("table");
                } else {
                    "userdata"
                }
            }
            Value::LightUserData(_) => "userdata",
            Value::Error(_) => "error",
            Value::Other(_) => "userdata",
        };
        Ok(type_str)
    })?;
    globals.set("type", type_fn)?;

    // Override rawget() to handle userdata gracefully
    // Blizzard's Dump.lua does `rawget(v, 0)` on things that pass `type(v) == "table"`
    let rawget_fn = lua.create_function(|lua, (table, key): (Value, Value)| {
        match table {
            Value::Table(t) => t.raw_get(key),
            Value::UserData(_) => Ok(Value::Nil),
            _ => {
                let original: mlua::Function = lua.globals().raw_get("__original_rawget")?;
                original.call((table, key))
            }
        }
    })?;
    let original_rawget: mlua::Function = globals.raw_get("rawget")?;
    globals.raw_set("__original_rawget", original_rawget)?;
    globals.set("rawget", rawget_fn)?;

    Ok(())
}

/// Override `xpcall()` with varargs support (Lua 5.2+ feature needed by WoW addons).
fn register_xpcall(lua: &Lua) -> Result<()> {
    let xpcall_fn = lua.create_function(|lua, args: mlua::MultiValue| {
        let mut args_vec: Vec<Value> = args.into_iter().collect();
        if args_vec.len() < 2 {
            return Err(mlua::Error::RuntimeError(
                "xpcall requires at least 2 arguments".to_string(),
            ));
        }

        let func = match args_vec.remove(0) {
            Value::Function(f) => f,
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "bad argument #1 to 'xpcall' (function expected)".to_string(),
                ))
            }
        };

        let error_handler = match args_vec.remove(0) {
            Value::Function(f) => f,
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "bad argument #2 to 'xpcall' (function expected)".to_string(),
                ))
            }
        };

        let call_args: mlua::MultiValue = args_vec.into_iter().collect();

        match func.call::<mlua::MultiValue>(call_args) {
            Ok(results) => {
                let mut ret = mlua::MultiValue::new();
                ret.push_back(Value::Boolean(true));
                for v in results {
                    ret.push_back(v);
                }
                Ok(ret)
            }
            Err(e) => {
                let error_msg = lua.create_string(&e.to_string())?;
                let handler_result = error_handler.call::<Value>(Value::String(error_msg));
                let mut ret = mlua::MultiValue::new();
                ret.push_back(Value::Boolean(false));
                match handler_result {
                    Ok(v) => ret.push_back(v),
                    Err(he) => ret.push_back(Value::String(lua.create_string(&he.to_string())?)),
                }
                Ok(ret)
            }
        }
    })?;
    lua.globals().set("xpcall", xpcall_fn)?;
    Ok(())
}

/// Register the `SlashCmdList` table.
fn register_slash_cmd_list(lua: &Lua) -> Result<()> {
    lua.globals().set("SlashCmdList", lua.create_table()?)?;
    Ok(())
}

/// Register `FireEvent()` - simulator utility to fire events for testing.
fn register_fire_event(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let fire_event = lua.create_function(move |lua, args: mlua::Variadic<Value>| {
        let mut args_iter = args.into_iter();
        let event_name: String = match args_iter.next() {
            Some(Value::String(s)) => s.to_str()?.to_string(),
            _ => return Err(mlua::Error::runtime("FireEvent requires event name as first argument")),
        };

        let event_args: Vec<Value> = args_iter.collect();

        let listeners = {
            let state = state.borrow();
            state.widgets.get_event_listeners(&event_name)
        };

        for widget_id in listeners {
            let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();

            if let Some(table) = scripts_table {
                let frame_key = format!("{}_OnEvent", widget_id);
                let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

                if let Some(handler) = handler {
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    let frame: Value = lua.globals().get(frame_ref_key.as_str()).unwrap_or(Value::Nil);

                    let mut call_args = vec![frame, Value::String(lua.create_string(&event_name)?)];
                    call_args.extend(event_args.iter().cloned());

                    handler.call::<()>(mlua::MultiValue::from_vec(call_args)).ok();
                }
            }
        }

        Ok(())
    })?;
    lua.globals().set("FireEvent", fire_event)?;
    Ok(())
}

/// Register `ReloadUI()` - reload the interface by firing startup events again.
fn register_reload_ui(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let reload_ui = lua.create_function(move |lua, ()| {
        fire_event_to_listeners(lua, &state, "ADDON_LOADED", |lua| {
            let event_str = lua.create_string("ADDON_LOADED")?;
            let addon_name = lua.create_string("WoWUISim")?;
            Ok(vec![Value::String(event_str), Value::String(addon_name)])
        })?;

        fire_event_to_listeners(lua, &state, "PLAYER_LOGIN", |lua| {
            let event_str = lua.create_string("PLAYER_LOGIN")?;
            Ok(vec![Value::String(event_str)])
        })?;

        fire_event_to_listeners(lua, &state, "PLAYER_ENTERING_WORLD", |lua| {
            let event_str = lua.create_string("PLAYER_ENTERING_WORLD")?;
            Ok(vec![Value::String(event_str), Value::Boolean(false), Value::Boolean(true)])
        })?;

        state.borrow_mut().console_output.push("UI Reloaded".to_string());
        Ok(())
    })?;
    lua.globals().set("ReloadUI", reload_ui)?;
    Ok(())
}

/// Fire an event to all registered listeners, building extra args via a closure.
fn fire_event_to_listeners<F>(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    event_name: &str,
    build_extra_args: F,
) -> Result<()>
where
    F: Fn(&Lua) -> Result<Vec<Value>>,
{
    let listeners = {
        let state = state.borrow();
        state.widgets.get_event_listeners(event_name)
    };
    for widget_id in listeners {
        if let Ok(Some(table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
            let frame_key = format!("{}_OnEvent", widget_id);
            if let Ok(Some(handler)) = table.get::<Option<mlua::Function>>(frame_key.as_str()) {
                let frame_ref_key = format!("__frame_{}", widget_id);
                if let Ok(frame) = lua.globals().get::<Value>(frame_ref_key.as_str()) {
                    let mut call_args = vec![frame];
                    call_args.extend(build_extra_args(lua)?);
                    let _ = handler.call::<()>(mlua::MultiValue::from_vec(call_args));
                }
            }
        }
    }
    Ok(())
}

/// Register build type check functions.
fn register_build_type_checks(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("IsPublicTestClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsBetaBuild", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsPublicBuild", lua.create_function(|_, ()| Ok(true))?)?;
    Ok(())
}

/// Register Battle.net stub functions.
fn register_battlenet_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("BNFeaturesEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNFeaturesEnabledAndConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNGetFriendInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    globals.set("BNGetNumFriends", lua.create_function(|_, ()| Ok((0, 0)))?)?;
    globals.set("BNGetInfo", lua.create_function(|lua, ()| {
        Ok((
            Value::Integer(0),
            Value::String(lua.create_string("SimPlayer#0000")?),
            Value::Nil,
            Value::String(lua.create_string("")?),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Boolean(false),
        ))
    })?)?;
    Ok(())
}

/// Register secure environment stubs.
fn register_secure_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("SwapToGlobalEnvironment", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("IsGMClient", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("RegisterStaticConstants", lua.create_function(|_, _tbl: Value| Ok(()))?)?;

    // GetFrameMetatable/GetButtonMetatable - return metatables with __index
    for name in &["GetFrameMetatable", "GetButtonMetatable"] {
        globals.set(*name, lua.create_function(|lua, ()| {
            let mt = lua.create_table()?;
            mt.set("__index", lua.create_table()?)?;
            Ok(Value::Table(mt))
        })?)?;
    }

    globals.set("C_GamePad", register_c_gamepad(lua)?)?;
    globals.set("C_AssistedCombat", register_c_assisted_combat(lua)?)?;
    globals.set("C_Widget", register_c_widget(lua)?)?;
    Ok(())
}

/// C_GamePad namespace stubs.
fn register_c_gamepad(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("IsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetActiveDeviceID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetDeviceMappedState", lua.create_function(|_, _id: Option<i32>| Ok(Value::Nil))?)?;
    t.set("SetLedColor", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    t.set("GetConfig", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetCombinedDeviceID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetPowerLevel", lua.create_function(|_, _id: Option<i32>| Ok(Value::Nil))?)?;
    Ok(t)
}

/// C_AssistedCombat namespace stubs.
fn register_c_assisted_combat(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("GetActionSpell", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetNextCastSpell", lua.create_function(|_, _check: Option<bool>| Ok(Value::Nil))?)?;
    t.set("GetRotationSpells", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("IsAvailable", lua.create_function(|lua, ()| {
        Ok((false, Value::String(lua.create_string("Not available")?)))
    })?)?;
    Ok(t)
}

/// C_Widget namespace - widget type checking.
fn register_c_widget(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("IsFrameWidget", lua.create_function(|_, _widget: Value| Ok(false))?)?;
    t.set("IsRenderableWidget", lua.create_function(|_, _widget: Value| Ok(false))?)?;
    t.set("IsWidget", lua.create_function(|_, _widget: Value| Ok(false))?)?;
    Ok(t)
}

/// Register `GetTime()` - returns seconds since UI load.
fn register_time_functions(lua: &Lua) -> Result<()> {
    let start = Instant::now();
    let get_time = lua.create_function(move |_, ()| {
        Ok(start.elapsed().as_secs_f64())
    })?;
    lua.globals().set("GetTime", get_time)?;
    Ok(())
}

/// Register streaming status stubs (simulator has no streaming).
fn register_streaming_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("GetFileStreamingStatus", lua.create_function(|_, ()| Ok(0i32))?)?;
    globals.set("GetBackgroundLoadingStatus", lua.create_function(|_, ()| Ok(0i32))?)?;
    Ok(())
}

/// Register error callstack height stubs (used by ErrorUtil.lua).
fn register_error_callstack_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("GetCallstackHeight", lua.create_function(|_, ()| Ok(2i32))?)?;
    globals.set("SetErrorCallstackHeight", lua.create_function(|_, _height: i32| Ok(()))?)?;
    Ok(())
}
