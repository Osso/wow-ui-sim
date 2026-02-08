//! Helper functions for script table access and error handling.
//!
//! All internal tables (__scripts, __script_hooks, __frame_fields) are stored
//! in the Lua registry, invisible to addon Lua code.

use mlua::{Lua, Value};

// ── __scripts table ──────────────────────────────────────────────────

const SCRIPTS_KEY: &str = "__scripts";
const SCRIPT_HOOKS_KEY: &str = "__script_hooks";
const FRAME_FIELDS_KEY: &str = "__frame_fields";
const ERROR_HANDLER_KEY: &str = "__wow_error_handler";

/// Get the __scripts table from the Lua registry. Returns None if not yet created.
pub fn get_scripts_table(lua: &Lua) -> Option<mlua::Table> {
    lua.named_registry_value(SCRIPTS_KEY).ok()
}

/// Get or create the __scripts table in the Lua registry.
pub fn get_or_create_scripts_table(lua: &Lua) -> mlua::Table {
    lua.named_registry_value(SCRIPTS_KEY).unwrap_or_else(|_| {
        let t = lua.create_table().unwrap();
        lua.set_named_registry_value(SCRIPTS_KEY, t.clone())
            .unwrap();
        t
    })
}

/// Get the script handler for a given frame + handler name.
pub fn get_script(lua: &Lua, widget_id: u64, handler_name: &str) -> Option<mlua::Function> {
    let table = get_scripts_table(lua)?;
    let key = format!("{}_{}", widget_id, handler_name);
    table.get(key.as_str()).ok()
}

/// Set a script handler for a given frame + handler name.
pub fn set_script(lua: &Lua, widget_id: u64, handler_name: &str, func: mlua::Function) {
    let table = get_or_create_scripts_table(lua);
    let key = format!("{}_{}", widget_id, handler_name);
    table.set(key.as_str(), func).ok();
}

/// Remove a script handler for a given frame + handler name.
pub fn remove_script(lua: &Lua, widget_id: u64, handler_name: &str) {
    if let Some(table) = get_scripts_table(lua) {
        let key = format!("{}_{}", widget_id, handler_name);
        table.set(key.as_str(), Value::Nil).ok();
    }
}

// ── __script_hooks table ─────────────────────────────────────────────

/// Get or create the __script_hooks table in the Lua registry.
pub fn get_or_create_hooks_table(lua: &Lua) -> mlua::Table {
    lua.named_registry_value(SCRIPT_HOOKS_KEY)
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.set_named_registry_value(SCRIPT_HOOKS_KEY, t.clone())
                .unwrap();
            t
        })
}

// ── __frame_fields table ─────────────────────────────────────────────

/// Get the __frame_fields table from the Lua registry. Returns None if not yet created.
pub fn get_frame_fields_table(lua: &Lua) -> Option<mlua::Table> {
    lua.named_registry_value(FRAME_FIELDS_KEY).ok()
}

/// Get or create the __frame_fields table in the Lua registry.
pub fn get_or_create_frame_fields_table(lua: &Lua) -> mlua::Table {
    lua.named_registry_value(FRAME_FIELDS_KEY)
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.set_named_registry_value(FRAME_FIELDS_KEY, t.clone())
                .unwrap();
            t
        })
}

/// Get or create a per-frame fields sub-table within __frame_fields.
pub fn get_or_create_frame_fields(lua: &Lua, frame_id: u64) -> mlua::Table {
    let fields_table = get_or_create_frame_fields_table(lua);
    fields_table
        .get::<mlua::Table>(frame_id)
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            fields_table.set(frame_id, t.clone()).unwrap();
            t
        })
}

// ── Frame reference ──────────────────────────────────────────────────

/// Get the frame userdata for a given widget ID (from `__frame_{id}` global).
pub fn get_frame_ref(lua: &Lua, widget_id: u64) -> Option<Value> {
    let key = format!("__frame_{}", widget_id);
    match lua.globals().get::<Value>(key.as_str()) {
        Ok(v) if v != Value::Nil => Some(v),
        _ => None,
    }
}

// ── Error handler ────────────────────────────────────────────────────

/// Call the WoW error handler (set via `seterrorhandler`) and always log to stderr.
pub fn call_error_handler(lua: &Lua, error_msg: &str) {
    eprintln!("Lua error: {error_msg}");
    let handler: Option<mlua::Function> = lua.named_registry_value(ERROR_HANDLER_KEY).ok();
    if let Some(h) = handler {
        if let Err(e) = h.call::<()>(error_msg.to_string()) {
            eprintln!("Error in error handler: {e}");
        }
    }
}

