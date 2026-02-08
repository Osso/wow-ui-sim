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
    register_network_stubs(lua)?;
    register_input_state_stubs(lua)?;
    register_screen_size_functions(lua, &state)?;
    register_request_time_played(lua, Rc::clone(&state))?;
    register_cursor_position(lua)?;
    register_localization_stubs(lua)?;
    register_ui_object_stubs(lua)?;
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
                let error_msg = lua.create_string(e.to_string())?;
                let handler_result = error_handler.call::<Value>(Value::String(error_msg));
                let mut ret = mlua::MultiValue::new();
                ret.push_back(Value::Boolean(false));
                match handler_result {
                    Ok(v) => ret.push_back(v),
                    Err(he) => ret.push_back(Value::String(lua.create_string(he.to_string())?)),
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
            if let Some(handler) = crate::lua_api::script_helpers::get_script(lua, widget_id, "OnEvent") {
                let frame_ref_key = format!("__frame_{}", widget_id);
                let frame: Value = lua.globals().get(frame_ref_key.as_str()).unwrap_or(Value::Nil);

                let mut call_args = vec![frame, Value::String(lua.create_string(&event_name)?)];
                call_args.extend(event_args.iter().cloned());

                if let Err(e) = handler.call::<()>(mlua::MultiValue::from_vec(call_args)) {
                    crate::lua_api::script_helpers::call_error_handler(lua, &e.to_string());
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

        fire_event_to_listeners(lua, &state, "VARIABLES_LOADED", |lua| {
            let event_str = lua.create_string("VARIABLES_LOADED")?;
            Ok(vec![Value::String(event_str)])
        })?;

        fire_event_to_listeners(lua, &state, "PLAYER_LOGIN", |lua| {
            let event_str = lua.create_string("PLAYER_LOGIN")?;
            Ok(vec![Value::String(event_str)])
        })?;

        fire_event_to_listeners(lua, &state, "PLAYER_ENTERING_WORLD", |lua| {
            let event_str = lua.create_string("PLAYER_ENTERING_WORLD")?;
            Ok(vec![Value::String(event_str), Value::Boolean(false), Value::Boolean(true)])
        })?;

        fire_event_to_listeners(lua, &state, "UPDATE_BINDINGS", |lua| {
            let event_str = lua.create_string("UPDATE_BINDINGS")?;
            Ok(vec![Value::String(event_str)])
        })?;

        fire_event_to_listeners(lua, &state, "DISPLAY_SIZE_CHANGED", |lua| {
            let event_str = lua.create_string("DISPLAY_SIZE_CHANGED")?;
            Ok(vec![Value::String(event_str)])
        })?;

        fire_event_to_listeners(lua, &state, "UI_SCALE_CHANGED", |lua| {
            let event_str = lua.create_string("UI_SCALE_CHANGED")?;
            Ok(vec![Value::String(event_str)])
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
        if let Some(handler) = crate::lua_api::script_helpers::get_script(lua, widget_id, "OnEvent") {
            if let Some(frame) = crate::lua_api::script_helpers::get_frame_ref(lua, widget_id) {
                let mut call_args = vec![frame];
                call_args.extend(build_extra_args(lua)?);
                if let Err(e) = handler.call::<()>(mlua::MultiValue::from_vec(call_args)) {
                    crate::lua_api::script_helpers::call_error_handler(lua, &e.to_string());
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
    // The __index table must contain forwarding functions so that
    // CopyTable(GetFrameMetatable().__index) produces a table where
    // e.g. LOCAL_CHECK_Frame.GetAttribute(frame, ...) works.
    for name in &["GetFrameMetatable", "GetButtonMetatable"] {
        globals.set(*name, lua.create_function(|lua, ()| {
            let mt = lua.create_table()?;
            let index = create_frame_method_forwarders(lua)?;
            mt.set("__index", index)?;
            Ok(Value::Table(mt))
        })?)?;
    }

    globals.set("C_GamePad", register_c_gamepad(lua)?)?;
    globals.set("C_AssistedCombat", register_c_assisted_combat(lua)?)?;
    globals.set("C_Widget", register_c_widget(lua)?)?;
    Ok(())
}

/// Build a table of forwarding functions for Frame methods.
///
/// SecureTemplates.lua does `LOCAL_CHECK_Frame = CopyTable(GetFrameMetatable().__index)`
/// then calls `LOCAL_CHECK_Frame.GetAttribute(frame, ...)` — i.e. methods as plain
/// functions with explicit self. We create Lua closures that forward these calls.
fn create_frame_method_forwarders(lua: &Lua) -> Result<mlua::Table> {
    let index = lua.create_table()?;
    let methods = &[
        "GetAttribute", "SetAttribute", "GetParent", "GetName",
        "GetObjectType", "IsObjectType", "GetFrameStrata",
        "GetFrameLevel", "IsShown", "IsVisible", "GetWidth",
        "GetHeight", "GetSize", "GetScale", "GetAlpha",
    ];
    for method in methods {
        let forwarder = lua.load(format!(
            "return function(self, ...) return self:{method}(...) end"
        )).eval::<mlua::Function>()?;
        index.set(*method, forwarder)?;
    }
    Ok(index)
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
    globals.set("SetErrorCallstackHeight", lua.create_function(|_, _height: Option<i32>| Ok(()))?)?;
    Ok(())
}

/// Network stats stubs (simulator has no real network).
fn register_network_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    // GetNetStats() -> bandwidthIn, bandwidthOut, latencyHome, latencyWorld
    globals.set(
        "GetNetStats",
        lua.create_function(|_, ()| Ok((0.0f64, 0.0f64, 0.0f64, 0.0f64)))?,
    )?;
    Ok(())
}

/// Keyboard/mouse modifier state stubs (simulator has no real input state).
fn register_input_state_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("IsShiftKeyDown", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsControlKeyDown", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsAltKeyDown", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsModifierKeyDown", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("IsModifiedClick", lua.create_function(|_, _action: Option<String>| Ok(false))?)?;
    globals.set("IsMouseButtonDown", lua.create_function(|_, _btn: Option<Value>| Ok(false))?)?;
    globals.set("GetMouseFocus", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetMouseButtonClicked", lua.create_function(|_, ()| Ok(""))?)?;
    Ok(())
}

/// Screen size functions reading from the rendering surface dimensions in SimState.
fn register_screen_size_functions(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    let st = Rc::clone(state);
    globals.set("GetScreenWidth", lua.create_function(move |_, ()| {
        Ok(st.borrow().screen_width as f64)
    })?)?;
    let st = Rc::clone(state);
    globals.set("GetScreenHeight", lua.create_function(move |_, ()| {
        Ok(st.borrow().screen_height as f64)
    })?)?;
    let st = Rc::clone(state);
    globals.set("GetPhysicalScreenSize", lua.create_function(move |_, ()| {
        let s = st.borrow();
        Ok((s.screen_width as i32, s.screen_height as i32))
    })?)?;
    Ok(())
}

/// Register `RequestTimePlayed()` - fires TIME_PLAYED_MSG with simulated play data.
///
/// In WoW, `RequestTimePlayed()` is asynchronous and fires `TIME_PLAYED_MSG`
/// with `(totalTimePlayed, timePlayedThisLevel)`. We fire it immediately with
/// fake data for the current simulated character.
fn register_request_time_played(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let request_fn = lua.create_function(move |lua, ()| {
        // Simulated: 15 days played total, 3 days at current level
        let total_played = 15 * 24 * 3600; // 15 days in seconds
        let level_played = 3 * 24 * 3600; // 3 days in seconds

        let listeners = {
            let s = state.borrow();
            s.widgets.get_event_listeners("TIME_PLAYED_MSG")
        };

        for widget_id in listeners {
            if let Some(handler) = crate::lua_api::script_helpers::get_script(lua, widget_id, "OnEvent") {
                if let Some(frame) = crate::lua_api::script_helpers::get_frame_ref(lua, widget_id) {
                    let args = vec![
                        frame,
                        Value::String(lua.create_string("TIME_PLAYED_MSG")?),
                        Value::Integer(total_played),
                        Value::Integer(level_played),
                    ];
                    if let Err(e) = handler.call::<()>(mlua::MultiValue::from_vec(args)) {
                        crate::lua_api::script_helpers::call_error_handler(lua, &e.to_string());
                    }
                }
            }
        }

        Ok(())
    })?;
    lua.globals().set("RequestTimePlayed", request_fn)?;
    Ok(())
}

/// Register `GetCursorPosition()` - returns cursor screen coordinates.
///
/// Returns simulated cursor at screen center. The MinimapButton addon uses this
/// for drag positioning.
fn register_cursor_position(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "GetCursorPosition",
        lua.create_function(|_, ()| Ok((512.0_f64, 384.0_f64)))?,
    )?;
    Ok(())
}

/// Localization stubs: GetText (string lookup), and other localization helpers.
fn register_localization_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // GetText(key) - localization lookup; just return the key as-is
    globals.set("GetText", lua.create_function(|_, key: String| Ok(key))?)?;

    Ok(())
}

/// Miscellaneous UI object stubs: AnimateCallout, WowStyle1DropdownMixin, etc.
fn register_ui_object_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // AnimateCallout global table (used by TutorialFrame)
    let animate_callout = lua.create_table()?;
    animate_callout.set("Start", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    animate_callout.set("Stop", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    globals.set("AnimateCallout", animate_callout)?;

    // WowStyle1DropdownMixin - dropdown mixin (used by WardrobeOutfits)
    let wow_style1 = lua.create_table()?;
    wow_style1.set("OnLoad", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    globals.set("WowStyle1DropdownMixin", wow_style1)?;

    // C_EventUtils - event scheduling and validation
    let c_event_utils = lua.create_table()?;
    c_event_utils.set("CanPlayerUseEventScheduler", lua.create_function(|_, ()| Ok(false))?)?;
    c_event_utils.set("IsEventValid", lua.create_function(|_, _event: String| Ok(true))?)?;
    globals.set("C_EventUtils", c_event_utils)?;

    // AnimateMouse global table (used by TutorialFrame)
    let animate_mouse = lua.create_table()?;
    animate_mouse.set("Start", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    animate_mouse.set("Stop", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    globals.set("AnimateMouse", animate_mouse)?;

    // Patch C_PlayerInfo (table created in c_misc_api.rs)
    if let Ok(c_player_info) = globals.get::<mlua::Table>("C_PlayerInfo") {
        c_player_info.set("IsPlayerInRPE", lua.create_function(|_, ()| Ok(false))?)?;
        c_player_info.set("GetAlternateFormInfo", lua.create_function(|_, ()| Ok((false, false)))?)?;
    }

    // Patch C_UIWidgetManager (table created in c_misc_api.rs)
    if let Ok(c_widget_mgr) = globals.get::<mlua::Table>("C_UIWidgetManager") {
        c_widget_mgr.set("GetPowerBarWidgetSetID", lua.create_function(|_, ()| Ok(0i32))?)?;
    }

    // Patch C_UnitAuras (table created in c_misc_api.rs)
    // GetAuraSlots returns (continuationToken, slot1, slot2, ...).
    // Returning nil means no auras and no more pages (stops the repeat..until loop).
    if let Ok(c_unit_auras) = globals.get::<mlua::Table>("C_UnitAuras") {
        c_unit_auras.set("GetAuraSlots", lua.create_function(|_, _args: mlua::MultiValue| {
            Ok(Value::Nil)
        })?)?;
    }

    Ok(())
}
