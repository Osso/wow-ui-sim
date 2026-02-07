//! Attribute methods: GetAttribute, SetAttribute, frame references, etc.

use super::FrameHandle;
use crate::widget::AttributeValue;
use mlua::{UserDataMethods, Value};

/// Add attribute methods to FrameHandle UserData.
pub fn add_attribute_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_get_set_attribute_methods(methods);
    add_frame_ref_methods(methods);
    add_security_and_input_stubs(methods);
}

/// GetAttribute, SetAttribute, ClearAttributes - core attribute CRUD.
fn add_get_set_attribute_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(attr) = frame.attributes.get(&name) {
                return match attr {
                    AttributeValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
                    AttributeValue::Number(n) => Ok(Value::Number(*n)),
                    AttributeValue::Boolean(b) => Ok(Value::Boolean(*b)),
                    AttributeValue::Nil => Ok(Value::Nil),
                };
            }
        Ok(Value::Nil)
    });

    methods.add_method("SetAttribute", |lua, this, (name, value): (String, Value)| {
        set_attribute_value(lua, this, &name, &value)?;
        fire_on_attribute_changed(lua, this, &name, value)?;
        Ok(())
    });

    methods.add_method("ClearAttributes", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.attributes.clear();
        }
        Ok(())
    });
}

/// Store the attribute value in Lua (tables) or Rust (simple types).
fn set_attribute_value(
    lua: &mlua::Lua,
    this: &FrameHandle,
    name: &str,
    value: &Value,
) -> mlua::Result<()> {
    if matches!(value, Value::Table(_)) {
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
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            let attr = match value {
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
                frame.attributes.remove(name);
                // Also remove from table attributes if it exists there
                if let Ok(table_attrs) =
                    lua.globals().get::<mlua::Table>("__frame_table_attributes")
                {
                    let key = format!("{}_{}", this.id, name);
                    table_attrs.set(key, Value::Nil).ok();
                }
            } else {
                frame.attributes.insert(name.to_string(), attr);
            }
        }
    }
    Ok(())
}

/// Fire OnAttributeChanged script handler if one exists.
fn fire_on_attribute_changed(
    lua: &mlua::Lua,
    this: &FrameHandle,
    name: &str,
    value: Value,
) -> mlua::Result<()> {
    let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();
    if let Some(table) = scripts_table {
        let frame_key = format!("{}_OnAttributeChanged", this.id);
        let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();
        if let Some(handler) = handler {
            let frame_ref_key = format!("__frame_{}", this.id);
            let frame_ud: Value = lua
                .globals()
                .get(frame_ref_key.as_str())
                .unwrap_or(Value::Nil);
            let name_str = lua.create_string(name)?;
            let _ = handler.call::<()>((frame_ud, name_str, value));
        }
    }
    Ok(())
}

/// SetFrameRef/GetFrameRef - secure frame reference stubs.
fn add_frame_ref_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "SetFrameRef",
        |_, _this, (_label, _frame): (String, Value)| Ok(()),
    );

    methods.add_method("GetFrameRef", |_, _this, _label: String| Ok(Value::Nil));
}

/// Security, input, and rendering stubs (no-op in simulation).
fn add_security_and_input_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetForbidden", |_, _this, _forbidden: Option<bool>| Ok(()));
    methods.add_method("IsForbidden", |_, _this, ()| Ok(false));
    methods.add_method("CanChangeProtectedState", |_, _this, ()| Ok(true));
    methods.add_method(
        "SetPassThroughButtons",
        |_, _this, _args: mlua::MultiValue| Ok(()),
    );
    methods.add_method(
        "SetFlattensRenderLayers",
        |_, _this, _flatten: Option<bool>| Ok(()),
    );
    methods.add_method("SetClipsChildren", |_, this, clips: Option<bool>| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.clips_children = clips.unwrap_or(false);
        }
        Ok(())
    });
    methods.add_method("DoesClipChildren", |_, this, ()| {
        let state = this.state.borrow();
        let clips = state
            .widgets
            .get(this.id)
            .map(|f| f.clips_children)
            .unwrap_or(false);
        Ok(clips)
    });
    methods.add_method(
        "SetMotionScriptsWhileDisabled",
        |_, _this, _enabled: Option<bool>| Ok(()),
    );
    methods.add_method("GetMotionScriptsWhileDisabled", |_, _this, ()| Ok(false));
    methods.add_method("SetHitRectInsets", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
    methods.add_method("GetHitRectInsets", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64))
    });
}
