//! LightUserData metatable for frame handles.
//!
//! Sets up a shared metatable for all LightUserData values (frames):
//! - `__index` = Rust fn that does rawget on methods_table, then falls back
//!   to children_keys / custom fields / numeric index / Lower/Raise/Clear
//! - `__newindex` = Rust fn (children_keys sync + __frame_fields storage)
//! - `__len` = Rust fn (children count)
//! - No `__eq` needed: same ID = same pointer = Lua `==` works natively.

use super::handle::{extract_frame_id, frame_lud, get_sim_state, lud_to_id};
use crate::widget::WidgetType;
use mlua::{LightUserData, Lua, Value};

/// Build and install the shared LightUserData metatable for frames.
pub fn setup_frame_metatable(lua: &Lua) -> mlua::Result<()> {
    let methods_table = lua.create_table()?;

    // Register all ~200 methods into the table
    super::methods::register_all_methods(lua, &methods_table)?;

    // Store methods_table in registry for getmetatable() and populate_method_index()
    lua.set_named_registry_value("__frame_methods_table", methods_table.clone())?;

    // Build the frame metatable
    let frame_mt = lua.create_table()?;
    frame_mt.set("__index", create_index(lua, methods_table)?)?;
    frame_mt.set("__newindex", create_newindex(lua)?)?;
    frame_mt.set("__len", create_len(lua)?)?;

    // Install as the shared metatable for ALL LightUserData values
    lua.set_type_metatable::<LightUserData>(Some(frame_mt));

    Ok(())
}

/// __index: method lookup via rawget on methods_table, then fallback.
fn create_index(lua: &Lua, methods_table: mlua::Table) -> mlua::Result<mlua::Function> {
    lua.create_function(move |lua, (ud, key): (LightUserData, Value)| {
        let frame_id = lud_to_id(ud);

        // Numeric index → returns n-th child frame (1-indexed)
        if let Value::Integer(idx) = key {
            return lookup_child_by_index(lua, frame_id, idx);
        }

        let key_str = match &key {
            Value::String(s) => s.to_string_lossy(),
            _ => return Ok(Value::Nil),
        };

        // Fast path: rawget on methods_table
        let method: Value = methods_table.raw_get(key_str.as_str())?;
        if method != Value::Nil {
            return Ok(method);
        }

        // Children_keys lookup
        if let Some(child) = lookup_child_by_key(lua, frame_id, &key_str)? {
            return Ok(child);
        }

        // Custom fields table
        if let Some(value) = lookup_custom_field(lua, frame_id, &key_str) {
            return Ok(value);
        }

        // Fallback methods (Clear for Cooldown, Lower, Raise)
        if let Some(func) = lookup_fallback_method(lua, frame_id, &key_str)? {
            return Ok(func);
        }

        Ok(Value::Nil)
    })
}

/// __newindex: children_keys sync + __frame_fields storage.
fn create_newindex(lua: &Lua) -> mlua::Result<mlua::Function> {
    lua.create_function(|lua, (ud, key, value): (LightUserData, String, Value)| {
        let frame_id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);

        // Sync children_keys with frame assignments
        if let Some(child_id) = extract_frame_id(&value) {
            let mut state = state_rc.borrow_mut();
            let is_real_child = state
                .widgets
                .get(child_id)
                .is_some_and(|c| c.parent_id == Some(frame_id));
            if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                parent_frame.children_keys.insert(key.clone(), child_id);
                if is_real_child && !parent_frame.children.contains(&child_id) {
                    parent_frame.children.push(child_id);
                }
            }
        } else {
            // Non-frame value — remove stale children_keys entry
            let mut state = state_rc.borrow_mut();
            if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                parent_frame.children_keys.remove(&key);
            }
        }

        // Store in frame fields table
        let frame_fields =
            crate::lua_api::script_helpers::get_or_create_frame_fields(lua, frame_id);
        frame_fields.set(key, value)?;
        Ok(())
    })
}

/// __len: returns number of children.
fn create_len(lua: &Lua) -> mlua::Result<mlua::Function> {
    lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let len = state.widgets.get(id).map(|f| f.children.len()).unwrap_or(0);
        Ok(len)
    })
}

// ── Lookup helpers ──────────────────────────────────────────────────

/// Look up a child frame by numeric index (1-indexed).
fn lookup_child_by_index(lua: &Lua, frame_id: u64, idx: i64) -> mlua::Result<Value> {
    if idx > 0 {
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(frame_id)
            && let Some(&child_id) = frame.children.get((idx - 1) as usize)
        {
            return Ok(frame_lud(child_id));
        }
    }
    Ok(Value::Nil)
}

/// Look up a child frame by name from children_keys.
fn lookup_child_by_key(lua: &Lua, frame_id: u64, key: &str) -> mlua::Result<Option<Value>> {
    let state_rc = get_sim_state(lua);
    let state = state_rc.borrow();
    if let Some(frame) = state.widgets.get(frame_id)
        && let Some(&child_id) = frame.children_keys.get(key)
    {
        return Ok(Some(frame_lud(child_id)));
    }
    Ok(None)
}

/// Look up a value from the __frame_fields Lua table (stored in registry).
fn lookup_custom_field(lua: &Lua, frame_id: u64, key: &str) -> Option<Value> {
    let fields_table = crate::lua_api::script_helpers::get_frame_fields_table(lua)?;
    let frame_fields: mlua::Table = fields_table.get::<mlua::Table>(frame_id).ok()?;
    let value: Value = frame_fields.get::<Value>(key).unwrap_or(Value::Nil);
    if value != Value::Nil {
        Some(value)
    } else {
        None
    }
}

/// Handle special fallback methods (Clear for Cooldown, Lower, Raise).
fn lookup_fallback_method(lua: &Lua, frame_id: u64, key: &str) -> mlua::Result<Option<Value>> {
    if key == "Clear" {
        let state_rc = get_sim_state(lua);
        let is_cooldown = {
            let state = state_rc.borrow();
            state
                .widgets
                .get(frame_id)
                .map(|f| f.widget_type == WidgetType::Cooldown)
                .unwrap_or(false)
        };
        if is_cooldown {
            return Ok(Some(Value::Function(
                lua.create_function(|_, _: mlua::MultiValue| Ok(()))?,
            )));
        }
    }

    if key == "Lower" {
        return Ok(Some(Value::Function(lua.create_function(
            |lua, ud: LightUserData| {
                let id = lud_to_id(ud);
                let state_rc = get_sim_state(lua);
                let mut state = state_rc.borrow_mut();
                if let Some(frame) = state.widgets.get_mut_visual(id) {
                    frame.frame_level = (frame.frame_level - 1).max(0);
                }
                Ok(())
            },
        )?)));
    }

    if key == "Raise" {
        return Ok(Some(Value::Function(lua.create_function(
            |lua, ud: LightUserData| {
                let id = lud_to_id(ud);
                let state_rc = get_sim_state(lua);
                let mut state = state_rc.borrow_mut();
                state.raise_frame(id);
                Ok(())
            },
        )?)));
    }

    Ok(None)
}
