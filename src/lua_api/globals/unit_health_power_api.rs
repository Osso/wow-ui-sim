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
                if u == "player" {
                    return Ok(st.borrow().player_health);
                }
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
                if u == "player" {
                    return Ok(state.borrow().player_health_max);
                }
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
///
/// The optional second argument is the power type (Enum.PowerType).
/// When absent or matching the unit's primary power type (Mana=0), returns
/// the primary power values.  For secondary resource types (HolyPower,
/// ComboPoints, etc.) returns 0 / small-cap values.
fn register_power_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    let st = state.clone();
    globals.set(
        "UnitPower",
        lua.create_function(move |_, args: mlua::MultiValue| {
            let args = args.into_vec();
            let power_type = args.get(1).and_then(|v| match v {
                Value::Integer(i) => Some(*i),
                Value::Number(n) => Some(*n as i64),
                _ => None,
            });
            if let Some(Value::String(s)) = args.first() {
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
            // Secondary resource types return 0 current power.
            if is_secondary_power_type(power_type) {
                return Ok(0i32);
            }
            Ok(50_000i32)
        })?,
    )?;
    globals.set(
        "UnitPowerMax",
        lua.create_function(move |_, args: mlua::MultiValue| {
            let args = args.into_vec();
            let power_type = args.get(1).and_then(|v| match v {
                Value::Integer(i) => Some(*i),
                Value::Number(n) => Some(*n as i64),
                _ => None,
            });
            if let Some(Value::String(s)) = args.first() {
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
            // Secondary resource types have small caps (holy power, combo points, etc.)
            if is_secondary_power_type(power_type) {
                return Ok(secondary_power_max(power_type.unwrap_or(0)));
            }
            Ok(100_000i32)
        })?,
    )?;
    Ok(())
}

/// Returns true when the power type is a secondary resource (not the primary
/// mana pool).  None / 0 (Mana) is considered the primary type.
fn is_secondary_power_type(power_type: Option<i64>) -> bool {
    matches!(power_type, Some(pt) if pt != 0)
}

/// Default max for secondary power types.
fn secondary_power_max(power_type: i64) -> i32 {
    match power_type {
        4 => 7,   // ComboPoints (5-7 depending on talents)
        5 => 6,   // Runes
        9 => 5,   // HolyPower
        16 => 4,  // ArcaneCharges
        _ => 5,   // Reasonable default for other secondary resources
    }
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
