//! Attribute methods: GetAttribute, SetAttribute, frame references, etc.

use super::FrameHandle;
use crate::widget::AttributeValue;
use mlua::{UserDataMethods, Value};

/// Add attribute methods to FrameHandle UserData.
pub fn add_attribute_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_get_set_attribute_methods(methods);
    add_execute_attribute(methods);
    add_frame_ref_methods(methods);
    add_security_and_input_stubs(methods);
}

/// GetAttribute, SetAttribute, ClearAttributes - core attribute CRUD.
fn add_get_set_attribute_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // GetAttribute supports WoW's multi-argument form: GetAttribute(prefix, name, suffix)
    // which concatenates to look up prefix..name..suffix. Also supports wildcard `*`
    // prefix fallback: if "prefix..name..suffix" not found, tries "*name..suffix".
    // This is required by SecureTemplates.lua (SecureButton_GetModifiedAttribute).
    methods.add_method("GetAttribute", |lua, this, args: mlua::MultiValue| {
        let keys = build_attribute_keys(&args);
        get_attribute_value(lua, this, &keys)
    });

    methods.add_method("SetAttribute", |lua, this, (name, value): (String, Value)| {
        set_attribute_value(lua, this, &name, &value)?;
        fire_on_attribute_changed(lua, this, &name, value)?;
        Ok(())
    });

    methods.add_method(
        "SetAttributeNoHandler",
        |lua, this, (name, value): (String, Value)| {
            set_attribute_value(lua, this, &name, &value)?;
            Ok(())
        },
    );

    methods.add_method("ClearAttributes", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_silent(this.id) {
            frame.attributes.clear();
        }
        Ok(())
    });
}

/// Build the list of attribute keys to try, in WoW's fallback order.
///
/// WoW's GetAttribute accepts 1 or 3 string arguments:
/// - 1 arg: `GetAttribute("name")` → tries just `["name"]`
/// - 3 args: `GetAttribute(prefix, name, suffix)` → tries in order:
///   1. `prefix..name..suffix` (exact)
///   2. `"*"..name..suffix`   (wildcard prefix)
///   3. `prefix..name.."*"`   (wildcard suffix)
///   4. `"*"..name.."*"`      (wildcard both)
///   5. `name`                (bare name, no prefix/suffix)
///
/// Ref: wowless data/uiobjects/Frame/GetAttribute.lua
fn build_attribute_keys(args: &mlua::MultiValue) -> Vec<String> {
    let strings: Vec<String> = args
        .iter()
        .filter_map(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
            _ => None,
        })
        .collect();

    match strings.len() {
        0 => vec![String::new()],
        1 => vec![strings[0].clone()],
        _ => {
            // 3-arg form: prefix, name, suffix
            let prefix = &strings[0];
            let name = &strings[1];
            let suffix = if strings.len() > 2 { strings[2].as_str() } else { "" };
            vec![
                format!("{}{}{}", prefix, name, suffix),
                format!("*{}{}", name, suffix),
                format!("{}{}*", prefix, name),
                format!("*{}*", name),
                name.clone(),
            ]
        }
    }
}

/// Look up an attribute, trying each key in order until one is found.
fn get_attribute_value(
    lua: &mlua::Lua,
    this: &FrameHandle,
    keys: &[String],
) -> mlua::Result<Value> {
    let table_attrs: Option<mlua::Table> =
        lua.globals().get("__frame_table_attributes").ok();
    let state = this.state.borrow();
    let frame = state.widgets.get(this.id);

    for key in keys {
        // Check table attributes stored in Lua
        if let Some(attrs) = &table_attrs {
            let lua_key = format!("{}_{}", this.id, key);
            let table_val: Value = attrs.get(lua_key.as_str()).unwrap_or(Value::Nil);
            if !matches!(table_val, Value::Nil) {
                return Ok(table_val);
            }
        }
        // Check non-table attributes stored in Rust
        if let Some(f) = frame {
            if let Some(attr) = f.attributes.get(key.as_str()) {
                return attribute_to_value(lua, attr);
            }
        }
    }
    Ok(Value::Nil)
}

/// Convert an AttributeValue to a Lua Value.
fn attribute_to_value(lua: &mlua::Lua, attr: &AttributeValue) -> mlua::Result<Value> {
    match attr {
        AttributeValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        AttributeValue::Number(n) => Ok(Value::Number(*n)),
        AttributeValue::Boolean(b) => Ok(Value::Boolean(*b)),
        AttributeValue::Nil => Ok(Value::Nil),
    }
}

/// Store the attribute value in Lua (tables) or Rust (simple types).
fn set_attribute_value(
    lua: &mlua::Lua,
    this: &FrameHandle,
    name: &str,
    value: &Value,
) -> mlua::Result<()> {
    if matches!(value, Value::Table(_) | Value::UserData(_) | Value::Function(_)) {
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
        if let Some(frame) = state.widgets.get_mut_silent(this.id) {
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
    use crate::lua_api::script_helpers::{call_error_handler, get_script};

    if let Some(handler) = get_script(lua, this.id, "OnAttributeChanged") {
        let frame_ref_key = format!("__frame_{}", this.id);
        let frame_ud: Value = lua
            .globals()
            .get(frame_ref_key.as_str())
            .unwrap_or(Value::Nil);
        let name_str = lua.create_string(name)?;
        if let Err(e) = handler.call::<()>((frame_ud, name_str, value)) {
            call_error_handler(lua, &e.to_string());
        }
    }
    Ok(())
}

/// ExecuteAttribute - look up attribute as function and call it. No-op in sim.
fn add_execute_attribute<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "ExecuteAttribute",
        |_, _this, _args: mlua::MultiValue| Ok(Value::Nil),
    );
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
    methods.add_method("SetHitRectInsets", |_, this, args: mlua::MultiValue| {
        let mut it = args.into_iter();
        let l = match it.next() { Some(Value::Number(n)) => n as f32, Some(Value::Integer(n)) => n as f32, _ => 0.0 };
        let r = match it.next() { Some(Value::Number(n)) => n as f32, Some(Value::Integer(n)) => n as f32, _ => 0.0 };
        let t = match it.next() { Some(Value::Number(n)) => n as f32, Some(Value::Integer(n)) => n as f32, _ => 0.0 };
        let b = match it.next() { Some(Value::Number(n)) => n as f32, Some(Value::Integer(n)) => n as f32, _ => 0.0 };
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.hit_rect_insets = (l, r, t, b);
        }
        Ok(())
    });
    methods.add_method("GetHitRectInsets", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            let (l, r, t, b) = frame.hit_rect_insets;
            return Ok((l as f64, r as f64, t as f64, b as f64));
        }
        Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64))
    });
}
