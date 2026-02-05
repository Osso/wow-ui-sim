//! Metamethod implementations for FrameHandle (__index, __newindex, __len, __eq).

use super::FrameHandle;
use crate::widget::WidgetType;
use mlua::{Lua, MetaMethod, UserDataMethods, Value};
use std::rc::Rc;

/// Add metamethods to FrameHandle UserData.
pub fn add_metamethods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // Support custom field access via __index/__newindex
    // This allows addons to do: frame.customField = value

    methods.add_meta_function(
        MetaMethod::Index,
        |lua: &Lua, (ud, key): (mlua::AnyUserData, Value)| {
            // Try to get from the custom fields table
            let handle = ud.borrow::<FrameHandle>()?;
            let frame_id = handle.id;
            let state_rc = Rc::clone(&handle.state);
            drop(handle); // Release borrow before accessing state

            // Handle numeric indices - returns n-th child frame (1-indexed)
            if let Value::Integer(idx) = key {
                if idx > 0 {
                    let state = state_rc.borrow();
                    if let Some(frame) = state.widgets.get(frame_id) {
                        if let Some(&child_id) = frame.children.get((idx - 1) as usize) {
                            drop(state);
                            let child_handle = FrameHandle {
                                id: child_id,
                                state: Rc::clone(&state_rc),
                            };
                            return lua.create_userdata(child_handle).map(Value::UserData);
                        }
                    }
                }
                return Ok(Value::Nil);
            }

            // Convert key to string for named access
            let key = match &key {
                Value::String(s) => s.to_string_lossy().to_string(),
                _ => return Ok(Value::Nil),
            };

            // First check children_keys (for template-created children like "Text")
            {
                let state = state_rc.borrow();
                if let Some(frame) = state.widgets.get(frame_id) {
                    if let Some(&child_id) = frame.children_keys.get(&key) {
                        // Create userdata for the child frame
                        drop(state); // Release borrow before creating userdata
                        let child_handle = FrameHandle {
                            id: child_id,
                            state: Rc::clone(&state_rc),
                        };
                        return lua.create_userdata(child_handle).map(Value::UserData);
                    }
                }
            }

            let fields_table: Option<mlua::Table> = lua.globals().get("__frame_fields").ok();

            if let Some(table) = fields_table {
                let frame_fields: Option<mlua::Table> = table.get::<mlua::Table>(frame_id).ok();
                if let Some(fields) = frame_fields {
                    let value: Value = fields.get::<Value>(key.as_str()).unwrap_or(Value::Nil);
                    if value != Value::Nil {
                        return Ok(value);
                    }
                }
            }

            // Special handling for Cooldown:Clear() - only for Cooldown frame type
            // This avoids conflicts with addons that use frame.Clear as a field
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
                    return Ok(Value::Function(
                        lua.create_function(|_, _: mlua::MultiValue| Ok(()))?,
                    ));
                }
            }

            // Fallback methods that might conflict with custom properties
            // These are only returned if no custom field was found above
            if key == "Lower" || key == "Raise" {
                // Lower() and Raise() adjust frame stacking order (no-op in simulator)
                return Ok(Value::Function(
                    lua.create_function(|_, _: mlua::MultiValue| Ok(()))?,
                ));
            }

            // Not found in custom fields, return nil (methods are handled separately by mlua)
            Ok(Value::Nil)
        },
    );

    methods.add_meta_function(
        MetaMethod::NewIndex,
        |lua: &Lua, (ud, key, value): (mlua::AnyUserData, String, Value)| {
            let handle = ud.borrow::<FrameHandle>()?;
            let frame_id: u64 = handle.id;
            let state_rc = Rc::clone(&handle.state);
            drop(handle);

            // If assigning a FrameHandle value, update children_keys in the Rust widget registry
            // This syncs parentKey relationships from Lua space to Rust
            if let Value::UserData(child_ud) = &value {
                if let Ok(child_handle) = child_ud.borrow::<FrameHandle>() {
                    let child_id = child_handle.id;
                    drop(child_handle);
                    let mut state = state_rc.borrow_mut();
                    if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                        parent_frame.children_keys.insert(key.clone(), child_id);
                    }
                }
            }

            // Get or create the fields table
            let fields_table: mlua::Table =
                lua.globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });

            // Get or create the frame's field table
            let frame_fields: mlua::Table =
                fields_table
                    .get::<mlua::Table>(frame_id)
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });

            frame_fields.set(key, value)?;
            Ok(())
        },
    );

    // __len metamethod - returns number of children (for array-like iteration)
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

    // __eq metamethod - compare FrameHandles by id
    // This is needed because __index creates new userdata objects when accessing children_keys,
    // so ParentFrame.child and ChildFrame would be different userdata objects with the same id
    methods.add_meta_function(
        MetaMethod::Eq,
        |_lua: &Lua, (ud1, ud2): (mlua::AnyUserData, mlua::AnyUserData)| {
            let handle1 = ud1.borrow::<FrameHandle>()?;
            let handle2 = ud2.borrow::<FrameHandle>()?;
            Ok(handle1.id == handle2.id)
        },
    );
}
