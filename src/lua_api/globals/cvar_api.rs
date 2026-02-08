//! CVar WoW API functions.
//!
//! Provides access to configuration variables (CVars).

use super::super::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register CVar global functions.
pub fn register_cvar_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_cvar_functions(lua, &state)?;
    register_c_cvar_namespace(lua, &state)?;
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

