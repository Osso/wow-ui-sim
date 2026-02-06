//! CVar and key binding WoW API functions.
//!
//! Provides access to configuration variables (CVars) and key binding management.

use super::super::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register CVar and key binding global functions.
pub fn register_cvar_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_cvar_functions(lua, &state)?;
    register_c_cvar_namespace(lua, &state)?;
    register_key_binding_functions(lua)?;
    Ok(())
}

/// C_CVar namespace — wraps the same CVar store as GetCVar/SetCVar.
fn register_c_cvar_namespace(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let t = lua.create_table()?;

    let s = Rc::clone(state);
    t.set("GetCVar", lua.create_function(move |lua, cvar: String| {
        let state = s.borrow();
        match state.cvars.get(&cvar) {
            Some(value) => Ok(Value::String(lua.create_string(&value)?)),
            None => Ok(Value::Nil),
        }
    })?)?;

    let s = Rc::clone(state);
    t.set("SetCVar", lua.create_function(move |_, (cvar, value): (String, Option<String>)| {
        let state = s.borrow();
        state.cvars.set(&cvar, value.as_deref().unwrap_or(""));
        Ok(true)
    })?)?;

    let s = Rc::clone(state);
    t.set("GetCVarBool", lua.create_function(move |_, cvar: String| {
        let state = s.borrow();
        Ok(state.cvars.get_bool(&cvar))
    })?)?;

    let s = Rc::clone(state);
    t.set("GetCVarDefault", lua.create_function(move |lua, cvar: String| {
        let state = s.borrow();
        match state.cvars.get(&cvar) {
            Some(value) => Ok(Value::String(lua.create_string(&value)?)),
            None => Ok(Value::Nil),
        }
    })?)?;

    t.set("GetCVarBitfield", lua.create_function(|_, (_name, _index): (String, Option<i32>)| {
        Ok(false)
    })?)?;

    t.set("SetCVarBitfield", lua.create_function(|_, (_name, _index, _value, _script): (String, i32, bool, Option<String>)| {
        Ok(true)
    })?)?;

    t.set("RegisterCVar", lua.create_function(|_, (_name, _value): (String, Option<String>)| {
        Ok(())
    })?)?;

    t.set("ResetTestCVars", lua.create_function(|_, ()| Ok(()))?)?;

    lua.globals().set("C_CVar", t)?;
    Ok(())
}

/// Register GetCVar and SetCVar functions.
fn register_cvar_functions(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    let state_for_getcvar = Rc::clone(state);
    let get_cvar = lua.create_function(move |lua, cvar: String| {
        let state = state_for_getcvar.borrow();
        match state.cvars.get(&cvar) {
            Some(value) => Ok(Value::String(lua.create_string(&value)?)),
            None => Ok(Value::Nil),
        }
    })?;
    globals.set("GetCVar", get_cvar)?;

    let state_for_setcvar = Rc::clone(state);
    let set_cvar = lua.create_function(move |_, (cvar, value): (String, String)| {
        let state = state_for_setcvar.borrow();
        Ok(state.cvars.set(&cvar, &value))
    })?;
    globals.set("SetCVar", set_cvar)?;

    Ok(())
}

/// Register key binding query and mutation functions.
fn register_key_binding_functions(lua: &Lua) -> Result<()> {
    register_binding_queries(lua)?;
    register_binding_mutations(lua)?;
    Ok(())
}

/// Binding query functions: GetBindingKey, GetBinding, GetNumBindings, etc.
fn register_binding_queries(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "GetBindingKey",
        lua.create_function(|_, _action: String| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetBindingKeyForAction",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetBinding",
        lua.create_function(|_lua, index: i32| {
            if index < 1 {
                return Ok(mlua::MultiValue::new());
            }
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Nil,
                Value::Nil,
                Value::Nil,
            ]))
        })?,
    )?;
    globals.set(
        "GetNumBindings",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    globals.set(
        "GetCurrentBindingSet",
        lua.create_function(|_, ()| Ok(1))?,
    )?;
    globals.set(
        "GetBindingAction",
        lua.create_function(|_, (_key, _check_override): (String, Option<bool>)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetBindingText",
        lua.create_function(
            |lua, (key, _prefix, _abbrev): (Option<String>, Option<String>, Option<bool>)| {
                match key {
                    Some(k) => Ok(Value::String(lua.create_string(&k)?)),
                    None => Ok(Value::String(lua.create_string("")?)),
                }
            },
        )?,
    )?;

    Ok(())
}

/// Binding mutation functions: SetBinding, SetBindingClick, SaveBindings, etc.
fn register_binding_mutations(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "SetBinding",
        lua.create_function(|_, (_key, _action): (String, Option<String>)| Ok(true))?,
    )?;
    globals.set(
        "SetBindingClick",
        lua.create_function(
            |_, (_key, _button, _mouse_button): (String, String, Option<String>)| Ok(true),
        )?,
    )?;
    globals.set(
        "SetBindingSpell",
        lua.create_function(|_, (_key, _spell): (String, String)| Ok(true))?,
    )?;
    globals.set(
        "SetBindingItem",
        lua.create_function(|_, (_key, _item): (String, String)| Ok(true))?,
    )?;
    globals.set(
        "SetBindingMacro",
        lua.create_function(|_, (_key, _macro): (String, String)| Ok(true))?,
    )?;
    globals.set(
        "SaveBindings",
        lua.create_function(|_, _which: i32| Ok(()))?,
    )?;
    globals.set(
        "LoadBindings",
        lua.create_function(|_, _which: i32| Ok(()))?,
    )?;

    Ok(())
}
