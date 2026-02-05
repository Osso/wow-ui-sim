//! Attribute methods: GetAttribute, SetAttribute, frame references, etc.

use super::FrameHandle;
use crate::widget::AttributeValue;
use mlua::{UserDataMethods, Value};

/// Add attribute methods to FrameHandle UserData.
pub fn add_attribute_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // GetAttribute(name) - get a named attribute from the frame
    methods.add_method("GetAttribute", |lua, this, name: String| {
        // First check for table attributes stored in Lua
        let table_attrs: Option<mlua::Table> =
            lua.globals().get("__frame_table_attributes").ok();
        if let Some(attrs) = table_attrs {
            let key = format!("{}_{}", this.id, name);
            let table_val: Value = attrs.get(key.as_str()).unwrap_or(Value::Nil);
            if !matches!(table_val, Value::Nil) {
                return Ok(table_val);
            }
        }

        // Fall back to non-table attributes stored in Rust
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            if let Some(attr) = frame.attributes.get(&name) {
                return match attr {
                    AttributeValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
                    AttributeValue::Number(n) => Ok(Value::Number(*n)),
                    AttributeValue::Boolean(b) => Ok(Value::Boolean(*b)),
                    AttributeValue::Nil => Ok(Value::Nil),
                };
            }
        }
        Ok(Value::Nil)
    });

    // SetAttribute(name, value) - set a named attribute on the frame
    methods.add_method("SetAttribute", |lua, this, (name, value): (String, Value)| {
        // For table values, store in a Lua table to preserve the reference
        if matches!(&value, Value::Table(_)) {
            // Ensure __frame_table_attributes exists
            let table_attrs: mlua::Table = lua
                .globals()
                .get("__frame_table_attributes")
                .unwrap_or_else(|_| {
                    let t = lua.create_table().unwrap();
                    lua.globals().set("__frame_table_attributes", t.clone()).ok();
                    t
                });
            let key = format!("{}_{}", this.id, name);
            table_attrs.set(key, value.clone())?;
        } else {
            // Store simple types in Rust
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                let attr = match &value {
                    Value::Nil => AttributeValue::Nil,
                    Value::Boolean(b) => AttributeValue::Boolean(*b),
                    Value::Integer(i) => AttributeValue::Number(*i as f64),
                    Value::Number(n) => AttributeValue::Number(*n),
                    Value::String(s) => {
                        AttributeValue::String(s.to_str().map(|s| s.to_string()).unwrap_or_default())
                    }
                    _ => AttributeValue::Nil,
                };
                if matches!(attr, AttributeValue::Nil) && matches!(value, Value::Nil) {
                    frame.attributes.remove(&name);
                    // Also remove from table attributes if it exists there
                    if let Ok(table_attrs) =
                        lua.globals().get::<mlua::Table>("__frame_table_attributes")
                    {
                        let key = format!("{}_{}", this.id, name);
                        table_attrs.set(key, Value::Nil).ok();
                    }
                } else {
                    frame.attributes.insert(name.clone(), attr);
                }
            }
        }

        // Trigger OnAttributeChanged script if one exists
        let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();
        if let Some(table) = scripts_table {
            let frame_key = format!("{}_OnAttributeChanged", this.id);
            let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();
            if let Some(handler) = handler {
                // Get frame userdata
                let frame_ref_key = format!("__frame_{}", this.id);
                let frame_ud: Value = lua
                    .globals()
                    .get(frame_ref_key.as_str())
                    .unwrap_or(Value::Nil);
                // Call handler with (self, name, value)
                let name_str = lua.create_string(&name)?;
                let _ = handler.call::<()>((frame_ud, name_str, value));
            }
        }
        Ok(())
    });

    // ClearAttributes() - remove all attributes from the frame
    methods.add_method("ClearAttributes", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.attributes.clear();
        }
        Ok(())
    });

    // SetFrameRef(label, frame) - Store a reference to another frame
    methods.add_method(
        "SetFrameRef",
        |_, _this, (_label, _frame): (String, Value)| {
            // Frame references are used for secure frame communication
            // Just a stub for simulation
            Ok(())
        },
    );

    // GetFrameRef(label) - Get a stored frame reference
    methods.add_method("GetFrameRef", |_, _this, _label: String| Ok(Value::Nil));

    // SetForbidden() - marks frame as forbidden (security feature, no-op in simulation)
    methods.add_method("SetForbidden", |_, _this, _forbidden: Option<bool>| Ok(()));

    // IsForbidden() - check if frame is forbidden
    methods.add_method("IsForbidden", |_, _this, ()| Ok(false));

    // CanChangeProtectedState() - check if we can change protected state
    methods.add_method("CanChangeProtectedState", |_, _this, ()| {
        Ok(true) // Always true in simulation
    });

    // SetPassThroughButtons(...) - set which mouse buttons pass through
    methods.add_method(
        "SetPassThroughButtons",
        |_, _this, _args: mlua::MultiValue| Ok(()),
    );

    // SetFlattensRenderLayers(flatten) - for render optimization
    methods.add_method(
        "SetFlattensRenderLayers",
        |_, _this, _flatten: Option<bool>| Ok(()),
    );

    // SetClipsChildren(clips) - whether to clip child frames
    methods.add_method("SetClipsChildren", |_, _this, _clips: Option<bool>| Ok(()));

    // SetMotionScriptsWhileDisabled(enabled) - enable/disable motion scripts when disabled
    methods.add_method(
        "SetMotionScriptsWhileDisabled",
        |_, _this, _enabled: Option<bool>| Ok(()),
    );

    // GetMotionScriptsWhileDisabled() - check if motion scripts run when disabled
    methods.add_method("GetMotionScriptsWhileDisabled", |_, _this, ()| Ok(false));

    // SetHitRectInsets(left, right, top, bottom) - extend/contract clickable area
    methods.add_method("SetHitRectInsets", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });

    // GetHitRectInsets() - get clickable area insets
    methods.add_method("GetHitRectInsets", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64))
    });
}
