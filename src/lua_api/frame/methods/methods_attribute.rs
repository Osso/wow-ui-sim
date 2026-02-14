//! Attribute methods: GetAttribute, SetAttribute, frame references, etc.

use crate::lua_api::frame::handle::{frame_lud, get_sim_state, lud_to_id};
use crate::widget::AttributeValue;
use mlua::{LightUserData, Lua, Value};

/// Add attribute methods to the shared methods table.
pub fn add_attribute_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_get_set_attribute_methods(lua, methods)?;
    add_execute_attribute(lua, methods)?;
    add_frame_ref_methods(lua, methods)?;
    add_security_and_input_stubs(lua, methods)?;
    Ok(())
}

/// GetAttribute, SetAttribute, ClearAttributes - core attribute CRUD.
fn add_get_set_attribute_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // GetAttribute supports WoW's multi-argument form: GetAttribute(prefix, name, suffix)
    // which concatenates to look up prefix..name..suffix. Also supports wildcard `*`
    // prefix fallback: if "prefix..name..suffix" not found, tries "*name..suffix".
    // This is required by SecureTemplates.lua (SecureButton_GetModifiedAttribute).
    methods.set(
        "GetAttribute",
        lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
            let id = lud_to_id(ud);
            let keys = build_attribute_keys(&args);
            get_attribute_value(lua, id, &keys)
        })?,
    )?;

    methods.set(
        "SetAttribute",
        lua.create_function(|lua, (ud, name, value): (LightUserData, String, Value)| {
            let id = lud_to_id(ud);
            set_attribute_value(lua, id, &name, &value)?;
            fire_on_attribute_changed(lua, id, &name, value)?;
            Ok(())
        })?,
    )?;

    methods.set(
        "SetAttributeNoHandler",
        lua.create_function(|lua, (ud, name, value): (LightUserData, String, Value)| {
            let id = lud_to_id(ud);
            set_attribute_value(lua, id, &name, &value)?;
            Ok(())
        })?,
    )?;

    methods.set(
        "ClearAttributes",
        lua.create_function(|lua, ud: LightUserData| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(id) {
                frame.attributes.clear();
            }
            Ok(())
        })?,
    )?;

    Ok(())
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
fn get_attribute_value(lua: &Lua, id: u64, keys: &[String]) -> mlua::Result<Value> {
    let table_attrs: Option<mlua::Table> =
        lua.globals().get("__frame_table_attributes").ok();
    let state_rc = get_sim_state(lua);
    let state = state_rc.borrow();
    let frame = state.widgets.get(id);

    for key in keys {
        // Check table attributes stored in Lua
        if let Some(attrs) = &table_attrs {
            let lua_key = format!("{}_{}", id, key);
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
fn attribute_to_value(lua: &Lua, attr: &AttributeValue) -> mlua::Result<Value> {
    match attr {
        AttributeValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        AttributeValue::Number(n) => Ok(Value::Number(*n)),
        AttributeValue::Boolean(b) => Ok(Value::Boolean(*b)),
        AttributeValue::Nil => Ok(Value::Nil),
    }
}

/// Store the attribute value in Lua (tables) or Rust (simple types).
fn set_attribute_value(lua: &Lua, id: u64, name: &str, value: &Value) -> mlua::Result<()> {
    if matches!(value, Value::Table(_) | Value::UserData(_) | Value::LightUserData(_) | Value::Function(_)) {
        store_table_attribute(lua, id, name, value)?;
    } else {
        store_simple_attribute(lua, id, name, value)?;
    }
    Ok(())
}

/// Store a complex Lua value (table/userdata/function) in the Lua-side attribute table.
fn store_table_attribute(lua: &Lua, id: u64, name: &str, value: &Value) -> mlua::Result<()> {
    let table_attrs: mlua::Table = lua
        .globals()
        .get("__frame_table_attributes")
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_table_attributes", t.clone()).ok();
            t
        });
    let key = format!("{}_{}", id, name);
    table_attrs.set(key, value.clone())?;
    Ok(())
}

/// Store a simple value (string/number/bool/nil) in the Rust-side attribute map.
fn store_simple_attribute(lua: &Lua, id: u64, name: &str, value: &Value) -> mlua::Result<()> {
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    if let Some(frame) = state.widgets.get_mut(id) {
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
                let key = format!("{}_{}", id, name);
                table_attrs.set(key, Value::Nil).ok();
            }
        } else {
            frame.attributes.insert(name.to_string(), attr);
        }
    }
    Ok(())
}

