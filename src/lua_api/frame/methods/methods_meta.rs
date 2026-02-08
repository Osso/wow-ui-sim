//! Metamethod implementations for FrameHandle (__index, __newindex, __len, __eq).

use super::FrameHandle;
use crate::widget::WidgetType;
use mlua::{Lua, MetaMethod, UserDataMethods, Value};
use std::rc::Rc;

/// Add metamethods to FrameHandle UserData.
pub fn add_metamethods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_index_metamethod(methods);
    add_newindex_metamethod(methods);
    add_len_and_eq_metamethods(methods);
}

/// Look up a child frame by numeric index (1-indexed).
fn lookup_child_by_index(
    lua: &Lua,
    state_rc: &Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    idx: i64,
) -> mlua::Result<Value> {
    if idx > 0 {
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(frame_id)
            && let Some(&child_id) = frame.children.get((idx - 1) as usize) {
                drop(state);
                let child_handle = FrameHandle {
                    id: child_id,
                    state: Rc::clone(state_rc),
                };
                return lua.create_userdata(child_handle).map(Value::UserData);
            }
    }
    Ok(Value::Nil)
}

/// Look up a child frame by name from children_keys.
fn lookup_child_by_key(
    lua: &Lua,
    state_rc: &Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    key: &str,
) -> mlua::Result<Option<Value>> {
    let state = state_rc.borrow();
    if let Some(frame) = state.widgets.get(frame_id)
        && let Some(&child_id) = frame.children_keys.get(key) {
            drop(state);
            let child_handle = FrameHandle {
                id: child_id,
                state: Rc::clone(state_rc),
            };
            return lua.create_userdata(child_handle).map(|ud| Some(Value::UserData(ud)));
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
fn lookup_fallback_method(
    lua: &Lua,
    state_rc: &Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    frame_id: u64,
    key: &str,
) -> mlua::Result<Option<Value>> {
    // Cooldown:Clear() - only for Cooldown frame type
    if key == "Clear" {
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

    // Lower() and Raise() adjust frame stacking order
    if key == "Lower" {
        let state_clone = Rc::clone(state_rc);
        return Ok(Some(Value::Function(lua.create_function(
            move |_, args: mlua::MultiValue| {
                // First arg is self (the frame handle)
                if let Some(Value::UserData(ud)) = args.into_iter().next()
                    && let Ok(handle) = ud.borrow::<FrameHandle>() {
                        let mut state = state_clone.borrow_mut();
                        if let Some(frame) = state.widgets.get_mut(handle.id) {
                            frame.frame_level = (frame.frame_level - 1).max(0);
                        }
                    }
                Ok(())
            },
        )?)));
    }
    if key == "Raise" {
        let state_clone = Rc::clone(state_rc);
        return Ok(Some(Value::Function(lua.create_function(
            move |_, args: mlua::MultiValue| {
                if let Some(Value::UserData(ud)) = args.into_iter().next()
                    && let Ok(handle) = ud.borrow::<FrameHandle>() {
                        let mut state = state_clone.borrow_mut();
                        if let Some(frame) = state.widgets.get_mut(handle.id) {
                            frame.frame_level += 1;
                        }
                    }
                Ok(())
            },
        )?)));
    }

    Ok(None)
}

/// __index metamethod - field access on FrameHandle.
fn add_index_metamethod<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_meta_function(
        MetaMethod::Index,
        |lua: &Lua, (ud, key): (mlua::AnyUserData, Value)| {
            let handle = ud.borrow::<FrameHandle>()?;
            let frame_id = handle.id;
            let state_rc = Rc::clone(&handle.state);
            drop(handle);

            // Handle numeric indices - returns n-th child frame (1-indexed)
            if let Value::Integer(idx) = key {
                return lookup_child_by_index(lua, &state_rc, frame_id, idx);
            }

            // Convert key to string for named access
            let key = match &key {
                Value::String(s) => s.to_string_lossy().to_string(),
                _ => return Ok(Value::Nil),
            };

            // First check children_keys (for template-created children like "Text")
            if let Some(child) = lookup_child_by_key(lua, &state_rc, frame_id, &key)? {
                return Ok(child);
            }

            // Check custom fields table
            if let Some(value) = lookup_custom_field(lua, frame_id, &key) {
                return Ok(value);
            }


            // Check fallback methods (Clear, Lower, Raise)
            if let Some(func) = lookup_fallback_method(lua, &state_rc, frame_id, &key)? {
                return Ok(func);
            }

            // Not found in custom fields, return nil (methods are handled separately by mlua)
            Ok(Value::Nil)
        },
    );
}

/// __newindex metamethod - field assignment on FrameHandle.
fn add_newindex_metamethod<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_meta_function(
        MetaMethod::NewIndex,
        |lua: &Lua, (ud, key, value): (mlua::AnyUserData, String, Value)| {
            let handle = ud.borrow::<FrameHandle>()?;
            let frame_id: u64 = handle.id;
            let state_rc = Rc::clone(&handle.state);
            drop(handle);

            // If assigning a FrameHandle value, update children_keys in the Rust widget registry
            if let Value::UserData(child_ud) = &value
                && let Ok(child_handle) = child_ud.borrow::<FrameHandle>() {
                    let child_id = child_handle.id;
                    drop(child_handle);
                    let mut state = state_rc.borrow_mut();
                    if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                        parent_frame.children_keys.insert(key.clone(), child_id);
                    }
                }

            // Get or create the frame's field table (stored in registry)
            let frame_fields =
                crate::lua_api::script_helpers::get_or_create_frame_fields(lua, frame_id);

            frame_fields.set(key, value)?;
            Ok(())
        },
    );
}

/// __len and __eq metamethods.
fn add_len_and_eq_metamethods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // __len - returns number of children (for array-like iteration)
    methods.add_meta_function(MetaMethod::Len, |_lua: &Lua, ud: mlua::AnyUserData| {
        let handle = ud.borrow::<FrameHandle>()?;
        let state = handle.state.borrow();
        let len = state
            .widgets
            .get(handle.id)
            .map(|f| f.children.len())
            .unwrap_or(0);
        Ok(len)
    });

    // __eq - compare FrameHandles by id
    // Needed because __index creates new userdata objects for children_keys lookups
    methods.add_meta_function(
        MetaMethod::Eq,
        |_lua: &Lua, (ud1, ud2): (mlua::AnyUserData, mlua::AnyUserData)| {
            let handle1 = ud1.borrow::<FrameHandle>()?;
            let handle2 = ud2.borrow::<FrameHandle>()?;
            Ok(handle1.id == handle2.id)
        },
    );
}
