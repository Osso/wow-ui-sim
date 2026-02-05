//! System utility functions.
//!
//! This module contains WoW's core system functions including:
//! - `type()` - Type introspection with Frame userdata support
//! - `rawget()` - Raw table access with userdata compatibility
//! - `xpcall()` - Protected call with error handler and varargs (Lua 5.2+ feature)
//! - `SlashCmdList` - Slash command registry table
//! - `FireEvent()` - Simulator utility to fire events for testing
//! - `ReloadUI()` - Reload the interface (fires startup events again)
//! - Build type checks: `IsPublicTestClient()`, `IsBetaBuild()`, `IsPublicBuild()`
//! - Battle.net stubs: `BNFeaturesEnabled()`, `BNConnected()`, etc.

use crate::lua_api::frame::FrameHandle;
use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register system utility functions in the Lua global namespace.
pub fn register_system_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
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
                // Check if this is a FrameHandle - if so, report it as "table"
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
    // Since our FrameHandle passes that check, rawget needs to handle it
    let rawget_fn = lua.create_function(|lua, (table, key): (Value, Value)| {
        match table {
            Value::Table(t) => t.raw_get(key),
            Value::UserData(_) => {
                // UserData doesn't support rawget, return nil instead of erroring
                Ok(Value::Nil)
            }
            _ => {
                // Call the original rawget for proper error
                let original: mlua::Function = lua.globals().raw_get("__original_rawget")?;
                original.call((table, key))
            }
        }
    })?;
    // Save original and install custom
    let original_rawget: mlua::Function = globals.raw_get("rawget")?;
    globals.raw_set("__original_rawget", original_rawget)?;
    globals.set("rawget", rawget_fn)?;

    // xpcall(func, errorhandler, ...) - Call function with error handler and varargs
    // Lua 5.1's native xpcall doesn't support varargs, but WoW's Lua does (Lua 5.2+ feature)
    // This is critical for AceAddon's safecall function to work
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

        // Remaining args are passed to the function
        let call_args: mlua::MultiValue = args_vec.into_iter().collect();

        // Call the function with the varargs
        match func.call::<mlua::MultiValue>(call_args) {
            Ok(results) => {
                // Success: return true followed by all results
                let mut ret = mlua::MultiValue::new();
                ret.push_back(Value::Boolean(true));
                for v in results {
                    ret.push_back(v);
                }
                Ok(ret)
            }
            Err(e) => {
                // Error: call error handler with the error message
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
    globals.set("xpcall", xpcall_fn)?;

    // SlashCmdList table
    let slash_cmd_list = lua.create_table()?;
    globals.set("SlashCmdList", slash_cmd_list)?;

    // FireEvent - simulator utility to fire events for testing
    let state_for_fire = Rc::clone(&state);
    let fire_event = lua.create_function(move |lua, args: mlua::Variadic<Value>| {
        let mut args_iter = args.into_iter();
        let event_name: String = match args_iter.next() {
            Some(Value::String(s)) => s.to_str()?.to_string(),
            _ => return Err(mlua::Error::runtime("FireEvent requires event name as first argument")),
        };

        // Collect remaining arguments
        let event_args: Vec<Value> = args_iter.collect();

        // Get listeners for this event
        let listeners = {
            let state = state_for_fire.borrow();
            state.widgets.get_event_listeners(&event_name)
        };

        // Fire to each listener
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
    globals.set("FireEvent", fire_event)?;

    // ReloadUI - reload the interface (fires startup events again)
    let state_for_reload = Rc::clone(&state);
    let reload_ui = lua.create_function(move |lua, ()| {
        // Fire ADDON_LOADED
        let addon_loaded_listeners = {
            let state = state_for_reload.borrow();
            state.widgets.get_event_listeners("ADDON_LOADED")
        };
        for widget_id in addon_loaded_listeners {
            if let Ok(Some(table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
                let frame_key = format!("{}_OnEvent", widget_id);
                if let Ok(Some(handler)) = table.get::<Option<mlua::Function>>(frame_key.as_str()) {
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    if let Ok(frame) = lua.globals().get::<Value>(frame_ref_key.as_str()) {
                        let event_str = lua.create_string("ADDON_LOADED")?;
                        let addon_name = lua.create_string("WoWUISim")?;
                        let _ = handler.call::<()>((frame, Value::String(event_str), Value::String(addon_name)));
                    }
                }
            }
        }

        // Fire PLAYER_LOGIN
        let login_listeners = {
            let state = state_for_reload.borrow();
            state.widgets.get_event_listeners("PLAYER_LOGIN")
        };
        for widget_id in login_listeners {
            if let Ok(Some(table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
                let frame_key = format!("{}_OnEvent", widget_id);
                if let Ok(Some(handler)) = table.get::<Option<mlua::Function>>(frame_key.as_str()) {
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    if let Ok(frame) = lua.globals().get::<Value>(frame_ref_key.as_str()) {
                        let event_str = lua.create_string("PLAYER_LOGIN")?;
                        let _ = handler.call::<()>((frame, Value::String(event_str)));
                    }
                }
            }
        }

        // Fire PLAYER_ENTERING_WORLD
        let entering_listeners = {
            let state = state_for_reload.borrow();
            state.widgets.get_event_listeners("PLAYER_ENTERING_WORLD")
        };
        for widget_id in entering_listeners {
            if let Ok(Some(table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
                let frame_key = format!("{}_OnEvent", widget_id);
                if let Ok(Some(handler)) = table.get::<Option<mlua::Function>>(frame_key.as_str()) {
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    if let Ok(frame) = lua.globals().get::<Value>(frame_ref_key.as_str()) {
                        let event_str = lua.create_string("PLAYER_ENTERING_WORLD")?;
                        let _ = handler.call::<()>((frame, Value::String(event_str), Value::Boolean(false), Value::Boolean(true)));
                    }
                }
            }
        }

        state_for_reload.borrow_mut().console_output.push("UI Reloaded".to_string());
        Ok(())
    })?;
    globals.set("ReloadUI", reload_ui)?;

    // IsPublicTestClient() - returns true if running on PTR
    globals.set("IsPublicTestClient", lua.create_function(|_, ()| Ok(false))?)?;

    // IsBetaBuild() - returns true if running on beta
    globals.set("IsBetaBuild", lua.create_function(|_, ()| Ok(false))?)?;

    // IsPublicBuild() - returns true if running on live servers
    globals.set("IsPublicBuild", lua.create_function(|_, ()| Ok(true))?)?;

    // Battle.net functions
    globals.set("BNFeaturesEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNFeaturesEnabledAndConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNConnected", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("BNGetFriendInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    globals.set("BNGetNumFriends", lua.create_function(|_, ()| Ok((0, 0)))?)?; // online, total
    globals.set("BNGetInfo", lua.create_function(|lua, ()| {
        // Return: presenceID, battleTag, toonID, currentBroadcast, bnetAFK, bnetDND, isRIDEnabled
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