/// Fire OnAttributeChanged script handler if one exists.
fn fire_on_attribute_changed(lua: &Lua, id: u64, name: &str, value: Value) -> mlua::Result<()> {
    use crate::lua_api::script_helpers::{call_error_handler, get_script};

    if let Some(handler) = get_script(lua, id, "OnAttributeChanged") {
        let name_str = lua.create_string(name)?;
        if let Err(e) = handler.call::<()>((frame_lud(id), name_str, value)) {
            call_error_handler(lua, &e.to_string());
        }
    }
    Ok(())
}

/// ExecuteAttribute - look up attribute as function and call it. No-op in sim.
fn add_execute_attribute(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "ExecuteAttribute",
        lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| {
            Ok(Value::Nil)
        })?,
    )?;
    Ok(())
}

/// SetFrameRef/GetFrameRef - secure frame reference stubs.
fn add_frame_ref_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "SetFrameRef",
        lua.create_function(|_, (_ud, _label, _frame): (LightUserData, String, Value)| Ok(()))?,
    )?;

    methods.set(
        "GetFrameRef",
        lua.create_function(|_, (_ud, _label): (LightUserData, String)| Ok(Value::Nil))?,
    )?;

    Ok(())
}

/// Security, input, and rendering stubs (no-op in simulation).
fn add_security_and_input_stubs(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_security_stubs(lua, methods)?;
    add_clip_children_methods(lua, methods)?;
    add_hit_rect_methods(lua, methods)?;
    Ok(())
}

/// Simple no-op security and input stubs.
fn add_security_stubs(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "SetForbidden",
        lua.create_function(|_, (_ud, _forbidden): (LightUserData, Option<bool>)| Ok(()))?,
    )?;
    methods.set(
        "IsForbidden",
        lua.create_function(|_, _ud: LightUserData| Ok(false))?,
    )?;
    methods.set(
        "CanChangeProtectedState",
        lua.create_function(|_, _ud: LightUserData| Ok(true))?,
    )?;
    methods.set(
        "SetPassThroughButtons",
        lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?,
    )?;
    methods.set(
        "SetFlattensRenderLayers",
        lua.create_function(|_, (_ud, _flatten): (LightUserData, Option<bool>)| Ok(()))?,
    )?;
    methods.set(
        "SetMotionScriptsWhileDisabled",
        lua.create_function(|_, (_ud, _enabled): (LightUserData, Option<bool>)| Ok(()))?,
    )?;
    methods.set(
        "GetMotionScriptsWhileDisabled",
        lua.create_function(|_, _ud: LightUserData| Ok(false))?,
    )?;
    Ok(())
}

/// SetClipsChildren / DoesClipChildren methods.
fn add_clip_children_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "SetClipsChildren",
        lua.create_function(|lua, (ud, clips): (LightUserData, Option<bool>)| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(id) {
                frame.clips_children = clips.unwrap_or(false);
            }
            Ok(())
        })?,
    )?;

    methods.set(
        "DoesClipChildren",
        lua.create_function(|lua, ud: LightUserData| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let state = state_rc.borrow();
            let clips = state
                .widgets
                .get(id)
                .map(|f| f.clips_children)
                .unwrap_or(false);
            Ok(clips)
        })?,
    )?;

    Ok(())
}

/// SetHitRectInsets / GetHitRectInsets methods.
fn add_hit_rect_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set(
        "SetHitRectInsets",
        lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
            let id = lud_to_id(ud);
            let (l, r, t, b) = parse_hit_rect_insets(args);
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(id) {
                frame.hit_rect_insets = (l, r, t, b);
            }
            Ok(())
        })?,
    )?;

    methods.set(
        "GetHitRectInsets",
        lua.create_function(|lua, ud: LightUserData| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let state = state_rc.borrow();
            if let Some(frame) = state.widgets.get(id) {
                let (l, r, t, b) = frame.hit_rect_insets;
                return Ok((l as f64, r as f64, t as f64, b as f64));
            }
            Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64))
        })?,
    )?;

    Ok(())
}

/// Parse 4 numeric values from a MultiValue for hit rect insets.
fn parse_hit_rect_insets(args: mlua::MultiValue) -> (f32, f32, f32, f32) {
    let mut it = args.into_iter();
    // Skip the first value (LightUserData self) - already consumed by tuple destructure
    let l = match it.next() {
        Some(Value::Number(n)) => n as f32,
        Some(Value::Integer(n)) => n as f32,
        _ => 0.0,
    };
    let r = match it.next() {
        Some(Value::Number(n)) => n as f32,
        Some(Value::Integer(n)) => n as f32,
        _ => 0.0,
    };
    let t = match it.next() {
        Some(Value::Number(n)) => n as f32,
        Some(Value::Integer(n)) => n as f32,
        _ => 0.0,
    };
    let b = match it.next() {
        Some(Value::Number(n)) => n as f32,
        Some(Value::Integer(n)) => n as f32,
        _ => 0.0,
    };
    (l, r, t, b)
}
