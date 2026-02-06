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
    register_key_binding_functions(lua)?;
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
    let globals = lua.globals();

    globals.set(
        "GetBindingKey",
        lua.create_function(|_, _action: String| Ok(Value::Nil))?,
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
        "GetCurrentBindingSet",
        lua.create_function(|_, ()| Ok(1))?,
    )?;
    globals.set(
        "SaveBindings",
        lua.create_function(|_, _which: i32| Ok(()))?,
    )?;
    globals.set(
        "LoadBindings",
        lua.create_function(|_, _which: i32| Ok(()))?,
    )?;
    globals.set(
        "GetBindingAction",
        lua.create_function(|_, (_key, _check_override): (String, Option<bool>)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetBindingText",
        lua.create_function(
            |lua, (key, _prefix, _abbrev): (String, Option<String>, Option<bool>)| {
                Ok(Value::String(lua.create_string(&key)?))
            },
        )?,
    )?;

    Ok(())
}
