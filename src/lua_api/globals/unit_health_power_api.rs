//! Unit health, power, and heal/absorb API functions.

use super::unit_api::parse_party_index;
use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register UnitHealth, UnitHealthMax, UnitPower, UnitPowerMax, UnitPowerType,
/// UnitGetIncomingHeals, UnitGetTotalAbsorbs, UnitGetTotalHealAbsorbs.
pub fn register_health_power_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_health_functions(lua, state.clone())?;
    register_power_functions(lua, state.clone())?;
    register_power_type_function(lua, state)?;
    register_heal_absorb_stubs(lua)
}

/// Register UnitHealth, UnitHealthMax with party and target awareness.
fn register_health_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    let st = state.clone();
    globals.set(
        "UnitHealth",
        lua.create_function(move |_, unit: Value| {
            if let Value::String(s) = &unit {
                let u = s.to_string_lossy();
                if u == "target" {
                    let st = st.borrow();
                    return Ok(st.current_target.as_ref().map(|t| t.health).unwrap_or(0));
                }
                if let Some(idx) = parse_party_index(&u) {
                    let st = st.borrow();
                    if let Some(m) = st.party_members.get(idx) {
                        return Ok(m.health);
                    }
                }
            }
            Ok(100_000i32)
        })?,
    )?;
    globals.set(
        "UnitHealthMax",
        lua.create_function(move |_, unit: Value| {
            if let Value::String(s) = &unit {
                let u = s.to_string_lossy();
                if u == "target" {
                    let st = state.borrow();
                    return Ok(st.current_target.as_ref().map(|t| t.health_max).unwrap_or(0));
                }
                if let Some(idx) = parse_party_index(&u) {
                    let st = state.borrow();
                    if let Some(m) = st.party_members.get(idx) {
                        return Ok(m.health_max);
                    }
                }
            }
            Ok(100_000i32)
        })?,
    )?;
    Ok(())
}

/// Register UnitPower, UnitPowerMax with party and target awareness.
fn register_power_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    let st = state.clone();
    globals.set(
        "UnitPower",
        lua.create_function(move |_, args: mlua::MultiValue| {
            if let Some(Value::String(s)) = args.into_vec().first() {
                let u = s.to_string_lossy();
                if u == "target" {
                    let st = st.borrow();
                    return Ok(st.current_target.as_ref().map(|t| t.power).unwrap_or(0));
                }
                if let Some(idx) = parse_party_index(&u) {
                    let st = st.borrow();
                    if let Some(m) = st.party_members.get(idx) {
                        return Ok(m.power);
                    }
                }
            }
            Ok(50_000i32)
        })?,
    )?;
    globals.set(
        "UnitPowerMax",
        lua.create_function(move |_, args: mlua::MultiValue| {
            if let Some(Value::String(s)) = args.into_vec().first() {
                let u = s.to_string_lossy();
                if u == "target" {
                    let st = state.borrow();
                    return Ok(st.current_target.as_ref().map(|t| t.power_max).unwrap_or(0));
                }
                if let Some(idx) = parse_party_index(&u) {
                    let st = state.borrow();
                    if let Some(m) = st.party_members.get(idx) {
                        return Ok(m.power_max);
                    }
                }
            }
            Ok(100_000i32)
        })?,
    )?;
    Ok(())
}

/// Register UnitPowerType with party and target awareness.
fn register_power_type_function(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    lua.globals().set(
        "UnitPowerType",
        lua.create_function(move |lua, unit: Value| {
            if let Value::String(s) = &unit {
                let u = s.to_string_lossy();
                if u == "target" {
                    let st = state.borrow();
                    if let Some(t) = &st.current_target {
                        return Ok((t.power_type, Value::String(lua.create_string(t.power_type_name)?)));
                    }
                }
                if let Some(idx) = parse_party_index(&u) {
                    let st = state.borrow();
                    if let Some(m) = st.party_members.get(idx) {
                        return Ok((m.power_type, Value::String(lua.create_string(m.power_type_name)?)));
                    }
                }
            }
            Ok((0i32, Value::String(lua.create_string("MANA")?)))
        })?,
    )
}

/// Register heal/absorb stubs (not party-dependent).
fn register_heal_absorb_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set(
        "UnitGetIncomingHeals",
        lua.create_function(|_, (_unit, _healer): (String, Option<String>)| Ok(0i32))?,
    )?;
    globals.set(
        "UnitGetTotalAbsorbs",
        lua.create_function(|_, _unit: Option<String>| Ok(0i32))?,
    )?;
    globals.set(
        "UnitGetTotalHealAbsorbs",
        lua.create_function(|_, _unit: Option<String>| Ok(0i32))?,
    )?;
    Ok(())
}
